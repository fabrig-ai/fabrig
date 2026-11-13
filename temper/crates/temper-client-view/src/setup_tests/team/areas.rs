//! Areas' words, who-acts chips and links (client/pages/team.md, 1).
pub(super) use super::f::team::{MADE, facts, holder_task, owner_task};
use super::*;
use crate::{bindings::Bindings, words::Times};
use skein_ui::{Node, State};
use temper_client_domain::{
    Action, Address,
    pages::{Act, team},
};
use temper_client_protocol_wire::v1 as w;

fn holder_party(number: u64) -> w::Holder {
    w::Holder::Party(w::HolderParty::new(&w::CEILINGS, w::HolderPartyParts { number }).unwrap())
}
fn holder_role(number: u32) -> w::Holder {
    w::Holder::Role(w::HolderRole::new(&w::CEILINGS, w::HolderRoleParts { number }).unwrap())
}

fn subtree(nodes: &[Node], style: skein_ui::Style) -> &[Node] {
    let at = nodes.iter().position(|n| n.styles.as_slice().contains(&style)).unwrap();
    &nodes[at..at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap()]
}
fn named(nodes: &[Node], role: Role, name: &str) -> bool {
    nodes.iter().any(|n| n.role == role && n.name.as_deref() == Some(name.as_bytes()))
}
#[test]
fn team_draws_areas_owners_and_card_links() {
    for words in [f::browser_words(), f::words()] {
        let mut facts = facts();
        let mut bindings = Bindings::new(f::tree_limits().bindings);
        let mut b = Builder::new(&f::tree_limits());
        draw(&mut b, &words, b"vsr", &Shown::Read(facts.clone()), 3, &mut bindings, &mut Times::new(MADE));
        let tree = b.finish();
        let nodes = tree.nodes();
        for (role, name) in [
            (Role::Heading { level: 2 }, "Areas"),
            (
                Role::Paragraph,
                "Two of you. Each area has one owner. Prover hands each finding and red to its area's owner. An owner that can't decide asks you, as How work runs says.",
            ),
            (Role::Paragraph, "A standing role · owns tests, and Anything else, since 2 November · started by you"),
            (Role::Link, "Talk to it"),
        ] {
            assert!(named(nodes, role, name), "{name}");
        }
        assert!(!named(nodes, Role::Heading { level: 2 }, "Standing roles"));
        let table = subtree(nodes, look::HOW_TABLE);
        let rows: Vec<_> = table
            .iter()
            .enumerate()
            .filter(|(_, n)| n.role == Role::Row)
            .map(|(i, n)| &table[i..i + usize::try_from(n.size).unwrap()])
            .collect();
        assert_eq!(rows.len(), 3);
        for (row, area, spend) in [(rows[1], "tests", "21 of 40"), (rows[2], "Anything else", "in Prover's")] {
            assert!(named(row, Role::Cell, area));
            assert!(
                row.iter().any(|n| n.text.iter().any(|r| r.run == Run::Strong && r.text.as_ref() == area.as_bytes()))
            );
            assert!(named(row, Role::Cell, "Reproduced the finding"));
            assert!(row.iter().any(|n| n.name.as_deref() == Some(spend.as_bytes())));
            let chip = row.iter().find(|n| n.role == Role::Link).unwrap();
            assert_eq!(
                bindings.decode(chip.binding.unwrap().token, skein_ui::Act::Press),
                Some(Action::Pages(Act::Go { address: Address::Team { project: 3, role: Some(52) } }))
            );
            assert_eq!(chip.states.as_slice().contains(&State::Current), area == "tests");
            assert_eq!(row[0].states.as_slice().contains(&State::Current), area == "tests");
        }
        assert!(
            rows[2]
                .iter()
                .any(|n| n.styles.as_slice().contains(&look::HOW_EMPTY) && n.name.as_deref() == Some(b"in Prover's"))
        );
        assert!(named(rows[1], Role::Cell, "Tests, fuzzing and the tiers"));
        assert!(named(rows[2], Role::Cell, "What fits none of these"));
        for (name, address) in
            [("Talk to it", Address::Task { task: 52 }), ("Change by proposal", Address::Ask { project: 3 })]
        {
            let link =
                nodes.iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(name.as_bytes())).unwrap();
            assert_eq!(
                bindings.decode(link.binding.unwrap().token, skein_ui::Act::Press),
                Some(Action::Pages(Act::Go { address }))
            );
        }
        let routing = subtree(nodes, look::TEAM_ROUTING);
        let said: Vec<_> = routing.iter().flat_map(|n| n.text.iter().map(|r| r.text.as_ref())).collect();
        assert_eq!(said.concat(), b"Prover reads what comes in against these words and its rules, and hands each to its owner. Change by proposal");
        let mut b = Builder::new(&f::tree_limits());
        draw(
            &mut b,
            &words,
            b"vsr",
            &Shown::PeopleUnread { facts: Box::new(facts.clone()), failed: false },
            3,
            &mut Bindings::new(f::tree_limits().bindings),
            &mut Times::new(MADE),
        );
        let tree = b.finish();
        assert!(named(
            tree.nodes(),
            Role::Paragraph,
            "Each area has one owner. Prover hands each finding and red to its area's owner. An owner that can't decide asks you, as How work runs says."
        ));
        facts.areas = Some(team::Areas { rows: List::with_capacity(0), triager: None, holder: holder_role(0) });
        let mut b = Builder::new(&f::tree_limits());
        draw(
            &mut b,
            &words,
            b"vsr",
            &Shown::Read(facts),
            3,
            &mut Bindings::new(f::tree_limits().bindings),
            &mut Times::new(MADE),
        );
        let tree = b.finish();
        assert!(named(tree.nodes(), Role::Heading { level: 2 }, "Areas"));
        assert!(named(tree.nodes(), Role::Paragraph, "No areas yet: the project's owners take every finding and red."));
        assert!(named(tree.nodes(), Role::Heading { level: 2 }, "Standing roles"));
        assert!(
            !tree.nodes().iter().any(|n| n.role == Role::Table || n.styles.as_slice().contains(&look::TEAM_ROUTING))
        );
    }
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table pins each owner's name, who-acts mark and routing grammar in both shells"
)]
fn team_names_routing_holders_and_owners_that_do_not_stand() {
    let cases = [
        (
            holder_task(52),
            "Prover",
            "hands",
            "reads what comes in against these words and its rules, and hands each to its owner.",
        ),
        (
            holder_task(61),
            "Task 61",
            "hands",
            "reads what comes in against these words and its rules, and hands each to its owner.",
        ),
        (holder_party(READER), "You", "hand", "read what comes in against these words, and hand each to its owner."),
        (holder_party(SAM), "Sam", "hands", "reads what comes in against these words, and hands each to its owner."),
        (
            holder_party(99),
            "Party 99",
            "hands",
            "reads what comes in against these words, and hands each to its owner.",
        ),
        (
            holder_role(0),
            "The project's owners",
            "hand",
            "read what comes in against these words, and hand each to its owner.",
        ),
        (
            holder_role(1),
            "The project's maintainers",
            "hand",
            "read what comes in against these words, and hand each to its owner.",
        ),
        (
            holder_role(2),
            "The project's members",
            "hand",
            "read what comes in against these words, and hand each to its owner.",
        ),
        (
            holder_role(3),
            "The project's observers",
            "hand",
            "read what comes in against these words, and hand each to its owner.",
        ),
        (
            holder_role(12),
            "Role 12's people",
            "hand",
            "read what comes in against these words, and hand each to its owner.",
        ),
    ];
    for words in [f::browser_words(), f::words()] {
        for (holder, name, verb, routes) in &cases {
            let mut facts = facts();
            facts.areas.as_mut().unwrap().holder = holder.clone();
            facts.areas.as_mut().unwrap().rows.get_mut(0).unwrap().holder = Some(holder.clone());
            let mut b = Builder::new(&f::tree_limits());
            draw(
                &mut b,
                &words,
                b"vsr",
                &Shown::Read(facts),
                3,
                &mut Bindings::new(f::tree_limits().bindings),
                &mut Times::new(MADE),
            );
            let tree = b.finish();
            assert!(named(
                tree.nodes(),
                Role::Paragraph,
                &format!(
                    "Two of you. Each area has one owner. {name} {verb} each finding and red to its area's owner. An owner that can't decide asks you, as How work runs says."
                )
            ));
            let routing = subtree(tree.nodes(), look::TEAM_ROUTING);
            assert_eq!(
                routing.iter().flat_map(|n| n.text.iter().map(|r| r.text.as_ref())).collect::<Vec<_>>().concat(),
                format!("{name} {routes} Change by proposal").as_bytes()
            );
            let table = subtree(tree.nodes(), look::HOW_TABLE);
            let mark = match holder {
                w::Holder::Task(_) => look::HOW_WHO_AGENT,
                w::Holder::Party(_) | w::Holder::Role(_) => look::HOW_WHO_PERSON,
            };
            assert!(
                table.iter().any(|n| n.styles.as_slice().contains(&mark) && n.name.as_deref() == Some(name.as_bytes()))
            );
            if let w::Holder::Party(party) = holder
                && [READER, SAM].contains(&party.number())
            {
                let role = if party.number() == READER { "Owner" } else { "Member" };
                assert!(named(tree.nodes(), Role::Paragraph, &format!("{role} · tests, Anything else")));
                let wait =
                    if party.number() == READER { "Nothing waits on you." } else { "One question waits on Sam." };
                assert!(named(table, Role::Cell, wait));
            }
            if *holder != holder_task(52) {
                assert!(table.iter().any(
                    |n| n.styles.as_slice().contains(&look::HOW_EMPTY) && n.name.as_deref() == Some("–".as_bytes())
                ));
            }
        }
        for (owner, name) in [
            (owner_task(61), "Task 61, ended"),
            (
                w::AreaOwner::Party(w::AreaParty::new(&w::CEILINGS, w::AreaPartyParts { number: 7 }).unwrap()),
                "Party 7, not of vsr",
            ),
            (
                w::AreaOwner::Charter(w::AreaCharter::new(&w::CEILINGS, w::AreaCharterParts { number: 15 }).unwrap()),
                "Charter 15's role, not started",
            ),
        ] {
            let mut facts = facts();
            let row = facts.areas.as_mut().unwrap().rows.get_mut(0).unwrap();
            row.owner = owner;
            row.holder = None;
            let mut b = Builder::new(&f::tree_limits());
            draw(
                &mut b,
                &words,
                b"vsr",
                &Shown::Read(facts),
                3,
                &mut Bindings::new(f::tree_limits().bindings),
                &mut Times::new(MADE),
            );
            let tree = b.finish();
            let table = subtree(tree.nodes(), look::HOW_TABLE);
            assert!(
                table
                    .iter()
                    .any(|n| n.styles.as_slice().contains(&look::HOW_EMPTY)
                        && n.name.as_deref() == Some(name.as_bytes()))
            );
        }
    }
}

use super::f::team::{READER, SAM};
