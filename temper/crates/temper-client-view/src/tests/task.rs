use super::fixtures::{self, task as f};
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Builder, Role};
const TASK: u64 = 9;
// Small's join bound, supplied as the read refusal fact.
const READ_JOINS: u32 = 4;
const CHILD: u64 = 12;
const PRIORITY: u32 = 1;
const CHARTER: u32 = 1;
const PROCEDURE: u32 = 2;
const PARTY: u64 = 73;
const ONE: u32 = 1;
const NO_ROWS: u32 = 0;
#[test]
fn the_task_header_names_the_playground_project_by_its_record() {
    let (mut root, _, watch, _, _) = fixtures::root_at_with_playground(
        temper_client_domain::Address::Task { task: TASK },
        &fixtures::domain_limits(),
        b"Project 1",
        Some((fixtures::PROJECT, true)),
    );
    let mut task = f::document();
    task.chip.task = TASK;
    task.project = fixtures::PROJECT;
    task.chip.executor = c::ChipExecutor::Party { party: PARTY };
    task.priority = None;
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("task watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, &root);
        let mut heading = None;
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&look::TASK_HEADING) {
                heading = Some(index);
                break;
            }
        }
        let heading = heading.expect("task header");
        assert!(skein_lib::bytes::find(&fixtures::node_text(tree, heading), b"Project 1").is_some());
        assert!(skein_lib::bytes::find(&fixtures::node_text(tree, heading), b"Playground").is_none());
    }
}
fn page(executor: c::ChipExecutor, priority: Option<u32>) -> crate::View {
    let root = f::root(executor, priority);
    let mut view = fixtures::view();
    fixtures::build(&mut view, &root);
    view
}
fn assert_header_and_plan(executor: c::ChipExecutor, name: &[u8]) {
    let view = page(executor, None);
    let tree = view.tree();
    assert!(fixtures::find(tree, Role::Heading { level: 2 }, b"Task details").is_some());
    assert!(fixtures::find(tree, Role::Heading { level: 2 }, name).is_some());
    assert_eq!(fixtures::find(tree, Role::Heading { level: 2 }, b"What was asked"), None);
    assert_eq!(fixtures::find(tree, Role::Heading { level: 2 }, b"Latest activity"), None);
    assert_eq!(fixtures::find(tree, Role::Region, b"Result"), None);
    assert_eq!(fixtures::find(tree, Role::Region, b"Budget and authority"), None);
}
#[test]
fn a_procedures_task_shows_its_header_and_plan_only() {
    assert_header_and_plan(c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE }, b"Its tasks");
}
#[test]
fn a_person_tasks_page_shows_its_header_and_plan_only() {
    assert_header_and_plan(c::ChipExecutor::Party { party: PARTY }, b"Plan");
}
const PROCEDURE_TASK: u64 = 34;
const SERVED_GOAL: u64 = 7;
#[test]
fn a_procedures_task_links_the_goal_it_serves() {
    for browser in [false, true] {
        for (goal, shown) in [(Some(SERVED_GOAL), true), (Some(PROCEDURE_TASK), false), (None, false)] {
            let mut task = f::document();
            task.chip.task = PROCEDURE_TASK;
            task.chip.executor = c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE };
            task.goal = goal;
            let root = f::supplied_root(task, false);
            let mut view = fixtures::view_of(if browser { fixtures::browser_words() } else { fixtures::words() });
            fixtures::build(&mut view, &root);
            let tree = view.tree();
            let details = subtree(tree, region_index(tree, b"Task details"));
            assert_eq!(named(details, Role::Paragraph, b"Goal"), shown);
            assert_eq!(named(details, Role::Link, b"Task 7"), shown);
        }
    }
}
#[test]
fn a_procedures_plan_is_named_its_tasks() {
    let view = page(c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE }, None);
    let tree = view.tree();
    for role in [Role::Region, Role::Heading { level: 2 }, Role::Tree] {
        assert!(fixtures::find(tree, role, b"Its tasks").is_some());
    }
    assert!(fixtures::find(tree, Role::Heading { level: 2 }, b"Plan").is_none());
}

#[test]
fn a_plans_chip_links_to_its_tasks_page_by_number() {
    let mut delivery = f::tree();
    let c::Delivery::Tree(owner) = &mut delivery else { panic!("owner tree") };
    owner.rows = List::with_capacity(ONE);
    owner.rows.push(row()).expect("row");
    let mut builder = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
    crate::pages::task::plan(
        &mut builder,
        Some(&delivery),
        &held_agent_root(),
        &fixtures::words(),
        &mut bindings,
        fixtures::domain_limits().client.domain.rows,
        crate::pages::task::Plan { word: crate::words::Word::Plan, ended: None },
    );
    let tree = builder.finish();
    let link = &tree.nodes()[index_of(&tree, Role::Link, b"Task 12")];
    assert_eq!(
        bindings.decode(link.binding.expect("bound").token, skein_ui::Act::Press),
        Some(Action::Pages(Act::Go { address: temper_client_domain::Address::Task { task: CHILD } }))
    );
}
#[test]
fn a_plan_with_no_live_task_says_so_in_its_region() {
    for rows in [0_usize, 1] {
        let mut delivery = f::tree();
        let c::Delivery::Tree(owner) = &mut delivery else { panic!("owner tree") };
        owner.rows = List::with_capacity(ONE);
        if rows == 1 {
            owner.rows.push(row()).expect("row");
        }
        owner.folded = List::with_capacity(0);
        owner.held_below = 0;
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::pages::task::plan(
            &mut builder,
            Some(&delivery),
            &held_agent_root(),
            &fixtures::words(),
            &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
            fixtures::domain_limits().client.domain.rows,
            crate::pages::task::Plan { word: crate::words::Word::Plan, ended: None },
        );
        let tree = builder.finish();
        let region = region_index(&tree, b"Plan");
        let said = index_of_text(&tree, b"No live tasks.");
        if rows == 0 {
            let said = said.expect("the empty plan's line");
            let end = region + usize::try_from(tree.nodes()[region].size).expect("size");
            assert!(region < said && said <= end, "the line is the region's");
            assert_eq!(tree.nodes()[said].role, Role::Paragraph);
        } else {
            assert!(said.is_none());
        }
    }
}
fn index_of_text(tree: &Tree, expected: &[u8]) -> Option<usize> {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::Paragraph && &*fixtures::node_text(tree, index) == expected {
            return Some(index);
        }
    }
    None
}

#[test]
fn a_plans_items_are_nested_under_their_supplied_parent() {
    let mut delivery = f::tree();
    let c::Delivery::Tree(owner) = &mut delivery else { panic!("owner tree") };
    owner.rows = List::with_capacity(2);
    owner.rows.push(row()).expect("row");
    let mut child = row();
    child.chip.task = GRANDCHILD;
    child.parent = CHILD;
    child.depth = 2;
    owner.rows.push(child).expect("child");
    let tree = plan(&delivery);
    let parent_index = index_of(&tree, Role::TreeItem, b"Task 12");
    let child_index = index_of(&tree, Role::TreeItem, b"Task 13");
    assert!(child_index < parent_index + usize::try_from(tree.nodes()[parent_index].size).expect("parent size"));
    assert!(fixtures::find(&tree, Role::TreeItem, b"Task 9").is_none());
}

// README 5.8: "a live root's own delegates stay under it, in the tree's order".
#[test]
fn a_later_grandchild_nests_under_its_parent_not_after_its_parents_sibling() {
    let delivery = plan_rows(&[(12, 9, 1), (13, 9, 1), (14, 12, 2)]);
    let tree = plan(&delivery);
    let parent = index_of(&tree, Role::TreeItem, b"Task 12");
    assert!(named(subtree(&tree, parent), Role::TreeItem, b"Task 14"));
    assert!(!named(subtree(&tree, parent), Role::TreeItem, b"Task 13"));
    assert_plan_roots(&tree, &[b"Task 12", b"Task 13"]);
    assert!(index_of(&tree, Role::TreeItem, b"Task 14") < index_of(&tree, Role::TreeItem, b"Task 13"));
}

// README 5.8: "a live root's own delegates stay under it, in the tree's order".
#[test]
fn a_plans_deeper_delegates_follow_parents_even_when_children_arrive_first() {
    for rows in [
        [(12, 9, 1), (13, 9, 1), (14, 12, 2), (15, 13, 2), (16, 14, 3)],
        [(16, 14, 1), (14, 12, 1), (12, 9, 1), (15, 13, 1), (13, 9, 1)],
    ] {
        let tree = plan(&plan_rows(&rows));
        let first = index_of(&tree, Role::TreeItem, b"Task 12");
        let child = index_of(&tree, Role::TreeItem, b"Task 14");
        let second = index_of(&tree, Role::TreeItem, b"Task 13");
        assert!(named(subtree(&tree, first), Role::TreeItem, b"Task 14"));
        assert!(named(subtree(&tree, child), Role::TreeItem, b"Task 16"));
        assert!(!named(subtree(&tree, first), Role::TreeItem, b"Task 13"));
        assert!(named(subtree(&tree, second), Role::TreeItem, b"Task 15"));
        assert_plan_roots(&tree, &[b"Task 12", b"Task 13"]);
        assert_tree_items(&tree, &[b"Task 12", b"Task 14", b"Task 16", b"Task 13", b"Task 15"]);
    }
}

// README 5.8: "An ended delegate is drawn before the first live root with a higher number";
// "a live root's own delegates stay under it, in the tree's order".
#[test]
fn an_ended_delegate_follows_the_whole_subtree_of_the_live_root_before_it() {
    let c::Delivery::Tree(tree) = plan_rows(&[(12, 9, 1), (14, 9, 1), (15, 12, 2)]) else { panic!("owner tree") };
    let rendered = its_tasks_tree(
        tree.rows.as_slice(),
        c::Phase::Running,
        Some(tasks_page(&[ended_line(13, c::TaskStatus::Done)], false, false)),
    );
    let tree = &rendered;
    assert_plan_roots(tree, &[b"Task 12", b"Task 13", b"Task 14"]);
    assert!(named(subtree(tree, index_of(tree, Role::TreeItem, b"Task 12")), Role::TreeItem, b"Task 15"));
    assert_tree_items(tree, &[b"Task 12", b"Task 15", b"Task 13", b"Task 14"]);
}

// README 5.8: "Its delegates are the plan's roots"; "in the tree's order".
#[test]
fn a_plan_row_with_its_parent_outside_the_window_stays_at_the_plan_level() {
    let tree = plan(&plan_rows(&[(12, 9, 1), (14, 30, 2)]));
    assert_plan_roots(&tree, &[b"Task 12", b"Task 14"]);
}

// README 5.8: "a live root's own delegates stay under it, in the tree's order".
#[test]
fn a_plans_siblings_keep_their_supplied_order_even_out_of_number_order() {
    let tree = plan(&plan_rows(&[(18, 9, 1), (16, 12, 1), (12, 9, 1), (14, 12, 1)]));
    assert_plan_roots(&tree, &[b"Task 18", b"Task 12"]);
    assert_tree_items(&tree, &[b"Task 18", b"Task 12", b"Task 16", b"Task 14"]);
}

fn plan_rows(rows: &[(u64, u64, u32)]) -> c::Delivery {
    let mut delivery = f::tree();
    let c::Delivery::Tree(tree) = &mut delivery else { panic!("owner tree") };
    tree.rows = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (task, parent, depth) in rows {
        let mut row = row();
        row.chip.task = *task;
        row.parent = *parent;
        row.depth = *depth;
        tree.rows.push(row).unwrap();
    }
    delivery
}

fn assert_plan_roots(tree: &Tree, names: &[&[u8]]) {
    let roots = children(tree, styled(tree, look::PLAN));
    assert_eq!(roots.as_slice().len(), names.len());
    for (index, name) in roots.as_slice().iter().zip(names) {
        assert_eq!(tree.nodes()[*index].role, Role::TreeItem);
        assert_eq!(tree.nodes()[*index].name.as_deref(), Some(*name));
    }
}

#[test]
fn the_release_confirmation_says_what_releasing_does() {
    use temper_client_domain::{Action, Event, pages::Act};
    let mut root = held_agent_root();
    fixtures::root_event(
        &mut root,
        Event::Act {
            action: Action::Pages(Act::Decide {
                object: c::ObjectKey::Task(TASK),
                revision: Some(f::HOLD_REVISION),
                act: c::ActKind::Release,
            }),
        },
    );
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Release this task?").is_some());
    let text = fixtures::text(tree);
    assert!(
        skein_lib::bytes::find(&text, b"The next run resumes its conversation; tries count again from zero").is_some()
    );
    assert!(skein_lib::bytes::find(&text, b"Hold revision 7").is_some());
}

#[test]
fn a_moved_hold_is_refused_and_the_card_asks_to_look_again() {
    let root = held_agent_root();
    let mut card = root.pages().cards().expect("cards").get(FIRST_CARD).expect("held").clone();
    card.outcome = Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::Moved { now: NEW_HOLD_REVISION, what: c::Moved::Hold { reason: None } },
        at: None,
    })));
    let limits = fixtures::tree_limits();
    let mut builder = Builder::new(&limits);
    crate::confirm::card(
        &mut builder,
        &root,
        &card,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(limits.bindings),
        fixtures::domain_limits().client.domain.rows,
    );
    let text = fixtures::text(&builder.finish());
    assert!(skein_lib::bytes::find(&text, b"Look again before deciding.").is_some());
}
const FIRST_CARD: u32 = 0;
const NEW_HOLD_REVISION: c::Revision = c::Revision { value: 8 };

#[test]
fn a_held_plans_release_has_its_public_escalation_card_identity() {
    let root = held_agent_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(has_release_card(tree));
}
fn has_release_card(tree: &Tree) -> bool {
    for node in tree.nodes() {
        if node.role == Role::Item
            && node.key == Some(skein_ui::Key { owner: crate::frame::owner(), kind: 2, a: TASK, b: 0 })
        {
            return true;
        }
    }
    false
}
#[test]
fn a_released_tasks_card_keeps_its_decision_result_while_running() {
    use temper_client_domain::{Action, Event, pages::Act};
    let mut root = held_agent_root();
    for act in [
        Act::Decide { object: c::ObjectKey::Task(TASK), revision: Some(f::HOLD_REVISION), act: c::ActKind::Release },
        Act::Confirm,
    ] {
        fixtures::root_event(&mut root, Event::Act { action: Action::Pages(act) });
    }
    let mut chip = f::document().chip;
    chip.task = TASK;
    chip.phase = c::Phase::Running;
    let limits = fixtures::tree_limits();
    let mut builder = Builder::new(&limits);
    crate::pages::task::controls(
        &mut builder,
        &chip,
        &root,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(limits.bindings),
        fixtures::domain_limits().client.domain.rows,
    );
    assert!(has_release_card(&builder.finish()));
}

