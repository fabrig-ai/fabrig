use crate::route::box_link::{Link, Out, Refused, Step};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Token};
const REQUEST: Token = Token::new(91);
const OTHER_REQUEST: Token = Token::new(92);

const LINE_BYTES: u32 = 32;
const FORGE: u16 = 7;
const OTHER: u16 = 8;
// Enough for the longest script below, including its ask's three outputs.
const OUTPUTS: u32 = 32;

fn asked() -> (Link, Queue<Out>) {
    let mut link = Link::new(LINE_BYTES, true);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.ask(FORGE, REQUEST, &mut out);
    discard(&mut out);
    (link, out)
}
fn discard(out: &mut Queue<Out>) {
    for _ in 0..out.len() {
        drop(out.pop());
    }
}
fn named(text: &[u8]) -> Out {
    Out::Named(Box::from(text))
}
fn accepted() -> [Out; 3] {
    [
        Out::Write(Box::from(b"start 7\n".as_slice())),
        Out::SettingUp { forge: FORGE, step: None },
        Out::Answered { forge: FORGE, request: REQUEST, answer: Ok(()) },
    ]
}
fn tells(out: &mut Queue<Out>, expected: &[Out]) {
    for event in expected {
        assert_eq!(out.pop(), Some(event.clone()));
    }
    assert_eq!(out.pop(), None);
}
#[test]
fn an_ask_writes_start_and_sets_up_with_no_step() {
    let mut link = Link::new(LINE_BYTES, true);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.ask(FORGE, REQUEST, &mut out);
    tells(&mut out, &accepted());
}
#[test]
fn an_ask_is_held_until_started_or_failed_and_a_second_held_ask_is_busy() {
    for ending in [b"started 7\n".as_slice(), b"failed 7 prepare exited 1\n"] {
        let (mut link, mut out) = asked();
        link.ask(OTHER, OTHER_REQUEST, &mut out);
        tells(
            &mut out,
            &[
                Out::SettingUp { forge: OTHER, step: None },
                Out::Answered { forge: OTHER, request: OTHER_REQUEST, answer: Ok(()) },
            ],
        );
        link.ask(9, REQUEST, &mut out);
        tells(
            &mut out,
            &[
                named(b"box link: fault: a setup ask while busy or closed"),
                Out::Answered { forge: 9, request: REQUEST, answer: Err(Refused::Busy) },
            ],
        );
        link.read(b"step 7 prepare\n", &mut out);
        tells(&mut out, &[Out::SettingUp { forge: FORGE, step: Some(Step::Prepare) }]);
        link.read(ending, &mut out);
        let event = if ending.starts_with(b"started") {
            Out::Started { forge: FORGE }
        } else {
            Out::Failed { forge: FORGE, why: b"prepare exited 1".as_slice().into() }
        };
        tells(&mut out, &[event, Out::Write(b"start 8\n".as_slice().into())]);
        link.read(b"started 8\n", &mut out);
        tells(&mut out, &[Out::Started { forge: OTHER }]);
        link.ask(FORGE, REQUEST, &mut out);
        tells(&mut out, &accepted());
    }
}
#[test]
fn a_held_ask_is_dropped_when_the_pipe_closes() {
    let (mut link, mut out) = asked();
    link.ask(OTHER, OTHER_REQUEST, &mut out);
    discard(&mut out);
    link.closed(&mut out);
    tells(&mut out, &[named(b"box link: the box closed the pipe"), Out::Closed]);
    link.read(b"started 7\n", &mut out);
    tells(&mut out, &[]);
}
#[test]
fn a_fault_ask_needs_only_one_forges_diagnostic_slot() {
    let mut service = super::fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    let occupied = engine.notes.room() - 1;
    for _ in 0..occupied {
        engine.notes.push(crate::Note::ForgeUnconfigured { number: 1 });
    }
    assert_eq!(engine.notes.room(), 1);
    let mut out = Queue::with_capacity(OUTPUTS);
    engine.box_link.ask(1, REQUEST, &mut out);
    crate::route::box_link::drain(engine, &mut out);
    engine.box_link.ask(1, OTHER_REQUEST, &mut out);
    crate::route::box_link::drain(engine, &mut out);
    for _ in 0..occupied {
        engine.notes.pop().unwrap();
    }
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::BoxLink { line: b"box link: fault: a setup ask while busy or closed".as_slice().into() })
    );
    assert!(engine.notes.is_empty());
    let Some(temper_engine_domain::Event::Forge(event)) = engine.events.iter().last() else {
        panic!("the fault ask is answered")
    };
    assert_eq!(
        **event,
        temper_engine_domain_forge::Event::SetUpAnswered {
            forge: 1,
            request: OTHER_REQUEST,
            answer: temper_engine_domain_forge::SetUpAnswer::Refused(temper_engine_domain_forge::SetUpRefusal::Busy),
        }
    );
}
#[test]
fn each_line_for_the_asked_forge_is_told() {
    let rows = [
        (b"step 7 fetch\n".as_slice(), Out::SettingUp { forge: FORGE, step: Some(Step::Fetch) }),
        (b"step 7 prepare\n", Out::SettingUp { forge: FORGE, step: Some(Step::Prepare) }),
        (b"step 7 run\n", Out::SettingUp { forge: FORGE, step: Some(Step::Run) }),
        (b"started 7\n", Out::Started { forge: FORGE }),
        (
            b"failed 7 prepare 7 exited 1\n",
            Out::Failed { forge: FORGE, why: Box::from(b"prepare 7 exited 1".as_slice()) },
        ),
        (b"failed 7  raw\xff  why \r\n", Out::Failed { forge: FORGE, why: Box::from(b" raw\xff  why \r".as_slice()) }),
    ];
    for (line, event) in rows {
        let (mut link, mut out) = asked();
        link.read(line, &mut out);
        tells(&mut out, &[event]);
    }
}
#[test]
fn a_line_for_another_forge_or_with_none_asked_is_named_and_dropped() {
    let rows = [
        (b"step 8 fetch\n".as_slice(), b"box link: dropped step for forge 8, not asked".as_slice()),
        (b"started 8\n", b"box link: dropped started for forge 8, not asked"),
        (b"failed 8 busy\n", b"box link: dropped failed for forge 8, not asked"),
    ];
    for (line, message) in rows {
        for ask in [false, true] {
            let mut link = Link::new(LINE_BYTES, true);
            let mut out = Queue::with_capacity(OUTPUTS);
            if ask {
                link.ask(FORGE, REQUEST, &mut out);
                discard(&mut out);
            }
            link.read(line, &mut out);
            tells(&mut out, &[named(message)]);
            if ask {
                link.read(b"started 7\n", &mut out);
                tells(&mut out, &[Out::Started { forge: FORGE }]);
            }
        }
    }
}
#[test]
fn an_unknown_word_step_or_number_is_named_and_dropped() {
    let lines: &[&[u8]] = &[
        b"unknown 7\n",
        b"step 7 stop\n",
        b"started x\n",
        b"started 65536\n",
        b"started +7\n",
        b"started -7\n",
        b"started \n",
        b"started 7 \n",
        b"started  7\n",
        b"step 7  fetch\n",
        b"step 7 fetch \n",
        b"step 7\n",
        b"failed 7\n",
        b"failed 7 \n",
        b"started 7\r\n",
        b"\n",
        b"\xff 7\n",
    ];
    for line in lines {
        let (mut link, mut out) = asked();
        link.read(line, &mut out);
        tells(&mut out, &[named(b"box link: dropped an unknown line")]);
        link.read(b"started 7\n", &mut out);
        tells(&mut out, &[Out::Started { forge: FORGE }]);
    }
}
#[test]
fn a_line_at_the_bound_is_read_and_one_byte_past_it_is_skipped_to_its_line_feed() {
    // "failed 7 " + 22 reason bytes + LF = LINE_BYTES.
    let rows = [
        (
            b"failed 7 1234567890123456789012\n".as_slice(),
            Out::Failed { forge: FORGE, why: Box::from(b"1234567890123456789012".as_slice()) },
        ),
        (b"failed 7 12345678901234567890123\n", named(b"box link: dropped a line past box.line_bytes (32)")),
    ];
    for (line, event) in rows {
        let (mut link, mut out) = asked();
        // The overflow ends in a later read, after the bound was filled.
        let (before_lf, lf) = line.split_at_checked(line.len() - 1).expect("fixture ends in LF");
        link.read(before_lf, &mut out);
        tells(&mut out, &[]);
        link.read(lf, &mut out);
        tells(&mut out, &[event]);
        link.ask(FORGE, REQUEST, &mut out);
        discard(&mut out);
        link.read(b"step 7 run\n", &mut out);
        tells(&mut out, &[Out::SettingUp { forge: FORGE, step: Some(Step::Run) }]);
    }
}
#[test]
fn lines_split_and_joined_across_reads_are_read_alike() {
    const SCRIPT: &[u8] = b"step 7 fetch\nstep 7 prepare\nstep 7 run\nfailed 7 12345678901234567890123\nstarted 7\n";
    const ODD_SPLIT: usize = 17; // Inside prepare's step line.
    let (mut whole, mut whole_out) = asked();
    let (mut bytes, mut bytes_out) = asked();
    let (mut chunks, mut chunks_out) = asked();
    whole.read(SCRIPT, &mut whole_out);
    for byte in SCRIPT {
        bytes.read(core::slice::from_ref(byte), &mut bytes_out);
    }
    let (first, second) = SCRIPT.split_at_checked(ODD_SPLIT).expect("split inside script");
    chunks.read(first, &mut chunks_out);
    chunks.read(second, &mut chunks_out);
    let expected = [
        Out::SettingUp { forge: FORGE, step: Some(Step::Fetch) },
        Out::SettingUp { forge: FORGE, step: Some(Step::Prepare) },
        Out::SettingUp { forge: FORGE, step: Some(Step::Run) },
        named(b"box link: dropped a line past box.line_bytes (32)"),
        Out::Started { forge: FORGE },
    ];
    tells(&mut whole_out, &expected);
    tells(&mut bytes_out, &expected);
    tells(&mut chunks_out, &expected);
}
#[test]
fn the_pipe_close_is_carried_once_to_the_root() {
    let (mut link, mut out) = asked();
    link.read(b"step 7 pre", &mut out);
    link.closed(&mut out);
    tells(&mut out, &[named(b"box link: the box closed the pipe"), Out::Closed]);
    link.read(b"pare\nstarted 7\n", &mut out);
    link.closed(&mut out);
    tells(&mut out, &[]);
}
#[test]
fn an_idle_pipe_closes_once_without_failing_a_forge() {
    let mut link = Link::new(LINE_BYTES, true);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.closed(&mut out);
    link.closed(&mut out);
    tells(&mut out, &[named(b"box link: the box closed the pipe"), Out::Closed]);
}
#[test]
fn forge_numbers_at_both_u16_bounds_are_read() {
    for (forge, line) in [(0, b"started 0\n".as_slice()), (u16::MAX, b"started 65535\n")] {
        let mut link = Link::new(LINE_BYTES, true);
        let mut out = Queue::with_capacity(OUTPUTS);
        link.ask(forge, REQUEST, &mut out);
        discard(&mut out);
        link.read(line, &mut out);
        tells(&mut out, &[Out::Started { forge }]);
    }
}

