//! Shared milestone facts and the forge's totals read.
use super::fixtures::{self as f, chat};
use crate::{
    forge::milestones as totals,
    milestones::{self, Facts, Of},
};
use jig_client::{Readable, Readout, contract as c};
use skein_lib::Wall;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[test]
fn the_gathering_executor_alone_marks_a_milestone() {
    assert_eq!(milestones::TEMPLATE, 8);
    let mut chip = chat::task().chip;
    for (executor, expected) in [
        (
            c::Executor::Procedure { connector: jig_client::shapes::RECURRING, code: jig_client::shapes::GATHERING },
            true,
        ),
        (
            c::Executor::Procedure {
                connector: jig_client::shapes::RECURRING,
                code: jig_client::shapes::GATHERING + 1,
            },
            false,
        ),
        (
            c::Executor::Procedure {
                connector: jig_client::shapes::RECURRING - 1,
                code: jig_client::shapes::GATHERING,
            },
            false,
        ),
        (c::Executor::Agent { charter: 8 }, false),
    ] {
        chip.details.details.executor = executor;
        assert_eq!(milestones::is(&chip), expected);
    }
}

#[test]
fn lineage_is_root_first_and_the_next_chip_is_the_goal() {
    // Jig reverses the requester walk in documents/task_tree.rs, task_with_lineage;
    // the wire's document translation preserves that order.
    let mut task = chat::task();
    let mut milestone = task.chip.clone();
    milestone.task = 40;
    milestone.details.details.executor =
        c::Executor::Procedure { connector: jig_client::shapes::RECURRING, code: jig_client::shapes::GATHERING };
    let mut goal = task.chip.clone();
    goal.task = 41;
    let mut root = task.chip.clone();
    root.task = 39;
    task.lineage_chips = skein_lib::List::from_boxed(Box::new([root, milestone.clone(), goal.clone()]));
    let lineage = milestones::lineage(&task).unwrap();
    assert_eq!((lineage.milestone.task, lineage.goal.task), (40, 41));
    task.lineage_chips = skein_lib::List::from_boxed(Box::new([milestone]));
    task.requester = c::Requester::Task(40);
    let lineage = milestones::lineage(&task).unwrap();
    assert_eq!((lineage.milestone.task, lineage.goal.task), (40, task.chip.task));
    task.requester = c::Requester::Person(40);
    assert!(milestones::lineage(&task).is_none());
    task.lineage_chips = skein_lib::List::from_boxed(Box::new([goal]));
    assert!(milestones::lineage(&task).is_none());
}

#[test]
fn facts_keep_the_name_words_maker_and_concluding_wall() {
    let mut task = chat::task();
    task.requester = c::Requester::Person(73);
    task.made = Wall::from_nanos(51);
    for (spec, name, words) in [
        (b"Store shape".as_slice(), b"Store shape".as_slice(), None),
        (b"Store shape\n\nThe store's new shape", b"Store shape", Some(b"The store's new shape".as_slice())),
        (b"Store shape\nNo blank line", b"Store shape", None),
        (b"Store shape\n\n", b"Store shape", None),
        (
            b"A milestone name made by another client stays whole beyond sixty characters",
            b"A milestone name made by another client stays whole beyond sixty characters",
            None,
        ),
    ] {
        task.spec.specification.words = c::DocumentText::complete(c::Text::new(spec.into(), 256).unwrap());
        for whole in [None, Some(Wall::from_nanos(89))] {
            task.concluded = match whole {
                Some(at) => Some(c::Concluded { by: 91, at }),
                None => None,
            };
            assert_eq!(
                milestones::facts(&task),
                Facts {
                    name: name.into(),
                    words: match words {
                        Some(words) => Some(words.into()),
                        None => None,
                    },
                    maker: task.requester,
                    whole,
                    made: Wall::from_nanos(51),
                }
            );
        }
    }
}

#[test]
fn a_goals_requester_names_its_milestone_or_the_one_line_to_read() {
    let task = chat::task();
    let mut line = c::TaskLine {
        chip: task.chip,
        project: task.project,
        requester: c::Requester::Task(812),
        tracked: task.priority,
        label: task.label,
        flags: task.flags,
        changed: task.changed,
        last_activity: task.last_activity,
        ended_at: None,
        budget: task.budget,
        waits_on: task.waits_on,
        phases: c::PhaseCounts::ZERO,
        delegates_done: 0,
        delegates_total: 0,
        held_below: 0,
        conversation: None,
    };
    assert_eq!(milestones::of(&line, &[811, 812]), Of::Milestone(812));
    assert_eq!(milestones::of(&line, &[811]), Of::Read(812));
    for requester in [c::Requester::Person(812), c::Requester::Deployment { project: 1 }] {
        line.requester = requester;
        assert_eq!(milestones::of(&line, &[812]), Of::None);
    }
}

#[test]
fn the_totals_read_names_its_project_scope_and_task() {
    let Readable::Family { scope, window, record } = totals::read(7, 812, &f::limits().forge).unwrap() else {
        panic!("forge family read")
    };
    assert_eq!(scope, c::Scope::Project(7));
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    let w::Read::Milestone(read) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().into_parts().read else {
        panic!("milestone read")
    };
    assert_eq!((read.project(), read.task()), (7, 812));
    let limits = crate::forge::Limits { field_bytes: u32::MAX, ..f::limits().forge };
    assert!(totals::read(7, 812, &limits).is_none());
}

#[test]
fn totals_answers_require_the_family_and_complete_versioned_bytes() {
    let result = w::MilestoneResult::Totals(
        w::MilestoneTotals::new(
            &w::CEILINGS,
            w::MilestoneTotalsParts {
                task: 812,
                done: 1,
                failed: 2,
                closed: 3,
                live: 4,
                ended: Some(89),
                last_landing: Some(
                    w::MilestoneLanding::new(&w::CEILINGS, w::MilestoneLandingParts { at: 87, goal: 815, change: 816 })
                        .unwrap(),
                ),
                spent: 100,
            },
        )
        .unwrap(),
    );
    let answer = w::MilestoneAnswer::new(&w::CEILINGS, w::MilestoneAnswerParts { result: result.clone() }).unwrap();
    let bytes = wire::milestone_answer(&answer);
    let readout = Readout::Family { family: wire::FORGE, record: Some(bytes.clone()) };
    assert_eq!(totals::answer(&readout, &f::limits().forge), Some(result));
    let absent =
        w::MilestoneAnswer::new(&w::CEILINGS, w::MilestoneAnswerParts { result: w::MilestoneResult::NotMilestone })
            .unwrap();
    assert_eq!(
        totals::answer(
            &Readout::Family { family: wire::FORGE, record: Some(wire::milestone_answer(&absent)) },
            &f::limits().forge
        ),
        Some(w::MilestoneResult::NotMilestone)
    );
    let mut trailing = skein_lib::Writer::new(bytes.len().checked_add(1).unwrap());
    trailing.put(&bytes).unwrap();
    trailing.put(&[0]).unwrap();
    let mut wrong_version = bytes.clone();
    wrong_version[1] = u8::try_from(wire::CLIENT_VERSION + 1).unwrap();
    for readout in [
        Readout::Family { family: wire::FORGE + 1, record: Some(bytes) },
        Readout::Family { family: wire::FORGE, record: Some(trailing.finish()) },
        Readout::Family { family: wire::FORGE, record: Some(wrong_version) },
        Readout::Family { family: wire::FORGE, record: Some(Box::default()) },
        Readout::Family { family: wire::FORGE, record: None },
        Readout::Limit,
    ] {
        assert_eq!(totals::answer(&readout, &f::limits().forge), None);
    }
}
