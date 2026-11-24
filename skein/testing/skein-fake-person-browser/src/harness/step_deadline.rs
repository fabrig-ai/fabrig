//! Per-person scheduler accounts and step findings (skein/docs/design/ui/web/testing.md, 2).
use super::Play;
use crate::BrowserFace;
use skein_lib::{Duration, Time};
use skein_world::{Deadline, Expired};

pub(super) struct StepAccount {
    pub step: u32,
    pub deadline: Deadline,
}
impl StepAccount {
    pub fn begin(account: &mut Option<Self>, step: u32, pids: [Option<u32>; 3]) {
        if account.as_ref().is_none_or(|account| account.step != step) {
            // The arriving expiry supplies its own bound. This account only measures.
            let mut deadline = Deadline::arm("the browser step", Duration::from_nanos(0));
            for pid in pids.into_iter().flatten() {
                deadline.watch_process(pid);
            }
            *account = Some(Self { step, deadline });
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct StepFailure {
    pub step: u32,
    pub why: String,
    pub bound: Option<Duration>,
}
impl StepFailure {
    pub fn classify(&self, classify_bound: impl FnOnce(Duration) -> Expired) -> Option<Expired> {
        self.bound.map(classify_bound)
    }
}

pub(super) fn step_failure(
    play: &mut dyn Play,
    face: &mut BrowserFace<'_>,
    now: Time,
    step: u32,
) -> Option<StepFailure> {
    face.tick(now);
    play.check_deadline(face, now);
    face_failure(face, step)
}

pub(super) fn face_failure(face: &mut BrowserFace<'_>, step: u32) -> Option<StepFailure> {
    let bound = face.expired();
    face.failure().map(|why| StepFailure { step, why, bound })
}
