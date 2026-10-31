//! The director, P and its implementers (temper/docs/design/domain/testing.md, 2.2).
use super::*;

struct Cast {
    director: engine::Assignment,
    p: engine::Assignment,
    implementers: [engine::Assignment; 2],
}

fn opening() -> (Driver, Cast) {
    // Four simultaneously running members, with distinct coordinator/implementer charters.
    const CAST_WRITES: u32 = 6_200;
    let (mut configuration, mut bounds) = batch_settings(4, 2, 500, false, false, false);
    bounds.tasks.charters = 2;
    bounds.tasks.executor_kinds = 2;
    bounds.authority.executors = 2;
    bounds.documents.executors = 2;
    bounds.journal.writes = CAST_WRITES;
    for config in &mut configuration {
        let mut coordinator = config.core.defaults.charters[0].clone();
        coordinator.policy.model.name = b"claude".as_slice().into();
        coordinator.policy.instructions = b"Coordinate the work your requester entrusted to you.".as_slice().into();
        let mut implementer = coordinator.clone();
        implementer.number = 2;
        implementer.policy.model.name = b"gpt".as_slice().into();
        config.core.settings.models = Box::new([coordinator.policy.model.clone(), implementer.policy.model.clone()]);
        config.core.settings.llm_providers.providers[0].models =
            Box::new([Box::from(b"claude".as_slice()), Box::from(b"gpt".as_slice())]);
        config.core.defaults.charters = Box::new([coordinator, implementer]);
        let kinds: Box<[_]> = Box::new([authority::Executor::Charter(1), authority::Executor::Charter(2)]);
        config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.kinds.clone_from(&kinds);
        policy.roles[0].authority.delegation.kinds = kinds;
    }
    let mut driver = Driver::configured(Store::new(), configuration, &bounds);
    let mut event = hello_with(Token::new(7), 4, Duration::from_secs(1), Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        unreachable!("worker hello")
    };
    hello.bounds.models = Box::new([b"claude".as_slice(), b"gpt".as_slice()].map(|name| host::ModelBound {
        name: name.into(),
        window: 1024,
        output: 1024,
    }));
    driver.send(event);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let director = assigned(&driver);
    let mut p = report_delegate(b"P: own Themes", Box::new([]));
    p.tracked = Some(2);
    p.authority.budget.spend = 40;
    p.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(2)]), tasks: 2, depth: 1 };
    let p = call_batch(&mut driver, &director, 930, Box::new([p]))[0];
    let p = assigned_task(&driver, p);
    let mut batch = Box::new([
        report_delegate(b"P/i1: first part of Themes", Box::new([])),
        report_delegate(b"P/i2: second part of Themes", Box::new([])),
    ]);
    for implementer in batch.iter_mut() {
        implementer.executor = tasks::Executor::Agent { charter: 2 };
    }
    let children = call_batch(&mut driver, &p, 931, batch);
    let implementers = [assigned_task(&driver, children[0]), assigned_task(&driver, children[1])];
    (driver, Cast { director, p, implementers })
}

#[test]
fn the_director_p_and_both_implementers_start_assigned_without_holds_or_findings() {
    let (driver, cast) = opening();
    let person = signed_in(&driver, 101).0;
    assert_eq!(live(&driver, cast.director.task).requester, tasks::Party::Person(person));
    assert_eq!(live(&driver, cast.p.task).requester, tasks::Party::Task(cast.director.task));
    assert_eq!(live(&driver, cast.p.task).tracked, Some(2));
    assert_eq!(live(&driver, cast.p.task).authority.delegation.tasks, 2);
    assert_eq!(cast.director.run.policy.model.name.as_ref(), b"claude");
    assert_eq!(cast.p.run.policy.model.name.as_ref(), b"claude");
    for run in &cast.implementers {
        let row = live(&driver, run.task);
        assert_eq!(row.requester, tasks::Party::Task(cast.p.task));
        assert_eq!(row.authority.delegation.depth, 0);
        assert_eq!(row.authority.delegation.tasks, 0);
        assert_eq!(row.executor, tasks::Executor::Agent { charter: 2 });
        assert_eq!(run.run.policy.model.name.as_ref(), b"gpt");
    }
    for run in [&cast.director, &cast.p, &cast.implementers[0], &cast.implementers[1]] {
        let row = live(&driver, run.task);
        assert_eq!(row.root, cast.director.task);
        assert!(
            matches!(row.phase, tasks::Phase::Active(tasks::Active::Running { attempt }) if attempt == run.attempt)
        );
        assert_eq!(row.hold, 0);
        assert!(row.proposal.is_none());
        assert!(matches!(row.escalation, tasks::Escalation::Unheld { revision: 0 }));
    }
    assert!(
        !driver.store.rows.values().any(|row| matches!(row, Record::Forge { .. })),
        "the opening creates no forge finding"
    );
}

