/// Aliases are currently not showed in help message.
/// Wait for https://github.com/clap-rs/clap/pull/5480 to be merged to make aliases visible in help message.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ExecutionStrategy {
    /// Run all test cases sequentially for a target before passing to the next target
    #[clap(aliases = &["s", "seq"])]
    Sequential,
    /// Run each test case for all targets before passing to the next test case
    #[clap(aliases = &["r", "rr"])]
    RoundRobin,
    /// Run all test cases for each targets in parallel
    #[clap(aliases = &["p", "par"])]
    Parallel,
}
