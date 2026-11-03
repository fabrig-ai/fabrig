//! Charter documents follow the run's revision (domain/engine.md, sections 9, 17.3 and 17.9).
use crate::settings::{Change, Decider, Edit, Item, Running, Setting, Value};
use crate::tests::*;
use jig_domain_brief as brief;

fn documents(prefix: &[u8]) -> Box<[crate::Document]> {
    let mut documents = List::with_capacity(2);
    for path in [b"first".as_slice(), b"second".as_slice()] {
        documents
            .push(crate::Document {
                connector: 1,
                name: authority::Name { segments: Box::new([prefix.into()]) },
                path: path.into(),
                heading: Some(b"Rules".as_slice().into()),
            })
            .unwrap();
    }
    documents.into_boxed()
}

fn fixture() -> (Core, Limits) {
    let (mut core, limits) = make_tree();
    core.current.test_charter_mut(1).documents = documents(b"old");
    (core, limits)
}

fn plan(core: &mut Core, limits: &Limits, task: u64) -> (List<brief::Planned>, Box<[PlannedDocument]>) {
    send_task(core, limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(task)), task });
    assert_eq!(core.tasks.task(task).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
    let context = core.tasks.run_context(task).unwrap();
    core.contexts.insert(task, context).unwrap();
    core.begin_transcript(task, 0);
    let mut work = Queue::uncapped();
    let mut out = crate::routing::run::preparation::plan_brief(
        core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
        &mut work,
        task,
        List::with_capacity(0),
        0,
    );
    assert!(work.is_empty());
    let Some(Request::Now(now)) = out.pop() else { panic!("brief plan") };
    let Now::BriefCorePlanned { task: planned, parent, sections, documents } = *now else { panic!("brief plan") };
    assert_eq!(planned, task);
    assert_eq!(parent, if task == 2 { Some(1) } else { None });
    assert!(out.is_empty());
    (sections, documents)
}

fn assert_documents(
    sections: &List<brief::Planned>,
    planned: &[PlannedDocument],
    expected: &[crate::Document],
    limits: &Limits,
) {
    // This fixture needs only the task and notes index before its documents.
    assert_eq!(skein_lib::length::usize_of(sections.len()), expected.len().checked_add(2).unwrap());
    for (index, expected) in [(0, brief::Core::Task), (1, brief::Core::NotesIndex)] {
        let brief::Planned::Core { kind, .. } = sections.get(index).unwrap() else { panic!("core section first") };
        assert_eq!(*kind, expected);
    }
    assert_eq!(planned.len(), expected.len());
    for (index, document) in expected.iter().enumerate() {
        let token = Token::new(u64::try_from(index).unwrap().checked_add(1).unwrap());
        assert_eq!(planned[index], PlannedDocument { token, document: document.clone() });
        assert_eq!(
            sections.get(u32::try_from(index).unwrap().checked_add(2).unwrap()).unwrap(),
            &brief::Planned::Connector {
                connector: document.connector,
                kind: DOCUMENT_KIND,
                token,
                size: 0,
                limit: limits.brief.read_bytes,
                priority: DOCUMENT_PRIORITY,
                required: false,
            }
        );
    }
}

fn replace_documents(core: &mut Core, limits: &Limits) {
    let mut charter = core.charter(1).unwrap().clone();
    charter.documents = documents(b"new");
    let mut work = Queue::uncapped();
    let mut routed = crate::routing::party::dispatch::Routed::new(&mut work);
    assert_eq!(
        crate::routing::settings::dispatch::decide(
            core,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            &mut routed,
            Decider::Party(1),
            None,
            Change {
                against: 1,
                items: Box::new([Item {
                    setting: Setting::Charter(1),
                    edit: Edit::Set(Value::Charter(Box::new(charter))),
                }]),
                running: Running::Stays
            },
            crate::routing::settings::dispatch::route_writes(limits).unwrap(),
        ),
        Ok(2)
    );
}

#[test]
fn documents_plan_optional_connector_sections_after_the_cores_sections() {
    let (mut core, mut limits) = fixture();
    limits.brief.sections = 4;
    let (sections, planned) = plan(&mut core, &limits, 2);
    assert_documents(&sections, &planned, &documents(b"old"), &limits);
}

#[test]
fn documents_without_section_room_are_absent_from_both_plan_carriers() {
    for (room, count) in [(3, 1), (2, 0)] {
        let (mut core, mut limits) = fixture();
        limits.brief.sections = room;
        let (sections, planned) = plan(&mut core, &limits, 2);
        assert_documents(&sections, &planned, &documents(b"old")[..count], &limits);
    }
}

#[test]
fn a_charter_without_documents_keeps_the_cores_plan() {
    let (mut core, limits) = make_tree();
    let (sections, planned) = plan(&mut core, &limits, 2);
    assert_documents(&sections, &planned, &[], &limits);
}

#[test]
fn documents_of_an_existing_tasks_revision_survive_a_change_and_new_tasks_read_the_new_ones() {
    let (mut core, mut limits) = fixture();
    limits.brief.sections = 4;
    replace_documents(&mut core, &limits);
    assert_eq!(core.tasks.task(2).unwrap().settings.task, 1);
    let new = new_row(&core, 5, tasks::Funder::Period { project: 1, period: 1 }, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([new]));
    assert_eq!(core.tasks.task(5).unwrap().settings.task, 2);
    let (sections, planned) = plan(&mut core, &limits, 2);
    assert_eq!(core.tasks.task(2).unwrap().settings.run, Some(1));
    assert_documents(&sections, &planned, &documents(b"old"), &limits);
    let (sections, planned) = plan(&mut core, &limits, 5);
    assert_eq!(core.tasks.task(5).unwrap().settings.run, Some(2));
    assert_documents(&sections, &planned, &documents(b"new"), &limits);
}

#[test]
fn documents_of_a_conversation_move_to_the_newest_at_its_next_run() {
    let (mut core, mut limits) = fixture();
    limits.brief.sections = 4;
    let mut conversation = new_row(&core, 5, tasks::Funder::Period { project: 1, period: 1 }, None);
    conversation.flags.conversation = true;
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([conversation]));
    replace_documents(&mut core, &limits);
    assert_eq!(core.tasks.task(5).unwrap().settings.task, 1);
    let (sections, planned) = plan(&mut core, &limits, 5);
    assert_eq!(core.tasks.task(5).unwrap().settings, tasks::SettingsRevision { task: 2, run: Some(2) });
    assert_documents(&sections, &planned, &documents(b"new"), &limits);
}
