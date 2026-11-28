//! Charter documents follow the run's revision (domain/engine/briefs.md, section 1, and domain/engine/settings.md, sections 3 and 9).
use super::briefs::{SERVING, plan_fixture, prepare, section, tree_with_sections};
use crate::routing::run::briefs::Sections;
use crate::settings::{Change, Decider, Edit, Item, Running, Setting, Value};
use crate::tests::*;
use jig_domain_contract::parts;

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

fn fixture(sections: u32) -> (Core, Limits) {
    let (mut core, limits) = tree_with_sections(&[SERVING], brief::Limits { sections, ..limits().brief });
    core.current.test_charter_mut(1).documents = documents(b"old");
    (core, limits)
}

fn plan(core: &mut Core, limits: &Limits, task: u64) -> parts::Preparation {
    prepare(core, limits, task);
    let mut out = plan_fixture(core, limits, task);
    let [Request::Ask { connector: SERVING, ask: Ask::Connector(contract::Ask::Parts(parts::Ask::Plan(plan))) }] =
        out.as_mut()
    else {
        panic!("one connector plans its brief: {out:?}")
    };
    assert_eq!(plan.task, task);
    assert_eq!(plan.requester, if task == 2 { tasks::Party::Task(1) } else { tasks::Party::Person(1) });
    plan.as_ref().clone()
}

fn assert_documents(plan: &parts::Preparation, expected: &[crate::Document], limits: &Limits) {
    assert_eq!(plan.documents.as_ref(), expected);
    assert_eq!(plan.core.len(), 2, "the task and notes before the documents");
    let slots = usize::try_from(limits.brief.sections).unwrap();
    let used = plan.core.len().checked_add(expected.len()).unwrap();
    assert_eq!(usize::try_from(plan.slots).unwrap(), slots.checked_sub(used).unwrap());
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
    let (mut core, limits) = fixture(4);
    let plan = plan(&mut core, &limits, 2);
    assert_documents(&plan, &documents(b"old"), &limits);
    let documents = [Token::new(1), Token::new(2)];
    let document = |index: u16| parts::Section {
        required: false,
        priority: DOCUMENT_PRIORITY,
        document: Some(index),
        ..section(b"document", documents[usize::from(index)], 9, limits.brief.read_bytes)
    };
    let gathered = watch_connector_outputs(
        &mut core,
        &limits,
        SERVING,
        contract::Up::Parts(parts::Up::Planned { task: 2, sections: Box::new([document(1), document(0)]) }),
    );
    let mut order = List::with_capacity(2);
    for request in &gathered {
        if let Request::Ask {
            connector: SERVING,
            ask: Ask::Connector(contract::Ask::Parts(parts::Ask::Gather { task: 2, section, .. })),
        } = request
        {
            order.push(*section).unwrap();
        }
    }
    assert_eq!(order.as_slice(), documents);
    for section in documents {
        watch_connector_outputs(
            &mut core,
            &limits,
            SERVING,
            contract::Up::Parts(parts::Up::Sized { task: 2, section, size: 1 }),
        );
    }
    let Some(Sections::Complete(order)) = core.brief_sections.get(&2) else { panic!("completed brief") };
    let [
        brief::GatherPlaced::Core { kind: brief::Core::Task, .. },
        brief::GatherPlaced::Core { kind: brief::Core::NotesIndex, .. },
        brief::GatherPlaced::Connector { token: first, .. },
        brief::GatherPlaced::Connector { token: second, .. },
    ] = order.as_ref()
    else {
        panic!("core sections followed by documents: {order:?}")
    };
    assert_eq!([*first, *second], documents);
}

#[test]
fn documents_without_section_room_are_absent_from_both_plan_carriers() {
    for (room, count) in [(3, 1), (2, 0)] {
        let (mut core, limits) = fixture(room);
        let plan = plan(&mut core, &limits, 2);
        assert_documents(&plan, &documents(b"old")[..count], &limits);
        let section = Token::new(23);
        let out = watch_connector_outputs(
            &mut core,
            &limits,
            SERVING,
            contract::Up::Parts(parts::Up::Planned {
                task: 2,
                sections: Box::new([parts::Section {
                    required: false,
                    document: Some(u16::try_from(count).unwrap()),
                    ..super::briefs::section(b"document", section, 9, limits.brief.read_bytes)
                }]),
            }),
        );
        let mut dropped = false;
        for request in &out {
            if let Request::Ask {
                connector: SERVING,
                ask: Ask::Connector(contract::Ask::Parts(parts::Ask::Drop { task: 2, section: dropped_section })),
            } = request
            {
                assert_eq!(*dropped_section, section);
                dropped = true;
            }
        }
        assert!(dropped, "the unadvertised document's section is dropped");
        let Some(Sections::Complete(order)) = core.brief_sections.get(&2) else { panic!("completed brief") };
        assert_eq!(order.len(), 2, "the omitted document is absent from the brief");
    }
}

#[test]
fn a_charter_without_documents_keeps_the_cores_plan() {
    for absent in [true, false] {
        let (mut core, limits) = fixture(4);
        let charter = core.current.test_charter_mut(1);
        if absent {
            charter.documents = Box::new([]);
        } else {
            for document in &mut charter.documents {
                document.connector = 2; // registered, but does not serve sections
            }
        }
        let plan = plan(&mut core, &limits, 2);
        assert_documents(&plan, &[], &limits);
        watch_connector_outputs(
            &mut core,
            &limits,
            SERVING,
            contract::Up::Parts(parts::Up::Planned { task: 2, sections: Box::new([]) }),
        );
        let Some(Sections::Complete(order)) = core.brief_sections.get(&2) else { panic!("completed brief") };
        let [
            brief::GatherPlaced::Core { kind: brief::Core::Task, .. },
            brief::GatherPlaced::Core { kind: brief::Core::NotesIndex, .. },
        ] = order.as_ref()
        else {
            panic!("only core sections: {order:?}")
        };
    }
}

#[test]
fn documents_of_an_existing_tasks_revision_survive_a_change_and_new_tasks_read_the_new_ones() {
    let (mut core, limits) = fixture(4);
    replace_documents(&mut core, &limits);
    assert_eq!(core.tasks.task(2).unwrap().settings.task, 1);
    let new = new_row(&core, 5, tasks::Funder::Period { project: 1, period: 1 }, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([new]));
    assert_eq!(core.tasks.task(5).unwrap().settings.task, 2);
    let old_plan = plan(&mut core, &limits, 2);
    assert_eq!(core.tasks.task(2).unwrap().settings.run, Some(1));
    assert_documents(&old_plan, &documents(b"old"), &limits);
    let plan = plan(&mut core, &limits, 5);
    assert_eq!(core.tasks.task(5).unwrap().settings.run, Some(2));
    assert_documents(&plan, &documents(b"new"), &limits);
}

#[test]
fn documents_of_a_conversation_move_to_the_newest_at_its_next_run() {
    let (mut core, limits) = fixture(4);
    let mut conversation = new_row(&core, 5, tasks::Funder::Period { project: 1, period: 1 }, None);
    conversation.flags.conversation = true;
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([conversation]));
    replace_documents(&mut core, &limits);
    assert_eq!(core.tasks.task(5).unwrap().settings.task, 1);
    let plan = plan(&mut core, &limits, 5);
    assert_eq!(core.tasks.task(5).unwrap().settings, tasks::SettingsRevision { task: 2, run: Some(2) });
    assert_documents(&plan, &documents(b"new"), &limits);
}
