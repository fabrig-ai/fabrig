//! Select once before committing the make (temper/docs/design/domain/forge/change/opened.md, 4).
use super::{
    ChangeRow, Domain, Env, Limits, PendingStep, Request, change, emit, finish_step, save_change, wait_for_forge,
};
use alloc::boxed::Box;
use skein_forge::api::{Answer, Error};
use skein_lib::{List, Queue, Wall};

pub(super) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    pending: PendingStep,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let mut row = d.changes.get(&pending.task).expect("taking row").clone();
    let source = row.source.as_ref().expect("taking has a source");
    let head = match result {
        Ok(Answer::Commit(head)) => Some(head),
        Err(Error::Missing) => None,
        Ok(
            Answer::Located { .. }
            | Answer::Items { .. }
            | Answer::Item { .. }
            | Answer::Pull(_)
            | Answer::Reviews { .. }
            | Answer::Statuses { .. }
            | Answer::Remarks { .. }
            | Answer::Branches(_)
            | Answer::PullFiles { .. }
            | Answer::Compare { .. }
            | Answer::Checks(_)
            | Answer::Job { .. }
            | Answer::JobState { .. }
            | Answer::File { .. }
            | Answer::Directory { .. }
            | Answer::Protection(_)
            | Answer::Settings(_)
            | Answer::Collaborators { .. }
            | Answer::Permission(_)
            | Answer::Created(_)
            | Answer::Commented(_)
            | Answer::Reviewed(_)
            | Answer::Merged(_)
            | Answer::Queued
            | Answer::Branch(_)
            | Answer::Runs { .. }
            | Answer::Run(_)
            | Answer::Tree { .. }
            | Answer::Commits { .. }
            | Answer::Done,
        ) => {
            read_error(d, env, row, &pending, Error::InvalidAnswer, out);
            return;
        }
        Err(error) => {
            read_error(d, env, row, &pending, error, out);
            return;
        }
    };
    let mut words = List::uncapped();
    put(&mut words, &source.branch);
    match head {
        Some(head) if source.at.is_none() || source.at == Some(head) => {
            row.kept = Some(head);
            row.effect_account = None;
            d.changes.insert(row.task, row.clone()).expect("known change");
            save_change(d, out, row);
            finish_step(d, env, pending, out);
            return;
        }
        Some(head) => {
            put(&mut words, b" is at ");
            put(&mut words, &crate::commit_text(&head));
            put(&mut words, b", not ");
            put(&mut words, &crate::commit_text(&source.at.expect("mismatch has at")));
            put(&mut words, b": push it and open again");
        }
        None => {
            let repository = d.repository(row.repository).expect("adopted source repository");
            put(&mut words, b" is not in ");
            put(&mut words, &repository.owner);
            put(&mut words, b"/");
            put(&mut words, &repository.name);
        }
    }
    row.ending = Some(crate::Ending::Failed { line: words.into_boxed() });
    d.read_sessions.remove(&row.task);
    d.changes.insert(row.task, row.clone()).expect("known change");
    save_change(d, out, row.clone());
    emit(
        out,
        Request::ChangeDecision { task: row.task, decision: change::Decision::Failed, entry: None, evidence: None },
    );
}

fn read_error(
    d: &mut Domain,
    env: &Env<Limits>,
    mut row: ChangeRow,
    pending: &PendingStep,
    error: Error,
    out: &mut Queue<Request>,
) {
    row.effect_account =
        Some(crate::EffectAccount { entry: 0, words: words(&row, &crate::effect_words::error_words(error)) });
    let retry = match error {
        Error::Timeout | Error::Unavailable | Error::InvalidAnswer | Error::Busy | Error::RateLimited { .. } => true,
        Error::Forbidden
        | Error::Protected
        | Error::Missing
        | Error::MissingJob
        | Error::TooLarge(_)
        | Error::Empty
        | Error::Full
        | Error::Exists
        | Error::NothingToMerge
        | Error::Closed
        | Error::Stale
        | Error::Conflict
        | Error::Refused
        | Error::NotConnected
        | Error::NotInstalled => false,
    };
    if !retry {
        row.change.state = change::State::Held { was: Box::new(row.change.state.clone()), why: change::Hold::Failed };
        d.read_sessions.remove(&row.task);
    }
    d.changes.insert(row.task, row.clone()).expect("known change");
    save_change(d, out, row.clone());
    if retry {
        let retry = Wall::from_nanos(env.wall.as_nanos().saturating_add(1_000_000_000));
        wait_for_forge(d, env, pending, Some(retry), out);
    } else {
        emit(
            out,
            Request::ChangeDecision {
                task: row.task,
                decision: change::Decision::Hold(change::Hold::Failed),
                entry: None,
                evidence: None,
            },
        );
    }
}

fn words(row: &ChangeRow, why: &[u8]) -> Box<[u8]> {
    let mut words = List::uncapped();
    put(&mut words, b"taking ");
    put(&mut words, &row.source.as_ref().expect("taking source").branch);
    put(&mut words, b": ");
    put(&mut words, why);
    crate::text::fit(&words.into_boxed(), 512, b"...")
}

pub(super) fn make_account(
    row: &ChangeRow,
    account: Option<crate::EffectAccount>,
    outcome: skein_forge_client::Outcome,
) -> Option<crate::EffectAccount> {
    if !active(&row.change.state) {
        return account;
    }
    let account = account?;
    let why = match outcome {
        skein_forge_client::Outcome::Failed(error) | skein_forge_client::Outcome::Raced { why: error, .. } => {
            crate::effect_words::error_words(error)
        }
        skein_forge_client::Outcome::Made { .. }
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn
        | skein_forge_client::Outcome::Uncertain
        | skein_forge_client::Outcome::Queued => account.words,
    };
    Some(crate::EffectAccount { entry: account.entry, words: words(row, &why) })
}

fn put(words: &mut List<u8>, text: &[u8]) {
    for byte in text {
        words.push(*byte).expect("bounded taking words");
    }
}

pub(super) fn active(state: &change::State) -> bool {
    if let change::State::Taking { .. } = state {
        true
    } else if let change::State::Held { was, .. } = state {
        active(was)
    } else {
        false
    }
}
