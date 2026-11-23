//! Owner plans and sub-owners (temper/docs/design/testing/reference-case.md).
use super::*;

fn owner(words: &[u8], depth: u32, spend: u64) -> engine::Delegate {
    let mut member = report_delegate(words, Box::new([]));
    member.authority.budget.spend = spend;
    member.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth };
    member
}

fn finish(driver: &mut Driver, run: &engine::Assignment, words: &[u8]) {
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

#[test]
fn three_owners_end_with_their_own_reports() {
    let (mut driver, director) = batch_fixture_with(3, 2);
    let a = call_batch(&mut driver, &director, 950, Box::new([owner(b"A", 1, 30)]))[0];
    let a = assigned_task(&driver, a);
    let a1 = call_batch(&mut driver, &a, 951, Box::new([report_delegate(b"A1", Box::new([]))]))[0];
    let a1 = assigned_task(&driver, a1);
    for (run, parent, words) in [
        (&a1, Some(a.task), b"M1 reported".as_slice()),
        (&a, Some(director.task), b"mission reported".as_slice()),
        (&director, None, b"direction reported".as_slice()),
    ] {
        finish(&mut driver, run, words);
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
            driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(run.task))))
        else {
            panic!("owner {} ended", run.task)
        };
        assert_eq!(
            row.phase,
            tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: words.into() }))
        );
        assert_eq!(row.project, 1);
        if let Some(parent) = parent {
            assert_eq!(row.requester, tasks::Party::Task(parent));
        }
    }
}

#[test]
fn a_tracked_sub_owner_carries_an_earlier_deadline_than_its_owner() {
    let (mut driver, director) = batch_fixture_with(3, 2);
    let later = Wall::from_nanos(Duration::from_secs(60).as_nanos());
    let earlier = Wall::from_nanos(Duration::from_secs(30).as_nanos());
    let mut a = owner(b"A", 1, 30);
    a.tracked = Some(1);
    a.authority.budget.deadline = Some(later);
    let a = call_batch(&mut driver, &director, 950, Box::new([a]))[0];
    let a_run = assigned_task(&driver, a);
    let mut a1 = report_delegate(b"A1", Box::new([]));
    a1.tracked = Some(1);
    a1.authority.budget.deadline = Some(earlier);
    let a1 = call_batch(&mut driver, &a_run, 951, Box::new([a1]))[0];
    for (task, deadline) in [(a, later), (a1, earlier)] {
        let row = live(&driver, task);
        assert_eq!(row.authority.budget.deadline, Some(deadline));
        assert_eq!(row.tracked, Some(1));
    }
    assert!(earlier < later);
    assert!(driver.env.wall < earlier, "neither deadline is fired");
}

#[test]
fn a_zero_depth_delegate_is_refused_for_depth_alone_without_a_child() {
    // Both sides of the boundary keep task capacity, executor, and funding unchanged.
    for depth in [0, 1] {
        let (mut driver, director) = batch_fixture_with(3, 2);
        let a1 = call_batch(&mut driver, &director, 950, Box::new([owner(b"A1", depth, 30)]))[0];
        let a1_run = assigned_task(&driver, a1);
        assert_eq!(live(&driver, a1).authority.delegation.depth, depth);
        let before = driver
            .store
            .rows
            .values()
            .filter(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_)))))
            .count();
        tool_call(
            &mut driver,
            &a1_run,
            951,
            engine::Tool::Delegate { batch: Box::new([report_delegate(b"helper", Box::new([]))]) },
        );
        if depth == 0 {
            let lacks = authority::Lacks {
                tools: false,
                grants: false,
                executors: false,
                tasks: false,
                depth: true,
                spend: false,
                deadline: false,
                notes: false,
                models: false,
            };
            assert_eq!(
                driver.answered(951),
                Some(&engine::CallAnswer::DelegationDenied {
                    answer: authority::Answer::Propose,
                    findings: Box::new([authority::Finding::Authority { source: authority::Source::Task, lacks }]),
                })
            );
            assert_eq!(
                driver
                    .store
                    .rows
                    .values()
                    .filter(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_)))))
                    .count(),
                before
            );
            assert!(!driver.store.rows.values().any(|row| matches!(row,
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
                    if row.requester == tasks::Party::Task(a1)
            )));
        } else {
            assert!(matches!(driver.answered(951), Some(engine::CallAnswer::Delegated(made)) if made.len() == 1));
        }
    }
}

