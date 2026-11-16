//! Temper's jobs at open (jig/docs/design/protocol/store/jobs.md, 3).
use alloc::boxed::Box;
use jig_protocol_store::{JobAnswer, JobPage, JobRefusal};
use temper_engine_domain_forge_vocabulary::Stored;

mod deleted;
pub use deleted::{Present, read_present};

/// Dispatch an application's job at open.
pub fn page(job: &str, page: &JobPage, present: &Present, bounds: &crate::Bounds) -> Result<JobAnswer, JobRefusal> {
    match job {
        "round-branches" => round_branches(page, bounds),
        "deleted-repository-forge-rows" => deleted::page(page, present, bounds),
        _ => Err(refusal(None, "no such temper job")),
    }
}

fn refusal(key: Option<Box<[u8]>>, why: &str) -> JobRefusal {
    JobRefusal { key, why: Box::from(why) }
}

fn round_branches(page: &JobPage, bounds: &crate::Bounds) -> Result<JobAnswer, JobRefusal> {
    let mut empty = 0_u64;
    let mut named = 0_u64;
    for row in &page.rows {
        let Ok(Stored::Landing(landing)) = crate::records::row(&row.key, &row.value, bounds) else {
            return Err(refusal(Some(row.key.clone()), "unreadable landing row"));
        };
        if landing.landing.round.branch.is_empty() {
            empty = empty.checked_add(1).expect("page row count fits");
        } else {
            named = named.checked_add(1).expect("page row count fits");
        }
    }
    Ok(JobAnswer { writes: Box::new([]), through: None, counts: Box::new([empty, named]) })
}
