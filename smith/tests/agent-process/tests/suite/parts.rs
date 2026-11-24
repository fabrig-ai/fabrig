//! A native agent goes on after the host keeps its answer part.
//! Contract: smith/docs/design/domain/turn-parts.md, section 3.
use skein_fake_llm_domain::api::Line;
use skein_lib::{Duration, Reader};
use smith_agent_process_world::{World, charter};
use smith_channel::{Record, RunResult, kind};

fn finish_after_kept(after: Duration) {
    let mut world = World::new(2041, &charter());
    world.tell_parts(after);
    world.acknowledgement_window(1);
    world.script(Box::new([smith_agent_world::fixture::calls(
        Box::new([Line::Call {
            name: b"finish".as_slice().into(),
            arguments: br#"{"outcome":{"form":"report","text":"Finished.","fields":{}}}"#.as_slice().into(),
        }]),
        1,
    )]));
    assert!(world.settle(), "{:?}", world.errors());
    let answer = world.answer().expect("native answer");
    assert!(matches!(answer.result, RunResult::Accepted(_)), "{answer:?}");
    assert_eq!(answer.turns, 1);
    let frames = world.observed();
    let parts: Vec<_> = frames
        .iter()
        .enumerate()
        .filter(|(_, frame)| frame.kind == kind::PART)
        .map(|(index, frame)| {
            (index, smith_channel::Part::decode(&smith_channel::CEILINGS, &mut Reader::new(&frame.body)).expect("part"))
        })
        .collect();
    assert_eq!(parts.len(), 2);
    let acknowledgements = world.part_acknowledgements();
    assert_eq!(acknowledgements.len(), 2);
    assert_eq!(acknowledgements[0].1, 1);
    assert_eq!(acknowledgements[1].1, 2);
    assert_eq!(parts[0].1.kind, smith_channel::PartKind::Answer);
    assert_eq!(parts[1].1.kind, smith_channel::PartKind::Result);
    assert!(parts[0].0 < acknowledgements[0].2);
    assert!(parts[1].0 >= acknowledgements[0].2, "the result waits for answer-part credit");
    let turn = frames.iter().position(|frame| frame.kind == kind::TURN).expect("whole turn");
    let answer = frames.iter().position(|frame| frame.kind == kind::ANSWER).expect("answer");
    assert!(parts[1].0 < turn && turn < answer, "call effect, whole turn, then answer");
    assert!(turn >= acknowledgements[1].2, "the whole turn waits for result-part credit");
    let result = smith_transcript::Turn::decode(&smith_transcript::CEILINGS, &mut Reader::new(&parts[1].1.body))
        .expect("result body");
    let smith_transcript::Block::Result(result) =
        result.messages.get(0).expect("result message").blocks.get(0).expect("result block")
    else {
        panic!("finish result")
    };
    let smith_transcript::Returned::Said(said) = &result.returned else { panic!("finish feedback") };
    assert_eq!(said.text.as_ref(), b"accepted");
    assert!(!said.error);
    assert!(acknowledgements[0].0.as_nanos() >= after.as_nanos());
    world.assert_agent_clean();
}

#[test]
fn a_parts_telling_native_agent_finishes_after_its_answer_is_kept() {
    finish_after_kept(Duration::ZERO);
}

#[test]
fn a_late_part_acknowledgement_releases_the_finished_calls_turn_and_answer() {
    finish_after_kept(Duration::from_secs(1));
}
