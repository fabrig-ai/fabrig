//! Shared milestone facts and the forge's totals read.
use super::fixtures::{self as f, chat};
use crate::{
    forge::milestones as totals,
    milestones::{self, Facts},
};
use jig_client::{Readable, Readout, contract as c};
use skein_lib::Wall;
use temper_client_protocol_wire::v1::Record;
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

fn row() -> c::TaskLine {
    let task = chat::task();
    c::TaskLine {
        parts: skein_lib::List::with_capacity(0),
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
        wait: None,
        nearest: None,
        phases: c::PhaseCounts::ZERO,
        delegates_done: 0,
        delegates_total: 0,
        held_below: 0,
        conversation: None,
    }
}

fn part(bytes: Box<[u8]>) -> c::RowPart {
    c::RowPart::Value(jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes })
}

#[test]
fn a_goals_part_alone_names_its_milestone() {
    let mut line = row();
    assert_eq!(milestones::of(&line), None);
    let part_value = w::GoalPart { report: None, milestone: Some(811) };
    let mut writer =
        skein_lib::Writer::new(usize::try_from(part_value.measure(&w::CEILINGS).expect("within ceilings")).unwrap());
    part_value.write(&w::CEILINGS, &mut writer).unwrap();
    line.parts = skein_lib::List::from_boxed(Box::new([part(writer.finish())]));
    assert_eq!(milestones::of(&line), Some(811));
    line.requester = c::Requester::Person(1);
    assert_eq!(milestones::of(&line), Some(811), "no inference from the requester");
}

#[test]
fn milestone_parts_translate_goal_lines_and_count_undecodable_lines_as_more() {
    use crate::parts;
    use skein_lib::{List, Writer};
    let limits =
        jig_client_protocol_link::Limits::new(f::limits().client.domain, f::limits().client.exchanges).unwrap();
    let mut line = row();
    let goal = jig_client_test_records::task_line_record();
    let mut writer = Writer::new(
        usize::try_from(goal.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    goal.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
    let bytes = writer.finish();
    let trailing = skein_lib::bytes::concat(&[&bytes, &[0]]);
    let goal_row =
        |bytes: &[u8]| w::MilestoneGoal { line: bytes.into(), headline: Some(b"Newest report.".as_slice().into()) };
    let totals = w::MilestoneTotals {
        task: line.chip.task,
        done: 2,
        failed: 0,
        closed: 0,
        live: 3,
        ended: None,
        last_landing: None,
        spent: 40,
        reds: 0,
    };
    let value = w::MilestonePart {
        whole: Some(17),
        totals: totals.clone(),
        goals: List::from_boxed(Box::new([
            goal_row(&bytes),
            goal_row(b"bad line"),
            goal_row(&trailing),
            goal_row(&bytes),
        ])),
        more: 3,
    };
    let mut writer = Writer::new(usize::try_from(value.measure(&w::CEILINGS).expect("within ceilings")).unwrap());
    value.write(&w::CEILINGS, &mut writer).unwrap();
    let bytes = writer.finish();
    line.parts = List::from_boxed(Box::new([part(bytes.clone())]));
    let decoded = parts::milestone(&line, &limits).unwrap();
    assert_eq!(decoded.whole, Some(Wall::from_nanos(17)));
    assert_eq!(decoded.totals, totals);
    assert_eq!(decoded.more, 5);
    assert_eq!(decoded.goals.len(), 2);
    let expected = jig_client_protocol_link::translate::queries::line(&goal, &limits).unwrap();
    assert_eq!(decoded.goals.get(0).unwrap().0, expected);
    assert_eq!(decoded.goals.get(0).unwrap().1.as_deref(), Some(b"Newest report.".as_slice()));
    assert_eq!(decoded.goals.get(1).unwrap().0, expected);
    let trailing = skein_lib::bytes::concat(&[&bytes, &[0]]);
    for bad in [
        c::RowPart::Unknown,
        c::RowPart::Failed(c::PartFailure::Unreadable),
        part(trailing),
        part(b"bad part".as_slice().into()),
        c::RowPart::Value(jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION + 1,
            bytes: bytes.clone(),
        }),
        c::RowPart::Value(jig_client::Family { family: wire::FORGE + 1, version: wire::CLIENT_VERSION, bytes }),
    ] {
        line.parts = List::from_boxed(Box::new([bad]));
        assert!(parts::milestone(&line, &limits).is_none());
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
    let w::Read::Milestone(read) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read else {
        panic!("milestone read")
    };
    assert_eq!((read.project, read.task), (7, 812));
    let limits = crate::forge::Limits { field_bytes: u32::MAX, ..f::limits().forge };
    assert!(totals::read(7, 812, &limits).is_none());
}

#[test]
fn totals_answers_require_the_family_and_complete_versioned_bytes() {
    let result = w::MilestoneResult::Totals(w::MilestoneTotals {
        task: 812,
        done: 1,
        failed: 2,
        closed: 3,
        live: 4,
        ended: Some(89),
        last_landing: Some(w::MilestoneLanding { at: 87, goal: 815, change: 816 }),
        spent: 100,
        reds: 0,
    });
    let answer = w::MilestoneAnswer { result: result.clone() };
    let bytes = wire::milestone_answer(&answer);
    let readout = Readout::Family { family: wire::FORGE, record: Some(bytes.clone()) };
    assert_eq!(totals::answer(&readout, &f::limits().forge), Some(result));
    let absent = w::MilestoneAnswer { result: w::MilestoneResult::NotMilestone };
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
