use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SuiteConfigIo(#[from] kind::SuiteConfigIo),

    #[error(transparent)]
    InvalidSuiteConfig(#[from] kind::InvalidSuiteConfig),

    #[error("unknown test driver `{0}`")]
    UnknownTestDriver(String),

    #[error("unknown test suite at `{0}`")]
    UnknownTestSuite(PathBuf),

    #[error(transparent)]
    TestDriverIo(#[from] kind::TestDriverIo),

    #[error(transparent)]
    TestFileExec(#[from] kind::TestFileExec),

    #[error("no test found in file `{0}`")]
    NoTestFound(PathBuf),

    #[error("multiple test function found with name `{0}`")]
    DuplicatedTestFn(String),

    #[error("unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;

pub mod kind {
    use super::*;

    #[derive(thiserror::Error, Debug)]
    #[error("cannot read the test suite config file `{}`", .filename.display())]
    pub struct SuiteConfigIo {
        pub filename: PathBuf,
        pub source: std::io::Error,
    }

    #[derive(thiserror::Error, Debug)]
    #[error("invalid test suite config file `{}`", .filename.display())]
    pub struct InvalidSuiteConfig {
        pub filename: PathBuf,
        pub source: serde_json::Error,
    }

    #[derive(thiserror::Error, Debug)]
    #[error("cannot execute test driver command `{}`", .filename.display())]
    pub struct TestDriverIo {
        pub filename: PathBuf,
        pub source: std::io::Error,
    }

    #[derive(thiserror::Error, Debug)]
    #[error("cannot execute test file `{}`", .filename.display())]
    pub struct TestFileExec {
        pub filename: PathBuf,
        pub details: String,
    }
}
