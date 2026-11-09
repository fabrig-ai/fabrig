//! Team's drawn facts (temper/docs/design/client/README.md, 3.5, Team).
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
            standing_roles: None,
        }),
        lede: Some(lede),
        unreadable: false,
        boxes,
    }
}
fn pair(label: &'static str, waits: Option<Waits>, line: Option<&'static str>) -> Row {
    read(
        label,
        vec![
            person(true, b"Alex", HeldRole::Owner, Some(Waits { questions: 2, reviews: 1, ..Waits::default() })),
            person(false, b"Sam", HeldRole::Member, waits),
        ],
        false,
        "Two of you.",
        vec![("You", "Owner of shop", Some("3 things wait on you.")), ("Sam", "Member", line)],
    )
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table covers Team's states, counts, roles and five wait kinds")]
fn team_draws_what_it_shows() {
    let mut rows = vec![
        Row { label: "reading", shown: Shown::Reading, lede: None, unreadable: false, boxes: vec![] },
        Row { label: "unreadable", shown: Shown::Unreadable, lede: None, unreadable: true, boxes: vec![] },
        pair("two people", Some(Waits { questions: 1, ..Waits::default() }), Some("1 question waits on Sam.")),
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
            vec![("You", "Owner of shop", Some("Nothing waits on you."))],
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
    // Each row pins the grammar of one kind at one and at more than one.
    for (waits, said) in [
        (Waits { questions: 2, ..Waits::default() }, "2 questions wait on Sam."),
        (Waits { reviews: 1, ..Waits::default() }, "1 review waits on Sam."),
        (Waits { reviews: 2, ..Waits::default() }, "2 reviews wait on Sam."),
        (Waits { proposals: 1, ..Waits::default() }, "1 proposal waits on Sam."),
        (Waits { proposals: 2, ..Waits::default() }, "2 proposals wait on Sam."),
        (Waits { held: 1, ..Waits::default() }, "1 held task waits on Sam."),
        (Waits { held: 2, ..Waits::default() }, "2 held tasks wait on Sam."),
        (Waits { person_tasks: 1, ..Waits::default() }, "1 person task waits on Sam."),
        (Waits { person_tasks: 2, ..Waits::default() }, "2 person tasks wait on Sam."),
        (Waits { reviews: 1, held: 1, cut: true, ..Waits::default() }, "2+ things wait on Sam."),
        (Waits { questions: u32::MAX, reviews: 1, ..Waits::default() }, "4294967295 things wait on Sam."),
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
        let mut boxes = vec![(if you { "You" } else { "Alex" }, "Owner of shop", None)];
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
            draw(&mut b, &w, b"shop", &row.shown);
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
