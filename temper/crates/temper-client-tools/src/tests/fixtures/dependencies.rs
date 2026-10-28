use super::{PARTY, TASK, document_text, goal};
use jig_client::contract as c;
use skein_lib::{List, Wall};

pub(crate) const DEPENDENCY_ROWS: u32 = 3;
pub(crate) const DEPENDENCY_ADDED_AT: Wall = Wall::from_nanos(1_792_645_800_000_000_000);

pub(crate) fn goal_with_dependencies() -> c::Task {
    let mut task = goal(TASK, c::WaitsOn::Nothing);
    task.instructions = Some(document_text(b"Keep the endpoint small."));
    let running = goal(44, c::WaitsOn::Nothing).chip;
    let mut waiting = goal(45, c::WaitsOn::Nothing).chip;
    waiting.phase = c::Phase::Waiting;
    task.dependencies = List::with_capacity(DEPENDENCY_ROWS);
    for link in [
        c::OwnerTaskLink::Stored { number: 43, status: Some(c::OwnerTaskStatus::Done) },
        c::OwnerTaskLink::Live(running),
        c::OwnerTaskLink::Retained { chip: waiting, at: None },
    ] {
        task.dependencies.push(link).expect("three dependencies");
    }
    task.dependency_facts = List::with_capacity(DEPENDENCY_ROWS);
    for fact in [
        c::Dependency {
            number: 45,
            pending: true,
            added: Some(c::DependencyAdded { at: DEPENDENCY_ADDED_AT, by: c::OwnerRequester::Task(7) }),
            keeps: false,
            status: None,
        },
        c::Dependency { number: 43, pending: false, added: None, keeps: false, status: Some(c::OwnerTaskStatus::Done) },
        c::Dependency {
            number: 44,
            pending: true,
            added: Some(c::DependencyAdded { at: DEPENDENCY_ADDED_AT, by: c::OwnerRequester::Person(PARTY) }),
            keeps: true,
            status: None,
        },
    ] {
        task.dependency_facts.push(fact).expect("three dependency facts");
    }
    task
}

pub(crate) fn goal_with_dependencies_and_long_specification() -> c::Task {
    let mut task = goal_with_dependencies();
    let mut words = skein_lib::Writer::new(usize::try_from(super::LONG_WORDS).expect("specification size fits"));
    for _ in 0..super::LONG_WORDS {
        words.put(b"x").expect("fixture length");
    }
    task.spec.specification.words = document_text(&words.finish());
    task
}
