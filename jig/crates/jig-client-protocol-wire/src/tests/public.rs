//! Public wire bounds and closure (jig/docs/design/client/domain/contract.md, section 12).
#[path = "public/fixtures.rs"]
mod fixtures;
#[path = "public/schema.rs"]
mod schema;

use crate::{PublicIndexBounds, public_bounds, public_limits, public_v1 as w};
use skein_lib::Reader;

// Public index figures: 20 live and ended goals, 50 lines, depth 16, 64 projects.
const PROPOSED: PublicIndexBounds = PublicIndexBounds { goals: 20, ended: 20, lines: 50, depth: 16, projects: 64 };

#[test]
fn public_bounds_hold_the_largest_answers() {
    for bounds in [
        PublicIndexBounds { goals: 0, ended: 0, lines: 0, depth: 0, projects: 0 },
        PublicIndexBounds { goals: 1, ended: 2, lines: 3, depth: 4, projects: 5 },
        PROPOSED,
    ] {
        let limits = public_limits(&bounds).expect("bounds within ceilings");
        let [projects_bound, board_bound, card_bound] = public_bounds(&bounds);
        let projects = fixtures::projects(&limits, limits.public_projects_projects);
        let board = fixtures::board(&limits, limits.public_board_live, limits.public_board_ended);
        let card = fixtures::card(&limits, limits.public_card_lineage, limits.public_card_delegates);
        let bytes = fixtures::encoded(&projects, projects_bound, w::PublicProjects::encode);
        assert_eq!(w::PublicProjects::decode(&limits, &mut Reader::new(&bytes)), Ok(projects));
        let bytes = fixtures::encoded(&board, board_bound, w::PublicBoard::encode);
        assert_eq!(w::PublicBoard::decode(&limits, &mut Reader::new(&bytes)), Ok(board));
        let bytes = fixtures::encoded(&card, card_bound, w::PublicCard::encode);
        assert_eq!(w::PublicCard::decode(&limits, &mut Reader::new(&bytes)), Ok(card));
    }
}

#[test]
fn one_more_entry_is_refused_by_each_public_list_limit() {
    let limits = public_limits(&PROPOSED).expect("proposed bounds within ceilings");
    let more = public_limits(&PublicIndexBounds {
        goals: PROPOSED.goals.checked_add(1).expect("small goals"),
        ended: PROPOSED.ended.checked_add(1).expect("small ended"),
        lines: PROPOSED.lines.checked_add(1).expect("small lines"),
        depth: PROPOSED.depth.checked_add(1).expect("small depth"),
        projects: PROPOSED.projects.checked_add(1).expect("small projects"),
    })
    .expect("one more within schema ceilings");
    let projects = fixtures::projects(&more, more.public_projects_projects);
    assert_eq!(
        w::PublicProjects::new(&limits, projects.into_parts()).expect_err("extra project"),
        w::Problem { path: w::Path::PublicProjectsProjects, reason: skein_codec::Reason::Bound }
    );
    for (live, ended, path) in [
        (more.public_board_live, limits.public_board_ended, w::Path::PublicBoardLive),
        (limits.public_board_live, more.public_board_ended, w::Path::PublicBoardEnded),
    ] {
        let board_limits = w::Limits { public_goal_tree: limits.public_goal_tree, ..more };
        let board = fixtures::board(&board_limits, live, ended);
        assert_eq!(
            w::PublicBoard::new(&limits, board.into_parts()).expect_err("extra goal"),
            w::Problem { path, reason: skein_codec::Reason::Bound }
        );
    }
    let goal = fixtures::goal(&more);
    assert_eq!(
        w::PublicGoal::new(&limits, goal.into_parts()).expect_err("extra tree line"),
        w::Problem { path: w::Path::PublicGoalTree, reason: skein_codec::Reason::Bound }
    );
    for (lineage, delegates, path) in [
        (more.public_card_lineage, limits.public_card_delegates, w::Path::PublicCardLineage),
        (limits.public_card_lineage, more.public_card_delegates, w::Path::PublicCardDelegates),
    ] {
        let card = fixtures::card(&more, lineage, delegates);
        assert_eq!(
            w::PublicCard::new(&limits, card.into_parts()).expect_err("extra card entry"),
            w::Problem { path, reason: skein_codec::Reason::Bound }
        );
    }
}

#[test]
fn bounds_past_a_ceiling_are_refused() {
    // contract.md, section 12: goals, ended and lines 256; depth 64; projects 4096.
    for bounds in [
        PublicIndexBounds { goals: 257, ..PROPOSED },
        PublicIndexBounds { ended: 257, ..PROPOSED },
        PublicIndexBounds { lines: 257, ..PROPOSED },
        PublicIndexBounds { depth: 65, ..PROPOSED },
        PublicIndexBounds { projects: 4097, ..PROPOSED },
    ] {
        assert_eq!(public_limits(&bounds), None, "past ceiling: {bounds:?}");
        assert_eq!(public_bounds(&bounds), [u32::MAX; 3], "invalid bounds refuse every reservation");
    }
}

#[test]
fn every_schema_ceiling_is_accepted() {
    let bounds = PublicIndexBounds { goals: 256, ended: 256, lines: 256, depth: 64, projects: 4096 };
    assert_eq!(public_limits(&bounds), Some(w::CEILINGS));
}

#[test]
fn proposed_index_figures_have_explicit_answer_reservations() {
    // Each maximally populated line is 358 bytes. Names are 64 bytes plus their length.
    // Projects: 8 + 64 * 76; board: 88 + 40 * (366 + 50 * 358);
    // card: 370 + 16 * 8 + 50 * 358.
    assert_eq!(public_bounds(&PROPOSED), [4_872, 730_728, 18_398]);
}
