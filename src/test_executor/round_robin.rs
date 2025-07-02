use crate::reporter::Reporter;
use crate::test_driver::TestDriver;
use crate::test_executor::{ExecutionContext, Executor};
use crate::test_suite::TestSuite;
use crate::test_suite::visitor::Visitor;

use std::collections::VecDeque;

pub(crate) struct RoundRobinExecutor;

impl<'tr> Executor<'tr> for RoundRobinExecutor {
    fn execute(
        &self,
        reporter: &'tr Box<dyn Reporter>,
        test_driver: &'tr Box<(dyn TestDriver + 'static)>,
        test_suite: &'tr TestSuite,
        exec_contexts: &'tr mut [ExecutionContext],
    ) {
        struct VisitorContext<'a> {
            execution_context: &'a mut ExecutionContext,
            visitor: Visitor<'a>,
        }

        let mut visitor_contexts = exec_contexts
            .iter_mut()
            .map(|exec_ctx| VisitorContext::<'tr> {
                execution_context: exec_ctx,
                visitor: Visitor::new(&test_suite),
            })
            .collect::<VecDeque<_>>();
        let mut finished_contexts = VecDeque::new();

        while !visitor_contexts.is_empty() {
            let visitor_context = &mut visitor_contexts[0];
            let exec_context = &mut visitor_context.execution_context;
            let visitor = &mut visitor_context.visitor;
            let done = visitor.visit_next(|test_case, should_skip| {
                exec_context.run(reporter, test_driver, test_suite, test_case, should_skip)
            });
            if done {
                if let Some(context) = visitor_contexts.pop_front() {
                    finished_contexts.push_back(context.execution_context);
                }
            } else {
                visitor_contexts.rotate_left(1);
            }
        }
    }
}
