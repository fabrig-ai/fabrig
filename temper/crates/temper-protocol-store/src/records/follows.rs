//! Follow records (temper/docs/design/protocol/store.md, 2, family 42).
use super::{Bounds, FITS, Undecodable, names, w};
use alloc::boxed::Box;
use skein_codec::Record;
use skein_lib::{List, Reader};
use temper_engine_domain_forge_vocabulary as d;
fn reason(bytes: &[u8]) -> w::FollowReason {
    w::FollowReason { reason: Box::from(bytes) }
}
fn encode_state(state: &d::FollowState) -> w::FollowState {
    match state {
        d::FollowState::NotMade => w::FollowState::NotMade,
        d::FollowState::WaitsWord => w::FollowState::WaitsWord,
        d::FollowState::HeldByOwner(words) => w::FollowState::HeldByOwner(reason(words)),
        d::FollowState::WaitsFor(number) => w::FollowState::WaitsFor(w::Number { number: *number }),
        d::FollowState::Producing => w::FollowState::Producing,
        d::FollowState::Held(words) => w::FollowState::Held(reason(words)),
        d::FollowState::Ended(words) => w::FollowState::Ended(reason(words)),
    }
}
fn decode_state(state: w::FollowState) -> d::FollowState {
    match state {
        w::FollowState::NotMade => d::FollowState::NotMade,
        w::FollowState::WaitsWord => d::FollowState::WaitsWord,
        w::FollowState::HeldByOwner(words) => d::FollowState::HeldByOwner(words.reason),
        w::FollowState::WaitsFor(number) => d::FollowState::WaitsFor(number.number),
        w::FollowState::Producing => d::FollowState::Producing,
        w::FollowState::Held(words) => d::FollowState::Held(words.reason),
        w::FollowState::Ended(words) => d::FollowState::Ended(words.reason),
    }
}
fn encode_entry(entry: &d::FollowEntry) -> w::FollowEntry {
    let mut commits = List::with_capacity(u32::try_from(entry.commits.len()).expect(FITS));
    for commit in &entry.commits {
        commits.push(w::FollowCommit { id: commit.id, message: commit.message.clone() }).expect(FITS);
    }
    let files = match &entry.files {
        Some(files) => Some(w::FollowFiles { files: names::encode_piece(files) }),
        None => None,
    };
    w::FollowEntry {
        before: entry.before,
        after: entry.after,
        pull: entry.pull,
        commits,
        more_commits: entry.more_commits,
        files,
        more_files: entry.more_files,
        change: entry.change,
        state: encode_state(&entry.state),
        word: entry.word,
        remake: entry.remake,
    }
}
fn decode_entry(entry: w::FollowEntry, bounds: &Bounds) -> Result<d::FollowEntry, Undecodable> {
    let w::FollowEntry { before, after, pull, commits, more_commits, files, more_files, change, state, word, remake } =
        entry;
    let mut decoded = List::with_capacity(commits.len());
    for commit in commits.into_boxed() {
        let w::FollowCommit { id, message } = commit;
        decoded.push(d::FollowCommit { id, message }).expect(FITS);
    }
    Ok(d::FollowEntry {
        before,
        after,
        pull,
        commits: decoded.into_boxed(),
        more_commits,
        files: match files {
            Some(files) => Some(decode_files(&files.files, bounds)?),
            None => None,
        },
        more_files,
        change,
        state: decode_state(state),
        word,
        remake,
    })
}
fn encode_parity(parity: Option<&d::FollowParity>) -> Option<w::FollowParity> {
    let parity = parity?;
    let verdict = match parity.verdict {
        d::FollowVerdict::Passed => w::FollowVerdict::Passed,
        d::FollowVerdict::Failing(count) => w::FollowVerdict::Failing(w::FollowFailing { count }),
        d::FollowVerdict::Many => w::FollowVerdict::Many,
    };
    Some(w::FollowParity { source: parity.source, head: parity.head, verdict, run: parity.run.clone() })
}
fn decode_parity(parity: Option<w::FollowParity>) -> Option<d::FollowParity> {
    let w::FollowParity { source, head, verdict, run } = parity?;
    Some(d::FollowParity {
        source,
        head,
        verdict: match verdict {
            w::FollowVerdict::Passed => d::FollowVerdict::Passed,
            w::FollowVerdict::Failing(count) => d::FollowVerdict::Failing(count.count),
            w::FollowVerdict::Many => d::FollowVerdict::Many,
        },
        run,
    })
}
pub(super) fn encode(row: &d::FollowRow) -> w::FollowRow {
    let mut queue = List::with_capacity(u32::try_from(row.queue.len()).expect(FITS));
    for entry in &row.queue {
        queue.push(encode_entry(entry)).expect(FITS);
    }
    let mut skips = List::with_capacity(u32::try_from(row.skips.len()).expect(FITS));
    for skip in &row.skips {
        skips.push(w::FollowSkip { after: skip.after, reason: skip.reason.clone() }).expect(FITS);
    }
    let mut questions = List::with_capacity(u32::try_from(row.questions.len()).expect(FITS));
    for question in &row.questions {
        questions
            .push(w::FollowQuestion {
                change: question.change,
                number: question.number,
                words: question.words.clone(),
                person: question.person,
                relayed: question.relayed,
                answer: question.answer.clone(),
            })
            .expect(FITS);
    }
    w::FollowRow {
        project: row.project,
        follow: row.follow,
        source: names::encode_repository(row.source),
        branch: row.branch.clone(),
        heard: row.heard,
        task: row.task,
        stopped: row.stopped.clone(),
        queue,
        skips,
        parity: encode_parity(row.parity.as_ref()),
        questions,
        pending: match &row.pending {
            Some(act) => Some(encode_act(act)),
            None => None,
        },
    }
}
pub(super) fn decode(row: w::FollowRow, bounds: &Bounds) -> Result<d::FollowRow, Undecodable> {
    let w::FollowRow {
        project,
        follow,
        source,
        branch,
        heard,
        task,
        stopped,
        queue,
        skips,
        parity,
        questions,
        pending,
    } = row;
    let mut entries = List::with_capacity(queue.len());
    for entry in queue.into_boxed() {
        entries.push(decode_entry(entry, bounds)?).expect(FITS);
    }
    let mut decoded_skips = List::with_capacity(skips.len());
    for skip in skips.into_boxed() {
        let w::FollowSkip { after, reason } = skip;
        decoded_skips.push(d::FollowSkip { after, reason }).expect(FITS);
    }
    let mut decoded_questions = List::with_capacity(questions.len());
    for question in questions.into_boxed() {
        let w::FollowQuestion { change, number, words, person, answer, relayed } = question;
        decoded_questions.push(d::FollowQuestion { change, number, words, person, answer, relayed }).expect(FITS);
    }
    Ok(d::FollowRow {
        project,
        follow,
        source: names::decode_repository(source),
        branch,
        heard,
        task,
        stopped,
        queue: entries.into_boxed(),
        skips: decoded_skips.into_boxed(),
        parity: decode_parity(parity),
        questions: decoded_questions.into_boxed(),
        pending: match pending {
            Some(act) => Some(decode_act(act)),
            None => None,
        },
    })
}
fn decode_files(bytes: &[u8], bounds: &Bounds) -> Result<Box<[Box<[u8]>]>, Undecodable> {
    let mut reader = Reader::new(bytes);
    let mut files = List::with_capacity(bounds.forge().follow_files);
    for _ in 0..=bounds.forge().follow_files {
        let Some(item) = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))? else {
            return Ok(files.into_boxed());
        };
        let file = w::AnswerName::decode(bounds.limits(), &mut Reader::new(&item)).or(Err(Undecodable::Malformed))?;
        files.push(file.name).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}