use super::inbox::{styled, subtree};
use crate::look;
use skein_ui::{Node, Tree};
fn children(tree: &Tree, parent: usize) -> List<usize> {
    let nodes = tree.nodes();
    let end = parent.checked_add(usize::try_from(nodes[parent].size).expect("subtree")).expect("subtree end");
    let mut result = List::with_capacity(nodes[parent].size);
    let mut index = parent.checked_add(1).expect("first child");
    while index < end {
        result.push(index).expect("children bounded by nodes");
        index = index.checked_add(usize::try_from(nodes[index].size).expect("child size")).expect("next child");
    }
    result
}
fn named(nodes: &[Node], role: Role, name: &[u8]) -> bool {
    for node in nodes {
        if node.role == role && node.name.as_deref() == Some(name) {
            return true;
        }
    }
    false
}
fn header(task: &c::Task, me: Option<&c::Me>) -> Tree {
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::pages::task::header(
        &mut builder,
        task,
        me,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        None,
    );
    builder.finish()
}
fn titled() -> c::Task {
    let mut task = f::document();
    task.chip.task = TASK;
    task.chip.title =
        c::DocumentText::complete(c::Text::new(Box::from(*b"Refresh token handling"), 32).expect("title"));
    task.project = fixtures::PROJECT;
    task
}
#[test]
fn the_title_sits_between_the_headers_two_meta_rows_without_a_title_paragraph() {
    let mut task = titled();
    task.priority = None;
    task.requester = c::Requester::Person(PARTY);
    task.budget.deadline = Some(skein_lib::Wall::EPOCH);
    let root = held_agent_root();
    let tree = header(&task, root.pages().me(root.client()));
    let heading = styled(&tree, look::TASK_HEADING);
    let parts = children(&tree, heading);
    assert_eq!(parts.len(), 4);
    let nodes = tree.nodes();
    let parts = parts.as_slice();
    assert_eq!(nodes[parts[0]].role, Role::Heading { level: 2 });
    assert_eq!(nodes[parts[1]].styles.as_slice(), [look::META]);
    assert_eq!(nodes[parts[2]].role, Role::Heading { level: 1 });
    assert_eq!(nodes[parts[2]].name.as_deref(), Some(task.chip.title.bytes()));
    assert_eq!(nodes[parts[3]].styles.as_slice(), [look::META]);
    assert!(!named(nodes, Role::Paragraph, task.chip.title.bytes()));
    let first = subtree(&tree, parts[1]);
    assert!(named(first, Role::Paragraph, b"Shop"));
    let meta = children(&tree, parts[1]);
    let chip = subtree(&tree, meta.as_slice()[0]);
    assert_eq!(chip[0].role, Role::Slot);
    assert!(named(chip, Role::Paragraph, b"Task 9"));
    assert!(named(chip, Role::Paragraph, b"Agent"));
    let second = subtree(&tree, parts[3]);
    assert!(named(second, Role::Paragraph, b"Asked by Pat"));
    assert_eq!(second[1].role, Role::Status);
    assert!(!named(second, Role::Paragraph, b"Due Jan 1"));
    assert_eq!(children(&tree, parts[3]).len(), 2);
}

#[test]
fn the_first_meta_omits_absent_priority_and_uses_the_project_fallback_and_procedure_part() {
    let mut task = titled();
    task.priority = None;
    let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
    details.procedure =
        Some(c::Part { connector: fixtures::FORGE, kind: c::PartKind::Procedure, key: c::PartKey::Number(1) });
    let tree = header(&task, None);
    let first = subtree(&tree, styled(&tree, look::META));
    assert!(!named(first, Role::Paragraph, "Goal · priority 1".as_bytes()));
    assert!(named(
        first,
        Role::Paragraph,
        &fixtures::words()
            .say(crate::words::Word::Project, &[skein_ui::phrases::Hole::Number(u64::from(task.project))])
    ));
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Shown on the forge.").is_some());
    let root = held_agent_root();
    let mut me = root.pages().me(root.client()).expect("me").clone();
    for index in 0..me.projects.len() {
        let row = me.projects.get_mut(index).expect("project");
        row.name = jig_client::contract::DocumentText::complete(
            jig_client::contract::Text::new(
                fixtures::words()
                    .say(crate::words::Word::Project, &[skein_ui::phrases::Hole::Number(u64::from(row.project))]),
                64,
            )
            .expect("project name"),
        );
    }
    let tree = header(&task, Some(&me));
    assert!(named(
        tree.nodes(),
        Role::Paragraph,
        &fixtures::words()
            .say(crate::words::Word::Project, &[skein_ui::phrases::Hole::Number(u64::from(task.project))])
    ));
}
#[test]
fn the_bar_links_only_a_requesting_task_before_the_current_task_paragraph() {
    use temper_client_domain::{Action, Address, Event, pages::Act};
    for requester in
        [c::Requester::Task(7), c::Requester::Person(PARTY), c::Requester::Deployment { project: fixtures::PROJECT }]
    {
        let limits = fixtures::domain_limits();
        let (mut root, _, watch, _) = fixtures::root_at_with_tree(Address::Task { task: TASK }, &limits);
        let mut task = titled();
        task.priority = None;
        task.chip.executor = c::ChipExecutor::Party { party: PARTY };
        task.requester = requester;
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: watch.expect("task watch"),
                        event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                    },
                },
            },
        );
        let mut view = fixtures::page_view(&root);
        let tree = view.tree();
        let mut main = None;
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.role == Role::Main {
                main = Some(index);
            }
        }
        let main = main.expect("main");
        let bar = children(tree, main).as_slice()[0];
        assert_eq!(tree.nodes()[bar].styles.as_slice(), [look::PAGEBAR]);
        let parts = children(tree, bar);
        let expected = if requester == c::Requester::Task(7) { 2 } else { 1 };
        assert_eq!(parts.len(), expected);
        let current = &tree.nodes()[*parts.as_slice().last().expect("current")];
        assert_eq!(current.role, Role::Paragraph);
        assert_eq!(current.name.as_deref(), Some(b"Task 9".as_slice()));
        if expected == 2 {
            let link = &tree.nodes()[parts.as_slice()[0]];
            assert_eq!(link.role, Role::Link);
            assert_eq!(link.name.as_deref(), Some(b"Task 7".as_slice()));
            let id = link.id;
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(Action::Pages(Act::Go { address: Address::Task { task: 7 } }))
            );
        }
    }
}
const PHASES: [(c::Phase, skein_ui::Style); 14] = [
    (c::Phase::Due, look::STATUS_ACTIVE),
    (c::Phase::Preparing, look::STATUS_ACTIVE),
    (c::Phase::Claimed, look::STATUS_ACTIVE),
    (c::Phase::Running, look::STATUS_ACTIVE),
    (c::Phase::Active, look::STATUS_ACTIVE),
    (c::Phase::Done, look::STATUS_DONE),
    (c::Phase::Held, look::STATUS_HELD),
    (c::Phase::Failed, look::STATUS_HELD),
    (c::Phase::BackingOff, look::STATUS_RETRYING),
    (c::Phase::Waiting, look::STATUS_WAITING),
    (c::Phase::Idle, look::STATUS_WAITING),
    (c::Phase::Closing, look::STATUS_QUIET),
    (c::Phase::Ended, look::STATUS_QUIET),
    (c::Phase::Cancelled, look::STATUS_QUIET),
];
#[test]
fn every_phase_has_the_same_status_token_in_the_header_and_its_plan_row() {
    let root = held_agent_root();
    for (phase, token) in PHASES {
        let mut task = titled();
        task.chip.phase = phase;
        task.waits_on = c::WaitsOn::Nothing;
        let header = header(&task, None);
        assert_eq!(fixtures::styles(&header, Role::Status).as_ref(), [Box::from([token])]);
        let mut delivery = f::tree();
        let c::Delivery::Tree(tree) = &mut delivery else { panic!("owner") };
        let mut row = row();
        row.chip = task.chip;
        row.chip.task = CHILD;
        tree.rows = List::with_capacity(ONE);
        tree.rows.push(row).expect("row");
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::pages::task::plan(
            &mut builder,
            Some(&delivery),
            &root,
            &fixtures::words(),
            &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
            fixtures::domain_limits().client.domain.rows,
            crate::pages::task::Plan { word: crate::words::Word::Plan, ended: None },
        );
        let tree = builder.finish();
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.role == Role::TreeItem {
                let row = children(&tree, index).as_slice()[0];
                assert_eq!(tree.nodes()[row].styles.as_slice(), [look::PLAN_ROW]);
                let parts = subtree(&tree, row);
                assert!(named(parts, Role::Link, b"Task 12"));
                let mut count = 0_u32;
                for status in parts {
                    if status.role == Role::Status {
                        count = count.checked_add(1).expect("bounded statuses");
                        assert_eq!(status.styles.as_slice(), [token]);
                    }
                }
                assert_eq!(count, 1);
            }
        }
    }
}
#[test]
fn the_agents_held_card_keeps_its_key_without_an_own_row() {
    let view = fixtures::page_view(&held_agent_root());
    let tree = view.tree();
    let hold = styled(tree, look::PLAN_HOLD);
    assert_eq!(tree.nodes()[hold].role, Role::List);
    let items = children(tree, hold);
    assert_eq!(items.len(), 1);
    let item = &tree.nodes()[items.as_slice()[0]];
    assert_eq!(item.role, Role::Item);
    assert_eq!(item.key, Some(skein_ui::Key { owner: crate::frame::owner(), kind: 2, a: TASK, b: 0 }));
    let plan = styled(tree, look::PLAN);
    assert!(children(tree, plan).is_empty());
    assert!(hold < plan);
    assert!(fixtures::find(tree, Role::TreeItem, b"Task 9").is_none());
}

const RESULT_TASK: u64 = 7;
const HEARD_WORDS: &[u8] = b"Two crates hold the export";
fn heard_word(from: c::Requester, kind: c::MessageKind) -> c::Word {
    c::Word {
        number: 1,
        from,
        kind,
        words: result_words(HEARD_WORDS),
        at: skein_lib::Wall::EPOCH,
        hits: 1,
        eligible: true,
    }
}

fn assert_no_activity_link(tree: &Tree) {
    for node in tree.nodes() {
        assert_ne!(node.role, Role::Link);
    }
}

#[test]
fn the_chats_owner_activity_draws_no_link() {
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::pages::task::owner_activity(
        &mut builder,
        &c::Activity::Heard(heard_word(c::Requester::Task(RESULT_TASK), c::MessageKind::Result(c::WordResult::Report))),
        &fixtures::words(),
    );
    let tree = builder.finish();
    assert_no_activity_link(&tree);
    assert_eq!(fixtures::text(&tree).as_ref(), HEARD_WORDS);
}

#[test]
fn a_kept_activity_is_parked() {
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::pages::task::owner_activity(
        &mut builder,
        &c::Activity::Kept { attempt: 1, pending: List::with_capacity(0), question: None },
        &fixtures::words(),
    );
    assert_eq!(fixtures::text(&builder.finish()).as_ref(), b"parked");
}

fn region_index(tree: &Tree, name: &[u8]) -> usize {
    let id = fixtures::find(tree, Role::Region, name).expect("page region");
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.id == id {
            return index;
        }
    }
    panic!("region index");
}
fn result_words(bytes: &[u8]) -> c::Text {
    c::Text::new(Box::from(bytes), 256).expect("result words")
}

const DEPENDENCY: u64 = 17;
const OTHER_DEPENDENCY: u64 = 23;
const GRANDCHILD: u64 = 13;
const ROW_SPENT: u64 = 5;
fn owner_phase(chip: &mut c::Chip, phase: c::ChipPhase) {
    chip.phase = phase.compact();
    let c::ChipPhaseFacts { phase: target, .. } = chip.details.as_mut();
    *target = phase;
}
fn row() -> c::TreeRow {
    let mut chip = f::document().chip;
    chip.task = CHILD;
    let c::ChipPhaseFacts { details, .. } = chip.details.as_mut();
    details.budget.numbers.spent = ROW_SPENT;
    c::TreeRow {
        chip,
        parent: TASK,
        depth: ONE,
        proposals: List::with_capacity(NO_ROWS),
        spent: ROW_SPENT + 7,
        dependencies: List::with_capacity(NO_ROWS),
        cut: false,
    }
}
fn plan(delivery: &c::Delivery) -> Tree {
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::pages::task::plan(
        &mut builder,
        Some(delivery),
        &f::root(c::ChipExecutor::Agent { charter: CHARTER }, Some(PRIORITY)),
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        fixtures::domain_limits().client.domain.rows,
        crate::pages::task::Plan { word: crate::words::Word::Plan, ended: None },
    );
    builder.finish()
}
fn plan_with_row(row: c::TreeRow) -> Tree {
    let mut delivery = f::tree();
    let c::Delivery::Tree(tree) = &mut delivery else { panic!("owner tree") };
    tree.rows = List::with_capacity(ONE);
    tree.rows.push(row).expect("row");
    plan(&delivery)
}
fn index_of(tree: &Tree, role: Role, name: &[u8]) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == role && node.name.as_deref() == Some(name) {
            return index;
        }
    }
    panic!("missing {role:?}: {}", String::from_utf8_lossy(name));
}
fn assert_status(tree: &Tree, name: &[u8], token: skein_ui::Style) {
    let index = index_of(tree, Role::Status, name);
    assert_eq!(tree.nodes()[index].styles.as_slice(), [token]);
    assert_eq!(fixtures::node_text(tree, index).as_ref(), name);
}

