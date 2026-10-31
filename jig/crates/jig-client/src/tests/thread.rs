use super::fixtures::{self, thread::*};
use crate::{
    contract::*,
    thread::{Line, Thread, Words, window::Edge},
};
use skein_lib::{List, Wall};
#[test]
fn activity_reads_newest_then_older_and_keeps_reverse_cursor_when_sliding() {
    let mut c = Thread::new(9, &crate::thread::Limits { lines: 3, bytes: 65536, text: 4096 });
    assert_eq!(c.moved(6), Some(Read::Activity { task: 9, before: None }));
    assert_eq!(c.moved(6), None);
    assert_eq!(c.accept(page(&[6, 5, 4], Some(b"older"), None)), None);
    assert_eq!(positions(&c).as_ref(), &[6, 5, 4]);
    assert_eq!(c.older(), Some(Read::Activity { task: 9, before: Some(cursor(b"older")) }));
    assert_eq!(c.accept(page(&[3, 2, 1], None, Some(b"newer"))), None);
    assert_eq!(positions(&c).as_ref(), &[3, 2, 1]);
    assert_eq!(c.lines.edges.before, Edge::More(cursor(b"newer")));
    assert_eq!(c.older(), None);
    assert_eq!(c.moved(7), None);
    assert_eq!(c.newer(), Some(Read::Activity { task: 9, before: Some(cursor(b"newer")) }));
    assert_eq!(c.accept(page(&[6, 5, 4], Some(b"older"), None)), Some(Read::Activity { task: 9, before: None }));
    assert_eq!(c.accept(page(&[7, 6, 5], Some(b"older"), None)), None);
    assert_eq!(positions(&c).as_ref(), &[7, 6, 5]);
}
#[test]
fn activity_document_movement_reads_only_new_lines_and_bridges_small_pages() {
    let mut c = Thread::new(9, &crate::thread::Limits { lines: 4, bytes: 65536, text: 4096 });
    assert!(c.moved(2).is_some());
    assert_eq!(c.accept(page(&[2, 1], None, None)), None);
    assert_eq!(c.moved(2), None);
    assert!(c.moved(5).is_some());
    assert_eq!(
        c.accept(page(&[5, 4], Some(b"tail"), None)),
        Some(Read::Activity { task: 9, before: Some(cursor(b"tail")) })
    );
    assert_eq!(c.accept(page(&[3, 2], Some(b"past"), Some(b"reverse"))), None);
    assert_eq!(positions(&c).as_ref(), &[5, 4, 3, 2]);
    assert_eq!(c.loaded, 5);
    assert_eq!(c.moved(5), None);
    assert!(c.moved(6).is_some());
    c.failed();
    assert!(c.moved(6).is_some());
    assert_eq!(
        c.accept(page(&[11, 10], Some(b"remaining"), None)),
        Some(Read::Activity { task: 9, before: Some(cursor(b"remaining")) })
    );
    assert_eq!(c.accept(page(&[9, 8, 7], Some(b"overflow"), Some(b"back"))), None);
    assert_eq!(positions(&c).as_ref(), &[11, 10]);
    assert_eq!(c.lines.edges.after, Edge::More(cursor(b"remaining")));
}

