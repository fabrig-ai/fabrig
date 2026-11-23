//! Deadline ordering belongs to the injected clock (testing/reference-case.md).
use super::*;

// A one-second delegate span made at ten seconds, with both park orders around its wall.
const MADE: Duration = Duration::from_secs(10);
const SPAN: Duration = Duration::from_secs(1);
const WALL: Wall = Wall::from_nanos(11_000_000_000);
const PAST: Duration = Duration::from_millis(11_001);
const LATER: Wall = Wall::from_nanos(20_000_000_000);
const SPLIT: &[u8] = b"What has landed is M1; a5's fix opens M2.";
// Director, owner, subowner and implementer occupy one slot each.
const TASKS: u32 = 4;
const DEPTH: u32 = 3;
// Six retained calls: three delegations, pass, amend and release.
const CALLS: u32 = 6;
// The batch fixture's 150-unit chat funds the 80-unit owner. Its 40-unit
// subowner funds the 10-unit implementer and the fixture's five-unit runs.
const OWNER_SPEND: u64 = 80;
const SUBOWNER_SPEND: u64 = 40;
const REPORTS: [&[u8]; 4] = [b"director done", b"owner done", b"split done", b"a5 done"];

fn delegator(words: &[u8], spend: u64, depth: u32) -> engine::Delegate {
    let mut member = report_delegate(words, Box::new([]));
    member.authority.budget.spend = spend;
    member.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: depth, depth };
    member
}

fn family() -> (Driver, engine::Assignment, engine::Assignment, engine::Assignment, engine::Assignment) {
    let (configuration, mut bounds) = batch_settings(TASKS, DEPTH, 500, false, false, false);
    bounds.call_records = CALLS;
    bounds.journal.writes = EXTENDED_BATCH_WRITES;
    let (mut driver, director) = batch_driver(configuration, &bounds, TASKS);
    let mut owner = delegator(b"owner", OWNER_SPEND, 2);
    owner.authority.budget.deadline = Some(LATER);
    let owner = call_batch(&mut driver, &director, 701, Box::new([owner]))[0];
    let owner = assigned_task(&driver, owner);
    turn(&mut driver, &director, 1, 0);
    driver.settle();
    driver.fire_at(MADE);
    driver.settle();
    let mut subowner = delegator(b"subowner", SUBOWNER_SPEND, 1);
    subowner.deadline = tasks::Deadline::Span(SPAN);
    let subowner = call_batch(&mut driver, &owner, 702, Box::new([subowner]))[0];
    let subowner = assigned_task(&driver, subowner);
    turn(&mut driver, &owner, 1, 0);
    driver.settle();
    let a5 = child(&mut driver, &subowner);
    turn(&mut driver, &subowner, 1, 0);
    driver.settle();
    (driver, director, owner, subowner, a5)
}

fn child(driver: &mut Driver, subowner: &engine::Assignment) -> engine::Assignment {
    let child = call_batch(driver, subowner, 703, Box::new([report_delegate(b"a5", Box::new([]))]))[0];
    assigned_task(driver, child)
}

fn activity(driver: &Driver, task: u64) -> Vec<&tasks::ActivityLine> {
    driver
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(row))) if row.task == task => Some(row),
            _ => None,
        })
        .collect()
}

fn held(driver: &mut Driver, subowner: &engine::Assignment, park_at: Duration) {
    driver.fire_at(park_at);
    driver.settle();
    park_task(driver, subowner);
    driver.fire_at(PAST);
    driver.settle();
}

fn pass(driver: &mut Driver, owner: &engine::Assignment, subowner: u64) {
    tool_call(
        driver,
        owner,
        704,
        engine::Tool::DecideEscalation { task: subowner, revision: 1, decision: engine::EscalationChoice::Pass },
    );
}

