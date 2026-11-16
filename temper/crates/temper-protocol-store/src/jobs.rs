//! Temper's jobs at open (jig/docs/design/protocol/store/jobs.md, 3).
use alloc::boxed::Box;
use jig_protocol_store::{JobAnswer, JobPage, JobRefusal};

mod deleted;
pub use deleted::{Present, read_present};

/// Dispatch an application's job at open.
pub fn page(job: &str, page: &JobPage, present: &Present, bounds: &crate::Bounds) -> Result<JobAnswer, JobRefusal> {
    match job {
        "deleted-repository-forge-rows" => deleted::page(page, present, bounds),
        _ => Err(refusal(None, "no such temper job")),
    }
}

fn refusal(key: Option<Box<[u8]>>, why: &str) -> JobRefusal {
    JobRefusal { key, why: Box::from(why) }
}
