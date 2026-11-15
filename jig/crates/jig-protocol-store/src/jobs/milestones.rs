//! Fold lifecycle milestones into attempts and holds (domain/tasks/lifecycle.md, 12.4).
use super::{JobAnswer, JobPage, JobRefusal, JobRow, JobWrite};
use crate::{codec, keys, tasks_v13};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::Key;
use jig_domain_tasks::{Active, AttemptRow, Held, HoldRow, Milestone, OpenAttempt, Phase, Stored};
use skein_lib::{List, length::usize_of};

pub(super) fn page(page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    let mut milestones = List::uncapped();
    for row in &page.rows {
        let Ok(Stored::Milestone(milestone)) = codec::tasks::decode(&row.value, &tasks_v13::CEILINGS, &[]) else {
            return Err(refusal(row, "not a decodable milestone"));
        };
        if keys::decode(&row.key)
            != Some(Key::Tasks(jig_domain_tasks::Key::Milestone { task: milestone.task, position: milestone.position }))
        {
            return Err(refusal(row, "milestone does not match its key"));
        }
        milestones.push(milestone).expect("page bounds milestones");
    }
    let count = prefix(page, milestones.as_slice())?;
    let mut fold = Fold::new();
    for (row, milestone) in page.rows.iter().zip(milestones.as_slice()).take(usize_of(count)) {
        fold.step(row, milestone)?;
    }
    fold.finish()?;
    let through = if usize_of(count) == page.rows.len() {
        None
    } else {
        Some(page.rows.get(usize_of(count).checked_sub(1).expect("nonempty prefix")).expect("page prefix").key.clone())
    };
    Ok(JobAnswer { writes: fold.writes.into_boxed(), through })
}

fn prefix(page: &JobPage, milestones: &[Milestone]) -> Result<u32, JobRefusal> {
    let Some(first) = milestones.first() else { return Ok(0) };
    let last = milestones.last().expect("nonempty page").task;
    let mut fold = Fold::new();
    let mut fitting = 0_u32;
    let mut safe = 0_u32;
    let mut whole = 0_u32;
    let mut previous = first.task;
    for (row, milestone) in page.rows.iter().zip(milestones) {
        if milestone.task != previous {
            whole = fitting;
            previous = milestone.task;
        }
        // The last task may continue beyond this page: leave it unread unless
        // it is the only task, in which case consuming it guarantees progress.
        if first.task != last && milestone.task == last {
            return Ok(fitting);
        }
        fold.step(row, milestone)?;
        let (writes, bytes) = fold.size()?;
        if writes > page.room.writes || bytes > page.room.bytes {
            let count = if whole != 0 {
                whole
            } else if safe != 0 {
                safe
            } else {
                fitting
            };
            return if count == 0 { Err(refusal(row, "room cannot hold one folded milestone")) } else { Ok(count) };
        }
        fitting = fitting.checked_add(1).expect("page row count fits");
        if jig_domain_tasks::run_attempt(&milestone.phase).is_none() {
            safe = fitting;
        }
    }
    Ok(fitting)
}

struct Fold {
    writes: List<JobWrite>,
    bytes: u64,
    task: Option<u64>,
    open: Option<OpenAttempt>,
}
impl Fold {
    fn new() -> Self {
        Self { writes: List::uncapped(), bytes: 0, task: None, open: None }
    }
    fn push(&mut self, write: JobWrite) -> Result<(), JobRefusal> {
        self.bytes = self.bytes.checked_add(write.bytes()).ok_or(refusal_without_key("fold byte count overflow"))?;
        self.writes.push(write).expect("uncapped fold writes");
        Ok(())
    }
    fn step(&mut self, row: &JobRow, milestone: &Milestone) -> Result<(), JobRefusal> {
        if self.task != Some(milestone.task) {
            self.finish()?;
            self.task = Some(milestone.task);
        }
        self.push(JobWrite::Erase { key: row.key.clone() })?;
        if let Phase::Held { was, why, source } = &milestone.phase {
            self.push(put(Stored::Hold(HoldRow {
                task: milestone.task,
                position: milestone.position,
                held: Held { was: was.clone(), why: *why, source: source.clone() },
                at: milestone.at,
            }))?)?;
        }
        let next = jig_domain_tasks::run_attempt(&milestone.phase);
        if let Some(open) = &self.open
            && Some(open.attempt) != next
        {
            let distance = milestone
                .position
                .checked_sub(open.from)
                .ok_or(refusal(row, "attempt phase positions out of order"))?;
            let Ok(phases) = u32::try_from(distance) else {
                return Err(refusal(row, "attempt phase count does not fit"));
            };
            let mut attempt = attempt(milestone.task, open);
            attempt.ended = Some(milestone.at);
            attempt.phases = Some(phases);
            self.push(put(Stored::Attempt(attempt))?)?;
            self.open = None;
        }
        if let Some(number) = next {
            let running = milestone.phase == Phase::Active(Active::Running { attempt: number });
            match &mut self.open {
                Some(open) => {
                    if running && open.started.is_none() {
                        open.started = Some(milestone.at);
                    }
                }
                None => {
                    self.open = Some(OpenAttempt {
                        attempt: number,
                        claimed: Some(milestone.at),
                        started: if running { Some(milestone.at) } else { None },
                        from: milestone.position,
                        end: None,
                    });
                }
            }
        }
        Ok(())
    }
    fn size(&self) -> Result<(u32, u64), JobRefusal> {
        let mut writes = self.writes.len();
        let mut bytes = self.bytes;
        if let Some(open) = &self.open {
            let write = put(Stored::Attempt(attempt(self.task.expect("open task"), open)))?;
            writes = writes.checked_add(1).ok_or(refusal_without_key("fold write count overflow"))?;
            bytes = bytes.checked_add(write.bytes()).ok_or(refusal_without_key("fold byte count overflow"))?;
        }
        Ok((writes, bytes))
    }
    fn finish(&mut self) -> Result<(), JobRefusal> {
        if let Some(open) = self.open.take() {
            self.push(put(Stored::Attempt(attempt(self.task.expect("open task"), &open)))?)?;
        }
        Ok(())
    }
}
fn attempt(task: u64, open: &OpenAttempt) -> AttemptRow {
    AttemptRow {
        task,
        attempt: open.attempt,
        claimed: open.claimed,
        started: open.started,
        ended: None,
        end: None,
        phases: None,
    }
}
fn put(row: Stored) -> Result<JobWrite, JobRefusal> {
    let key = keys::encode(&Key::Tasks(row.key()), u32::MAX).expect("fixed-size history key");
    let Ok(value) = codec::tasks::encode(&row, &tasks_v13::CEILINGS, &[]) else {
        return Err(JobRefusal { key: Some(key), why: Box::from("folded row does not encode") });
    };
    Ok(JobWrite::Put { key, value })
}
fn refusal(row: &JobRow, why: &str) -> JobRefusal {
    JobRefusal { key: Some(row.key.clone()), why: Box::from(why) }
}
fn refusal_without_key(why: &str) -> JobRefusal {
    JobRefusal { key: None, why: Box::from(why) }
}
