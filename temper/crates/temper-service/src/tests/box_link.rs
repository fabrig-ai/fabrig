use crate::route::box_link::{Link, Out, Refused, Step};
use alloc::boxed::Box;
use skein_lib::{Queue, Token};
const REQUEST: Token = Token::new(91);
const OTHER_REQUEST: Token = Token::new(92);

const LINE_BYTES: u32 = 32;
const FORGE: u16 = 7;
const OTHER: u16 = 8;
// Enough for the longest script below, including its ask's three outputs.
const OUTPUTS: u32 = 32;

fn asked() -> (Link, Queue<Out>) {
    let mut link = Link::new(LINE_BYTES, true, 4);
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
    let mut link = Link::new(LINE_BYTES, true, 4);
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
            let mut link = Link::new(LINE_BYTES, true, 4);
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
    let mut link = Link::new(LINE_BYTES, true, 4);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.closed(&mut out);
    link.closed(&mut out);
    tells(&mut out, &[named(b"box link: the box closed the pipe"), Out::Closed]);
}
#[test]
fn forge_numbers_at_both_u16_bounds_are_read() {
    for (forge, line) in [(0, b"started 0\n".as_slice()), (u16::MAX, b"started 65535\n")] {
        let mut link = Link::new(LINE_BYTES, true, 4);
        let mut out = Queue::with_capacity(OUTPUTS);
        link.ask(forge, REQUEST, &mut out);
        discard(&mut out);
        link.read(line, &mut out);
        tells(&mut out, &[Out::Started { forge }]);
    }
}

