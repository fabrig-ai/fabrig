//! Placement and fencing at the engine's worker link (temper/docs/design/protocol/cluster.md, 6).
use super::*;
use jig_domain_core::{documents, reads};

const FIRST: Token = Token::new(7);
const OTHER: Token = Token::new(8);
const RETURNED: Token = Token::new(9);
const FIRST_POSITION: u32 = 0;
const OTHER_POSITION: u32 = 1;
const ONE_SLOT: u32 = 1;
const FIRST_CHAT: u8 = 31;
const NEXT_CHAT: u8 = 32;
const CLOSE_FIRST: u8 = 33;
const CLOSE_NEXT: u8 = 34;
const FIRST_TURN: u32 = 1;
const SPENT: u64 = 1;
const REPLY: &[u8] = b"First worker replied.";
const TICK: Duration = Duration::from_nanos(1);
const READ: ReplyTo = ReplyTo::new(Token::new(402));
const LOST: tasks::EndKind = tasks::EndKind::Failed { class: tasks::Class::Lost, invalid_result: None };

fn chat_on_first_worker() -> (Driver, engine::Assignment) {
    let (configs, bounds) = stopped_worker::two_workers([91, 91].map(sharing_config), limits());
    let mut driver = Driver::configured(Store::new(), configs, &bounds);
    driver.send(protocol::hello_at(FIRST, FIRST_POSITION, ONE_SLOT, Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    start_chat(&mut driver, FIRST_CHAT);
    let first = driver.last_assignment();
    (driver, first)
}

fn start_chat(driver: &mut Driver, key: u8) -> u64 {
    let since = driver.delivered.len();
    chat_ending(driver, key, people::EndedBy::Requester);
    // A chat waiting for a slot is intentionally not quiescent.
    driver.pump();
    driver.delivered[since..]
        .iter()
        .find_map(|delivery| match delivery {
            Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::Created { first, last, .. }), ..
            } if first == last => Some(*first),
            _ => None,
        })
        .expect("one chat created")
}

fn link_other(driver: &mut Driver) {
    driver.send(protocol::hello_at(OTHER, OTHER_POSITION, ONE_SLOT, Box::new([])));
    driver.settle();
}

fn lose_first(driver: &mut Driver) -> Duration {
    driver.send(protocol::lost(FIRST));
    driver.pump();
    Duration::from_nanos(driver.env.now.as_nanos()).saturating_add(driver.env.limits.fleet.grace)
}

fn retry_after_grace(driver: &mut Driver, grace: Duration) -> engine::Assignment {
    driver.fire_at(grace);
    driver.pump();
    // The first lost retry can wait up to the configured backoff ceiling.
    driver.fire_at(grace.saturating_add(driver.env.limits.tasks.retries.lost.max));
    driver.settle();
    driver.last_assignment()
}

fn report_phase(driver: &mut Driver, channel: Token, run: &engine::Assignment, phase: jig_domain_link::Phase) {
    driver.send(protocol::up(
        channel,
        host::LinkUp::Activity {
            run: Token::new(run.task),
            attempt: Token::new(run.attempt),
            activity: host::AgentActivity::Phase(phase),
        },
        engine::Parts::none(),
    ));
    driver.settle();
}

fn reply_and_park(driver: &mut Driver, channel: Token, run: &engine::Assignment, words: &[u8]) {
    driver.send(protocol::turn(channel, run.task, run.attempt, FIRST_TURN, SPENT, None, words));
    driver.settle();
    driver.answer(channel, run.task, run.attempt, SPENT, tasks::End::Parked, engine::Parts::none());
    driver.settle();
}

fn close(driver: &mut Driver, task: u64, key: u8) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(403)),
        sign_in: driver.session(),
        key: [key; 16],
        ask: people::Ask::Cancel { project: 1, task, reason: b"chat closed".as_slice().into(), grace: None },
    }));
    driver.settle();
    let ended = ended_task(driver, task);
    assert!(matches!(ended.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { by, result: None, .. })
        if by == ended.requester));
}

fn activity(driver: &Driver, task: u64) -> Vec<tasks::ActivityLine> {
    let mut lines: Vec<_> = driver
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == task => {
                Some(line.clone())
            }
            _ => None,
        })
        .collect();
    lines.sort_by_key(|line| line.position);
    lines
}

