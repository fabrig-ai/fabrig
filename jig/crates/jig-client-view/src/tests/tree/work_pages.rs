#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary fixtures script public events and query semantic nodes"
)]
use crate::tests::fixtures::{Fixture, ReadPage};
use crate::tests::tree::{
    cards::{contains, document, drive, text, words},
    pages::{build, deliver},
};
use crate::{JigBindings, words::Words};
use skein_ui::{Role, Tree};

use crate::{
    Event, Request,
    state::{
        action::Act,
        address::{Address, Place, TaskSection},
    },
};
use jig_client::contract::*;
use skein_lib::List;
pub(in crate::tests) fn list<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let values: Vec<_> = values.into_iter().collect();
    let mut rows = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        rows.push(value).ok().unwrap();
    }
    rows
}
pub(in crate::tests) fn part() -> Part {
    Part::Panel { connector: 2, task: 8 }
}
fn counts() -> PhaseCounts {
    PhaseCounts { ended: 0, waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 }
}
pub(in crate::tests) fn chip(task: u64) -> Chip {
    let mut chip = crate::tests::tree::cards_objects::task_doc(&[]).chip;
    chip.task = task;
    chip
}
pub(in crate::tests) fn entered(address: Address) -> (Fixture, Vec<Request>) {
    entered_arranged(address, crate::state::page::Arrangement::none())
}
pub(in crate::tests) fn entered_arranged(
    address: Address,
    arrangement: crate::state::page::Arrangement,
) -> (Fixture, Vec<Request>) {
    let (mut d, f) = crate::tests::tree::cards::start_arranged(arrangement);
    let mut frame = document();
    frame.projects.get_mut(0).unwrap().pool.as_mut().unwrap().left = 20;
    let initial = deliver(&mut d, f, Delivery::Me(Box::new(frame)));
    for request in &initial {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Inbox { .. }) },
        } = request
        {
            drive(
                &mut d,
                ReadPage {
                    read: *read,
                    result: ReadResult::Value(ReadValue::OwnerInbox(Box::new(OwnerInboxPage {
                        selection: None,
                        cut: false,
                        rows: List::with_capacity(0),
                        before: None,
                        after: None,
                    }))),
                },
            );
        }
    }
    d.reclaim();
    let out = drive(&mut d, Event::Went { place: Place::Jig(address) });
    (d, out)
}
pub(in crate::tests) fn task(section: TaskSection, executor: Executor) -> (Fixture, Vec<Request>) {
    task_arranged(section, executor, crate::state::page::Arrangement::none())
}
fn task_arranged(
    section: TaskSection,
    executor: Executor,
    arrangement: crate::state::page::Arrangement,
) -> (Fixture, Vec<Request>) {
    let (mut d, out) = entered_arranged(Address::Task { task: 8, section }, arrangement);
    let watch =
        out.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let mut doc = crate::tests::tree::cards_objects::task_doc(&[ActKind::Cancel]);
    doc.chip.executor = executor;
    doc.context.as_mut().expect("supplied task context").parts = list([part()]);
    doc.inputs = list([OwnerTaskLink::Live(chip(9))]);
    doc.dependencies = list([OwnerTaskLink::Live(chip(9))]);
    let mut outputs = deliver(&mut d, watch, Delivery::Task(Box::new(doc)));
    outputs.extend(out);
    (d, outputs)
}
fn acts_are_named(tree: &Tree) {
    assert!(
        tree.nodes().iter().filter(|n| n.binding.is_some()).all(|n| n.name.as_ref().is_some_and(|n| !n.is_empty()))
    );
}
fn slot(tree: &Tree, bindings: &JigBindings) {
    let node = tree.nodes().iter().find(|n| n.role == Role::Slot).unwrap();
    let reference = node.slot.unwrap();
    assert_eq!(bindings.part(reference.token), Some(&part()));
}
fn task_sections(words: &Words) {
    for executor in [
        Executor::Agent { charter: 2 },
        Executor::Procedure { connector: 2, kind: 1 },
        Executor::Party { party: 1 },
        Executor::Role { project: 4, role: 1 },
    ] {
        for section in [TaskSection::Overview, TaskSection::Activity, TaskSection::Budget, TaskSection::Panels] {
            let (d, _) = task(section, executor);
            let (tree, bindings) = build(&d, 3, words);
            assert!(contains(&tree, b"Task 8"));
            assert!(contains(&tree, b"held") || contains(&tree, b"Held"));
            assert!(contains(&tree, b"Goal"));
            assert!(contains(&tree, b"Priority 1"));
            acts_are_named(&tree);
            if section == TaskSection::Overview {
                assert!(contains(&tree, b"whole task specification"));
                slot(&tree, &bindings);
            }
            if section == TaskSection::Panels {
                slot(&tree, &bindings);
            }
            if section == TaskSection::Activity && executor == (Executor::Procedure { connector: 2, kind: 1 }) {
                slot(&tree, &bindings);
            }
            if section == TaskSection::Budget {
                assert!(contains(&tree, b"Budget: 20"));
                assert!(contains(&tree, b"Left: 20"));
            }
        }
    }
}