#[test]
fn an_answer_starts_a_fresh_owner_that_recalls_its_plan_with_ordered_successful_attempts() {
    const PLAN: &[u8] = b"M1: build; M2: check; ask for B's files";
    let (mut configs, bounds) = batch_settings(2, 1, 500, false, false, false);
    for config in &mut configs {
        config.core.defaults.charters[0].policy.resume = false;
        config.core.defaults.rules.ceiling.notes = authority::Scopes(7);
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.notes = authority::Scopes(7);
        policy.roles[0].authority.notes = authority::Scopes(7);
    }
    let mut driver = Driver::configured(Store::new(), configs, &bounds);
    driver.send(hello_with(Token::new(7), 2, Duration::from_secs(1), Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let director = assigned(&driver);
    let mut a = report_delegate(b"A", Box::new([]));
    a.authority.notes = tasks::Scopes(7);
    a.authority.budget.spend = 30;
    let a = call_batch(&mut driver, &director, 950, Box::new([a]))[0];
    let first = assigned_task(&driver, a);
    tool_call(
        &mut driver,
        &first,
        951,
        new_note(b"mission plan", PLAN, notes::Author::Task { task: a, attempt: first.attempt }),
    );
    let note = written_note_name(&driver, 951);
    tool_call(
        &mut driver,
        &first,
        952,
        engine::Tool::Message { target: director.task, form: engine::MessageForm::Question, words: PLAN.into() },
    );
    let Some(engine::CallAnswer::Sent { message: question }) = driver.answered(952).cloned() else {
        panic!("question sent")
    };
    driver.send(protocol::turn(Token::new(7), a, first.attempt, 1, 0, None, b"old planning turn"));
    driver.settle();
    driver.answer(Token::new(7), a, first.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    assert_eq!(live(&driver, a).phase, tasks::Phase::Active(tasks::Active::Idle));
    let before = attempts(&driver, a);
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].end, Some(tasks::EndKind::Parked));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(953)),
        sign_in: driver.session(),
        key: [95; 16],
        ask: people::Ask::AnswerQuestion {
            project: 1,
            task: director.task,
            question,
            words: b"green light".as_slice().into(),
        },
    }));
    driver.settle();
    let fresh = driver.last_assignment();
    assert_eq!(fresh.task, a);
    assert!(fresh.attempt > first.attempt);
    assert!(!fresh.run.policy.resume);
    assert!(fresh.transcript.is_empty());
    assert_eq!(fresh.messages.iter().filter(|word| word.kind == tasks::MessageKind::Answer { question }).count(), 1);
    tool_call(&mut driver, &fresh, 954, engine::Tool::Recall { by: notes::Recall::Name { name: note }, page: 0 });
    assert!(matches!(driver.answered(954), Some(engine::CallAnswer::NoteRecalled { entries, more: false })
        if entries.len() == 1 && entries[0].body.as_ref() == PLAN && entries[0].revision == 1));
    finish(&mut driver, &fresh, b"plan recalled");
    let after = attempts(&driver, a);
    assert_eq!(after.len(), 2);
    assert_eq!(&after[..1], before);
    assert_eq!(after[1].end, Some(tasks::EndKind::Finished));
    assert!(after.windows(2).all(|pair| pair[0].attempt < pair[1].attempt
        && pair[0].ended.zip(pair[1].claimed).is_some_and(|(ended, claimed)| ended <= claimed)));
}

fn attempts(driver: &Driver, task: u64) -> Vec<tasks::AttemptRow> {
    driver
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row))) if row.task == task => {
                Some(row.clone())
            }
            _ => None,
        })
        .collect()
}
