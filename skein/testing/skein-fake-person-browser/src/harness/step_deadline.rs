//! Per-person scheduler accounts and step findings (skein/docs/design/ui/web/testing.md, 2).
use super::{Account, Arming, Play};
use crate::BrowserFace;
use skein_lib::{Duration, Time};
use skein_world::deadline::ProcessSampler;

pub(super) struct StepAccount {
    pub step: u32,
    pub deadline: Account,
}
impl StepAccount {
    pub fn begin(
        account: &mut Option<Self>,
        step: u32,
        command: Option<u32>,
        arming: &mut Arming,
        sampler: &mut ProcessSampler,
    ) {
        if account.as_ref().is_none_or(|account| account.step != step) {
            // The arriving expiry supplies its own bound. This account only measures.
            let deadline = arming.arm("the browser step", Duration::from_nanos(0), command, sampler);
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
    pub fn classify<T>(&self, classify_bound: impl FnOnce(Duration) -> T) -> Option<T> {
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
