//! Team's drawn facts (temper/docs/design/client/pages/team.md, 1).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests build table rows and inspect semantic subtrees"
)]
use super::fixtures as f;
use crate::{look, pages::team::draw};
use jig_client::contract::HeldRole;
use skein_lib::List;
use skein_ui::{Builder, Role, Run};
use temper_client_domain::pages::team::{Facts, Person, Shown, Waits};

const READER: u64 = 7;
const SAM: u64 = 8;
type BoxWords = (&'static str, &'static str, Option<&'static str>);
struct Row {
    label: &'static str,
    shown: Shown,
    lede: Option<&'static str>,
    unreadable: bool,
    shared: Option<&'static str>,
    boxes: Vec<BoxWords>,
}
fn person(you: bool, name: &[u8], role: HeldRole, waits: Option<Waits>) -> Person {
    Person { party: if you { READER } else { SAM }, you, name: name.into(), role, waits, signed: None }
}
fn read(label: &'static str, people: Vec<Person>, cut: bool, lede: &'static str, boxes: Vec<BoxWords>) -> Row {
    Row {
        label,
        shown: Shown::Read(Facts {
            people: List::from_boxed(people.into_boxed_slice()),
            cut,
            triager: None,
            follows: List::with_capacity(0),
            roles: List::with_capacity(0),
            roles_cut: false,
            selected: None,
            card: None,
        }),
        lede: Some(lede),
        unreadable: false,
        shared: None,
        boxes,
    }
}
fn pair(label: &'static str, waits: Option<Waits>, line: Option<&'static str>) -> Row {
    read(
        label,
        vec![
            person(
                true,
                b"Alex",
                HeldRole::Owner,
                Some(Waits { questions: 1, reviews: 1, proposals: 1, ..Waits::default() }),
            ),
            person(false, b"Sam", HeldRole::Member, waits),
        ],
        false,
        "Two of you.",
        vec![("You", "Owner", Some("Three things wait on you.")), ("Sam", "Member", line)],
    )
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table covers Team's states, counts, roles and five wait kinds")]
fn team_draws_what_it_shows() {
    let mut rows = vec![
        Row { label: "reading", shown: Shown::Reading, lede: None, unreadable: false, shared: None, boxes: vec![] },
        Row {
            label: "unreadable",
            shown: Shown::Unreadable,
            lede: None,
            unreadable: true,
            shared: None,
            boxes: vec![],
        },
        pair("two people", Some(Waits { questions: 1, ..Waits::default() }), Some("One question waits on Sam.")),
        pair("unread waits", None, None),
        pair(
            "cut waits",
            Some(Waits { questions: 2, cut: true, ..Waits::default() }),
            Some("2+ questions wait on Sam."),
        ),
        pair(
            "cut singular",
            Some(Waits { questions: 1, cut: true, ..Waits::default() }),
            Some("1+ question waits on Sam."),
        ),
        read(
            "alone",
            vec![person(true, b"Alex", HeldRole::Owner, Some(Waits::default()))],
            false,
            "Just you.",
            vec![("You", "Owner", Some("Nothing waits on you."))],
        ),
        read(
            "reader absent",
            vec![person(false, b"Sam", HeldRole::Member, None), person(false, b"Kit", HeldRole::Observer, None)],
            false,
            "Two people.",
            vec![("Sam", "Member", None), ("Kit", "Observer", None)],
        ),
        read(
            "maintainer",
            vec![person(false, b"Sam", HeldRole::Maintainer, None)],
            false,
            "One person.",
            vec![("Sam", "Maintainer", None)],
        ),
        read(
            "policy",
            vec![person(false, b"Sam", HeldRole::Policy { role: 12 }, None)],
            false,
            "One person.",
            vec![("Sam", "Role 12", None)],
        ),
        read("empty document", vec![], false, "0 people.", vec![]),
    ];
    for (role, word, shared, cut) in [
        (HeldRole::Owner, "Owner", Some("owners of shop"), false),
        (HeldRole::Maintainer, "Maintainer", Some("maintainers of shop"), false),
        (HeldRole::Member, "Member", Some("members of shop"), false),
        (HeldRole::Observer, "Observer", Some("observers of shop"), false),
        (HeldRole::Owner, "Owner", None, true),
        (HeldRole::Policy { role: 12 }, "Role 12", None, false),
    ] {
        let mut row = read(
            "shared role",
            vec![person(true, b"Alex", role, None), person(false, b"Sam", role, None)],
            cut,
            if cut { "2 or more of you." } else { "Two of you." },
            vec![("You", word, None), ("Sam", word, None)],
        );
        row.shared = shared;
        rows.push(row);
    }
    for (count, line) in [(1, "One question waits on you."), (2, "Two questions wait on you.")] {
        rows.push(read(
            line,
            vec![person(true, b"Alex", HeldRole::Owner, Some(Waits { questions: count, ..Waits::default() }))],
            false,
            "Just you.",
            vec![("You", "Owner", Some(line))],
        ));
    }
    // Each row pins the grammar of one kind at one and at more than one.
    for (waits, said) in [
        (Waits { questions: 2, ..Waits::default() }, "Two questions wait on Sam."),
        (Waits { reviews: 1, ..Waits::default() }, "One review waits on Sam."),
        (Waits { reviews: 2, ..Waits::default() }, "Two reviews wait on Sam."),
        (Waits { proposals: 1, ..Waits::default() }, "One proposal waits on Sam."),
        (Waits { proposals: 2, ..Waits::default() }, "Two proposals wait on Sam."),
        (Waits { held: 1, ..Waits::default() }, "One held task waits on Sam."),
        (Waits { held: 2, ..Waits::default() }, "Two held tasks wait on Sam."),
        (Waits { person_tasks: 1, ..Waits::default() }, "One person task waits on Sam."),
        (Waits { person_tasks: 2, ..Waits::default() }, "Two person tasks wait on Sam."),
        (Waits { reviews: 1, held: 1, cut: true, ..Waits::default() }, "2+ things wait on Sam."),
        (Waits { questions: u32::MAX, reviews: 1, ..Waits::default() }, "4294967295 things wait on Sam."),
        (Waits { questions: 9, ..Waits::default() }, "Nine questions wait on Sam."),
        (Waits { questions: 10, ..Waits::default() }, "10 questions wait on Sam."),
        (Waits { questions: 12, ..Waits::default() }, "12 questions wait on Sam."),
        (Waits::default(), "Nothing waits on Sam."),
    ] {
        rows.push(pair(said, Some(waits), Some(said)));
    }
    // Distinct people in the document's order; the model has placed the reader first.
    for (count, you, cut, said) in [
        (3, true, false, "Three of you."),
        (4, true, false, "Four of you."),
        (5, true, false, "5 of you."),
        (5, true, true, "5 or more of you."),
        (3, false, false, "Three people."),
        (4, false, false, "Four people."),
        (5, false, false, "5 people."),
        (5, false, true, "5 or more people."),
        (1, true, true, "1 or more of you."),
        (1, false, true, "1 or more people."),
    ] {
        let mut people = vec![person(you, b"Alex", HeldRole::Owner, None)];
        let mut boxes = vec![(if you { "You" } else { "Alex" }, "Owner", None)];
        for (name, party) in [("Sam", 8), ("Kit", 9), ("Jo", 10), ("Lee", 11)].into_iter().take(count - 1) {
            let mut member = person(false, name.as_bytes(), HeldRole::Member, None);
            member.party = party;
            people.push(member);
            boxes.push((name, "Member", None));
        }
        rows.push(read(said, people, cut, said, boxes));
    }
    for w in [f::browser_words(), f::words()] {
        for row in &rows {
            let mut b = Builder::new(&f::tree_limits());
            draw(
                &mut b,
                &w,
                b"shop",
                &row.shown,
                7,
                &mut crate::frame::Bindings::new(64),
                &mut crate::words::Times::new(skein_lib::Wall::EPOCH),
            );
            let tree = b.finish();
            let nodes = tree.nodes();
            for (role, name) in [
                (Role::Main, None),
                (Role::Heading { level: 1 }, Some("Who owns what on shop")),
                (Role::Heading { level: 2 }, Some("People")),
            ] {
                assert_eq!(
                    nodes.iter().filter(|n| n.role == role && n.name.as_deref() == name.map(str::as_bytes)).count(),
                    1,
                    "{} {role:?}",
                    row.label
                );
            }
            let head_at =
                nodes.iter().position(|n| n.styles.as_slice().contains(&look::SETTINGS_SECTION_HEAD)).unwrap();
            let head = &nodes[head_at..head_at + usize::try_from(nodes[head_at].size).unwrap()];
            let mut head_words = vec![(Role::Heading { level: 2 }, "People")];
            if let Some(shared) = row.shared {
                head_words.push((Role::Text, shared));
                assert!(
                    head.iter().any(|n| n.styles.as_slice().contains(&look::TEAM_SHARED)
                        && n.name.as_deref() == Some(shared.as_bytes())),
                    "{} shared style",
                    row.label
                );
            }
            assert_eq!(
                head.iter()
                    .filter_map(|n| n.name.as_deref().map(|name| (n.role, core::str::from_utf8(name).unwrap())))
                    .collect::<Vec<_>>(),
                head_words,
                "{} head",
                row.label
            );
            let mut paragraphs = vec!["shop · Team"];
            paragraphs.extend(row.lede);
            for (_, role, line) in &row.boxes {
                paragraphs.push(role);
                paragraphs.extend(line);
            }
            if row.unreadable {
                paragraphs.push("This project's people can't be read now.");
            }
            assert_eq!(
                nodes
                    .iter()
                    .filter(|n| n.role == Role::Paragraph)
                    .map(|n| core::str::from_utf8(n.name.as_deref().unwrap()).unwrap())
                    .collect::<Vec<_>>(),
                paragraphs,
                "{}",
                row.label
            );
            let boxes: Vec<_> =
                nodes.iter().enumerate().filter(|(_, n)| n.styles.as_slice().contains(&look::TEAM_PERSON)).collect();
            assert_eq!(boxes.len(), row.boxes.len(), "{}", row.label);
            assert_eq!(
                nodes.iter().filter(|n| n.styles.as_slice().contains(&look::TEAM_PEOPLE)).count(),
                usize::from(!row.boxes.is_empty()),
                "{}",
                row.label
            );
            for ((at, node), (name, role, line)) in boxes.iter().zip(&row.boxes) {
                let children = &nodes[*at..at.checked_add(usize::try_from(node.size).unwrap()).unwrap()];
                for (role_kind, value) in
                    [(Role::Text, Some(*name)), (Role::Paragraph, Some(*role)), (Role::Paragraph, *line)]
                {
                    if let Some(value) = value {
                        assert!(
                            children.iter().any(|n| n.role == role_kind && n.name.as_deref() == Some(value.as_bytes())),
                            "{} {value}",
                            row.label
                        );
                    }
                }
                assert!(
                    children
                        .iter()
                        .any(|n| n.text.iter().any(|r| r.run == Run::Strong && r.text.as_ref() == name.as_bytes())),
                    "{} strong name",
                    row.label
                );
                assert!(
                    children.iter().any(|n| n.styles.as_slice().contains(&look::TEAM_ROLE)
                        && n.name.as_deref() == Some(role.as_bytes())),
                    "{} role",
                    row.label
                );
                assert_eq!(
                    children.iter().filter(|n| n.styles.as_slice().contains(&look::TEAM_LINE)).count(),
                    usize::from(line.is_some()),
                    "{} line",
                    row.label
                );
            }
        }
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "both phrasebooks draw the row, head, controls and Now without empty parts")]
fn team_draws_a_standing_roles_row_head_and_now() {
    use crate::{frame::Bindings, words::Times};
    use skein_lib::Wall;
    use temper_client_domain::{
        Action, Address,
        pages::{Act, team},
    };
    // The gallery's role dates, 2 November and 8 November at 02:41 UTC.
    let made = Wall::from_nanos(1_793_577_600_000_000_000);
    let changed = Wall::from_nanos(1_794_105_660_000_000_000);
    for w in [f::browser_words(), f::words()] {
        let facts = Facts {
            people: List::with_capacity(0),
            cut: false,
            triager: None,
            follows: List::with_capacity(0),
            roles: List::from_boxed(Box::new([team::Role {
                task: 52,
                name: b"Prover".as_slice().into(),
                now: Some(b"- A finding".as_slice().into()),
                spent: 21,
                budget: 40,
            }])),
            roles_cut: true,
            selected: Some(52),
            card: Some(Box::new(team::Card {
                task: 52,
                name: b"Prover".as_slice().into(),
                made: Some(made),
                requester: Some(team::StartedBy::You),
                hold: None,
                paused: false,
                pause: true,
                resume: false,
                sessions: List::with_capacity(0),
                rules: None,
                wakes: List::with_capacity(0),
                wakes_cut: false,
                now: Some(team::Now {
                    body: b"- A finding\nIts next step.\n- Await a change".as_slice().into(),
                    changed: Some(changed),
                    cut: true,
                }),
            })),
        };
        let mut bindings = Bindings::new(f::tree_limits().bindings);
        let mut b = Builder::new(&f::tree_limits());
        draw(&mut b, &w, b"shop", &Shown::Read(facts.clone()), 7, &mut bindings, &mut Times::new(changed));
        let tree = b.finish();
        let nodes = tree.nodes();
        for (role, said) in [
            (Role::Heading { level: 2 }, "Standing roles"),
            (Role::Text, "21 of 40"),
            (Role::Paragraph, "Only the first 16 are shown."),
            (Role::Paragraph, "A standing role · since 2 November · started by you"),
            (Role::Button, "Pause"),
            (Role::Paragraph, "Its Now · written at 02:41"),
            (Role::Item, "A finding"),
            (Role::Paragraph, "Its next step."),
            (Role::Item, "Await a change"),
            (Role::Paragraph, "…"),
        ] {
            assert_eq!(
                nodes.iter().filter(|n| n.role == role && n.name.as_deref() == Some(said.as_bytes())).count(),
                1,
                "{role:?}: {said}"
            );
        }
        assert_eq!(nodes.iter().filter(|n| n.styles.as_slice().contains(&look::TEAM_ROW)).count(), 1, "one role row");
        let at = nodes.iter().position(|n| n.styles.as_slice().contains(&look::TEAM_ROW)).unwrap();
        let row = &nodes[at..at + usize::try_from(nodes[at].size).unwrap()];
        let chip = row.iter().find(|n| n.role == Role::Link).unwrap();
        assert!(chip.styles.as_slice().contains(&look::HOW_WHO_AGENT));
        assert!(chip.states.as_slice().contains(&skein_ui::State::Current));
        assert_eq!(
            bindings.decode(chip.binding.unwrap().token, skein_ui::Act::Press),
            Some(Action::Pages(Act::Go { address: Address::Team { project: 7, role: Some(52) } }))
        );
        assert_eq!(
            temper_client_domain::address::print(&Address::Team { project: 7, role: Some(52) }).as_ref(),
            b"/projects/7/team?role=52"
        );
        assert!(row.iter().any(|n| n.name.as_deref() == Some(b"- A finding")));
        let head = nodes
            .iter()
            .find(|n| {
                n.role == Role::Link
                    && !n.styles.as_slice().contains(&look::HOW_WHO_AGENT)
                    && n.name.as_deref() == Some(b"Prover")
            })
            .unwrap();
        assert_eq!(
            bindings.decode(head.binding.unwrap().token, skein_ui::Act::Press),
            Some(Action::Pages(Act::Go { address: Address::Task { task: 52 } }))
        );
        let pause = nodes.iter().find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Pause")).unwrap();
        assert_eq!(
            bindings.decode(pause.binding.unwrap().token, skein_ui::Act::Press),
            Some(Action::Pages(Act::Pause))
        );
        assert!(!nodes.iter().any(|n| n.styles.as_slice().contains(&look::TEAM_RIGHT)), "no empty right column");
        let mut unread = facts;
        let card = unread.card.as_mut().unwrap();
        card.now = None;
        card.pause = false;
        card.made = None;
        card.requester = None;
        let mut b = Builder::new(&f::tree_limits());
        draw(
            &mut b,
            &w,
            b"shop",
            &Shown::Read(unread),
            7,
            &mut Bindings::new(f::tree_limits().bindings),
            &mut Times::new(changed),
        );
        let tree = b.finish();
        assert!(
            !tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::TEAM_NOW)
                || n.styles.as_slice().contains(&look::TEAM_COLUMNS)
                || n.styles.as_slice().contains(&look::TEAM_ACTS)),
            "unread parts have no empty frame"
        );
    }
}