#[test]
fn stop_writes_its_line_and_stopped_tells_the_root_nothing() {
    let mut link = Link::new(LINE_BYTES, true);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.stop(FORGE, &mut out);
    tells(&mut out, &[Out::Write(b"stop 7\n".as_slice().into())]);
    link.read(b"stopped 8\n", &mut out);
    tells(&mut out, &[named(b"box link: dropped stopped for forge 8, not asked")]);
    link.read(b"started 7\n", &mut out);
    tells(&mut out, &[named(b"box link: dropped started for forge 7, not asked")]);
    link.read(b"stopped 7\n", &mut out);
    tells(&mut out, &[]);
    link.ask(FORGE, REQUEST, &mut out);
    tells(&mut out, &accepted());
}
#[test]
fn stop_is_held_behind_start_and_failed_is_only_logged() {
    let (mut link, mut out) = asked();
    link.stop(OTHER, &mut out);
    tells(&mut out, &[]);
    link.read(b"started 7\n", &mut out);
    tells(&mut out, &[Out::Started { forge: FORGE }, Out::Write(b"stop 8\n".as_slice().into())]);
    link.read(b"failed 8 run exited 1\n", &mut out);
    tells(&mut out, &[named(b"box link: stop failed for forge 8: run exited 1")]);
    link.ask(FORGE, REQUEST, &mut out);
    tells(&mut out, &accepted());
}
#[test]
fn stop_cancels_a_held_demo_start_and_preserves_another_held_start() {
    let (mut link, mut out) = asked();
    link.ask(OTHER, OTHER_REQUEST, &mut out);
    discard(&mut out);
    link.stop(OTHER, &mut out);
    link.read(b"started 7\n", &mut out);
    tells(&mut out, &[Out::Started { forge: FORGE }, Out::Write(b"stop 8\n".as_slice().into())]);
    link.read(b"stopped 8\n", &mut out);
    tells(&mut out, &[]);
    let (mut link, mut out) = asked();
    link.ask(OTHER, OTHER_REQUEST, &mut out);
    discard(&mut out);
    link.stop(FORGE, &mut out);
    link.read(b"started 7\n", &mut out);
    tells(&mut out, &[Out::Started { forge: FORGE }, Out::Write(b"stop 7\n".as_slice().into())]);
    link.read(b"stopped 7\n", &mut out);
    tells(&mut out, &[Out::Write(b"start 8\n".as_slice().into())]);
}
#[test]
fn closing_the_pipe_drops_active_and_held_stops() {
    for active in [false, true] {
        let mut link = Link::new(LINE_BYTES, true);
        let mut out = Queue::with_capacity(OUTPUTS);
        if !active {
            link.ask(OTHER, OTHER_REQUEST, &mut out);
        }
        link.stop(FORGE, &mut out);
        discard(&mut out);
        link.closed(&mut out);
        tells(&mut out, &[named(b"box link: the box closed the pipe"), Out::Closed]);
        link.read(b"stopped 7\nstarted 8\n", &mut out);
        link.stop(FORGE, &mut out);
        tells(&mut out, &[]);
    }
}

