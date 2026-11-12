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
            deliver_ways(&mut root, watch, current, List::with_capacity(0));
        }
    }
    root
}
fn deliver_ways(root: &mut Domain, watch: Token, current: Option<u32>, rows: List<w::PathRule>) {
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
            },
        )
        .unwrap(),
    )
    .unwrap();
    let panel = w::WaysPanel::new(
        &w::CEILINGS,
        w::WaysPanelParts {
            rules: w::PathRules::new(&w::CEILINGS, w::PathRulesParts { rows, note: Box::default() }).unwrap(),
            words: None,
            project: f::PROJECT,
            ways,
            current,
            own: path,
            may_change: false,
        },
    )
    .unwrap();
    up(
        root,
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
    let mut project = super::team::project(Box::default(), false);
    let began = skein_lib::Wall::from_nanos(10);
    for (next, week) in
        [(Some(604_800_000_000_010), true), (Some(2_592_000_000_000_010), false), (None, false), (Some(9), false)]
    {
        let next = match next {
            Some(next) => Some(skein_lib::Wall::from_nanos(next)),
            None => None,
        };
        project.current_period = Some(c::PoolLedger {
            source: c::Funder::Period { project: f::PROJECT, period: 1 },
            parent: None,
            numbers: c::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
            left: 100,
            closed: false,
            began: Some(began),
            next,
        });
        assert_eq!(how::period(&project), Some(how::Period { week, next }));
    }
    project.current_period = None;
    assert_eq!(how::period(&project), None);
    let mut task = super::chats::chat_line();
    task.label = 3;
    task.chip.task = 9;
    task.budget.left = 28;
    task.budget.numbers.budget = 40;
    let mut first = task.clone();
    first.chip.task = 2;
    let mut ended = task.clone();
    ended.chip.task = 1;
    ended.ended_at = Some(began);
    let rows = how::standing(&[task.clone(), ended, first.clone()], f::PROJECT);
    assert_eq!(
        rows.as_ref(),
        &[
            how::Standing { number: 2, title: first.chip.title.bytes().into(), left: 28, budget: 40 },
            how::Standing { number: 9, title: task.chip.title.bytes().into(), left: 28, budget: 40 }
        ]
    );
    let facts = Facts {
        rules: List::with_capacity(0),
        period: Some(how::Period { week: true, next: Some(began) }),
        standing: rows,
        roles: false,
        asking: false,
        read: Read::Reading,
        goals: false,
        budget: Some(c::TemplateBudget::Amount(80)),
        classes: false,
        several: false,
    };
    let parts = how::parts(&facts);
    assert_eq!(parts.period, facts.period);
    assert_eq!(parts.standing, facts.standing);
    assert_eq!(parts.budget, None);

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
                    how::parts(&Facts {
                        rules: List::with_capacity(0),
                        period: None,
                        standing: Box::default(),
                        roles,
                        asking,
                        read,
                        goals: false,
                        budget: None,
                        classes,
                        several
                    }),
                    Parts {
                        rules: None,
                        period: None,
                        standing: Box::default(),
                        roles,
                        head,
                        ask: asking,
                        path,
                        budget: None,
                        reaches: classes,
                        several
                    },
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
                    rules: List::with_capacity(0),
                    period: None,
                    standing: Box::default(),
                    roles: false,
                    asking: false,
                    read: Read::Reading,
                    goals,
                    budget,
                    classes: false,
                    several: false
                }),
                Parts {
                    rules: None,
                    period: None,
                    standing: Box::default(),
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
                rules: List::with_capacity(0),
                period: None,
                standing: Box::default(),
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
                rules: List::with_capacity(0),
                period: None,
                standing: Box::default(),
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
                rules: List::with_capacity(0),
                period: None,
                standing: Box::default(),
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
                rules: List::with_capacity(0),
                period: None,
                standing: Box::default(),
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
                rules: List::with_capacity(0),
                period: None,
                standing: Box::default(),
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
                rules: List::with_capacity(0),
                period: None,
                standing: Box::default(),
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
            let mut expected = expected.clone();
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
                    rules: List::with_capacity(0),
                    period: None,
                    standing: Box::default(),
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
            rules: List::with_capacity(0),
            period: None,
            standing: Box::default(),
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

#[test]
fn rules_follow_the_shown_rows_and_ask_offering() {
    for asking in [false, true] {
        let mut root = fixture(asking, false, None, Read::Reading, None, ClassesRead::Unread, false);
        event(&mut root, Event::Went { place: Place::Address(Address::Repositories { project: f::PROJECT }) });
        let out = event(&mut root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
        let watch = opened(&out);
        up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
        assert_eq!(how::parts(&how::facts(&root, f::PROJECT)).rules, None, "reading");
        for (reviews, review_column) in
            [(&[][..], false), (&[b"".as_slice()][..], false), (&[b"".as_slice(), b"an agent"][..], true)]
        {
            let mut rows = List::with_capacity(u32::try_from(reviews.len()).unwrap());
            for review in reviews {
                rows.push(
                    w::PathRule::new(
                        &w::CEILINGS,
                        w::PathRuleParts {
                            paths: b"payments/**".as_slice().into(),
                            review: (*review).into(),
                            sign_off: b"two of you".as_slice().into(),
                            lands: b"once two have signed this head".as_slice().into(),
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
            }
            deliver_ways(&mut root, watch, None, rows);
            let facts = how::facts(&root, f::PROJECT);
            assert_eq!(facts.rules.len(), u32::try_from(reviews.len()).unwrap());
            let parts = how::parts(&facts);
            let expected =
                if reviews.is_empty() { None } else { Some(how::Rules { add: asking, review: review_column }) };
            assert_eq!(parts.rules, expected);
            if let Some(rules) = parts.rules {
                assert_eq!(rules.add, parts.ask);
            }
            assert_eq!(how::parts(&Facts { read: Read::Reading, ..facts.clone() }).rules, None);
            assert_eq!(how::parts(&Facts { read: Read::Ended, ..facts }).rules, None);
        }
        up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded });
        let facts = how::facts(&root, f::PROJECT);
        assert!(facts.rules.is_empty());
        assert_eq!(how::parts(&facts).rules, None, "ended after rows were shown");
    }
}

#[expect(clippy::disallowed_methods, reason = "ordinary fixture maps owned wire fields")]
fn follow_row(paths: &[&[u8]], start: w::FollowStart, lag: u32, verdict: Option<w::FollowVerdict>) -> w::FollowView {
    let repo = w::FollowRepository::new(&w::CEILINGS, w::FollowRepositoryParts { forge: 1, repository: 7 }).unwrap();
    let setting = w::Follow::new(
        &w::CEILINGS,
        w::FollowParts {
            source: w::FollowSource::new(
                &w::CEILINGS,
                w::FollowSourceParts {
                    repository: repo.clone(),
                    branch: b"main".as_slice().into(),
                    paths: List::from_boxed(paths.iter().map(|p| Box::from(*p)).collect()),
                },
            )
            .unwrap(),
            target: w::FollowTarget::new(
                &w::CEILINGS,
                w::FollowTargetParts { repository: repo, branch: b"main".as_slice().into() },
            )
            .unwrap(),
            owner: w::FollowOwner::Person(
                w::FollowPerson::new(&w::CEILINGS, w::FollowPersonParts { person: 4 }).unwrap(),
            ),
            start,
            producer: None,
            verification: w::FollowCheck::new(
                &w::CEILINGS,
                w::FollowCheckParts {
                    workflow: b".forgejo/workflows/vsr-tests.yml".as_slice().into(),
                    artifact: b"results".as_slice().into(),
                    format: w::FollowFormat::Junit,
                },
            )
            .unwrap(),
        },
    )
    .unwrap();
    let parity = verdict.map(|verdict| {
        w::FollowParity::new(
            &w::CEILINGS,
            w::FollowParityParts { source: [0xab; 32], head: [0xcd; 32], verdict, run: b"check".as_slice().into() },
        )
        .unwrap()
    });
    w::FollowView::new(
        &w::CEILINGS,
        w::FollowViewParts {
            number: 1,
            setting,
            source: b"cowsql/vsr".as_slice().into(),
            source_branch: b"main".as_slice().into(),
            target: b"cowsql/vsr-rs".as_slice().into(),
            target_branch: b"main".as_slice().into(),
            task: None,
            stopped: None,
            lag,
            entries: List::with_capacity(0),
            skips: List::with_capacity(0),
            parity,
        },
    )
    .unwrap()
}
fn tier_stop(name: &[u8], checks: Option<&[u8]>, step: w::PathStep) -> w::Stop {
    let mut facts = List::with_capacity(1);
    if let Some(checks) = checks {
        facts
            .push(
                w::Fact::new(&w::CEILINGS, w::FactParts { label: b"Checks".as_slice().into(), value: checks.into() })
                    .unwrap(),
            )
            .unwrap();
    }
    w::Stop::new(
        &w::CEILINGS,
        w::StopParts {
            step,
            actor: w::Actor::Code,
            words: b"verify".as_slice().into(),
            needed: false,
            over: Box::default(),
            opens: w::Opens::Tier(w::OpensTier::new(&w::CEILINGS, w::OpensTierParts { name: name.into() }).unwrap()),
            card: w::Card::new(&w::CEILINGS, w::CardParts { name: name.into(), who: Box::default(), facts }).unwrap(),
        },
    )
    .unwrap()
}
#[test]
#[expect(clippy::disallowed_macros, clippy::disallowed_types, reason = "ordinary table varies optional member lists")]
#[expect(clippy::too_many_lines, reason = "one table checks owners, paths, parity and tier selection")]
fn follows_boxes_follow_only_the_read_documents() {
    use how::{Follow, Landed, Owner, Tier, Tiers};
    let pat = super::team::member(4, c::PartyKind::Person, b"Pat", c::HeldRole::Member);
    let login = super::team::member(4, c::PartyKind::Person, b"", c::HeldRole::Member);
    let main = w::BranchPath::new(
        &w::CEILINGS,
        w::BranchPathParts {
            repository: None,
            branch: b"main".as_slice().into(),
            stops: List::from_boxed(Box::new([
                tier_stop(b"fuzz", Some(b"fuzzing for 6 hours"), w::PathStep::Verify),
                tier_stop(b"msan", None, w::PathStep::Verify),
                tier_stop(b"goal-check", None, w::PathStep::Check),
            ])),
        },
    )
    .unwrap();
    let tiers = Tiers {
        branch: b"main".as_slice().into(),
        tiers: Box::new([
            Tier { name: b"fuzz".as_slice().into(), checks: Some(b"fuzzing for 6 hours".as_slice().into()) },
            Tier { name: b"msan".as_slice().into(), checks: None },
        ]),
    };
    let base = Follow {
        source: b"cowsql/vsr".as_slice().into(),
        source_branch: b"main".as_slice().into(),
        target: b"cowsql/vsr-rs".as_slice().into(),
        target_branch: b"main".as_slice().into(),
        paths: Box::new([b"src".as_slice().into(), b"include".as_slice().into()]),
        owner: Owner::Named(b"Pat".as_slice().into()),
        code_starts: false,
        workflow: b".forgejo/workflows/vsr-tests.yml".as_slice().into(),
        lag: 2,
        parity: b"check passes at abababababab".as_slice().into(),
        stopped: None,
    };
    let passed = follow_row(&[b"src", b"include"], w::FollowStart::Owner, 2, Some(w::FollowVerdict::Passed));
    let code = follow_row(&[], w::FollowStart::Code, 0, None);
    let failing = follow_row(
        &[b"src", b"include", b"tests"],
        w::FollowStart::Owner,
        1,
        Some(w::FollowVerdict::Failing(
            w::FollowFailing::new(&w::CEILINGS, w::FollowFailingParts { count: 7 }).unwrap(),
        )),
    );
    let mut stopped = follow_row(&[], w::FollowStart::Owner, 0, Some(w::FollowVerdict::Many)).into_parts();
    stopped.stopped = Some(b"Check is missing".as_slice().into());
    let stopped = w::FollowView::new(&w::CEILINGS, stopped).unwrap();
    let panel = |row| {
        w::FollowsPanel::new(
            &w::CEILINGS,
            w::FollowsPanelParts {
                project: f::PROJECT,
                follows: List::from_boxed(Box::new([row])),
                dependencies: List::with_capacity(0),
            },
        )
        .unwrap()
    };
    for (name, panel, path, members, reader, expected) in [
        (
            "Pat and tiers",
            Some(panel(passed.clone())),
            Some(&main),
            Some(vec![pat.clone()]),
            2,
            Landed { follows: Box::new([base.clone()]), tiers: Some(tiers.clone()) },
        ),
        (
            "reader owns",
            Some(panel(passed.clone())),
            None,
            None,
            4,
            Landed { follows: Box::new([Follow { owner: Owner::You, ..base.clone() }]), tiers: None },
        ),
        (
            "login",
            Some(panel(passed.clone())),
            None,
            Some(vec![login]),
            2,
            Landed {
                follows: Box::new([Follow { owner: Owner::Named(b"login".as_slice().into()), ..base.clone() }]),
                tiers: None,
            },
        ),
        (
            "members unread",
            Some(panel(passed.clone())),
            None,
            None,
            2,
            Landed { follows: Box::new([Follow { owner: Owner::Unknown, ..base.clone() }]), tiers: None },
        ),
        (
            "owner absent",
            Some(panel(passed)),
            None,
            Some(vec![]),
            2,
            Landed { follows: Box::new([Follow { owner: Owner::Unknown, ..base.clone() }]), tiers: None },
        ),
        (
            "code, no paths, no parity",
            Some(panel(code)),
            None,
            Some(vec![pat.clone()]),
            2,
            Landed {
                follows: Box::new([Follow {
                    paths: Box::default(),
                    code_starts: true,
                    lag: 0,
                    parity: b"no check yet".as_slice().into(),
                    ..base.clone()
                }]),
                tiers: None,
            },
        ),
        (
            "three paths, failing",
            Some(panel(failing)),
            None,
            Some(vec![pat.clone()]),
            2,
            Landed {
                follows: Box::new([Follow {
                    paths: Box::new([
                        b"src".as_slice().into(),
                        b"include".as_slice().into(),
                        b"tests".as_slice().into(),
                    ]),
                    lag: 1,
                    parity: b"7 failing at abababababab".as_slice().into(),
                    ..base.clone()
                }]),
                tiers: None,
            },
        ),
        (
            "stopped, many",
            Some(panel(stopped)),
            None,
            Some(vec![pat]),
            2,
            Landed {
                follows: Box::new([Follow {
                    paths: Box::default(),
                    lag: 0,
                    parity: b"many failing at abababababab".as_slice().into(),
                    stopped: Some(b"Check is missing".as_slice().into()),
                    ..base
                }]),
                tiers: None,
            },
        ),
        ("no follows read", None, Some(&main), None, 2, Landed { follows: Box::default(), tiers: Some(tiers) }),
        ("neither read", None, None, None, 2, Landed { follows: Box::default(), tiers: None }),
    ] {
        assert_eq!(how::follows(panel.as_ref(), path, members.as_deref(), reader), expected, "{name}");
    }
}
