//! A kept part releases the shared record window without stopping the turn.
//! Contract: smith/docs/design/domain/turn-parts.md, section 3.

use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Line, Script};
use skein_lib::{Duration, Time, Token};
use skein_world::domain::Span;
use smith_agent_world::{Job, Settings, World, scripted_charter};
use smith_domain::{run, session};

fn finishing() -> World {
    let calm = Settings::calm(2041);
    let settings = Settings {
        job: Job::Reporting,
        network: Span::millis(0, 0),
        provider: skein_fake_llm_domain::Config {
            latency_min: Duration::from_millis(100),
            latency_max: Duration::from_millis(100),
            ..calm.provider
        },
        ..calm
    };
    let mut charter = scripted_charter(&settings);
    charter.instructions = b"@kept-part".as_slice().into();
    charter.brief.sections = Box::default();
    charter.conventions = None;
    let script = Script {
        cue: b"@kept-part".as_slice().into(),
        turns: Box::new([smith_agent_world::fixture::calls(
            Box::new([Line::Call {
                name: b"finish".as_slice().into(),
                arguments: br#"{"report":"Finished.","source":"workspace"}"#.as_slice().into(),
            }]),
            1,
        )]),
    };
    let mut world =
        World::with_workspace_scripts_charter(settings, None, None, Checkout::new(), Box::new([script]), charter);
    world.tell_parts();
    world.message_at(
        Time::from_nanos(1_000_000),
        Token::new(9),
        b"host".as_slice().into(),
        b"Finish.".as_slice().into(),
    );
    world
}

#[test]
fn a_kept_answer_part_lets_the_finish_turn_and_answer_out() {
    let mut world = finishing();
    world.record_window(smith_domain::Window { turns: 1, bytes: u64::MAX });
    let answer_kept = Time::from_nanos(500_000_000);
    let result_kept = Time::from_nanos(600_000_000);
    world.part_kept_at(answer_kept, 1);
    world.part_kept_at(result_kept, 2);
    world.run(2000);
    assert!(matches!(world.answer(), run::Answer::Accepted { turns: 1, .. }), "{:?}", world.answer());
    assert_eq!(world.parts().len(), 2);
    assert_eq!(world.parts()[0].3.kind, session::record::PartKind::Answer);
    assert_eq!(world.parts()[1].3.kind, session::record::PartKind::Result);
    assert_eq!(world.turns().len(), 1);
    assert!(world.parts()[0].0 < answer_kept);
    assert_eq!(world.parts()[1].0, answer_kept, "the result stays held until the answer part is kept");
    let outputs: Vec<_> = world
        .messages_seen()
        .iter()
        .filter_map(|(at, seen)| match seen {
            smith_agent_world::messages_referee::Seen::Turn { .. } => Some((*at, "turn")),
            smith_agent_world::messages_referee::Seen::Answer { .. } => Some((*at, "answer")),
            _ => None,
        })
        .collect();
    assert_eq!(outputs, [(result_kept, "turn"), (result_kept, "answer")]);
}
