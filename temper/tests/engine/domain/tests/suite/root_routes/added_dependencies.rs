//! Batch dependencies and kept delegates (jig/docs/design/domain/tasks/lifecycle.md, 8 and jig/docs/design/domain/tasks/changes.md).
use super::*;

fn finish_report(driver: &mut Driver, run: &engine::Assignment, words: &[u8]) {
    driver.answer(
        Token::new(7),
        run.task,
        run.attempt,
        0,
        tasks::End::Finished { result: tasks::TaskResult::Report { words: words.into() }, cancel_delegates: false },
        engine::Parts::none(),
    );
    driver.settle();
}

fn assignments(driver: &Driver, task: u64) -> usize {
    driver
        .delivered
        .iter()
        .filter(|delivery| matches!(delivery, Delivery::Assigned { assignment, .. } if assignment.task == task))
        .count()
}

fn four_delegate_settings() -> ([engine::Config; 2], engine::Limits) {
    const PLAN_WRITES: u32 = 6_200;
    let (mut configuration, mut bounds) = batch_settings(5, 1, 500, false, false, false);
    for config in &mut configuration {
        config.core.defaults.rules.ceiling.delegation.tasks = 5;
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.tasks = 5;
        policy.roles[0].authority.delegation.tasks = 5;
    }
    bounds.tasks.tasks = 5;
    bounds.tasks.project_tasks = 5;
    bounds.tasks.tree_tasks = 5;
    bounds.tasks.delegates = 4;
    bounds.forge.tasks = 5;
    bounds.people.inbox_tasks = 10;
    bounds.tasks.inbox_messages = 12;
    bounds.tasks.inbox_bytes = 768;
    bounds.people.inbox_recipients = 29;
    bounds.journal.writes = PLAN_WRITES;
    (configuration, bounds)
}