#[test]
fn demo_end_diagnostics_reach_serves_named_notes() {
    let mut service = super::fixtures::demo_service();
    let engine = service.engine.as_mut().unwrap();
    for _ in 0..engine.notes.len() {
        engine.notes.pop().unwrap();
    }
    let line = b"demo end: chat 7 not closed: no first owner configured".as_slice();
    let cohort = temper_engine_domain_forge::demo_act_bound(&engine.env.limits.forge).unwrap();
    for _ in 0..cohort {
        crate::route::request(
            engine,
            temper_engine_domain::Request::Party(temper_engine_domain::PartyRequest::Deliver(
                temper_engine_domain::Delivery::DemoNote { line: line.into() },
            )),
        );
    }
    for _ in 0..cohort {
        assert_eq!(engine.notes.pop(), Some(crate::Note::Demo { line: line.into() }));
    }
    assert!(engine.notes.is_empty());
}

const FILE_FD_BASE: i32 = 9_000;

// The system fixture's kernel handles worker IO. These are the box link's
// directory and atomic store operations, driven in the same turn.
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the script observes box file operations; file_answer rejects other operations"
)]
pub(crate) fn turn(stand: &mut super::fixtures::system::SystemStand) {
    use skein_io::kernel::{Complete, Fd, Op};
    crate::iterate(&mut stand.service, skein_lib::Time::ZERO, skein_lib::Wall::EPOCH);
    for _ in 0..stand.service.submissions.len() {
        let submit = stand.service.submissions.pop().unwrap();
        let file = match &submit.kind {
            Op::Open { root, .. } => *root == Fd::new(90) || root.raw() >= FILE_FD_BASE,
            Op::MakeDirectory { dir, .. } | Op::Rename { from_dir: dir, .. } | Op::Remove { dir, .. } => {
                dir.raw() >= FILE_FD_BASE
            }
            Op::Close { fd } | Op::Stat { fd } | Op::Sync { fd, .. } | Op::Write { fd, .. } => {
                *fd == Fd::new(90) || fd.raw() >= FILE_FD_BASE
            }
            _ => false,
        };
        if file {
            let fd = Fd::new(i32::try_from(submit.op.raw()).unwrap().checked_add(FILE_FD_BASE).unwrap());
            if let Op::Close { fd } = &submit.kind {
                stand.kernel.closed.push(*fd).unwrap();
            }
            let result = file_answer(&submit.kind, fd);
            stand.service.completions.push(Complete { op: submit.op, kind: submit.kind, result });
        } else {
            stand.service.submissions.push(submit);
        }
    }
    stand.kernel.answer(&mut stand.service);
}

