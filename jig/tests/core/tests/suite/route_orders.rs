//! Synchronous child routes remain in one decision under watermark pressure.
use jig_core_world::direct::Driver;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token, Wall};

const FIRST: Token = Token::new(9701);
const LATER: Token = Token::new(9702);
const FIRST_KEY: [u8; 16] = [71; 16];
const LATER_KEY: [u8; 16] = [72; 16];
const PROJECT: u32 = 2;
// MakeProject fans out to Roles, OpenPeriod, PeriodNext and Decided.
const HANDOFFS: usize = 4;
const ONCE: usize = 1;

/// One entrance's route finishes within its decision, its widest fan-out
/// included, before a later entrance runs, with new work waiting at the
/// watermark.
#[test]
fn a_wide_route_finishes_within_its_decision_before_a_later_entrance() {
    let mut driver = Driver::project_maker(9701);
    let session = begin_story(&mut driver);
    driver.send(ask(session, FIRST, FIRST_KEY, people::Ask::MakeProject { name: "Tally".into(), policy: 1 }));
    hold_later(&mut driver, session, PROJECT);
    driver.settle();

    let (first_index, writes) = answer_transaction(&driver, FIRST_KEY);
    // Every sibling's durable consequence, plus the policy, is in that transaction.
    let mut consequences = Vec::new();
    for write in writes {
        match write {
            root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Policy {
                project: PROJECT,
                name,
                ..
            }))) if name.as_ref() == "Tally" => consequences.push("policy"),
            root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Roles {
                project: PROJECT,
                ..
            }))) => consequences.push("roles"),
            root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(
                tasks::FundingRecord { funder: tasks::Funder::Period { project: PROJECT, .. }, .. },
            )))) => consequences.push("period"),
            root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Answer {
                key,
                outcome,
                ..
            }))) if key.key == FIRST_KEY => {
                assert_eq!(*outcome, people::Outcome::ProjectMade { project: PROJECT });
                consequences.push("answer");
            }
            root::Write::CopySettingsRevision(_)
            | root::Write::Wires
            | root::Write::Save(_)
            | root::Write::Erase(_) => {}
        }
    }
    assert_eq!(consequences, ["policy", "roles", "period", "answer"], "all {HANDOFFS} hand-offs finish together");
    assert!(answer_transaction(&driver, LATER_KEY).0 > first_index);
    let replies: Vec<_> = driver
        .released
        .iter()
        .filter_map(|request| {
            if let root::Request::Deliver(root::Delivery::Core(core::Held::PeopleReply { to, reply })) = request
                && [ReplyTo::new(FIRST), ReplyTo::new(LATER)].contains(to)
            {
                Some((to, reply))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        replies,
        [
            (&ReplyTo::new(FIRST), &people::Reply::Outcome(people::Outcome::ProjectMade { project: PROJECT })),
            (&ReplyTo::new(LATER), &people::Reply::Outcome(people::Outcome::Renamed { project: PROJECT })),
        ]
    );
    assert!(driver.domain.quiescent() && !driver.stopped);
}

// The chat worker and project are the direct world's signed-in fixture.
const WORKER: Token = Token::new(7);
const WORKER_NUMBER: u32 = 0;
const RECONNECTED_WORKER: Token = Token::new(8);
const STALE_RUN: Token = Token::new(9801);
const STALE_ATTEMPT: Token = Token::new(9802);
// The live run and one stale listing share the reconnecting worker.
const HOSTED: u32 = 2;
const STOP_BOUND: Duration = Duration::from_millis(100);
const CHAT_PROJECT: u32 = 1;
const SUBSCRIBE: Token = Token::new(9703);
const SUBSCRIPTION: u64 = 1;
const TURN: u32 = 1;
const TIMER_DELAY: Duration = Duration::from_millis(1);

/// A due timer generates a numbered notice while task requests drain. Its
/// subscription removal, inbox write and delivery all precede later work.
#[test]
fn a_task_route_finishes_its_generated_requests_within_its_decision() {
    let mut driver = Driver::chat(9702);
    let session = driver.session();
    let task = driver.assignment().task;
    let at = Wall::from_nanos(driver.env.wall.as_nanos() + TIMER_DELAY.as_nanos());
    driver.send(root::Event::Core(core::Event::Tasks(tasks::Event::Subscribe {
        reply_to: ReplyTo::new(SUBSCRIBE),
        task,
        subscription: tasks::Subscription {
            number: SUBSCRIPTION,
            kind: tasks::SubscriptionKind::Timer { at, period: None },
        },
    })));
    driver.settle();
    driver.env.now = driver.env.now.checked_add(TIMER_DELAY).expect("fixture timer");
    driver.env.wall = at;
    clear_story(&mut driver);
    driver.send(root::Event::Timer(core::Timer::Tasks));
    hold_later(&mut driver, session, CHAT_PROJECT);
    assert!(timer_deliveries(&driver, task).is_empty(), "generated notice waits for durability");
    driver.settle();

    let notice_transactions: Vec<_> = driver.transactions.iter().enumerate().filter(|(_, writes)| {
        writes.iter().any(|write| matches!(write,
            root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))))
                if line.task == task && matches!(line.line,
                    tasks::Line::Heard(tasks::Word { kind: tasks::MessageKind::Timer { subscription: SUBSCRIPTION }, .. }))
        ))
    }).collect();
    assert_eq!(notice_transactions.len(), ONCE, "the generated notice has one decision");
    let (first_index, writes) = notice_transactions[0];
    let live: Vec<_> = writes
        .iter()
        .filter_map(|write| {
            if let root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) = write
                && row.number == task
            {
                Some(row)
            } else {
                None
            }
        })
        .collect();
    assert!(
        live.iter().any(|row| row
            .inbox
            .iter()
            .any(|word| word.kind == tasks::MessageKind::Timer { subscription: SUBSCRIPTION })),
        "the generated inbox write is in the timer decision"
    );
    assert!(
        live.last().expect("timer saves its task").subscriptions.is_empty(),
        "the one-shot subscription is removed in the same decision"
    );
    let task_decisions: Vec<_> = driver
        .transactions
        .iter()
        .enumerate()
        .filter_map(|(index, writes)| {
            writes
                .iter()
                .any(|write| {
                    matches!(
                        write,
                        root::Write::Save(root::Record::Core(core::Record::Tasks(_)))
                            | root::Write::Erase(root::Key::Core(core::Key::Tasks(_)))
                    )
                })
                .then_some(index)
        })
        .collect();
    assert_eq!(task_decisions, [first_index], "all generated task writes finish in one decision");
    assert!(answer_transaction(&driver, LATER_KEY).0 > first_index);
    assert_eq!(timer_deliveries(&driver, task).len(), ONCE, "the generated notice completes once");
    assert!(driver.domain.quiescent() && !driver.stopped);
}

