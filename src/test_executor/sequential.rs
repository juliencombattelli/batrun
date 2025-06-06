use crate::test_driver::TestDriver;
use crate::test_executor::utils::simple_executor;
use crate::test_executor::{ExecutionContext, Executor};

use std::pin::Pin;
pub struct SequentialExecutor;

impl Executor for SequentialExecutor {
    fn execute(
        &self,
        execution_contexts: &[ExecutionContext],
        test_driver: &Box<(dyn TestDriver + 'static)>,
    ) {
        for exec_context in execution_contexts {
            let task = async || {
                let target = exec_context.target.clone();
                let test_suite_dir = exec_context.test_suite.path();
                exec_context
                    .test_suite
                    .visit2(async |test_case, should_skip| {
                        test_driver.run_test(test_suite_dir, &target, test_case);
                        simple_executor::wait_until_next_poll().await;
                        crate::test_suite::TestSuiteVisitResult::Ok
                    })
                    .await;
            };
            simple_executor::execute_many(vec![Box::pin(task())]);
        }
    }
}
