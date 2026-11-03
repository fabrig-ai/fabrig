//! A red tier run told to a goal (temper/docs/design/domain/after-landing.md, 7).

use alloc::boxed::Box;
use skein_forge::api::Commit;
use temper_engine_domain_forge_tiers::Failures;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RedNews {
    pub tier: Box<[u8]>,
    /// The run's number in the tier's sequence.
    pub run: u64,
    /// The forge run's link, once observed.
    pub url: Option<Box<[u8]>>,
    pub on: RedOn,
    /// Each failed job of the run's last failed part, in the run's order.
    pub logs: Box<[JobLog]>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RedOn {
    /// A red after landing: the landing, its branch, the failures blamed on it.
    Landing { commit: Commit, branch: Box<[u8]>, failing: Failures },
    /// A red of a tier following goal branches: the branch, the run's tip, its tests new and still failing.
    Goal { branch: Box<[u8]>, tip: Commit, new: Failures, still: Failures },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct JobLog {
    pub name: Box<[u8]>,
    /// The log's end, cut to its share; `None` when it could not be read.
    pub end: Option<Box<[u8]>>,
}
