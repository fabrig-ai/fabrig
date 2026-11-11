//! Printed view lines and their paint (temper/docs/design/client/tui/pages/README.md).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests build wire values and collect ordered text and style diagnostics"
)]
use super::fixtures as f;
use crate::{look, view::printed};
use skein_lib::List;
use skein_ui::{Builder, Role, Style};
use temper_client_domain::pages::chat::Shown;
use temper_client_protocol_wire::v1 as w;

pub(super) const ADDRESS: &[u8] = b"way You approve";
pub(super) const MIXED: &[u8] = b"Under You approve:\n\n```view\nway You approve\n```\n\nNothing lands without you.";
pub(super) fn card() -> w::Card {
    w::Card::new(
        &w::CEILINGS,
        w::CardParts {
            name: b"The coding agent".as_slice().into(),
            who: b"an agent".as_slice().into(),
            facts: List::from_boxed(Box::new([w::Fact::new(
                &w::CEILINGS,
                w::FactParts { label: b"Can".as_slice().into(), value: b"push its branch".as_slice().into() },
            )
            .unwrap()])),
        },
    )
    .unwrap()
}
pub(super) fn path(stops: &[(w::Actor, w::PathStep, &[u8])]) -> w::BranchPath {
    w::BranchPath::new(
        &w::CEILINGS,
        w::BranchPathParts {
            repository: None,
            branch: b"main".as_slice().into(),
            stops: List::from_boxed(
                stops
                    .iter()
                    .map(|(actor, step, words)| {
                        w::Stop::new(
                            &w::CEILINGS,
                            w::StopParts {
                                actor: actor.clone(),
                                step: step.clone(),
                                words: (*words).into(),
                                needed: false,
                                over: Box::default(),
                                opens: w::Opens::Nothing,
                                card: card(),
                            },
                        )
                        .unwrap()
                    })
                    .collect(),
            ),
        },
    )
    .unwrap()
}
pub(super) fn way(current: bool) -> w::View {
    w::View::Way(
        w::WayView::new(
            &w::CEILINGS,
            w::WayViewParts {
                name: b"You approve".as_slice().into(),
                line: b"An agent makes each change; nothing reaches main until you approve it.".as_slice().into(),
                current,
                path: path(&[
                    (w::Actor::Person, w::PathStep::Ask, b"you ask"),
                    (w::Actor::Agent, w::PathStep::Produce, b"produce"),
                    (w::Actor::Code, w::PathStep::Check, b"check"),
                    (w::Actor::Agent, w::PathStep::Repair, b"repair"),
                    (w::Actor::Code, w::PathStep::Check, b"check"),
                    (w::Actor::Person, w::PathStep::Approve, b"you approve"),
                    (w::Actor::Agent, w::PathStep::Resolve, b"resolve"),
                    (w::Actor::Code, w::PathStep::Merge, b"merge"),
                    (w::Actor::Bar, w::PathStep::Branch, b"main"),
                ]),
            },
        )
        .unwrap(),
    )
}
// A span's own styles, plus Description columns above it; parent paint stays inherited.
type Span = (&'static str, Option<Style>, u32);
const WAY: &[Span] = &[
    ("You approve", Some(look::NAME), 0),
    ("An agent makes each change; nothing reaches main until you approve it.", Some(look::HINT), 0),
    ("○", Some(look::SIGNAL), 0),
    (" ", None, 0),
    ("you ask", None, 0),
    ("✦", None, 0),
    (" ", None, 0),
    ("produce", None, 0),
    ("▪", Some(look::HINT), 4),
    (" ", None, 4),
    ("check", Some(look::HINT), 4),
    ("✦", None, 0),
    (" ", None, 0),
    ("repair", None, 0),
    ("▪", Some(look::HINT), 4),
    (" ", None, 4),
    ("check", Some(look::HINT), 4),
    ("○", Some(look::SIGNAL), 0),
    (" ", None, 0),
    ("you approve", None, 0),
    ("✦", None, 0),
    (" ", None, 0),
    ("resolve", None, 0),
    ("▪", Some(look::HINT), 4),
    (" ", None, 4),
    ("merge", Some(look::HINT), 4),
    ("┃", Some(look::FAINT), 4),
    (" ", None, 4),
    ("main", None, 4),
];
const PATH: &[Span] = &[("✦", None, 0), (" ", None, 0), ("derive these words", None, 0)];
const CARD: &[Span] = &[
    ("The coding agent", Some(look::NAME), 0),
    ("an agent", Some(look::HINT), 0),
    ("Can", None, 0),
    ("push its branch", None, 2),
];
#[test]
#[expect(clippy::too_many_lines, reason = "one table checks each printed wire variant and read state")]
fn each_view_prints_its_lines() {
    let short = || path(&[(w::Actor::Agent, w::PathStep::Produce, b"derive these words")]);
    let mut current = WAY.to_vec();
    current.insert(1, (" · now", Some(look::HINT), 0));
    let mut proposal = vec![("Before", Some(look::HEAD), 0)];
    proposal.extend_from_slice(PATH);
    proposal.push(("After", Some(look::HEAD), 0));
    proposal.extend_from_slice(PATH);
    let rows = [
        ("way", Some(Shown::Drawn(Box::new(way(false)))), WAY.to_vec()),
        ("current way", Some(Shown::Drawn(Box::new(way(true)))), current),
        (
            "goal bar",
            Some(Shown::Drawn(Box::new(w::View::Path(path(&[
                (w::Actor::Bar, w::PathStep::Branch, b"goal"),
                (w::Actor::Agent, w::PathStep::Produce, b"derive these words"),
            ]))))),
            [&[("┃", Some(look::FAINT), 4), (" ", None, 4), ("goal", None, 4)][..], PATH].concat(),
        ),
        ("path", Some(Shown::Drawn(Box::new(w::View::Path(short())))), PATH.to_vec()),
        (
            "proposal",
            Some(Shown::Drawn(Box::new(w::View::Proposal(
                w::ProposalView::new(&w::CEILINGS, w::ProposalViewParts { before: short(), after: short() }).unwrap(),
            )))),
            proposal,
        ),
        (
            "charter",
            Some(Shown::Drawn(Box::new(w::View::Charter(
                w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 1, card: card() }).unwrap(),
            )))),
            CARD.to_vec(),
        ),
        (
            "rule",
            Some(Shown::Drawn(Box::new(w::View::Rule(
                w::RuleView::new(&w::CEILINGS, w::RuleViewParts { kind: 2, number: 1, card: card() }).unwrap(),
            )))),
            CARD.to_vec(),
        ),
        (
            "roles",
            Some(Shown::Drawn(Box::new(w::View::Roles(
                w::RolesView::new(
                    &w::CEILINGS,
                    w::RolesViewParts {
                        roles: List::from_boxed(Box::new([w::CharterView::new(
                            &w::CEILINGS,
                            w::CharterViewParts { charter: 1, card: card() },
                        )
                        .unwrap()])),
                    },
                )
                .unwrap(),
            )))),
            CARD.to_vec(),
        ),
        (
            "not drawn",
            Some(Shown::Drawn(Box::new(w::View::NotDrawn(
                w::NotDrawn::new(
                    &w::CEILINGS,
                    w::NotDrawnParts { words: b"This revision isn't drawn yet.".as_slice().into() },
                )
                .unwrap(),
            )))),
            vec![("This revision isn't drawn yet.", None, 0)],
        ),
        ("asking", Some(Shown::Asking), vec![("Drawing way You approve…", Some(look::HINT), 0)]),
        ("not read", None, vec![("Drawing way You approve…", Some(look::HINT), 0)]),
        ("failed", Some(Shown::Failed), vec![("Couldn't draw way You approve.", None, 0)]),
    ];
    let mut failures = Vec::new();
    let domain = f::limits();
    let page = temper_client_view::Limits::of(&domain, f::words().view()).unwrap();
    let limits = crate::limits(&domain, &page.tree, 512).unwrap();
    let words = f::words();
    for (name, shown, want) in rows {
        let mut b = Builder::new(&limits);
        printed::draw(&mut b, &printed::read(ADDRESS, shown.as_ref()), &words);
        let tree = b.finish();
        let spans: Vec<_> = tree
            .nodes()
            .iter()
            .enumerate()
            .flat_map(|(at, node)| {
                let columns = tree
                    .nodes()
                    .iter()
                    .enumerate()
                    .take(at)
                    .filter(|(start, parent)| {
                        parent.role == Role::Description && start + usize::try_from(parent.size).unwrap() > at
                    })
                    .count()
                    * 2;
                let style = tree
                    .nodes()
                    .iter()
                    .enumerate()
                    .take(at + 1)
                    .rev()
                    .filter(|(start, parent)| start + usize::try_from(parent.size).unwrap() > at)
                    .flat_map(|(_, parent)| parent.styles.as_slice())
                    .find(|style| **style != look::SPACED)
                    .copied();
                node.text.iter().map(move |span| {
                    (String::from_utf8_lossy(&span.text).into_owned(), style, u32::try_from(columns).unwrap())
                })
            })
            .collect();
        let want: Vec<_> = want.into_iter().map(|(text, style, columns)| (text.to_owned(), style, columns)).collect();
        if spans != want {
            failures.push(format!("{name}: {spans:?}, expected {want:?}"));
        }
        assert!(tree.nodes().iter().all(|n| n.binding.is_none() && n.drawing.is_none()));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
