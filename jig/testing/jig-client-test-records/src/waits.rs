//! Paired wire and client literals for structured waits (jig/docs/design/domain/engine/contracts.md, 4).
use jig_client_domain::contract::{self as c, waits::Bound};
use jig_client_protocol_wire as w;
use skein_lib::{List, Wall};

/// Values in task order, including every object and bound form.
#[must_use]
pub fn structured_waits() -> (List<w::Wait>, List<c::Wait>) {
    let (mut wire, mut client) = (List::with_capacity(10), List::with_capacity(10));
    for (kind, object) in objects() {
        let since = 11 + u64::from(wire.len());
        let (wire_bound, client_bound) = bounds(wire.len());
        let (wire_holder, client_holder) = holders(wire.len());
        wire.push(w::Wait { task: 7, kind: kind.0, object: object.0, since, bound: wire_bound, hears: wire_holder })
            .expect("ten supplied wait examples fit");
        client
            .push(c::Wait {
                task: 7,
                kind: kind.1,
                object: object.1,
                since: Wall::from_nanos(since),
                bound: client_bound,
                hears: client_holder,
            })
            .expect("ten supplied wait examples fit");
    }
    (wire, client)
}

type Object = ((w::WaitKind, c::WaitKind), (w::WaitObject, c::WaitObject));
fn objects() -> [Object; 10] {
    use c::WaitObject as C;
    use w::WaitObject as W;
    let things = [(1, 113, Some(29)), (2, 127, None), (3, 131, Some(31)), (4, 137, None)];
    [
        entry(w::WaitedFor::Question, c::WaitedFor::Question, 401),
        entry(w::WaitedFor::Proposal, c::WaitedFor::Proposal, 403),
        entry(w::WaitedFor::Escalation, c::WaitedFor::Escalation, 409),
        entry(w::WaitedFor::PersonTask, c::WaitedFor::PersonTask, 7),
        (
            (w::WaitKind::Question, c::WaitKind::Question),
            (W::Question(w::WaitQuestion { number: 23, answerer: 29 }), C::Question { number: 23, answerer: 29 }),
        ),
        ((w::WaitKind::Dependency, c::WaitKind::Dependency), (W::Task(w::WaitTask { value: 17 }), C::Task(17))),
        (
            (w::WaitKind::Resource, c::WaitKind::Resource),
            (
                W::Resource(w::WaitResource {
                    name: w::WaitResourceName { connector: 13, path: resource_path() },
                    holder: Some(97),
                }),
                C::Resource { name: c::ResourceName { connector: 13, path: resource_path() }, holder: Some(97) },
            ),
        ),
        (
            (w::WaitKind::Host, c::WaitKind::Host),
            (
                W::Run(w::WaitRun { value: w::TaskRunWait::Worker(w::TaskWaitWorker { worker: 43, until: 173 }) }),
                C::Run(c::TaskRunWait::Worker { worker: 43, until: Wall::from_nanos(173) }),
            ),
        ),
        (
            (w::WaitKind::News, c::WaitKind::News),
            (
                W::News(w::WaitNews { subscriptions: List::from_boxed(Box::new([37, 31, 41, 47])), timer: Some(83) }),
                C::News {
                    subscriptions: List::from_boxed(Box::new([37, 31, 41, 47])),
                    timer: Some(Wall::from_nanos(83)),
                },
            ),
        ),
        (
            (w::WaitKind::Step, c::WaitKind::Step),
            (
                W::Procedure(w::WaitProcedure {
                    step: 19,
                    things: List::from_boxed(Box::new(things.map(|(what, number, owner)| w::Thing {
                        what,
                        number,
                        owner,
                    }))),
                }),
                C::Procedure {
                    step: 19,
                    things: List::from_boxed(Box::new(things.map(|(what, number, owner)| c::Thing {
                        what,
                        number,
                        owner,
                    }))),
                },
            ),
        ),
    ]
}

fn entry(wire: w::WaitedFor, client: c::WaitedFor, number: u64) -> Object {
    (
        (w::WaitKind::Party, c::WaitKind::Party),
        (
            w::WaitObject::Entry(w::WaitEntry {
                kind: wire,
                number,
                at: w::EscalationHolder::Role(w::EscalationHolderRole { project: 7, role: 5 }),
            }),
            c::WaitObject::Entry { kind: client, number, at: c::EscalationHolder::Role { project: 7, role: 5 } },
        ),
    )
}

fn resource_path() -> List<Box<[u8]>> {
    List::from_boxed(Box::new([Box::<[u8]>::from(&b"pool/tiny"[..]), Box::new([0xff, b':'])]))
}

fn bounds(index: u32) -> (w::WaitBound, Bound) {
    match index {
        0 => at(201, w::AtBound::PassUp, c::AtBound::PassUp),
        2 => at(101, w::AtBound::Hold, c::AtBound::Hold),
        4 => at(301, w::AtBound::Overdue, c::AtBound::Overdue),
        7 => at(701, w::AtBound::GiveUp, c::AtBound::GiveUp),
        3 | 8 => none(w::Unbounded::Standing, c::Unbounded::Standing),
        5 => none(w::Unbounded::Behind(w::UnboundedBehind { task: 17 }), c::Unbounded::Behind(17)),
        6 => none(w::Unbounded::OperatorHold, c::Unbounded::OperatorHold),
        _ => none(w::Unbounded::NoDeadline, c::Unbounded::NoDeadline),
    }
}

fn at(at: u64, wire: w::AtBound, client: c::AtBound) -> (w::WaitBound, Bound) {
    (w::WaitBound::At(w::WaitAtBound { at, then: wire }), Bound::At { at: Wall::from_nanos(at), then: client })
}

fn none(wire: w::Unbounded, client: c::Unbounded) -> (w::WaitBound, Bound) {
    (w::WaitBound::None(w::WaitNoBound { reason: wire }), Bound::None(client))
}

fn holders(index: u32) -> (Option<w::EscalationHolder>, Option<c::EscalationHolder>) {
    use c::EscalationHolder as C;
    use w::EscalationHolder as W;
    match index {
        1 => (Some(W::Task(w::EscalationHolderTask { value: 51 })), Some(C::Task(51))),
        2 => (Some(W::Person(w::EscalationHolderPerson { value: 53 })), Some(C::Person(53))),
        3 => (Some(W::Role(w::EscalationHolderRole { project: 7, role: 59 })), Some(C::Role { project: 7, role: 59 })),
        _ => (None, None),
    }
}
