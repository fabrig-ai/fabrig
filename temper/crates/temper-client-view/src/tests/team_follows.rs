//! Follows occupy each owner's row and the card's left column (client/pages/team.md, 1).
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect semantic subtrees")]
use super::fixtures as f;
use crate::{frame::Bindings, look, pages::team::draw, words::Times};
use skein_lib::{List, Wall};
use skein_ui::{Builder, Node, Role, Run};
use temper_client_domain::pages::team::{
    self,
    follows::{Entry, Follow, Owner},
};

fn follow(owner: Owner, source: &[u8], status: &[u8]) -> Follow {
    Follow {
        owner,
        source: source.into(),
        status: status.into(),
        line: skein_lib::bytes::concat(&[b"Follows ", source, b" \xc2\xb7 ", status]),
        entries: List::uncapped(),
        more: 0,
    }
}
fn subtree(nodes: &[Node], style: skein_ui::Style) -> &[Node] {
    let at = nodes.iter().position(|n| n.styles.as_slice().contains(&style)).unwrap();
    &nodes[at..at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap()]
}
fn position(nodes: &[Node], name: &[u8]) -> usize {
    nodes.iter().position(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(name)).unwrap()
}

#[test]
fn follows_draw_under_now_after_signed_and_before_rules_in_both_shells() {
    let mut facts = super::team_sessions::facts();
    facts.roles.get_mut(0).unwrap().now = Some(b"The row's Now".as_slice().into());
    let card = facts.card.as_mut().unwrap();
    card.now = Some(team::Now { body: b"The card's Now".as_slice().into(), changed: None, cut: false });
    card.rules = Some(team::Rules { charter: 13, asked: b"The role's rules".as_slice().into() });
    facts.people = List::from_boxed(Box::new([team::Person {
        party: 1,
        you: true,
        name: b"Reader".as_slice().into(),
        role: jig_client::contract::HeldRole::Owner,
        waits: None,
        signed: Some(team::Signed { count: 1, floor: false }),
    }]));
    let mut role = follow(Owner::Task(52), b"acme/upstream", b"2 behind \xc2\xb7 check passes at 9c04d7e1a2b3");
    role.entries = List::from_boxed(Box::new([
        Entry { after: *b"9c04d7e1a2b3", state: b"held by its owner: waits for the handoff fix".as_slice().into() },
        Entry { after: *b"abababababab", state: b"waits for change 812".as_slice().into() },
    ]));
    let person = follow(Owner::Person(1), b"acme/tools", b"caught up \xc2\xb7 no check yet");
    facts.follows =
        List::from_boxed(Box::new([role.clone(), person.clone(), follow(Owner::Task(99), b"unlisted", b"stopped")]));
    for words in [f::browser_words(), f::words()] {
        let mut b = Builder::new(&f::tree_limits());
        draw(
            &mut b,
            &words,
            b"shop",
            &team::Shown::Read(facts.clone()),
            1,
            &mut Bindings::new(f::tree_limits().bindings),
            &mut Times::new(Wall::EPOCH),
        );
        let tree = b.finish();
        let nodes = tree.nodes();
        let row = subtree(nodes, look::TEAM_ROW);
        assert!(position(row, b"The row's Now") < position(row, &role.line));
        let line = &row[position(row, &role.line)];
        assert!(line.styles.as_slice().contains(&look::TEAM_ROLE), "small muted follow line");
        let person_box = subtree(nodes, look::TEAM_PERSON);
        assert!(position(person_box, b"Signed one last week.") < position(person_box, &person.line));
        let left = subtree(nodes, look::TEAM_LEFT);
        let now = position(left, b"The card's Now");
        let kicker = position(left, b"Follows");
        let line = position(left, &role.line);
        let held = position(left, b"9c04d7e1a2b3 \xc2\xb7 held by its owner: waits for the handoff fix");
        let waits = position(left, b"abababababab \xc2\xb7 waits for change 812");
        let rules = position(left, b"The role's rules");
        assert!(now < kicker && kicker < line && line < held && held < waits && waits < rules);
        let entry = &left[held..held + usize::try_from(left[held].size).unwrap()];
        assert!(
            entry.iter().any(|n| n.text.iter().any(|r| r.run == Run::Code && r.text.as_ref() == b"9c04d7e1a2b3")),
            "AFTER12 has code's mark"
        );
        assert_eq!(
            nodes.iter().filter(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(role.line.as_ref())).count(),
            2,
            "one row line, one card line"
        );
        assert_eq!(
            nodes
                .iter()
                .filter(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(person.line.as_ref()))
                .count(),
            1
        );
        assert!(!nodes.iter().any(|n| n.name.as_deref() == Some(b"Follows unlisted \xc2\xb7 stopped")));
    }
}

#[test]
fn follows_card_draws_eight_entries_then_more_and_no_empty_part() {
    for words in [f::browser_words(), f::words()] {
        for count in [0, 8, 10] {
            let mut facts = super::team_sessions::facts();
            facts.card = None;
            let lag = temper_client_domain::forge::follows::lag(u32::try_from(count).unwrap());
            let status = skein_lib::bytes::concat(&[&lag, b" \xc2\xb7 no check yet"]);
            let mut role = follow(Owner::Task(52), b"acme/upstream", &status);
            for _ in 0..count {
                role.entries.push(Entry { after: *b"9c04d7e1a2b3", state: b"not made yet".as_slice().into() }).unwrap();
            }
            facts.follows = List::from_boxed(Box::new([role]));
            for present in [true, false] {
                if !present {
                    facts.follows.clear();
                }
                let mut b = Builder::new(&f::tree_limits());
                draw(
                    &mut b,
                    &words,
                    b"shop",
                    &team::Shown::Read(facts.clone()),
                    1,
                    &mut Bindings::new(f::tree_limits().bindings),
                    &mut Times::new(Wall::EPOCH),
                );
                let tree = b.finish();
                let nodes = tree.nodes();
                assert_eq!(
                    nodes.iter().filter(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(b"Follows")).count(),
                    usize::from(present)
                );
                assert_eq!(
                    nodes
                        .iter()
                        .filter(|n| n.role == Role::Paragraph
                            && n.name.as_deref() == Some(b"9c04d7e1a2b3 \xc2\xb7 not made yet"))
                        .count(),
                    if present { count.min(8) } else { 0 }
                );
                assert_eq!(
                    nodes
                        .iter()
                        .filter(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(b"and 2 more."))
                        .count(),
                    usize::from(present && count == 10)
                );
                assert_eq!(
                    nodes.iter().filter(|n| n.styles.as_slice().contains(&look::TEAM_LEFT)).count(),
                    usize::from(present),
                    "follows need no task document"
                );
            }
        }
    }
}
