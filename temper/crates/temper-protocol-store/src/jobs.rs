//! Temper's jobs at open (jig/docs/design/protocol/store/jobs.md, 3).
use alloc::boxed::Box;
use jig_protocol_store::{JobAnswer, JobPage, JobRefusal};

/// No job is declared until its conversion is implemented here.
pub fn page(_job: &str, _page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    Err(JobRefusal { key: None, why: Box::from("no such temper job") })
}