fn encode_act(act: &d::follows::FollowAct) -> w::FollowAct {
    use d::follows::FollowAct as A;
    match act {
        A::Start { after } => w::FollowAct::Start(w::FollowAt { after: *after }),
        A::Hold { after, reason } => w::FollowAct::Hold(w::FollowReasonAct { after: *after, reason: reason.clone() }),
        A::Skip { after, reason } => w::FollowAct::Skip(w::FollowReasonAct { after: *after, reason: reason.clone() }),
        A::Merge { afters } => {
            let mut commits = List::with_capacity(u32::try_from(afters.len()).expect(FITS));
            for after in afters {
                commits.push(*after).expect(FITS);
            }
            w::FollowAct::Merge(w::FollowMergeAct { afters: commits })
        }
    }
}
fn decode_act(act: w::FollowAct) -> d::follows::FollowAct {
    use d::follows::FollowAct as A;
    match act {
        w::FollowAct::Start(value) => A::Start { after: value.after },
        w::FollowAct::Hold(value) => {
            let p = value;
            A::Hold { after: p.after, reason: p.reason }
        }
        w::FollowAct::Skip(value) => {
            let p = value;
            A::Skip { after: p.after, reason: p.reason }
        }
        w::FollowAct::Merge(value) => A::Merge { afters: value.afters.into_boxed() },
    }
}
