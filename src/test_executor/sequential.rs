use crate::reporter::Reporter;
use crate::test_driver::TestDriver;
use crate::test_executor::{ExecutionContext, Executor};
use crate::test_suite::TestSuite;
use crate::test_suite::visitor::Visitor;

pub struct SequentialExecutor;

impl<'tr> Executor<'tr> for SequentialExecutor {
    fn execute(
        &self,
        reporter: &'tr Box<dyn Reporter>,
        test_driver: &'tr Box<(dyn TestDriver + 'static)>,
        test_suite: &'tr TestSuite,
        exec_contexts: &'tr mut [ExecutionContext],
    ) {
        for exec_context in exec_contexts {
            let mut visitor = Visitor::new(&test_suite);
            loop {
                // Ignore result as it is internally used by the visitor to know whether test cases should be skipped
                let (done, _result) = visitor.visit_next(|test_case, should_skip| {
                    exec_context.run(reporter, test_driver, test_suite, test_case, should_skip)
                });
                if done {
                    break;
                }
            }
        }
    }
}
