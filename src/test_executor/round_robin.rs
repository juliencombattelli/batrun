use crate::test_driver::TestDriver;
use crate::test_executor::Executor;

pub struct RoundRobinExecutor;

impl Executor for RoundRobinExecutor {
    fn execute(&self, _: &Box<(dyn TestDriver + 'static)>) {
        todo!()
    }
}