fn person_ask(driver: &mut Driver, key: u8, ask: people::Ask) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        ask,
    }));
    driver.settle();
}

#[test]
fn the_persons_stop_requires_its_owners_acceptance_and_resumes_the_subtrees_transcripts() {
    let (mut driver, cast) = opening();
    let runs = [&cast.p, &cast.implementers[0], &cast.implementers[1]];
    for run in runs {
        driver.send(protocol::turn(Token::new(7), run.task, run.attempt, 1, 1, None, &run.task.to_be_bytes()));
        driver.settle();
    }
    person_ask(&mut driver, 80, people::Ask::Stop { project: 1, task: cast.p.task });
    assert!(replied(&driver, &people::Outcome::Stopped { task: cast.p.task }));
    let hold = live(&driver, cast.p.task).placed.expect("person's subtree hold");
    for run in runs {
        assert_eq!(live(&driver, run.task).placed, Some(hold));
        assert!(matches!(live(&driver, run.task).phase,
            tasks::Phase::Held { why: tasks::Hold::Placed { hold: found }, .. } if found == hold));
        assert!(cancelled(&driver).contains(&(run.task, run.attempt)));
        driver.answer(Token::new(7), run.task, run.attempt, 1, tasks::End::Parked, engine::Parts::none());
        driver.settle();
    }
    assert_eq!(live(&driver, cast.director.task).placed, None);
    assert!(!cancelled(&driver).contains(&(cast.director.task, cast.director.attempt)));
    let revision = live(&driver, cast.p.task).revisions.hold;
    tool_call(&mut driver, &cast.director, 932, engine::Tool::Release { target: cast.p.task, revision, again: None });
    let person = signed_in(&driver, 101).0;
    assert_eq!(
        driver.answered(932),
        Some(&engine::CallAnswer::HoldOwner {
            task: cast.p.task,
            hold,
            revision,
            owner: people::HoldOwner::Party { person },
        })
    );
    tool_call(
        &mut driver,
        &cast.director,
        933,
        engine::Tool::Propose {
            action: engine::ProposedAction::Release { task: cast.p.task, hold, revision },
            reason: b"Use every free slot".as_slice().into(),
            as_holder: false,
        },
    );
    let proposal = proposal_of(&driver, 933);
    assert!(proposal_waits_at(&driver, cast.director.task, tasks::ProposalHolder::Person(person)));
    for run in runs {
        assert_eq!(
            driver
                .delivered
                .iter()
                .filter(|item| matches!(item,
            Delivery::Assigned { assignment, .. } if assignment.task == run.task))
                .count(),
            1
        );
    }
    person_ask(
        &mut driver,
        81,
        people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Task(cast.director.task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: None,
        },
    );
    assert!(driver.delivered.iter().any(|item| matches!(item,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::ProposalDecided {
            proposal: found, choice: people::ProposalChoice::Accepted, .. }), .. } if *found == proposal)));
    for run in runs {
        let resumed = driver
            .delivered
            .iter()
            .rev()
            .find_map(|item| {
                if let Delivery::Assigned { assignment, .. } = item
                    && assignment.task == run.task
                {
                    Some(assignment)
                } else {
                    None
                }
            })
            .expect("subtree resumes");
        assert!(resumed.attempt > run.attempt);
        assert_eq!(resumed.transcript.as_ref(), [Box::<[u8]>::from(run.task.to_be_bytes())]);
        assert_eq!(live(&driver, run.task).placed, None);
        assert!(matches!(live(&driver, run.task).phase,
            tasks::Phase::Active(tasks::Active::Running { attempt }) if attempt == resumed.attempt));
    }
}
