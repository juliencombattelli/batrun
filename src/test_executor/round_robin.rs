use crate::reporter::Reporter;
use crate::test_driver::TestDriver;
use crate::test_executor::{ExecutionContext, Executor};
use crate::test_suite::TestSuite;
use crate::test_suite::visitor::Visitor;

use std::collections::VecDeque;

pub struct RoundRobinExecutor;

impl<'tr> Executor<'tr> for RoundRobinExecutor {
    fn execute(
        &self,
        reporter: &'tr Box<dyn Reporter>,
        test_driver: &'tr Box<(dyn TestDriver + 'static)>,
        test_suite: &'tr TestSuite,
        execution_contexts: &'tr mut [ExecutionContext],
    ) {
        struct ExecutorContext<'a> {
            execution_context: &'a mut ExecutionContext,
            visitor: Visitor<'a>,
        }

        let mut contexts = execution_contexts
            .iter_mut()
            .map(|exec_ctx| ExecutorContext::<'tr> {
                execution_context: exec_ctx,
                visitor: Visitor::new(&test_suite),
            })
            .collect::<VecDeque<_>>();

        while !contexts.is_empty() {
            let context = &mut contexts[0];
            let exec_context = &mut context.execution_context;
            let visitor = &mut context.visitor;
            // Ignore result as it is internally used by the visitor to know whether test cases should be skipped
            let (done, _result) = visitor.visit_next(|test_case, should_skip| {
                exec_context.run(reporter, test_driver, test_suite, test_case, should_skip)
            });
            if done {
                contexts.pop_front();
            } else {
                contexts.rotate_left(1);
            }
        }
    }
}
