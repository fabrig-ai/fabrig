//! The root composes milestone totals, without reservations (domain/milestones.md, 7).
use super::*;
use temper_engine_domain_forge as forge;

fn assigned_task(driver: &Driver, task: u64) -> engine::Assignment {
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| match delivery {
            Delivery::Assigned { assignment, .. } if assignment.task == task => Some(assignment.clone()),
            _ => None,
        })
        .expect("latest task assignment")
}

fn advance(driver: &mut Driver) {
    for _ in 0..30 {
        driver.advance(true);
    }
}

fn ask(driver: &mut Driver, key: u8, ask: people::Ask) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        ask,
    }));
    advance(driver);
    assert!(driver.delivered.iter().any(|d| matches!(d, Delivery::WebReply { to, reply: people::Reply::Outcome(_), .. } if *to == ReplyTo::new(Token::new(u64::from(key))))), "{:?}", driver.delivered);
}

fn member(driver: &Driver, words: &[u8], spend: u64) -> people::Member {
    let people::Ask::Create { batch, .. } = driver.chat_ask(1, words) else { panic!("create fixture") };
    let mut member = batch[0].clone();
    member.budget = people::Budget::Named;
    member.authority.spend = spend;
    member.authority.delegation =
        people::Delegation { kinds: Box::new([people::Executor::Charter(1)]), tasks: 1, depth: 1 };
    member.contract = people::Contract::Report { words: 128 };
    member.flags = people::Flags::PLAIN;
    member.tracked = Some(0);
    member.label = 2;
    member.wake = Some(people::WakePolicy {
        words: people::WakeRule::Immediate,
        notices: people::WakeRule::Immediate,
        news: people::WakeRule::Immediate,
        results: people::ResultsWake::LastOrFailure,
        questions: true,
        answers: true,
        timers: true,
    });
    member
}

fn fixture() -> Driver {
    let (mut configs, mut bounds) = batch_settings(1, 2, 500, false, true, false);
    bounds.tasks.tree_tasks = 6;
    bounds.tasks.project_goals = 4;
    bounds.tasks.inbox_messages = 8;
    bounds.people.inbox_recipients = 21;
    bounds.people.requests = 8;
    bounds.journal.transcript_bytes = 131_072;
    bounds.people.projects = 3;
    bounds.people.initial_owners = 2;
    bounds.authority.projects = 3;
    bounds.forge.judge_projects = 3;
    for config in &mut configs {
        let kinds: Box<[authority::Executor]> =
            Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(jig_domain_core::GATHERING)]);
        config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
        config.core.defaults.rules.ceiling.delegation.tasks = 6;
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.kinds.clone_from(&kinds);
        policy.ceiling.delegation.tasks = 6;
        policy.roles[0].authority.delegation.kinds = kinds;
        policy.roles[0].authority.delegation.tasks = 6;
        let second = jig_domain_core::SeedProject {
            number: 2,
            name: "Project 2".into(),
            policy: config.core.seed_projects[0].policy.clone(),
        };
        let first = std::mem::replace(&mut config.core.seed_projects, Box::new([])).into_vec().remove(0);
        config.core.seed_projects = Box::new([first, second]);
        config.core.owners = Box::new([
            people::InitialOwner { project: 1, identity: forge::user_key(1, 7_u64) },
            people::InitialOwner { project: 2, identity: forge::user_key(1, 7_u64) },
        ]);
    }
    let restart = temper_engine_domain_world::oracle::config::copy(&configs[0]);
    let mut driver = Driver::configured(Store::new(), configs, &bounds);
    // A kept forge row supplies a last landing independently of jig's task records.
    driver.store.seed(
        Box::new([Record::Forge {
            row: Box::new(forge::Stored::Milestone(forge::MilestoneRow {
                project: 1,
                task: 1,
                name: b"Store shape".as_slice().into(),
                last_landing: Some(landing()),
                ended: None,
            })),
        }]),
        Box::default(),
    );
    driver.reopen(restart);
    driver.settle();
    driver.hello(Token::new(7));
    driver.settle();
    driver.sign_in();
    driver.settle();
    let mut gathering = member(&driver, b"Store shape", 100);
    gathering.executor =
        people::TaskExecutor::Procedure { connector: jig_domain_core::RECURRING, code: jig_domain_core::GATHERING };
    gathering.authority.delegation.tasks = 4;
    gathering.authority.delegation.depth = 2;
    gathering.flags.ended_by = people::EndedBy::Requester;
    gathering.label = 8;
    ask(&mut driver, 80, people::Ask::Create { project: 1, batch: Box::new([gathering]) });
    assert!(tasks::gathering::executor(driver.task(1).expect("party made gathering").executor));
    let mut goals = [b"done".as_slice(), b"failed", b"live"].map(|words| member(&driver, words, 20));
    for goal in &mut goals[..2] {
        goal.authority.delegation = people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    }
    ask(&mut driver, 81, people::Ask::Delegate { project: 1, task: 1, batch: Box::new(goals) });
    assert_eq!(driver.task(1).expect("gathering").delegates.as_ref(), &[2, 3, 4], "{:?}", driver.delivered);
    driver
}

fn landing() -> forge::MilestoneLanding {
    forge::MilestoneLanding { at: Wall::from_nanos(31), goal: 2, change: 91 }
}