pub(crate) fn until_hosting(stand: &mut super::fixtures::system::SystemStand) {
    for _ in 0_u16..256 {
        stand.service.engine.as_mut().unwrap().events.push(temper_engine_domain::Event::Resume);
        turn(stand);
        if super::fixtures::system::hello(&stand.service).is_some() {
            return;
        }
    }
    panic!("the worker's hello reaches the engine");
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the script observes box file operations; file_answer rejects other operations"
)]
fn file_answer(
    kind: &skein_io::kernel::Op,
    next: skein_io::kernel::Fd,
) -> Result<skein_io::kernel::Done, skein_io::kernel::Error> {
    use skein_io::kernel::{Done, Error, Kind, Op, OpenHow, Stat};
    match kind {
        Op::Open { how: OpenHow::ReadNoFollow, path, .. } if path.as_ref() != b"groups" && path.as_ref() != b"box" => {
            Err(Error::NotFound)
        }
        Op::Open { how: OpenHow::CreateNoFollow { mode }, .. } => {
            assert_eq!(*mode, Some(0o600), "the group file's temporary is private");
            Ok(Done::Fd(next))
        }
        Op::Open { .. } => Ok(Done::Fd(next)),
        Op::MakeDirectory { mode, .. } => {
            assert_eq!(*mode, 0o700);
            Ok(Done::Nothing)
        }
        Op::Stat { .. } => Ok(Done::Stat(Stat { kind: Kind::Directory, size: 0, mode: 0o700, owner: 0, links: 1 })),
        Op::Write { bytes, .. } => Ok(Done::Count(u32::try_from(bytes.len()).unwrap())),
        Op::Sync { .. } | Op::Close { .. } | Op::Rename { .. } | Op::Remove { .. } => Ok(Done::Nothing),
        other => panic!("unexpected group file operation: {other:?}"),
    }
}