#[test]
fn a_plan_row_says_what_its_task_waits_for() {
    let cases: &[(&[u64], bool, bool, &[u8])] = &[
        (&[DEPENDENCY], false, true, b"waiting on task 17"),
        (&[DEPENDENCY, OTHER_DEPENDENCY], false, false, b"waiting on 2 tasks"),
        (&[DEPENDENCY], true, true, b"waiting on more than 1 tasks"),
        (&[], true, true, b"waiting on more than 0 tasks"),
        (&[], false, true, b"waiting on its resources"),
        (&[], false, false, b"waiting"),
    ];
    for (numbers, more, holds, expected) in cases {
        let mut row = row();
        let mut dependencies = List::with_capacity(2);
        for number in *numbers {
            dependencies.push(*number).expect("dependency");
        }
        owner_phase(&mut row.chip, c::ChipPhase::Waiting { dependencies, more: *more, holds: *holds });
        let tree = plan_with_row(row);
        assert_status(&tree, expected, look::STATUS_WAITING);
        assert!(skein_lib::bytes::find(&fixtures::text(&tree), expected).is_some());
    }
}
fn document_text(words: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(skein_lib::bytes::copy_of(words), 32).expect("document text"))
}
#[test]
fn an_ended_row_says_its_ending() {
    let cases: [(c::ChipEnding, &[u8], skein_ui::Style); 3] = [
        (c::ChipEnding::Done(c::ResultValue::Report(document_text(b"Done"))), b"Completed", look::STATUS_DONE),
        (c::ChipEnding::Failed(document_text(b"Failed")), b"Failed", look::STATUS_HELD),
        (
            c::ChipEnding::Cancelled {
                reason: document_text(b"Cancelled"),
                result: None,
                by: c::Requester::Person(PARTY),
                at: skein_lib::Wall::EPOCH,
            },
            b"Cancelled",
            look::STATUS_QUIET,
        ),
    ];
    for (ending, expected, token) in cases {
        let mut row = row();
        owner_phase(&mut row.chip, c::ChipPhase::Ended);
        let c::ChipPhaseFacts { details, .. } = row.chip.details.as_mut();
        details.ending = Some(ending);
        assert_status(&plan_with_row(row), expected, token);
    }
}
#[test]
fn a_rows_spend_is_its_supplied_spent() {
    let tree = plan_with_row(row());
    let spend = styled(&tree, look::PLAN_SPEND);
    let node = &tree.nodes()[spend];
    assert_eq!(node.role, Role::Paragraph);
    assert_eq!(node.name.as_deref(), Some(b"5 spent".as_slice()));
    assert_eq!(fixtures::node_text(&tree, spend).as_ref(), b"5 spent");
    let slot = styled(&tree, look::PLAN_ROW);
    let parts = children(&tree, slot);
    assert_eq!(parts.as_slice().last(), Some(&spend));
    assert_eq!(tree.nodes()[parts.as_slice()[2]].role, Role::Status);
    let mut row = row();
    row.chip.details.details.budget.numbers.spent = 0;
    assert!(fixtures::find(&plan_with_row(row), Role::Paragraph, b"0 spent").is_some());
}
fn dependency_row() -> c::TreeRow {
    let mut row = row();
    let mut live = f::document().chip;
    live.task = DEPENDENCY;
    owner_phase(&mut live, c::ChipPhase::Active(c::RunStage::Running { attempt: 1 }));
    row.dependencies = List::with_capacity(4);
    for link in [
        c::TaskLink::Live(live),
        c::TaskLink::Stored { number: OTHER_DEPENDENCY, status: Some(c::TaskStatus::Done) },
        c::TaskLink::Stored { number: TASK, status: None },
    ] {
        row.dependencies.push(link).expect("dependency");
    }
    row
}
#[test]
fn a_rows_dependency_line_lists_its_dependencies() {
    let mut delivery = f::tree();
    let c::Delivery::Tree(owner) = &mut delivery else { panic!("owner tree") };
    owner.rows = List::with_capacity(2);
    owner.rows.push(dependency_row()).expect("row");
    let mut child = row();
    child.chip.task = GRANDCHILD;
    child.parent = CHILD;
    child.depth = 2;
    owner.rows.push(child).expect("child");
    let tree = plan(&delivery);
    let list = styled(&tree, look::DEPENDENCIES);
    assert_eq!(tree.nodes()[list].role, Role::List);
    assert_eq!(tree.nodes()[list].name.as_deref(), Some(b"Dependencies of task 12".as_slice()));
    let items = children(&tree, list);
    assert_eq!(items.len(), 3);
    for (index, number, name, status) in [
        (0, DEPENDENCY, b"Task 17".as_slice(), Some(b"running".as_slice())),
        (1, OTHER_DEPENDENCY, b"Task 23", Some(b"Completed".as_slice())),
        (2, TASK, b"Task 9", None),
    ] {
        let item = items.as_slice()[index];
        assert_eq!(tree.nodes()[item].role, Role::Item);
        assert_eq!(tree.nodes()[item].key, Some(crate::pages::inbox::card_key(c::ObjectKey::Task(number))));
        let parts = children(&tree, item);
        let link = &tree.nodes()[parts.as_slice()[0]];
        assert_eq!(link.role, Role::Link);
        assert_eq!(link.name.as_deref(), Some(name));
        assert!(link.binding.is_some());
        if let Some(status) = status {
            assert_eq!(parts.len(), 2);
            assert_eq!(tree.nodes()[parts.as_slice()[1]].name.as_deref(), Some(status));
        } else {
            assert_eq!(parts.len(), 1);
        }
    }
    assert_status(&tree, b"running", look::STATUS_ACTIVE);
    assert_status(&tree, b"Completed", look::STATUS_DONE);
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::TreeItem && node.name.as_deref() == Some(b"Task 12") {
            let parts = children(&tree, index);
            assert_eq!(parts.len(), 3);
            assert_eq!(tree.nodes()[parts.as_slice()[0]].styles.as_slice(), [look::PLAN_ROW]);
            assert_eq!(parts.as_slice()[1], list);
            assert_eq!(tree.nodes()[parts.as_slice()[2]].role, Role::TreeItem);
        }
    }
}
fn amended_dependency_row() -> c::TreeRow {
    let mut row = dependency_row();
    row.dependencies
        .push(c::TaskLink::Stored { number: GRANDCHILD, status: Some(c::TaskStatus::Done) })
        .expect("original dependency");
    let c::ChipPhaseFacts { details, .. } = row.chip.details.as_mut();
    details.dependencies = f::amended_dependency_facts();
    row
}
fn assert_dependency_item(tree: &Tree, item: usize, number: u64, status: Option<&[u8]>, mark: Option<&[u8]>) {
    assert_eq!(tree.nodes()[item].role, Role::Item);
    assert_eq!(tree.nodes()[item].key, Some(crate::pages::inbox::card_key(c::ObjectKey::Task(number))));
    let parts = children(tree, item);
    let link = &tree.nodes()[parts.as_slice()[0]];
    assert_eq!(link.role, Role::Link);
    let name = fixtures::words().say(crate::words::Word::Task, &[skein_ui::phrases::Hole::Number(number)]);
    assert_eq!(link.name.as_deref(), Some(name.as_ref()));
    assert!(link.binding.is_some());
    let mut count = 1;
    if let Some(status) = status {
        let node = &tree.nodes()[parts.as_slice()[count]];
        assert_eq!(node.role, Role::Status);
        assert_eq!(node.name.as_deref(), Some(status));
        count = count.checked_add(1).expect("dependency children fit");
    }
    if let Some(mark) = mark {
        let index = parts.as_slice()[count];
        let node = &tree.nodes()[index];
        assert_eq!(node.role, Role::Paragraph);
        assert_eq!(node.name.as_deref(), Some(mark));
        assert_eq!(fixtures::node_text(tree, index).as_ref(), mark);
        assert!(node.binding.is_none());
        count = count.checked_add(1).expect("dependency children fit");
    }
    assert_eq!(parts.as_slice().len(), count);
}
#[test]
fn an_added_dependency_is_marked_in_its_item() {
    let tree = plan_with_row(amended_dependency_row());
    let list = styled(&tree, look::DEPENDENCIES);
    assert_eq!(tree.nodes()[list].name.as_deref(), Some(b"Dependencies of task 12".as_slice()));
    let items = children(&tree, list);
    assert_eq!(items.len(), 4);
    assert_dependency_item(
        &tree,
        items.as_slice()[0],
        DEPENDENCY,
        Some(b"running"),
        Some("Added by you · keeps task 12 from ending".as_bytes()),
    );
    assert_dependency_item(&tree, items.as_slice()[1], OTHER_DEPENDENCY, Some(b"Completed"), Some(b"Added by task 7"));
    assert_dependency_item(&tree, items.as_slice()[2], TASK, None, None);
    assert_dependency_item(&tree, items.as_slice()[3], GRANDCHILD, Some(b"Completed"), None);
}
#[test]
fn a_cut_dependency_line_ends_with_its_last_line() {
    for populated in [false, true] {
        let mut row = if populated { dependency_row() } else { row() };
        row.cut = true;
        let tree = plan_with_row(row);
        let list = styled(&tree, look::DEPENDENCIES);
        let items = children(&tree, list);
        assert_eq!(items.len(), if populated { 4 } else { 1 });
        let item = *items.as_slice().last().expect("cut item");
        assert_eq!(tree.nodes()[item].role, Role::Item);
        assert_eq!(tree.nodes()[item].key, None);
        let parts = children(&tree, item);
        assert_eq!(parts.len(), 1);
        let line = &tree.nodes()[parts.as_slice()[0]];
        assert_eq!(line.role, Role::Paragraph);
        assert_eq!(line.name.as_deref(), Some(b"More dependencies not shown.".as_slice()));
    }
}
#[test]
fn the_goal_has_no_plan_row() {
    let mut delivery = f::tree();
    let c::Delivery::Tree(owner) = &mut delivery else { panic!("owner tree") };
    let mut dependencies = List::with_capacity(ONE);
    dependencies.push(DEPENDENCY).expect("dependency");
    owner_phase(&mut owner.root, c::ChipPhase::Waiting { dependencies, more: false, holds: false });
    let c::ChipPhaseFacts { details, .. } = owner.root.details.as_mut();
    details.dependencies = List::with_capacity(ONE);
    details
        .dependencies
        .push(c::Dependency { number: DEPENDENCY, pending: true, added: None, keeps: true, status: None })
        .expect("dependency fact");
    let tree = plan(&delivery);
    assert!(fixtures::find(&tree, Role::TreeItem, b"Task 9").is_none());
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&look::DEPENDENCIES));
        assert!(!node.styles.as_slice().contains(&look::PLAN_ROW));
    }
}
#[test]
fn the_header_says_what_the_task_waits_for() {
    let mut task = titled();
    let mut dependencies = List::with_capacity(ONE);
    dependencies.push(DEPENDENCY).expect("dependency");
    task.waits_on = c::WaitsOn::Dependencies { pending: dependencies, count: 1 };
    assert_status(&header(&task, None), b"waiting on task 17", look::STATUS_WAITING);
}

pub(super) fn dependency_plan() -> Tree {
    plan_with_row(dependency_row())
}

fn observed(chip: &mut c::Chip, work: Option<c::ChipWork>) {
    owner_phase(chip, c::ChipPhase::Active(c::RunStage::Running { attempt: ONE.into() }));
    let c::ChipPhaseFacts { details, .. } = chip.details.as_mut();
    if let Some(work) = work {
        details.observation = Some(c::ChipObservation {
            channel: skein_lib::Token::new(1),
            work,
            since: skein_lib::Wall::EPOCH,
            reported_at: skein_lib::Wall::EPOCH,
            started: None,
        });
    } else {
        details.observation = None;
    }
}
#[test]
fn a_running_row_and_header_say_the_observed_work() {
    let cases: [(Option<c::ChipWork>, &[u8]); 4] = [
        (None, b"running"),
        (Some(c::ChipWork::Writing), "running · writing".as_bytes()),
        (
            Some(c::ChipWork::Tool { call: Box::from(*b"call"), tool: Box::from(*b"read_file") }),
            "running · a tool · read_file".as_bytes(),
        ),
        (Some(c::ChipWork::Waiting), b"running"),
    ];
    for (work, expected) in cases {
        let mut row = row();
        observed(&mut row.chip, work);
        let mut task = titled();
        task.chip = row.chip.clone();
        task.waits_on = c::WaitsOn::Nothing;
        assert_status(&header(&task, None), expected, look::STATUS_ACTIVE);
        assert_status(&plan_with_row(row), expected, look::STATUS_ACTIVE);
    }
}
fn party_wait(whom: c::Whom) -> c::WaitsOn {
    c::WaitsOn::Party { whom, entries: ONE, task: CHILD, kind: c::WaitedFor::Question }
}
#[test]
fn the_header_says_its_exact_wait_in_both_shells() {
    let mut task = titled();
    task.priority = None;
    let root = held_agent_root();
    let mut me = root.pages().me(root.client()).expect("me").clone();
    me.projects.get_mut(0).expect("project").role = c::HeldRole::Owner;
    let mut pending = List::with_capacity(ONE);
    pending.push(DEPENDENCY).expect("pending");
    let cases: [(c::WaitsOn, &[u8]); 6] = [
        (c::WaitsOn::Dependencies { pending, count: 3 }, b"waiting on 3 tasks"),
        (party_wait(c::Whom::Person(me.party.number)), b"waiting on an answer from you"),
        (party_wait(c::Whom::Role { project: task.project, role: 0 }), b"waiting on an answer from you"),
        (party_wait(c::Whom::Person(DEPENDENCY)), b"waiting on an answer from person 17"),
        (party_wait(c::Whom::Role { project: task.project, role: 7 }), b"waiting on an answer from role 7"),
        (c::WaitsOn::News, b"waiting on news"),
    ];
    for words in [fixtures::words(), fixtures::browser_words()] {
        for (waits_on, expected) in &cases {
            task.waits_on = waits_on.clone();
            let mut builder = Builder::new(&fixtures::tree_limits());
            crate::pages::task::header(
                &mut builder,
                &task,
                Some(&me),
                &words,
                &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
                None,
            );
            let tree = builder.finish();
            assert_status(&tree, expected, look::STATUS_WAITING);
        }
        task.waits_on = c::WaitsOn::Delegates;
        for (count, expected) in
            [(0, b"waiting on 0 delegates".as_slice()), (1, b"waiting on 1 delegate"), (2, b"waiting on 2 delegates")]
        {
            let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
            details.live_delegates = count;
            let mut builder = Builder::new(&fixtures::tree_limits());
            crate::pages::task::header(
                &mut builder,
                &task,
                Some(&me),
                &words,
                &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
                None,
            );
            assert_status(&builder.finish(), expected, look::STATUS_WAITING);
        }
    }
}
#[test]
fn the_plan_omits_the_goal_and_its_hold_and_keep_its_folds() {
    let root = held_agent_root();
    let mut owner = f::tree();
    let c::Delivery::Tree(tree) = &mut owner else { panic!("owner tree") };
    tree.root.phase = c::Phase::Held;
    tree.rows = List::with_capacity(ONE);
    tree.rows.push(row()).expect("row");
    tree.folded = List::with_capacity(ONE);
    tree.folded
        .push(c::TreeFold {
            parent: TASK,
            phases: c::PhaseCounts { running: 2, ..c::PhaseCounts::ZERO },
            held_below: 0,
        })
        .expect("root fold");
    {
        let delivery = owner;
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::pages::task::plan(
            &mut builder,
            Some(&delivery),
            &root,
            &fixtures::words(),
            &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
            fixtures::domain_limits().client.domain.rows,
            crate::pages::task::Plan { word: crate::words::Word::Plan, ended: None },
        );
        let tree = builder.finish();
        assert!(fixtures::find(&tree, Role::TreeItem, b"Task 9").is_none());
        let top = children(&tree, styled(&tree, look::PLAN));
        let nodes = tree.nodes();
        assert!(!has_style(tree.nodes(), look::PLAN_HOLD));
        assert_eq!(nodes[top.as_slice()[0]].role, Role::Paragraph);
        assert_eq!(nodes[top.as_slice()[0]].name.as_deref(), Some(b"Folded: 2 Running".as_slice()));
        assert_eq!(nodes[top.as_slice()[1]].role, Role::TreeItem);
        assert_eq!(nodes[top.as_slice()[1]].name.as_deref(), Some(b"Task 12".as_slice()));
    }
}

