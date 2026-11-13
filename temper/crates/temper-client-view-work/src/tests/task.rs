pub(super) use super::fixtures::task_helpers::held_agent_root;

use super::fixtures::{self, task as f};

use jig_client::contract as c;

use skein_lib::List;

use skein_ui::{Builder, Role};

use super::inbox::{styled, subtree};

use crate::look;

use skein_ui::{Node, Tree};

use temper_client_domain::{Action, pages::Act};

const TASK: u64 = 9;

const CHILD: u64 = 12;

const PRIORITY: u32 = 1;

const CHARTER: u32 = 1;

const PROCEDURE: u32 = 2;

const PARTY: u64 = 73;

const ONE: u32 = 1;

const NO_ROWS: u32 = 0;

#[test]
fn a_plans_chip_links_to_its_tasks_page_by_number() {
    let mut delivery = f::tree();
    let c::Delivery::Tree(owner) = &mut delivery else { panic!("owner tree") };
    owner.rows = List::with_capacity(ONE);
    owner.rows.push(row()).expect("row");
    let mut builder = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::bindings::Bindings::new(fixtures::tree_limits().bindings);
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
            &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
        &mut crate::bindings::Bindings::new(limits.bindings),
        fixtures::domain_limits().client.domain.rows,
    );
    let text = fixtures::text(&builder.finish());
    assert!(skein_lib::bytes::find(&text, b"Look again before deciding.").is_some());
}

const FIRST_CARD: u32 = 0;

const NEW_HOLD_REVISION: c::Revision = c::Revision { value: 8 };

fn has_release_card(tree: &Tree) -> bool {
    for node in tree.nodes() {
        if node.role == Role::Item && node.key == Some(skein_ui::Key { owner: look::owner(), kind: 2, a: TASK, b: 0 }) {
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
        &mut crate::bindings::Bindings::new(limits.bindings),
        fixtures::domain_limits().client.domain.rows,
    );
    assert!(has_release_card(&builder.finish()));
}

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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
            &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
    panic!("missing {role:?}: {}", name.escape_ascii());
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
                &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
                &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
            &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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

fn has_style(nodes: &[Node], style: skein_ui::Style) -> bool {
    for node in nodes {
        if node.styles.as_slice().contains(&style) {
            return true;
        }
    }
    false
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
