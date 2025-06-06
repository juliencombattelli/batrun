use crate::test_driver::TestDriver;
use crate::test_executor::Executor;

pub struct SequentialExecutor;

impl Executor for SequentialExecutor {
    fn execute(&self, _: &Box<(dyn TestDriver + 'static)>) {
        todo!()
    }
}