const FIRST_WORDS: crate::Key = crate::Key { asker: crate::Asker::JIG, number: 11 };
const SECOND_WORDS: crate::Key = crate::Key { asker: crate::Asker::JIG, number: 12 };
const FULL_WORDS: crate::Key = crate::Key { asker: crate::Asker::JIG, number: 13 };
const ARCHIVE_WORDS: crate::Key = crate::Key { asker: crate::Asker::JIG, number: 77 };
const ATTEMPT: u64 = 1;
const TURN: u32 = 1;
const MESSAGE: u64 = 41;
const FIRST_MESSAGE: u64 = 40;
const TURN_AT: Wall = Wall::from_nanos(17);
const FIRST_WORDS_AT: Wall = Wall::from_nanos(11);
const SECOND_WORDS_AT: Wall = Wall::from_nanos(13);
fn run() -> RunState {
    RunState { task: fixtures::SUBJECT_TASK, attempt: Some(ATTEMPT), committed_turn: 0, writing: None, call: None }
}
fn provisional(number: u32, words: &[u8]) -> RunText {
    RunText {
        writing: None,
        attempt: ATTEMPT,
        began: None,
        turn: ProvisionalTurnName { attempt: ATTEMPT, number, retry: 0 },
        from: 0,
        through: u32::try_from(words.len()).unwrap(),
        text: text(words),
        cut: false,
        call: None,
    }
}
fn commit(thread: &mut Thread, turn: TurnRead) {
    let mut page = page(&[], None, None);
    page.rows
        .push(ActivityLine {
            source: Box::new(OwnerActivityLine {
                task: 7,
                position: 1,
                at: TURN_AT,
                line: OwnerActivity::Turn {
                    reply: 1,
                    attempt: (TurnName { attempt: ATTEMPT, number: TURN }).attempt,
                    turn: (TurnName { attempt: ATTEMPT, number: TURN }).number,
                    read: None,
                },
            }),
            decoded: Some(ArchivedTurn {
                task: 7,
                name: TurnName { attempt: ATTEMPT, number: TURN },
                cumulative: 0,
                read: None,
                at: TURN_AT,
                turn: turn,
            }),
        })
        .unwrap();
    assert!(thread.moved(1).is_some());
    assert_eq!(thread.page(page), None);
}
#[test]
fn a_committed_turn_replaces_its_provisional_text() {
    let mut thread = model();
    thread.run(&run(), Some(&provisional(TURN, b"first")));
    thread.run(&run(), Some(&provisional(TURN + 1, b"second")));
    assert_eq!(thread.live.len(), 2);
    let turn = Turn {
        messages: List::with_capacity(0),
        used: Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        cut: false,
    };
    commit(&mut thread, TurnRead::Decoded(turn.clone()));
    assert_eq!(thread.live.len(), 1);
    let lines = thread.lines();
    assert_eq!(
        lines.as_slice(),
        &[
            Line::Turn { name: TurnName { attempt: ATTEMPT, number: TURN }, turn: &turn, at: TURN_AT },
            Line::Provisional { name: provisional(TURN + 1, b"").turn, text: b"second", cut: false }
        ]
    );
}
#[test]
fn a_live_turn_stays_until_its_staged_commit_merges() {
    let mut thread = model();
    assert!(thread.moved(2).is_some());
    assert_eq!(thread.page(page(&[2, 1], None, None)), None);
    thread.run(&run(), Some(&provisional(TURN, b"The answer")));
    assert!(thread.moved(4).is_some());
    let turn = empty_turn();
    let name = TurnName { attempt: ATTEMPT, number: TURN };
    let mut newest = page(&[4, 3], Some(b"older"), None);
    *newest.rows.get_mut(0).unwrap() = ActivityLine {
        source: Box::new(OwnerActivityLine {
            task: 7,
            position: 4,
            at: TURN_AT,
            line: OwnerActivity::Turn { reply: 4, attempt: (name).attempt, turn: (name).number, read: None },
        }),
        decoded: Some(ArchivedTurn {
            task: 7,
            name: name,
            cumulative: 0,
            read: None,
            at: TURN_AT,
            turn: TurnRead::Decoded(turn.clone()),
        }),
    };
    assert_eq!(
        thread.page(newest),
        Some(Read::Activity { task: fixtures::SUBJECT_TASK, before: Some(cursor(b"older")) })
    );
    assert_eq!(positions(&thread).as_ref(), &[2, 1]);
    assert_eq!(thread.fresh.rows.len(), 2);
    assert_eq!(thread.live.len(), 1);
    let live = Line::Provisional { name: provisional(TURN, b"").turn, text: b"The answer", cut: false };
    assert_eq!(thread.lines().as_slice(), &[Line::Activity { line: &row(1) }, Line::Activity { line: &row(2) }, live]);
    thread.run(&run(), Some(&provisional(TURN, b"The answer is")));
    let live = Line::Provisional { name: provisional(TURN, b"").turn, text: b"The answer is", cut: false };
    assert!(thread.lines().as_slice().contains(&live));
    assert_eq!(thread.page(page(&[2, 1], None, Some(b"newer"))), None);
    assert_eq!(positions(&thread).as_ref(), &[4, 3, 2, 1]);
    assert_eq!(
        thread.lines().as_slice(),
        &[
            Line::Activity { line: &row(1) },
            Line::Activity { line: &row(2) },
            Line::Activity { line: &row(3) },
            Line::Turn { name, turn: &turn, at: TURN_AT }
        ]
    );
    assert!(thread.live.is_empty());
}
#[test]
fn a_later_retry_replaces_the_same_provisional_turn() {
    let mut thread = model();
    let first = provisional(TURN, b"first");
    let mut retry = provisional(TURN, b"replacement");
    retry.turn.retry = 1;
    thread.run(&run(), Some(&first));
    thread.run(&run(), Some(&retry));
    thread.run(&run(), Some(&first));
    assert_eq!(thread.lines().as_slice(), &[Line::Provisional { name: retry.turn, text: b"replacement", cut: false }]);
}