#[test]
fn stop_writes_its_line_and_stopped_tells_the_root_nothing() {
    let mut link = Link::new(LINE_BYTES, true, 4);
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
        let mut link = Link::new(LINE_BYTES, true, 4);
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
fn act_diagnostics_reach_serves_named_notes() {
    let mut service = super::fixtures::demo_service();
    let engine = service.engine.as_mut().unwrap();
    for _ in 0..engine.notes.len() {
        engine.notes.pop().unwrap();
    }
    let line = b"forge removal: task 7 not cancelled: no first owner configured".as_slice();
    let cohort = engine.env.limits.forge.tasks.checked_add(1).unwrap();
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

fn group_request(service: &mut crate::Service, entry: &[u8]) {
    crate::route::request(
        service.engine.as_mut().unwrap(),
        temper_engine_domain::Request::SetUpGroup { forge: 1, request: REQUEST, entry: entry.into() },
    );
}
#[test]
fn a_set_up_group_with_an_entry_writes_its_file_then_asks_start() {
    let mut service = super::fixtures::ready();
    let entry = b"{\"mode\":\"owned\"}";
    group_request(&mut service, entry);
    assert!(service.engine.as_ref().unwrap().box_link.write.is_none());
    let write = service.take_group_write().expect("group write before start");
    assert_eq!(write.forge, 1);
    assert_eq!(write.text, temper_forge_setup::setup::group_settings(entry, 1).unwrap());
    assert!(service.take_group_write().is_none());
    service.group_written(99, Ok(()));
    assert!(service.engine.as_ref().unwrap().box_link.write.is_none());
    service.group_written(1, Ok(()));
    assert_eq!(service.engine.as_ref().unwrap().box_link.write.as_deref(), Some(b"start 1\n".as_slice()));
    service.group_written(1, Ok(()));
}
#[test]
fn a_failed_group_write_fails_the_group_without_asking_start() {
    use temper_engine_domain_forge::{Connection, Event};
    for entry in [b"{\"mode\":\"owned\"}".as_slice(), b"invalid"] {
        let mut service = super::fixtures::ready();
        group_request(&mut service, entry);
        if let Some(write) = service.take_group_write() {
            service.group_written(write.forge, Err(5_i32));
        }
        let engine = service.engine.as_mut().unwrap();
        assert!(engine.box_link.write.is_none());
        let mut failed = 0_u32;
        for event in &engine.events {
            if let temper_engine_domain::Event::Forge(event) = event
                && let Event::Connection { forge: 1, state: Connection::SetUpFailed { why } } = event.as_ref()
            {
                assert!(why.starts_with(b"group settings write"));
                failed += 1;
            }
        }
        assert_eq!(failed, 1);
    }
}
#[test]
fn an_empty_entry_asks_start_at_once() {
    let mut service = super::fixtures::ready();
    group_request(&mut service, b"");
    assert!(service.take_group_write().is_none());
    assert_eq!(service.engine.as_ref().unwrap().box_link.write.as_deref(), Some(b"start 1\n".as_slice()));
}
#[test]
fn remove_is_sent_answered_and_routed_as_group_removed() {
    let mut service = super::fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    engine.box_link = Link::new(LINE_BYTES, true, 4);
    crate::route::request(engine, temper_engine_domain::Request::RemoveGroup { forge: FORGE });
    assert_eq!(engine.box_link.write.take().as_deref(), Some(b"remove 7\n".as_slice()));
    let mut out = Queue::with_capacity(OUTPUTS);
    engine.box_link.read(b"removed 8\n", &mut out);
    tells(&mut out, &[named(b"box link: dropped removed for forge 8, not asked")]);
    engine.box_link.read(b"removed 7\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let Some(temper_engine_domain::Event::Forge(event)) = engine.events.pop() else { panic!("root answer") };
    assert_eq!(*event, temper_engine_domain_forge::Event::GroupRemoved { forge: FORGE });
    assert!(engine.events.is_empty());
    engine.box_link.read(b"removed 7\n", &mut out);
    tells(&mut out, &[named(b"box link: dropped removed for forge 7, not asked")]);
}
#[test]
fn removes_wait_behind_a_live_ask_in_order() {
    let (mut link, mut out) = asked();
    link.remove(8, &mut out);
    link.remove(9, &mut out);
    tells(&mut out, &[]);
    link.read(b"started 7\n", &mut out);
    tells(&mut out, &[Out::Started { forge: 7 }, Out::Write(b"remove 8\n".as_slice().into())]);
    link.read(b"removed 8\n", &mut out);
    tells(&mut out, &[Out::Removed { forge: 8 }, Out::Write(b"remove 9\n".as_slice().into())]);
    link.read(b"removed 9\n", &mut out);
    tells(&mut out, &[Out::Removed { forge: 9 }]);
}
#[test]
fn a_failed_remove_names_it_and_sends_no_event() {
    for (reply, message) in [
        (b"failed 7 not removed\n".as_slice(), b"box link: remove failed for forge 7: not removed".as_slice()),
        (b"failed 7 busy\n", b"box link: remove failed for forge 7: busy"),
    ] {
        let mut link = Link::new(LINE_BYTES, true, 4);
        let mut out = Queue::with_capacity(OUTPUTS);
        link.remove(FORGE, &mut out);
        tells(&mut out, &[Out::Write(b"remove 7\n".as_slice().into())]);
        link.read(reply, &mut out);
        tells(&mut out, &[named(message)]);
        link.remove(FORGE, &mut out);
        tells(&mut out, &[Out::Write(b"remove 7\n".as_slice().into())]);
    }
}
#[test]
fn a_remove_with_the_link_ended_is_dropped_and_named() {
    for piped in [false, true] {
        let mut link = Link::new(LINE_BYTES, piped, 4);
        let mut out = Queue::with_capacity(OUTPUTS);
        if piped {
            link.closed(&mut out);
            discard(&mut out);
        }
        link.remove(FORGE, &mut out);
        tells(&mut out, &[named(b"box link: dropped remove for forge 7, link ended")]);
    }
}
#[test]
fn a_start_failed_no_settings_fails_the_group() {
    let (mut link, mut out) = asked();
    link.read(b"failed 7 no settings\n", &mut out);
    tells(&mut out, &[Out::Failed { forge: FORGE, why: b"no settings".as_slice().into() }]);
}

#[test]
fn the_remove_queue_holds_the_forge_bound_and_drops_overflow() {
    let mut link = Link::new(LINE_BYTES, true, 2);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.ask(FORGE, REQUEST, &mut out);
    discard(&mut out);
    link.remove(8, &mut out);
    link.remove(9, &mut out);
    tells(&mut out, &[]);
    link.remove(10, &mut out);
    tells(&mut out, &[named(b"box link: dropped remove, queue full")]);
    link.read(b"started 7\n", &mut out);
    tells(&mut out, &[Out::Started { forge: 7 }, Out::Write(b"remove 8\n".as_slice().into())]);
    link.read(b"failed 8 busy\n", &mut out);
    tells(
        &mut out,
        &[named(b"box link: remove failed for forge 8: busy"), Out::Write(b"remove 9\n".as_slice().into())],
    );
    link.closed(&mut out);
    tells(&mut out, &[named(b"box link: the box closed the pipe"), Out::Closed]);
    link.read(b"removed 9\n", &mut out);
    tells(&mut out, &[]);
}
