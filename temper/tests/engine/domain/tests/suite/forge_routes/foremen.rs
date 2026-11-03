//! Each question gets a read-only foreman (temper/docs/design/domain/forge.md, 8.11).
use super::questions::{call, make_change, next_producer, plan_world, replies, row, wait_answer};
use super::*;
use temper_engine_domain_forge_change::Stands;

const TICKS: u32 = 200;
const QUESTION: &[u8] = b"Which behavior should hold?\nKeep the design or widen the change?\nI recommend the design.";
const ANSWER: &[u8] = b"Keep the documented behavior.";

fn foreman_world() -> (World, engine::Assignment, u64, engine::Assignment) {
    let (mut world, requester) = plan_world(None);
    let change = make_change(&mut world, &requester, 93, engine::ChangeQuestions::Foreman);
    let producer = next_producer(&mut world, change);
    world.until(Until::Ready);
    (world, requester, change, producer)
}

fn ask(world: &mut World, producer: &engine::Assignment, change: u64) -> (u64, engine::Assignment) {
    let number = world.restore_delegate_question(producer.task, QUESTION);
    for _ in 0..TICKS {
        world.tick();
        if let Some(question) = row(world, change).questions.open.first()
            && let Stands::Foreman { task: Some(task) } = question.stands
            && let Some(run) = world.assigned.iter().find(|run| run.task == task)
            && world.store.pending.is_empty()
        {
            return (number, run.clone());
        }
    }
    let requester = live_task(world, change).requester;
    panic!(
        "foreman assigned: questions={:?}; tasks={:?}; inbox={:?}; errors={:?}",
        row(world, change).questions,
        world
            .store
            .rows
            .values()
            .filter_map(|record| match record {
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) =>
                    Some((row.number, &row.phase, &row.numbers)),
                _ => None,
            })
            .collect::<Vec<_>>(),
        match requester {
            tasks::Party::Task(task) =>
                replies(world, task, change).iter().map(|word| String::from_utf8_lossy(&word.words)).collect::<Vec<_>>(),
            _ => Vec::new(),
        },
        world.call_errors
    );
}

fn verdict(world: &mut World, foreman: &engine::Assignment, code: u32, words: &[u8]) {
    super::reviews::finish_review(
        world,
        foreman,
        tasks::TaskResult::Verdict { code, fields: Box::new([]), words: words.into() },
    );
}

fn wait_up(world: &mut World, change: u64) {
    for _ in 0..TICKS {
        world.tick();
        if row(world, change).questions.open.first().is_some_and(|q| q.stands == Stands::Asked)
            && world.store.pending.is_empty()
        {
            return;
        }
    }
    panic!("question passed up: {:?}", row(world, change));
}

#[test]
fn a_question_gets_a_foreman_reading_only() {
    let (mut world, _, change, producer) = foreman_world();
    let branch = row(&world, change).branch.clone();
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"work to inspect", 1)
        .expect("producer pushed");
    let (number, foreman) = ask(&mut world, &producer, change);
    let task = live_task(&world, foreman.task);
    assert_eq!(task.executor, tasks::Executor::Agent { charter: 6 });
    assert_eq!(task.authority.tools.0, jig_domain_core::ToolFamilies::JIG.read.0);
    assert!(!task.authority.grants.is_empty());
    assert!(
        task.authority.grants.iter().all(|grant| grant.connector == 0 && forge_top::READ_KINDS.contains(&grant.kind))
    );
    assert_eq!(task.authority.tools.0 & jig_domain_core::ToolFamilies::JIG.message.0, 0);
    assert!(task.authority.delegation.kinds.is_empty());
    assert!(!foreman.run.policy.modify);
    assert!(!foreman.run.policy.shell);
    assert!(!foreman.run.policy.agents);
    let tasks::Contract::Verdict { choices } = &task.contract else { panic!("foreman verdict") };
    assert_eq!(choices.iter().map(|choice| choice.code).collect::<Vec<_>>(), [1, 2, 3]);
    assert!(
        choices
            .iter()
            .all(|choice| choice.words == world.env.limits.tasks.message_bytes - 64 && choice.fields.is_empty())
    );
    let brief = core::str::from_utf8(&task.spec.words).expect("brief text");
    let expected = format!(
        "You are the foreman of change task {change}, a question from task {} (its question {number}).\n\nThe change's words:\nMake the agreed fix\nThe question:\n{}\nThe change's branch: {}, pushed at {}.\nAnswer with a verdict: answer, finish or escalate.",
        producer.task,
        core::str::from_utf8(QUESTION).expect("question text"),
        core::str::from_utf8(&branch).expect("branch text"),
        core::str::from_utf8(&engine::commit_text(&translate::commit(head))).expect("head text"),
    );
    assert_eq!(brief, expected);
    assert!(task_brief(&foreman).windows(task.spec.words.len()).any(|part| part == task.spec.words.as_ref()));
    assert!(task_brief(&foreman).windows(QUESTION.len()).any(|part| part == QUESTION));
    assert!(task_brief(&foreman).windows(branch.len()).any(|part| part == branch.as_ref()));
    assert!(
        task_brief(&foreman).windows(40).any(|part| part == engine::commit_text(&translate::commit(head)).as_ref())
    );
    assert_eq!(task.spec.parameters.len(), 2);
    assert!(task.spec.parameters.contains(&tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() }));
    assert_eq!(foreman.workspace.repositories[0].start, engine::ForgeStart::Base(b"main".as_slice().into()));
    assert!(foreman.workspace.repositories[0].push.is_none());
    assert_eq!(row(&world, change).delegate.map(|(task, _)| task), Some(producer.task));
}

