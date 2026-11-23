//! The opener's policy and ordering, below its three-worker real-loop seam
//! (temper/docs/design/testing/checks.md, 1).
use super::*;

const CHANNEL: Token = Token::new(7);
const POOL: u64 = 500; // The batch fixture's person pool.
const SLOTS: u32 = 3; // Director, owner and its delegate, all live.
const DEPTH: u32 = 2; // Two delegation edges below the director.
const REASON: &[u8] = b"the check found the benchmark stalled";

fn kit_grant(last: tasks::Last) -> tasks::Grant {
    tasks::Grant {
        connector: 0,
        kind: 1,
        pattern: tasks::Pattern { segments: Box::new([b"kit".as_slice().into()]), last },
    }
}

fn subtree() -> (Driver, engine::Assignment, engine::Assignment, engine::Assignment) {
    let (mut configuration, bounds) = batch_settings(SLOTS, DEPTH, POOL, false, false, false);
    for config in &mut configuration {
        let grants: Box<[authority::Grant]> = Box::new([authority::Grant {
            connector: 0,
            kind: 1,
            pattern: authority::Pattern {
                segments: Box::new([]),
                last: authority::Last::Open(b"kit".as_slice().into()),
            },
        }]);
        config.core.defaults.rules.ceiling.grants.clone_from(&grants);
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.grants.clone_from(&grants);
        policy.roles[0].authority.grants = grants;
    }
    let mut driver = Driver::configured(Store::new(), configuration, &bounds);
    driver.send(hello_with(CHANNEL, SLOTS, Duration::from_secs(1), Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let director = assigned(&driver);
    let mut owner = report_delegate(b"owner", Box::new([]));
    owner.tracked = Some(1);
    owner.authority.budget.spend = 20;
    owner.authority.tools = tasks::Tools(7);
    owner.authority.grants = Box::new([kit_grant(tasks::Last::Open(b"doc".as_slice().into()))]);
    owner.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth: 1 };
    let owner = call_batch(&mut driver, &director, 90, Box::new([owner]))[0];
    let owner = assigned_task(&driver, owner);
    let mut delegate = report_delegate(b"delegate", Box::new([]));
    delegate.authority.tools = tasks::Tools(2);
    delegate.authority.grants = Box::new([kit_grant(tasks::Last::Exact(b"docs.md".as_slice().into()))]);
    let delegate = call_batch(&mut driver, &owner, 91, Box::new([delegate]))[0];
    let delegate = assigned_task(&driver, delegate);
    (driver, director, owner, delegate)
}

#[test]
fn a_subtree_within_its_owners_authority_is_admitted() {
    let (driver, director, owner, delegate) = subtree();
    let parent = live(&driver, director.task);
    let owner = live(&driver, owner.task);
    let delegate = live(&driver, delegate.task);
    assert_eq!(owner.requester, tasks::Party::Task(parent.number));
    assert_eq!(delegate.requester, tasks::Party::Task(owner.number));
    assert_eq!(owner.authority.tools, tasks::Tools(7));
    assert_eq!(delegate.authority.tools, tasks::Tools(2));
    assert_eq!(owner.authority.budget.spend, 20);
    assert_eq!(delegate.authority.budget.spend, 10);
    assert_eq!(
        owner.authority.delegation,
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth: 1 }
    );
    assert_eq!(delegate.authority.delegation, tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 });
    assert_eq!(owner.authority.grants[0].pattern.last, tasks::Last::Open(b"doc".as_slice().into()));
    assert_eq!(delegate.authority.grants.as_ref(), [kit_grant(tasks::Last::Exact(b"docs.md".as_slice().into()))]);
}

#[test]
fn a_subtrees_spend_rolls_up_to_its_goal() {
    let (mut driver, director, owner, delegate) = subtree();
    turn(&mut driver, &owner, 1, 2);
    driver.settle();
    turn(&mut driver, &delegate, 1, 3);
    driver.settle();
    driver.answer(
        CHANNEL,
        delegate.task,
        delegate.attempt,
        3,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    driver.settle();
    assert_eq!(live(&driver, owner.task).tracked, Some(1));
    assert_eq!(live(&driver, owner.task).numbers.spent, 2);
    assert_eq!(live(&driver, owner.task).numbers.spent_below, 3);
    driver.answer(
        CHANNEL,
        owner.task,
        owner.attempt,
        2,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"subtree done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    driver.settle();
    assert_eq!(live(&driver, director.task).numbers.spent_below, 5);
}

#[test]
fn an_introduced_ancestor_cancels_a_run_before_its_owner_reports() {
    let (mut driver, director, owner, delegate) = subtree();
    tool_call(&mut driver, &owner, 92, engine::Tool::Introduce { left: director.task, right: delegate.task });
    assert_eq!(driver.answered(92), Some(&engine::CallAnswer::Introduced));
    assert!(live(&driver, delegate.task).references.contains(&director.task));
    assert!(live(&driver, director.task).references.contains(&delegate.task));
    tool_call(&mut driver, &director, 94, engine::Tool::Cancel { target: delegate.task, reason: REASON.into() });
    assert_eq!(driver.answered(94), Some(&engine::CallAnswer::Controlled));
    assert_eq!(cancelled(&driver), [(delegate.task, delegate.attempt)]);
    driver.answer(CHANNEL, delegate.task, delegate.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    let message = live(&driver, owner.task)
        .inbox
        .iter()
        .find(|word| {
            word.from == tasks::Party::Task(delegate.task)
                && word.kind == tasks::MessageKind::Result(tasks::ResultKind::Cancelled)
        })
        .expect("owner hears its delegate's cancelled ending")
        .clone();
    assert_eq!(message.words.as_ref(), REASON);
    assert!(
        messages_to(&driver, &owner)
            .iter()
            .any(|(number, words)| *number == message.number && words.windows(REASON.len()).any(|part| part == REASON))
    );
    driver.send(protocol::turn(CHANNEL, owner.task, owner.attempt, 1, 1, Some(message.number), b"cancellation read"));
    driver.settle();
    assert!(!live(&driver, owner.task).inbox.iter().any(|word| word.number == message.number));
    driver.answer(
        CHANNEL,
        owner.task,
        owner.attempt,
        1,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"nothing running".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    driver.settle();
    finish_task(&mut driver, &director, tasks::TaskResult::Report { words: b"checked".as_slice().into() });
    let ended = ended_task(&driver, delegate.task);
    assert!(matches!(&ended.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { by, reason, .. })
        if *by == tasks::Party::Task(director.task) && reason.as_ref() == REASON));
    for (task, report) in [(owner.task, b"nothing running".as_slice()), (director.task, b"checked".as_slice())] {
        assert_eq!(
            ended_task(&driver, task).phase,
            tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: report.into() }))
        );
    }
    for task in [director.task, owner.task, delegate.task] {
        assert!(driver.task(task).is_none(), "no task left running: {task}");
        assert!(ended_task(&driver, task).ended_at.is_some());
    }
}
