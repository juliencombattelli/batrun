use crate::test_executor::{ExecutionContext, Executor};

pub struct ParallelExecutor;

impl Executor for ParallelExecutor {
    fn execute(&self, _execution_contexts: &[ExecutionContext]) {
        todo!()
    }
}