fn has_role(nodes: &[Node], role: Role) -> bool {
    for node in nodes {
        if node.role == role {
            return true;
        }
    }
    false
}
fn has_style(nodes: &[Node], style: skein_ui::Style) -> bool {
    for node in nodes {
        if node.styles.as_slice().contains(&style) {
            return true;
        }
    }
    false
}
fn agent(task: c::Task) -> crate::View {
    fixtures::page_view(&f::agent_root(task, Some(f::tree()), true))
}
// client/pages/task.md, 2: an agent shares the chat's work line and reading control.
fn agent_work_page(ended: bool) -> c::ActivityPage {
    use fixtures::first_run as run;
    let mut rows = List::with_capacity(2);
    if ended {
        for (position, seconds, block) in
            [(1, 12_u64, c::Block::Text(run::text(b"Checking."))), (2, 41, run::call(1, b"read", b"{}", Some(b"Read")))]
        {
            let mut row = run::turn(
                position,
                f::RUN_ATTEMPT,
                u32::try_from(position).unwrap(),
                &[run::message(c::Speaker::Assistant, &[block])],
                512,
            );
            row.source.task = TASK;
            row.source.at = skein_lib::Wall::from_nanos(seconds.checked_mul(1_000_000_000).unwrap());
            let decoded = row.decoded.as_mut().unwrap();
            decoded.task = TASK;
            decoded.at = row.source.at;
            rows.push(row).unwrap();
        }
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}
fn agent_work_root(page: c::ActivityPage, writing: Option<&[u8]>) -> (Domain, skein_lib::Token) {
    use jig_client::{Event as E, Up};
    use skein_lib::{Env, Queue, Time, Token};
    let limits = fixtures::domain_limits();
    let (mut root, _, watch, _, _, _) = fixtures::root_at_with_forges(
        temper_client_domain::Address::Task { task: TASK },
        &limits,
        b"Shop",
        None,
        Some(page),
        None,
    );
    let watch = watch.unwrap();
    let mut task = f::delegate();
    task.activity = 2;
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        &mut root,
        &Env { now: Time::ZERO, wall: fixtures::NOW, limits },
        Event::Client {
            event: E::Up {
                up: Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))) },
            },
        },
        &mut out,
    );
    let mut run_watch = None;
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Run { .. }) },
                },
        } = request
        {
            run_watch = Some(watch);
        }
    }
    let text = fixtures::first_run::text;
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: E::Up {
                up: Up::Watched {
                    watch: run_watch.expect("agent run opened"),
                    event: c::WatchEvent::Delivered(c::Delivery::RunState(Box::new(c::RunState {
                        task: TASK,
                        attempt: Some(f::RUN_ATTEMPT),
                        committed_turn: 2,
                        writing: match writing {
                            Some(words) => Some(c::RunText {
                                attempt: f::RUN_ATTEMPT,
                                began: None,
                                turn: c::ProvisionalTurnName { attempt: f::RUN_ATTEMPT, number: 3, retry: 0 },
                                from: 0,
                                through: u32::try_from(words.len()).unwrap(),
                                text: text(words),
                                cut: false,
                                call: None,
                                writing: None,
                            }),
                            None => None,
                        },
                        call: Some(c::CurrentCall {
                            identity: c::CallIdentity {
                                owner: Token::new(1),
                                sequence: 2,
                                retry: 0,
                                id: text(b"current"),
                            },
                            tool: text(b"settings"),
                            input: text(br#"{"setting":{"kind":"template","number":2}}"#),
                            cut: false,
                            began: None,
                        }),
                    }))),
                },
            },
        },
    );
    assert!(root.pages().task().unwrap().current_call().is_some(), "call delivered through the run watch");
    (root, watch)
}
fn agent_run_line(tree: &Tree) -> &Node {
    &tree.nodes()[styled(tree, look::WORK)]
}
#[test]
fn an_agents_run_says_the_call_under_way() {
    for writing in [None, Some(b"".as_slice())] {
        let (root, _) = agent_work_root(agent_work_page(true), writing);
        let view = fixtures::browser_page_view(&root);
        let line = agent_run_line(view.tree());
        assert!(line.styles.as_slice().contains(&look::WORK_RUNNING));
        assert_eq!(line.description.as_deref(), Some(b"checking template 2".as_slice()));
        assert_eq!(line.name.as_deref(), Some(b"Read 1 file".as_slice()));
    }
}
#[test]
fn an_agents_call_with_none_ended_is_a_line() {
    let (root, _) = agent_work_root(agent_work_page(false), None);
    let view = fixtures::browser_page_view(&root);
    let tree = view.tree();
    assert_eq!(agent_run_line(tree).name.as_deref(), Some(b"Checking template 2".as_slice()));
    let calls = subtree(tree, styled(tree, look::WORK_CALLS));
    assert_eq!(children(tree, styled(tree, look::WORK_CALLS)).len(), 1);
    assert!(named(calls, Role::Paragraph, b"settings template 2"));
}
#[test]
fn an_agents_written_words_settle_its_run() {
    let (root, _) = agent_work_root(agent_work_page(true), Some(b"The answer."));
    let view = fixtures::browser_page_view(&root);
    let line = agent_run_line(view.tree());
    assert!(!line.styles.as_slice().contains(&look::WORK_RUNNING));
    assert_eq!(line.description.as_deref(), Some(b"29 s".as_slice()));
    assert!(!named(view.tree().nodes(), Role::Paragraph, b"settings template 2"));
    assert!(contains(view.tree(), b"The answer."));
}
#[test]
fn no_run_out_settles_an_agents_line() {
    let (mut root, watch) = agent_work_root(agent_work_page(true), None);
    let mut task = f::delegate();
    task.activity = 2;
    f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    let view = fixtures::browser_page_view(&root);
    let line = agent_run_line(view.tree());
    assert!(!line.styles.as_slice().contains(&look::WORK_RUNNING));
    assert_eq!(line.description.as_deref(), Some(b"29 s".as_slice()));
    assert!(!named(view.tree().nodes(), Role::Paragraph, b"settings template 2"));
}
#[test]
fn an_agents_page_has_no_dots() {
    for running in [false, true] {
        let mut task = f::delegate();
        task.activity = 0;
        if !running {
            f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
        }
        let root = f::agent_root(task, None, false);
        let view = fixtures::browser_page_view(&root);
        assert!(!has_style(view.tree().nodes(), look::TYPING));
    }
}
#[test]
fn an_agents_page_has_latest_in_a_browser() {
    let mut root = amend_root(false);
    let view = fixtures::browser_page_view(&root);
    let tree = view.tree();
    let pill = index_of(tree, Role::Button, b"Latest");
    let log = index_of(tree, Role::Log, b"Conversation");
    assert_eq!(pill, log + usize::try_from(tree.nodes()[log].size).unwrap());
    assert!(tree.nodes()[pill].styles.as_slice().contains(&look::LATEST));
    assert!(tree.nodes()[pill].states.as_slice().contains(&skein_ui::State::ToLogEnd));
    let brief = styled(tree, look::AGENT_BRIEF);
    let run = styled(tree, look::AGENT_RUN);
    let foot = styled(tree, look::AGENT_FOOT);
    assert!(log < brief && brief < run && run < foot && foot < pill);
    press_button(&mut root, b"Amend");
    let view = fixtures::browser_page_view(&root);
    let tree = view.tree();
    assert!(styled(tree, look::TASK_AMEND) < index_of(tree, Role::Log, b"Conversation"));
}
#[test]
fn latest_binds_where_the_window_let_its_end_go() {
    for dropped in [false, true] {
        let mut page = agent_work_page(true);
        if dropped {
            page.after = Some(c::Cursor::new(Box::from(*b"newer"), 32).unwrap());
        }
        let (root, _) = agent_work_root(page, None);
        let mut view = fixtures::browser_page_view(&root);
        let id = fixtures::find(view.tree(), Role::Button, b"Latest").expect("Latest");
        assert_eq!(
            crate::act(&mut view, id, skein_ui::Act::Press),
            if dropped { Some(Action::Pages(Act::Latest)) } else { None }
        );
    }
}
// Context and date headings must arrive above existing words (client/pages/chat/conversation.md, 2).
fn reading_chat(times: &[u64], earlier: bool) -> crate::View {
    let mut rows = List::with_capacity(u32::try_from(times.len()).unwrap());
    for (index, nanos) in times.iter().enumerate() {
        let position = u64::try_from(index).unwrap().checked_add(1).unwrap();
        rows.push(super::documents::line(
            position,
            skein_lib::Wall::from_nanos(*nanos),
            c::Activity::Words {
                message: position,
                by: c::Requester::Person(PARTY),
                kind: c::MessageKind::Words,
                words: fixtures::first_run::text(b"Retained words"),
            },
        ))
        .unwrap();
    }
    let page = c::ActivityPage {
        rows,
        before: if earlier { Some(c::Cursor::new(b"earlier".as_slice().into(), 32).unwrap()) } else { None },
        after: None,
        cut: false,
    };
    let (mut root, _, watch, _, _) = fixtures::root_at_with_activity(
        temper_client_domain::Address::Chat { task: TASK },
        &fixtures::domain_limits(),
        b"Shop",
        Some((fixtures::PROJECT, true, temper_client_protocol_wire::v1::PlaygroundState::Running)),
        Some(page),
        None,
    );
    let source = fixtures::chat::chat_root();
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.activity = u64::try_from(times.len()).unwrap();
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    fixtures::browser_page_view(&root)
}
#[test]
fn the_chat_context_arrives_only_with_its_start() {
    for earlier in [false, true] {
        let view = reading_chat(&[2_000_000_000], earlier);
        let tree = view.tree();
        let log = index_of(tree, Role::Log, b"Conversation");
        assert_eq!(has_style(subtree(tree, log), look::CONTEXT), !earlier);
    }
}
#[test]
fn a_days_key_follows_its_first_line_when_earlier_words_arrive() {
    const DAY: u64 = 86_400_000_000_000;
    let keys = |times: &[u64]| {
        let view = reading_chat(times, true);
        let mut keys = List::with_capacity(u32::try_from(times.len()).unwrap());
        for node in view.tree().nodes() {
            if node.styles.as_slice().contains(&look::DAY) {
                keys.push(node.key.expect("keyed day")).unwrap();
            }
        }
        keys.into_boxed()
    };
    let original = keys(&[DAY + 2_000_000_000]);
    let same_day = keys(&[DAY + 1_000_000_000, DAY + 2_000_000_000]);
    let earlier_day = keys(&[1_000_000_000, DAY + 2_000_000_000]);
    assert_eq!(original.len(), 1);
    assert_eq!(same_day.len(), 1);
    assert_eq!(earlier_day.len(), 2);
    assert_ne!(original[0], same_day[0], "an earlier line replaces the day heading above the words");
    assert_eq!(original[0], earlier_day[1], "an earlier day leaves this heading in place");
}
#[test]
fn an_agents_terminal_has_no_latest() {
    let root = f::agent_root(f::delegate(), None, true);
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Button, b"Latest").is_none());
}
#[test]
fn an_agents_task_has_its_parts_in_order() {
    let view = agent(f::delegate());
    let tree = view.tree();
    let lineage = styled(tree, look::AGENT_LINEAGE);
    let bar = styled(tree, look::TASKBAR);
    let brief = styled(tree, look::AGENT_BRIEF);
    let run = styled(tree, look::AGENT_RUN);
    let log = index_of(tree, Role::Log, b"Conversation");
    assert!(lineage < bar && bar < brief && brief < run && run < log);
    assert_eq!(styled(tree, look::AGENT_FOOT), log + usize::try_from(tree.nodes()[log].size).unwrap());
    for name in [b"Task details".as_slice(), b"What was asked", b"Latest activity", b"Budget and authority"] {
        assert!(!named(tree.nodes(), Role::Heading { level: 2 }, name));
    }
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&look::COMPOSER));
        assert!(!node.styles.as_slice().contains(&look::COMPOSER_AREA));
        assert_ne!(node.role, Role::TextArea);
        for name in [b"Pause".as_slice(), b"Older", b"Mark seen"] {
            assert!(!(node.role == Role::Button && node.name.as_deref() == Some(name)));
        }
    }
}
#[test]
fn the_lineage_bar_links_the_parent_by_number_with_its_title() {
    for (lineage, requester, parent, title) in [
        (true, c::Requester::Task(23), Some(b"Task 12".as_slice()), true),
        (false, c::Requester::Task(23), Some(b"Task 23".as_slice()), false),
        (false, c::Requester::Person(PARTY), None, false),
    ] {
        let mut task = f::delegate();
        task.requester = requester;
        if !lineage {
            task.lineage_chips.clear();
        }
        let mut view = agent(task);
        let tree = view.tree();
        let bar = styled(tree, look::AGENT_LINEAGE);
        let parts = children(tree, bar);
        assert_eq!(parts.len(), if parent.is_some() { 2 } else { 1 });
        let nodes = subtree(tree, bar);
        assert!(named(nodes, Role::Paragraph, b"Task 9"));
        assert!(named(nodes, Role::Paragraph, b"Refresh token handling"));
        assert_eq!(named(nodes, Role::Paragraph, b"Rotate the credentials"), title);
        assert!(!named(nodes, Role::Link, b"Task 3"), "only the immediate parent");
        if let Some(name) = parent {
            let link = &tree.nodes()[index_of(tree, Role::Link, name)];
            assert!(link.binding.is_some(), "parent link navigates");
            assert!(named(subtree(tree, parts.as_slice()[0]), Role::Link, name));
            let id = link.id;
            let number = if lineage { CHILD } else { OTHER_DEPENDENCY };
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(Action::Pages(Act::Go { address: temper_client_domain::Address::Task { task: number } })),
            );
        }
    }
}
#[test]
fn the_taskbar_says_number_title_state_executor_and_own_spend() {
    let mut task = f::delegate();
    task.budget.numbers.spent = 7;
    task.budget.numbers.spent_below = 91;
    let view = agent(task);
    let tree = view.tree();
    let nodes = subtree(tree, styled(tree, look::TASKBAR));
    assert!(named(nodes, Role::Heading { level: 1 }, "T9 · Refresh token handling".as_bytes()));
    assert!(named(nodes, Role::Status, "running · writing".as_bytes()));
    assert!(named(nodes, Role::Paragraph, b"Agent"));
    assert!(named(nodes, Role::Paragraph, b"7 spent"));
    assert!(!named(nodes, Role::Paragraph, b"91 spent"));
    for node in nodes {
        if node.role == Role::Status {
            assert_eq!(node.styles.as_slice(), [look::STATUS_ACTIVE]);
        }
    }
}
#[test]
fn a_closed_agents_task_says_what_it_ran_on_after_its_spend() {
    for latest in [None, Some(super::chat::model_effort("recorded-model", Some("low")))] {
        let mut task = f::delegate();
        task.activity = 1;
        task.llm = Some(super::chat::model_choice("next-model", Some("high")));
        task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(c::ResultValue::Report(
            f::document_words(b"Done"),
        )))));
        let expected = if latest.is_some() { "recorded-model · low" } else { "next-model · high" };
        let root = f::agent_root_with_page(task, None, super::chat::model_line_page(latest));
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let tree = view.tree();
            let bar = subtree(tree, styled(tree, look::TASKBAR));
            let mut spent = None;
            for (index, node) in bar.iter().enumerate() {
                if node.name.as_deref() == Some(b"7 spent") {
                    spent = Some(index);
                    break;
                }
            }
            let spent = spent.unwrap();
            let line = &bar[spent + usize::try_from(bar[spent].size).unwrap()];
            assert_eq!(line.role, Role::Paragraph);
            assert_eq!(line.name.as_deref(), Some(expected.as_bytes()));
            assert!(fixtures::find(tree, Role::Disclosure, b"Model").is_none());
            for node in tree.nodes() {
                assert!(!node.styles.as_slice().contains(&look::ACCOUNT_LINE));
            }
        }
    }
}
#[test]
fn the_brief_says_its_spec_who_asked_and_its_dependencies() {
    for dependencies in [false, true] {
        let mut task = f::delegate();
        task.cut = true;
        if dependencies {
            task.dependencies = List::with_capacity(2);
            task.dependencies.push(c::TaskLink::Live(row().chip)).expect("live dependency");
            task.dependencies
                .push(c::TaskLink::Stored { number: GRANDCHILD, status: Some(c::TaskStatus::Done) })
                .expect("ended dependency");
        }
        let view = agent(task);
        let tree = view.tree();
        let index = styled(tree, look::AGENT_BRIEF);
        let nodes = subtree(tree, index);
        assert!(named(nodes, Role::Heading { level: 2 }, b"Task brief"));
        assert!(skein_lib::bytes::find(&fixtures::node_text(tree, index), b"Rotate tokens safely.").is_some());
        assert!(named(nodes, Role::Paragraph, b"Asked by task 12"));
        assert_eq!(named(nodes, Role::List, b"Dependencies of task 9"), dependencies);
        assert!(!named(nodes, Role::Paragraph, b"More dependencies not shown."));
        if dependencies {
            let list = index_of(tree, Role::List, b"Dependencies of task 9");
            let items = children(tree, list);
            assert_eq!(items.len(), 2);
            for (item, number, name) in [
                (items.as_slice()[0], CHILD, b"Task 12".as_slice()),
                (items.as_slice()[1], GRANDCHILD, b"Task 13".as_slice()),
            ] {
                assert_eq!(tree.nodes()[item].key, Some(crate::pages::inbox::card_key(c::ObjectKey::Task(number))));
                assert!(named(subtree(tree, item), Role::Link, name));
                assert!(has_role(subtree(tree, item), Role::Status));
            }
            assert!(named(subtree(tree, items.as_slice()[1]), Role::Status, b"Completed"));
        }
    }
}
#[test]
fn the_briefs_added_dependency_is_marked() {
    let mut task = f::delegate();
    task.dependencies = amended_dependency_row().dependencies;
    task.dependency_facts = f::amended_dependency_facts();
    let view = agent(task);
    let tree = view.tree();
    let brief = styled(tree, look::AGENT_BRIEF);
    assert!(named(subtree(tree, brief), Role::List, b"Dependencies of task 9"));
    let list = index_of(tree, Role::List, b"Dependencies of task 9");
    let items = children(tree, list);
    assert_eq!(items.len(), 4);
    assert_dependency_item(
        tree,
        items.as_slice()[0],
        DEPENDENCY,
        Some(b"running"),
        Some("Added by you · keeps task 9 from ending".as_bytes()),
    );
    assert_dependency_item(tree, items.as_slice()[1], OTHER_DEPENDENCY, Some(b"Completed"), Some(b"Added by task 7"));
    assert_dependency_item(tree, items.as_slice()[2], TASK, None, None);
    assert_dependency_item(tree, items.as_slice()[3], GRANDCHILD, Some(b"Completed"), None);
}
#[test]
fn an_added_dependency_names_another_party_or_the_deployment() {
    for (by, expected) in [
        (c::Requester::Person(74), b"Added by party 74".as_slice()),
        (c::Requester::Deployment { project: 1 }, b"Added by the deployment".as_slice()),
    ] {
        let mut task = f::delegate();
        task.dependencies = List::with_capacity(ONE);
        task.dependencies
            .push(c::TaskLink::Stored { number: DEPENDENCY, status: Some(c::TaskStatus::Done) })
            .expect("dependency");
        task.dependency_facts = List::with_capacity(ONE);
        task.dependency_facts.push(f::added_dependency(DEPENDENCY, by, false)).expect("added dependency");
        let view = agent(task);
        let tree = view.tree();
        let items = children(tree, index_of(tree, Role::List, b"Dependencies of task 9"));
        assert_eq!(items.len(), 1);
        assert_dependency_item(tree, items.as_slice()[0], DEPENDENCY, Some(b"Completed"), Some(expected));
    }
}
#[test]
fn the_run_heading_says_each_phase() {
    for (phase, attempt, heading, started) in [
        (c::ChipPhase::Active(c::RunStage::Claimed { attempt: 2 }), Some(1), Some("Run 2 · live now"), true),
        (c::ChipPhase::Active(c::RunStage::Running { attempt: 2 }), Some(2), Some("Run 2 · live now"), true),
        (c::ChipPhase::Closing(c::ClosingStage::Run { attempt: 2 }), Some(2), Some("Run 2 · live now"), true),
        (c::ChipPhase::Active(c::RunStage::Idle), Some(2), Some("Run 2 · waiting"), true),
        (c::ChipPhase::Ended, Some(2), Some("Run 2 · ended"), false),
        (c::ChipPhase::Active(c::RunStage::Idle), None, Some("No run now"), false),
        (c::ChipPhase::Ended, None, None, false),
    ] {
        let mut task = f::delegate();
        f::phase(&mut task, phase);
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
        details.attempt = attempt;
        let view = agent(task);
        let tree = view.tree();
        assert_eq!(has_style(tree.nodes(), look::AGENT_RUN), heading.is_some());
        if let Some(heading) = heading {
            let nodes = subtree(tree, styled(tree, look::AGENT_RUN));
            assert!(named(nodes, Role::Heading { level: 2 }, heading.as_bytes()));
            assert_eq!(named(nodes, Role::Paragraph, b"Started 10-22 03:48Z"), started);
        }
    }
    let mut task = f::delegate();
    let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
    details.run_since = None;
    let view = agent(task);
    let tree = view.tree();
    assert!(!has_role(subtree(tree, styled(tree, look::AGENT_RUN)), Role::Paragraph));
}
#[test]
fn an_agents_conversation_is_drawn_as_a_chats() {
    let view = agent(f::delegate());
    let tree = view.tree();
    let log = index_of(tree, Role::Log, b"Conversation");
    let nodes = subtree(tree, log);
    assert!(named(nodes, Role::Paragraph, b"Temper"));
    assert!(skein_lib::bytes::find(&fixtures::node_text(tree, log), b"Checking the credentials.").is_some());
    assert!(has_style(nodes, look::WORK));
    assert!(named(nodes, Role::Paragraph, b"delegate: Delegated tasks: 17"));
    super::chat::assert_run_line(tree, b"delegate", b"", &[b"delegate: Delegated tasks: 17"]);
    let content = children(tree, styled(tree, look::SAYS));
    let first = *content.as_slice().first().expect("message text");
    assert_eq!(fixtures::node_text(tree, first).as_ref(), b"Checking the credentials.");
    assert!(first < index_of(tree, Role::Disclosure, b"delegate"));
    assert!(!has_style(nodes, look::CARD));
    let mut thread = jig_client::thread::Thread::new(
        TASK,
        &temper_client_domain::pages::chat::thread_limits(&fixtures::domain_limits()),
    );
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    thread.page(f::activity(true));
    let chat = fixtures::chat::tree(&thread);
    let chat_log = index_of(&chat, Role::Log, b"Conversation");
    assert_eq!(fixtures::node_text(tree, log), fixtures::node_text(&chat, chat_log));
}
#[test]
fn an_agents_message_folds_each_call_and_results_into_rows() {
    use fixtures::first_run as run;
    let assistant = run::message(
        c::Speaker::Assistant,
        &[run::call(1, b"read", b"", Some(b"First file")), run::call(2, b"read", b"", Some(b"Second file"))],
    );
    let tool = run::message(
        c::Speaker::Tool,
        &[
            c::Block::Result { call: 1, summary: run::text(b"Read completed"), failed: false },
            c::Block::Result { call: 2, summary: run::text(b"Read failed"), failed: true },
        ],
    );
    let mut page = f::activity(false);
    page.rows.push(run::turn(1, 1, 1, &[assistant, tool], 512)).unwrap();
    let mut task = f::delegate();
    task.activity = 1;
    let view = fixtures::page_view(&f::agent_root_with_page(task, Some(f::tree()), page));
    let tree = view.tree();
    super::chat::assert_run_line(tree, "Read 2 files".as_bytes(), b"", &[b"read: First file", b"read: Second file"]);
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&look::BY_TOOL));
    }
}
#[test]
// client/pages/task.md, 2: the conversation draws "Run N began" and
// "Run N ended"; milestones are not among its lines.
fn an_agent_pages_conversation_draws_its_runs_but_not_its_milestones() {
    let mut task = f::delegate();
    task.activity = 3;
    let mut page = f::activity(false);
    page.rows = List::with_capacity(3);
    for (position, line) in [
        (1, c::Activity::RunBegan { attempt: 1, why: c::RunWhy::Made, resumed: false }),
        (2, c::Activity::Milestone { position: 2 }),
        (3, c::Activity::RunEnded { attempt: 1, end: c::RunEnd::Finished, spent: 0 }),
    ] {
        page.rows
            .push(jig_client::contract::DecodedActivityLine {
                source: Box::new(c::ActivityLine { task: TASK, position, at: skein_lib::Wall::EPOCH, line }),
                decoded: None,
            })
            .expect("run and milestone lines");
    }
    let view = fixtures::page_view(&f::agent_root_with_page(task, Some(f::tree()), page));
    let tree = view.tree();
    let said = fixtures::node_text(tree, index_of(tree, Role::Log, b"Conversation"));
    assert!(skein_lib::bytes::find(&said, b"Run 1 began").is_some());
    assert!(skein_lib::bytes::find(&said, b"Run 1 ended").is_some());
    assert!(skein_lib::bytes::find(&said, b"Milestone").is_none(), "milestones stay out of the conversation");
}
#[test]
fn an_ended_agents_task_shows_its_result() {
    let mut task = f::delegate();
    f::phase(&mut task, c::ChipPhase::Ended);
    let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
    details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(f::document_words(b"Tokens rotated safely."))));
    let root = f::agent_root(task, None, true);
    let view = fixtures::page_view(&root);
    let tree = view.tree();
    let result = region_index(tree, b"Result");
    assert_eq!(tree.nodes()[result].styles.as_slice(), [look::AGENT_RESULT]);
    assert!(named(subtree(tree, result), Role::Status, b"Completed"));
    assert!(skein_lib::bytes::find(&fixtures::node_text(tree, result), b"Tokens rotated safely.").is_some());
    assert!(index_of(tree, Role::Log, b"Conversation") < result);
    assert!(!named(agent(f::delegate()).tree().nodes(), Role::Region, b"Result"));
    let view = fixtures::browser_page_view(&root);
    let tree = view.tree();
    let log = index_of(tree, Role::Log, b"Conversation");
    assert!(named(subtree(tree, log), Role::Region, b"Result"));
}
#[test]
fn an_agents_plan_shows_only_with_a_delegate_or_a_card() {
    let mut delegates = f::tree();
    let c::Delivery::Tree(tree) = &mut delegates else { panic!("owner tree") };
    tree.rows = List::with_capacity(1);
    tree.rows.push(row()).expect("delegate");
    let mut held_tree = f::tree();
    let c::Delivery::Tree(tree) = &mut held_tree else { panic!("owner tree") };
    tree.root.phase = c::Phase::Held;
    for (delivery, plan) in [(Some(f::tree()), false), (None, false), (Some(delegates), true), (Some(held_tree), true)]
    {
        let root = f::agent_root(f::delegate(), delivery, true);
        let view = fixtures::page_view(&root);
        for role in [Role::Region, Role::Heading { level: 2 }, Role::Tree] {
            assert_eq!(named(view.tree().nodes(), role, b"Plan"), plan);
        }
        assert!(!named(view.tree().nodes(), Role::Heading { level: 2 }, b"Its tasks"));
    }
}
#[test]
fn an_agents_page_carries_its_tokens_on_their_roles() {
    let live = agent(f::delegate());
    assert_eq!(fixtures::styles(live.tree(), Role::Main).as_ref(), [Box::from([look::MAIN, look::AGENT_PAGE])]);
    for (token, role) in
        [(look::AGENT_LINEAGE, Role::Slot), (look::AGENT_BRIEF, Role::Region), (look::AGENT_RUN, Role::Slot)]
    {
        assert_eq!(live.tree().nodes()[styled(live.tree(), token)].role, role);
    }
    let mut task = f::delegate();
    task.result =
        Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(c::ResultValue::Report(f::document_words(b"Done"))))));
    let ended = agent(task);
    assert_eq!(ended.tree().nodes()[styled(ended.tree(), look::AGENT_RESULT)].role, Role::Region);
}

