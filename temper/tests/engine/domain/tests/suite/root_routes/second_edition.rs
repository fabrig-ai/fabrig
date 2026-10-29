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
