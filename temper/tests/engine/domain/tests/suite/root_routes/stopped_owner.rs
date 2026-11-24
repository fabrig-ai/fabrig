//! Stop isolates a subtree; reports and queued endings survive its release
//! (temper/docs/design/testing/scenarios.md, 1.2).
use super::*;

fn person(driver: &mut Driver, key: u8, ask: people::Ask) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        ask,
    }));
    driver.settle();
}

fn report(driver: &mut Driver, run: &engine::Assignment, words: &[u8]) {
    finish_task(driver, run, tasks::TaskResult::Report { words: words.into() });
}

fn ended(driver: &Driver, task: u64) -> &tasks::TaskRecord {
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
    else {
        panic!("task {task} ended")
    };
    row
}

fn history(driver: &Driver, task: u64) -> Vec<tasks::ActivityLine> {
    driver
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(row))) if row.task == task => {
                Some(row.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn an_unstopped_sibling_finishes_without_a_hold_or_decision() {
    let (mut driver, director) = batch_fixture_with(3, 1);
    let children = call_batch(
        &mut driver,
        &director,
        950,
        Box::new([report_delegate(b"A", Box::new([])), report_delegate(b"B", Box::new([]))]),
    );
    let a = assigned_task(&driver, children[0]);
    person(
        &mut driver,
        81,
        people::Ask::Stop { project: 1, task: children[1], reason: Box::from(b"Parking the task".as_slice()) },
    );
    report(&mut driver, &a, b"A remained free and finished.");
    let row = ended(&driver, a.task);
    assert_eq!(
        row.phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
            words: b"A remained free and finished.".as_slice().into()
        }))
    );
    assert_eq!(row.placed, None);
    assert_eq!(row.hold, 0);
    let lines = history(&driver, a.task);
    assert!(!lines.iter().any(|row| matches!(row.line, tasks::Line::Decided { .. })));
    assert_eq!(lines.iter().filter(|row| matches!(row.line, tasks::Line::RunBegan { .. })).count(), 1);
    assert_eq!(lines.iter().filter(|row| matches!(row.line, tasks::Line::Milestone { .. })).count(), 2);
}

#[test]
fn a_report_reaches_its_owner_and_the_owners_chat() {
    let (mut driver, director) = batch_fixture_with(3, 2);
    let mut owner = report_delegate(b"B", Box::new([]));
    owner.authority.budget.spend = 40;
    owner.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth: 1 };
    let b = call_batch(&mut driver, &director, 950, Box::new([owner]))[0];
    let b = assigned_task(&driver, b);
    let implementer = call_batch(&mut driver, &b, 951, Box::new([report_delegate(b"implementer", Box::new([]))]))[0];
    let implementer = assigned_task(&driver, implementer);
    report(&mut driver, &implementer, b"implementation finished");
    assert_eq!(ended(&driver, implementer.task).requester, tasks::Party::Task(b.task));
    assert_eq!(
        ended(&driver, implementer.task).phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
            words: b"implementation finished".as_slice().into()
        }))
    );
    assert!(
        live(&driver, b.task).inbox.iter().any(|word| word.from == tasks::Party::Task(implementer.task)
            && word.words.as_ref() == b"implementation finished")
    );
    report(&mut driver, &b, b"B received the implementation");
    assert!(
        live(&driver, director.task)
            .inbox
            .iter()
            .any(|word| word.from == tasks::Party::Task(b.task)
                && word.words.as_ref() == b"B received the implementation")
    );
    report(&mut driver, &director, b"B reported to the chat");
    assert_eq!(
        ended(&driver, b.task).phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
            words: b"B received the implementation".as_slice().into()
        }))
    );
    assert_eq!(
        ended(&driver, director.task).phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
            words: b"B reported to the chat".as_slice().into()
        }))
    );
}

#[test]
fn a_stopped_owner_hears_both_delegates_only_after_release() {
    let (mut driver, owner) = batch_fixture_with(3, 1);
    let mut change = report_delegate(b"change", Box::new([]));
    change.contract = tasks::Contract::Change { connector: 1, kind: 2, words: 128, choices: Box::new([]) };
    let children =
        call_batch(&mut driver, &owner, 950, Box::new([report_delegate(b"implementer", Box::new([])), change]));
    let runs = children.iter().map(|task| assigned_task(&driver, *task)).collect::<Vec<_>>();
    person(
        &mut driver,
        81,
        people::Ask::Stop { project: 1, task: owner.task, reason: Box::from(b"Parking the task".as_slice()) },
    );
    for run in std::iter::once(&owner).chain(runs.iter()) {
        driver.answer(Token::new(7), run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
        driver.settle();
    }
    assert!(!history(&driver, owner.task).iter().any(
        |row| matches!(&row.line, tasks::Line::Heard(word) if matches!(word.kind, tasks::MessageKind::Result(_)))
    ));
    let revision = live(&driver, owner.task).revisions.hold;
    person(&mut driver, 82, people::Ask::Release { project: 1, task: owner.task, revision, again: None });
    for (child, result) in children.iter().zip([
        tasks::TaskResult::Report { words: b"finished after release".as_slice().into() },
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"h1".as_slice().into()),
            connector: 1,
            kind: 2,
            resource: 1,
            words: b"landed after release".as_slice().into(),
        },
    ]) {
        let resumed = driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| match delivery {
                Delivery::Assigned { assignment, .. } if assignment.task == *child => Some(assignment.clone()),
                _ => None,
            })
            .expect("resumed delegate");
        finish_task(&mut driver, &resumed, result);
    }
    let lines = history(&driver, owner.task);
    let release = lines.iter().find(|row| matches!(row.line, tasks::Line::Decided { .. })).expect("release").position;
    for child in &children {
        let heard: Vec<_> = lines.iter().filter(|row| matches!(&row.line, tasks::Line::Heard(word) if word.from == tasks::Party::Task(*child) && matches!(word.kind, tasks::MessageKind::Result(_)))).collect();
        assert_eq!(heard.len(), 1);
        assert!(heard[0].position > release);
    }
}
