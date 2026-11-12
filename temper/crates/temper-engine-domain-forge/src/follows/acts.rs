//! Owner acts retained until the follow's next decision (follows.md, 5).
use super::{FollowRefusal, FollowRow, FollowState};
use crate::{Domain, Limits, Request};
use alloc::boxed::Box;
use skein_forge::api::Commit;
use skein_lib::{Duration, Env, Queue, Token};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FollowAct {
    Start { after: Commit },
    Hold { after: Commit, reason: Box<[u8]> },
    Skip { after: Commit, reason: Box<[u8]> },
    Merge { afters: Box<[Commit]> },
}
impl FollowAct {
    #[must_use]
    pub fn afters(&self) -> &[Commit] {
        match self {
            Self::Start { after } | Self::Hold { after, .. } | Self::Skip { after, .. } => core::slice::from_ref(after),
            Self::Merge { afters } => afters,
        }
    }
}
impl FollowRefusal {
    #[must_use]
    pub const fn words(self) -> &'static [u8] {
        match self {
            Self::UnknownFollow => b"no such follow",
            Self::Stopped => b"the follow is stopped",
            Self::Skipped => b"this entry was skipped",
            Self::UnknownEntry => b"no queued entry has this commit; it may already be followed or skipped",
            Self::AlreadyStarted => b"the change has started producing",
            Self::NotConsecutive => b"merge needs consecutive entries in queue order",
            Self::MergeBound => b"merge needs two to forge.follow_merge entries",
            Self::ReasonBound => b"the reason exceeds forge.follow_reason_bytes",
            Self::NoWord => b"code starts this entry; it has no owner's hold to lift",
            Self::Pending => b"the follow has an owner act awaiting its step",
            Self::NotOwner => b"only the follow's owner or an owner of its project may act",
        }
    }
}
impl Domain {
    pub fn follow_act(&mut self, project: u32, follow: u32, act: FollowAct) -> Result<(), FollowRefusal> {
        let mut row = self.follow_row(project, follow).cloned().ok_or(FollowRefusal::UnknownFollow)?;
        if row.stopped.is_some() {
            return Err(FollowRefusal::Stopped);
        }
        if row.pending.is_some() {
            return Err(FollowRefusal::Pending);
        }
        match &act {
            FollowAct::Start { after } => {
                // Reuse the owner-word validation without publishing it before the step.
                let setting = self.follow(project, follow).ok_or(FollowRefusal::UnknownFollow)?;
                if setting.start == crate::settings::FollowStart::Code {
                    let mut held = false;
                    for entry in &row.queue {
                        if entry.after == *after {
                            held = matches_hold(&entry.state);
                        }
                    }
                    if !held {
                        return Err(FollowRefusal::NoWord);
                    }
                }
                drop(self.follow_start(project, follow, *after)?);
            }
            FollowAct::Hold { reason, .. } | FollowAct::Skip { reason, .. } => {
                if reason.len() > skein_lib::usize_of(self.follow_reason_bytes) {
                    return Err(FollowRefusal::ReasonBound);
                }
            }
            FollowAct::Merge { afters } => {
                if afters.len() < 2 || afters.len() > skein_lib::usize_of(self.follow_merge) {
                    return Err(FollowRefusal::MergeBound);
                }
            }
        }
        let mut prior: Option<usize> = None;
        for after in act.afters() {
            let mut found = None;
            for (at, entry) in row.queue.iter().enumerate() {
                if entry.after == *after {
                    found = Some((at, entry));
                    break;
                }
            }
            let Some((at, entry)) = found else {
                for skip in &row.skips {
                    if skip.after == *after {
                        return Err(FollowRefusal::Skipped);
                    }
                }
                return Err(FollowRefusal::UnknownEntry);
            };
            match &act {
                FollowAct::Hold { .. } | FollowAct::Merge { .. } => {
                    if let Some(change) = entry.change
                        && let Some(change) = self.change(change)
                        && produced(&change.change.state)
                    {
                        return Err(FollowRefusal::AlreadyStarted);
                    }
                    match &entry.state {
                        FollowState::Producing => return Err(FollowRefusal::AlreadyStarted),
                        FollowState::NotMade
                        | FollowState::WaitsWord
                        | FollowState::WaitsFor(_)
                        | FollowState::HeldByOwner(_)
                        | FollowState::Held(_)
                        | FollowState::Ended(_) => {}
                    }
                }
                FollowAct::Start { .. } | FollowAct::Skip { .. } => {}
            }
            if let Some(prior) = prior
                && prior.checked_add(1) != Some(at)
            {
                return Err(FollowRefusal::NotConsecutive);
            }
            prior = Some(at);
        }
        row.pending = Some(expand(&row, act, self.follow_merge)?);
        self.follow_rows.insert((project, follow), row).expect("existing follow");
        Ok(())
    }
}
fn matches_hold(state: &FollowState) -> bool {
    match state {
        FollowState::HeldByOwner(_) => true,
        FollowState::NotMade
        | FollowState::WaitsWord
        | FollowState::WaitsFor(_)
        | FollowState::Producing
        | FollowState::Held(_)
        | FollowState::Ended(_) => false,
    }
}
#[expect(clippy::too_many_arguments, reason = "the keyed admission and act travel together")]
pub(crate) fn asked(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    retention: Duration,
    project: u32,
    follow: u32,
    act: FollowAct,
    authorized: bool,
    out: &mut Queue<Request>,
) {
    if !crate::adoption::room(d, env, request, out) {
        return;
    }
    let mut words = skein_lib::List::uncapped();
    for after in act.afters() {
        for byte in crate::commit_text(after) {
            words.push(byte).expect("act commits");
        }
        words.push(b' ').expect("separator");
    }
    let result = if authorized { d.follow_act(project, follow, act) } else { Err(FollowRefusal::NotOwner) };
    let outcome = match result {
        Ok(()) => {
            let row = d.follow_row(project, follow).expect("accepted act");
            super::save(row, out);
            crate::ApplicationOutcome::Follow(None)
        }
        Err(why) => {
            for byte in why.words() {
                words.push(*byte).expect("refusal words");
            }
            crate::ApplicationOutcome::Follow(Some(words.into_boxed()))
        }
    };
    crate::adoption::finish(
        d,
        env,
        request,
        crate::adoption::Pending { token, retention, observed: None },
        outcome,
        out,
    );
}