#[test]
fn one_batch_of_four_resolves_its_local_references_and_gates_only_the_fourth() {
    let (configuration, mut bounds) = four_delegate_settings();
    bounds.tasks.batch = 4;
    bounds.authority.batch = 4;
    // The durable call record holds the whole four-member batch.
    let mut store_bounds = temper_engine_domain_world::store::widest();
    store_bounds.tasks.batch = 4;
    store_bounds.authority.batch = 4;
    let mut driver = Driver::configured(Store::with_limits(&store_bounds), configuration, &bounds);
    driver.send(hello_with(Token::new(7), 5, Duration::from_secs(1), Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let parent = assigned(&driver);
    let children = call_batch(
        &mut driver,
        &parent,
        922,
        Box::new([
            report_delegate(b"first prerequisite", Box::new([])),
            report_delegate(b"second prerequisite", Box::new([])),
            report_delegate(b"unrelated third work", Box::new([])),
            report_delegate(b"fourth delegate", Box::new([engine::Dependency::Batch(0), engine::Dependency::Batch(1)])),
        ]),
    );
    let [first, second, third, fourth] = *children else { panic!("one batch of four delegates") };
    let runs = [assigned_task(&driver, first), assigned_task(&driver, second), assigned_task(&driver, third)];
    assert_eq!(live(&driver, fourth).dependencies.as_ref(), [first, second]);
    assert_eq!(live(&driver, fourth).waiting_on.as_ref(), [first, second]);
    assert_eq!(assignments(&driver, fourth), 0);
    for run in &runs {
        assert_eq!(assignments(&driver, run.task), 1);
        assert_eq!(
            live(&driver, run.task).phase,
            tasks::Phase::Active(tasks::Active::Running { attempt: run.attempt })
        );
    }
    finish_report(&mut driver, &runs[0], b"first completion");
    assert_eq!(assignments(&driver, fourth), 0);
    assert_eq!(live(&driver, fourth).waiting_on.as_ref(), [second]);
    finish_report(&mut driver, &runs[1], b"second completion");
    assert_eq!(assignments(&driver, fourth), 1);
    assert!(live(&driver, fourth).waiting_on.is_empty());
    let fourth_run = assigned_task(&driver, fourth);
    assert_eq!(
        live(&driver, fourth).phase,
        tasks::Phase::Active(tasks::Active::Running { attempt: fourth_run.attempt })
    );
    assert_eq!(live(&driver, third).phase, tasks::Phase::Active(tasks::Active::Running { attempt: runs[2].attempt }));
}

#[test]
fn both_prerequisite_orders_and_completion_orders_gate_only_the_fourth_delegate() {
    // Five tasks: the director, two prerequisites, unrelated work and their waiter.
    for (prerequisites, completions) in [([0, 1], [0, 1]), ([1, 0], [0, 1]), ([0, 1], [1, 0]), ([1, 0], [1, 0])] {
        let (configuration, bounds) = four_delegate_settings();
        let mut driver = Driver::configured(Store::new(), configuration, &bounds);
        driver.send(hello_with(Token::new(7), 5, Duration::from_secs(1), Box::new([])));
        driver.settle();
        driver.sign_in();
        driver.settle();
        chat(&mut driver, 12);
        driver.settle();
        let parent = assigned(&driver);
        let children = call_batch(
            &mut driver,
            &parent,
            920,
            Box::new([
                report_delegate(b"first prerequisite", Box::new([])),
                report_delegate(b"second prerequisite", Box::new([])),
                report_delegate(b"unrelated third work", Box::new([])),
            ]),
        );
        let runs = [assigned_task(&driver, children[0]), assigned_task(&driver, children[1])];
        let unrelated = assigned_task(&driver, children[2]);
        let dependencies = prerequisites.map(|index| engine::Dependency::Existing(children[index]));
        let fourth = call_batch(
            &mut driver,
            &parent,
            921,
            Box::new([report_delegate(b"fourth delegate", Box::new(dependencies))]),
        )[0];
        assert_eq!(assignments(&driver, fourth), 0);
        assert_eq!(live(&driver, fourth).waiting_on.len(), 2);
        finish_report(&mut driver, &runs[completions[0]], b"first completion");
        assert_eq!(assignments(&driver, fourth), 0);
        assert_eq!(live(&driver, fourth).waiting_on.as_ref(), [runs[completions[1]].task]);
        assert!(matches!(live(&driver, unrelated.task).phase,
            tasks::Phase::Active(tasks::Active::Running { attempt }) if attempt == unrelated.attempt));
        finish_report(&mut driver, &runs[completions[1]], b"second completion");
        assert_eq!(assignments(&driver, fourth), 1);
        assert!(live(&driver, fourth).waiting_on.is_empty());
        assert!(matches!(live(&driver, unrelated.task).phase,
            tasks::Phase::Active(tasks::Active::Running { attempt }) if attempt == unrelated.attempt));
    }
}

#[test]
fn a_live_delegate_is_kept_until_both_added_dependencies_are_met() {
    let (mut driver, parent) = batch_fixture_with(4, 1);
    let children = call_batch(
        &mut driver,
        &parent,
        910,
        Box::new([
            report_delegate(b"a3", Box::new([])),
            report_delegate(b"e1", Box::new([])),
            report_delegate(b"r", Box::new([])),
        ]),
    );
    let [a3, e1, r] = *children else { panic!("three delegates") };
    let a3_run = assigned_task(&driver, a3);
    let e1_run = assigned_task(&driver, e1);
    let r_run = assigned_task(&driver, r);
    let mut amendment = task_amendment(None, None, b"wait for landing and repin");
    amendment.dependencies = Some(Box::new([e1, r]));
    let revision = live(&driver, a3).revisions.spec;
    let at = driver.env.wall;
    let amend = engine::Tool::Amend { keep_parameters: false, target: a3, revision, amendment };
    tool_call(&mut driver, &parent, 911, amend);
    assert_eq!(
        driver.answered(911),
        Some(&engine::CallAnswer::Amended {
            task: a3,
            revision: revision + 1,
            changed: engine::Changed { tracked: None, fields: engine::Changed::DEPENDENCIES }
        })
    );
    let row = live(&driver, a3);
    assert_eq!(row.dependencies.as_ref(), [e1, r]);
    assert_eq!(
        row.added.as_ref(),
        [
            tasks::Added { task: e1, at, by: tasks::Party::Task(parent.task) },
            tasks::Added { task: r, at, by: tasks::Party::Task(parent.task) },
        ]
    );
    finish_report(&mut driver, &a3_run, b"first turn done");
    let row = live(&driver, a3);
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(row.waiting_on.as_ref(), [e1, r]);
    assert_eq!(assignments(&driver, a3), 1);

    // e1 is a report delegate here; the forge world owns settled landing.
    finish_report(&mut driver, &e1_run, b"e1 landed");
    let row = live(&driver, a3);
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(row.waiting_on.as_ref(), [r]);
    assert_eq!(assignments(&driver, a3), 1);

    finish_report(&mut driver, &r_run, b"app repinned");
    let row = live(&driver, a3);
    assert!(row.waiting_on.is_empty());
    assert!(row.inbox.iter().any(|word| word.kind == tasks::MessageKind::Met { dependency: r, removed: false }));
    assert_eq!(assignments(&driver, a3), 2);
    let resumed = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::Assigned { assignment, .. } = delivery
                && assignment.task == a3
            {
                Some(assignment)
            } else {
                None
            }
        })
        .expect("a3 resumed after Met");
    assert!(resumed.attempt > a3_run.attempt);
    let results = resumed
        .sections
        .iter()
        .find_map(|section| match (&section.kind, &section.body) {
            (engine::BriefKind::Core(brief::Core::Results), engine::BriefBody::Text(words)) => Some(words),
            _ => None,
        })
        .expect("dependency results in the resumed brief");
    let results = std::str::from_utf8(results).expect("brief text");
    assert!(results.contains("e1 landed") && results.contains("app repinned"), "{results}");
}