/// `MakeService` steps people again inside `tag_people`, appending its person
/// and roles before the keyed terminal, with a later entrance held.
#[test]
fn a_people_route_finishes_its_continuation_within_its_decision() {
    let mut driver = Driver::chat(9703);
    let session = begin_story(&mut driver);
    driver.send(ask(
        session,
        FIRST,
        FIRST_KEY,
        people::Ask::MakeService {
            project: CHAT_PROJECT,
            name: b"Service".as_slice().into(),
            role: people::Role::Owner,
        },
    ));
    hold_later(&mut driver, session, CHAT_PROJECT);
    assert!(story_replies(&driver).is_empty(), "the service answer waits for durability");
    driver.settle();

    let (first_index, writes) = answer_transaction(&driver, FIRST_KEY);
    let mut consequences = Vec::new();
    let mut service = None;
    for write in writes {
        match write {
            root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Person {
                number,
                identity,
            }))) if identity.name.as_ref() == b"Service" => {
                service = Some(*number);
                consequences.push("person");
            }
            root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Roles {
                project: CHAT_PROJECT,
                holdings,
            }))) => {
                assert!(
                    holdings
                        .iter()
                        .any(|holding| Some(holding.person) == service && holding.role == people::Role::Owner)
                );
                consequences.push("roles");
            }
            root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Answer {
                key,
                outcome,
                ..
            }))) if key.key == FIRST_KEY => {
                assert_eq!(*outcome, people::Outcome::ServiceMade { person: service.expect("generated service") });
                consequences.push("answer");
            }
            root::Write::CopySettingsRevision(_)
            | root::Write::Wires
            | root::Write::Save(_)
            | root::Write::Erase(_) => {}
        }
    }
    assert_eq!(consequences, ["person", "roles", "answer"], "the keyed terminal follows the continuation");
    assert!(answer_transaction(&driver, LATER_KEY).0 > first_index);
    assert_eq!(story_replies(&driver), [FIRST, LATER]);
    assert!(driver.domain.quiescent() && !driver.stopped);
}

