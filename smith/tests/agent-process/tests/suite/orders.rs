//! Loaded entrance orders through the real service loop and simulated pipes
//! (skein/docs/design/draft/dynamic-memory.md, 4.6).

use skein_fake_llm_domain::api;
use skein_lib::Duration;
use smith_agent_process_world::{Order, World, charter};
use smith_events::Event;

const SEED: u64 = 47;
// A first text turn leaves another completion pending while the Message arrives.
const REQUEST_TURNS: u32 = 2;
const HOLD: &[u8] = b"receipt-pressure";
// Keep the provider alive beyond the two Message entrances and sink release.
const PROVIDER_RELEASE: Duration = Duration::from_millis(100);

fn pressured(order: Order, charter: &[u8]) -> World {
    let mut world = World::new(SEED, charter);
    world.order_pressure(order);
    world
}

fn receipts(records: &[smith_events::Record]) -> Vec<u64> {
    records
        .iter()
        .filter_map(
            |record| {
                if let Event::MessageReceived(message) = &record.event { Some(message.name) } else { None }
            },
        )
        .collect()
}

/// Order: `domain_requests_dispatched` (phase C).
#[test]
fn domain_requests_dispatch_before_a_later_entrances_facts() {
    let mut world = pressured(Order::Requests, &two_turn_charter());
    world.anthropic();
    world.ask(true, false, false);
    let turn = api::Turn {
        lines: Box::new([api::Line::Text { text: b"Hello".as_slice().into() }]),
        finish: api::Finish::Stop,
        tokens: 1,
    };
    world.script(Box::new([turn.clone(), turn]));
    assert!(world.settle(), "seed {SEED}");
    let blocked = world.order_snapshot();
    assert!(blocked.output_held, "the held admission write exhausts the constrained output path");
    assert_eq!(blocked.messages_written, 1, "the later entrance crossed the host pipe while output was held");
    assert!(receipts(&blocked.records).is_empty(), "its fact cannot pass the preceding undispatched request");
    assert_eq!(receipts(world.records()), [1], "releasing host output lets the queued entrance run once");
    assert!(
        receipt_times(world.records()).iter().all(|time| *time >= blocked.released_at_ms),
        "the Message entrance runs after the preceding request can dispatch"
    );
    world.assert_event_conformance();
    world.assert_agent_clean();
}

/// Order: `domain_facts_dispatched` (phase C).
#[test]
fn domain_facts_reach_the_sink_before_the_next_entrance() {
    let mut world = pressured(Order::Facts, &charter());
    world.anthropic();
    world.script(Box::new([api::Turn {
        lines: Box::new([api::Line::Pieces {
            pieces: Box::new([
                api::Piece { text: b"prefix".as_slice().into(), hold: None },
                api::Piece { text: b"suffix".as_slice().into(), hold: Some(HOLD.into()) },
            ]),
        }]),
        finish: api::Finish::Stop,
        tokens: 1,
    }]));
    world.release_after(HOLD, 1, PROVIDER_RELEASE);
    assert!(world.settle(), "seed {SEED}");
    let blocked = world.order_snapshot();
    assert!(blocked.output_held, "the Complete trace's response-start append is held");
    assert_eq!(blocked.messages_written, 2, "both Message entrances crossed the host pipe while the sink was full");
    assert!(receipts(&blocked.records).is_empty(), "both receipts wait behind the held Complete output");
    assert_eq!(receipts(world.records()), [1, 2], "the first receipt reaches the sink before the second entrance");
    let times = receipt_times(world.records());
    assert!(times[0] < blocked.released_at_ms, "the first entrance emits a fact while the sink is blocked");
    assert!(times[1] >= blocked.released_at_ms, "the second entrance waits until the first fact can drain");
    world.assert_event_conformance();
    world.assert_agent_clean();
}

// The first text turn leaves a second completion request behind its Turn output.
fn two_turn_charter() -> Box<[u8]> {
    smith_agent_process_world::wire::edit_charter(&charter(), |parts| {
        let mut budget = parts.budget.clone().into_parts();
        budget.turns = REQUEST_TURNS;
        parts.budget = smith_charter::Budget::new(&smith_charter::CEILINGS, budget).expect("two-turn budget");
    })
}

fn receipt_times(records: &[smith_events::Record]) -> Vec<u64> {
    records
        .iter()
        .filter_map(|record| if matches!(record.event, Event::MessageReceived(_)) { Some(record.t_ms) } else { None })
        .collect()
}