#[test]
fn a_foremans_answer_reaches_the_producer() {
    let (mut world, requester, change, producer) = foreman_world();
    let (number, foreman) = ask(&mut world, &producer, change);
    verdict(&mut world, &foreman, 1, ANSWER);
    wait_answer(&mut world, producer.task, change);
    let words = replies(&world, producer.task, change);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].kind, tasks::MessageKind::Answer { question: number });
    assert_eq!(words[0].words.as_ref(), ANSWER);
    assert!(replies(&world, requester.task, change).is_empty());
    assert_eq!(row(&world, change).delegate.map(|(task, _)| task), Some(producer.task));
    assert_eq!(row(&world, change).delegate_status, temper_engine_domain_forge_change::Status::Unknown);
}

#[test]
fn a_foremans_finish_tells_the_producer_to_finish() {
    let (mut world, requester, change, producer) = foreman_world();
    let (number, foreman) = ask(&mut world, &producer, change);
    verdict(&mut world, &foreman, 2, ANSWER);
    wait_answer(&mut world, producer.task, change);
    let words = replies(&world, producer.task, change);
    assert_eq!(words[0].kind, tasks::MessageKind::Answer { question: number });
    assert_eq!(
        words[0].words.as_ref(),
        b"Finish: the work done holds; list the rest under Left.\nKeep the documented behavior."
    );
    assert!(replies(&world, requester.task, change).is_empty());
}

#[test]
fn a_foremans_escalation_goes_up_with_its_reasons() {
    let (mut world, requester, change, producer) = foreman_world();
    let (_, foreman) = ask(&mut world, &producer, change);
    verdict(&mut world, &foreman, 3, b"Please decide.");
    wait_up(&mut world, change);
    let words = replies(&world, requester.task, change);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].kind, tasks::MessageKind::Question);
    let expected = [QUESTION, b"\n\nForeman:\nPlease decide."].concat();
    assert_eq!(words[0].words.as_ref(), expected);
    assert!(replies(&world, producer.task, change).is_empty());
}

#[test]
fn a_failed_foreman_goes_up() {
    let (mut world, requester, change, producer) = foreman_world();
    let (_, foreman) = ask(&mut world, &producer, change);
    super::reviews::finish_review(
        &mut world,
        &foreman,
        tasks::TaskResult::Failure { reason: b"Cannot read the design.".as_slice().into() },
    );
    wait_up(&mut world, change);
    let words = replies(&world, requester.task, change);
    assert_eq!(words[0].kind, tasks::MessageKind::Question);
    assert_eq!(words[0].words.as_ref(), [QUESTION, b"\n\nForeman:\nCannot read the design."].concat());
}

