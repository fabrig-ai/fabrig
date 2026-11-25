//! Stopped workers retain committed work without overlapping attempts
//! (testing/reference-case.md's 4.6 row).
use super::*;

const FIRST: Token = Token::new(7);
const OTHER: Token = Token::new(8);
const SLOTS: u32 = 2; // The director and its questioning delegate.
const COMMITTED: &[u8] = b"question read before stop";
const ONE: usize = 1;

pub(super) fn two_workers(
    mut configs: [engine::Config; 2],
    mut bounds: engine::Limits,
) -> ([engine::Config; 2], engine::Limits) {
    const WORKERS: u32 = 2;
    const PROGRAM_HOSTS: u32 = WORKERS + 1;
    bounds.fleet.configured_workers = WORKERS;
    bounds.fleet.workers = WORKERS;
    bounds.people.program_hosts = PROGRAM_HOSTS;
    bounds.accounts.program_hosts = PROGRAM_HOSTS;
    bounds.journal.deliveries = bounds.journal.deliveries.max(TWO_WORKER_DELIVERIES);
    bounds.journal.held = 3 * bounds.journal.deliveries + 16;
    for configuration in &mut configs {
        configuration.core.defaults.workers = Box::new([
            jig_domain_core::settings::WorkerSetting {
                number: 1,
                name: b"first-worker".as_slice().into(),
                places: 1,
                digest: Some([7; 32]),
                withdrawn: false,
            },
            jig_domain_core::settings::WorkerSetting {
                number: 2,
                name: b"second-worker".as_slice().into(),
                places: 1,
                digest: Some([7; 32]),
                withdrawn: false,
            },
        ]);
    }
    (configs, bounds)
}

fn chat_on_first_worker() -> (Driver, engine::Assignment) {
    let (configs, bounds) = batch_settings(SLOTS, 1, 500, false, false, false);
    let (configs, bounds) = two_workers(configs, bounds);
    let mut driver = Driver::configured(Store::new(), configs, &bounds);
    driver.settle();
    driver.send(protocol::hello_at(FIRST, 0, SLOTS, Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let first = driver.last_assignment();
    (driver, first)
}

fn link_other(driver: &mut Driver) {
    driver.send(protocol::hello_at(OTHER, 1, SLOTS, Box::new([])));
    driver.settle();
}

fn stop(driver: &mut Driver, run: &engine::Assignment) -> Wall {
    driver.send(protocol::up(FIRST, host::LinkUp::Draining, engine::Parts::none()));
    driver.send(protocol::shutdown(FIRST, run.task, run.attempt, 1));
    driver.pump();
    let until = live(driver, run.task).suspension.expect("stopped run suspended").until;
    driver.send(protocol::lost(FIRST));
    driver.pump();
    until
}

fn past_restart_window(driver: &mut Driver, until: Wall) {
    driver.fire_at(Duration::from_nanos(until.as_nanos() + 1));
    driver.settle();
}

pub(super) fn finish(driver: &mut Driver, channel: Token, run: &engine::Assignment, words: &[u8]) {
    driver.answer(
        channel,
        run.task,
        run.attempt,
        1,
        tasks::End::Finished { result: tasks::TaskResult::Report { words: words.into() }, cancel_delegates: false },
        engine::Parts::none(),
    );
    driver.settle();
}

#[test]
fn a_stopped_chat_retries_elsewhere_with_a_fenced_journal() {
    let (mut driver, first) = chat_on_first_worker();
    link_other(&mut driver);
    driver.send(protocol::turn(FIRST, first.task, first.attempt, 1, 1, None, COMMITTED));
    driver.settle();
    let until = stop(&mut driver, &first);
    driver.fire_at(Duration::from_nanos(until.as_nanos() - 1));
    driver.pump();
    assert_eq!(placements(&driver), [(FIRST, first.task, first.attempt)], "no retry before the restart grace");
    past_restart_window(&mut driver, until);
    let retry = driver.last_assignment();
    assert_eq!(placements(&driver), [(FIRST, first.task, first.attempt), (OTHER, first.task, retry.attempt)]);
    assert!(retry.attempt > first.attempt);
    assert_eq!(retry.transcript.as_ref(), [Box::<[u8]>::from(COMMITTED)]);
    assert_eq!(live(&driver, first.task).tries, tasks::Tries::NONE);
    driver.send(protocol::turn(FIRST, first.task, first.attempt, 2, 1, None, b"fenced"));
    driver.settle();
    finish(&mut driver, OTHER, &retry, b"done");
    let mut lines: Vec<_> = driver
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(row))) = row
                && row.task == first.task
            {
                Some(row)
            } else {
                None
            }
        })
        .collect();
    lines.sort_by_key(|row| row.position);
    let runs: Vec<_> = lines
        .iter()
        .filter_map(|row| match &row.line {
            tasks::Line::RunBegan { attempt, .. } => Some((*attempt, None)),
            tasks::Line::RunEnded { attempt, end, .. } => Some((*attempt, Some(*end))),
            _ => None,
        })
        .collect();
    assert_eq!(
        runs,
        [
            (first.attempt, None),
            (first.attempt, Some(tasks::EndKind::Suspended { worker: 1, why: tasks::SuspendWhy::Stopped })),
            (retry.attempt, None),
            (retry.attempt, Some(tasks::EndKind::Finished)),
        ],
        "complete journal: each attempt closes before the next opens"
    );
    let turns: Vec<_> = driver
        .store
        .rows
        .keys()
        .filter(|key| {
            matches!(key,
                Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Turn { task, attempt, .. }))
                    if *task == first.task && *attempt == first.attempt
            )
        })
        .collect();
    assert_eq!(turns.len(), ONE, "the stopped attempt cannot commit a later turn");
}

