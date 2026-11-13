//! Follow records (temper/docs/design/protocol/store.md, 2, family 42).
use super::{Bounds, FITS, Undecodable, names, w};
use alloc::boxed::Box;
use skein_lib::{List, Reader};
use temper_engine_domain_forge_vocabulary as d;
fn reason(bytes: &[u8]) -> w::FollowReason {
    w::FollowReason::new(&w::CEILINGS, w::FollowReasonParts { reason: Box::from(bytes) }).expect(FITS)
}
fn encode_state(state: &d::FollowState) -> w::FollowState {
    match state {
        d::FollowState::NotMade => w::FollowState::NotMade,
        d::FollowState::WaitsWord => w::FollowState::WaitsWord,
        d::FollowState::HeldByOwner(words) => w::FollowState::HeldByOwner(reason(words)),
        d::FollowState::WaitsFor(number) => {
            w::FollowState::WaitsFor(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::FollowState::Producing => w::FollowState::Producing,
        d::FollowState::Held(words) => w::FollowState::Held(reason(words)),
        d::FollowState::Ended(words) => w::FollowState::Ended(reason(words)),
    }
}
fn decode_state(state: w::FollowState) -> d::FollowState {
    match state {
        w::FollowState::NotMade => d::FollowState::NotMade,
        w::FollowState::WaitsWord => d::FollowState::WaitsWord,
        w::FollowState::HeldByOwner(words) => d::FollowState::HeldByOwner(words.into_parts().reason),
        w::FollowState::WaitsFor(number) => d::FollowState::WaitsFor(number.into_parts().number),
        w::FollowState::Producing => d::FollowState::Producing,
        w::FollowState::Held(words) => d::FollowState::Held(words.into_parts().reason),
        w::FollowState::Ended(words) => d::FollowState::Ended(words.into_parts().reason),
    }
}
fn encode_entry(entry: &d::FollowEntry) -> w::FollowEntry {
    let mut commits = List::with_capacity(u32::try_from(entry.commits.len()).expect(FITS));
    for commit in &entry.commits {
        commits
            .push(
                w::FollowCommit::new(
                    &w::CEILINGS,
                    w::FollowCommitParts { id: commit.id, message: commit.message.clone() },
                )
                .expect(FITS),
            )
            .expect(FITS);
    }
    let files = match &entry.files {
        Some(files) => Some(
            w::FollowFiles::new(&w::CEILINGS, w::FollowFilesParts { files: names::encode_piece(files) }).expect(FITS),
        ),
        None => None,
    };
    w::FollowEntry::new(
        &w::CEILINGS,
        w::FollowEntryParts {
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
        },
    )
    .expect(FITS)
}
fn decode_entry(entry: w::FollowEntry, bounds: &Bounds) -> Result<d::FollowEntry, Undecodable> {
    let w::FollowEntryParts {
        before,
        after,
        pull,
        commits,
        more_commits,
        files,
        more_files,
        change,
        state,
        word,
        remake,
    } = entry.into_parts();
    let mut decoded = List::with_capacity(commits.len());
    for commit in commits.into_boxed() {
        let w::FollowCommitParts { id, message } = commit.into_parts();
        decoded.push(d::FollowCommit { id, message }).expect(FITS);
    }
    Ok(d::FollowEntry {
        before,
        after,
        pull,
        commits: decoded.into_boxed(),
        more_commits,
        files: match files {
            Some(files) => Some(decode_files(&files.into_parts().files, bounds)?),
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
        d::FollowVerdict::Failing(count) => {
            w::FollowVerdict::Failing(w::FollowFailing::new(&w::CEILINGS, w::FollowFailingParts { count }).expect(FITS))
        }
        d::FollowVerdict::Many => w::FollowVerdict::Many,
    };
    Some(
        w::FollowParity::new(
            &w::CEILINGS,
            w::FollowParityParts { source: parity.source, head: parity.head, verdict, run: parity.run.clone() },
        )
        .expect(FITS),
    )
}
fn decode_parity(parity: Option<w::FollowParity>) -> Option<d::FollowParity> {
    let w::FollowParityParts { source, head, verdict, run } = parity?.into_parts();
    Some(d::FollowParity {
        source,
        head,
        verdict: match verdict {
            w::FollowVerdict::Passed => d::FollowVerdict::Passed,
            w::FollowVerdict::Failing(count) => d::FollowVerdict::Failing(count.into_parts().count),
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
        skips
            .push(
                w::FollowSkip::new(&w::CEILINGS, w::FollowSkipParts { after: skip.after, reason: skip.reason.clone() })
                    .expect(FITS),
            )
            .expect(FITS);
    }
    let mut questions = List::with_capacity(u32::try_from(row.questions.len()).expect(FITS));
    for question in &row.questions {
        questions
            .push(
                w::FollowQuestion::new(
                    &w::CEILINGS,
                    w::FollowQuestionParts {
                        change: question.change,
                        number: question.number,
                        words: question.words.clone(),
                        person: question.person,
                        relayed: question.relayed,
                        answer: question.answer.clone(),
                    },
                )
                .expect(FITS),
            )
            .expect(FITS);
    }
    w::FollowRow::new(
        &w::CEILINGS,
        w::FollowRowParts {
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
        },
    )
    .expect(FITS)
}
pub(super) fn decode(row: w::FollowRow, bounds: &Bounds) -> Result<d::FollowRow, Undecodable> {
    let w::FollowRowParts {
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
    } = row.into_parts();
    let mut entries = List::with_capacity(queue.len());
    for entry in queue.into_boxed() {
        entries.push(decode_entry(entry, bounds)?).expect(FITS);
    }
    let mut decoded_skips = List::with_capacity(skips.len());
    for skip in skips.into_boxed() {
        let w::FollowSkipParts { after, reason } = skip.into_parts();
        decoded_skips.push(d::FollowSkip { after, reason }).expect(FITS);
    }
    let mut decoded_questions = List::with_capacity(questions.len());
    for question in questions.into_boxed() {
        let w::FollowQuestionParts { change, number, words, person, answer, relayed } = question.into_parts();
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
        files.push(file.into_parts().name).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}

fn encode_act(act: &d::follows::FollowAct) -> w::FollowAct {
    use d::follows::FollowAct as A;
    match act {
        A::Start { after } => {
            w::FollowAct::Start(w::FollowAt::new(&w::CEILINGS, w::FollowAtParts { after: *after }).expect(FITS))
        }
        A::Hold { after, reason } => w::FollowAct::Hold(
            w::FollowReasonAct::new(&w::CEILINGS, w::FollowReasonActParts { after: *after, reason: reason.clone() })
                .expect(FITS),
        ),
        A::Skip { after, reason } => w::FollowAct::Skip(
            w::FollowReasonAct::new(&w::CEILINGS, w::FollowReasonActParts { after: *after, reason: reason.clone() })
                .expect(FITS),
        ),
        A::Merge { afters } => {
            let mut commits = List::with_capacity(u32::try_from(afters.len()).expect(FITS));
            for after in afters {
                commits.push(*after).expect(FITS);
            }
            w::FollowAct::Merge(
                w::FollowMergeAct::new(&w::CEILINGS, w::FollowMergeActParts { afters: commits }).expect(FITS),
            )
        }
    }
}
fn decode_act(act: w::FollowAct) -> d::follows::FollowAct {
    use d::follows::FollowAct as A;
    match act {
        w::FollowAct::Start(value) => A::Start { after: *value.after() },
        w::FollowAct::Hold(value) => {
            let p = value.into_parts();
            A::Hold { after: p.after, reason: p.reason }
        }
        w::FollowAct::Skip(value) => {
            let p = value.into_parts();
            A::Skip { after: p.after, reason: p.reason }
        }
        w::FollowAct::Merge(value) => A::Merge { afters: value.into_parts().afters.into_boxed() },
    }
}