fn record_service() -> crate::Service {
    use temper_engine_domain_forge::record as r;
    let mut service = super::fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    let mut record = super::fixtures::engine().seed[0].clone();
    record.number = FORGE;
    record.identity.number = u32::from(FORGE);
    record.files = None;
    record.setup = Some(r::Setup {
        directory: b"/data/owned".as_slice().into(),
        bot: r::Account { login: b"temper".as_slice().into(), email: b"temper@example.invalid".as_slice().into() },
        orgs: Box::new([]),
        asked: true,
        mode: r::SetupMode::Owned(r::OwnedEntry {
            public: None,
            fetch: r::Fetch { forgejo: false, runner: false, mirror: None },
            forgejo: b"/opt/forgejo".as_slice().into(),
            runner: None,
            listen: ([127, 0, 0, 1], 3000).into(),
            person: r::Account {
                login: b"person".as_slice().into(),
                email: b"person@example.invalid".as_slice().into(),
            },
        }),
    });
    crate::forge::table::kept(engine, record);
    assert!(crate::forge::table::record(engine, FORGE).unwrap().setup.is_some());
    engine.box_link = Link::new(LINE_BYTES, true);
    engine.box_link.files = Some(crate::route::group_file::Files::new(
        skein_io::kernel::Fd::new(90),
        0,
        skein_lib::Duration::from_secs(1),
        7,
    ));
    service
}

