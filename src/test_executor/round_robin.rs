use crate::error::Error;
use crate::test_driver::TestDriver;
use crate::test_executor::utils::simple_executor;
use crate::test_executor::{ExecutionContext, Executor};
use crate::test_suite::visitor::Visitor;

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
                let mut visitor = Visitor::new(&exec_context.test_suite);
                loop {
                    let (done, _) =
                        visitor.visit_next(|test_case, should_skip| -> Result<(), Error> {
                            let _tc_state =
                                test_driver.run_test(test_suite_dir, &target, test_case)?;
                            Ok(())
                        });
                    if done {
                        break;
                    }
                    simple_executor::wait_until_next_poll().await;
                }
            };

            per_target_tasks.push(Box::pin(task()));
        }
        simple_executor::block_on_many(per_target_tasks);
    }
}
