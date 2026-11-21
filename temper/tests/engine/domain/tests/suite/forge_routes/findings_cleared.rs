//! Cleared findings leave mine lists and ownership handoffs (domain/red-findings.md, 6).
use super::super::findings as fixture;
use jig_domain_tasks as tasks;
use temper_engine_domain::{self as engine, Application, ApplicationAnswer, FindingCall, FindingToolAnswer};
use temper_engine_domain_forge::{self as forge, settings::AreaOwner};
use temper_engine_domain_world::{direct::Driver, fixtures::tool_call};

fn seeded(triager: AreaOwner) -> (Driver, engine::Assignment) {
    let (mut driver, run) = fixture::driver();
    fixture::settings(&mut driver, None, Some(triager));
    for number in 1..=2 {
        let mut call = fixture::file(None);
        call.kind = forge::FindingKind::Red;
        assert_eq!(
            fixture::application(&mut driver, Application::FileFinding(call)),
            ApplicationAnswer::FileFinding(forge::FileFindingAnswer::Filed { number })
        );
    }
    let mut row = fixture::row(&driver, 1).clone();
    row.blame = Some(forge::FindingBlame { strength: forge::BlameStrength::Owned, landings: Box::new([]) });
    row.cleared = Some(forge::FindingCleared { run: 8, at: row.at });
    driver.store.seed(Box::new([engine::Record::Forge { row: Box::new(forge::Stored::Finding(row)) }]), Box::new([]));
    fixture::restart(&mut driver, &run);
    (driver, run)
}

#[test]
fn a_cleared_finding_is_not_in_its_triagers_mine_document() {
    let (driver, _) = seeded(AreaOwner::Party(1));
    assert_eq!(engine::finding_holder(&driver.root, 1, None), forge::Holder::Party(1));
    for state in [None, Some(forge::FindingState::New)] {
        let mine = engine::findings_document(&driver.root, 1, 1, forge::FindingsFilter::Triage { state, mine: true });
        assert_eq!(mine.rows.iter().map(|row| row.number).collect::<Vec<_>>(), [2]);
        assert_eq!(
            engine::findings_document(&driver.root, 1, 1, forge::FindingsFilter::Triage { state, mine: false })
                .rows
                .len(),
            2
        );
    }
}

#[test]
fn a_cleared_finding_is_not_in_its_triagers_mine_tool_list() {
    let (mut driver, run) = seeded(AreaOwner::Party(1));
    fixture::settings(&mut driver, None, Some(AreaOwner::Task(run.task)));
    assert_eq!(engine::finding_holder(&driver.root, 1, None), forge::Holder::Task(run.task));
    for (call, mine) in [(301, true), (302, false)] {
        tool_call(
            &mut driver,
            &run,
            call,
            engine::Tool::Finding(FindingCall::List { state: Some(forge::FindingState::New), mine }),
        );
        let Some(engine::CallAnswer::Finding(answer)) = driver.answered(call) else { panic!("finding tool answer") };
        let FindingToolAnswer::List(document) = answer.as_ref() else { panic!("finding list") };
        let expected: &[u32] = if mine { &[2] } else { &[2, 1] };
        assert_eq!(document.rows.iter().map(|row| row.number).collect::<Vec<_>>(), expected);
    }
}

#[test]
fn changing_the_triager_hands_on_no_cleared_finding() {
    let (mut driver, run) = seeded(AreaOwner::Party(1));
    let before = driver.root.tasks().task(run.task).unwrap().inbox.len();
    fixture::settings(&mut driver, None, Some(AreaOwner::Task(run.task)));
    let news: Vec<_> = driver
        .root
        .tasks()
        .task(run.task)
        .unwrap()
        .inbox
        .iter()
        .skip(before)
        .filter(|word| matches!(word.kind, tasks::MessageKind::News { .. }))
        .collect();
    assert_eq!(news.len(), 1);
    assert_eq!(news[0].words, forge::findings::ownership_words(1, &[2], driver.env.limits.tasks.message_bytes));
}