#[test]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the script observes box file operations; file_answer rejects other operations"
)]
fn a_record_forge_start_writes_its_file_first() {
    use skein_io::kernel::{Complete, Fd, Op};
    let mut service = record_service();
    let engine = service.engine.as_mut().unwrap();
    let mut out = Queue::with_capacity(OUTPUTS);
    engine.box_link.ask(FORGE, REQUEST, &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let mut made = List::with_capacity(2);
    let mut written = false;
    let mut synced = 0_u8;
    let mut renamed = false;
    for index in 0_u16..64 {
        assert!(engine.box_link.write.is_none(), "start waits for the whole store");
        crate::route::group_file::drive(engine, skein_lib::Time::ZERO, false);
        if let Some(line) = &engine.box_link.write {
            assert_eq!(line.as_ref(), b"start 7\n");
            assert!(written && renamed && synced >= 3);
            assert_eq!(made.as_slice(), &[Box::from(b"box".as_slice()), Box::from(b"groups".as_slice())]);
            return;
        }
        let files = engine.box_link.files.as_mut().unwrap();
        for _ in 0..files.submitted.len() {
            let submit = files.submitted.pop().unwrap();
            match &submit.kind {
                Op::MakeDirectory { name, .. } => {
                    made.push(name.clone()).unwrap();
                }
                Op::Write { bytes, .. } => {
                    let directories = jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
                    let file = temper_forge_setup::file::read(
                        bytes,
                        FORGE,
                        &temper_forge_setup::limits::small(),
                        &directories,
                    )
                    .unwrap();
                    assert_eq!(file.entry.directory.as_ref(), b"/data/owned");
                    written = true;
                }
                Op::Rename { to, .. } => {
                    assert!(written && synced >= 2, "directory and temporary are synced before rename");
                    assert_eq!(to.as_ref(), b"7");
                    renamed = true;
                }
                Op::Sync { .. } => {
                    synced = synced.checked_add(1).unwrap();
                }
                _ => {}
            }
            let result = file_answer(&submit.kind, Fd::new(i32::from(index).checked_add(FILE_FD_BASE).unwrap()));
            files.complete(Complete { op: submit.op, kind: submit.kind, result });
        }
    }
    panic!("the stored group file releases start");
}

#[test]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the script observes box file operations; file_answer rejects other operations"
)]
fn a_failed_write_answers_set_up_failed_and_sends_no_start() {
    use skein_io::kernel::{Complete, Error, Fd, Op};
    for failure in ["open", "directory", "store", "rename", "sync", "file sync", "directory sync"] {
        let mut service = record_service();
        let engine = service.engine.as_mut().unwrap();
        let mut out = Queue::with_capacity(OUTPUTS);
        engine.box_link.ask(FORGE, REQUEST, &mut out);
        crate::route::box_link::drain(engine, &mut out);
        let mut failed = false;
        let mut written = false;
        let mut renamed = false;
        for index in 0_u16..96 {
            crate::route::group_file::drive(engine, skein_lib::Time::ZERO, false);
            assert!(engine.box_link.write.is_none());
            let files = engine.box_link.files.as_mut().unwrap();
            for _ in 0..files.submitted.len() {
                let submit = files.submitted.pop().unwrap();
                let inject = !failed
                    && match &submit.kind {
                        Op::Open { .. } => failure == "open",
                        Op::MakeDirectory { .. } => failure == "directory",
                        Op::Write { .. } => failure == "store",
                        Op::Rename { .. } => failure == "rename",
                        Op::Sync { .. } => {
                            failure == "sync"
                                || failure == "file sync" && written && !renamed
                                || failure == "directory sync" && renamed
                        }
                        _ => false,
                    };
                match &submit.kind {
                    Op::Write { .. } => written = true,
                    Op::Rename { .. } => renamed = true,
                    _ => {}
                }
                let result = if inject {
                    failed = true;
                    Err(Error::Permission)
                } else {
                    file_answer(&submit.kind, Fd::new(i32::from(index).checked_add(FILE_FD_BASE).unwrap()))
                };
                files.complete(Complete { op: submit.op, kind: submit.kind, result });
            }
        }
        assert!(failed, "failure reached {failure}");
        let mut failures = 0_u8;
        for event in &engine.events {
            if let temper_engine_domain::Event::Forge(event) = event
                && let temper_engine_domain_forge::Event::Connection { forge, state } = &**event
                && *forge == FORGE
                && *state
                    == (temper_engine_domain_forge::Connection::SetUpFailed {
                        why: b"group file not written".as_slice().into(),
                    })
            {
                failures = failures.checked_add(1).unwrap();
            }
        }
        assert_eq!(failures, 1, "one failed setup answer for {failure}");
    }
}

