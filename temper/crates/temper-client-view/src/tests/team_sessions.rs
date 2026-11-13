//! Both shells draw the same session parts (client/pages/team.md, 1).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary tests inspect the semantic subtrees from both shells"
)]
use super::fixtures as f;
use crate::{frame::Bindings, look, pages::team::draw, words::Times};
use skein_lib::Wall;
use skein_ui::{Builder, Role};
use temper_client_domain::pages::team::Shown;

pub(super) use super::fixtures::team_sessions::facts;

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
