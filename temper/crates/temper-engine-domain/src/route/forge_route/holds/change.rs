//! A change's drift and stall name the heads and the wait (domain/forge/change/repairs.md).
use super::super::{Domain, Env, Limits, List, append, commit_text, forge, forge_change as change, tasks};
use alloc::boxed::Box;

pub(in super::super) fn account(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    why: change::Hold,
    evidence: Option<&forge::ChangeEvidence>,
) -> Option<Box<[u8]>> {
    let row = domain.forge.change(task)?;
    let change::State::Held { was, .. } = &row.change.state else { return None };
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    if why == change::Hold::BranchMoved {
        let name = super::super::topic_name(
            &forge::Topic::Landings { repository: row.repository, branch: row.branch.clone() },
            &env.limits.forge,
        )?;
        let mut observed = match evidence {
            Some(read) => read.head,
            None => domain.forge.branch_head(&name),
        };
        let mut expected = expected_head(was).or(row.change.last_head);
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
        if evidence.is_some() {
            waiting_on_reads(&mut words, was, row);
        } else {
            waiting(&mut words, was, row);
        }
        if let change::State::Checking { .. } = was.as_ref() {
            return Some(domain.forge.ci_stall_words(task, words.as_slice(), tasks::HOLD_ACCOUNT_BYTES));
        }
    } else {
        return None;
    }
    Some(forge::text::fit(words.as_slice(), skein_lib::usize_of(tasks::HOLD_ACCOUNT_BYTES), b"..."))
}

/// Every change hold has an account even when the deciding neighbour gave none.
pub(in super::super) fn fallback(domain: &Domain, env: &Env<Limits>, task: u64, why: change::Hold) -> Box<[u8]> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(
        &mut words,
        match why {
            change::Hold::Stalled => b"its forge wait exceeded the stall bound",
            change::Hold::Repairs => b"its repair allowance is spent; amend the allowance before releasing",
            change::Hold::Resolutions => {
                b"its conflict resolution allowance is spent; resolve the conflict or amend the allowance"
            }
            change::Hold::Updates => b"its branch update allowance is spent; update the branch or amend the allowance",
            change::Hold::BranchMoved => b"its branch moved outside its writer; release to accept the current head",
            change::Hold::BranchMissing => b"its branch is missing; release to recreate it",
            change::Hold::PullClosed => b"its pull request closed without merging; release to reopen it",
            change::Hold::Retargeted => b"its pull request targets another base; release to restore its target",
            change::Hold::Failed => b"its delegate or forge effect failed without an account",
            change::Hold::Rejected => b"a person rejected its required gate; resolve the rejection before releasing",
            change::Hold::Authority(_) => {
                b"its next delegate exceeds its authority; amend the authority before releasing"
            }
            change::Hold::Questions => {
                b"its question allowance is spent; answer the pending question or amend the allowance"
            }
        },
    );
    if let Some(row) = domain.forge.change(task) {
        if let change::State::Held { was, .. } = &row.change.state {
            append(&mut words, b"; change was ");
            append(&mut words, step_name(was));
        }
        if why == change::Hold::Resolutions || why == change::Hold::Updates {
            let (taken, bound) = if why == change::Hold::Resolutions {
                (row.change.resolutions, env.limits.forge.change_policy.resolutions)
            } else {
                (row.change.updates, env.limits.forge.change_policy.updates)
            };
            append(&mut words, b"; ");
            append(&mut words, skein_lib::Decimal::of(u64::from(taken)).as_bytes());
            append(&mut words, b" of ");
            append(&mut words, skein_lib::Decimal::of(u64::from(bound)).as_bytes());
            append(&mut words, b" taken");
        }
    }
    let revision = match domain.core.tasks.task(task) {
        Some(row) => row.revisions.hold.saturating_add(1),
        None => 1,
    };
    super::append_recovery(task, revision, &mut words);
    words.into_boxed()
}

fn expected_head(state: &change::State) -> Option<[u8; 32]> {
    match state {
        change::State::Checking { head }
        | change::State::Gating { head, .. }
        | change::State::Queued { head, .. }
        | change::State::First { head, .. }
        | change::State::Ready { head, .. }
        | change::State::Judging { head, .. }
        | change::State::CancellingGates { head }
        | change::State::Recreating { head }
        | change::State::Landing { head } => Some(*head),
        change::State::Updating { from, .. } | change::State::Preparing { from, .. } => Some(*from),
        change::State::Held { was, .. } => expected_head(was),
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Reopening { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::NoChange { .. }
        | change::State::Ended => None,
    }
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

fn waiting_on_reads(words: &mut List<u8>, state: &change::State, row: &forge::ChangeRow) {
    match state {
        change::State::Opening { .. }
        | change::State::Checking { .. }
        | change::State::Gating { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::Landing { .. } => waiting(words, state, row),
        change::State::Held { was, .. } => waiting_on_reads(words, was, row),
        change::State::Producing { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Judging { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Queued { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::NoChange { .. }
        | change::State::Ended => append(words, b"the forge to answer its reads"),
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
        change::State::Judging { target, .. } => {
            append(words, b"its gates at ");
            head(words, Some(&target.prefix));
        }
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
