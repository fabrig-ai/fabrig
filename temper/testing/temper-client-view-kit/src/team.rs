//! Shared team document facts.
use jig_client::contract::HeldRole;
use skein_lib::{List, Wall};
use temper_client_domain::pages::team::{self, Facts, Person, Waits};
use temper_client_protocol_wire::v1 as w;

pub const READER: u64 = 7;

pub const SAM: u64 = 8;

pub fn person(you: bool, name: &[u8], role: HeldRole, waits: Option<Waits>) -> Person {
    Person { party: if you { READER } else { SAM }, you, name: name.into(), role, waits, signed: None }
}

pub const MADE: Wall = Wall::from_nanos(1_793_577_600_000_000_000);

pub fn holder_task(number: u64) -> w::Holder {
    w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number }).unwrap())
}

pub fn owner_task(number: u64) -> w::AreaOwner {
    w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number }).unwrap())
}

pub fn facts() -> Facts {
    Facts {
        member: true,
        people: List::from_boxed(Box::new([
            person(true, b"Alex", HeldRole::Owner, Some(Waits::default())),
            person(false, b"Sam", HeldRole::Member, Some(Waits { questions: 1, ..Waits::default() })),
        ])),
        cut: false,
        areas: Some(team::Areas {
            rows: List::from_boxed(Box::new([team::Area {
                name: b"tests".as_slice().into(),
                words: b"Tests, fuzzing and the tiers".as_slice().into(),
                owner: owner_task(52),
                holder: Some(holder_task(52)),
            }])),
            triager: Some(owner_task(52)),
            holder: holder_task(52),
        }),
        follows: List::with_capacity(0),
        roles: List::from_boxed(Box::new([team::Role {
            task: 52,
            name: b"Prover".as_slice().into(),
            now: Some(b"Reproduced the finding".as_slice().into()),
            spent: 21,
            budget: 40,
        }])),
        roles_cut: false,
        selected: Some(52),
        card: Some(Box::new(team::Card {
            task: 52,
            name: b"Prover".as_slice().into(),
            made: Some(MADE),
            requester: Some(team::StartedBy::You),
            hold: None,
            paused: false,
            pause: true,
            resume: false,
            talk: true,
            report: None,
            now: None,
            sessions: List::with_capacity(0),
            rules: None,
            wakes: List::with_capacity(0),
            wakes_cut: false,
        })),
    }
}