use temper_client_domain::{Action, Domain, Event, pages::Act};
fn offers(task: &mut c::Task, values: &[(c::ActKind, c::Admission)]) {
    task.acts = List::with_capacity(u32::try_from(values.len()).expect("fixture offers"));
    for (act, admission) in values {
        task.acts.push(c::Offered { act: *act, admission: admission.clone(), effect: None }).expect("offer");
    }
}
fn press(root: &mut Domain, act: Act) {
    fixtures::root_event(root, Event::Act { action: Action::Pages(act) });
}
fn press_button(root: &mut Domain, name: &[u8]) {
    let mut view = fixtures::page_view(root);
    let id = fixtures::find(view.tree(), Role::Button, name).expect("offered button");
    let action = crate::act(&mut view, id, skein_ui::Act::Press).expect("button binding");
    fixtures::root_event(root, Event::Act { action });
}

fn contains(tree: &Tree, expected: &[u8]) -> bool {
    skein_lib::bytes::find(&fixtures::text(tree), expected).is_some()
}

#[test]
fn task_acts_show_their_name_and_admission_or_nothing_when_absent() {
    for admission in [
        None,
        Some(c::Admission::Refused(c::Refusal::NotOffered)),
        Some(c::Admission::Proposed { to: c::Recipient::Party(PARTY) }),
    ] {
        let mut task = f::document();
        offers(&mut task, &[]);
        if let Some(admission) = &admission {
            offers(
                &mut task,
                &[
                    (c::ActKind::Amend, admission.clone()),
                    (c::ActKind::Stop, admission.clone()),
                    (c::ActKind::Cancel, admission.clone()),
                ],
            );
        }
        let view = agent(task);
        for name in [b"Amend".as_slice(), b"Pause", b"Close"] {
            assert!(fixtures::find(view.tree(), Role::Button, name).is_none());
            assert_eq!(fixtures::find(view.tree(), Role::Paragraph, name).is_some(), admission.is_some());
        }
        match admission {
            Some(c::Admission::Refused(_)) => assert!(contains(view.tree(), b"This act is not offered")),
            Some(c::Admission::Proposed { .. }) => assert!(contains(view.tree(), b"This needs a proposal to")),
            None => assert!(!has_style(view.tree().nodes(), look::TASK_ACTS)),
            _ => unreachable!("fixture admissions"),
        }
    }
}

