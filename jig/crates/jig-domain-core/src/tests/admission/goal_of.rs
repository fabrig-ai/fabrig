//! Nearest tracked ancestors shared by documents, note scopes and projections.
use crate::tests::*;

const ROOT: u64 = 10;
const CHILD: u64 = 11;
const GRANDCHILD: u64 = 12;
const REQUESTER: u64 = 1;
const PROJECT: u32 = 1;

fn make_tracked(core: &mut Core, limits: &Limits, number: u64, priority: u32) {
    let row = new_row(core, number, tasks::Funder::Period { project: PROJECT, period: 1 }, Some(priority));
    fixtures::make_batch(core, limits, tasks::Party::Person(REQUESTER), Box::new([row]));
    assert!(core.tasks.task(number).is_some(), "made tracked task {number}");
}

fn make_below(core: &mut Core, limits: &Limits, parent: u64, number: u64, tracked: Option<u32>) {
    let row = new_row(core, number, tasks::Funder::Task(parent), tracked);
    fixtures::make_batch(core, limits, tasks::Party::Task(parent), Box::new([row]));
    assert!(core.tasks.task(number).is_some(), "made delegate {number}");
}

fn notes_tree() -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    let mut rules = core.authority.rules().clone();
    rules.ceiling.notes = authority::Scopes::GOAL;
    let mut policy = core.authority.policy(PROJECT).unwrap().clone();
    policy.ceiling.notes = authority::Scopes::GOAL;
    policy.roles[0].authority.notes = authority::Scopes::GOAL;
    core.authority = authority::Domain::new(rules, limits.authority).unwrap();
    let mut out = Queue::uncapped();
    authority::step(&mut core.authority, authority::Event::Policy { project: PROJECT, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: PROJECT }));
    (core, limits)
}

fn nested() -> (Core, Limits) {
    let (mut core, limits) = notes_tree();
    make_tracked(&mut core, &limits, ROOT, 1);
    make_below(&mut core, &limits, ROOT, CHILD, Some(2));
    make_below(&mut core, &limits, CHILD, GRANDCHILD, None);
    (core, limits)
}

#[test]
fn a_tracked_task_is_the_goal_it_serves() {
    let (mut core, limits) = tree_core();
    make_tracked(&mut core, &limits, ROOT, 1);
    assert_eq!(core.goal_of(core.tasks.task(ROOT).unwrap()), Some(ROOT));
}

#[test]
fn an_untracked_delegate_serves_its_nearest_tracked_ancestor() {
    let (mut core, limits) = tree_core();
    make_tracked(&mut core, &limits, ROOT, 1);
    make_below(&mut core, &limits, ROOT, CHILD, None);
    make_below(&mut core, &limits, CHILD, GRANDCHILD, None);
    assert_eq!(core.goal_of(core.tasks.task(GRANDCHILD).unwrap()), Some(ROOT));
}

#[test]
fn a_tracked_delegate_under_a_tracked_root_serves_itself() {
    let (core, _) = nested();
    assert_eq!(core.goal_of(core.tasks.task(CHILD).unwrap()), Some(CHILD));
    assert_eq!(core.goal_of(core.tasks.task(GRANDCHILD).unwrap()), Some(CHILD));
}

#[test]
fn a_tree_with_no_tracked_task_serves_no_goal() {
    let (mut core, limits) = tree_core();
    make_task(&mut core, &limits, ROOT);
    make_below(&mut core, &limits, ROOT, CHILD, None);
    assert_eq!(core.goal_of(core.tasks.task(CHILD).unwrap()), None);
}

#[test]
fn the_task_document_names_the_nearest_tracked_goal() {
    let (core, limits) = nested();
    for number in [CHILD, GRANDCHILD] {
        let Document::Task(task) =
            make(&core, &limits, REQUESTER, &Subject::Task { task: number }, &window(), Wall::EPOCH).unwrap()
        else {
            panic!("task document");
        };
        assert_eq!(task.goal, Some(CHILD), "task {number}");
    }
}

