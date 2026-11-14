//! Shared history referee (temper/docs/design/domain/landing-history.md, 2.2–2.3, 3.2 and 8).
use std::mem::discriminant;
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
    if matches!(event.what, What::Recorded { .. } | What::After { .. }) {
        return false;
    }
    // A landing can still finish its round after a member lands or leaves (3.3).
    event.landing.is_none()
        || !matches!(
            event.what,
            What::Formed { .. }
                | What::RoundEnded { .. }
                | What::Gate { .. }
                | What::GateEnded { .. }
                | What::Proved { .. }
                | What::Search
                | What::Searched { .. }
                | What::Probe
                | What::ProbeEnded { .. }
                | What::LandingEnded { .. }
                | What::Held { on: temper_engine_domain_forge::history::On::Landing, .. }
                | What::Released { on: temper_engine_domain_forge::history::On::Landing, .. }
        )
}

pub(super) fn check(seed: u64, change: u64, events: &[Event]) {
    let mut identities = Vec::new();
    let mut closed = Vec::new();
    let mut terminal = false;
    let mut previous = 0;
    for (index, event) in events.iter().enumerate() {
        let context = || format!("seed {seed}, change {change}, event {event:?}\nhistory: {events:#?}");
        assert_eq!(event.change, change, "{}", context());
        assert!(event.number > previous, "numbers rise: {}", context());
        previous = event.number;
        let identity = (discriminant(&event.what), event.operation, event.attempt, probe_head(event));
        assert!(!identities.contains(&identity), "unique kind and identity: {}", context());
        identities.push(identity);
        assert!(!terminal || !own(event), "no own step after terminal: {}", context());
        if matches!(event.what, What::Landed | What::Ended { .. }) {
            terminal = true;
        }
        if !end(&event.what) {
            continue;
        }
        let paired = events.iter().position(|candidate| {
            start(&event.what, &candidate.what)
                && candidate.operation == event.operation
                && candidate.attempt == event.attempt
                && probe_head(candidate) == probe_head(event)
        });
        if let Some(paired) = paired {
            assert!(paired < index, "start before end: {}", context());
            assert!(!closed.contains(&paired), "one end per start: {}", context());
            closed.push(paired);
        } else {
            // An already-ended gate can first be heard without a start (8).
            assert!(matches!(event.what, What::GateEnded { .. }), "end has a start: {}", context());
        }
    }
    // Unclosed intervals, including asks whose delegate is still running, remain open.
}
