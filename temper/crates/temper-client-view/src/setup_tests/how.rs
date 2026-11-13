//! How work runs' parts in both shells (temper/docs/design/client/pages/settings.md, 2).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary tests inspect semantic drawings and terminal lines"
)]
use super::fixtures as f;
use crate::look;
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Node, Role, Style, Tree};
use temper_client_domain as d;

pub(crate) fn subtree<'a>(tree: &'a Tree, node: &Node) -> &'a [Node] {
    let at = tree.nodes().iter().position(|n| core::ptr::eq(n, node)).unwrap();
    &tree.nodes()[at..at.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}
pub(crate) fn styled(tree: &Tree, style: Style) -> &[Node] {
    subtree(tree, tree.nodes().iter().find(|n| n.styles.as_slice().contains(&style)).unwrap())
}
pub(crate) fn children(nodes: &[Node]) -> Vec<&Node> {
    let mut at = 1;
    let mut result = Vec::new();
    while at < nodes.len() {
        let node = &nodes[at];
        result.push(node);
        at = at.checked_add(usize::try_from(node.size).unwrap()).unwrap();
    }
    result
}
pub(crate) fn named<'a>(nodes: &'a [Node], role: Role, name: &[u8]) -> &'a Node {
    nodes
        .iter()
        .find(|n| n.role == role && n.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing {role:?}: {}", String::from_utf8_lossy(name)))
}

#[test]
fn budgets_draw_the_period_in_reader_time_and_roles_before_goals() {
    use d::pages::how::{self, Facts, Period, Read, Standing};
    // Tuesday 1970-01-06 10:00 at UTC+12, still Monday in UTC.
    let next = skein_lib::Wall::from_nanos(424_800_000_000_000);
    for (period, expected) in [
        (Some(Period { week: true, next: Some(next) }), Some("a week · renews Tuesday")),
        (Some(Period { week: false, next: Some(next) }), Some("each period · renews Tuesday")),
        (Some(Period { week: false, next: None }), Some("each period")),
        (None, None),
    ] {
        for goals in [false, true] {
            let mut words = f::words();
            words.zone_minutes = 720_i32;
            let facts = Facts {
                rules: List::with_capacity(0),
                period,
                standing: Box::new([
                    Standing { number: 2, title: b"Owns tests".as_slice().into(), left: 28, budget: 40 },
                    Standing { number: 9, title: b"Owns docs".as_slice().into(), left: 5, budget: 12 },
                ]),
                asking: false,
                read: Read::Reading,
                roles: false,
                budget: Some(c::TemplateBudget::Amount(80)),
                goals,
                classes: false,
                several: false,
            };
            let mut builder = skein_ui::Builder::new(&f::tree_limits());
            crate::pages::how::draw_budgets(&mut builder, &words, &how::parts(&facts));
            let tree = builder.finish();
            let nodes = tree.nodes();
            named(nodes, Role::Heading { level: 2 }, b"Budgets");
            if let Some(expected) = expected {
                named(nodes, Role::Text, expected.as_bytes());
            } else {
                assert_eq!(children(styled(&tree, look::SETTINGS_SECTION_HEAD)).len(), 1);
            }
            let rows: Vec<_> = nodes.iter().filter(|n| n.styles.as_slice().contains(&look::HOW_ROW)).collect();
            assert_eq!(rows.len(), if goals { 3 } else { 2 });
            for (row, title, small, figure) in [
                (rows[0], b"Owns tests".as_slice(), "a standing role · 28 left", b"40".as_slice()),
                (rows[1], b"Owns docs".as_slice(), "a standing role · 5 left", b"12".as_slice()),
            ] {
                let row = subtree(&tree, row);
                named(row, Role::Text, title);
                named(row, Role::Text, small.as_bytes());
                named(row, Role::Text, figure);
            }
            if goals {
                let row = subtree(&tree, rows[2]);
                named(row, Role::Text, b"Goals");
                named(row, Role::Text, b"80");
            }
            let empty = Facts { standing: Box::default(), goals: false, ..facts };
            let mut builder = skein_ui::Builder::new(&f::tree_limits());
            crate::pages::how::draw_budgets(&mut builder, &words, &how::parts(&empty));
            assert_eq!((builder.finish().nodes()).len(), 0);
        }
    }
}