#[test]
fn a_group_file_write_holds_the_links_one_ask_rule() {
    let mut service = record_service();
    let engine = service.engine.as_mut().unwrap();
    let mut out = Queue::with_capacity(OUTPUTS);
    engine.box_link.ask(FORGE, REQUEST, &mut out);
    tells(
        &mut out,
        &[
            Out::GroupFile { forge: FORGE },
            Out::SettingUp { forge: FORGE, step: None },
            Out::Answered { forge: FORGE, request: REQUEST, answer: Ok(()) },
        ],
    );
    engine.box_link.ask(OTHER, OTHER_REQUEST, &mut out);
    tells(
        &mut out,
        &[
            Out::SettingUp { forge: OTHER, step: None },
            Out::Answered { forge: OTHER, request: OTHER_REQUEST, answer: Ok(()) },
        ],
    );
    engine.box_link.ask(FORGE, REQUEST, &mut out);
    tells(
        &mut out,
        &[
            named(b"box link: fault: a setup ask while busy or closed"),
            Out::Answered { forge: FORGE, request: REQUEST, answer: Err(Refused::Busy) },
        ],
    );
    engine.box_link.read(b"started 7\n", &mut out);
    tells(&mut out, &[named(b"box link: dropped started for forge 7, not asked")]);
    engine.box_link.file_written(false, &mut out);
    tells(
        &mut out,
        &[
            Out::Failed { forge: FORGE, why: b"group file not written".as_slice().into() },
            Out::GroupFile { forge: OTHER },
        ],
    );
}

#[test]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the script observes box file operations; file_answer rejects other operations"
)]
fn a_group_file_write_closes_its_descriptors_when_the_service_ends() {
    use skein_io::kernel::{Complete, Error, Fd, Op};
    let mut service = record_service();
    let engine = service.engine.as_mut().unwrap();
    let mut out = Queue::with_capacity(OUTPUTS);
    engine.box_link.ask(FORGE, REQUEST, &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let mut closing = false;
    let mut closed = List::with_capacity(32);
    for index in 0_u16..96 {
        crate::route::group_file::drive(engine, skein_lib::Time::ZERO, closing);
        assert!(engine.box_link.write.is_none());
        let files = engine.box_link.files.as_mut().unwrap();
        if files.closed() {
            assert!(closing);
            assert!(closed.as_slice().contains(&Fd::new(90)));
            assert_eq!(closed.len(), 10, "all transient and held directory descriptors close");
            return;
        }
        for _ in 0..files.submitted.len() {
            let submit = files.submitted.pop().unwrap();
            match &submit.kind {
                Op::Write { .. } => closing = true,
                Op::Close { fd } => {
                    closed.push(*fd).unwrap();
                }
                _ => {}
            }
            let result = if let Op::Cancel { .. } = &submit.kind {
                Err(Error::TooLate)
            } else {
                file_answer(&submit.kind, Fd::new(i32::from(index).checked_add(FILE_FD_BASE).unwrap()))
            };
            files.complete(Complete { op: submit.op, kind: submit.kind, result });
        }
    }
    panic!("group file IO closes on service shutdown");
}

#[test]
fn a_connected_record_under_the_box_starts_without_writing_a_group_file() {
    let mut service = super::fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    assert!(crate::forge::table::record(engine, 1).unwrap().setup.is_none());
    engine.box_link = Link::new(LINE_BYTES, true);
    engine.box_link.files = Some(crate::route::group_file::Files::new(
        skein_io::kernel::Fd::new(90),
        0,
        skein_lib::Duration::from_secs(1),
        7,
    ));
    let mut out = Queue::with_capacity(OUTPUTS);
    engine.box_link.ask(1, REQUEST, &mut out);
    crate::route::box_link::drain(engine, &mut out);
    assert_eq!(engine.box_link.write.as_deref(), Some(b"start 1\n".as_slice()));
    assert!(engine.box_link.files.as_ref().unwrap().submitted.is_empty());
}