fn empty_person_inbox(driver: &mut Driver) {
    use jig_domain_core::reads;
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(707)),
        sign_in: driver.session(),
        read: Box::new(reads::Read::Inbox {
            group: reads::Group::Needs,
            filter: reads::Filter { project: None, goal: None },
            from: None,
        }),
    }));
    driver.settle();
    let page = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| match delivery {
            Delivery::Read { answer: reads::Answer::Page(page), .. } => Some(page),
            _ => None,
        })
        .expect("person's inbox");
    assert!(page.rows.is_empty() && !page.cut && page.next.is_none());
}

fn ended_attempts(driver: &Driver, task: u64) {
    let rows = activity(driver, task);
    let began: std::collections::BTreeSet<_> = rows
        .iter()
        .filter_map(|row| match row.line {
            tasks::Line::RunBegan { attempt, .. } => Some(attempt),
            _ => None,
        })
        .collect();
    let ended: Vec<_> = rows
        .iter()
        .filter_map(|row| match row.line {
            tasks::Line::RunEnded { attempt, end, spent } => {
                assert!(
                    matches!(end, tasks::EndKind::Parked | tasks::EndKind::Finished)
                        || (end == tasks::EndKind::Refused
                            && spent == 0
                            && !rows.iter().any(
                                |row| matches!(row.line, tasks::Line::Turn { attempt: turned, .. } if turned == attempt)
                            ))
                );
                Some(attempt)
            }
            _ => None,
        })
        .collect();
    assert_eq!(began, ended.iter().copied().collect());
    assert_eq!(began.len(), ended.len());
}

#[test]
fn a_running_delegate_finishes_across_its_parked_creators_wall() {
    let (mut driver, _, _, subowner, a5) = family();
    park_task(&mut driver, &subowner);
    let opening = activity(&driver, subowner.task);
    assert!(opening.iter().any(|row| matches!(row.line, tasks::Line::Turn { .. }) && row.at < WALL));
    assert!(
        opening
            .iter()
            .any(|row| matches!(row.line, tasks::Line::RunEnded { end: tasks::EndKind::Parked, .. }) && row.at < WALL)
    );
    assert_eq!(live(&driver, subowner.task).authority.budget.deadline, Some(WALL));
    assert_eq!(live(&driver, a5.task).authority.budget.deadline, Some(WALL));
    driver.fire_at(PAST);
    driver.settle();
    assert!(matches!(live(&driver, subowner.task).phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. }));
    assert!(matches!(live(&driver, a5.task).phase, tasks::Phase::Active(tasks::Active::Running { .. })));
    finish_task(&mut driver, &a5, tasks::TaskResult::Report { words: REPORTS[3].into() });
    let rows = activity(&driver, a5.task);
    assert_eq!(rows.iter().filter(|row| matches!(row.line, tasks::Line::RunBegan { .. })).count(), 1);
    assert_eq!(rows.iter().filter(|row| matches!(row.line, tasks::Line::RunEnded { .. })).count(), 1);
    assert!(
        rows.iter()
            .any(|row| matches!(row.line, tasks::Line::RunBegan { attempt, .. } if attempt == a5.attempt)
                && row.at < WALL)
    );
    assert!(rows.iter().any(|row| matches!(row.line, tasks::Line::RunEnded { attempt, end: tasks::EndKind::Finished, .. } if attempt == a5.attempt) && row.at > WALL));
}

#[test]
fn either_park_order_holds_once_without_a_late_turn() {
    for park_at in [MADE, PAST] {
        let (mut driver, _, owner, subowner, _) = family();
        held(&mut driver, &subowner, park_at);
        driver.fire_at(Duration::from_secs(12));
        driver.settle();
        assert!(
            matches!(live(&driver, subowner.task).phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. }),
            "park at {park_at:?}: {:?}",
            live(&driver, subowner.task).phase
        );
        assert!(live(&driver, subowner.task).subscriptions.is_empty());
        assert!(matches!(live(&driver, subowner.task).escalation,
            tasks::Escalation::Waiting { revision: 1, holder: tasks::EscalationHolder::Task(task), .. } if task == owner.task));
        assert_eq!(messages_to(&driver, &owner).len(), 1);
        let rows = activity(&driver, subowner.task);
        assert_eq!(rows.iter().filter(|row| matches!(row.line, tasks::Line::RunBegan { .. })).count(), 1);
        assert!(rows.iter().filter(|row| matches!(row.line, tasks::Line::Turn { .. })).all(|row| row.at < WALL));
    }
}

