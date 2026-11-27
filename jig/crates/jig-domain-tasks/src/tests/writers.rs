//! Writer room at claim admission (domain/tasks/holds.md, 3).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "unit tests build fixtures and inspect selected child events in ordinary Rust"
)]
use super::revisions::{Harness, LIMITS, task, to};
use crate::{
    Capacity, CapacityAction, CapacityCause, CapacityRefusal, Domain, Event, HoldKind, Holding, Kind, Limits, Name,
    Party, Refusal, Request, Stored, Taken, Writer, WriterSlot,
};
use alloc::collections::BTreeMap;

const HOLDER: u64 = 1;
const CLAIMANT: u64 = 2;
const ATTEMPT: u64 = 1;
const PROJECT: u32 = 1;
const CONNECTOR: u16 = 1;
const KIND: u16 = 1;
const OCCUPIED: [u8; 3] = [1, 2, 3];
const HOLDER_FREE: u8 = 4;
const FIRST: u8 = 5;
const SECOND: u8 = 6;
// The two live tasks, with two new holds apiece, give four writer slots.
const TINY: Limits = Limits { tasks: 2, holdings: 2, ..LIMITS };
const WRITER_LIMIT: u32 = 4;
const LAST_SLOT: u64 = 4; // The fourth acquisition, after the three occupied slots.
const RUN_BUDGET: u64 = 10;

fn name(id: u8) -> Name {
    Name { connector: CONNECTOR, path: Box::new([Box::new([id])]) }
}

fn write_hold(id: u8) -> Holding {
    Holding::Write { resource: name(id), kind: KIND }
}

fn configure(h: &mut Harness) {
    h.send(Event::Kinds {
        connector: CONNECTOR,
        kinds: Box::new([Kind { connector: CONNECTOR, kind: KIND, hold: HoldKind::Exclusive { taken: Taken::Waits } }]),
    });
}

fn one_slot_left() -> Harness {
    let mut source = Harness::new();
    configure(&mut source);
    let mut holder = task(HOLDER, &[]);
    let mut claimant = task(CLAIMANT, &[]);
    holder.holdings = OCCUPIED.into_iter().chain([HOLDER_FREE]).map(write_hold).collect();
    claimant.holdings = [FIRST, SECOND].into_iter().map(write_hold).collect();
    source.make(vec![holder, claimant]);

    // The root may keep four holds under the smaller configuration, but any
    // one claim can name only two writes. Leave three slots in effect flight.
    let mut h = Harness {
        domain: Domain::new(&TINY, super::KEPT, 91, 1, super::charters(), Box::new([0])),
        env: source.env.with(TINY),
        stored: BTreeMap::new(),
    };
    configure(&mut h);
    for record in source.stored.into_values() {
        if let Stored::Live(_) | Stored::Ledger(_) = &record {
            h.send(Event::Restore { record });
        }
    }
    h.send(Event::Check);
    h.send(Event::Restored);
    for id in OCCUPIED {
        h.send(Event::EffectInFlight { reply_to: to(), task: HOLDER, resource: name(id), entry: u64::from(id) });
    }
    h
}

fn claim(h: &mut Harness, task: u64, writes: &[u8]) -> Vec<Request> {
    h.send(Event::Prepare { reply_to: to(), task });
    h.send(Event::Claim {
        reply_to: to(),
        fresh: None,
        session_end: None,
        offered: crate::Reasons::ALL,
        reasons: crate::Reasons::FIRST,
        task,
        attempt: ATTEMPT,
        account: None,
        resumed: false,
        budget: RUN_BUDGET,
        writes: writes.iter().map(|&id| name(id)).collect(),
    })
}

fn saved_slots(h: &Harness) -> Vec<WriterSlot> {
    h.stored.values().filter_map(|row| if let Stored::Writer(slot) = row { Some(slot.clone()) } else { None }).collect()
}

#[test]
fn a_claim_takes_the_last_writer_slot() {
    let mut h = one_slot_left();
    claim(&mut h, CLAIMANT, &[FIRST]);
    let resource = name(FIRST);
    let slot = WriterSlot {
        number: LAST_SLOT,
        resource: resource.clone(),
        writer: Writer::Run { task: CLAIMANT, attempt: ATTEMPT },
        lost: false,
    };
    assert_eq!(h.domain.writer(&resource), Some(&slot));
    assert_eq!(h.stored.get(&crate::Key::Writer(LAST_SLOT)), Some(&Stored::Writer(slot)));
}

#[test]
fn writer_room_refuses_the_whole_claim_one_past_capacity() {
    // One past the bound, starting at full room or with one vacant slot.
    for (first, next) in [([HOLDER_FREE].as_slice(), [FIRST].as_slice()), (&[], &[FIRST, SECOND])] {
        let mut h = one_slot_left();
        claim(&mut h, HOLDER, first);
        let before = saved_slots(&h);
        let out = claim(&mut h, CLAIMANT, next);
        let [Request::Refused { problem, .. }] = out.as_slice() else { panic!("one claim refusal: {out:?}") };
        let capacity = Capacity {
            project: Some(PROJECT),
            by: Party::Task(CLAIMANT),
            action: CapacityAction::Claim { attempt: ATTEMPT },
            cause: CapacityCause::Writers { limit: WRITER_LIMIT },
        };
        assert_eq!(problem.why, Refusal::Holds);
        assert_eq!(problem.capacity, Some(capacity));
        assert_eq!(h.domain.capacity_refusals().as_slice(), &[CapacityRefusal { request: capacity, at: h.env.wall }]);
        assert_eq!(saved_slots(&h), before);
        for &id in next {
            assert_eq!(h.domain.writer(&name(id)), None);
        }
    }
}
