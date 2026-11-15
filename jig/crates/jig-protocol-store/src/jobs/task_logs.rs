//! Move map history through the commit path (protocol/store/logs.md, 6).
use super::{Job, JobAnswer, JobPage, JobRefusal, JobWrite, KeyRange};
use alloc::boxed::Box;
use skein_lib::List;

pub(crate) const JOB: Job = Job {
    name: "task-logs",
    keys: &[
        KeyRange { from: &[1, 0, 5], to: &[1, 0, 6] },
        KeyRange { from: &[1, 0, 6, 0], to: &[1, 0, 6, 1] },
        KeyRange { from: &[1, 0, 7], to: &[1, 0, 8] },
        KeyRange { from: &[1, 1, 6], to: &[1, 1, 7] },
        KeyRange { from: &[1, 1, 9], to: &[1, 1, 12] },
        KeyRange { from: &[1, 1, 22], to: &[1, 1, 24] },
    ],
    converts: None,
};

pub(super) fn page(page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    let mut writes = List::uncapped();
    let mut through = None;
    let mut bytes = 0_u64;
    for row in &page.rows {
        let write = JobWrite::Put { key: row.key.clone(), value: row.value.clone() };
        let next = bytes.checked_add(write.bytes());
        let fits = match next {
            Some(next) => next <= page.room.bytes,
            None => false,
        };
        if writes.len() >= page.room.writes / 4 || !fits {
            if through.is_none() {
                return Err(JobRefusal { key: Some(row.key.clone()), why: Box::from("history row exceeds job room") });
            }
            return Ok(JobAnswer { writes: writes.into_boxed(), through });
        }
        bytes = next.expect("row fits the job byte edge");
        through = Some(row.key.clone());
        writes.push(write).expect("uncapped job writes");
    }
    Ok(JobAnswer { writes: writes.into_boxed(), through: None })
}
