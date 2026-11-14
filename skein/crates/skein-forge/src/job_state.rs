//! Bounded job-state labels (forge/job-state.md, 2).
use crate::api;
use alloc::boxed::Box;
pub(crate) fn keep(labels: &mut Box<[Box<[u8]>]>, cut: &mut bool, label: Box<[u8]>) {
    if labels.len() >= 16 || label.len() > 255 {
        *cut = true;
    } else {
        let mut kept = skein_lib::List::with_capacity(16);
        for previous in labels.iter() {
            kept.push(previous.clone()).expect("bounded labels");
        }
        kept.push(label).expect("label room checked");
        *labels = kept.into_boxed();
    }
}
pub(crate) fn answer(
    attempt: api::JobAttempt,
    state: api::JobState,
    labels: Box<[Box<[u8]>]>,
    step: Option<api::Step>,
    mut cut: bool,
    bytes: u32,
) -> Result<api::Answer, api::Error> {
    let step = if state == api::JobState::Running { step } else { None };
    let mut size = match &step {
        Some(step) => skein_lib::len_u64(step.name.len()),
        None => 0,
    };
    if size > u64::from(bytes) {
        return Err(api::Error::TooLarge(api::Excess::unnamed()));
    }
    let mut kept = skein_lib::List::with_capacity(16);
    for label in labels {
        let next = size.saturating_add(16).saturating_add(skein_lib::len_u64(label.len()));
        if next > u64::from(bytes) {
            cut = true;
        } else {
            size = next;
            kept.push(label).expect("bounded labels");
        }
    }
    let labels = kept.into_boxed();
    Ok(api::Answer::JobState { attempt, state, labels, step, cut })
}