#[test]
fn a_send_asks_for_the_agents_composer() {
    let mut task = f::delegate();
    offers(&mut task, &[(c::ActKind::Message, c::Admission::Allowed)]);
    let mut root = f::agent_root(task, Some(f::tree()), true);
    for count in 0..=2 {
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let composer = &view.tree().nodes()[index_of(view.tree(), Role::TextArea, b"Message")];
            assert_eq!(composer.focus, if count == 0 { None } else { Some(count) });
        }
        fixtures::root_event(&mut root, Event::Act { action: Action::Pages(Act::Send) });
    }
}

#[test]
fn an_agents_taskbar_has_pause_and_its_composer_has_the_steering_note() {
    let mut task = f::delegate();
    offers(
        &mut task,
        &[
            (c::ActKind::Stop, c::Admission::Allowed),
            (c::ActKind::Cancel, c::Admission::Allowed),
            (c::ActKind::Message, c::Admission::Allowed),
        ],
    );
    let mut root = f::agent_root(task, Some(f::tree()), true);
    let mut view = fixtures::page_view(&root);
    let tree = view.tree();
    let bar = styled(tree, look::TASKBAR);
    assert!(named(subtree(tree, bar), Role::Button, b"Pause"));
    let composer = styled(tree, look::COMPOSER_AREA);
    let steer = styled(tree, look::AGENT_STEER);
    assert_eq!(tree.nodes()[steer].role, Role::Paragraph);
    assert!(contains(tree, "Steer this task · words reach its live run at the next turn.".as_bytes()));
    let textarea = fixtures::find(tree, Role::TextArea, b"Message").expect("message");
    assert!(named(subtree(tree, composer), Role::Button, b"Send"));
    assert!(index_of(tree, Role::Log, b"Conversation") < composer);
    let action = crate::act(&mut view, textarea, skein_ui::Act::Input { value: Box::from(*b"Steer") });
    assert_eq!(action, Some(Action::Pages(Act::MessageWords { words: Box::from(*b"Steer") })));
    press_button(&mut root, b"Close");
    let view = fixtures::page_view(&root);
    assert!(!named(subtree(view.tree(), styled(view.tree(), look::TASKBAR)), Role::Button, b"Pause"));
}
#[test]
fn an_agents_refused_or_proposed_message_says_why_in_the_composers_place() {
    for admission in [
        c::Admission::Refused(c::Refusal::NotWaiting),
        c::Admission::Refused(c::Refusal::NoFurther),
        c::Admission::Proposed { to: c::Recipient::Party(PARTY) },
    ] {
        let mut task = f::delegate();
        offers(&mut task, &[(c::ActKind::Message, admission.clone())]);
        let view = agent(task);
        let tree = view.tree();
        let composer = styled(tree, look::COMPOSER_AREA);
        assert!(named(subtree(tree, composer), Role::Paragraph, b"Message"));
        assert!(fixtures::find(tree, Role::TextArea, b"Message").is_none());
        assert!(fixtures::find(tree, Role::Button, b"Send").is_none());
        match admission {
            c::Admission::Proposed { .. } => {
                assert!(!has_style(tree.nodes(), look::AGENT_STEER));
                assert!(contains(tree, b"This needs a proposal to"));
            }
            // Why Message isn't offered is a note in the steer note's look.
            c::Admission::Refused(c::Refusal::NotWaiting) => {
                let note = styled(tree, look::AGENT_STEER);
                assert!(composer < note && note <= composer + usize::try_from(tree.nodes()[composer].size).unwrap());
                assert_eq!(&*fixtures::node_text(tree, note), b"Messages reach it once it runs.");
                let refused =
                    fixtures::words().say_word(jig_client::words::Word::refusal(&c::Refusal::NotWaiting), &[]);
                assert!(!contains(tree, &refused));
            }
            c::Admission::Refused(refusal) => {
                let note = styled(tree, look::AGENT_STEER);
                assert!(composer < note && note <= composer + usize::try_from(tree.nodes()[composer].size).unwrap());
                let expected = fixtures::words().say_word(jig_client::words::Word::refusal(&refusal), &[]);
                assert_eq!(&*fixtures::node_text(tree, note), &*expected);
            }
            c::Admission::Allowed | c::Admission::Bounded { .. } => unreachable!(),
        }
    }
}
#[test]
fn the_own_held_card_follows_the_agents_run_heading() {
    let source = held_agent_root();
    let mut task = source.pages().task().expect("page").document(source.client()).expect("task").clone();
    let release = task.acts.as_slice()[0].clone();
    offers(&mut task, &[(c::ActKind::Cancel, c::Admission::Allowed)]);
    task.acts = {
        let mut acts = List::with_capacity(2);
        acts.push(release).unwrap();
        acts.push(task.acts.as_slice()[0].clone()).unwrap();
        acts
    };
    task.priority = None;
    task.activity = 1;
    let root = f::agent_root(task, Some(tree_delivery()), true);
    let view = fixtures::page_view(&root);
    let tree = view.tree();
    let hold = styled(tree, look::PLAN_HOLD);
    let preceding = styled(tree, look::AGENT_RUN);
    assert!(preceding < hold);
    assert!(hold < styled(tree, look::PLAN));
    assert!(named(subtree(tree, hold), Role::Button, b"Release task"));
}
fn tree_delivery() -> c::Delivery {
    let mut delivery = f::tree();
    let c::Delivery::Tree(tree) = &mut delivery else { panic!("tree") };
    tree.root.phase = c::Phase::Held;
    delivery
}
#[test]
fn an_agents_log_reads_at_its_head_and_foot() {
    let mut task = f::delegate();
    task.activity = 2;
    let mut page = f::activity(true);
    let mut second = page.rows.as_slice()[0].clone();
    second.source.position = 2;
    page.rows = {
        let first = page.rows.as_slice()[0].clone();
        let mut rows = List::with_capacity(2);
        rows.push(first).unwrap();
        rows.push(second).unwrap();
        rows
    };
    page.before = Some(c::Cursor::new(Box::from(*b"older"), 32).expect("cursor"));
    let mut root = f::agent_root_with_page(task, Some(f::tree()), page);
    let mut view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Button, b"Older").is_none());
    let id = fixtures::find(view.tree(), Role::Log, b"Conversation").expect("log");
    assert_eq!(view.tree().find(id).unwrap().end_request, None);
    assert_eq!(
        crate::act(&mut view, id, skein_ui::Act::Near { edge: skein_ui::LogEdge::Foot }),
        Some(Action::Pages(Act::Newer))
    );
    let action =
        crate::act(&mut view, id, skein_ui::Act::Near { edge: skein_ui::LogEdge::Head }).expect("older action");
    assert_eq!(action, Action::Pages(Act::Older));
    fixtures::root_event(&mut root, Event::Act { action });
    match root.pages().task().expect("page").thread.reading {
        Some(jig_client::thread::Reading::Older) => {}
        Some(jig_client::thread::Reading::Latest { .. } | jig_client::thread::Reading::Newer) | None => {
            panic!("older read pending")
        }
    }
    let view = fixtures::browser_page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Button, b"Older").is_none());
    let status = fixtures::find(view.tree(), Role::Status, b"Loading earlier messages").expect("loading status");
    assert!(view.tree().find(status).unwrap().styles.as_slice().contains(&look::THREAD_LOADING));
    assert!(fixtures::find(view.tree(), Role::Button, b"Latest").is_some());
}

fn sending_task_act(root: &mut Domain, act: Act) -> skein_lib::Token {
    let mut out = skein_lib::Queue::uncapped();
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: fixtures::domain_limits() },
        Event::Act { action: Action::Pages(act) },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            return attempt;
        }
    }
    panic!("task act sent");
}
fn paused_agent_document() -> c::Task {
    let mut task = super::goal::paused_document();
    task.priority = None;
    task.activity = 1;
    task
}
#[test]
fn task_pause_and_resume_show_progress_then_only_refusals() {
    for act in [Act::Pause, Act::Resume] {
        for refused in [false, true] {
            let mut task = if act == Act::Resume { paused_agent_document() } else { f::delegate() };
            offers(
                &mut task,
                &[(c::ActKind::Stop, c::Admission::Allowed), (c::ActKind::Release, c::Admission::Allowed)],
            );
            let mut root = f::agent_root(task, Some(f::tree()), true);
            let attempt = sending_task_act(&mut root, act.clone());
            assert!(root.pages().confirmation().is_none());
            let page = root.pages().task().expect("page");
            let progress = root.client().pending(page.pending.as_slice()[0].0).expect("pending").progress;
            let mut builder = Builder::new(&fixtures::tree_limits());
            crate::confirm::progress_text(&mut builder, &fixtures::words(), progress);
            let expected_progress = fixtures::text(&builder.finish());
            for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                assert!(contains(view.tree(), &expected_progress));
            }
            let answer = if refused {
                c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None }
            } else {
                c::Answer::Done(c::Outcome::Document(if act == Act::Pause {
                    c::DocumentOutcome::Stopped { task: 9 }
                } else {
                    c::DocumentOutcome::Released { task: 9 }
                }))
            };
            fixtures::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Answer {
                            attempt,
                            envelope: skein_lib::RequestEnvelope::Final,
                            answer: Some(jig_client::Answered::Jig(answer)),
                            why: None,
                        },
                    },
                },
            );
            for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                assert!(!contains(view.tree(), b"Applied"));
                assert_eq!(contains(view.tree(), b"This act is not offered"), refused);
            }
            assert!(root.pages().task().expect("page").pending.is_empty());
        }
    }
}
#[test]
fn an_agents_own_pause_has_resume_and_close_without_its_held_card() {
    let mut task = paused_agent_document();
    offers(
        &mut task,
        &[
            (c::ActKind::Amend, c::Admission::Allowed),
            (c::ActKind::Release, c::Admission::Allowed),
            (c::ActKind::Cancel, c::Admission::Allowed),
        ],
    );
    let root = f::supplied_root(task, true);
    assert!(!root.pages().task().expect("page").cards.is_empty());
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let tree = view.tree();
        assert!(contains(tree, b"Paused"));
        for name in [b"Amend".as_slice(), b"Resume", b"Close"] {
            assert!(named(subtree(tree, styled(tree, look::TASKBAR)), Role::Button, name));
        }
        assert!(!named(tree.nodes(), Role::Button, b"Release task"));
        assert!(!named(tree.nodes(), Role::Button, b"Pause"));
    }
}

#[test]
fn a_held_procedure_goal_shows_its_own_release_before_its_plan() {
    let source = held_agent_root();
    let mut task = source.pages().task().expect("page").document(source.client()).expect("task").clone();
    task.chip.executor = c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE };
    task.priority = None;
    let root = f::supplied_root(task, true);
    let view = fixtures::page_view(&root);
    let tree = view.tree();
    let release = index_of(tree, Role::Button, b"Release task");
    let plan = region_index(tree, b"Its tasks");
    assert!(region_index(tree, b"Task details") < release);
    assert!(release < plan);
}

fn send_amendment(root: &mut Domain) -> skein_lib::Token {
    let limits = fixtures::domain_limits();
    let mut out = skein_lib::Queue::uncapped();
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits },
        Event::Act { action: Action::Pages(Act::SendAmendment) },
        &mut out,
    );
    let mut attempt = None;
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: token, .. } },
        } = request
        {
            attempt = Some(token);
        }
    }
    attempt.expect("amendment sent")
}
fn amend_root(cut: bool) -> Domain {
    let mut task = f::delegate();
    task.spec.specification.words.cut = cut;
    offers(
        &mut task,
        &[
            (c::ActKind::Amend, c::Admission::Allowed),
            (c::ActKind::Stop, c::Admission::Allowed),
            (c::ActKind::Cancel, c::Admission::Allowed),
        ],
    );
    f::agent_root(task, Some(f::tree()), true)
}
#[test]
fn amend_is_first_in_the_agents_taskbar() {
    let mut root = amend_root(false);
    let view = fixtures::page_view(&root);
    let tree = view.tree();
    assert!(index_of(tree, Role::Button, b"Amend") < index_of(tree, Role::Button, b"Pause"));
    press_button(&mut root, b"Close");
    assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Button, b"Amend").is_none());
}
#[test]
fn amendment_has_specification_controls_and_discard() {
    let mut root = amend_root(false);
    press_button(&mut root, b"Amend");
    let mut view = fixtures::page_view(&root);
    let tree = view.tree();
    let region = styled(tree, look::TASK_AMEND);
    assert_eq!(tree.nodes()[region].role, Role::Region);
    assert_eq!(tree.nodes()[region].name.as_deref(), Some(b"Amend task 9".as_slice()));
    assert!(named(subtree(tree, region), Role::Heading { level: 2 }, b"Amend task 9"));
    assert!(named(subtree(tree, region), Role::TextArea, b"Specification"));
    // Its label is drawn by the view, just before the text area, not by the shell.
    let label = index_of(tree, Role::Paragraph, b"Specification");
    assert!(region < label && label < index_of(tree, Role::TextArea, b"Specification"));
    assert!(!named(subtree(tree, region), Role::Check, b"Tracked"));
    assert!(!named(subtree(tree, region), Role::TextField, b"Priority"));
    assert!(named(subtree(tree, region), Role::Button, b"Send amendment"));
    assert!(!named(tree.nodes(), Role::Button, b"Amend"));
    let field = fixtures::find(tree, Role::TextArea, b"Specification").unwrap();
    assert_eq!(
        crate::act(&mut view, field, skein_ui::Act::Input { value: Box::from(*b"New spec") }),
        Some(Action::Pages(Act::AmendWords { words: Box::from(*b"New spec") }))
    );

    press_button(&mut root, b"Discard");
    assert!(!has_style(fixtures::page_view(&root).tree().nodes(), look::TASK_AMEND));
}
#[test]
fn a_cut_specification_has_an_explanation_and_no_textarea() {
    let mut root = amend_root(true);
    press_button(&mut root, b"Amend");
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::TextArea, b"Specification").is_none());
    assert!(contains(
        view.tree(),
        b"This task's specification is too long to edit here: use `temper ctl DIR task amend TASK --rev REV --spec FILE REASON`"
    ));
}
#[test]
fn no_change_is_shown_until_an_amendment_is_edited() {
    let mut root = amend_root(false);
    press_button(&mut root, b"Amend");
    press_button(&mut root, b"Send amendment");
    assert!(contains(fixtures::page_view(&root).tree(), b"No change"));
    press(&mut root, Act::AmendWords { words: Box::from(*b"New spec") });
    assert!(!contains(fixtures::page_view(&root).tree(), b"No change"));
}
#[test]
fn a_moved_amendment_hides_send_and_its_refusal_offers_look_again() {
    let mut task = f::document();
    task.chip.task = TASK;
    task.chip.executor = c::ChipExecutor::Agent { charter: CHARTER };
    task.priority = None;
    offers(&mut task, &[(c::ActKind::Amend, c::Admission::Allowed)]);
    let (mut root, watch) = f::supplied_root_watching(task, false);
    press_button(&mut root, b"Amend");
    press(&mut root, Act::AmendWords { words: Box::from(*b"New spec") });
    let attempt = send_amendment(&mut root);
    let page = root.pages().task().unwrap();
    let mut task = page.document(root.client()).unwrap().clone();
    task.spec_revision.value += 1;
    let revision = task.spec_revision;
    // Deliver the changed document on the existing watch, as the engine does.
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    let view = fixtures::page_view(&root);
    assert!(contains(view.tree(), b"This task changed since the editor opened"));
    assert!(fixtures::find(view.tree(), Role::Button, b"Send amendment").is_none());
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                        refusal: c::Refusal::Moved { now: revision, what: c::Moved::Spec },
                        at: None,
                    })),
                    why: None,
                },
            },
        },
    );
    press_button(&mut root, b"Look again");
    assert_eq!(root.pages().task().unwrap().amending.as_ref().unwrap().draft.revision(), revision);
    assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Button, b"Send amendment").is_some());
}

