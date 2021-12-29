use crate::CandidType;
use crate::Deserialize;
use crate::TimeMillis;

pub trait Environment {
    //trait need self param to be pass as object
    fn now(&self) -> TimeMillis;
}

#[derive(CandidType, Deserialize)]
pub struct CanisterEnvironment {}

impl CanisterEnvironment {
    pub fn new() -> Self {
        CanisterEnvironment {}
    }
}

impl Environment for CanisterEnvironment {
    fn now(&self) -> TimeMillis {
        ic_cdk::api::time()
    }
}

pub struct TestEnvironment {
    pub now: u64,
}

impl Environment for TestEnvironment {
    fn now(&self) -> TimeMillis {
        self.now
    }
}

pub struct EmptyEnvironment {}

impl Environment for EmptyEnvironment {
    fn now(&self) -> TimeMillis {
        0
    }
}
