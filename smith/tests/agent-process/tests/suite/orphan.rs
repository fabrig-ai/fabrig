//! The process settles itself when its last host pipe is lost.

use skein_fake_llm_domain::api;
use skein_io::kernel::Exit;
use smith_agent_process_world::{TraceCase, World, charter, wire::edit_charter};
use smith_channel::kind;
use smith_events::{Event, Outcome, Verdict};

const ENDED: &str = "smith: the run could not answer: the channel ended before the answer";
const WRITE_FAILED: &str = "smith: the run could not answer: the channel's output failed";

fn exits_without_an_answer(world: &mut World, line: &str) {
    assert!(world.settle(), "the world settles, not at its iteration limit");
    assert_eq!(world.exit(), Some(Exit::Code(1)));
    assert!(world.answer().is_none());
    let (lost, queries) = world.loss();
    let exited = world.exited_at().expect("agent exited on the world's clock");
    let elapsed = exited.saturating_since(lost);
    let bound = smith_agent_profile::orphan_exit(&world.link()).expect("derived orphan exit");
    assert!((lost..=lost.saturating_add(bound)).contains(&exited), "loss to exit: {elapsed:?}; bound: {bound:?}");
    assert_eq!(world.peer_queries().len(), queries, "no provider call starts after the loss");
    let mut ends = world.records().iter().filter_map(|record| match &record.event {
        Event::SessionEnded(end) => Some(end),
        _ => None,
    });
    assert!(!ends.next().expect("session end").aborted, "the cancellation settled inside its grace");
    assert!(ends.next().is_none());
    let errors = String::from_utf8(world.errors()).expect("bounded operator words");
    assert_eq!(errors.lines().last(), Some(line));
    assert!(!world.signalled());
    world.assert_agent_clean();
    eprintln!("loss to exit: {elapsed:?}; orphan_exit: {bound:?}");
}

fn live_text(mid_write: bool) -> World {
    const NEXT_PIECE: &[u8] = b"next live piece";
    let mut world = World::new(8, &charter());
    world.observe_facts(TraceCase::Accepted, 8);
    world.ask(true, false, false);
    let mut pieces = vec![api::Piece { text: b"held text".as_slice().into(), hold: None }];
    if mid_write {
        pieces
            .push(api::Piece { text: b"write to the missing reader".as_slice().into(), hold: Some(NEXT_PIECE.into()) });
        let interval = smith_agent_process_world::profile().policy.live_interval;
        world.release_after(NEXT_PIECE, 1, interval.checked_add(interval).expect("two live intervals"));
    }
    pieces.push(api::Piece { text: b"unreleased text".as_slice().into(), hold: Some(World::ORPHAN_ANSWER.into()) });
    world.script(Box::new([smith_agent_world::fixture::says(
        Box::new([api::Line::Pieces { pieces: pieces.into_boxed_slice() }]),
        1,
    )]));
    world
}

fn one_cancelled_completion(world: &World) {
    let responses: Vec<_> = world
        .records()
        .iter()
        .filter_map(|record| match &record.event {
            Event::ResponseCompleted(response) => Some(&response.outcome),
            _ => None,
        })
        .collect();
    assert_eq!(responses, [&Outcome::Cancelled]);
    assert_eq!(world.peer_queries().len(), 1);
}

#[test]
fn both_ends_gone_with_text_held_cancel_the_only_completion() {
    let mut world = live_text(false);
    world.hang_up_with_live_text();
    exits_without_an_answer(&mut world, ENDED);
    one_cancelled_completion(&world);
    assert!(world.pieces().is_empty(), "the short live piece had not flushed");
    assert_eq!(world.lost_live_bytes(), skein_lib::len_u64(b"held text".len()));
}

#[test]
fn losing_only_the_hosts_reading_end_abandons_the_other_pipe() {
    let mut world = live_text(true);
    world.close_reading_end();
    exits_without_an_answer(&mut world, WRITE_FAILED);
    assert!(world.writing_end_at_exit(), "the parent did not close its other end to help the agent exit");
    one_cancelled_completion(&world);
}

#[test]
fn losing_the_host_during_a_command_stops_its_group() {
    let mut world = World::new(18, &charter());
    world.configured_environment();
    world.observe_facts(TraceCase::Accepted, 8);
    world.script(Box::new([smith_agent_world::fixture::calls(
        Box::new([api::Line::Call {
            name: b"shell".as_slice().into(),
            arguments: br#"{"command":"hang"}"#.as_slice().into(),
        }]),
        1,
    )]));
    world.hang_up_during_command();
    exits_without_an_answer(&mut world, ENDED);
    assert_eq!(world.command_environments().len(), 1);
    assert_eq!(world.peer_queries().len(), 1);
    let mut tools = world.records().iter().filter(|record| matches!(record.event, Event::ToolStarted(_)));
    assert!(
        matches!(tools.next().map(|record| &record.event), Some(Event::ToolStarted(start)) if start.tool.as_ref() == b"shell")
    );
    assert!(tools.next().is_none(), "no new tool call starts after the loss");
    let mut terminals = world.records().iter().filter(|record| matches!(record.event, Event::ToolCompleted(_)));
    assert!(
        matches!(terminals.next().map(|record| &record.event), Some(Event::ToolCompleted(end)) if end.verdict == Verdict::Cancelled)
    );
    assert!(terminals.next().is_none(), "one terminal for the cancelled group");
}

#[test]
fn losing_the_host_while_waiting_for_its_next_message_ends_the_run() {
    let charter = edit_charter(&charter(), |parts| {
        parts.mode = smith_charter::Mode::Open;
        smith_agent_process_world::wire::budget(parts, 8, 20);
        parts.waiting = skein_lib::Duration::from_secs(60);
        let mut tools = parts.tools.clone();
        tools.wait = true;
        parts.tools = tools;
    });
    let mut world = World::new(28, &charter);
    world.observe_facts(TraceCase::Accepted, 8);
    world.initial_message(17, b"request", b"Wait for my next message.");
    world.script(Box::new([smith_agent_world::fixture::calls(
        Box::new([api::Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() }]),
        1,
    )]));
    world.hang_up_while_waiting();
    exits_without_an_answer(&mut world, ENDED);
    assert_eq!(world.peer_queries().len(), 1);
    assert!(world.observed().iter().any(|frame| frame.kind == kind::WAITING));
    assert!(world.peer_queries()[0].messages.iter().flat_map(|message| &message.parts).any(|part| {
        matches!(part, api::Part::Text { text } if skein_lib::bytes::find(text, b"Wait for my next message.").is_some())
    }), "the only message was read into the provider prompt before Waiting");
}