#[test]
fn amendment_progress_and_done_stay_in_its_region_until_amend_again() {
    let mut root = amend_root(false);
    press_button(&mut root, b"Amend");
    press(&mut root, Act::AmendWords { words: Box::from(*b"New spec") });
    let attempt = send_amendment(&mut root);
    let page = root.pages().task().expect("task page");
    let key = page.pending.as_slice()[0].0;
    let progress = root.client().pending(key).expect("pending amendment").progress;
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::confirm::progress_text(&mut builder, &fixtures::words(), progress);
    let expected = fixtures::text(&builder.finish());
    let view = fixtures::page_view(&root);
    let region = styled(view.tree(), look::TASK_AMEND);
    let progress_node = index_of(view.tree(), Role::Status, &expected);
    assert!(
        region < progress_node && progress_node < region + usize::try_from(view.tree().nodes()[region].size).unwrap()
    );
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                        c::DocumentOutcome::Amended { task: 9 },
                    )))),
                    why: None,
                },
            },
        },
    );
    let view = fixtures::page_view(&root);
    let region = styled(view.tree(), look::TASK_AMEND);
    let nodes = subtree(view.tree(), region);
    assert!(named(nodes, Role::Status, b"Applied"));
    assert!(!named(nodes, Role::TextArea, b"Specification"));
    assert!(!named(nodes, Role::Check, b"Tracked"));
    assert!(!named(nodes, Role::TextField, b"Priority"));
    assert!(!named(nodes, Role::Button, b"Send amendment"));
    assert!(!named(nodes, Role::Button, b"Discard"));
    press_button(&mut root, b"Amend");
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::TextArea, b"Specification").is_some());
    assert!(!contains(view.tree(), b"Applied"));
}

const ENDED_CHILD: u64 = 23;
fn ended_line(number: u64, status: c::TaskStatus) -> c::TaskLine {
    let mut line = super::chats::line();
    line.chip = row().chip;
    line.chip.task = number;
    line.chip.phase = c::Phase::Ended;
    let c::ChipPhaseFacts { phase, details } = line.chip.details.as_mut();
    *phase = c::ChipPhase::Ended;
    details.ending = Some(match status {
        c::TaskStatus::Done => c::ChipEnding::Done(c::ResultValue::Report(f::document_words(b"Done"))),
        c::TaskStatus::Failed => c::ChipEnding::Failed(f::document_words(b"Failed")),
        c::TaskStatus::Cancelled => c::ChipEnding::Cancelled {
            reason: f::document_words(b"Cancelled"),
            result: None,
            by: c::Requester::Person(PARTY),
            at: skein_lib::Wall::EPOCH,
        },
    });
    line
}
fn tasks_page(lines: &[c::TaskLine], after: bool, cut: bool) -> c::TaskPage {
    let mut rows = List::with_capacity(u32::try_from(lines.len()).unwrap());
    for line in lines {
        rows.push(line.clone()).expect("ended rows");
    }
    c::TaskPage {
        live: List::with_capacity(0),
        live_after: None,
        ended: c::EndedGroup::Rows {
            rows,
            after: if after { Some(c::Cursor::new(Box::from(*b"next"), 8).unwrap()) } else { None },
            cut,
        },
    }
}
fn procedure_document(phase: c::Phase) -> c::Task {
    let mut task = f::document();
    task.chip.task = TASK;
    task.chip.executor = c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE };
    task.chip.phase = phase;
    task.priority = None;
    task
}
fn its_tasks_tree(live: &[c::TreeRow], phase: c::Phase, page: Option<c::TaskPage>) -> Tree {
    let mut delivery = f::tree();
    let c::Delivery::Tree(tree) = &mut delivery else { panic!("owner tree") };
    tree.rows = List::with_capacity(u32::try_from(live.len()).unwrap());
    for row in live {
        tree.rows.push(row.clone()).expect("live row");
    }
    its_tasks_delivery(delivery, phase, page)
}
fn its_tasks_delivery(delivery: c::Delivery, phase: c::Phase, page: Option<c::TaskPage>) -> Tree {
    let root = f::root_with_ended(procedure_document(phase), delivery, page);
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::pages::task::main(
        &mut builder,
        &root,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        fixtures::domain_limits().client.domain.rows,
        &mut crate::words::Times::new(fixtures::NOW),
        &crate::forge_sign_in::Forges::new(),
    );
    builder.finish()
}
#[test]
fn its_ended_tasks_keep_their_endings_spend_and_muted_links() {
    let lines = [
        ended_line(ENDED_CHILD, c::TaskStatus::Done),
        ended_line(24, c::TaskStatus::Failed),
        ended_line(25, c::TaskStatus::Cancelled),
    ];
    let tree = its_tasks_tree(&[row()], c::Phase::Running, Some(tasks_page(&lines, false, false)));
    let live = index_of(&tree, Role::TreeItem, b"Task 12");
    let mut previous = live;
    for (number, status) in
        [(b"Task 23".as_slice(), b"Completed".as_slice()), (b"Task 24", b"Failed"), (b"Task 25", b"Cancelled")]
    {
        let index = index_of(&tree, Role::TreeItem, number);
        assert!(index > previous);
        assert!(named(subtree(&tree, index), Role::Status, status));
        assert!(named(subtree(&tree, index), Role::Link, number));
        assert!(index >= live + usize::try_from(tree.nodes()[live].size).unwrap(), "ended row is top level");
        assert!(has_style(subtree(&tree, index), look::PLAN_ROW));
        assert!(tree.nodes()[index].styles.as_slice().contains(&look::PLAN_DONE));
        assert!(named(subtree(&tree, index), Role::Paragraph, b"5 spent"));
        assert!(!has_style(subtree(&tree, index), look::DEPENDENCIES));
        previous = index;
    }
    let region = subtree(&tree, region_index(&tree, b"Its tasks"));
    assert!(!named(region, Role::Paragraph, b"No live tasks."));
    assert!(!named(region, Role::Paragraph, b"None yet."));
}
#[test]
fn its_tasks_says_no_live_tasks_until_the_ended_read_answers() {
    let tree = its_tasks_tree(&[], c::Phase::Running, None);
    let region = subtree(&tree, region_index(&tree, b"Its tasks"));
    assert!(named(region, Role::Paragraph, b"No live tasks."));
    assert!(!named(region, Role::Paragraph, b"None yet."));
    assert!(!named(region, Role::Paragraph, b"None."));
}
#[test]
fn its_empty_tasks_says_none_yet_while_it_lives_and_none_once_it_ends() {
    for (phase, expected) in [
        (c::Phase::Running, b"None yet.".as_slice()),
        (c::Phase::Ended, b"None."),
        (c::Phase::Done, b"None."),
        (c::Phase::Failed, b"None."),
        (c::Phase::Cancelled, b"None."),
    ] {
        let tree = its_tasks_tree(&[], phase, Some(tasks_page(&[], false, false)));
        let region = subtree(&tree, region_index(&tree, b"Its tasks"));
        assert!(named(region, Role::Paragraph, expected));
        assert!(!named(region, Role::Paragraph, b"No live tasks."));
    }
}
#[test]
fn its_tasks_without_a_tree_draws_its_answered_ended_page() {
    for (phase, empty_word) in [(c::Phase::Ended, b"None.".as_slice()), (c::Phase::Running, b"None yet.")] {
        for empty in [false, true] {
            let lines = [ended_line(ENDED_CHILD, c::TaskStatus::Done)];
            let page = tasks_page(if empty { &[] } else { &lines }, false, false);
            let root = f::root_with_optional_tree_and_ended(procedure_document(phase), None, Some(page));
            let view = fixtures::page_view(&root);
            let tree = view.tree();
            let region = subtree(tree, region_index(tree, b"Its tasks"));
            assert!(named(region, Role::Heading { level: 2 }, b"Its tasks"));
            assert_eq!(named(region, Role::Paragraph, empty_word), empty);
            assert!(!named(region, Role::Paragraph, b"No live tasks."));
            assert_eq!(named(region, Role::TreeItem, b"Task 23"), !empty);
            assert_eq!(named(region, Role::Link, b"Task 23"), !empty);
            if !empty {
                let child = subtree(tree, index_of(tree, Role::TreeItem, b"Task 23"));
                assert!(named(child, Role::Status, b"Completed"));
                assert!(has_style(child, look::PLAN_ROW));
            }
        }
    }
}
#[test]
fn its_tasks_with_only_folds_or_held_below_does_not_say_none() {
    for phase in [c::Phase::Running, c::Phase::Ended] {
        for (running, held_below, has_fold) in [(ONE, NO_ROWS, true), (NO_ROWS, ONE, false), (NO_ROWS, NO_ROWS, false)]
        {
            let mut owner = f::tree();
            let c::Delivery::Tree(tree) = &mut owner else { panic!("owner tree") };
            tree.rows = List::with_capacity(NO_ROWS);
            tree.folded = List::with_capacity(ONE);
            tree.folded
                .push(c::TreeFold {
                    parent: TASK,
                    phases: c::PhaseCounts { running, ..c::PhaseCounts::ZERO },
                    held_below,
                })
                .expect("root fold");
            let mut owner_held = tree.clone();
            owner_held.folded = List::with_capacity(NO_ROWS);
            owner_held.held_below = held_below;
            for (delivery, has_fold) in [(owner, has_fold), (c::Delivery::Tree(owner_held), false)] {
                let tree = its_tasks_delivery(delivery, phase, Some(tasks_page(&[], false, false)));
                let region = subtree(&tree, region_index(&tree, b"Its tasks"));
                let empty = !has_fold && held_below == 0;
                assert_eq!(named(region, Role::Paragraph, b"Folded: 1 Running"), has_fold);
                assert_eq!(named(region, Role::Status, b"Held below: 1"), held_below != 0);
                assert_eq!(named(region, Role::Paragraph, b"None yet."), empty && phase == c::Phase::Running);
                assert_eq!(named(region, Role::Paragraph, b"None."), empty && phase == c::Phase::Ended);
                assert!(!named(region, Role::Paragraph, b"No live tasks."));
            }
        }
    }
}
#[test]
fn its_tasks_reports_more_ended_tasks_for_a_cursor_or_a_cut_page() {
    for (after, cut) in [(false, false), (true, false), (false, true)] {
        let tree = its_tasks_tree(
            &[],
            c::Phase::Running,
            Some(tasks_page(&[ended_line(ENDED_CHILD, c::TaskStatus::Done)], after, cut)),
        );
        let region = region_index(&tree, b"Its tasks");
        let more = b"More of its ended tasks are not shown.";
        assert_eq!(named(subtree(&tree, region), Role::Paragraph, more), after || cut);
        if after || cut {
            let line = index_of(&tree, Role::Paragraph, more);
            let tree_index = index_of(&tree, Role::Tree, b"Its tasks");
            assert!(line >= tree_index + usize::try_from(tree.nodes()[tree_index].size).unwrap());
        }
    }
}
#[test]
fn an_ended_task_still_in_its_live_tree_is_drawn_once_in_that_tree() {
    let mut parent = row();
    parent.chip.task = GRANDCHILD;
    let mut child = row();
    child.parent = GRANDCHILD;
    child.depth = 2;
    let tree = its_tasks_tree(
        &[parent, child],
        c::Phase::Running,
        Some(tasks_page(&[ended_line(CHILD, c::TaskStatus::Done)], false, false)),
    );
    let mut count = NO_ROWS;
    for node in tree.nodes() {
        if node.role == Role::TreeItem && node.name.as_deref() == Some(b"Task 12") {
            count += ONE;
        }
    }
    assert_eq!(count, ONE);
    let parent = index_of(&tree, Role::TreeItem, b"Task 13");
    let child = index_of(&tree, Role::TreeItem, b"Task 12");
    assert!(child > parent && child < parent + usize::try_from(tree.nodes()[parent].size).unwrap());
}

/// The tree's items by name, in order and none else.
fn assert_tree_items(tree: &Tree, expected: &[&[u8]]) {
    let mut at = 0;
    for node in tree.nodes() {
        if node.role == Role::TreeItem {
            assert_eq!(node.name.as_deref(), expected.get(at).copied(), "tree item {at}");
            at = at.checked_add(1).expect("bounded items");
        }
    }
    assert_eq!(at, expected.len(), "tree items");
}
#[test]
fn ended_rows_merge_among_roots_and_leave_each_roots_delegates_under_it() {
    let mut first = row();
    first.chip.task = 12;
    let mut child = row();
    child.chip.task = 30;
    child.parent = 12;
    child.depth = 2;
    let mut last = row();
    last.chip.task = 18;
    let live = [first, child, last];
    let lines = [10, 12, 15, 18, 20, 30].map(|number| ended_line(number, c::TaskStatus::Done));
    let page = tasks_page(&lines, false, false);
    let procedure = its_tasks_tree(&live, c::Phase::Running, Some(page));
    let tree = &procedure;
    assert_tree_items(tree, &[b"Task 10", b"Task 12", b"Task 30", b"Task 15", b"Task 18", b"Task 20"]);
    let parent = index_of(tree, Role::TreeItem, b"Task 12");
    let child = index_of(tree, Role::TreeItem, b"Task 30");
    assert!(child < parent + usize::try_from(tree.nodes()[parent].size).unwrap());
    let between = index_of(tree, Role::TreeItem, b"Task 15");
    assert!(between >= parent + usize::try_from(tree.nodes()[parent].size).unwrap());
    assert!(!has_style(subtree(tree, index_of(tree, Role::TreeItem, b"Task 12")), look::PLAN_DONE));
    let ended = index_of(tree, Role::TreeItem, b"Task 10");
    assert!(tree.nodes()[ended].styles.as_slice().contains(&look::PLAN_DONE));
    assert!(named(subtree(tree, ended), Role::Status, b"Completed"));
    assert!(named(subtree(tree, ended), Role::Paragraph, b"5 spent"));
    assert!(!has_style(subtree(tree, ended), look::DEPENDENCIES));
}

