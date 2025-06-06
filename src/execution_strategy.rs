#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ExecutionStrategy {
    /// Run all test cases sequentially for a target before passing to the next target
    Sequential,
    /// Run each test case for all targets before passing to the next test case
    RoundRobin,
    /// Run all test cases for each targets in parallel
    Parallel,
}
