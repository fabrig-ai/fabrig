//! Deleting projects stay invisible during inbox restoration (protocol/store.md, 7.2; contracts J4).
use super::*;

#[test]
fn deleting_projects_do_not_rebuild_archived_chat_documents() {
    let (mut core, limits) = make_tree();
    let row = fixtures::ended_report(12, tasks::Party::Person(1));
    let _out = watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::Deletion { project: row.project, stage: Some(people::Stage::Ending) }),
    );
    core.cache_archived_chat(&limits, &row, Some(0));
    assert!(core.archived_chats.is_empty());
}