fn attempt_activity(driver: &Driver, run: &engine::Assignment) -> Vec<tasks::ActivityLine> {
    activity(driver, run.task)
        .into_iter()
        .filter(|row| {
            matches!(row.line, tasks::Line::RunBegan { attempt, .. } | tasks::Line::RunEnded { attempt, .. }
            | tasks::Line::Turn { attempt, .. } if attempt == run.attempt)
        })
        .collect()
}

fn turns(driver: &Driver, task: u64) -> Vec<(u64, u32, Box<[u8]>)> {
    driver
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn)))
                if turn.task == task =>
            {
                Some((turn.attempt, turn.turn, turn.transcript.clone()))
            }
            _ => None,
        })
        .collect()
}

fn document(driver: &mut Driver, read: reads::Read) -> documents::Document {
    let since = driver.delivered.len();
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: READ,
        sign_in: driver.session(),
        read: Box::new(read),
    }));
    driver.pump();
    driver.delivered[since..]
        .iter()
        .find_map(|delivery| match delivery {
            Delivery::Read { reply, answer: reads::Answer::Document(document) } if *reply == READ => {
                Some(*document.clone())
            }
            _ => None,
        })
        .expect("requested document")
}

fn chip(driver: &mut Driver, task: u64) -> documents::Chip {
    let documents::Document::Task(task) = document(driver, reads::Read::Task { task }) else {
        panic!("chat document");
    };
    task.chip
}

#[test]
fn a_lost_attempts_reply_never_commits_and_its_replacement_commits_it_once() {
    let (mut driver, first) = chat_on_first_worker();
    link_other(&mut driver);
    // The link does not carry the worker's unfinished turn. Only its Waiting report reaches the engine.
    report_phase(&mut driver, FIRST, &first, jig_domain_link::Phase::Waiting);
    let grace = lose_first(&mut driver);
    driver.fire_at(Duration::from_nanos(grace.as_nanos() - TICK.as_nanos()));
    driver.pump();
    assert_eq!(placements(&driver), [(FIRST, first.task, first.attempt)], "no replacement within grace");
    let retry = retry_after_grace(&mut driver, grace);
    assert_eq!(placements(&driver), [(FIRST, first.task, first.attempt), (OTHER, first.task, retry.attempt)]);
    assert!(retry.attempt > first.attempt);
    assert!(retry.transcript.is_empty(), "the retry resumes with no committed turns");
    reply_and_park(&mut driver, OTHER, &retry, REPLY);
    close(&mut driver, first.task, CLOSE_FIRST);

    let activity = activity(&driver, first.task);
    let journal: Vec<_> = activity
        .iter()
        .filter_map(|row| match row.line {
            tasks::Line::RunBegan { attempt, .. } => Some((attempt, None)),
            tasks::Line::RunEnded { attempt, end, .. } => Some((attempt, Some(end))),
            _ => None,
        })
        .collect();
    assert_eq!(
        journal,
        [
            (first.attempt, None),
            (first.attempt, Some(LOST)),
            (retry.attempt, None),
            (retry.attempt, Some(tasks::EndKind::Parked)),
        ]
    );
    assert_eq!(turns(&driver, first.task), [(retry.attempt, FIRST_TURN, REPLY.into())]);
    let turn_lines: Vec<_> = activity
        .iter()
        .filter_map(|row| match row.line {
            tasks::Line::Turn { attempt, turn, .. } => Some((attempt, turn)),
            _ => None,
        })
        .collect();
    assert_eq!(turn_lines, [(retry.attempt, FIRST_TURN)]);
}

