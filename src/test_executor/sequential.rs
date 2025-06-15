use crate::test_driver::TestDriver;
use crate::test_executor::{ExecutionContext, Executor};
use crate::test_suite::TestSuite;
use crate::test_suite::visitor::Visitor;

pub struct SequentialExecutor;

impl<'tr> Executor<'tr> for SequentialExecutor {
    fn execute(
        &self,
        test_driver: &'tr Box<(dyn TestDriver + 'static)>,
        test_suite: &'tr TestSuite,
        execution_contexts: &'tr mut [ExecutionContext],
    ) {
        for exec_context in execution_contexts {
            let mut visitor = Visitor::new(&test_suite);
            loop {
                let done = visitor.visit_next_ok(|test_case, should_skip| {
                    exec_context.run(test_driver, test_suite, test_case, should_skip);
                });
                if done {
                    break;
                }
            }
        }
    }
}
