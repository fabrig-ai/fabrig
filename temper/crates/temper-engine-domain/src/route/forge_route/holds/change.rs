//! A change's drift and stall name the heads and the wait (domain/forge/change/repairs.md).
use super::super::{Domain, Env, Limits, List, append, commit_text, forge, forge_change as change, tasks};
use alloc::boxed::Box;

pub(in super::super) fn account(domain: &Domain, env: &Env<Limits>, task: u64, why: change::Hold) -> Option<Box<[u8]>> {
    let row = domain.forge.change(task)?;
    let change::State::Held { was, .. } = &row.change.state else { return None };
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    if why == change::Hold::BranchMoved {
        let name = super::super::topic_name(
            &forge::Topic::Landings { repository: row.repository, branch: row.branch.clone() },
            &env.limits.forge,
        )?;
        let mut observed = match row.ci {
            Some(ci) => Some(ci.head),
            None => domain.forge.branch_head(&name),
        };
        let mut expected = row.change.last_head;
        if let Some(hold) = domain.forge.hold(&name)
            && let Some(forge::Drift {
                change: forge::DriftChange::Branch { expected: before, observed: found }, ..
            }) = &hold.drift
        {
            expected = Some(*before);
            observed = *found;
        }
        append(&mut words, b"its branch moved: expected ");
        head(&mut words, expected.as_ref());
        append(&mut words, b", found ");
        head(&mut words, observed.as_ref());
    } else if why == change::Hold::Stalled {
        append(&mut words, b"no progress at ");
        append(&mut words, step_name(was));
        append(&mut words, b" for ");
        append(
            &mut words,
            skein_lib::Decimal::of(env.limits.forge.change_policy.stall.as_nanos() / 60_000_000_000).as_bytes(),
        );
        append(&mut words, b" minutes: waiting for ");
        waiting(&mut words, was, row);
    } else {
        return None;
    }
    Some(forge::text::fit(words.as_slice(), skein_lib::usize_of(tasks::HOLD_ACCOUNT_BYTES), b"..."))
}

fn head(words: &mut List<u8>, commit: Option<&[u8; 32]>) {
    if let Some(commit) = commit {
        let text = commit_text(commit);
        append(words, text.get(..12).expect("commit text has at least twelve digits"));
    } else {
        append(words, b"unknown");
    }
}

fn step_name(state: &change::State) -> &'static [u8] {
    match state {
        change::State::Producing { .. } => b"producing",
        change::State::Opening { .. } => b"opening",
        change::State::Recreating { .. } => b"recreating",
        change::State::Reopening { .. } => b"reopening",
        change::State::Checking { .. } => b"checking",
        change::State::Gating { .. } => b"gating",
        change::State::Judging { .. } => b"judging",
        change::State::CancellingGates { .. } => b"cancelling gates",
        change::State::Ready { .. } => b"ready",
        change::State::Queued { .. } => b"queued",
        change::State::First { .. } => b"first",
        change::State::Updating { .. } => b"updating",
        change::State::Preparing { .. } => b"preparing",
        change::State::WaitingProducer { .. } => b"waiting producer",
        change::State::Resolving { .. } => b"resolving",
        change::State::Repairing { .. } => b"repairing",
        change::State::Landing { .. } => b"landing",
        change::State::Landed { .. } => b"landed",
        change::State::Red { .. } => b"red",
        change::State::Cleared { .. } => b"cleared",
        change::State::Uncleared { .. } => b"uncleared",
        change::State::NoChange { .. } => b"no change",
        change::State::Ended => b"ended",
        change::State::Held { was, .. } => step_name(was),
    }
}

fn waiting(words: &mut List<u8>, state: &change::State, row: &forge::ChangeRow) {
    match state {
        change::State::Opening { .. } => append(words, b"its pull request to open"),
        change::State::Checking { head: at } => {
            append(words, b"CI at ");
            head(words, Some(at));
        }
        change::State::Gating { head: at, .. } | change::State::CancellingGates { head: at } => {
            append(words, b"its gates at ");
            head(words, Some(at));
        }
        change::State::Judging { target, .. } => { append(words, b"its gates at "); head(words, Some(&target.prefix)); }
        change::State::First { head: at, .. } => {
            append(words, b"the forge to settle whether ");
            head(words, Some(at));
            append(words, b" merges");
        }
        change::State::Updating { from, base, .. } | change::State::Preparing { from, main: base, .. } => {
            append(words, b"its update of ");
            head(words, Some(from));
            append(words, b" with ");
            append(words, &forge::text::fit(&row.base, 256, b"..."));
            append(words, b" at ");
            head(words, Some(base));
        }
        change::State::Landing { head: at } => {
            append(words, b"its merge of ");
            head(words, Some(at));
        }
        change::State::Recreating { .. } => append(words, b"its branch to be recreated"),
        change::State::Reopening { .. } => append(words, b"its pull request to reopen"),
        change::State::Producing { .. } => append(words, b"its producer's head and result"),
        change::State::WaitingProducer { .. } => append(words, b"its producer to settle"),
        change::State::Resolving { .. } => append(words, b"its resolution's head and result"),
        change::State::Repairing { .. } => append(words, b"its repair's head and result"),
        change::State::Ready { .. } | change::State::Queued { .. } => {
            append(words, b"its landing turn and branch writer");
        }
        change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => append(words, b"its landing's verification"),
        change::State::NoChange { .. } | change::State::Ended => append(words, b"its task to end"),
        change::State::Held { was, .. } => waiting(words, was, row),
    }
}
