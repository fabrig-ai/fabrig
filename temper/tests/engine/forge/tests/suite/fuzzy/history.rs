//! Shared history referee (temper/docs/design/domain/landing-history.md, 2.2–2.3, 3.2 and 8).
use std::{
    collections::{BTreeMap, BTreeSet},
    mem::discriminant,
};
use temper_engine_domain_forge::history::{Event, What};

fn start(end: &What, candidate: &What) -> bool {
    match (end, candidate) {
        (What::Unready { .. }, What::Ready)
        | (What::Updated { .. }, What::Update { .. })
        | (What::Resumed, What::Paused { .. })
        | (What::RoundEnded { .. }, What::Formed { .. })
        | (What::Searched { .. }, What::Search)
        | (What::ProbeEnded { .. }, What::Probe) => true,
        (What::Departed { line, .. }, What::Joined { line: other, .. }) => line == other,
        (What::JobEnded { job, .. }, What::Job { job: other, .. }) => job == other,
        (What::Answered { ask, .. }, What::Asked { ask: other, .. }) => ask == other,
        (What::GateEnded { gate, key, .. }, What::Gate { gate: other, key: other_key }) => {
            gate == other && key == other_key
        }
        (What::Released { on, .. }, What::Held { on: other, .. }) => on == other,
        _ => false,
    }
}
fn end(what: &What) -> bool {
    matches!(
        what,
        What::Unready { .. }
            | What::Departed { .. }
            | What::Updated { .. }
            | What::JobEnded { .. }
            | What::Answered { .. }
            | What::GateEnded { .. }
            | What::RoundEnded { .. }
            | What::Searched { .. }
            | What::ProbeEnded { .. }
            | What::Resumed
            | What::Released { .. }
    )
}
fn probe_head(event: &Event) -> Option<skein_forge::api::Commit> {
    matches!(event.what, What::Probe | What::ProbeEnded { .. }).then_some(event.head).flatten()
}
fn own(event: &Event) -> bool {
    // Only verification and ends of intervals open at its end survive (2.2).
    !matches!(
        event.what,
        What::Recorded { .. }
            | What::After { .. }
            | What::JobEnded { .. }
            | What::Answered { .. }
            | What::Released { .. }
            | What::RoundEnded { .. }
            | What::LandingEnded { .. }
    )
}

fn identity(event: &Event) -> (u64, u32, Option<skein_forge::api::Commit>) {
    (event.operation, event.attempt, probe_head(event))
}

pub(super) fn check(seed: u64, change: u64, events: &[Event]) {
    // Include future starts so a reversed interval still fails "start before end".
    let mut by_identity = BTreeMap::<_, Vec<_>>::new();
    for (index, event) in events.iter().enumerate() {
        by_identity.entry(identity(event)).or_default().push((index, &event.what));
    }
    let mut identities = BTreeMap::<_, Vec<_>>::new();
    let mut closed = BTreeSet::new();
    let mut terminal = false;
    let mut landed = false;
    let mut previous = 0;
    for (index, event) in events.iter().enumerate() {
        let context = || format!("seed {seed}, change {change}, event {event:?}\nhistory: {events:#?}");
        assert_eq!(event.change, change, "{}", context());
        assert!(event.number > previous, "numbers rise: {}", context());
        previous = event.number;
        // One bucket has at most one of each kind; discriminants need equality, not an invented order.
        let kinds = identities.entry(identity(event)).or_default();
        let kind = discriminant(&event.what);
        assert!(!kinds.contains(&kind), "unique kind and identity: {}", context());
        kinds.push(kind);
        assert!(!terminal || !own(event), "no own step after terminal: {}", context());
        if terminal && matches!(event.what, What::Recorded { .. } | What::After { .. }) {
            assert!(landed, "verification belongs to a landed change: {}", context());
        }
        if matches!(event.what, What::Landed | What::Ended { .. }) {
            landed = matches!(event.what, What::Landed);
            terminal = true;
        }
        if !end(&event.what) {
            continue;
        }
        // Uniqueness bounds each identity's candidates by the number of What kinds.
        let paired = by_identity.get(&identity(event)).and_then(|candidates| {
            candidates.iter().find(|(_, what)| start(&event.what, what)).map(|(index, _)| *index)
        });
        if let Some(paired) = paired {
            assert!(paired < index, "start before end: {}", context());
            assert!(!closed.contains(&paired), "one end per start: {}", context());
            closed.insert(paired);
        } else {
            // An already-ended gate can first be heard without a start (8).
            assert!(matches!(event.what, What::GateEnded { .. }), "end has a start: {}", context());
        }
    }
    // Unclosed intervals, including asks whose delegate is still running, remain open.
}
