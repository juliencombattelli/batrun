use crate::test_suite::status::SkipReason;
use crate::test_suite::{TestCase, TestFile, TestSuite};

/// The trait used when the visitor visites a test case
pub trait VisitorFnMut<E>: FnMut(&TestCase, ShouldSkip) -> Result<(), E> {}
impl<E, T: FnMut(&TestCase, ShouldSkip) -> Result<(), E>> VisitorFnMut<E> for T {}

/// The trait used when the visitor visites a test case for operation that cannot fail
pub trait VisitorFnMutOk: FnMut(&TestCase, ShouldSkip) {}
impl<T: FnMut(&TestCase, ShouldSkip)> VisitorFnMutOk for T {}

/// The visitor type allowing to visit the test cases in a test suite
/// It implements internally a state machine described below
pub struct Visitor<'ts> {
    test_suite: &'ts TestSuite,
    state: State,
    test_file_iter: std::iter::Peekable<std::slice::Iter<'ts, TestFile>>,
    test_case_iter: std::slice::Iter<'ts, TestCase>,
    should_skip: ShouldSkip,
}

impl<'ts> Visitor<'ts> {
    pub fn new(test_suite: &'ts TestSuite) -> Self {
        Self {
            test_suite,
            state: State::TestSuiteSetup,
            test_file_iter: std::slice::Iter::default().peekable(),
            test_case_iter: std::slice::Iter::default(),
            should_skip: ShouldSkip::No,
        }
    }

    pub fn visit_next<E>(&mut self, f: impl VisitorFnMut<E>) -> (bool, Result<(), E>) {
        let mut done = false;
        let (next_state, result) = match self.state {
            State::TestSuiteSetup => self.visit_test_suite_setup(f),
            State::TestCaseSetup => self.visit_test_case_setup(f),
            State::TestCase => self.visit_test_case(f),
            State::TestCaseTeardown => self.visit_test_case_teardown(f),
            State::TestSuiteTeardown => self.visit_test_suite_teardown(f),
            // Treat all other states as a state machine termination point
            termination_state => {
                done = true;
                (termination_state, Ok(()))
            }
        };
        self.state = next_state;
        (done, result)
    }

    pub fn visit_next_ok(&mut self, mut f: impl VisitorFnMutOk) -> bool {
        let mut f_wrapped = |test_case: &TestCase, should_skip: ShouldSkip| -> Result<(), ()> {
            f(&test_case, should_skip);
            Ok(())
        };
        let (is_done, _result) = self.visit_next(&mut f_wrapped);
        is_done
    }

    pub fn visit_all<E>(&mut self, mut f: impl VisitorFnMut<E>) {
        loop {
            let (is_done, _result) = self.visit_next(&mut f);
            if is_done {
                break;
            }
        }
    }

    pub fn visit_all_ok(&mut self, mut f: impl VisitorFnMutOk) {
        let mut f_wrapped = |test_case: &TestCase, should_skip: ShouldSkip| -> Result<(), ()> {
            f(&test_case, should_skip);
            Ok(())
        };
        loop {
            let (is_done, _result) = self.visit_next(&mut f_wrapped);
            if is_done {
                break;
            }
        }
    }

    fn visit_test_suite_setup<E>(&mut self, mut f: impl VisitorFnMut<E>) -> (State, Result<(), E>) {
        let mut result = Ok(());
        if let Some(tc) = &self.test_suite.fixture.setup_test_case {
            if let Err(e) = f(tc, self.should_skip.clone()) {
                self.should_skip
                    .skip_with_reason(SkipReason::TestSuiteSetupError);
                result = Err(e);
            }
        }
        self.test_file_iter = self.test_suite.test_files.iter().peekable();
        (State::TestCaseSetup, result)
    }

    fn visit_test_case_setup<E>(&mut self, mut f: impl VisitorFnMut<E>) -> (State, Result<(), E>) {
        // Reset the should_skip status if the stored advise was to skip the test cases from the
        // previous test file due to setup failure
        if let ShouldSkip::Yes(SkipReason::TestCaseSetupError) = self.should_skip {
            self.should_skip = ShouldSkip::No;
        }
        let mut result = Ok(());
        if let Some(test_file) = self.test_file_iter.peek() {
            self.test_case_iter = test_file.test_cases.iter();
            if let Some(tc) = &test_file.setup_test_case {
                if let Err(e) = f(tc, self.should_skip.clone()) {
                    self.should_skip
                        .skip_with_reason(SkipReason::TestCaseSetupError);
                    result = Err(e);
                }
            }
        }
        (State::TestCase, result)
    }

    fn visit_test_case<E>(&mut self, mut f: impl VisitorFnMut<E>) -> (State, Result<(), E>) {
        if let Some(test_case) = self.test_case_iter.next() {
            let result = f(test_case, self.should_skip.clone());
            (State::TestCase, result)
        } else {
            (State::TestCaseTeardown, Ok(()))
        }
    }

    fn visit_test_case_teardown<E>(
        &mut self,
        mut f: impl VisitorFnMut<E>,
    ) -> (State, Result<(), E>) {
        if let Some(test_file) = self.test_file_iter.next() {
            let mut result = Ok(());
            if let Some(tc) = &test_file.teardown_test_case {
                result = f(tc, self.should_skip.clone());
            }
            (State::TestCaseSetup, result)
        } else {
            (State::TestSuiteTeardown, Ok(()))
        }
    }

    fn visit_test_suite_teardown<E>(
        &mut self,
        mut f: impl VisitorFnMut<E>,
    ) -> (State, Result<(), E>) {
        let mut result = Ok(());
        if let Some(tc) = &self.test_suite.fixture.teardown_test_case {
            result = f(tc, self.should_skip.clone());
        }
        (State::Done, result)
    }
}

/// The state of the state machine
///
/// ┌────────────────┐    ┌───────────────┐    ┌──────────┐    ┌──────────────────┐    ┌───────────────────┐    ┌──────┐    ///
/// │                │    │               │    │          │    │                  │    │                   │    │      │    ///
/// │ TestSuiteSetup │───>│ TestCaseSetup │───>│ TestCase │───>│ TestCaseTeardown │───>│ TestSuiteTeardown │───>│ Done │    ///
/// │                │    │               │    │          │    │                  │    │                   │    │      │    ///
/// └────────────────┘    └───────────────┘    └──────────┘    └──────────────────┘    └───────────────────┘    └──────┘    ///
///                               ^              ^      │                 │                                                 ///
///                               │              └──────┘                 │                     ┌───────┐    ┌─────────┐    ///
///                               └───────────────────────────────────────┘                     │       │    │         │    ///
///                                                                                             │ <All> │───>│ Aborted │    ///
///                                                                                             │       │    │         │    ///
///                                                                                             └───────┘    └─────────┘    ///
#[derive(Clone, Copy)]
pub enum State {
    TestSuiteSetup,
    TestCaseSetup,
    TestCase,
    TestCaseTeardown,
    TestSuiteTeardown,
    Done,
    Aborted,
}

/// A boolean indicating if the current test case should be skipped
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShouldSkip {
    No,
    Yes(SkipReason),
}
impl ShouldSkip {
    fn skip_with_reason(&mut self, reason: SkipReason) {
        *self = match self {
            ShouldSkip::No => ShouldSkip::Yes(reason),
            // SkipReason variants are sorted from the least priority to the highest one
            ShouldSkip::Yes(self_reason) => {
                ShouldSkip::Yes(std::cmp::max(self_reason.clone(), reason))
            }
        };
    }
}