#[test]
fn wake_rows_say_periods_task_states_topics_and_classes() {
    use crate::{pages::team::wakes::words, words::Times};
    use jig_client::contract::WakeRule;
    use skein_lib::{Duration, Wall};
    use temper_client_domain::pages::team::wakes::{Interest, Row};
    // Sunday 23:00 UTC is Monday 06:00 in the view's +07:00 zone.
    let at = Wall::from_nanos(342_000_000_000_000);
    let timer = |period: Option<u64>| Interest::Timer { at, period: period.map(Duration::from_secs), wakes: true };
    for mut w in [f::browser_words(), f::words()] {
        w.set_zone_minutes(420);
        for (interest, class, left, right) in [
            (timer(Some(604_800)), WakeRule::Never, "A timer", Some("Mondays 06:00")),
            (timer(Some(86_400)), WakeRule::Never, "A timer", Some("daily 06:00")),
            (timer(Some(14_400)), WakeRule::Never, "A timer", Some("every 4 hours")),
            (timer(Some(5400)), WakeRule::Never, "A timer", Some("every 90 minutes")),
            (timer(Some(3600)), WakeRule::Never, "A timer", Some("every hour")),
            (timer(Some(45)), WakeRule::Never, "A timer", Some("every 45 seconds")),
            (timer(Some(1_209_600)), WakeRule::Never, "A timer", Some("every 2 weeks")),
            (timer(Some(172_800)), WakeRule::Never, "A timer", Some("every 2 days")),
            (timer(None), WakeRule::Never, "A timer", Some("once, at 06:00")),
            (
                Interest::Timer { at, period: Some(Duration::from_secs(3600)), wakes: false },
                WakeRule::Immediate,
                "A timer, with its next wake",
                None,
            ),
            (
                Interest::Task { target: 61, held: false, result: true },
                WakeRule::Immediate,
                "Task 61's result",
                Some("at once"),
            ),
            (
                Interest::Task { target: 61, held: true, result: false },
                WakeRule::Never,
                "Task 61 held",
                Some("with its next wake"),
            ),
            (
                Interest::Task { target: 61, held: true, result: true },
                WakeRule::Batch { count: 5, age: Duration::from_secs(7200) },
                "Task 61 held, or its result",
                Some("5 at a time, or after 2 hours"),
            ),
            (
                Interest::Topics { connector: temper_client_protocol_wire::FORGE, count: 2 },
                WakeRule::Paced { span: Duration::from_secs(1800) },
                "News from the forge, on 2 topics",
                Some("every 30 minutes"),
            ),
            (
                Interest::Topics { connector: temper_client_protocol_wire::FORGE, count: 1 },
                WakeRule::Immediate,
                "News from the forge",
                Some("at once"),
            ),
            (
                Interest::Topics { connector: 4, count: 2 },
                WakeRule::Paced { span: Duration::from_secs(86_400) },
                "News from connector 4",
                Some("every day"),
            ),
        ] {
            let (said, classified) = words(&w, &Row { interest, class }, &mut Times::new(at));
            assert_eq!(said.as_ref(), left.as_bytes());
            assert_eq!(classified.as_deref(), right.map(str::as_bytes), "{left}");
        }
        let (_, once) = words(&w, &Row { interest: timer(None), class: WakeRule::Never }, &mut Times::new(Wall::EPOCH));
        assert_eq!(
            once.as_deref(),
            Some(b"once, at 5 January 06:00".as_slice()),
            "single firing includes a date outside today"
        );
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "both shells pin rules links, ordered wake chips and absent columns")]
fn team_draws_rules_and_wakes_in_their_columns() {
    use crate::{frame::Bindings, words::Times};
    use jig_client::contract as c;
    use skein_lib::{Duration, Wall};
    use temper_client_domain::{
        Action, Address,
        pages::{Act, team},
    };
    let at = Wall::from_nanos(367_200_000_000_000);
    let subscriptions = c::Subscriptions {
        entries: List::from_boxed(Box::new([
            c::Subscription {
                number: 1,
                kind: c::SubscriptionKind::Topic { connector: temper_client_protocol_wire::FORGE, topic: 1 },
            },
            c::Subscription {
                number: 2,
                kind: c::SubscriptionKind::Timer { at, period: Some(Duration::from_secs(604_800)) },
            },
            c::Subscription {
                number: 3,
                kind: c::SubscriptionKind::Topic { connector: temper_client_protocol_wire::FORGE, topic: 2 },
            },
            c::Subscription { number: 4, kind: c::SubscriptionKind::Task { target: 61, held: false, result: true } },
        ])),
    };
    let policy = c::WakePolicy {
        words: c::WakeRule::Never,
        results: c::ResultsWake::Never,
        questions: false,
        answers: false,
        timers: true,
        notices: c::WakeRule::Immediate,
        news: c::WakeRule::Paced { span: Duration::from_secs(1800) },
    };
    for w in [f::browser_words(), f::words()] {
        let mut facts = Facts {
            people: List::with_capacity(0),
            cut: false,
            triager: None,
            roles_cut: false,
            selected: Some(52),
            follows: List::with_capacity(0),
            roles: List::from_boxed(Box::new([team::Role {
                task: 52,
                name: b"Prover".as_slice().into(),
                now: None,
                spent: 0,
                budget: 40,
            }])),
            card: Some(Box::new(team::Card {
                task: 52,
                name: b"Prover".as_slice().into(),
                made: None,
                requester: None,
                hold: None,
                paused: false,
                pause: false,
                resume: false,
                now: Some(team::Now { body: b"A finding.".as_slice().into(), changed: None, cut: false }),
                rules: Some(team::Rules { charter: 13, asked: b"Reproduce before routing.".as_slice().into() }),
                wakes: team::wakes::rows(&subscriptions, &policy),
                wakes_cut: true,
                sessions: List::from_boxed(Box::new([team::Session { start: at, end: None, words: None, spent: 0 }])),
            })),
        };
        let mut bindings = Bindings::new(f::tree_limits().bindings);
        let mut b = Builder::new(&f::tree_limits());
        draw(&mut b, &w, b"shop", &Shown::Read(facts.clone()), 7, &mut bindings, &mut Times::new(at));
        let tree = b.finish();
        let nodes = tree.nodes();
        let subtree = |style| {
            let at = nodes.iter().position(|n| n.styles.as_slice().contains(&style)).unwrap();
            &nodes[at..at + usize::try_from(nodes[at].size).unwrap()]
        };
        let left = subtree(look::TEAM_LEFT);
        let right = subtree(look::TEAM_RIGHT);
        for said in ["Its rules · change by proposal", "Reproduce before routing."] {
            assert!(left.iter().any(|n| n.name.as_deref() == Some(said.as_bytes())), "{said} in left column");
        }
        let now = left.iter().position(|n| n.styles.as_slice().contains(&look::TEAM_NOW)).unwrap();
        let rules = left.iter().position(|n| n.styles.as_slice().contains(&look::TEAM_RULES)).unwrap();
        assert!(now < rules, "rules follow Now");
        let proposal =
            left.iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"change by proposal")).unwrap();
        let address = Address::Role { project: 7, charter: 13 };
        assert_eq!(
            bindings.decode(proposal.binding.unwrap().token, skein_ui::Act::Press),
            Some(Action::Pages(Act::Go { address: address.clone() }))
        );
        assert_eq!(temper_client_domain::address::print(&address).as_ref(), b"/projects/7/settings/roles/13");
        for said in
            ["Wakes on", "News from the forge, on 2 topics", "A timer", "Task 61's result", "Not all are shown."]
        {
            assert!(right.iter().any(|n| n.name.as_deref() == Some(said.as_bytes())), "{said} in right column");
        }
        for (row, (words, class)) in right.iter().filter(|n| n.styles.as_slice().contains(&look::TEAM_WAKE_ROW)).zip([
            ("News from the forge, on 2 topics", "every 30 minutes"),
            ("A timer", "Mondays 06:00"),
            ("Task 61's result", "at once"),
        ]) {
            let at = nodes.iter().position(|n| core::ptr::eq(n, row)).unwrap();
            let row = &nodes[at..at + usize::try_from(row.size).unwrap()];
            assert!(row.iter().any(|n| n.name.as_deref() == Some(words.as_bytes())));
            let chip = row.iter().position(|n| n.styles.as_slice().contains(&look::TEAM_WAKE_CLASS)).unwrap();
            let chip = &row[chip..chip + usize::try_from(row[chip].size).unwrap()];
            assert!(
                chip.iter()
                    .any(|n| n.text.iter().any(|run| run.run == Run::Code && run.text.as_ref() == class.as_bytes())),
                "{class} is in code's chip"
            );
        }
        assert_eq!(right.iter().filter(|n| n.styles.as_slice().contains(&look::TEAM_WAKE_ROW)).count(), 3);
        let wakes = right.iter().position(|n| n.styles.as_slice().contains(&look::TEAM_WAKES)).unwrap();
        let sessions = right.iter().position(|n| n.styles.as_slice().contains(&look::TEAM_SESSIONS)).unwrap();
        assert!(wakes < sessions, "sessions follow what wakes the role");
        let task_link = right.iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"61")).unwrap();
        assert_eq!(
            bindings.decode(task_link.binding.unwrap().token, skein_ui::Act::Press),
            Some(Action::Pages(Act::Go { address: Address::Task { task: 61 } }))
        );
        // Rules alone never reserve an empty right column; wakes alone need no left one.
        for rules in [true, false] {
            let card = facts.card.as_mut().unwrap();
            card.now = None;
            card.sessions.clear();
            if rules {
                card.wakes = List::with_capacity(0);
            } else {
                card.rules = None;
                card.wakes = team::wakes::rows(&subscriptions, &policy);
            }
            let mut b = Builder::new(&f::tree_limits());
            draw(
                &mut b,
                &w,
                b"shop",
                &Shown::Read(facts.clone()),
                7,
                &mut Bindings::new(f::tree_limits().bindings),
                &mut Times::new(at),
            );
            let tree = b.finish();
            assert_eq!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::TEAM_LEFT)), rules);
            assert_eq!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::TEAM_RIGHT)), !rules);
        }
    }
}

