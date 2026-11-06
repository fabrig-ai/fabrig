use super::fixtures::{self, thread::*};
use crate::{
    contract::*,
    thread::{Line, Thread, Words, window::Edge},
};
use skein_lib::{List, Wall};

// Two 193-byte lines fit: line, message and block rooms, plus one text byte.
const WINDOW_BYTES: u64 = 512;
const SIZED_LINES: u32 = 8;
fn sized_window() -> Thread {
    bounded_window(WINDOW_BYTES, SIZED_LINES)
}
fn bounded_window(bytes: u64, lines: u32) -> Thread {
    Thread::new(fixtures::SUBJECT_TASK, &crate::thread::Limits { lines, bytes, text: 100, live: 100 })
}
fn window_at_end(positions: &[u64], older: Option<&[u8]>) -> Thread {
    let mut thread = sized_window();
    thread.moved(*positions.first().unwrap()).expect("initial read");
    assert_eq!(thread.page(sized_page(positions, older, None)), None);
    thread
}
#[test]
fn a_window_lets_whole_lines_go_past_its_bytes() {
    let mut thread = window_at_end(&[6, 5], Some(b"older"));
    assert_eq!(positions(&thread).as_ref(), &[6, 5]);
    assert!(thread.older().is_some());
    assert_eq!(thread.page(sized_page(&[4, 3], Some(b"past"), None)), None);
    assert_eq!(positions(&thread).as_ref(), &[4, 3]);
}
#[test]
fn bytes_keep_the_line_at_the_bound_and_drop_it_one_beyond() {
    // Each sized line weighs 193, so two meet 386 exactly.
    for (bytes, expected) in [(386, &[2, 1][..]), (385, &[1][..])] {
        let mut thread = bounded_window(bytes, u32::try_from(bytes / u64::from(LINE_ROOM)).unwrap());
        assert!(thread.moved(3).is_some());
        assert_eq!(thread.page(sized_page(&[3, 2], Some(b"older"), None)), None);
        assert!(thread.older().is_some());
        assert_eq!(thread.page(sized_page(&[1], None, None)), None);
        assert_eq!(positions(&thread).as_ref(), expected);
    }
}
#[test]
fn new_tail_staging_stops_at_its_bytes() {
    let mut thread = window_at_end(&[2, 1], None);
    assert!(thread.moved(8).is_some());
    assert_eq!(
        thread.page(sized_page(&[8, 7], Some(b"bridge"), None)),
        Some(Read::Activity { task: fixtures::SUBJECT_TASK, before: Some(cursor(b"bridge")) })
    );
    assert_eq!(thread.page(sized_page(&[6, 5], Some(b"past"), None)), None);
    assert_eq!(positions(&thread).as_ref(), &[8, 7]);
    assert_eq!(thread.lines.edges.after, Edge::More(cursor(b"bridge")));
}
#[test]
fn reading_older_lets_the_newest_go_and_reading_newer_the_oldest() {
    for (bytes, lines) in [(WINDOW_BYTES, SIZED_LINES), (65536, 2)] {
        let mut thread = bounded_window(bytes, lines);
        assert!(thread.moved(6).is_some());
        assert_eq!(thread.page(sized_page(&[6, 5], Some(b"older"), None)), None);
        assert!(thread.older().is_some());
        assert_eq!(thread.page(sized_page(&[4, 3], Some(b"past"), None)), None);
        assert_eq!(thread.lines.edges.before, Edge::Dropped);
        assert_eq!(
            thread.newer(),
            Some(Read::ActivityBeside { task: fixtures::SUBJECT_TASK, position: 4, side: Side::Later })
        );
        assert_eq!(thread.page(sized_page(&[5, 6], None, None)), None);
        assert_eq!(positions(&thread).as_ref(), &[6, 5]);
        assert_eq!(thread.lines.edges.after, Edge::Dropped);
        assert_eq!(thread.lines.edges.before, Edge::End);
    }
}
#[test]
fn the_end_comes_back_after_enough_older() {
    // Three forward pages return two lines each.
    const RETURNED_LINES: u32 = 6;
    let mut thread = window_at_end(&[8, 7], Some(b"older"));
    for positions in [&[6, 5], &[4, 3], &[2, 1]] {
        assert!(thread.older().is_some());
        assert_eq!(thread.page(sized_page(positions, Some(b"older"), None)), None);
    }
    let mut returned = List::with_capacity(RETURNED_LINES);
    for (positions, after) in [([3, 4], Some(b"more".as_slice())), ([5, 6], Some(b"more".as_slice())), ([7, 8], None)] {
        let held = thread.lines.rows.get(0).unwrap().source.position;
        assert_eq!(
            thread.newer(),
            Some(Read::ActivityBeside { task: fixtures::SUBJECT_TASK, position: held, side: Side::Later })
        );
        assert_eq!(thread.page(sized_page(&positions, None, after)), None);
        let expected = sized_page(&positions, None, after);
        assert!(thread.lines.rows.iter().rev().eq(expected.rows.iter()), "lines return whole");
        for row in thread.lines.rows.iter().rev() {
            returned.push(row.source.position).unwrap();
        }
    }
    assert_eq!(returned.as_slice(), &[3, 4, 5, 6, 7, 8]);
    assert_eq!(thread.lines.edges.before, Edge::End);
    assert_eq!(thread.loaded, 8);
    assert_eq!(thread.moved(9), Some(Read::Activity { task: fixtures::SUBJECT_TASK, before: None }));
}
#[test]
fn a_page_alone_past_the_bound_stays() {
    let mut thread = window_at_end(&[5, 4], Some(b"older"));
    assert!(thread.older().is_some());
    assert_eq!(thread.page(sized_page(&[3, 2, 1], None, None)), None);
    assert_eq!(positions(&thread).as_ref(), &[3, 2, 1]);
    assert_eq!(thread.lines.edges.before, Edge::Dropped);
}
#[test]
fn older_past_a_dropped_edge_reads_beside_the_oldest_line() {
    let mut thread = window_at_end(&[4, 3], Some(b"older"));
    assert!(thread.moved(6).is_some());
    assert_eq!(
        thread.page(sized_page(&[6, 5], Some(b"bridge"), None)),
        Some(Read::Activity { task: fixtures::SUBJECT_TASK, before: Some(cursor(b"bridge")) })
    );
    assert_eq!(thread.page(sized_page(&[4, 3], None, None)), None);
    assert_eq!(
        thread.older(),
        Some(Read::ActivityBeside { task: fixtures::SUBJECT_TASK, position: 5, side: Side::Earlier })
    );
}
#[test]
fn latest_replaces_the_window() {
    let mut thread = window_at_end(&[4, 3], Some(b"older"));
    assert!(thread.older().is_some());
    assert_eq!(thread.page(sized_page(&[2, 1], None, None)), None);
    assert_eq!(thread.latest(), Some(Read::Activity { task: fixtures::SUBJECT_TASK, before: None }));
    assert_eq!(thread.latest(), None);
    assert_eq!(thread.page(sized_page(&[6], Some(b"latest"), None)), None);
    assert_eq!(positions(&thread).as_ref(), &[6]);
    assert_eq!(thread.loaded, 6);
    assert_eq!(thread.lines.edges.after, Edge::More(cursor(b"latest")));
    assert_eq!(thread.lines.edges.before, Edge::End);
}
#[test]
fn a_window_that_let_its_end_go_reads_no_tail_and_draws_nothing_live() {
    let mut thread = window_at_end(&[4, 3], Some(b"older"));
    thread.run(&run(), Some(&provisional(TURN + 10, b"first")));
    assert!(thread.older().is_some());
    assert_eq!(thread.page(sized_page(&[2, 1], None, None)), None);
    assert_eq!(thread.moved(5), None);
    assert_eq!(thread.committed, 5);
    assert!(thread.sending(fixtures::STOP, fixtures::PARTY, text(b"in the gap"), 0));
    thread.answered(
        fixtures::STOP,
        &Answer::Done(Outcome::Document(DocumentOutcome::Said { task: fixtures::SUBJECT_TASK, message: MESSAGE })),
    );
    thread.run(&run(), Some(&provisional(TURN + 10, b"first grows")));
    for line in thread.lines().into_boxed() {
        match line {
            Line::Turn { .. } => {}
            Line::Words { .. } | Line::Provisional { .. } | Line::Undecoded { .. } | Line::Activity { .. } => {
                panic!("only committed turns are shown while the newest end is absent");
            }
        }
    }
    assert!(thread.newer().is_some());
    assert_eq!(thread.page(sized_page(&[3, 4], None, Some(b"more"))), None);
    assert!(thread.newer().is_some());
    assert_eq!(thread.page(sized_page(&[5], None, None)), None);
    let shown = thread.lines();
    assert!(shown.as_slice().contains(&Line::Words {
        by: thread.words.rows.get(0).unwrap().by.as_ref(),
        words: b"in the gap",
        state: Words::Sent { message: MESSAGE },
        at: None,
    }));
    assert!(shown.as_slice().contains(&Line::Provisional {
        name: provisional(TURN + 10, b"").turn,
        text: b"first grows",
        cut: false,
    }));
    assert_eq!(thread.loaded, 5);
}
#[test]
fn read_failed_says_which_end_until_it_is_read() {
    use crate::thread::window::End;
    let mut thread = sized_window();
    assert!(thread.moved(4).is_some());
    thread.failed();
    assert_eq!(thread.read_failed, Some(End::Before));
    assert!(thread.latest().is_some());
    assert_eq!(thread.read_failed, None);
    assert_eq!(thread.page(sized_page(&[4, 3], Some(b"older"), None)), None);
    assert!(thread.older().is_some());
    thread.failed();
    assert_eq!(thread.read_failed, Some(End::After));
    assert!(thread.latest().is_some());
    assert_eq!(thread.read_failed, Some(End::After));
    assert_eq!(thread.page(sized_page(&[4, 3], Some(b"older"), None)), None);
    assert!(thread.older().is_some());
    assert_eq!(thread.read_failed, None);
    assert_eq!(thread.page(sized_page(&[2, 1], None, None)), None);
    assert!(thread.newer().is_some());
    thread.failed();
    assert_eq!(thread.read_failed, Some(End::Before));
    assert!(thread.newer().is_some());
    assert_eq!(thread.read_failed, None);
    thread.read_failed = Some(End::Before);
    assert_eq!(thread.page(sized_page(&[3, 4], None, None)), None);
    assert_eq!(thread.read_failed, None);
}
#[test]
fn a_line_weighs_its_bytes_and_its_room() {
    assert_eq!(LINE_ROOM, 64);
    assert_eq!(SIZED_LINES, u32::try_from(WINDOW_BYTES / u64::from(LINE_ROOM)).unwrap());
    assert_eq!(row(1).weight(), 64);
    let mut page = sized_page(&[1], None, None);
    let line = page.rows.get_mut(0).unwrap();
    assert_eq!(line.weight(), 193);
    let TurnRead::Decoded(turn) = &mut line.decoded.as_mut().unwrap().turn else { panic!("decoded fixture") };
    let mut blocks = List::with_capacity(3);
    blocks.push(Block::Text(text(b"ab"))).unwrap();
    blocks.push(Block::Text(text(b"cd"))).unwrap();
    blocks.push(Block::Text(text(b"ef"))).unwrap();
    turn.messages.get_mut(0).unwrap().blocks = blocks;
    assert_eq!(line.weight(), 326);
    let mut blocks = List::with_capacity(3);
    blocks.push(Block::Text(text(b"Working"))).unwrap();
    blocks.push(Block::Opaque { dialect: text(b"provider"), size: u64::MAX }).unwrap();
    let mut objects = List::with_capacity(1);
    objects.push(ObjectKey::Task(fixtures::SUBJECT_TASK)).unwrap();
    let mut path = List::with_capacity(2);
    path.push(Box::from(b"src".as_slice())).unwrap();
    path.push(Box::from(b"lib.rs".as_slice())).unwrap();
    let mut parts = List::with_capacity(2);
    parts.push(ApplicationPart::Resource { connector: 1, path }).unwrap();
    parts.push(ApplicationPart::Connector { connector: 1 }).unwrap();
    blocks
        .push(Block::Call(Call {
            number: 1,
            tool: text(b"read"),
            input: text(b"file"),
            outcome: Some(text(b"done")),
            failed: false,
            objects,
            parts,
        }))
        .unwrap();
    let mut messages = List::with_capacity(2);
    messages.push(Message { speaker: Speaker::Assistant, blocks }).unwrap();
    let mut blocks = List::with_capacity(1);
    blocks.push(Block::Result { call: 1, summary: text(b"complete"), failed: false }).unwrap();
    messages.push(Message { speaker: Speaker::Tool, blocks }).unwrap();
    let archive = line.decoded.as_mut().unwrap();
    let TurnRead::Decoded(turn) = &mut archive.turn else { panic!("decoded fixture") };
    turn.messages = messages;
    // 44 retained text/path bytes plus 10 structural rooms, regardless of opaque size.
    assert_eq!(line.weight(), 684);
    line.decoded = Some(ArchivedTurn {
        turn: TurnRead::Undecodable { why: TurnProblem::TooLarge { bytes: u64::MAX } },
        ..line.decoded.take().unwrap()
    });
    assert_eq!(line.weight(), 64);
    line.source.line = Activity::Words {
        message: MESSAGE,
        by: Requester::Task(fixtures::SUBJECT_TASK),
        kind: MessageKind::Words,
        words: text(b"hello"),
    };
    assert_eq!(line.weight(), 69);
    let mut fields = List::with_capacity(1);
    fields.push(ResultField { name: Box::from(b"key".as_slice()), value: Box::from(b"value".as_slice()) }).unwrap();
    line.source.line = Activity::Heard(Word {
        number: MESSAGE,
        from: Requester::Task(fixtures::SUBJECT_TASK),
        kind: MessageKind::Result(WordResult::Verdict { code: 1, fields }),
        words: text(b"hello"),
        at: Wall::EPOCH,
        hits: 0,
        eligible: false,
    });
    assert_eq!(line.weight(), 205);
}
#[test]
fn activity_reads_newest_then_older_and_returns_beside_the_window() {
    let mut c = Thread::new(9, &crate::thread::Limits { lines: 3, bytes: 65536, text: 4096, live: 4096 });
    assert_eq!(c.moved(6), Some(Read::Activity { task: 9, before: None }));
    assert_eq!(c.moved(6), None);
    assert_eq!(c.accept(page(&[6, 5, 4], Some(b"older"), None)), None);
    assert_eq!(positions(&c).as_ref(), &[6, 5, 4]);
    assert_eq!(c.older(), Some(Read::Activity { task: 9, before: Some(cursor(b"older")) }));
    assert_eq!(c.accept(page(&[3, 2, 1], None, Some(b"newer"))), None);
    assert_eq!(positions(&c).as_ref(), &[3, 2, 1]);
    assert_eq!(c.lines.edges.before, Edge::Dropped);
    assert_eq!(c.older(), None);
    assert_eq!(c.moved(7), None);
    assert_eq!(c.newer(), Some(Read::ActivityBeside { task: 9, position: 3, side: Side::Later }));
    assert_eq!(c.accept(page(&[6, 5, 4], Some(b"older"), None)), Some(Read::Activity { task: 9, before: None }));
    assert_eq!(c.accept(page(&[7, 6, 5], Some(b"older"), None)), None);
    assert_eq!(positions(&c).as_ref(), &[7, 6, 5]);
}
#[test]
fn activity_document_movement_reads_only_new_lines_and_bridges_small_pages() {
    let mut c = Thread::new(9, &crate::thread::Limits { lines: 4, bytes: 65536, text: 4096, live: 4096 });
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

const ARCHIVE_WORDS: crate::Key = crate::Key { asker: crate::Asker::JIG, number: 77 };
const ATTEMPT: u64 = 1;
const TURN: u32 = 1;
const MESSAGE: u64 = 41;
const FIRST_MESSAGE: u64 = 40;
const TURN_AT: Wall = Wall::from_nanos(17);
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
        .push(DecodedActivityLine {
            source: Box::new(ActivityLine {
                task: 7,
                position: 1,
                at: TURN_AT,
                line: Activity::Turn {
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
                turn,
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
    *newest.rows.get_mut(0).unwrap() = DecodedActivityLine {
        source: Box::new(ActivityLine {
            task: 7,
            position: 4,
            at: TURN_AT,
            line: Activity::Turn { reply: 4, attempt: (name).attempt, turn: (name).number, read: None },
        }),
        decoded: Some(ArchivedTurn {
            task: 7,
            name,
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
        &Answer::Done(Outcome::Document(DocumentOutcome::Compound {
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
fn archive_reference_keeps_committed_read_and_local_author_separate_from_raw_sender() {
    use crate::contract::{Activity, ActivityLine, MessageKind, Requester};
    use crate::thread::words::{KeptWords, Words};
    let mut kept = KeptWords::new(2);
    let request = ARCHIVE_WORDS;
    assert!(kept.begin(request, Some(party(7)), text(b"local")));
    assert!(kept.sent(request, 0));
    let line = |position, source| DecodedActivityLine {
        source: Box::new(ActivityLine { task: 8, position, at: Wall::EPOCH, line: source }),
        decoded: None,
    };
    let mut rows = List::with_capacity(2);
    rows.push(line(2, Activity::Turn { reply: 9, attempt: 3, turn: 4, read: Some(0) })).unwrap();
    rows.push(line(
        1,
        Activity::Words { message: 0, by: Requester::Task(7), kind: MessageKind::Question, words: text(b"task words") },
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
fn a_thread_keeps_the_newest_text_within_its_bounds() {
    let mut thread =
        Thread::new(fixtures::SUBJECT_TASK, &crate::thread::Limits { lines: 1, bytes: 2, text: 2, live: 2 });
    thread.run(&run(), Some(&provisional(TURN, b"first")));
    thread.run(&run(), Some(&provisional(TURN + 1, b"second")));
    let lines = thread.lines();
    assert_eq!(
        lines.as_slice(),
        &[Line::Provisional { name: provisional(TURN + 1, b"").turn, text: b"se", cut: true }]
    );
}

#[test]
fn a_live_line_past_a_fields_text_arrives_whole_within_its_bound() {
    let mut thread =
        Thread::new(fixtures::SUBJECT_TASK, &crate::thread::Limits { lines: 4, bytes: 64, text: 2, live: 16 });
    for words in [b"ab".as_slice(), b"abcdefgh"] {
        thread.run(&run(), Some(&provisional(TURN, words)));
        assert_eq!(
            thread.lines().as_slice(),
            &[Line::Provisional { name: provisional(TURN, b"").turn, text: words, cut: false }]
        );
    }
}

#[test]
fn a_live_line_past_its_bound_keeps_its_start_on_a_character_boundary() {
    for (bound, expected) in [(0, ""), (1, "a"), (2, "a"), (3, "aé"), (5, "aé"), (6, "aé界")] {
        let mut thread =
            Thread::new(fixtures::SUBJECT_TASK, &crate::thread::Limits { lines: 4, bytes: 64, text: 2, live: bound });
        for words in ["aé界z", "aé界zmore"] {
            thread.run(&run(), Some(&provisional(TURN, words.as_bytes())));
            assert_eq!(
                thread.lines().as_slice(),
                &[Line::Provisional { name: provisional(TURN, b"").turn, text: expected.as_bytes(), cut: true }],
                "bound={bound}"
            );
        }
    }
}

#[test]
fn a_persons_words_stay_bounded_by_text_beside_a_larger_live_bound() {
    let mut thread =
        Thread::new(fixtures::SUBJECT_TASK, &crate::thread::Limits { lines: 4, bytes: 64, text: 2, live: 16 });
    assert!(!thread.sending(fixtures::STOP, fixtures::PARTY, text(b"abc"), 0));
    assert!(thread.sending(fixtures::STOP, fixtures::PARTY, text(b"ab"), 0));
    assert_eq!(thread.words.rows.get(0).unwrap().words.bytes(), b"ab");
}