/// Reconnection cancels a stale listing while fleet requests drain; stopping
/// a live turn appends its busy notice. Both finish before later keyed work.
#[test]
fn a_fleet_route_finishes_its_follow_ups_within_its_decision() {
    let mut driver = Driver::reconnecting_chat(9704);
    let session = driver.session();
    let (task, attempt) = (driver.assignment().task, driver.assignment().attempt);
    clear_story(&mut driver);
    driver.send(ask(
        session,
        FIRST,
        FIRST_KEY,
        people::Ask::Say { project: CHAT_PROJECT, task, words: b"before stopping".as_slice().into() },
    ));
    driver.collect();
    driver.fill_to_watermark();
    reconnect_with_stale_attempt(&mut driver, task, attempt);
    driver.send(root::Event::Core(core::Event::Stopping));
    let returned = driver.returned;
    driver.send(root::Event::Turn {
        opens_window: true,
        refused: Box::new([]),
        channel: RECONNECTED_WORKER,
        task,
        attempt,
        turn: TURN,
        cumulative: 0,
        read: None,
        transcript: b"discarded while stopping".as_slice().into(),
    });
    driver.collect();
    assert_eq!(driver.returned, returned, "the stopping turn is admitted at the watermark");
    hold_later(&mut driver, session, CHAT_PROJECT);
    assert!(busy_turns(&driver).is_empty(), "the follow-up waits for preceding durability");
    assert!(stale_notices(&driver).is_empty(), "cancellation and acknowledgement wait for durability");
    driver.settle();

    assert_eq!(busy_turns(&driver), [(task, attempt, TURN)], "the recursive busy follow-up leaves once");
    assert_eq!(stale_notices(&driver), ["cancel", "acknowledge"], "the stale attempt is cancelled and forgotten once");
    let busy = driver
        .released
        .iter()
        .position(|request| {
            matches!(request, root::Request::Deliver(root::Delivery::Core(core::Held::TurnBusy { .. })))
        })
        .expect("busy follow-up released");
    let later = driver.released.iter().position(|request| matches!(request,
        root::Request::Deliver(root::Delivery::Core(core::Held::PeopleReply { to, reply: people::Reply::Again(people::Again::NotReady) }))
            if *to == ReplyTo::new(LATER)
    )).expect("later keyed entrance answered while stopping");
    assert!(busy < later, "the fleet continuation precedes the later entrance");
    assert_eq!(
        stale_notices_before(&driver, later),
        ["cancel", "acknowledge"],
        "the stale attempt's consequences precede the later entrance"
    );
    assert_eq!(story_replies(&driver), [FIRST, LATER]);
    let (first_index, _) = answer_transaction(&driver, FIRST_KEY);
    assert!(
        !driver.transactions[first_index..].iter().flatten().any(|write| matches!(write,
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(row))))
                if row.task == STALE_RUN.raw()
        )),
        "the stale answer is discarded without a task terminal"
    );
    assert!(
        !driver.transactions.iter().flatten().any(|write| matches!(write,
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Turn(row))))
                if row.task == task && row.attempt == attempt
        )),
        "the stopping route drops the turn instead of writing it"
    );
    assert!(driver.domain.quiescent() && !driver.stopped);
}

