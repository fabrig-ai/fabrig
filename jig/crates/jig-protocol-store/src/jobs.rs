//! Owner jobs at open (jig/docs/design/protocol/store/jobs.md, 1–3).
use alloc::boxed::Box;

#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub name: &'static str,
    pub keys: &'static [KeyRange],
    pub converts: Option<Converts>,
    pub counts: &'static [&'static str],
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
    pub room: JobRoom,
}
/// The answer's write and byte allowance, after the commit frame and job row.
#[derive(Clone, Copy, Debug)]
pub struct JobRoom {
    pub writes: u32,
    pub bytes: u64,
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
    pub counts: Box<[u64]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobCount {
    pub name: Box<str>,
    pub count: u64,
}
#[derive(Debug)]
pub enum JobWrite {
    Put { key: Box<[u8]>, value: Box<[u8]> },
    Erase { key: Box<[u8]> },
}
impl JobWrite {
    /// Encoded operation bytes. Application keys are relative to their tag;
    /// the runner reserves that tag separately in the page's room.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        let (key, value) = match self {
            Self::Put { key, value } => (key, Some(value)),
            Self::Erase { key } => (key, None),
        };
        let Ok(key) = u32::try_from(key.len()) else { return u64::MAX };
        let bytes = match value {
            Some(value) => {
                let Ok(value) = u32::try_from(value.len()) else { return u64::MAX };
                skein_kv::put_bytes(key, value)
            }
            None => skein_kv::erase_bytes(key),
        };
        match bytes {
            Some(bytes) => u64::from(bytes),
            None => u64::MAX,
        }
    }
}
#[derive(Debug)]
pub struct JobRefusal {
    pub key: Option<Box<[u8]>>,
    pub why: Box<str>,
}

/// Routes a page to its named jig job (protocol/store/jobs.md, 3–4).
pub fn page(job: &str, page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    if job == "charter-versions" {
        return charter_versions::page(page);
    }
    if job == "retention-rows" {
        return retention::rows(page);
    }
    if job == "retention-proposals" {
        return retention::proposals(page);
    }
    if job == "milestones" {
        return milestones::page(page);
    }
    if job == "live-task-parts" {
        return live_task_parts::page(page);
    }
    if job == "projection-parts" {
        return projection_parts::page(page);
    }
    Err(JobRefusal { key: None, why: Box::from("no such jig job") })
}

mod charter_versions;
mod live_task_parts;
mod milestones;
mod projection_parts;
mod retention;

pub(crate) mod admission;
pub(crate) mod kept;
