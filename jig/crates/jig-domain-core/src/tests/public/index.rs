//! Public membership is observed through the one project lookup (jig/docs/design/domain/people.md, 13.7 and 13.9).
#[path = "index/fixtures.rs"]
mod fixtures;
#[path = "index/lifecycle.rs"]
mod lifecycle;
#[path = "index/limits.rs"]
mod limits;
#[path = "index/projection.rs"]
mod projection;

use crate::public::index::{Entry, Project};
use crate::{Core, Event, Limits, PublicCounts, PublicIndexLimits, people, tasks};
use fixtures::*;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

#[test]
fn a_goal_beneath_a_conversation_is_a_public_goal_with_no_parent() {
    let (mut core, limits) = fixture();
    make(&mut core, &limits, 1, None, None, true);
    make(&mut core, &limits, 2, Some(1), Some(3), false);
    visibility(&mut core, &limits, true);
    let project = ready(&core);
    assert_eq!(project.live(), &[2]);
    assert_eq!(project.line(2).unwrap().parent, None);
    assert_eq!(project.goal_of(2), Some(2));
    assert_eq!(project.lineage(2), Some([].as_slice()));
    assert_eq!(project.goal_of(1), None);
    assert_eq!(project.tree(2), Some(([].as_slice(), 0)));
    assert_eq!(project.delegates(1), ([].as_slice(), 0));
    assert_eq!(project.counts(1), zero());
    assert_eq!(project.public_tasks().as_ref(), &[2]);
}

#[test]
fn a_conversations_other_delegates_count_nowhere() {
    let (mut core, limits) = fixture();
    make(&mut core, &limits, 1, None, None, true);
    make(&mut core, &limits, 2, Some(1), Some(3), false);
    visibility(&mut core, &limits, true);
    let before = snapshot(ready(&core));
    let folded = (ready(&core).live_folded(), ready(&core).tree(2).unwrap().1, ready(&core).delegates(2).1);
    make(&mut core, &limits, 3, Some(1), None, false);
    assert_eq!(snapshot(ready(&core)), before);
    assert_eq!((ready(&core).live_folded(), ready(&core).tree(2).unwrap().1, ready(&core).delegates(2).1), folded);
    assert_eq!(ready(&core).goal_of(3), None);
    assert_eq!(ready(&core).counts(1), zero());
}

#[test]
fn a_nested_goal_is_a_line_of_the_outer_tree() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(2), false);
    make(&mut core, &limits, 2, Some(1), Some(1), false);
    make(&mut core, &limits, 3, Some(2), None, false);
    let project = ready(&core);
    assert_eq!(project.live(), &[1]);
    assert_eq!(project.goal_of(2), Some(1));
    assert_eq!(project.goal_of(3), Some(1));
    assert_eq!(project.tree(1), Some(([2, 3].as_slice(), 0)));
    assert_eq!(project.lineage(3), Some([2, 1].as_slice()));
    assert_eq!(project.line(2).unwrap().parent, Some(1));
    assert!(project.line(2).unwrap().goal);
    assert_eq!(project.line(2).unwrap().priority, Some(1));
    assert_eq!(project.delegates(1), ([2].as_slice(), 0));
    assert_eq!(project.counts(1), PublicCounts { working: 1, ..zero() });
}
