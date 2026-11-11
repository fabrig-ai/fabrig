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
    Person { party: if you { READER } else { SAM }, you, name: name.into(), role, waits }
}
fn read(label: &'static str, people: Vec<Person>, cut: bool, lede: &'static str, boxes: Vec<BoxWords>) -> Row {
    Row {
        label,
        shown: Shown::Read(Facts {
            people: List::from_boxed(people.into_boxed_slice()),
            cut,
            triager: None,
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