fn produced(state: &temper_engine_domain_forge_change::State) -> bool {
    use temper_engine_domain_forge_change::State;
    match state {
        State::Producing { requested } => *requested,
        State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Gating { .. }
        | State::Judging { .. }
        | State::CancellingGates { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. }
        | State::Held { .. }
        | State::Landed { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::NoChange { .. }
        | State::Ended => true,
    }
}

fn expand(row: &FollowRow, mut act: FollowAct, bound: u32) -> Result<FollowAct, FollowRefusal> {
    match &act {
        FollowAct::Merge { afters } => {
            let mut changes = skein_lib::List::uncapped();
            for entry in &row.queue {
                if afters.contains(&entry.after)
                    && let Some(change) = entry.change
                {
                    changes.push(change).expect("selected changes");
                }
            }
            let mut merged = skein_lib::List::uncapped();
            for entry in &row.queue {
                let carried = match entry.change {
                    Some(change) => changes.as_slice().contains(&change),
                    None => false,
                };
                if afters.contains(&entry.after) || carried {
                    merged.push(entry.after).expect("merged queue entries");
                }
            }
            if merged.len() > bound {
                return Err(FollowRefusal::MergeBound);
            }
            act = FollowAct::Merge { afters: merged.into_boxed() };
        }
        FollowAct::Start { .. } | FollowAct::Hold { .. } | FollowAct::Skip { .. } => {}
    }
    Ok(act)
}
