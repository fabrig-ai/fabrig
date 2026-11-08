//! How work runs' shared parts (temper/docs/design/client/README.md, 5.13).
use super::fixtures as f;
use crate::{
    Address, Domain, Event, Place, Request,
    pages::how::{self, Budget, Facts, Head, Parts, Path, Read},
};
use jig_client::{Down, Readable, Readout, Up, contract as c};
use skein_lib::{List, Queue, Token, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};

const OTHER: u32 = 17;
const FIRST: u32 = 2;
const SELECTED: u32 = 4;
const AMOUNT: u64 = 77;
const CAP: u64 = 9;
// A person, agent and code on each side of the goal branch's bar, then main's bar.
const STOPS: u32 = 8;
// Two tracked templates and Ask temper, where listed.
const TEMPLATES: u32 = 3;

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn opened(out: &Queue<Request>) -> Token {
    let wanted = crate::forge::stock_ways::watch(f::PROJECT, &f::limits().forge).unwrap();
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
            && *subject == wanted
        {
            return *watch;
        }
    }
    panic!("project ways watch")
}
fn setting_read(out: &Queue<Request>) -> Token {
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Family { record, .. } } },
        } = request
            && let w::Read::Setting(setting) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
            && setting.kind() == 7
        {
            return *read;
        }
    }
    panic!("kind 7 read")
}
fn me(asking: bool, tracked: bool) -> c::Me {
    let mut me = f::chats::me();
    let member = me.projects.get_mut(0).unwrap();
    let mut template = member.templates.get(0).unwrap().clone();
    member.templates = List::with_capacity(TEMPLATES);
    // Selection differs from the lowest template, and delivery order differs from number order.
    for (number, budget) in
        [(SELECTED, c::TemplateBudget::Amount(AMOUNT)), (FIRST, c::TemplateBudget::AsAllowed { cap: Some(CAP) })]
    {
        template.number = number;
        template.budget = budget;
        template.tracked = tracked.then_some(1);
        member.templates.push(template.clone()).unwrap();
    }
    if asking {
        template.number = crate::pages::chats::ASKING_TEMPLATE;
        template.tracked = None;
        member.templates.push(template).unwrap();
    }
    me
}
fn fixture(asking: bool, tracked: bool, ways: Option<(bool, u32)>, read: Read) -> Domain {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me_watch = f::start(&mut root, List::with_capacity(0));
    up(
        &mut root,
        Up::FamilyServed {
            family: wire::FORGE,
            versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
        },
    );
    let out = up(
        &mut root,
        Up::Watched {
            watch: me_watch,
            event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me(asking, tracked)))),
        },
    );
    if let Some((goals, way)) = ways {
        let record =
            w::ProjectWays::new(&w::CEILINGS, w::ProjectWaysParts { skein_bools: [true, goals, false, false], way })
                .unwrap();
        let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut writer).unwrap();
        let bytes = wire::setting_answer(
            &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(writer.finish()) }).unwrap(),
        );
        up(
            &mut root,
            Up::ReadDone {
                read: setting_read(&out),
                result: Readout::Family { family: wire::FORGE, record: Some(bytes) },
            },
        );
    }
    let out = event(&mut root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    let watch = opened(&out);
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    match read {
        Read::Reading => {}
        Read::Ended => {
            up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded });
        }
        Read::Shown { current } => {
            let path = path();
            let mut ways = List::with_capacity(1);
            ways.push(
                w::StockWay::new(
                    &w::CEILINGS,
                    w::StockWayParts {
                        name: b"A way".as_slice().into(),
                        line: b"Its line".as_slice().into(),
                        path: path.clone(),
                        items: List::with_capacity(0),
                        delegation: false,
                    },
                )
                .unwrap(),
            )
            .unwrap();
            let panel = w::WaysPanel::new(
                &w::CEILINGS,
                w::WaysPanelParts { project: f::PROJECT, ways, current, own: path, may_change: false },
            )
            .unwrap();
            up(
                &mut root,
                Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::ways_panel(&panel),
                    },
                },
            );
        }
    }
    root
}
fn path() -> w::BranchPath {
    let mut stops = List::with_capacity(STOPS);
    for (actor, step, words) in [
        (w::Actor::Person, w::PathStep::Draft, b"draft".as_slice()),
        (w::Actor::Agent, w::PathStep::Produce, b"produce".as_slice()),
        (w::Actor::Code, w::PathStep::Check, b"CI".as_slice()),
        (w::Actor::Bar, w::PathStep::Branch, b"goal branch".as_slice()),
        (w::Actor::Person, w::PathStep::Approve, b"you approve".as_slice()),
        (w::Actor::Agent, w::PathStep::Resolve, b"resolve".as_slice()),
        (w::Actor::Code, w::PathStep::Merge, b"queue".as_slice()),
        (w::Actor::Bar, w::PathStep::Branch, b"main".as_slice()),
    ] {
        stops
            .push(
                w::Stop::new(
                    &w::CEILINGS,
                    w::StopParts {
                        needed: false,
                        over: Box::default(),
                        actor,
                        step,
                        words: words.into(),
                        opens: w::Opens::Nothing,
                        card: w::Card::new(
                            &w::CEILINGS,
                            w::CardParts {
                                who: b"Actor".as_slice().into(),
                                name: Box::default(),
                                facts: List::with_capacity(0),
                            },
                        )
                        .unwrap(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    w::BranchPath::new(&w::CEILINGS, w::BranchPathParts { repository: None, branch: b"main".as_slice().into(), stops })
        .unwrap()
}

#[test]
fn parts_and_facts_follow_the_project_rules() {
    for (read, head, path) in [
        (Read::Reading, Head::Reading, Path::Reading),
        (Read::Ended, Head::Reading, Path::Ended),
        (Read::Shown { current: Some(0) }, Head::Way(0), Path::Shown),
        (Read::Shown { current: Some(2) }, Head::Way(2), Path::Shown),
        (Read::Shown { current: None }, Head::Own, Path::Shown),
    ] {
        for asking in [false, true] {
            assert_eq!(
                how::parts(&Facts { asking, read, goals: false, budget: None }),
                Parts { head, ask: asking, path, budget: None },
                "{read:?}, asking={asking}"
            );
        }
    }
    for (budget, shown) in [
        (None, None),
        (Some(c::TemplateBudget::Given), None),
        (Some(c::TemplateBudget::Amount(AMOUNT)), Some(Budget::Amount(AMOUNT))),
        (Some(c::TemplateBudget::Amount(0)), Some(Budget::Amount(0))),
        (Some(c::TemplateBudget::AsAllowed { cap: None }), None),
        (Some(c::TemplateBudget::AsAllowed { cap: Some(CAP) }), Some(Budget::UpTo(CAP))),
        (Some(c::TemplateBudget::AsAllowed { cap: Some(0) }), Some(Budget::UpTo(0))),
    ] {
        for (goals, expected) in [(false, None), (true, shown)] {
            assert_eq!(
                how::parts(&Facts { asking: false, read: Read::Reading, goals, budget }),
                Parts { head: Head::Reading, ask: false, path: Path::Reading, budget: expected },
                "{budget:?}, goals={goals}"
            );
        }
    }
    for (asking, tracked, ways, read, expected) in [
        (false, true, None, Read::Reading, Facts { asking: false, read: Read::Reading, goals: false, budget: None }),
        (
            true,
            true,
            Some((true, SELECTED)),
            Read::Shown { current: Some(0) },
            Facts {
                asking: true,
                read: Read::Shown { current: Some(0) },
                goals: true,
                budget: Some(c::TemplateBudget::Amount(AMOUNT)),
            },
        ),
        (
            false,
            true,
            Some((false, SELECTED)),
            Read::Shown { current: None },
            Facts {
                asking: false,
                read: Read::Shown { current: None },
                goals: false,
                budget: Some(c::TemplateBudget::Amount(AMOUNT)),
            },
        ),
        (
            true,
            true,
            Some((true, 0)),
            Read::Ended,
            Facts {
                asking: true,
                read: Read::Ended,
                goals: true,
                budget: Some(c::TemplateBudget::AsAllowed { cap: Some(CAP) }),
            },
        ),
        (
            true,
            true,
            Some((true, OTHER)),
            Read::Reading,
            Facts {
                asking: true,
                read: Read::Reading,
                goals: true,
                budget: Some(c::TemplateBudget::AsAllowed { cap: Some(CAP) }),
            },
        ),
        (
            false,
            false,
            Some((true, SELECTED)),
            Read::Reading,
            Facts { asking: false, read: Read::Reading, goals: true, budget: None },
        ),
    ] {
        let root = fixture(asking, tracked, ways, read);
        assert_eq!(how::facts(&root, f::PROJECT), expected);
        assert_eq!(how::facts(&root, OTHER), Facts { asking: false, read: Read::Reading, goals: false, budget: None });
    }
    let root = crate::unlinked(&f::limits(), f::SEED);
    assert_eq!(how::facts(&root, f::PROJECT), Facts { asking: false, read: Read::Reading, goals: false, budget: None });
}