#[test]
fn a_procedures_plan_keeps_rows_folds_held_counts_and_held_cards() {
    let source = held_agent_root();
    let mut held = source.pages().task().unwrap().document(source.client()).unwrap().clone();
    held.chip.task = CHILD;
    held.requester = c::Requester::Task(TASK);
    let mut live = row();
    live.chip = held.chip.clone();
    let mut delivery = f::tree();
    let c::Delivery::Tree(tree) = &mut delivery else { panic!("owner") };
    tree.rows = List::with_capacity(ONE);
    tree.rows.push(live).unwrap();
    tree.folded = List::with_capacity(ONE);
    tree.folded
        .push(c::TreeFold {
            parent: TASK,
            phases: c::PhaseCounts { running: 1, ..c::PhaseCounts::ZERO },
            held_below: 0,
        })
        .unwrap();
    tree.held_below = 1;
    let mut task = f::document();
    task.priority = None;
    task.chip.executor = c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE };
    let page = tasks_page(&[ended_line(23, c::TaskStatus::Done)], true, false);
    let root = f::root_with_ended_and_held(task, Some(delivery), Some(page), Some(held));
    let view = fixtures::page_view(&root);
    let nodes = subtree(view.tree(), region_index(view.tree(), b"Its tasks"));
    assert!(named(nodes, Role::TreeItem, b"Task 12"));
    assert!(named(nodes, Role::Paragraph, b"Folded: 1 Running"));
    assert!(named(nodes, Role::Status, b"Held below: 1"));
    assert!(has_style(nodes, look::PLAN_HOLD));
    assert!(named(nodes, Role::Button, b"Release task"));
    assert!(!named(nodes, Role::Paragraph, b"No live tasks."));
    assert!(named(nodes, Role::TreeItem, b"Task 23"));
    assert!(named(nodes, Role::Paragraph, b"More of its ended tasks are not shown."));
}

fn unread_task() -> (Domain, skein_lib::Token) {
    let (root, _, watch) =
        fixtures::root_at(temper_client_domain::Address::Task { task: TASK }, &fixtures::domain_limits());
    (root, watch.expect("task watch"))
}
fn end_task_read(root: &mut Domain, watch: skein_lib::Token, result: jig_client::Readout) {
    use jig_client::{Down, Readable, Up};
    use skein_lib::{Env, Queue, Time};
    let limits = fixtures::domain_limits();
    let env = Env { now: Time::ZERO, wall: fixtures::NOW, limits };
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        root,
        &env,
        Event::Client { event: jig_client::Event::Up { up: Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded } } },
        &mut out,
    );
    let mut token = None;
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, what } },
        } = request
            && what == Readable::Jig(c::Read::Task { task: TASK })
        {
            token = Some(read);
        }
    }
    fixtures::root_event(
        root,
        Event::Client { event: jig_client::Event::Up { up: Up::ReadDone { read: token.expect("task read"), result } } },
    );
}
#[test]
fn a_task_without_its_document_says_it_is_reading() {
    let (root, _) = unread_task();
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, &root);
        assert!(fixtures::find(tree, Role::Status, "Reading task 9…".as_bytes()).is_some());
        assert!(skein_lib::bytes::find(&fixtures::text(tree), "Reading task 9…".as_bytes()).is_some());
    }
}
#[test]
fn a_task_waiting_to_read_again_keeps_its_reading_status() {
    let (mut root, watch) = unread_task();
    end_task_read(&mut root, watch, jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)));
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Status, "Reading task 9…".as_bytes()).is_some());
}
#[test]
fn lasting_task_read_ends_say_why() {
    use jig_client::Readout;
    for (result, text) in [
        (Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)), b"This task could not be found.".as_slice()),
        (Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)), b"You are not allowed to read this task.".as_slice()),
        (Readout::Jig(c::ReadResult::None(c::ReadNone::Gone)), b"This task is no longer available.".as_slice()),
        (
            Readout::Jig(c::ReadResult::None(c::ReadNone::TooLarge { bytes: 1 })),
            b"This task is too large to read.".as_slice(),
        ),
        (Readout::Jig(c::ReadResult::None(c::ReadNone::Malformed)), b"This task could not be read.".as_slice()),
        (
            Readout::NotServed { family: None, served: None },
            b"This version of temper cannot read this task.".as_slice(),
        ),
    ] {
        let (mut root, watch) = unread_task();
        end_task_read(&mut root, watch, result);
        for words in [fixtures::words(), fixtures::browser_words()] {
            let mut view = fixtures::view_of(words);
            let tree = fixtures::build(&mut view, &root);
            assert!(fixtures::find(tree, Role::Status, text).is_some());
            assert!(skein_lib::bytes::find(&fixtures::text(tree), text).is_some());
        }
    }
}
#[test]
fn a_task_read_ended_at_sign_out_says_to_sign_in() {
    let (mut root, _) = unread_task();
    fixtures::root_event(&mut root, Event::SignedOut);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Status, b"Sign in to read this task.").is_some());
}
#[test]
fn a_task_document_replaces_its_reading_status() {
    let (mut root, watch) = unread_task();
    let mut document = f::document();
    document.chip.task = TASK;
    document.priority = None;
    document.chip.executor = c::ChipExecutor::Party { party: PARTY };
    end_task_read(
        &mut root,
        watch,
        jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(document)))),
    );
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Status, "Reading task 9…".as_bytes()).is_none());
    assert!(fixtures::find(tree, Role::Heading { level: 2 }, b"Task details").is_some());
}

#[test]
fn a_too_linked_task_page_says_so() {
    let (mut root, watch) = unread_task();
    end_task_read(
        &mut root,
        watch,
        jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::TooLinked { joins: READ_JOINS })),
    );
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, &root);
        assert!(
            fixtures::find(tree, Role::Status, b"Too linked to show: it needs more than 4 joins in one read.")
                .is_some()
        );
    }
}

/// A held agent task keeps the cards shared by the other task pages (5.11).
pub(super) fn held_agent_root() -> Domain {
    let root = f::held_root();
    let mut task = root.pages().task().expect("task page").document(root.client()).expect("task").clone();
    task.priority = None;
    f::supplied_root(task, true)
}

#[test]
fn a_tracked_task_hands_over_to_the_goals_page() {
    let root = f::root_with(f::document());
    let view = fixtures::page_view(&root);
    assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([look::MAIN, look::GOAL])]);
    for name in [b"Report".as_slice(), b"Tasks", b"Landing", b"How it runs"] {
        assert!(fixtures::find(view.tree(), Role::Region, name).is_some());
    }
    for name in [b"Task details".as_slice(), b"What was asked", b"Plan", b"Latest activity", b"Budget and authority"] {
        assert!(fixtures::find(view.tree(), Role::Region, name).is_none());
    }
}

#[test]
fn a_goals_terminal_delegate_line_separates_title_and_state() {
    let mut task = f::document();
    task.chip.task = TASK;
    task.priority = Some(PRIORITY);
    let mut delivery = f::tree();
    let c::Delivery::Tree(tree) = &mut delivery else { panic!("tree") };
    let mut delegate = row();
    delegate.chip.title = document_text(b"Review the change");
    owner_phase(&mut delegate.chip, c::ChipPhase::Active(c::RunStage::Running { attempt: 1 }));
    tree.rows = List::with_capacity(ONE);
    tree.rows.push(delegate).expect("delegate");
    let root = f::root_with_ended(task, delivery, None);
    let view = fixtures::page_view(&root);
    assert!(contains(view.tree(), "Review the change · running".as_bytes()));
}

#[test]
fn an_agents_long_line_draws_whole() {
    for length in [1149, 10_660] {
        let bytes = super::limits::repeated(b'x', length);
        let (root, domain) = super::chat::long_line_page(
            temper_client_domain::Address::Task { task: TASK },
            c::Activity::Words {
                message: 1,
                by: c::Requester::Person(1),
                kind: c::MessageKind::Words,
                words: fixtures::first_run::text(&bytes),
            },
            false,
        );
        super::chat::assert_long_line(&root, &domain, &bytes);
    }
}

#[test]
fn a_heard_or_notified_long_word_draws_whole() {
    for length in [1149, 10_660] {
        let bytes = super::limits::repeated(b'x', length);
        let mut word = heard_word(c::Requester::Task(RESULT_TASK), c::MessageKind::Words);
        word.words = fixtures::first_run::text(&bytes);
        for event in
            [c::Activity::Heard(word.clone()), c::Activity::Notified { recipient: c::Requester::Person(1), word }]
        {
            let mut domain = fixtures::domain_limits();
            domain.client.domain.long_text = 131_072;
            for words in [fixtures::words(), fixtures::browser_words()] {
                let bounds = crate::Limits::of(&domain, &words).unwrap();
                let mut builder = Builder::new(&bounds.tree);
                crate::pages::task::owner_activity(&mut builder, &event, &words);
                assert_eq!(fixtures::text(&builder.finish()).as_ref(), bytes.as_ref());
            }
        }
    }
}

#[test]
fn a_goals_own_long_line_draws_whole() {
    for length in [1149, 10_660] {
        let bytes = super::limits::repeated(b'x', length);
        let (root, domain) = super::chat::long_line_page(
            temper_client_domain::Address::Task { task: TASK },
            c::Activity::Words {
                message: 1,
                by: c::Requester::Person(fixtures::chat::PARTY),
                kind: c::MessageKind::Words,
                words: fixtures::first_run::text(&bytes),
            },
            true,
        );
        assert_eq!(root.pages().task().unwrap().shape, temper_client_domain::pages::task::Shape::Tracked);
        super::chat::assert_long_line(&root, &domain, &bytes);
    }
}

#[test]
fn an_agents_heard_or_notified_long_word_draws_whole() {
    for length in [1149, 10_660] {
        let bytes = super::limits::repeated(b'x', length);
        let mut word = heard_word(c::Requester::Task(RESULT_TASK), c::MessageKind::Words);
        word.words = fixtures::first_run::text(&bytes);
        for event in
            [c::Activity::Heard(word.clone()), c::Activity::Notified { recipient: c::Requester::Person(1), word }]
        {
            let (root, domain) =
                super::chat::long_line_page(temper_client_domain::Address::Task { task: TASK }, event, false);
            assert_eq!(root.pages().task().unwrap().shape, temper_client_domain::pages::task::Shape::Agent);
            super::chat::assert_long_line(&root, &domain, &bytes);
        }
    }
}

#[test]
fn invalid_run_ends_draw_their_reason() {
    for (reason, expected) in [
        (c::InvalidReason::Verdict, b"Run 3 ended: its result broke its contract (verdict)".as_slice()),
        (c::InvalidReason::Form, b"Run 3 ended: its result broke its contract (form)"),
        (c::InvalidReason::Words, b"Run 3 ended: its result broke its contract (words)"),
        (c::InvalidReason::Change, b"Run 3 ended: its result broke its contract (change)"),
        (c::InvalidReason::Followups, b"Run 3 ended: its result broke its contract (followups)"),
        (c::InvalidReason::UnreadableResult, b"Run 3 ended: its result couldn't be read"),
    ] {
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::pages::task::owner_activity(
            &mut builder,
            &c::Activity::RunEnded {
                attempt: 3,
                end: c::RunEnd::Failed { class: c::HoldFailure::Invalid, invalid_reason: Some(reason) },
                spent: 0,
            },
            &fixtures::words(),
        );
        assert_eq!(fixtures::text(&builder.finish()).as_ref(), expected, "{reason:?}");
    }
}

// client/pages/task.md, 4: a role's talk is distinct from steering.
fn role_talk_document() -> c::Task {
    let mut task = f::delegate();
    task.chip.task = 52;
    task.activity = 0;
    task.chip.title = f::document_words(b"Prover");
    task.flags.ended_by = c::EndedBy::Standing;
    task.talk = Some(c::Talk { party: 3, opened: skein_lib::Wall::EPOCH, last: skein_lib::Wall::EPOCH });
    offers(
        &mut task,
        &[
            (c::ActKind::Talk, c::Admission::Allowed),
            (c::ActKind::Message, c::Admission::Allowed),
            (c::ActKind::Leave, c::Admission::Allowed),
            (c::ActKind::Stop, c::Admission::Allowed),
        ],
    );
    task
}
fn role_talk_root(task: c::Task) -> Domain {
    f::agent_root_as(task, None, f::activity(false), 3)
}
#[test]
fn a_roles_composer_names_talk_and_its_terminal_and_leave_precedes_pause() {
    let root = role_talk_root(role_talk_document());
    for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let tree = view.tree();
        let composer = index_of(tree, Role::TextArea, b"Talk");
        assert_eq!(tree.nodes()[composer].placeholder.as_deref(), Some(b"Talk to Prover".as_slice()));
        assert!(contains(tree, "Talk with Prover · it answers here, and the talk stays open until you leave or 30 minutes pass with nothing said.".as_bytes()));
        let command = index_of_text(tree, b"Or in a terminal: temper ctl DIR role talk 52 TEXT --key KEY")
            .expect("terminal command");
        assert!(command > composer);
        assert!(tree.nodes()[command].styles.as_slice().contains(&look::HINT));
        assert!(!contains(tree, b"Steer this task"));
        assert!(fixtures::find(tree, Role::TextArea, b"Message").is_none());
        let leave = index_of(tree, Role::Button, b"Leave");
        let bar = styled(tree, look::TASKBAR);
        assert!(named(subtree(tree, bar), Role::Button, b"Leave"));
        assert!(leave < index_of(tree, Role::Button, b"Pause"));
        let id = tree.nodes()[leave].id;
        assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(Action::Pages(Act::LeaveTalk)));
    }
}
#[test]
fn leave_is_absent_without_the_readers_admitted_talk_and_other_agents_keep_message() {
    for change in 0_u32..4 {
        let mut task = role_talk_document();
        match change {
            0 => task.talk.as_mut().unwrap().party = 4,
            1 => task.talk = None,
            2 => task.acts.get_mut(2).unwrap().admission = c::Admission::Refused(c::Refusal::NotOffered),
            3 => {
                task.flags.ended_by = c::EndedBy::Result;
                task.talk = None;
            }
            _ => unreachable!("four changes"),
        }
        let view = fixtures::page_view(&role_talk_root(task));
        assert!(fixtures::find(view.tree(), Role::Button, b"Leave").is_none());
        if change == 3 {
            assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_some());
            assert!(contains(view.tree(), b"Steer this task"));
            assert!(!contains(view.tree(), b"Or in a terminal"));
        }
    }
}
#[test]
fn a_refused_talk_is_said_in_jigs_words_instead_of_a_message_composer() {
    let mut task = role_talk_document();
    task.acts.get_mut(0).unwrap().admission = c::Admission::Refused(c::Refusal::NotOffered);
    let view = fixtures::page_view(&role_talk_root(task));
    assert!(contains(view.tree(), b"This act is not offered"));
    assert!(named(view.tree().nodes(), Role::Paragraph, b"Talk"));
    assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_none());
    assert!(fixtures::find(view.tree(), Role::TextArea, b"Talk").is_none());
}
