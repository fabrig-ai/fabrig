use super::Worker;
use skein_lib::Time;

#[must_use]
pub fn jobs_empty(worker: &Worker) -> bool {
    worker.jobs.empty()
}

#[must_use]
pub fn jobs_deadline(worker: &Worker) -> Option<Time> {
    worker.jobs.deadline()
}

pub fn jobs_fire(worker: &mut Worker, now: Time, abort: bool) {
    worker.jobs_fire(now, abort);
}