#[test]
fn a_deadline_hold_passes_from_owner_to_director_once() {
    let (mut driver, director, owner, subowner, _) = family();
    held(&mut driver, &subowner, MADE);
    pass(&mut driver, &owner, subowner.task);
    driver.fire_at(Duration::from_secs(12));
    driver.settle();
    assert!(matches!(
        driver.answered(704),
        Some(engine::CallAnswer::EscalationDecided { outcome: tasks::EscalationOutcome::Passed { .. }, .. })
    ));
    assert!(matches!(live(&driver, subowner.task).escalation,
        tasks::Escalation::Waiting { revision: 1, holder: tasks::EscalationHolder::Task(task), .. } if task == director.task));
    assert_eq!(messages_to(&driver, &owner).len(), 1);
    assert_eq!(messages_to(&driver, &director).len(), 1);
    empty_person_inbox(&mut driver);
    assert!(matches!(live(&driver, subowner.task).phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. }));
}

#[test]
fn amending_then_releasing_a_deadline_hold_runs_the_split() {
    let (mut driver, director, owner, subowner, a5) = family();
    held(&mut driver, &subowner, MADE);
    pass(&mut driver, &owner, subowner.task);
    let mut authority = live(&driver, subowner.task).authority.clone();
    authority.budget.deadline = Some(LATER);
    let mut amendment = task_amendment(None, Some(authority), b"split at the cap");
    amendment.instructions = Some(SPLIT.into());
    tool_call(
        &mut driver,
        &director,
        705,
        engine::Tool::Amend { keep_parameters: false, target: subowner.task, revision: 0, amendment },
    );
    assert!(
        matches!(driver.answered(705), Some(engine::CallAnswer::Amended { revision: 1, .. })),
        "{:?}",
        driver.answered(705)
    );
    assert_eq!(live(&driver, subowner.task).instructions.as_deref(), Some(SPLIT));
    assert_eq!(live(&driver, subowner.task).authority.budget.deadline, Some(LATER));
    assert_eq!(
        activity(&driver, subowner.task).iter().filter(|row| matches!(row.line, tasks::Line::RunBegan { .. })).count(),
        1
    );
    tool_call(&mut driver, &director, 706, engine::Tool::Release { target: subowner.task, revision: 1, again: None });
    driver.settle();
    let next = driver.last_assignment();
    assert_eq!(next.task, subowner.task);
    assert!(next.attempt > subowner.attempt);
    assert_eq!(next.run.policy.instructions.as_ref(), SPLIT);
    assert_eq!(next.run.revision, 1);
    let rows = activity(&driver, subowner.task);
    let decisions: Vec<_> = rows
        .iter()
        .filter(|row| row.at > WALL && matches!(row.line, tasks::Line::Decided { .. }))
        .map(|row| row.position)
        .collect();
    let begin = rows
        .iter()
        .find(|row| matches!(row.line, tasks::Line::RunBegan { attempt, .. } if attempt == next.attempt))
        .expect("released begin")
        .position;
    assert_eq!(decisions.len(), 2);
    assert!(decisions[0] < decisions[1] && decisions[1] < begin);
    assert!(rows.iter().filter(|row| matches!(row.line, tasks::Line::Turn { .. })).all(|row| row.at < WALL));
    for (run, report) in [&a5, &next, &owner, &director].into_iter().zip(REPORTS.into_iter().rev()) {
        finish_task(&mut driver, run, tasks::TaskResult::Report { words: report.into() });
        ended_attempts(&driver, run.task);
        assert!(matches!(driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(run.task)))),
            Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(ended)))) if ended.phase == tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: report.into() }))));
    }
    empty_person_inbox(&mut driver);
}