fn reconnect_with_stale_attempt(driver: &mut Driver, task: u64, attempt: u64) {
    driver.send(root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: WORKER })));
    driver.collect();
    driver.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: WORKER_NUMBER,
        channel: RECONNECTED_WORKER,
        hello: fleet::Hello {
            bounds: jig_core_world::fixtures::bounds(),
            stop_bound: STOP_BOUND,
            slots: HOSTED,
            workstreams: Box::new([]),
            hosting: Box::new([
                fleet::Hosted { run: Token::new(task), attempt: Token::new(attempt), phase: fleet::Phase::Active },
                fleet::Hosted { run: STALE_RUN, attempt: STALE_ATTEMPT, phase: fleet::Phase::Active },
            ]),
        },
    })));
    driver.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: RECONNECTED_WORKER,
        task: STALE_RUN.raw(),
        attempt: STALE_ATTEMPT.raw(),
        cumulative: 0,
        end: tasks::End::Parked,
    });
}

fn hold_later(driver: &mut Driver, session: u64, project: u32) {
    driver.collect();
    driver.fill_to_watermark();
    let returned = driver.returned;
    driver.send_retaining(ask(session, LATER, LATER_KEY, people::Ask::Rename { project, name: "Later".into() }));
    assert_eq!(driver.returned, returned + 1, "later keyed entrance waits at the watermark");
    assert_eq!(driver.events.len(), ONCE, "later work is retained");
}

fn answer_transaction(driver: &Driver, key: [u8; 16]) -> (usize, &[root::Write]) {
    let transactions: Vec<_> = driver
        .transactions
        .iter()
        .enumerate()
        .filter(|(_, writes)| {
            writes.iter().any(|write| matches!(write,
            root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Answer { key: saved, .. })))
                if saved.key == key
        ))
        })
        .collect();
    assert_eq!(transactions.len(), ONCE, "one transaction per keyed terminal");
    (transactions[0].0, transactions[0].1)
}

fn timer_deliveries(driver: &Driver, task: u64) -> Vec<&tasks::Word> {
    driver
        .released
        .iter()
        .filter_map(|request| {
            if let root::Request::Deliver(root::Delivery::Message { task: owner, word, .. }) = request
                && *owner == task
                && word.kind == (tasks::MessageKind::Timer { subscription: SUBSCRIPTION })
            {
                Some(word)
            } else {
                None
            }
        })
        .collect()
}

fn story_replies(driver: &Driver) -> Vec<Token> {
    driver
        .released
        .iter()
        .filter_map(|request| {
            if let root::Request::Deliver(root::Delivery::Core(core::Held::PeopleReply { to, .. })) = request {
                [FIRST, LATER].into_iter().find(|token| *to == ReplyTo::new(*token))
            } else {
                None
            }
        })
        .collect()
}

fn busy_turns(driver: &Driver) -> Vec<(u64, u64, u32)> {
    driver
        .released
        .iter()
        .filter_map(|request| {
            if let root::Request::Deliver(root::Delivery::Core(core::Held::TurnBusy {
                channel: RECONNECTED_WORKER,
                run,
                attempt,
                turn,
            })) = request
            {
                Some((run.raw(), attempt.raw(), *turn))
            } else {
                None
            }
        })
        .collect()
}

fn stale_notices(driver: &Driver) -> Vec<&'static str> {
    stale_notices_before(driver, driver.released.len())
}

fn stale_notices_before(driver: &Driver, end: usize) -> Vec<&'static str> {
    driver.released[..end]
        .iter()
        .filter_map(|request| {
            if let root::Request::Deliver(root::Delivery::Core(core::Held::Cancel {
                channel: RECONNECTED_WORKER,
                run: STALE_RUN,
                attempt: STALE_ATTEMPT,
            })) = request
            {
                Some("cancel")
            } else if let root::Request::Deliver(root::Delivery::Core(core::Held::Acknowledge {
                channel: RECONNECTED_WORKER,
                run: STALE_RUN,
                attempt: STALE_ATTEMPT,
            })) = request
            {
                Some("acknowledge")
            } else {
                None
            }
        })
        .collect()
}

fn begin_story(driver: &mut Driver) -> u64 {
    let session = driver.session();
    clear_story(driver);
    session
}

fn clear_story(driver: &mut Driver) {
    driver.transactions.clear();
    driver.released.clear();
}

fn ask(session: u64, reply: Token, key: [u8; 16], ask: people::Ask) -> root::Event {
    root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(reply),
        sign_in: session,
        key,
        ask,
    }))
}