#[test]
fn foremen_count_against_the_bound_and_the_changes_tasks() {
    let (mut world, _, change, producer) = foreman_world();
    assert_eq!(live_task(&world, change).authority.delegation.tasks, 7);
    for taken in 1..=4 {
        let (_, foreman) = ask(&mut world, &producer, change);
        assert_eq!(row(&world, change).questions.taken, taken);
        assert_eq!(live_task(&world, change).made, 2 + taken);
        verdict(&mut world, &foreman, 1, ANSWER);
        wait_answer(&mut world, producer.task, change);
    }
    world.restore_delegate_question(producer.task, QUESTION);
    for _ in 0..TICKS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { why: tasks::Hold::Procedure, .. })
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert!(matches!(live_task(&world, change).phase, tasks::Phase::Held { why: tasks::Hold::Procedure, .. }));
    assert_eq!(row(&world, change).questions.taken, 4);
    assert_eq!(live_task(&world, change).made, 6);
    assert_eq!(world.assigned.iter().filter(|run| run.charter == 6).count(), 4);
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Taken);
}

#[test]
fn an_ended_asker_cancels_its_foreman() {
    let (mut world, _, change, producer) = foreman_world();
    let (_, foreman) = ask(&mut world, &producer, change);
    super::reviews::finish_review(
        &mut world,
        &producer,
        tasks::TaskResult::Failure { reason: b"the asker ended".as_slice().into() },
    );
    for _ in 0..TICKS {
        world.tick();
        if row(&world, change).questions.open.is_empty() && world.store.pending.is_empty() {
            break;
        }
    }
    assert!(row(&world, change).questions.open.is_empty());
    assert!(
        matches!(live_task(&world, foreman.task).phase, tasks::Phase::Closing(_) | tasks::Phase::Ended(_)),
        "foreman phase: {:?}",
        live_task(&world, foreman.task).phase
    );
}

#[test]
fn a_cancelled_foreman_goes_up() {
    let (mut world, requester, change, producer) = foreman_world();
    let (_, foreman) = ask(&mut world, &producer, change);
    call(
        &mut world,
        &requester,
        94,
        engine::Tool::Cancel { target: foreman.task, reason: b"stop the foreman".as_slice().into() },
    );
    let event = world.protocol.answer(
        Token::new(7),
        foreman.task,
        foreman.attempt,
        5,
        tasks::End::Parked,
        engine::Parts::none(),
    );
    world.send(event);
    wait_up(&mut world, change);
    assert_eq!(
        replies(&world, requester.task, change)[0].words.as_ref(),
        [QUESTION, b"\n\nForeman:\nthe foreman was cancelled"].concat()
    );
}

#[test]
fn a_refused_foreman_goes_up_without_holding_the_change() {
    let (mut world, requester, change, producer) = foreman_world();
    let mut authority = live_task(&world, change).authority.clone();
    authority.delegation.kinds = authority
        .delegation
        .kinds
        .iter()
        .filter(|kind| **kind != tasks::AuthorityExecutor::Charter(6))
        .copied()
        .collect();
    let amendment = temper_engine_domain_world::fixtures::task_amendment(
        None,
        Some(authority),
        b"Keep the existing coding and review delegation",
    );
    let revision = live_task(&world, change).revisions.spec;
    call(&mut world, &requester, 94, engine::Tool::Amend { target: change, revision, amendment });
    world.restore_delegate_question(producer.task, b"Which behavior?");
    wait_up(&mut world, change);
    let words = replies(&world, requester.task, change);
    assert_eq!(words[0].kind, tasks::MessageKind::Question);
    let text = core::str::from_utf8(&words[0].words).expect("refusal words");
    assert!(text.starts_with("Which behavior?\n\nForeman:\nForeman delegate refused"));
    assert!(text.contains("executor"), "authority's reason: {text}");
    assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
    assert_eq!(row(&world, change).delegate.map(|(task, _)| task), Some(producer.task));
    assert!(!world.assigned.iter().any(|run| run.charter == 6));
}