#[test]
fn a_kept_delegate_resumes_with_its_committed_turns_and_its_journal_whole() {
    const COMMITTED: &[u8] = b"kept report: the sibling must finish before I go on";
    let (mut configuration, bounds) = batch_settings(3, 1, 500, false, false, false);
    for config in &mut configuration {
        config.core.defaults.charters[0].policy.resume = true;
    }
    let mut driver = Driver::configured(Store::new(), configuration, &bounds);
    driver.send(hello_with(Token::new(7), 3, Duration::from_secs(1), Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let parent = assigned(&driver);
    let children = call_batch(
        &mut driver,
        &parent,
        923,
        Box::new([report_delegate(b"kept delegate", Box::new([])), report_delegate(b"sibling", Box::new([]))]),
    );
    let [kept, sibling] = *children else { panic!("two delegates") };
    let first = assigned_task(&driver, kept);
    let sibling_run = assigned_task(&driver, sibling);
    let mut amendment = task_amendment(None, None, b"wait for the sibling");
    amendment.dependencies = Some(Box::new([sibling]));
    let revision = live(&driver, kept).revisions.spec;
    tool_call(
        &mut driver,
        &parent,
        924,
        engine::Tool::Amend { keep_parameters: false, target: kept, revision, amendment },
    );
    assert_eq!(live(&driver, kept).dependencies.as_ref(), [sibling]);
    let read = live(&driver, kept).inbox.last().map(|word| word.number);
    driver.send(protocol::turn(Token::new(7), kept, first.attempt, 1, 1, read, COMMITTED));
    driver.settle();
    driver.answer(
        Token::new(7),
        kept,
        first.attempt,
        1,
        tasks::End::Finished { result: tasks::TaskResult::Report { words: COMMITTED.into() }, cancel_delegates: false },
        engine::Parts::none(),
    );
    driver.settle();
    assert_eq!(live(&driver, kept).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(live(&driver, kept).waiting_on.as_ref(), [sibling]);
    assert_eq!(assignments(&driver, kept), 1);
    let milestones = |driver: &Driver| -> Vec<tasks::Milestone> {
        driver
            .store
            .rows
            .values()
            .filter_map(|row| match row {
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Milestone(row))) if row.task == kept => {
                    Some(row.clone())
                }
                _ => None,
            })
            .collect()
    };
    let before = milestones(&driver);
    assert_eq!(before.last().expect("kept lifecycle journal").phase, tasks::Phase::Active(tasks::Active::Idle));
    finish_report(&mut driver, &sibling_run, b"sibling done");
    assert_eq!(assignments(&driver, kept), 2);
    let resumed = driver.last_assignment();
    assert_eq!(resumed.task, kept);
    assert!(resumed.attempt > first.attempt);
    assert_eq!(resumed.transcript.as_ref(), [Box::<[u8]>::from(COMMITTED)]);
    assert_eq!(
        resumed
            .messages
            .iter()
            .filter(|word| word.kind == tasks::MessageKind::Met { dependency: sibling, removed: false })
            .count(),
        1
    );
    let after = milestones(&driver);
    assert!(after.len() > before.len(), "the next attempt adds lifecycle milestones");
    assert!(after.starts_with(&before), "the kept journal remains an unchanged prefix: {before:?} -> {after:?}");
}

#[test]
fn an_added_dependency_that_closes_a_circle_is_refused() {
    let (mut driver, parent) = batch_fixture_with(3, 1);
    let children = call_batch(
        &mut driver,
        &parent,
        912,
        Box::new([
            report_delegate(b"a", Box::new([])),
            report_delegate(b"b", Box::new([engine::Dependency::Batch(0)])),
        ]),
    );
    let [a, b] = *children else { panic!("two delegates") };
    let mut amendment = task_amendment(None, None, b"wait for b");
    amendment.dependencies = Some(Box::new([b]));
    let revision = live(&driver, a).revisions.spec;
    tool_call(
        &mut driver,
        &parent,
        913,
        engine::Tool::Amend { keep_parameters: false, target: a, revision, amendment },
    );
    let Some(engine::CallAnswer::ControlRefused(problem)) = driver.answered(913) else {
        panic!("circle refused: {:?}", driver.answered(913))
    };
    // This domain world answers with call-name bytes; smith tests the rendered words.
    assert_eq!(problem.why, tasks::Refusal::Circle { cut: false });
    assert_eq!(problem.blocked_by.as_deref(), Some([a, b].as_slice()));
    assert!(live(&driver, a).dependencies.is_empty());
    assert_eq!(live(&driver, a).revisions.spec, revision);
}
