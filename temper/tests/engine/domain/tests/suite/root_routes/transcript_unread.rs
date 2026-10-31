use super::*;
use temper_engine_domain_world::direct::say_ask;

const CHANNEL: Token = Token::new(7);
// The direct fixture also bounds settlement at 200 steps.
const STEPS: usize = 200;
const BODY: &[u8] = b"old window";

/// Restore a parked task with an existing window, as an upgrade would.
fn parked_window(closed: bool) -> (Driver, engine::Assignment) {
    let (mut driver, first) = Driver::chat();
    driver.send(protocol::turn(CHANNEL, first.task, first.attempt, 1, 1, None, BODY));
    driver.settle();
    driver.answer(CHANNEL, first.task, first.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    let key = Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(first.task)));
    let mut record = driver.store.rows.get(&key).expect("parked run proof").clone();
    let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof))) = &mut record else {
        panic!("run proof row")
    };
    proof.window.as_mut().expect("committed opening turn").closed = closed;
    driver.store.seed(Box::new([record]), Box::new([]));
    let mut driver = Driver::new(driver.store);
    driver.hello(CHANNEL);
    driver.settle();
    driver.sign_in();
    driver.settle();
    (driver, first)
}

/// Keep the existing store's answer back at the public store/core seam.
fn take_transcript_reply(driver: &mut Driver, task: u64) -> engine::Event {
    for _ in 0..STEPS {
        if let Some(owner) = driver.loads.iter().rev().find_map(|(owner, range)| {
            matches!(range, engine::Range::Core(engine::CoreRange::TaskTranscript { task: named, .. })
                if *named == task)
            .then_some(*owner)
        }) && let Some(index) = driver.events.iter().position(|event| {
            matches!(event, engine::Event::Store(engine::StoreInput::Loaded { owner: named, .. })
                if *named == owner)
        }) {
            return driver.events.remove(index).expect("queued transcript answer");
        }
        driver.advance(true);
    }
    panic!("transcript load issued within the step budget")
}

fn owner(reply: &engine::Event) -> Token {
    let engine::Event::Store(engine::StoreInput::Loaded { owner, .. }) = reply else { panic!("held store page") };
    *owner
}

fn assignments(driver: &Driver) -> usize {
    driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Assigned { .. })).count()
}

fn unread_preparation(closed: bool) -> (Driver, engine::Assignment, Token) {
    let (mut driver, first) = parked_window(closed);
    driver.send(say_ask(driver.session(), first.task, 48));
    let reply = take_transcript_reply(&mut driver, first.task);
    let owner = owner(&reply);
    driver.send(engine::Event::Store(engine::StoreInput::Unloaded { owner }));
    (driver, first, owner)
}

#[test]
fn a_closed_unread_window_assigns_fresh_without_counting_or_retrying() {
    let (mut driver, first, _) = unread_preparation(true);
    driver.settle();
    let next = driver.last_assignment();
    assert!(next.attempt > first.attempt);
    assert_eq!(next.conversation, next.attempt, "the assignment starts a fresh conversation");
    assert!(next.transcript.is_empty());
    assert!(
        next.sections.iter().all(|section| {
            section.kind != engine::BriefKind::Core(brief::Core::TranscriptTail)
                || matches!(&section.body, engine::BriefBody::Text(text) if text.is_empty())
        }),
        "no unusable window text enters the brief"
    );
    assert_eq!(assignments(&driver), 1);
    assert_eq!(live(&driver, first.task).tries, tasks::Tries::NONE);
    assert_eq!(live(&driver, first.task).refusals, 0);
    assert_eq!(
        driver
            .loads
            .iter()
            .filter(|(_, range)| matches!(range, engine::Range::Core(engine::CoreRange::TaskTranscript { .. })))
            .count(),
        1,
        "one issued transcript load"
    );
    let loads = driver.loads.len();
    driver.pump();
    assert_eq!(driver.loads.len(), loads, "no read retry loop");
    assert_eq!(assignments(&driver), 1, "one fresh assignment");
}

#[test]
fn an_open_unread_window_keeps_the_transient_failure_policy() {
    let (mut driver, first, _) = unread_preparation(false);
    driver.pump();
    let row = live(&driver, first.task);
    assert_eq!(row.tries, tasks::Tries { transient: 1, ..tasks::Tries::NONE });
    assert_eq!(row.refusals, 0, "issued unread is a failure, not an admission refusal");
    assert!(matches!(row.phase, tasks::Phase::Active(tasks::Active::BackingOff { .. })));
    assert_eq!(assignments(&driver), 0);
    // The existing simulated retry starts fresh after dropping the open window.
    driver.fire_at(Duration::from_secs(2));
    driver.settle();
    let next = driver.last_assignment();
    assert!(next.attempt > first.attempt);
    assert_eq!(next.conversation, next.attempt);
    assert!(next.transcript.is_empty());
    assert_eq!(assignments(&driver), 1);
}

#[test]
fn a_stale_unread_terminal_cannot_fail_a_later_preparation() {
    let (mut driver, first, stale) = unread_preparation(true);
    driver.settle();
    let fresh = driver.last_assignment();
    // Acknowledge the offered messages so parking waits for the next wake.
    let read = fresh.messages.last().map(|message| message.number);
    driver.send(protocol::turn(CHANNEL, fresh.task, fresh.attempt, 1, 1, read, b"fresh window"));
    driver.settle();
    driver.answer(CHANNEL, fresh.task, fresh.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.send(say_ask(driver.session(), first.task, 49));
    let reply = take_transcript_reply(&mut driver, first.task);
    assert_ne!(owner(&reply), stale, "the later IO has its own generational owner");
    driver.send(engine::Event::Store(engine::StoreInput::Unloaded { owner: stale }));
    driver.pump();
    assert_eq!(assignments(&driver), 1, "later preparation still awaits its own page");
    driver.send(reply);
    driver.settle();
    let resumed = driver.last_assignment();
    assert!(resumed.attempt > fresh.attempt);
    assert_eq!(resumed.conversation, fresh.conversation);
    assert_eq!(resumed.transcript.as_ref(), [Box::<[u8]>::from(b"fresh window".as_slice())]);
    assert_eq!(assignments(&driver), 2);
    assert_eq!(live(&driver, first.task).tries, tasks::Tries::NONE);
    assert_eq!(live(&driver, first.task).refusals, 0);
}
