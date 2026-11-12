//! How work runs' shared parts (temper/docs/design/client/pages/settings.md, 2).
use super::fixtures as f;
use crate::{
    Address, Domain, Event, Place, Request,
    pages::how::{self, Budget, Facts, Head, Line, Parts, Path, Reaches, Read},
};
use jig_client::{Down, Readable, Readout, Up, contract as c};
use skein_lib::{List, Queue, Token, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_protocol_reaches::{Class, Classes, Item, Kind, PathSet, Subject, Way};

enum ClassesRead {
    Unread,
    Stock,
    Given(Classes),
}
type ReachesRow = (&'static str, Classes, Box<[Line]>, Box<[Line]>);

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
fn setting_read(out: &Queue<Request>, kind: u32) -> Token {
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Family { record, .. } } },
        } = request
            && let w::Read::Setting(setting) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
            && setting.kind() == kind
        {
            return *read;
        }
    }
    panic!("kind {kind} read")
}
fn me(asking: bool, tracked: bool, several: bool) -> c::Me {
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
    if several {
        let mut other = me.projects.get(0).unwrap().clone();
        other.project = OTHER;
        other.templates = List::with_capacity(0);
        me.projects = List::from_boxed(Box::new([me.projects.get(0).unwrap().clone(), other]));
    }
    me
}
#[expect(clippy::too_many_lines, reason = "one fixture delivers the independent ways, classes and roles answers")]
fn fixture(
    asking: bool,
    tracked: bool,
    ways: Option<(bool, u32)>,
    read: Read,
    roles: Option<w::View>,
    classes: ClassesRead,
    several: bool,
) -> Domain {
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
            event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me(asking, tracked, several)))),
        },
    );
    if let Some((goals, way)) = ways {
        let record =
            w::ProjectWays::new(&w::CEILINGS, w::ProjectWaysParts { drafts: true, goals, changes: false, way })
                .unwrap();
        let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut writer).unwrap();
        let bytes = wire::setting_answer(
            &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(writer.finish()) }).unwrap(),
        );
        up(
            &mut root,
            Up::ReadDone {
                read: setting_read(&out, 7),
                result: Readout::Family { family: wire::FORGE, record: Some(bytes) },
            },
        );
    }
    let classes = match classes {
        ClassesRead::Unread => None,
        ClassesRead::Stock => Some(None),
        ClassesRead::Given(table) => Some(Some(table)),
    };
    if let Some(table) = classes {
        let value = match table {
            Some(table) => {
                let record = temper_protocol_reaches::classes_record(&table).unwrap();
                let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
                record.encode(&mut writer).unwrap();
                Some(writer.finish())
            }
            None => None,
        };
        let answer =
            wire::setting_answer(&w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value }).unwrap());
        up(
            &mut root,
            Up::ReadDone {
                read: setting_read(&out, 8),
                result: Readout::Family { family: wire::FORGE, record: Some(answer) },
            },
        );
    }
    let out = event(&mut root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    if let Some(view) = roles {
        let mut read = None;
        for request in &out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Read { read: token, what: Readable::Family { record, .. } },
                    },
            } = request
                && let w::Read::View(view) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
                && view.address() == b"roles"
            {
                read = Some(*token);
            }
        }
        let read = read.expect("roles read");
        let answer = w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view }).unwrap();
        up(
            &mut root,
            Up::ReadDone {
                read,
                result: Readout::Family { family: wire::FORGE, record: Some(wire::view_answer(&answer)) },
            },
        );
    }
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
                        headline: b"".as_slice().into(),
                        name: b"A way".as_slice().into(),
                        line: b"Its line".as_slice().into(),
                        path: path.clone(),
                        items: List::with_capacity(0),
                        delegation: false,
                        roles: List::with_capacity(0),
                    },
                )
                .unwrap(),
            )
            .unwrap();
            let panel = w::WaysPanel::new(
                &w::CEILINGS,
                w::WaysPanelParts { words: None, project: f::PROJECT, ways, current, own: path, may_change: false },
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
#[expect(
    clippy::too_many_lines,
    reason = "one table covers the independent page facts, their reads and reaches columns"
)]
fn parts_and_facts_follow_the_project_rules() {
    for (read, head, path) in [
        (Read::Reading, Head::Reading, Path::Reading),
        (Read::Ended, Head::Reading, Path::Ended),
        (Read::Shown { current: Some(0) }, Head::Way(0), Path::Shown),
        (Read::Shown { current: Some(2) }, Head::Way(2), Path::Shown),
        (Read::Shown { current: None }, Head::Own, Path::Shown),
    ] {
        for (asking, roles) in [(false, false), (true, false), (false, true), (true, true)] {
            for (classes, several) in [(false, false), (true, false), (false, true), (true, true)] {
                assert_eq!(
                    how::parts(&Facts { roles, asking, read, goals: false, budget: None, classes, several }),
                    Parts { roles, head, ask: asking, path, budget: None, reaches: classes, several },
                    "{read:?}, asking={asking}, roles={roles}, classes={classes}, several={several}"
                );
            }
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
                how::parts(&Facts {
                    roles: false,
                    asking: false,
                    read: Read::Reading,
                    goals,
                    budget,
                    classes: false,
                    several: false
                }),
                Parts {
                    roles: false,
                    head: Head::Reading,
                    ask: false,
                    path: Path::Reading,
                    budget: expected,
                    reaches: false,
                    several: false
                },
                "{budget:?}, goals={goals}"
            );
        }
    }
    for (asking, tracked, ways, read, expected) in [
        (
            false,
            true,
            None,
            Read::Reading,
            Facts {
                roles: false,
                asking: false,
                read: Read::Reading,
                goals: false,
                budget: None,
                classes: false,
                several: false,
            },
        ),
        (
            true,
            true,
            Some((true, SELECTED)),
            Read::Shown { current: Some(0) },
            Facts {
                roles: false,
                asking: true,
                read: Read::Shown { current: Some(0) },
                goals: true,
                budget: Some(c::TemplateBudget::Amount(AMOUNT)),
                classes: false,
                several: false,
            },
        ),
        (
            false,
            true,
            Some((false, SELECTED)),
            Read::Shown { current: None },
            Facts {
                roles: false,
                asking: false,
                read: Read::Shown { current: None },
                goals: false,
                budget: Some(c::TemplateBudget::Amount(AMOUNT)),
                classes: false,
                several: false,
            },
        ),
        (
            true,
            true,
            Some((true, 0)),
            Read::Ended,
            Facts {
                roles: false,
                asking: true,
                read: Read::Ended,
                goals: true,
                budget: Some(c::TemplateBudget::AsAllowed { cap: Some(CAP) }),
                classes: false,
                several: false,
            },
        ),
        (
            true,
            true,
            Some((true, OTHER)),
            Read::Reading,
            Facts {
                roles: false,
                asking: true,
                read: Read::Reading,
                goals: true,
                budget: Some(c::TemplateBudget::AsAllowed { cap: Some(CAP) }),
                classes: false,
                several: false,
            },
        ),
        (
            false,
            false,
            Some((true, SELECTED)),
            Read::Reading,
            Facts {
                roles: false,
                asking: false,
                read: Read::Reading,
                goals: true,
                budget: None,
                classes: false,
                several: false,
            },
        ),
    ] {
        for (classes, several) in [
            (ClassesRead::Unread, false),
            (ClassesRead::Stock, false),
            (ClassesRead::Given(Classes::stock(Way::AgentsReview)), false),
            (ClassesRead::Unread, true),
            (ClassesRead::Stock, true),
        ] {
            let mut expected = expected;
            expected.classes = match &classes {
                ClassesRead::Unread => false,
                ClassesRead::Stock | ClassesRead::Given(_) => true,
            };
            expected.several = several;
            let mut root = fixture(asking, tracked, ways, read, None, classes, several);
            assert_eq!(how::facts(&root, f::PROJECT), expected);
            assert_eq!(
                how::facts(&root, OTHER),
                Facts {
                    roles: false,
                    asking: false,
                    read: Read::Reading,
                    goals: false,
                    budget: None,
                    classes: false,
                    several
                }
            );
            event(&mut root, Event::Went { place: Place::Address(Address::Repositories { project: f::PROJECT }) });
            assert_eq!(how::facts(&root, f::PROJECT).classes, expected.classes, "classes need no How page");
            assert_eq!(how::facts(&root, f::PROJECT).several, several, "project count needs no How page");
        }
    }
    let kind_line = |kind, item, class| Line { subject: Subject::Kind(kind), item, class };
    let set_line = |number, item, class| Line { subject: Subject::Set(number), item, class };
    let item = |subject, words: &[u8], class| Item { subject, words: words.into(), class };
    let set =
        |number| PathSet { number, name: b"Core".as_slice().into(), patterns: Box::new([b"src/*".as_slice().into()]) };
    let movable = [Kind::Wording, Kind::Retry, Kind::Implementation, Kind::Tests, Kind::Dependency, Kind::Departure];
    let ask_away: Box<[_]> = movable.map(|kind| kind_line(kind, 0, Class::AskAway)).into();
    let all_decide = Classes {
        items: movable.map(|kind| item(Subject::Kind(kind), b"", Class::Decide)).into(),
        sets: Box::default(),
    };
    let mut decide_lines = List::uncapped();
    for (index, kind) in movable.into_iter().enumerate() {
        decide_lines.push(kind_line(kind, u8::try_from(index + 1).unwrap(), Class::Decide)).unwrap();
    }
    let decide_lines = decide_lines.into_boxed();
    let mixed = Classes {
        items: Box::new([
            item(Subject::Kind(Kind::Implementation), b"", Class::Ask),
            item(Subject::Kind(Kind::Dependency), b"", Class::AskAway),
            item(Subject::Kind(Kind::Wording), b"within the spec", Class::Ask),
            item(Subject::Set(1), b"", Class::Ask),
            item(Subject::Kind(Kind::Retry), b"one retry", Class::AskAway),
            item(Subject::Set(1), b"test fixes", Class::Decide),
        ]),
        // The bare set comes first here, but follows the table's items in settings order.
        sets: Box::new([set(2), set(1)]),
    };
    mixed.check().unwrap();
    let unchecked = Classes {
        items: Box::new([
            item(Subject::Kind(Kind::Money), b"within budget", Class::Decide),
            item(Subject::Kind(Kind::Retry), b"", Class::Wait),
            item(Subject::Set(3), b"", Class::Ask),
        ]),
        sets: Box::default(),
    };
    let mut oversized_items = List::uncapped();
    for _ in 0..u8::MAX {
        oversized_items.push(item(Subject::Kind(Kind::Money), b"", Class::Decide)).unwrap();
    }
    oversized_items.push(item(Subject::Kind(Kind::Retry), b"one retry", Class::Ask)).unwrap();
    let reaches_rows: [ReachesRow; 6] = [
        (
            "Agents review",
            Classes::stock(Way::AgentsReview),
            Box::new([
                kind_line(Kind::Wording, 1, Class::Decide),
                kind_line(Kind::Retry, 2, Class::Decide),
                kind_line(Kind::Tests, 3, Class::Decide),
            ]),
            Box::new([
                kind_line(Kind::Implementation, 4, Class::Ask),
                kind_line(Kind::Dependency, 5, Class::Ask),
                kind_line(Kind::Departure, 6, Class::Ask),
            ]),
        ),
        ("You approve", Classes::stock(Way::YouApprove), Box::default(), ask_away.clone()),
        (
            "mixed",
            mixed,
            Box::new([set_line(1, 6, Class::Decide)]),
            Box::new([
                kind_line(Kind::Wording, 0, Class::AskAway),
                kind_line(Kind::Retry, 0, Class::AskAway),
                kind_line(Kind::Tests, 0, Class::AskAway),
                kind_line(Kind::Dependency, 2, Class::AskAway),
                kind_line(Kind::Departure, 0, Class::AskAway),
                kind_line(Kind::Retry, 5, Class::AskAway),
                set_line(2, 0, Class::AskAway),
                kind_line(Kind::Implementation, 1, Class::Ask),
                kind_line(Kind::Wording, 3, Class::Ask),
                set_line(1, 4, Class::Ask),
            ]),
        ),
        ("all decide", all_decide, decide_lines, Box::default()),
        (
            "unchecked",
            unchecked,
            Box::default(),
            Box::new([
                kind_line(Kind::Wording, 0, Class::AskAway),
                kind_line(Kind::Implementation, 0, Class::AskAway),
                kind_line(Kind::Tests, 0, Class::AskAway),
                kind_line(Kind::Dependency, 0, Class::AskAway),
                kind_line(Kind::Departure, 0, Class::AskAway),
            ]),
        ),
        (
            "item number does not fit",
            Classes { items: oversized_items.into_boxed(), sets: Box::default() },
            Box::default(),
            ask_away,
        ),
    ];
    for (name, classes, decide, ask) in reaches_rows {
        assert_eq!(
            how::reaches(&classes),
            Reaches {
                decide,
                ask,
                wait: Box::new([
                    kind_line(Kind::Money, 0, Class::Wait),
                    kind_line(Kind::Irreversible, 0, Class::Wait),
                    kind_line(Kind::Rules, 0, Class::Wait),
                    kind_line(Kind::Other, 0, Class::Wait),
                ]),
            },
            "{name}: columns keep settings order and omit unchecked entries"
        );
    }
    let card = w::Card::new(
        &w::CEILINGS,
        w::CardParts {
            name: b"Chat".as_slice().into(),
            who: b"For each chat".as_slice().into(),
            facts: List::with_capacity(0),
        },
    )
    .unwrap();
    let charter = w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 2, card }).unwrap();
    let roles = |rows| {
        w::View::Roles(w::RolesView::new(&w::CEILINGS, w::RolesViewParts { roles: List::from_boxed(rows) }).unwrap())
    };
    for (answer, expected) in [
        (None, false),
        (Some(roles(Box::default())), false),
        (Some(roles(Box::new([charter.clone()]))), true),
        (Some(w::View::Charter(charter)), false),
        (
            Some(w::View::NotDrawn(
                w::NotDrawn::new(&w::CEILINGS, w::NotDrawnParts { words: b"Not drawn".as_slice().into() }).unwrap(),
            )),
            false,
        ),
    ] {
        let mut root = fixture(true, false, None, Read::Reading, answer, ClassesRead::Unread, false);
        assert_eq!(how::facts(&root, f::PROJECT).roles, expected);
        assert_eq!(how::parts(&how::facts(&root, f::PROJECT)).roles, expected);
        assert!(!how::facts(&root, OTHER).roles);
        event(&mut root, Event::Went { place: Place::Address(Address::Repositories { project: f::PROJECT }) });
        assert!(root.pages().roles().is_none(), "Repositories reads no roles");
        assert!(!how::facts(&root, f::PROJECT).roles);
    }
    let root = crate::unlinked(&f::limits(), f::SEED);
    assert_eq!(
        how::facts(&root, f::PROJECT),
        Facts {
            roles: false,
            asking: false,
            read: Read::Reading,
            goals: false,
            budget: None,
            classes: false,
            several: false
        }
    );
}
