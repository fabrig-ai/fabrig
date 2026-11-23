//! Questions, delegation and effort decisions below the real loop.
use super::*;
use jig_domain_core::{documents, reads};
use temper_engine_domain_world::budget as b;

fn fixture() -> (Driver, engine::Assignment) {
    b::configured([config(965), config(965)])
}

fn inbox(driver: &mut Driver) -> Vec<documents::InboxEntry> {
    let reads::Answer::Page(page) = b::read(
        driver,
        reads::Read::Inbox {
            group: reads::Group::Needs,
            filter: reads::Filter { project: Some(1), goal: None },
            from: None,
        },
    ) else {
        panic!("inbox")
    };
    assert!(!page.cut);
    assert!(page.next.is_none());
    page.rows
        .into_iter()
        .filter_map(|row| match row {
            reads::PageRow::Inbox(entry) => Some(entry.clone()),
            _ => None,
        })
        .collect()
}

fn lines(driver: &Driver, task: u64) -> Vec<&tasks::ActivityLine> {
    driver
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == task => {
                Some(line)
            }
            _ => None,
        })
        .collect()
}

fn heard_once(driver: &Driver, task: u64, message: u64, from: u64, kind: &tasks::MessageKind, words: &[u8]) {
    assert_eq!(
        lines(driver, task)
            .iter()
            .filter(|line| match &line.line {
                tasks::Line::Words { message: number, by, kind: found, words: text } =>
                    *number == message && *by == tasks::Party::Task(from) && found == kind && text.as_ref() == words,
                _ => false,
            })
            .count(),
        1
    );
}

#[test]
fn an_answer_descends_two_owners_before_the_investigation_starts() {
    let (mut driver, director) = fixture();
    let [a, a1] = b::owners(&mut driver, &director);
    let question = b::message(&mut driver, &a1, a.task, 103, engine::MessageForm::Question, b::QUESTION);
    driver.send(protocol::turn(b::CHANNEL, a.task, a.attempt, 1, 0, Some(question), b"read question"));
    driver.settle();
    let up = b::message(&mut driver, &a, director.task, 104, engine::MessageForm::Question, b::UP);
    driver.send(protocol::turn(b::CHANNEL, director.task, director.attempt, 1, 0, Some(up), b"read owner's question"));
    driver.settle();
    let before = inbox(&mut driver);
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].object, documents::InboxObject::Question { message: up });
    assert!(before[0].reply.is_none());
    b::end(&mut driver, &a1, tasks::End::Parked);
    b::end(&mut driver, &a, tasks::End::Parked);
    b::end(&mut driver, &director, tasks::End::Parked);
    let kept = inbox(&mut driver);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].object, before[0].object, "parking and rereading keep the question");
    assert!(kept[0].reply.is_none());
    b::ask_person(
        &mut driver,
        32,
        people::Ask::AnswerQuestion { project: 1, task: director.task, question: up, words: b::ANSWER.into() },
    );
    assert_eq!(inbox(&mut driver), []);
    let a = b::latest(&driver, a.task);
    let person_answer = a
        .messages
        .iter()
        .find(|m| m.kind == tasks::MessageKind::Answer { question: up })
        .expect("person's answer reaches A without a chat call")
        .number;
    heard_once(&driver, a.task, person_answer, director.task, &tasks::MessageKind::Answer { question: up }, b::ANSWER);
    driver.send(protocol::turn(b::CHANNEL, a.task, a.attempt, 1, 0, Some(person_answer), b"heard answer"));
    driver.settle();
    let answer = b::message(&mut driver, &a, a1.task, 105, engine::MessageForm::Answer { question }, b::ANSWER);
    let a1 = b::latest(&driver, a1.task);
    assert_eq!(a1.messages.iter().filter(|m| m.number == answer).count(), 1);
    driver.send(protocol::turn(b::CHANNEL, a1.task, a1.attempt, 1, 0, Some(answer), b"heard budget answer"));
    driver.settle();
    let heard = driver
        .transactions
        .iter()
        .position(|writes| {
            writes.iter().any(|write| matches!(write,
        Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(line))))
        if line.task == a1.task && matches!(line.line, tasks::Line::Turn { read: Some(m), .. } if m == answer)
    ))
        })
        .expect("answer committed before delegation");
    let mut investigate = b::delegate(b"Investigate the budget.", 10, 0);
    investigate.llm = Some(b::choice(0, 0));
    let a5 = b::make(&mut driver, &a1, 106, investigate);
    let made = driver
        .transactions
        .iter()
        .position(|writes| {
            writes.iter().any(|write| matches!(write,
        Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) if task.number == a5.task
    ))
        })
        .expect("investigator saved");
    assert!(made > heard, "delegation follows the read");
    assert_eq!(b::document(&mut driver, a5.task).llm, Some(b::choice(0, 0)));
    heard_once(&driver, a.task, question, a1.task, &tasks::MessageKind::Question, b::QUESTION);
    heard_once(&driver, director.task, up, a.task, &tasks::MessageKind::Question, b::UP);
    heard_once(&driver, a1.task, answer, a.task, &tasks::MessageKind::Answer { question }, b::ANSWER);
    b::report(&mut driver, &a5, b::REPORT);
    b::report(&mut driver, &a1, b"Investigation done.");
    b::report(&mut driver, &a, b"Work done.");
    let director = b::latest(&driver, director.task);
    b::report(&mut driver, &director, b"Done.");
    for (run, requester) in [
        (&director, tasks::Party::Person(1)),
        (&a, tasks::Party::Task(director.task)),
        (&a1, tasks::Party::Task(a.task)),
        (&a5, tasks::Party::Task(a1.task)),
    ] {
        let doc = b::document(&mut driver, run.task);
        assert_eq!(doc.chip.number, run.task);
        assert_eq!(doc.project, 1);
        assert_eq!(doc.requester, requester);
        assert!(matches!(doc.result, Some(documents::Ending::Done(_))));
    }
    let owner = b::document(&mut driver, a.task);
    assert_eq!(owner.goal, Some(a.task));
    assert_eq!(owner.authority.ceiling.deadline, Some(b::A_DEADLINE));
    let subowner = b::document(&mut driver, a1.task);
    assert_eq!(subowner.authority.ceiling.deadline, Some(b::A1_DEADLINE));
    assert_eq!(subowner.authority.depth, 1);
    b::healthy(&driver);
}

