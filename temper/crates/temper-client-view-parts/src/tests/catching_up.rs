//! A forge's progress stays visible in its collapsed row (client/forge/deployment.md).
use super::fixtures as f;
use temper_client_protocol_wire::v1 as w;

#[test]
fn a_forges_row_shows_catching_up_counts_and_clears_them() {
    let zero =
        w::CatchingUp { fresh: 0, gathers: 0, ci: 0, goal_files: 0, lost_writers: 0, goal_branches: 0, uncertain: 0 };
    for (state, expected) in [
        (
            Some(w::CatchingUp { fresh: 3, uncertain: 2, ..zero.clone() }),
            Some("Catching up: 3 fresh reads, 2 uncertain writes"),
        ),
        (Some(w::CatchingUp { goal_branches: 1, ..zero.clone() }), Some("Catching up: 1 goal-branch read")),
        (
            Some(w::CatchingUp {
                fresh: 1,
                gathers: 1,
                ci: 1,
                goal_files: 1,
                lost_writers: 1,
                goal_branches: 1,
                uncertain: 1,
            }),
            Some(
                "Catching up: 1 fresh read, 1 landing read, 1 CI read, 1 goal-file read, 1 lost-writer read, 1 goal-branch read, 1 uncertain write",
            ),
        ),
        (
            Some(w::CatchingUp {
                fresh: 2,
                gathers: 2,
                ci: 2,
                goal_files: 2,
                lost_writers: 2,
                goal_branches: 2,
                uncertain: 2,
            }),
            Some(
                "Catching up: 2 fresh reads, 2 landing reads, 2 CI reads, 2 goal-file reads, 2 lost-writer reads, 2 goal-branch reads, 2 uncertain writes",
            ),
        ),
        (Some(zero), Some("Catching up")),
        (None, None),
    ] {
        let mut rows = f::connect_forge::rows(f::connect_forge::connected(), w::Offer::Offered);
        rows.get_mut(1).unwrap().catching_up = state;
        let root = f::forge_root(temper_client_domain::Address::Forges, rows);
        let mut b = skein_ui::Builder::new(&f::tree_limits());
        crate::forge::forges::main(
            &mut b,
            &root,
            &f::words(),
            &mut crate::bindings::Bindings::new(f::tree_limits().bindings),
            &mut crate::forge::bindings::Bindings::new(200, 8, f::FORGE, f::tree_limits().text_per_node),
            &crate::forge_sign_in::Forges::new(),
            None,
            &mut crate::words::Times::new(f::NOW),
        );
        let tree = b.finish();
        let mut line = None;
        for node in tree.nodes() {
            if let Some(description) = &node.description
                && let Some(start) = skein_lib::bytes::find(description, b"Catching up")
            {
                assert!(line.is_none(), "only the catching-up forge has a line");
                line = Some(description.get(start..).unwrap());
            }
        }
        let expected = match expected {
            Some(expected) => Some(expected.as_bytes()),
            None => None,
        };
        assert_eq!(line, expected);
    }
}