#[test]
fn team_draws_signoffs_after_waits_or_alone_and_never_at_zero() {
    for words in [f::browser_words(), f::words()] {
        let mut reader = person(true, b"Reader", HeldRole::Owner, Some(Waits::default()));
        reader.signed = Some(temper_client_domain::pages::team::Signed { count: 4, floor: false });
        let mut sam = person(false, b"Sam", HeldRole::Member, None);
        sam.signed = Some(temper_client_domain::pages::team::Signed { count: 2, floor: true });
        let mut kit = person(false, b"Kit", HeldRole::Member, None);
        kit.party = 9;
        kit.signed = Some(temper_client_domain::pages::team::Signed { count: 0, floor: true });
        let shown = Shown::Read(Facts {
            people: List::from_boxed(Box::new([reader, sam, kit])),
            cut: false,
            triager: None,
            follows: List::with_capacity(0),
            roles: List::with_capacity(0),
            roles_cut: false,
            selected: None,
            card: None,
        });
        let mut b = Builder::new(&f::tree_limits());
        draw(
            &mut b,
            &words,
            b"shop",
            &shown,
            7,
            &mut crate::frame::Bindings::new(f::tree_limits().bindings),
            &mut crate::words::Times::new(skein_lib::Wall::EPOCH),
        );
        let tree = b.finish();
        let lines: Vec<_> = tree
            .nodes()
            .iter()
            .filter(|node| node.styles.as_slice().contains(&look::TEAM_LINE))
            .map(|node| node.name.as_deref().unwrap())
            .collect();
        assert_eq!(lines, [b"Nothing waits on you.".as_slice(), b"Signed four last week.", b"Signed 2+ last week."]);
    }
}
