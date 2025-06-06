use crate::test_driver::TestDriver;
use crate::test_executor::Executor;

pub struct ParallelExecutor;

impl Executor for ParallelExecutor {
    fn execute(&self, _: &Box<(dyn TestDriver + 'static)>) {
        todo!()
    }
}