#[test]
fn a_fenced_attempts_late_traffic_changes_no_turn_and_its_worker_takes_new_work() {
    let (mut driver, first) = chat_on_first_worker();
    link_other(&mut driver);
    let grace = lose_first(&mut driver);
    let retry = retry_after_grace(&mut driver, grace);
    report_phase(&mut driver, OTHER, &retry, jig_domain_link::Phase::Waiting);
    let observation = chip(&mut driver, first.task).details.observation.expect("replacement's waiting observation");
    let old_activity = attempt_activity(&driver, &first);
    let committed = turns(&driver, first.task);
    let live_retry = live(&driver, first.task).clone();

    let listing = protocol::hosting(first.task, first.attempt, jig_domain_link::Phase::Waiting);
    driver.send(protocol::hello_at(RETURNED, FIRST_POSITION, ONE_SLOT, Box::new([listing])));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|item| matches!(item,
            Delivery::Link { channel, down: host::LinkDown::Cancel { run, attempt } }
                if *channel == RETURNED && run.raw() == first.task && attempt.raw() == first.attempt
        )),
        "the returning worker is told to cancel its fenced listing"
    );
    report_phase(&mut driver, RETURNED, &first, jig_domain_link::Phase::Ending);
    reply_and_park(&mut driver, RETURNED, &first, REPLY);
    assert!(
        driver.delivered.iter().any(|item| matches!(item,
            Delivery::Link { channel, down: host::LinkDown::Acknowledge { run, attempt } }
                if *channel == RETURNED && run.raw() == first.task && attempt.raw() == first.attempt
        )),
        "the returned worker's stale terminal reached the engine"
    );
    assert_eq!(chip(&mut driver, first.task).details.observation, Some(observation));
    assert_eq!(attempt_activity(&driver, &first), old_activity);
    assert_eq!(turns(&driver, first.task), committed);
    assert_eq!(live(&driver, first.task), &live_retry);

    let next = start_chat(&mut driver, NEXT_CHAT);
    let new_run = driver.last_assignment();
    assert_eq!(
        placements(&driver),
        [(FIRST, first.task, first.attempt), (OTHER, first.task, retry.attempt), (RETURNED, next, new_run.attempt),]
    );
    assert_eq!(live(&driver, first.task), &live_retry, "the replacement is still live beside the new chat");
    reply_and_park(&mut driver, RETURNED, &new_run, b"Second worker replied.");
    reply_and_park(&mut driver, OTHER, &retry, REPLY);
    close(&mut driver, first.task, CLOSE_FIRST);
    close(&mut driver, next, CLOSE_NEXT);
    assert_eq!(turns(&driver, first.task), [(retry.attempt, FIRST_TURN, REPLY.into())]);
}

#[test]
fn a_late_workers_hello_takes_the_task_waiting_for_a_slot() {
    let (mut driver, first) = chat_on_first_worker();
    let occupied = live(&driver, first.task).clone();
    let next = start_chat(&mut driver, NEXT_CHAT);
    let waiting = chip(&mut driver, next);
    assert_eq!(live(&driver, next).phase, tasks::Phase::Active(tasks::Active::Due));
    assert!(live(&driver, next).open_attempt.is_none());
    assert!(waiting.details.attempt.is_none());
    assert!(waiting.details.observation.is_none());
    assert_eq!(placements(&driver), [(FIRST, first.task, first.attempt)]);

    link_other(&mut driver);
    let placed = driver.last_assignment();
    assert_eq!(placements(&driver), [(FIRST, first.task, first.attempt), (OTHER, next, placed.attempt)]);
    assert_eq!(live(&driver, first.task), &occupied, "the first run keeps its slot");
}

fn worker_name(driver: &mut Driver, channel: Token) -> Box<[u8]> {
    let documents::Document::Deployment(deployment) = document(driver, reads::Read::Deployment) else {
        panic!("deployment document");
    };
    deployment
        .hosts
        .iter()
        .find(|host| host.channel == channel.raw() && host.connected)
        .and_then(|host| host.name.clone())
        .expect("connected worker's configured name")
}

#[test]
fn a_worker_back_at_its_position_after_its_run_ended_takes_new_work_there() {
    let (mut driver, first) = chat_on_first_worker();
    reply_and_park(&mut driver, FIRST, &first, REPLY);
    let name = worker_name(&mut driver, FIRST);
    driver.send(protocol::lost(FIRST));
    driver.settle();
    let mut fresh = protocol::hello_at(RETURNED, FIRST_POSITION, ONE_SLOT, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut fresh
    else {
        panic!("worker hello")
    };
    hello.fresh = true;
    driver.send(fresh);
    driver.settle();
    assert_eq!(worker_name(&mut driver, RETURNED), name, "fresh hello keeps the configured position");

    let next = start_chat(&mut driver, NEXT_CHAT);
    let placed = driver.last_assignment();
    assert_eq!(placements(&driver), [(FIRST, first.task, first.attempt), (RETURNED, next, placed.attempt)]);
}
