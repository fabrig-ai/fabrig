//! Both shells draw the same session parts (client/pages/team.md, 1).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary tests inspect the semantic subtrees from both shells"
)]
use super::fixtures as f;
use crate::{frame::Bindings, look, pages::team::draw, words::Times};
use skein_lib::{List, Wall};
use skein_ui::{Builder, Role};
use temper_client_domain::pages::team::{self, Facts, Shown};

pub(super) fn facts() -> Facts {
    Facts {
        member: true,
        people: List::with_capacity(0),
        cut: false,
        areas: None,
        follows: List::with_capacity(0),
        roles: List::from_boxed(Box::new([team::Role {
            task: 52,
            name: b"Prover".as_slice().into(),
            now: None,
            spent: 17,
            budget: 40,
        }])),
        roles_cut: false,
        selected: Some(52),
        card: Some(Box::new(team::Card {
            task: 52,
            name: b"Prover".as_slice().into(),
            made: None,
            requester: None,
            hold: None,
            paused: false,
            pause: false,
            resume: false,
            talk: false,
            now: None,
            rules: None,
            wakes: List::with_capacity(0),
            wakes_cut: false,
            report: None,
            sessions: List::from_boxed(Box::new([
                team::Session { start: Wall::from_nanos(1_794_102_720_000_000_000), end: None, words: None, spent: 0 },
                team::Session {
                    start: Wall::from_nanos(1_794_068_400_000_000_000),
                    end: Some(Wall::from_nanos(1_794_069_720_000_000_000)),
                    words: Some(b"Triaged three findings; routed two.".as_slice().into()),
                    spent: 17,
                },
            ])),
        })),
    }
}
#[test]
fn team_draws_sessions_and_their_closing_line_only_with_rows() {
    // Sunday 8 November, 02:10 UTC, including a start on the same day.
    let wall = Wall::from_nanos(1_794_103_800_000_000_000);
    for w in [f::browser_words(), f::words()] {
        let mut facts = facts();
        for present in [true, false] {
            if !present {
                facts.card.as_mut().unwrap().sessions.clear();
            }
            let mut b = Builder::new(&f::tree_limits());
            draw(
                &mut b,
                &w,
                b"shop",
                &Shown::Read(facts.clone()),
                1,
                &mut Bindings::new(f::tree_limits().bindings),
                &mut Times::new(wall),
            );
            let tree = b.finish();
            let nodes = tree.nodes();
            assert_eq!(
                nodes.iter().filter(|n| n.styles.as_slice().contains(&look::TEAM_RIGHT)).count(),
                usize::from(present)
            );
            assert!(!nodes.iter().any(|n| n.styles.as_slice().contains(&look::TEAM_LEFT)), "no unread left column");
            let session_rows: Vec<_> =
                nodes.iter().enumerate().filter(|(_, n)| n.styles.as_slice().contains(&look::TEAM_SESSION)).collect();
            assert_eq!(session_rows.len(), if present { 2 } else { 0 });
            for phrase in [
                "Sessions this week",
                "Sun 01:52 · running",
                "Sat 16:20 · 22 min",
                "Triaged three findings; routed two.",
                "Each session starts fresh from its rules, its Now and what woke it, and ends by writing its Now.",
            ] {
                assert_eq!(
                    nodes
                        .iter()
                        .filter(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(phrase.as_bytes()))
                        .count(),
                    usize::from(present),
                    "{phrase}"
                );
            }
            for ((at, row), (start, cost, words)) in session_rows.into_iter().zip([
                ("Sun 01:52 · running", "0", None),
                ("Sat 16:20 · 22 min", "17", Some("Triaged three findings; routed two.")),
            ]) {
                let children = &nodes[at..at + usize::try_from(row.size).unwrap()];
                assert!(children.iter().any(|n| n.name.as_deref() == Some(start.as_bytes())
                    && n.styles.as_slice().contains(&look::TEAM_SESSION_TIME)));
                assert!(children.iter().any(|n| n.name.as_deref() == Some(cost.as_bytes())
                    && n.styles.as_slice().contains(&look::TEAM_SESSION_COST)));
                assert_eq!(
                    children.iter().filter(|n| n.role == Role::Paragraph).count(),
                    if words.is_some() { 3 } else { 2 },
                    "no placeholder for absent Stood words"
                );
            }
        }
    }
}