#[test]
fn an_agents_effort_amendment_precedes_the_next_investigation_attempt() {
    let (mut driver, owner) = fixture();
    let mut member = b::delegate(b"Investigate the budget.", 10, 0);
    member.llm = Some(b::choice(0, 0));
    let first = b::make(&mut driver, &owner, 101, member);
    tool_call(
        &mut driver,
        &first,
        103,
        engine::Tool::Subscribe {
            kind: tasks::SubscriptionKind::Timer { at: Wall::from_nanos(u64::MAX), period: None },
        },
    );
    let finding = b::message(&mut driver, &first, owner.task, 104, engine::MessageForm::Words, b::FINDING);
    driver.send(protocol::turn(b::CHANNEL, owner.task, owner.attempt, 1, 0, Some(finding), b"read finding"));
    driver.settle();
    b::end(&mut driver, &first, tasks::End::Parked);
    heard_once(&driver, owner.task, finding, first.task, &tasks::MessageKind::Words, b::FINDING);
    let medium = b::document(&mut driver, first.task);
    assert_eq!(medium.llm, Some(b::choice(0, 0)));
    assert_eq!(medium.spec_revision, 0);
    let before = lines(&driver, first.task).into_iter().cloned().collect::<Vec<_>>();
    let mut amendment = task_amendment(None, None, b::REASON);
    amendment.llm = Some(b::choice(0, 1));
    tool_call(
        &mut driver,
        &owner,
        102,
        engine::Tool::Amend { keep_parameters: false, target: first.task, revision: 0, amendment },
    );
    assert!(matches!(driver.answered(102), Some(engine::CallAnswer::Amended { .. })));
    let next = b::latest(&driver, first.task);
    assert!(next.attempt > first.attempt);
    assert_eq!(next.run.policy.model.effort.as_deref(), Some(b"high".as_slice()));
    let high = b::document(&mut driver, first.task);
    assert_eq!(high.llm, Some(b::choice(0, 1)));
    assert_eq!(high.spec_revision, 1);
    let after = lines(&driver, first.task);
    assert!(before.iter().all(|line| after.contains(&line)));
    let decisions: Vec<_> =
        after.iter().filter(|l| matches!(l.line, tasks::Line::Decided { revision: 1, .. })).collect();
    assert_eq!(decisions.len(), 1);
    let reasons: Vec<_> = after
        .iter()
        .filter(|l| {
            matches!(&l.line, tasks::Line::Heard(word)
        if word.from == tasks::Party::Task(owner.task) && word.kind == tasks::MessageKind::Amendment { revision: 1 }
            && word.words.as_ref() == b::REASON)
        })
        .collect();
    assert_eq!(reasons.len(), 1);
    let starts: Vec<_> = after
        .iter()
        .filter(|l| l.position > decisions[0].position && matches!(l.line, tasks::Line::RunBegan { .. }))
        .collect();
    assert_eq!(starts.len(), 1);
    assert!(starts[0].position > reasons[0].position);
    assert!(matches!(starts[0].line, tasks::Line::RunBegan { attempt, .. } if attempt == next.attempt));
    b::report(&mut driver, &next, b::REPORT);
    b::report(&mut driver, &owner, b"Done.");
    assert_eq!(b::document(&mut driver, next.task).llm, Some(b::choice(0, 1)));
    b::healthy(&driver);
}