#[test]
fn a_new_attempt_clears_every_provisional_turn() {
    let mut thread = model();
    thread.run(&run(), Some(&provisional(TURN, b"first")));
    thread.run(&run(), Some(&provisional(TURN + 1, b"second")));
    let mut next = run();
    next.attempt = Some(ATTEMPT + 1);
    thread.run(&next, None);
    assert!(thread.lines().is_empty());
}

#[test]
fn a_turn_that_does_not_decode_stays_a_line_saying_why() {
    let mut thread = model();
    commit(&mut thread, TurnRead::Undecodable { why: TurnProblem::Version { found: 99 } });
    let lines = thread.lines();
    assert_eq!(
        lines.as_slice(),
        &[Line::Undecoded {
            name: TurnName { attempt: ATTEMPT, number: TURN },
            why: TurnProblem::Version { found: 99 },
            at: TURN_AT
        }]
    );
}
#[test]
fn words_sent_in_a_compound_are_numbered_by_their_place() {
    let mut thread = model();
    assert!(thread.sending(fixtures::STOP, fixtures::PARTY, text(b"second operation"), 1));
    let by = thread.words.rows.get(0).unwrap().by.as_ref();
    assert_eq!(
        thread.lines().as_slice(),
        &[Line::Words { by, words: b"second operation", state: Words::BeingSent, at: None }]
    );
    thread.answered(
        fixtures::STOP,
        &Answer::Done(Outcome::Owner(OwnerOutcome::Compound {
            project: fixtures::PROJECT,
            operations: 2,
            first_message: FIRST_MESSAGE,
        })),
    );
    assert_eq!(thread.words.rows.get(0).unwrap().state, Words::Sent { message: MESSAGE });
    let by = thread.words.rows.get(0).unwrap().by.as_ref();
    assert_eq!(
        thread.lines().as_slice(),
        &[Line::Words { by, words: b"second operation", state: Words::Sent { message: MESSAGE }, at: None }]
    );
}

#[test]
fn owner_archive_reference_keeps_committed_read_and_local_author_separate_from_raw_sender() {
    use crate::contract::{OwnerActivity, OwnerActivityLine, OwnerMessageKind, OwnerRequester};
    use crate::thread::words::{KeptWords, Words};
    let mut kept = KeptWords::new(2);
    let request = ARCHIVE_WORDS;
    assert!(kept.begin(request, Some(party(7)), text(b"local")));
    assert!(kept.sent(request, 0));
    let line = |position, source| ActivityLine {
        source: Box::new(OwnerActivityLine { task: 8, position, at: Wall::EPOCH, line: source }),
        decoded: None,
    };
    let mut rows = List::with_capacity(2);
    rows.push(line(2, OwnerActivity::Turn { reply: 9, attempt: 3, turn: 4, read: Some(0) })).unwrap();
    rows.push(line(
        1,
        OwnerActivity::Words {
            message: 0,
            by: OwnerRequester::Task(7),
            kind: OwnerMessageKind::Question,
            words: text(b"task words"),
        },
    ))
    .unwrap();
    let page = ActivityPage { rows, before: None, after: None, cut: false };
    kept.observe(&page);
    assert_eq!(kept.rows.len(), 1);
    assert_eq!(kept.rows.get(0).unwrap().state, Words::Read { message: 0 });
    assert_eq!(kept.rows.get(0).unwrap().by.as_ref().unwrap().name.bytes(), b"Pat");
    assert_eq!(page.rows.get(1).unwrap().word_message().unwrap().1.bytes(), b"task words");
    assert!(page.rows.get(0).unwrap().decoded_turn().is_none());
    assert_eq!(page.rows.get(0).unwrap().committed_turn(), Some(TurnName { attempt: 3, number: 4 }));
}

fn party(number: u64) -> Party {
    Party { number, kind: PartyKind::Person, name: text(b"Pat") }
}
fn empty_turn() -> Turn {
    Turn {
        messages: List::with_capacity(0),
        used: Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        cut: false,
    }
}

#[test]
fn a_thread_keeps_the_newest_text_within_its_bounds_and_names_its_loss() {
    let mut thread = Thread::new(fixtures::SUBJECT_TASK, &crate::thread::Limits { lines: 1, bytes: 2, text: 2 });
    thread.run(&run(), Some(&provisional(TURN, b"first")));
    thread.run(&run(), Some(&provisional(TURN + 1, b"second")));
    let lines = thread.lines();
    assert_eq!(
        lines.as_slice(),
        &[Line::Left { lines: 1 }, Line::Provisional { name: provisional(TURN + 1, b"").turn, text: b"nd", cut: true }]
    );
}
