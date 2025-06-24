use crate::reporter::Reporter;
use crate::test_driver::TestDriver;
use crate::test_executor::{ExecutionContext, Executor};
use crate::test_suite::TestSuite;

pub struct ParallelExecutor;

impl<'tr> Executor<'tr> for ParallelExecutor {
    fn execute(
        &self,
        _reporter: &'tr Box<dyn Reporter>,
        _test_driver: &'tr Box<(dyn TestDriver + 'static)>,
        _test_suite: &'tr TestSuite,
        _execution_contexts: &mut [ExecutionContext],
    ) {
        todo!()
    }
}
