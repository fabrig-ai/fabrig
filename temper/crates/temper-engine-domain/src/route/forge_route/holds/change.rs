//! A change's drift and stall name the heads and the wait (domain/forge/change/stalls.md, 3).
use super::super::{Domain, Env, Limits, List, append, commit_text, forge, forge_change as change, tasks};
use alloc::boxed::Box;

#[expect(clippy::too_many_lines, reason = "one hold account chooses among runner, repair and forge read evidence")]
pub(in super::super) fn account(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    why: change::Cause,
    evidence: Option<&forge::ChangeEvidence>,
) -> Option<Box<[u8]>> {
    let row = domain.forge.change(task)?;
    if why == change::Cause::Runner
        && let Some(forge::RunnerCi::Held { words }) = &row.runner_ci
    {
        return Some(words.clone());
    }
    if why == change::Cause::RedStands {
        return row.producer_failure.clone();
    }
    let change::State::Held { was, .. } = &row.change.state else { return None };
    if why == change::Cause::Stalled
        && row.room.is_some()
        && let Some(account) = &row.effect_account
    {
        return Some(account.words.clone());
    }
    if let change::State::Taking { .. } = was.as_ref()
        && (why == change::Cause::Stalled || why == change::Cause::Failed)
    {
        if let Some(account) = &row.effect_account {
            return Some(account.words.clone());
        }
        let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
        append(&mut words, b"taking ");
        append(&mut words, &row.source.as_ref()?.branch);
        append(&mut words, b": waiting for forge");
        return Some(words.into_boxed());
    }
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    if why == change::Cause::BranchMoved {
        let name = super::super::topic_name(
            &forge::Topic::Landings { repository: row.repository, branch: row.branch.clone() },
            &env.limits.forge,
        )?;
        let mut expected = was.pinned_head().or(row.change.last_head).or(row.kept);
        let observed = match evidence {
            Some(read) => read.head,
            None => match domain.forge.hold(&name) {
                Some(hold) => match &hold.drift {
                    Some(forge::Drift {
                        change: forge::DriftChange::Branch { expected: before, observed }, ..
                    }) => {
                        expected = Some(*before);
                        *observed
                    }
                    Some(_) | None => domain.forge.branch_head(&name),
                },
                None => domain.forge.branch_head(&name),
            },
        };
        append(&mut words, b"its branch moved: expected ");
        head(&mut words, expected.as_ref());
        append(&mut words, b", found ");
        head(&mut words, observed.as_ref());
        let order = match evidence {
            Some(read) => read.nearest_write,
            None => None,
        };
        let nearest = match order {
            Some(order) => domain.forge.write_entry(&name, order),
            None => domain.forge.nearest_write(&name),
        };
        if let Some(entry) = nearest {
            last_write(&mut words, entry);
        }
    } else if why == change::Cause::PullClosed {
        append(&mut words, b"its pull request closed: ");
        append(&mut words, skein_lib::Decimal::of(row.pull?).as_bytes());
    } else if why == change::Cause::Stalled
        && let Some(read) = evidence
        && let Some(behind) = read.behind
    {
        behind_words(&mut words, behind);
    } else if why == change::Cause::Stalled
        && let Some(read) = evidence
        && read.readback_owed
    {
        append(&mut words, b"can't tell whether the forge read shows a write of temper's: its readback did not settle");
    } else if why == change::Cause::Stalled {
        append(&mut words, b"no progress at ");
        append(&mut words, was.step_name());
        append(&mut words, b" for ");
        let runner = runner_stalled(was, row, env);
        let bound =
            if runner { env.limits.forge.change_policy.runner_wait } else { env.limits.forge.change_policy.stall };
        append(&mut words, skein_lib::Decimal::of(bound.as_nanos() / 60_000_000_000).as_bytes());
        append(&mut words, b" minutes: waiting for ");
        if runner {
            append(&mut words, b"a runner for ");
        }
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

fn runner_stalled(state: &change::State, row: &forge::ChangeRow, env: &Env<Limits>) -> bool {
    if let change::State::Checking { .. } = state
        && let Some(clock) = row.change.clock
    {
        return clock.waited >= env.limits.forge.change_policy.runner_wait;
    }
    false
}

/// Every change hold has an account even when the deciding neighbour gave none.
pub(in super::super) fn fallback(domain: &Domain, env: &Env<Limits>, task: u64, why: change::Cause) -> Box<[u8]> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, fallback_reason(why));
    if let Some(row) = domain.forge.change(task) {
        if let change::State::Held { was, .. } = &row.change.state {
            append(&mut words, b"; change was ");
            append(&mut words, was.step_name());
        }
        if why == change::Cause::Resolutions || why == change::Cause::Updates {
            let (taken, bound) = if why == change::Cause::Resolutions {
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
    words.into_boxed()
}

pub(in super::super) fn fallback_reason(why: change::Cause) -> &'static [u8] {
    match why {
        change::Cause::Stalled => b"its forge wait exceeded the stall bound",
        change::Cause::Runner => b"CI's runner ran out of re-runs; the change wasn't judged",
        change::Cause::Repairs => b"its repair allowance is spent; amend the allowance before releasing",
        change::Cause::Resolutions => {
            b"its conflict resolution allowance is spent; resolve the conflict or amend the allowance"
        }
        change::Cause::Updates => b"its branch update allowance is spent; update the branch or amend the allowance",
        change::Cause::BranchMoved => b"its branch moved outside its writer; release to accept the current head",
        change::Cause::BranchMissing => b"its branch is missing; release to recreate it",
        change::Cause::PullClosed => b"its pull request closed without merging; release to reopen it",
        change::Cause::Retargeted => b"its pull request targets another base; release to restore its target",
        change::Cause::CloseRefused => {
            b"the close check refused the goal's branch; it checks again once the goal's branch or main moves"
        }
        change::Cause::Failed => b"its delegate or forge effect failed without an account",
        change::Cause::RedStands => {
            b"its repair changed nothing and its red stands; release to ask a fresh repair, or cancel it"
        }
        change::Cause::Rejected => b"a person rejected its required gate; resolve the rejection before releasing",
        change::Cause::Authority(_) => b"its next delegate exceeds its authority; amend the authority before releasing",
        change::Cause::Questions => {
            b"its question allowance is spent; answer the pending question or amend the allowance"
        }
    }
}

fn behind_words(words: &mut List<u8>, behind: forge::ReadBehind) {
    append(
        words,
        match behind.read {
            forge::BehindRead::Branch => b"its branch reads ",
            forge::BehindRead::Base => b"its base branch reads ",
            forge::BehindRead::PullHead => b"its pull request's head reads ",
            forge::BehindRead::PullBase => b"its pull request's base reads ",
            forge::BehindRead::PullState => b"its pull request reads an older state of temper's",
        },
    );
    if let Some(found) = behind.found {
        head(words, Some(&found));
        append(words, b", an older head of temper's (expected ");
        head(words, behind.expected.as_ref());
        append(words, b")");
    } else if behind.read != forge::BehindRead::PullState {
        append(words, b"as missing (expected ");
        head(words, behind.expected.as_ref());
        append(words, b")");
    }
    append(words, b": the read may be stale or another's write");
}

pub(super) fn last_write(words: &mut List<u8>, entry: &forge::Written) {
    use forge::writes::{Operation, Sender, WriteState};
    use skein_lib::Decimal;
    append(words, b"; temper's last write on it: ");
    let (action, commit) = match &entry.operation {
        Operation::Push { after, .. } => (b"push".as_slice(), Some(after)),
        Operation::Create { after } => (b"create branch".as_slice(), Some(after)),
        Operation::Delete { before } => (b"delete branch".as_slice(), before.as_ref()),
        Operation::Update { after, .. } => (b"update".as_slice(), after.as_ref()),
        Operation::MergedInto { after, .. } => (b"merge into branch".as_slice(), after.as_ref()),
        Operation::Merge { head, .. } => (b"merge".as_slice(), Some(head)),
        Operation::Open { head, .. } => (b"open pull".as_slice(), Some(head)),
        Operation::Close { .. } => (b"close pull".as_slice(), None),
        Operation::Reopen { .. } => (b"reopen pull".as_slice(), None),
        Operation::Retarget { .. } => (b"retarget pull".as_slice(), None),
        Operation::Writer { before } => (b"writer".as_slice(), before.as_ref()),
        Operation::Compare { after } => (b"comparison".as_slice(), Some(after)),
    };
    append(words, action);
    if let Some(commit) = commit {
        append(words, b" of ");
        head(words, Some(commit));
    }
    match entry.sender {
        Sender::Attempt { task, attempt } => {
            append(words, b" by task ");
            append(words, Decimal::of(task).as_bytes());
            append(words, b" attempt ");
            append(words, Decimal::of(attempt).as_bytes());
        }
        Sender::Entry { task, entry } => {
            append(words, b" by task ");
            append(words, Decimal::of(task).as_bytes());
            append(words, b" entry ");
            append(words, Decimal::of(entry).as_bytes());
        }
        Sender::Previous => append(words, b" by the previous build"),
    }
    append(
        words,
        match entry.state {
            WriteState::Admitted => b", admitted",
            WriteState::Sent => b", sent",
            WriteState::Confirmed => b", confirmed",
            WriteState::Refused => b", refused",
            WriteState::Uncertain => b", uncertain",
        },
    );
}

fn head(words: &mut List<u8>, commit: Option<&[u8; 32]>) {
    if let Some(commit) = commit {
        let text = commit_text(commit);
        append(words, text.get(..12).expect("commit text has at least twelve digits"));
    } else {
        append(words, b"unknown");
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
        change::State::Taking { .. }
        | change::State::Producing { .. }
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
        change::State::Taking { .. } => append(words, b"its source read or branch make"),
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