fn read(driver: &mut Driver, project: u32, task: u64) -> Option<forge::MilestoneTotals> {
    driver.send(engine::Event::Party(engine::PartyInput::ForgeRead {
        reply: ReplyTo::new(Token::new(900)),
        sign_in: driver.session(),
        scope: people::ApplicationScope::Project(project),
        query: engine::ForgeRead::Milestone { project, task },
    }));
    advance(driver);
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| match delivery {
            Delivery::ForgeRead { answer: engine::ForgeReadAnswer::Milestone(value), .. } => Some(value.clone()),
            _ => None,
        })
        .expect("milestone answer")
}

fn finish(driver: &mut Driver, task: u64, spent: u64, result: tasks::TaskResult) {
    let run = assigned_task(driver, task);
    driver.answer(
        Token::new(7),
        task,
        run.attempt,
        spent,
        tasks::End::Finished { result, cancel_delegates: false },
        engine::Parts::none(),
    );
    advance(driver);
}

#[test]
fn milestone_totals_sum_settled_and_live_spend_then_keep_the_ended_row() {
    let mut driver = fixture();
    finish(&mut driver, 2, 2, tasks::TaskResult::Report { words: b"done".as_slice().into() });
    finish(&mut driver, 3, 3, tasks::TaskResult::Failure { reason: b"failed".as_slice().into() });
    let live = assigned_task(&driver, 4);
    let mut child = plain_delegate(b"settled below live goal");
    child.authority.budget.spend = 5;
    tool_call(&mut driver, &live, 100, engine::Tool::Delegate { batch: Box::new([child]) });
    assert!(matches!(driver.answered(100), Some(engine::CallAnswer::Delegated(_))), "{:?}", driver.answered(100));
    driver.answer(Token::new(7), 4, live.attempt, 2, tasks::End::Parked, engine::Parts::none());
    advance(&mut driver);
    finish(&mut driver, 5, 1, tasks::TaskResult::Report { words: b"below".as_slice().into() });
    let resumed = assigned_task(&driver, 4);
    assert!(resumed.attempt > live.attempt, "the live goal resumes after its delegate ends");
    let live = resumed;
    driver.send(protocol::turn(Token::new(7), 4, live.attempt, 1, 1, None, b"live spend"));
    advance(&mut driver);
    assert_eq!(driver.task(1).unwrap().numbers.spent_below, 5);
    assert_eq!(driver.task(4).unwrap().numbers.spent_below, 1);
    assert!(driver.task(1).unwrap().numbers.reserved > 0);
    assert!(driver.task(4).unwrap().numbers.reserved > 0);
    let expected = forge::MilestoneTotals {
        task: 1,
        done: 1,
        failed: 1,
        closed: 0,
        live: 1,
        ended: None,
        last_landing: Some(landing()),
        spent: 9,
    };
    assert_eq!(read(&mut driver, 1, 1), Some(expected));
    ask(&mut driver, 82, people::Ask::Conclude { project: 1, task: 1 });
    driver.env.wall = Wall::from_nanos(driver.env.wall.as_nanos() + 1);
    finish(&mut driver, 4, 1, tasks::TaskResult::Report { words: b"last".as_slice().into() });
    let Record::Forge { row } = driver.store.rows.get(&Key::Forge(forge::Key::Milestone(1))).unwrap() else {
        panic!("row")
    };
    let forge::Stored::Milestone(row) = row.as_ref() else { panic!("milestone row") };
    let end = row.ended.as_ref().expect("concluded and last goal ended");
    assert_eq!((end.done, end.failed, end.closed, end.spent), (2, 1, 0, 9));
    let at = end.at;
    assert_eq!(
        read(&mut driver, 1, 1),
        Some(forge::MilestoneTotals {
            task: 1,
            done: 2,
            failed: 1,
            closed: 0,
            live: 0,
            ended: Some(at),
            last_landing: Some(landing()),
            spent: 9,
        })
    );
}

#[test]
fn milestone_reads_reject_other_tasks_projects_and_parties_without_a_role() {
    let mut driver = fixture();
    assert_eq!(read(&mut driver, 1, 2), None, "an agent is not a milestone");
    assert_eq!(read(&mut driver, 2, 1), None, "a milestone of another project");
    assert_eq!(read(&mut driver, 1, 999), None, "no milestone record");
    driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
        until: None,
        held: None,
        reply_to: ReplyTo::new(Token::new(901)),
        identity: people::Identity {
            key: forge::user_key(1, 8_u64),
            login: b"stranger".as_slice().into(),
            name: b"Stranger".as_slice().into(),
        },
    }));
    advance(&mut driver);
    let (_, stranger) = signed_in(&driver, 901);
    driver.send(engine::Event::Party(engine::PartyInput::ForgeRead {
        reply: ReplyTo::new(Token::new(902)),
        sign_in: stranger,
        scope: people::ApplicationScope::Project(1),
        query: engine::ForgeRead::Milestone { project: 1, task: 1 },
    }));
    advance(&mut driver);
    assert!(driver.delivered.iter().any(|d| matches!(d, Delivery::Read { reply, answer: jig_domain_core::reads::Answer::None(jig_domain_core::reads::Why::Unknown) } if *reply == ReplyTo::new(Token::new(902)))));
    driver.send(engine::Event::Party(engine::PartyInput::ForgeRead {
        reply: ReplyTo::new(Token::new(903)),
        sign_in: driver.session(),
        scope: people::ApplicationScope::Project(1),
        query: engine::ForgeRead::Milestone { project: 2, task: 1 },
    }));
    advance(&mut driver);
    assert!(driver.delivered.iter().any(|d| matches!(d, Delivery::Read { reply, answer: jig_domain_core::reads::Answer::None(jig_domain_core::reads::Why::Unknown) } if *reply == ReplyTo::new(Token::new(903)))));
}