#[test]
fn a_foreman_beyond_task_authority_goes_up_with_its_words() {
    let (mut world, requester, change, producer) = foreman_world();
    let mut authority = live_task(&world, change).authority.clone();
    authority.delegation.tasks = 1;
    let amendment =
        temper_engine_domain_world::fixtures::task_amendment(None, Some(authority), b"Keep only the producing task");
    let revision = live_task(&world, change).revisions.spec;
    call(&mut world, &requester, 94, engine::Tool::Amend { target: change, revision, amendment });
    world.restore_delegate_question(producer.task, b"Which behavior?");
    wait_up(&mut world, change);
    let text =
        core::str::from_utf8(&replies(&world, requester.task, change)[0].words).expect("authority words").to_owned();
    assert!(text.starts_with("Which behavior?\n\nForeman:\nits 1 tasks are made"), "{text}");
    assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
    assert_eq!(live_task(&world, change).made, 2);
}

#[test]
fn a_restart_keeps_the_foreman_and_relays_its_result_once() {
    let (mut world, requester, change, producer) = foreman_world();
    let (number, foreman) = ask(&mut world, &producer, change);
    world.until(Until::Ready);
    world.restart(true, true);
    world.until(Until::Ready);
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Foreman { task: Some(foreman.task) });
    assert_eq!(world.assigned.iter().filter(|run| run.charter == 6).count(), 1);
    verdict(&mut world, &foreman, 1, ANSWER);
    wait_answer(&mut world, producer.task, change);
    assert_eq!(replies(&world, producer.task, change).len(), 1);
    assert_eq!(replies(&world, producer.task, change)[0].kind, tasks::MessageKind::Answer { question: number });
    assert!(replies(&world, requester.task, change).is_empty());
}

#[test]
fn a_large_changes_words_are_cut_before_the_question() {
    let (mut world, requester) = plan_world(None);
    let mut change = engine::ChangeCall::words(vec![b'x'; 400].into_boxed_slice(), None);
    change.questions = Some(engine::ChangeQuestions::Foreman);
    call(&mut world, &requester, 93, engine::Tool::Change { changes: Box::new([change]) });
    let Some(engine::CallAnswer::Delegated(made)) = world.answers.last() else { panic!("large change made") };
    let change = made[0].task;
    let producer = next_producer(&mut world, change);
    world.until(Until::Ready);
    let (_, foreman) = ask(&mut world, &producer, change);
    let spec = &live_task(&world, foreman.task).spec.words;
    assert!(spec.windows("…\nThe question:\n".len()).any(|part| part == "…\nThe question:\n".as_bytes()));
    assert!(spec.windows(QUESTION.len()).any(|part| part == QUESTION));
    assert!(task_brief(&foreman).windows(QUESTION.len()).any(|part| part == QUESTION));
    assert!(task_brief(&foreman).windows(spec.len()).any(|part| part == spec.as_ref()));
}

#[test]
fn queued_questions_get_distinct_foremen_in_order() {
    let (mut world, _, change, producer) = foreman_world();
    let (first, foreman) = ask(&mut world, &producer, change);
    let second = world.restore_delegate_question(producer.task, QUESTION);
    for _ in 0..TICKS {
        world.tick();
        if row(&world, change).questions.open.len() == 2 && world.store.pending.is_empty() {
            break;
        }
    }
    assert_eq!(row(&world, change).questions.open.len(), 2, "{:?}", row(&world, change).questions);
    assert_eq!(world.assigned.iter().filter(|run| run.charter == 6).count(), 1);
    verdict(&mut world, &foreman, 1, ANSWER);
    let mut next = None;
    for _ in 0..TICKS {
        world.tick();
        next = world.assigned.iter().find(|run| run.charter == 6 && run.task != foreman.task).cloned();
        if next.is_some() && world.store.pending.is_empty() {
            break;
        }
    }
    let next = next.expect("the next question gets its own foreman");
    assert_eq!(world.env.limits.tasks.batch, 1);
    assert_eq!(replies(&world, producer.task, change)[0].kind, tasks::MessageKind::Answer { question: first });
    assert_eq!(row(&world, change).questions.open[0].number, second);
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Foreman { task: Some(next.task) });
    assert_eq!(row(&world, change).questions.taken, 2);
    verdict(&mut world, &next, 1, ANSWER);
    wait_answer(&mut world, producer.task, change);
    assert_eq!(replies(&world, producer.task, change).len(), 2);
    assert_eq!(replies(&world, producer.task, change)[1].kind, tasks::MessageKind::Answer { question: second });
}
