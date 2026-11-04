//! A pace holds news until the span since the last run start (domain/tasks.md, 8.3).
use jig_domain_tasks::{
    Active, End, Event, MessageKind, NewsClass, Party, Phase, Refusal, ResultsWake, Subscription, SubscriptionKind,
    WakeRule, Word,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::Duration;

const PERSON: Party = Party::Person(9);
const COORDINATOR: u64 = 1;
const DELEGATE: u64 = 2;
const TOPIC: u64 = 10;
const SPAN: Duration = Duration::from_nanos(1_000);
// Making the task delivers its initial activation; each story adds one wake.
const INITIAL_ACTIVATIONS: u32 = 1;
const WOKEN_ACTIVATIONS: u32 = 2;

fn parked(seed: u64, delegate: bool) -> World {
    let mut world = World::new(seed, LIMITS);
    let mut watch = task(COORDINATOR, &[]);
    watch.flags =
        jig_domain_tasks::Flags { conversation: true, ended_by: jig_domain_tasks::EndedBy::Requester, ..watch.flags };
    watch.wake.news = WakeRule::Paced { span: SPAN };
    watch.wake.results = ResultsWake::Each;
    world.make(PERSON, vec![watch]);
    if delegate {
        world.make(Party::Task(COORDINATOR), vec![task(DELEGATE, &[])]);
    }
    let reply_to = world.to();
    world.send(Event::SubscribeTopic {
        reply_to,
        task: COORDINATOR,
        subscription: Subscription { number: TOPIC, kind: SubscriptionKind::Topic { connector: 0, topic: TOPIC } },
    });
    world.claim(COORDINATOR, 1);
    world.terminal(COORDINATOR, End::Parked);
    world
}

fn news(world: &mut World, number: u64) {
    world.send(Event::Notice {
        task: COORDINATOR,
        word: Word {
            number,
            from: Party::Task(COORDINATOR),
            kind: MessageKind::News { subscription: TOPIC, class: NewsClass::Wakes },
            words: Box::new([]),
            at: world.env.wall,
            hits: 1,
            eligible: false,
        },
    });
}

#[test]
fn paced_news_wakes_at_once_after_a_quiet_span() {
    let mut world = parked(870, false);
    world.elapse(SPAN);
    news(&mut world, 1);
    assert_eq!(world.activations(COORDINATOR), WOKEN_ACTIVATIONS);
    assert_eq!(world.record(COORDINATOR).phase, Phase::Active(Active::Due));
    assert!(world.contexts[&COORDINATOR].inbox.iter().all(|word| word.eligible));
}

#[test]
fn paced_news_within_the_span_waits_for_its_end() {
    let mut world = parked(871, false);
    news(&mut world, 1);
    assert_eq!(world.activations(COORDINATOR), INITIAL_ACTIVATIONS);
    assert_eq!(world.record(COORDINATOR).phase, Phase::Active(Active::Idle));
    news(&mut world, 2);
    world.elapse(Duration::from_nanos(999));
    assert_eq!(world.activations(COORDINATOR), INITIAL_ACTIVATIONS);
    assert!(!world.record(COORDINATOR).inbox[0].eligible);
    world.elapse(Duration::from_nanos(1));
    assert_eq!(world.activations(COORDINATOR), WOKEN_ACTIVATIONS);
    // News from one topic merges, retaining both arrivals in its hit count.
    let inbox = &world.contexts[&COORDINATOR].inbox;
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].hits, 2);
    assert!(inbox[0].eligible);
    world.elapse(SPAN);
    assert_eq!(world.activations(COORDINATOR), WOKEN_ACTIVATIONS);
}

#[test]
fn a_result_wakes_through_a_pace_and_carries_the_news() {
    let mut world = parked(872, true);
    news(&mut world, 1);
    assert_eq!(world.activations(COORDINATOR), INITIAL_ACTIVATIONS);
    world.claim(DELEGATE, 2);
    world.finish(DELEGATE);
    world.settle(DELEGATE);
    assert_eq!(world.activations(COORDINATOR), WOKEN_ACTIVATIONS);
    let inbox = &world.contexts[&COORDINATOR].inbox;
    assert_eq!(inbox.len(), 2);
    assert!(matches!(inbox[0].kind, MessageKind::News { .. }));
    assert!(!inbox[0].eligible);
    assert!(matches!(inbox[1].kind, MessageKind::Result(_)));
}

#[test]
fn a_held_pace_survives_a_restart() {
    let mut world = parked(873, false);
    news(&mut world, 1);
    world.elapse(Duration::from_nanos(500));
    world.restart();
    assert_eq!(world.activations(COORDINATOR), INITIAL_ACTIVATIONS);
    world.elapse(Duration::from_nanos(500));
    assert_eq!(world.activations(COORDINATOR), WOKEN_ACTIVATIONS);
    assert!(world.contexts[&COORDINATOR].inbox[0].eligible);
}

#[test]
fn a_zero_span_is_refused() {
    let mut world = World::new(874, LIMITS);
    let mut watch = task(COORDINATOR, &[]);
    watch.wake.news = WakeRule::Paced { span: Duration::ZERO };
    assert!(matches!(world.make(PERSON, vec![watch]), Reply::Refused(problem) if problem.why == Refusal::Spec));
    assert!(world.activations.is_empty());
    assert!(!world.records.contains_key(&jig_domain_tasks::Key::Live(COORDINATOR)));
}
