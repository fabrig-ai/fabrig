//! Summary-only owners serve ended lists, never their other reads (protocol/store/retention.md, 7).
use super::*;
use fixtures::PROJECT;
use jig_domain_core_vocabulary::documents;
use skein_lib::Wall;

fn summary(requester: tasks::Party) -> tasks::TaskSummary {
    let core::Record::Tasks(tasks::Stored::Ended(mut owner)) = fixtures::ended_of(CHILD, requester) else {
        panic!("ended fixture")
    };
    owner.flags.listed = true;
    owner.ended_at = Some(Wall::from_nanos(19));
    tasks::TaskSummary::of(&owner)
}

fn query() -> documents::TaskQuery {
    documents::TaskQuery {
        project: Some(PROJECT),
        requester: None,
        parent: None,
        root: None,
        label: None,
        phase: None,
        executor: None,
        tracked: None,
        order: documents::QueryOrder::Number,
    }
}

fn party_index(task: u64) -> core::Record {
    core::Record::Tasks(tasks::Stored::EndedByParty(tasks::EndedByParty {
        party: ALICE,
        label: 0,
        ended_at: Wall::from_nanos(19),
        task,
    }))
}

fn project_index(task: u64) -> core::Record {
    core::Record::Tasks(tasks::Stored::EndedByProject(tasks::EndedByProject {
        project: PROJECT,
        label: 0,
        ended_at: Wall::from_nanos(19),
        task,
    }))
}

fn summary_store(requester: tasks::Party) -> Store {
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            fixtures::save(core::Record::Tasks(tasks::Stored::Summary(Box::new(summary(requester))))),
            fixtures::save(party_index(CHILD)),
            fixtures::save(project_index(CHILD)),
        ]),
    );
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(2),
            fixtures::save(fixtures::ended_delegate(GOAL, CHILD)),
            fixtures::save(fixtures::history(CHILD, 1)),
            fixtures::save(fixtures::activity_line(CHILD, 1)),
        ]),
    );
    store
}

#[test]
fn ended_indexes_join_a_summary_with_one_owner_join() {
    for (scope, value, requester) in [
        (
            reads::Scope::Tasks {
                query: documents::TaskQuery { requester: Some(tasks::Party::Person(ALICE)), ..query() },
            },
            party_index(CHILD),
            tasks::Party::Person(ALICE),
        ),
        (reads::Scope::Tasks { query: query() }, project_index(CHILD), tasks::Party::Person(ALICE)),
        (
            reads::Scope::Tasks { query: documents::TaskQuery { parent: Some(GOAL), ..query() } },
            fixtures::ended_delegate(GOAL, CHILD),
            tasks::Party::Task(GOAL),
        ),
        (reads::Scope::EndedDelegates { task: GOAL }, fixtures::ended_delegate(GOAL, CHILD), tasks::Party::Task(GOAL)),
    ] {
        let mut store = summary_store(requester);
        // No read-position or ancestry joins survive shrinking, even for a conversation.
        store.limits.joins = 1;
        let range = fixtures::read(ALICE, scope);
        let (rows, next) = page(load(&mut store, range.clone()));
        let [row] = rows.as_ref() else { panic!("summary row: {rows:?}") };
        assert_eq!(row.value, value);
        assert_eq!(row.summary.as_deref(), Some(&summary(requester)));
        assert_eq!(row.owner, None);
        assert_eq!(row.ended_read, None);
        assert_eq!(row.ancestors.as_ref(), []);
        assert_eq!(row.linked.as_ref(), []);
        assert_eq!(next, None);
        // Summary ownership is part of the same row's byte charge.
        let mut tight = range;
        tight.bytes = row.owned_bytes().unwrap() - 1;
        let (rows, _) = page(load(&mut store, tight));
        assert_eq!(rows.as_ref(), []);
    }
}

#[test]
fn a_summary_only_task_answers_point_reads_as_gone() {
    let mut store = summary_store(tasks::Party::Task(GOAL));
    for scope in [
        reads::Scope::Task { task: CHILD },
        reads::Scope::Result { task: CHILD },
        reads::Scope::TaskText { task: CHILD, field: reads::TaskTextField::Specification, from: 0 },
    ] {
        let result = load(&mut store, fixtures::read(ALICE, scope));
        let reads::Loaded::Gone = result else { panic!("shrunk task is gone: {result:?}") };
    }
}

#[test]
fn a_summary_only_tasks_history_and_activity_are_empty() {
    let mut store = summary_store(tasks::Party::Task(GOAL));
    for scope in [
        reads::Scope::History { task: CHILD, subtree: false },
        reads::Scope::History { task: CHILD, subtree: true },
        reads::Scope::Activity { task: CHILD },
    ] {
        assert_empty(&load(&mut store, fixtures::read(ALICE, scope)));
    }
}

#[test]
fn an_ended_index_row_without_record_or_summary_is_skipped() {
    let mut store = summary_store(tasks::Party::Task(GOAL));
    fixtures::committed(&mut store, Box::new([fixtures::header(3), fixtures::save(project_index(GRANDCHILD))]));
    let (rows, next) = page(load(&mut store, fixtures::read(ALICE, reads::Scope::Tasks { query: query() })));
    assert_eq!(values(&rows).as_ref(), [project_index(CHILD)]);
    assert_eq!(next, None);
}
