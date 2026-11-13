//! Every start crosses the switch and its durable mark before adopting runs.
use jig_domain_core as core;
use temper_engine_domain::{self as engine, Range, Record, Write};
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

const FIRST_START: &str = "first start";
const RESTART: &str = "restart with a live run";
// A direct world's complete restart settles within its existing 256-step horizon.
const STEPS: u32 = 256;
// The start's own decision contains only its deployment header and wires (root.md, 8.1).
const MARK_WRITES: usize = 2;

#[test]
fn every_start_takes_the_switch_step_and_commits_its_wires() {
    let (mut running, _) = Driver::chat();
    for (label, mut store) in vec![(FIRST_START, Store::new()), (RESTART, running.take_store())].into_boxed_slice() {
        store.crash(9761);
        let pending = store.pending_generation;
        let mut driver = Driver::cold(store, walking::config(91), &walking::limits());
        for _ in 0..STEPS {
            if driver.store.pending.front().is_some_and(|(_, writes)| writes.contains(&Write::Wires)) {
                break;
            }
            if !driver.store.pending.is_empty() {
                let number = driver.store.apply();
                driver.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
            }
            driver.advance(false);
        }
        let (mark, writes) = driver.store.pending.front().expect("start reaches its mark");
        let mark = *mark;
        assert_eq!(writes.len(), MARK_WRITES, "{label}: a separate start mark");
        assert!(writes.contains(&Write::Wires), "{label}");
        assert!(writes.iter().any(|write| matches!(write,
            Write::Save(Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))) if header.commits == mark)), "{label}: header and wires in one commit");
        let switches = if pending { vec![driver.loads.len()] } else { Vec::new() };
        assert_eq!(driver.switches, switches, "{label}: only a pending generation asks the store to switch");
        assert_eq!(
            driver.loads.last().map(|(_, range)| range),
            Some(&Range::Core(engine::CoreRange::Sweep)),
            "{label}: sweep finished before switch"
        );
        driver.store.tick(true).expect("mark becomes durable with its answer held");
        for _ in 0..STEPS {
            driver.advance(false);
        }
        assert!(!driver.root.ready(), "{label}: held mark holds the script");
        assert_eq!(
            driver.root.start_phase(),
            Some(core::start::Phase::Wires),
            "{label}: adoption remains behind Switch"
        );
        assert!(driver.store.pending.is_empty(), "{label}: adoption issued no later decision");
        assert!(driver.delivered.is_empty(), "{label}: no run delivery before the mark");
        let number = driver.store.release_held().expect("held mark answer");
        assert_eq!(number, mark, "{label}");
        driver.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        driver.pump();
        assert!(driver.root.ready(), "{label}: durable mark releases the script");
        assert_eq!(driver.store.header().id, walking::config(91).core.deployment, "{label}: deployment retained");
        let wires = driver.take_store().into_wires();
        let kept: Vec<_> = wires.wires().iter().map(|wire| (wire.name(), wire.version())).collect();
        let expected: Vec<_> = temper_engine_service_vocabulary::store::WIRES
            .iter()
            .map(|wire| (wire.name.as_bytes(), wire.version))
            .collect();
        assert_eq!(kept, expected, "{label}: actual component Wires row");
    }
}
