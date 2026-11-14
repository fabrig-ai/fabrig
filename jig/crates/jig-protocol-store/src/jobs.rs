//! Owner jobs at open (jig/docs/design/protocol/store/jobs.md, 1–3).
use alloc::boxed::Box;

#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub name: &'static str,
    pub keys: &'static [KeyRange],
    pub converts: Option<Converts>,
}
#[derive(Clone, Copy, Debug)]
pub struct Converts {
    pub family: &'static str,
    pub value_type: u16,
    pub from: [u8; 32],
}
/// Inclusive start and exclusive end; application keys are relative to its tag.
#[derive(Clone, Copy, Debug)]
pub struct KeyRange {
    pub from: &'static [u8],
    pub to: &'static [u8],
}
#[derive(Debug)]
pub struct JobPage {
    pub rows: Box<[JobRow]>,
    pub end: bool,
    pub layout: Option<JobLayout>,
}
#[derive(Debug)]
pub struct JobRow {
    pub key: Box<[u8]>,
    pub value: Box<[u8]>,
}
#[derive(Debug)]
pub struct JobLayout {
    pub root: Box<str>,
    pub variant: Option<Box<str>>,
    pub text: Box<[u8]>,
    pub held: Box<[Box<[u8]>]>,
}
#[derive(Debug)]
pub struct JobAnswer {
    pub writes: Box<[JobWrite]>,
    pub through: Option<Box<[u8]>>,
}
#[derive(Debug)]
pub enum JobWrite {
    Put { key: Box<[u8]>, value: Box<[u8]> },
    Erase { key: Box<[u8]> },
}
#[derive(Debug)]
pub struct JobRefusal {
    pub key: Option<Box<[u8]>>,
    pub why: Box<str>,
}

/// Jig declares no jobs until their owners add their dispatches.
pub fn page(_job: &str, _page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    Err(JobRefusal { key: None, why: Box::from("no such jig job") })
}

pub(crate) mod admission;
pub(crate) mod kept;