#[test]
fn a_delegates_note_scope_is_its_nearest_goal() {
    let (core, _) = nested();
    for (goal, readable) in [(ROOT, false), (CHILD, true)] {
        assert_eq!(core.note_readable(GRANDCHILD, &notes::Scope::Goal { project: PROJECT, goal }), readable);
    }
}

#[test]
fn a_task_with_no_goal_reads_no_goal_scoped_note() {
    let (mut core, limits) = tree_core();
    let row = new_row(&core, ROOT, tasks::Funder::Period { project: PROJECT, period: 1 }, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(REQUESTER), Box::new([row]));
    assert!(!core.note_readable(ROOT, &notes::Scope::Goal { project: PROJECT, goal: ROOT }));
}

#[test]
fn each_tracked_subtree_feeds_its_own_projection() {
    let (core, limits) = nested();
    for (goal, expected) in [(ROOT, &[ROOT][..]), (CHILD, &[CHILD, GRANDCHILD][..])] {
        let mut numbers = List::with_capacity(limits.tasks.tree_tasks);
        for row in &core.projections.get(&goal).unwrap().plan {
            numbers.push(row.number).unwrap();
        }
        assert_eq!(numbers.as_slice(), expected, "projection {goal}");
    }
}

fn scope_refused(core: &Core, task: u64, project: u32, goal: u64) {
    let answer = core.authorize_tool(
        CallKey { task, attempt: 1, completion: 1, position: 0 },
        ToolKind::Note { scope: notes::Scope::Goal { project, goal } },
    );
    let Some(CallPart::ToolDenied { answer, findings }) = answer else { panic!("scope refused") };
    assert_eq!(answer, authority::Answer::Refuse);
    assert_eq!(findings.as_ref(), &[authority::Finding::Scope { source: authority::Source::Task }]);
    assert!(!core.note_readable(task, &notes::Scope::Goal { project, goal }));
}

#[test]
fn a_goals_task_cannot_write_a_chats_scope() {
    let (mut core, limits) = notes_tree();
    make_task(&mut core, &limits, ROOT);
    make_tracked(&mut core, &limits, CHILD, 1);
    make_below(&mut core, &limits, CHILD, GRANDCHILD, None);
    scope_refused(&core, GRANDCHILD, PROJECT, ROOT);
}

#[test]
fn a_goal_writes_only_its_own() {
    let (core, _) = nested();
    assert_eq!(core.note_goal(core.tasks.task(CHILD).unwrap()), Some(CHILD));
    assert!(core.note_readable(CHILD, &notes::Scope::Goal { project: PROJECT, goal: CHILD }));
    assert!(
        core.authorize_tool(
            CallKey { task: CHILD, attempt: 1, completion: 1, position: 0 },
            ToolKind::Note { scope: notes::Scope::Goal { project: PROJECT, goal: CHILD } },
        )
        .is_none()
    );
    scope_refused(&core, CHILD, PROJECT, ROOT);
}

#[test]
fn a_task_of_another_project_cannot_write_it() {
    let (mut core, limits) = notes_tree();
    make_task(&mut core, &limits, ROOT);
    scope_refused(&core, ROOT, PROJECT + 1, ROOT);
}

#[test]
fn a_delegate_of_a_chat_has_no_scope_of_its_own() {
    let (mut core, limits) = notes_tree();
    make_task(&mut core, &limits, ROOT);
    let mut row = new_row(&core, CHILD, tasks::Funder::Task(ROOT), None);
    row.flags.conversation = true;
    fixtures::make_batch(&mut core, &limits, tasks::Party::Task(ROOT), Box::new([row]));
    assert_eq!(core.note_goal(core.tasks.task(CHILD).unwrap()), None);
    scope_refused(&core, CHILD, PROJECT, CHILD);
    scope_refused(&core, CHILD, PROJECT, ROOT);
}

#[test]
fn a_task_not_flagged_conversation_has_none() {
    let (mut core, limits) = notes_tree();
    let row = new_row(&core, ROOT, tasks::Funder::Period { project: PROJECT, period: 1 }, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(REQUESTER), Box::new([row]));
    assert_eq!(core.note_goal(core.tasks.task(ROOT).unwrap()), None);
    scope_refused(&core, ROOT, PROJECT, ROOT);
}
