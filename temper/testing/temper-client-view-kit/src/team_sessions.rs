use skein_lib::{List, Wall};
use temper_client_domain::pages::team::{self, Facts};

#[must_use]
pub fn facts() -> Facts {
    Facts {
        member: true,
        people: List::with_capacity(0),
        cut: false,
        areas: None,
        follows: List::with_capacity(0),
        roles: List::from_boxed(Box::new([team::Role {
            task: 52,
            name: b"Prover".as_slice().into(),
            now: None,
            spent: 17,
            budget: 40,
        }])),
        roles_cut: false,
        selected: Some(52),
        card: Some(Box::new(team::Card {
            task: 52,
            name: b"Prover".as_slice().into(),
            made: None,
            requester: None,
            hold: None,
            paused: false,
            pause: false,
            resume: false,
            talk: false,
            now: None,
            rules: None,
            wakes: List::with_capacity(0),
            wakes_cut: false,
            report: None,
            sessions: List::from_boxed(Box::new([
                team::Session { start: Wall::from_nanos(1_794_102_720_000_000_000), end: None, words: None, spent: 0 },
                team::Session {
                    start: Wall::from_nanos(1_794_068_400_000_000_000),
                    end: Some(Wall::from_nanos(1_794_069_720_000_000_000)),
                    words: Some(b"Triaged three findings; routed two.".as_slice().into()),
                    spent: 17,
                },
            ])),
        })),
    }
}
