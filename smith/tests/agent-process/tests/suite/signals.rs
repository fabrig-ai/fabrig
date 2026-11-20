//! A signal leaves the live run to the scripted host's word.
use skein_fake_llm_domain::api;
use skein_lib::{Duration, Reader};
use smith_agent_process_world::{TraceCase, World, charter, wire::edit_charter};
use smith_channel::{Record, RunFailure, RunResult};

fn held() -> World {
    let charter = edit_charter(&charter(), |parts| parts.budget.turns = 2);
    let mut world = World::new(8, &charter);
    world.script(Box::new([smith_agent_world::fixture::says(
        Box::new([api::Line::Pieces {
            pieces: Box::new([
                api::Piece { text: b"committed ".as_slice().into(), hold: None },
                api::Piece { text: b"answer".as_slice().into(), hold: Some(b"signal".as_slice().into()) },
            ]),
        }]),
        1,
    )]));
    world.release_after(b"signal", 1, Duration::from_millis(100));
    world
}

fn notice(world: &World) -> smith_channel::Signalled {
    let records: Vec<_> =
        world.observed().iter().filter(|frame| frame.kind == smith_channel::kind::SIGNALLED).collect();
    assert_eq!(records.len(), 1, "one signal notice per live run");
    smith_channel::Signalled::decode(&smith_channel::CEILINGS, &mut Reader::new(&records[0].body))
        .expect("signal record")
}

#[test]
fn a_signalled_turn_is_committed_before_the_hosts_drain_ends_it() {
    let mut world = held();
    world.signal();
    world.signal();
    world.drain_after_signal();
    assert!(world.settle());
    let heard = notice(&world);
    assert_eq!(heard.signal, smith_channel::Signal::Terminate);
    assert_eq!(world.signals(), 2, "the second signal was separately sent after the notice");
    assert!(world.frames_at_release().iter().any(|frame| frame.kind == smith_channel::kind::SIGNALLED));
    assert!(
        !world.frames_at_release().iter().any(|frame| frame.kind == smith_channel::kind::ANSWER),
        "the signal did not end the turn"
    );
    assert!(world.peer_replied());
    let turns: Vec<_> = world.observed().iter().filter(|frame| frame.kind == smith_channel::kind::TURN).collect();
    assert_eq!(turns.len(), 1, "the held answer was committed as one turn");
    let envelope =
        smith_channel::Turn::decode(&smith_channel::CEILINGS, &mut Reader::new(&turns[0].body)).expect("turn envelope");
    let turn = smith_transcript::Turn::decode(&smith_transcript::CEILINGS, &mut Reader::new(envelope.body.as_ref()))
        .expect("committed transcript");
    assert!(turn.messages.iter().any(|message| message.blocks.iter().any(
        |block| matches!(block, smith_transcript::Block::Text(text) if text.text.as_ref() == b"committed answer")
    )));
    let answer = world.answer().expect("drained answer");
    assert_eq!(answer.turns, 1);
    assert!(
        matches!(answer.result, RunResult::Failed(failed) if failed.reason == RunFailure::Cancelled),
        "a drained run cancels only at the turn boundary"
    );
    world.assert_agent_clean();
}

#[test]
fn a_signal_then_the_hosts_cancel_answers_cancelled_once() {
    let mut world = held();
    world.signal();
    world.cancel_after_signal();
    assert!(world.settle());
    assert_eq!(notice(&world).signal, smith_channel::Signal::Terminate);
    assert_eq!(world.observed().iter().filter(|frame| frame.kind == smith_channel::kind::ANSWER).count(), 1);
    assert!(
        matches!(world.answer().expect("answer").result, RunResult::Failed(failed) if failed.reason == RunFailure::Cancelled)
    );
    world.assert_agent_clean();
}

#[test]
fn a_signal_then_the_hosts_hangup_cancels_the_live_run() {
    let mut world = held();
    world.observe_facts(TraceCase::Failed, 8);
    world.signal();
    world.hang_up_after_signal();
    assert!(world.settle());
    assert_eq!(notice(&world).signal, smith_channel::Signal::Terminate);
    assert!(world.answer().is_none(), "the hung-up channel cannot take the answer");
    let answers: Vec<_> = world
        .records()
        .iter()
        .filter_map(
            |record| if let smith_events::Event::RunCompleted(answer) = &record.event { Some(answer) } else { None },
        )
        .collect();
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].failure.as_ref().expect("cancelled answer").class, smith_events::AnswerClass::Cancelled);
    world.assert_agent_clean();
}

#[test]
fn a_signal_before_admission_is_told_at_admission_and_the_run_goes_on() {
    let mut world = World::new(8, &charter());
    world.signal_before_admission();
    assert!(world.settle());
    notice(&world);
    let kinds: Vec<_> = world.observed().iter().map(|frame| frame.kind).collect();
    assert!(
        kinds.iter().position(|kind| *kind == smith_channel::kind::ADMITTED).expect("admission")
            < kinds.iter().position(|kind| *kind == smith_channel::kind::SIGNALLED).expect("signal")
    );
    assert!(world.peer_replied());
    assert_eq!(world.answer().expect("drained answer").turns, 1);
    world.assert_agent_clean();
}