pub(in crate::tests) fn recurring_task(number: u64, words: &[u8]) -> ProposedTask {
    ProposedTask {
        deadline: NewDeadline::Named,
        number,
        spec: SpecificationDocument::complete(Specification {
            asks: false,
            words: text(words),
            parameters: list([]),
            inputs: list([]),
        }),
        executor: OwnerExecutor::Agent { charter: 2 },
        budget: 5,
        contract: TaskContractDocument::Report { words: 32 },
        authority: OwnerAuthorityDocument {
            authority: OwnerAuthority {
                models: None,
                tools: 0,
                grants: list([]),
                delegation: OwnerDelegation { kinds: list([]), tasks: 2, depth: 1 },
                spend: 5,
                deadline: None,
                notes: 0,
                note_resources: list([]),
            },
            cut: false,
        },
        funder: Funder::Recurring { project: 4, task: 7, period: 3 },
        dependencies: list([]),
        resources: list([]),
        wake: OwnerWakePolicy {
            words: WakeRule::Immediate,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Each,
            questions: false,
            answers: false,
            timers: false,
        },
        priority: None,
        cut: false,
    }
}

#[test]
fn a_task_section_not_shown_builds_nothing() {
    let (d, _) = task(TaskSection::Overview, Executor::Agent { charter: 2 });
    let mut b = skein_ui::Builder::new(&crate::tests::tree::cards::limits());
    let mut bindings = JigBindings::new(64, crate::tests::tree::cards::CLIENT.text);
    b.open(Role::Page);
    assert!(!crate::tree::sections::section(
        &mut b,
        &d.view(),
        words(),
        &mut bindings,
        crate::state::page::Section::Task { task: 9, section: TaskSection::Overview }
    ));
    b.close();
    assert_eq!(b.finish().len(), 1);
}
#[test]
fn the_tabs_mark_the_addresss_own_section_current() {
    let (d, _) = task(TaskSection::Plan, Executor::Agent { charter: 2 });
    let (tree, mut bindings) = build(&d, 3, words());
    let tabs: Vec<_> = tree.nodes().iter().filter(|n| n.role == Role::Tab).collect();
    assert_eq!(tabs.len(), 6);
    for (tab, section) in tabs.iter().zip([
        TaskSection::Overview,
        TaskSection::Activity,
        TaskSection::Plan,
        TaskSection::History,
        TaskSection::Budget,
        TaskSection::Panels,
    ]) {
        assert_eq!(
            tab.states.as_slice(),
            if section == TaskSection::Plan { &[skein_ui::State::Current][..] } else { &[] },
            "{section:?}"
        );
        assert_eq!(
            crate::decode(&mut bindings, tab.binding.unwrap().token, skein_ui::Act::Press),
            Some(Act::Go { address: Address::Task { task: 8, section } }),
            "{section:?}"
        );
    }
}
#[test]
fn each_task_section_carries_its_region_name() {
    use crate::regions::JigRegion;
    for (section, region, name) in [
        (TaskSection::Overview, JigRegion::TaskOverview, b"jig-task-overview".as_slice()),
        (TaskSection::Activity, JigRegion::TaskActivity, b"jig-task-activity".as_slice()),
        (TaskSection::Plan, JigRegion::TaskPlan, b"jig-task-plan".as_slice()),
        (TaskSection::History, JigRegion::TaskHistory, b"jig-task-history".as_slice()),
        (TaskSection::Budget, JigRegion::TaskBudget, b"jig-task-budget".as_slice()),
        (TaskSection::Panels, JigRegion::TaskPanels, b"jig-task-panels".as_slice()),
    ] {
        let (d, _) = task(section, Executor::Agent { charter: 2 });
        let (tree, _) = build(&d, 3, words());
        let node = tree.nodes().iter().find(|n| n.region == Some(region.region())).unwrap();
        assert_eq!(node.role, Role::Region, "{section:?}");
        assert_eq!(region.name(), name);
        let index = tree.nodes().iter().position(|n| core::ptr::eq(n, node)).unwrap();
        assert_eq!(tree.nodes()[index + 1].role, Role::Heading { level: 2 });
    }
}

fn arranged_task() -> Fixture {
    task_arranged(
        TaskSection::Plan,
        Executor::Agent { charter: 2 },
        crate::state::page::Arrangement {
            task: list([
                TaskSection::Overview,
                TaskSection::Activity,
                TaskSection::Plan,
                TaskSection::History,
                TaskSection::Budget,
                TaskSection::Panels,
            ]),
            project: list([]),
        },
    )
    .0
}
#[test]
fn an_arranged_task_page_shows_its_six_sections_in_order() {
    use crate::regions::JigRegion;
    let (tree, _) = build(&arranged_task(), 3, words());
    let regions: Vec<_> =
        tree.nodes().iter().filter_map(|n| n.region).filter(|r| *r != JigRegion::TaskHeader.region()).collect();
    let sections = [
        JigRegion::TaskOverview,
        JigRegion::TaskActivity,
        JigRegion::TaskPlan,
        JigRegion::TaskHistory,
        JigRegion::TaskBudget,
        JigRegion::TaskPanels,
    ];
    let shown: Vec<_> = regions.into_iter().filter(|r| sections.iter().any(|s| s.region() == *r)).collect();
    assert_eq!(shown, sections.map(JigRegion::region));
}
#[test]
fn an_arranged_task_page_showing_every_section_has_no_tabs() {
    let (tree, _) = build(&arranged_task(), 3, words());
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Tabs).count(), 0);
}
#[test]
fn the_addresss_own_section_is_marked_current_on_an_arranged_page() {
    use crate::regions::JigRegion;
    let (tree, _) = build(&arranged_task(), 3, words());
    for region in [
        JigRegion::TaskOverview,
        JigRegion::TaskActivity,
        JigRegion::TaskPlan,
        JigRegion::TaskHistory,
        JigRegion::TaskBudget,
        JigRegion::TaskPanels,
    ] {
        let node = tree.nodes().iter().find(|n| n.region == Some(region.region())).unwrap();
        assert_eq!(
            node.states.as_slice(),
            if region == JigRegion::TaskPlan { &[skein_ui::State::Current][..] } else { &[] },
            "{region:?}"
        );
    }
}
