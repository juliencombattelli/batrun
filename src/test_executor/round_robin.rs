use crate::error::Error;
use crate::test_driver::TestDriver;
use crate::test_executor::{ExecutionContext, Executor};
use crate::test_suite::visitor::Visitor;

use std::collections::VecDeque;

pub struct RoundRobinExecutor;

impl Executor for RoundRobinExecutor {
    fn execute(
        &self,
        execution_contexts: &[ExecutionContext],
        test_driver: &Box<(dyn TestDriver + 'static)>,
    ) {
        struct ExecutorContext<'ts> {
            execution_context: &'ts ExecutionContext<'ts>,
            visitor: Visitor<'ts>,
        }

        let mut contexts = execution_contexts
            .iter()
            .map(|exec_ctx| ExecutorContext {
                execution_context: &exec_ctx,
                visitor: Visitor::new(&exec_ctx.test_suite),
            })
            .collect::<VecDeque<_>>();

        while !contexts.is_empty() {
            let target = &contexts[0].execution_context.target;
            let test_suite_dir = contexts[0].execution_context.test_suite.path();
            let (done, _) =
                contexts[0]
                    .visitor
                    .visit_next(|test_case, should_skip| -> Result<(), Error> {
                        let _tc_state = test_driver.run_test(test_suite_dir, &target, test_case)?;
                        Ok(())
                    });
            if done {
                contexts.pop_front();
            } else {
                contexts.rotate_left(1);
            }
        }
    }
}