#[test]
fn a_stopped_director_keeps_its_read_unanswered_question_once() {
    let (driver, parent) = chat_on_first_worker();
    let (mut driver, parent, question, asker) = ask_question(driver, parent);
    park_task(&mut driver, &asker);
    link_other(&mut driver);
    driver.send(protocol::turn(FIRST, parent.task, parent.attempt, 1, 1, Some(question), COMMITTED));
    driver.settle();
    assert!(!live(&driver, parent.task).inbox.iter().any(|word| word.number == question));
    driver.send(protocol::up(
        FIRST,
        host::LinkUp::Activity {
            run: Token::new(parent.task),
            attempt: Token::new(parent.attempt),
            activity: host::AgentActivity::Phase(jig_domain_link::Phase::Waiting),
        },
        engine::Parts::none(),
    ));
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(2039)),
        sign_in: driver.session(),
        read: Box::new(jig_domain_core::reads::Read::Task { task: parent.task }),
    }));
    driver.settle();
    let document = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| match delivery {
            Delivery::Read { answer: jig_domain_core::reads::Answer::Document(document), .. } => Some(document),
            _ => None,
        })
        .expect("director document");
    let jig_domain_core::documents::Document::Task(document) = document.as_ref() else { panic!("task document") };
    assert_eq!(
        document.chip.details.observation.as_ref().map(|o| &o.work),
        Some(&jig_domain_core::documents::ChipWork::Waiting)
    );
    let until = stop(&mut driver, &parent);
    let waiting = &live(&driver, parent.task).question_lines;
    assert_eq!(waiting.len(), ONE);
    assert!(waiting.iter().any(|line| line.message == question && line.answer.is_none()));
    past_restart_window(&mut driver, until);
    let retry = driver.last_assignment();
    assert_eq!(retry.task, parent.task);
    assert_eq!(retry.transcript.as_ref(), [Box::<[u8]>::from(COMMITTED)]);
    let questions: Vec<_> =
        live(&driver, parent.task).question_lines.iter().filter(|line| line.message == question).collect();
    assert_eq!(questions.len(), ONE);
    assert!(questions[0].answer.is_none());
    assert!(!retry.messages.iter().any(|word| word.number == question), "the committed read is not delivered again");
}
