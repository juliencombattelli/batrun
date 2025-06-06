use crate::test_driver::TestDriver;
use crate::test_executor::utils::simple_executor;
use crate::test_executor::{ExecutionContext, Executor};

use std::pin::Pin;

pub struct RoundRobinExecutor;

impl Executor for RoundRobinExecutor {
    fn execute(
        &self,
        execution_contexts: &[ExecutionContext],
        test_driver: &Box<(dyn TestDriver + 'static)>,
    ) {
        let mut per_target_tasks: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();
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
            per_target_tasks.push(Box::pin(task()));
        }
        simple_executor::execute_many(per_target_tasks);
    }
}
