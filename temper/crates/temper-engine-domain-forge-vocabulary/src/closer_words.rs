//! Closer change and hold words (temper/docs/design/domain/forge/branches.md, 2).
use crate::routing::{append, decimal};
use crate::{Ending, hex, text};
use alloc::boxed::Box;
use jig_domain_tasks::HOLD_ACCOUNT_BYTES;
use skein_forge::api::Commit;
use skein_lib::List;
use temper_engine_domain_forge_change::{Cause, CloseAt, CloseOutdated};

const UPDATE: &[u8] =
    b". Catch the goal up from main with a change naming main's tip and release the closer, or release it to ";

pub fn catch_up(goal: u64, commit: &Commit) -> Box<[u8]> {
    let mut words = List::uncapped();
    append(&mut words, b"Catch goal ");
    append(&mut words, &decimal(goal));
    append(&mut words, b" up from main at ");
    short_commit(&mut words, commit);
    append(&mut words, b".");
    words.into_boxed()
}

pub fn catch_up_ended(
    change: u64,
    ended: Option<Ending>,
    close: u64,
    at: CloseAt,
    finding: CloseOutdated,
) -> Box<[u8]> {
    let mut words = List::uncapped();
    append(&mut words, b"the catch-up from main at ");
    short_commit(&mut words, &at.landing);
    append(&mut words, b", change T");
    append(&mut words, &decimal(change));
    append(&mut words, b", ended ");
    append_ending(&mut words, ended);
    append(&mut words, b", after ");
    close_words(&mut words, close, &at.branch);
    append(&mut words, b" outdated: ");
    append(&mut words, finding_words(finding));
    append(&mut words, UPDATE);
    append(&mut words, b"catch up again.");
    fit(&words)
}

pub fn outdated(close: u64, at: CloseAt, finding: CloseOutdated, update: Result<(u64, Commit), &[u8]>) -> Box<[u8]> {
    let mut words = List::uncapped();
    close_words(&mut words, close, &at.branch);
    match update {
        Ok((change, commit)) => {
            append(&mut words, b" outdated again, after its catch-up from main at ");
            short_commit(&mut words, &commit);
            append(&mut words, b", change T");
            append(&mut words, &decimal(change));
            append(&mut words, b", landed: ");
            append(&mut words, finding_words(finding));
            append(&mut words, UPDATE);
            append(&mut words, b"close again as it stands.");
        }
        Err(why) => {
            append(&mut words, b" outdated: ");
            append(&mut words, finding_words(finding));
            append(&mut words, b"; the closer can't catch it up from main: ");
            append(&mut words, why);
            append(&mut words, b". Merge main into the goal's branch and release the closer to close again.");
        }
    }
    fit(&words)
}

fn close_words(words: &mut List<u8>, close: u64, branch: &Commit) {
    append(words, b"the close step's change T");
    append(words, &decimal(close));
    append(words, b" found the goal's branch at ");
    short_commit(words, branch);
}

fn short_commit(words: &mut List<u8>, commit: &Commit) {
    for byte in commit.get(..6).expect("six bytes of a commit") {
        append(words, &hex(*byte));
    }
}

fn finding_words(finding: CloseOutdated) -> &'static [u8] {
    match finding {
        CloseOutdated::Window => b"it holds none of the commits main's published counts cover",
        CloseOutdated::Ancestry => b"its ancestry was incomplete",
    }
}

fn fit(words: &List<u8>) -> Box<[u8]> {
    text::fit(words.as_slice(), skein_lib::usize_of(HOLD_ACCOUNT_BYTES), b"")
}

pub fn branch_reason(held: Option<Cause>) -> Option<&'static [u8]> {
    match held {
        Some(Cause::Repairs) => Some(b"repairs"),
        Some(Cause::Runner) => Some(b"its CI runner's re-runs"),
        Some(Cause::Resolutions) => Some(b"conflict resolutions"),
        Some(Cause::Rejected) => Some(b"a rejection"),
        Some(Cause::PullClosed) => Some(b"its pull request closed"),
        Some(Cause::BranchMissing) => Some(b"its branch missing"),
        Some(
            Cause::Stalled
            | Cause::Updates
            | Cause::BranchMoved
            | Cause::Retargeted
            | Cause::Failed
            | Cause::RedStands
            | Cause::CloseRefused
            | Cause::Authority(_)
            | Cause::Questions,
        )
        | None => None,
    }
}

pub fn append_ending(words: &mut List<u8>, ended: Option<Ending>) {
    match ended {
        Some(Ending::Cancelled) => append(words, b"cancelled"),
        Some(Ending::Failed { line }) => {
            append(words, b"failed: ");
            append(words, temper_engine_domain_forge_closer::first_line(&line));
        }
        Some(Ending::NoChange) => append(words, b"with no change"),
        Some(Ending::Landed) => append(words, b"landed"),
        None => append(words, b"without an ending account"),
    }
}
