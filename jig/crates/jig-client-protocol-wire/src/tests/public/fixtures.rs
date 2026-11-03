//! Largest public answers under index limits (client/domain/contract.md, section 12).
use crate::public_v1 as w;
use alloc::boxed::Box;
use skein_lib::{List, Overflow, Writer};

pub fn encoded<T>(value: &T, bound: u32, encode: fn(&T, &mut Writer) -> Result<(), Overflow>) -> Box<[u8]> {
    let mut writer = Writer::new(skein_lib::length::usize_of(bound));
    encode(value, &mut writer).expect("answer reservation holds");
    let bytes = writer.finish();
    assert_eq!(bytes.len(), skein_lib::length::usize_of(bound), "answer bound is exact");
    bytes
}

fn text(bytes: u32) -> Box<[u8]> {
    let mut text = skein_lib::bytes::zeroed(skein_lib::length::usize_of(bytes));
    text.fill(b'x');
    text
}

fn line(limits: &w::Limits) -> w::PublicLine {
    let delegates = w::PublicCounts::new(
        limits,
        w::PublicCountsParts {
            working: u32::MAX,
            waiting: u32::MAX,
            held: u32::MAX,
            closing: u32::MAX,
            ended: u32::MAX,
        },
    )
    .expect("phase counts");
    w::PublicLine::new(
        limits,
        w::PublicLineParts {
            task: u64::MAX,
            title: text(limits.public_line_title),
            title_cut: true,
            parent: Some(u64::MAX),
            goal: true,
            priority: Some(u32::MAX),
            executor: w::PublicExecutor::Job,
            phase: w::PublicPhase::Held,
            hold: Some(w::PublicHold::Exceeded),
            made: u64::MAX,
            changed: u64::MAX,
            active: Some(u64::MAX),
            ended: Some(u64::MAX),
            spent: u64::MAX,
            budget: u64::MAX,
            delegates,
        },
    )
    .expect("largest line")
}

fn lines(limits: &w::Limits, count: u32) -> List<w::PublicLine> {
    let mut lines = List::with_capacity(count);
    let line = line(limits);
    for _ in 0..count {
        lines.push(line.clone()).expect("line room");
    }
    lines
}

pub fn goal(limits: &w::Limits) -> w::PublicGoal {
    w::PublicGoal::new(
        limits,
        w::PublicGoalParts { line: line(limits), tree: lines(limits, limits.public_goal_tree), folded: u32::MAX },
    )
    .expect("largest goal")
}

pub fn goals(limits: &w::Limits, count: u32) -> List<w::PublicGoal> {
    let mut goals = List::with_capacity(count);
    let goal = goal(limits);
    for _ in 0..count {
        goals.push(goal.clone()).expect("goal room");
    }
    goals
}

pub fn board(limits: &w::Limits, live: u32, ended: u32) -> w::PublicBoard {
    w::PublicBoard::new(
        limits,
        w::PublicBoardParts {
            project: u64::MAX,
            name: text(limits.public_board_name),
            live: goals(limits, live),
            live_folded: u32::MAX,
            ended: goals(limits, ended),
        },
    )
    .expect("largest board")
}

pub fn card(limits: &w::Limits, ancestors: u32, delegates: u32) -> w::PublicCard {
    let mut lineage = List::with_capacity(ancestors);
    for _ in 0..ancestors {
        lineage.push(u64::MAX).expect("lineage room");
    }
    w::PublicCard::new(
        limits,
        w::PublicCardParts { line: line(limits), lineage, delegates: lines(limits, delegates), folded: u32::MAX },
    )
    .expect("largest card")
}

pub fn projects(limits: &w::Limits, count: u32) -> w::PublicProjects {
    let mut projects = List::with_capacity(count);
    let project = w::PublicProject::new(
        limits,
        w::PublicProjectParts { project: u64::MAX, name: text(limits.public_project_name) },
    )
    .expect("largest project");
    for _ in 0..count {
        projects.push(project.clone()).expect("project room");
    }
    w::PublicProjects::new(limits, w::PublicProjectsParts { projects, max_age: u32::MAX }).expect("largest projects")
}
