use skein_forge::api::Error;
use skein_forge_connection::{Request, Settings, Stopped, Value, settings::Forge};
use skein_forge_machine_world::{
    fixtures::{self, FORGE, REPO, TOKEN, TOKEN_B, WRITER, access, locate, locate_script},
    hooks::{self as deliveries, Listener},
    machine::{Answer, Machine, Pause},
};
use skein_lib::{Duration, List};

const SEED: u64 = 7;
const SECOND: Duration = Duration::from_secs(1);
// The fixture's two start reads precede a call's exchange.
const CALL_REQUEST: usize = 2;
// The fake's tokens live for an hour; this crosses both renewal and several log polls.
const AFTER_RENEWAL: Duration = Duration::from_secs(3601);
const LOG_POLL: Duration = Duration::from_secs(10);
const GITHUB_START: [&str; 5] = [
    "GET /app/installations?per_page=100&page=1",
    "GET /app/installations?per_page=100&page=2",
    "POST /app/installations/1/access_tokens",
    "POST /app/installations/1/access_tokens",
    "POST /graphql",
];
fn forge() -> Forge {
    Forge {
        access: Some(access(TOKEN, WRITER, None)),
        ..fixtures::settings().forges.into_boxed().into_iter().next().expect("fixture forge")
    }
}
fn empty() -> Settings {
    Settings { forges: List::with_capacity(0), ..fixtures::settings() }
}
fn world(script: Vec<Answer>) -> Machine {
    Machine::with(SEED, fixtures::limits(), empty(), script)
}
fn add(world: &mut Machine, settings: Forge) {
    world.action(Request::Add { forge: settings.number, settings });
}
fn remove(world: &mut Machine) {
    world.action(Request::Remove { forge: FORGE });
}
fn failed(world: &Machine, call: u64, error: Error, cost: u32) {
    let answer = world.answered(call).expect("call ended");
    assert_eq!((&answer.result, answer.cost), (&Err(error), cost));
}
fn located(world: &Machine, call: u64) {
    let answer = world.answered(call).expect("located");
    assert!(answer.result.is_ok());
    assert_eq!(answer.cost, 1);
}

#[test]
fn a_forge_added_after_start_starts_says_connected_and_answers_its_calls() {
    let mut world = world(locate_script());
    add(&mut world, forge());
    world.call(1, REPO, locate());
    world.run_for(SECOND);
    assert_eq!(world.connected_forges(), &[FORGE]);
    located(&world, 1);
    assert!(world.script_done());
    assert!(world.close());
}

#[test]
fn a_forge_added_with_no_access_answers_not_connected_until_a_connect() {
    let mut world = world(locate_script());
    add(&mut world, Forge { access: None, ..forge() });
    world.call(1, REPO, locate());
    world.run_for(SECOND);
    failed(&world, 1, Error::NotConnected, 0);
    assert_eq!(world.connections_opened(), 0);
    assert!(world.connected_forges().is_empty());
    world.action(Request::Connect { forge: FORGE, access: access(TOKEN, WRITER, None) });
    world.call(2, REPO, locate());
    world.run_for(SECOND);
    assert_eq!(world.connected_forges(), &[FORGE]);
    located(&world, 2);
    assert!(world.close());
}

#[test]
fn an_add_with_a_refused_access_says_stopped_invalid() {
    let mut world = world(vec![]);
    add(&mut world, Forge { access: Some(access(b"", WRITER, None)), ..forge() });
    assert_eq!(world.stopped(), Some((FORGE, Stopped::Invalid { value: Value::Token })));
    assert_eq!(world.connections_opened(), 0);
    world.call(1, REPO, locate());
    world.run_for(SECOND);
    failed(&world, 1, Error::NotConnected, 0);
    assert!(world.close());
}

#[test]
fn a_removal_ends_a_waiting_call_unavailable_and_a_written_one_as_its_deadline_would() {
    for (pause, error, cost) in [(0, Error::Unavailable, 0), (CALL_REQUEST, Error::Timeout, 1)] {
        let mut world = Machine::new(SEED, locate_script());
        world.pause_at(pause, Pause::Reading(0));
        world.call(1, REPO, locate());
        world.run_for(SECOND);
        world.hold_settlement();
        remove(&mut world);
        failed(&world, 1, error, cost);
        let connected = world.connected_forges().len();
        let deliveries = world.deliveries().len();
        world.run_for(fixtures::limits().deadline.saturating_mul(2));
        world.release_settlement();
        assert_eq!(world.start(FORGE), None);
        assert_eq!(world.deliveries().len(), deliveries);
        assert_eq!(world.connected_forges().len(), connected);
        assert_eq!(world.stopped(), None);
        assert!(world.addresses().is_empty());
        assert!(world.close());
    }
}

#[test]
fn a_call_and_a_connect_after_a_removal_are_answered_as_for_a_forge_never_added() {
    let mut world = world(vec![]);
    add(&mut world, Forge { access: None, ..forge() });
    world.call(1, REPO, locate());
    world.run_for(SECOND);
    let held = world.answered(1).expect("answer handed to owner").clone();
    world.action(Request::Refused(held.clone()));
    remove(&mut world);
    remove(&mut world);
    world.action(Request::Offer);
    assert_eq!(world.answered(1), Some(&held));
    world.action(Request::Connect { forge: FORGE, access: access(TOKEN, WRITER, None) });
    world.action(Request::Installations { forge: FORGE, accepted: Box::from([1]) });
    world.call(2, REPO, locate());
    world.run_for(SECOND);
    failed(&world, 2, Error::Unavailable, 0);
    assert!(world.connected_forges().is_empty());
    assert_eq!(world.stopped(), None);
    assert_eq!(world.connections_opened(), 0);
    assert!(world.close());
}

#[test]
fn a_removed_forge_added_again_starts_a_new_generation() {
    let script = locate_script().into_iter().chain(locate_script()).collect();
    let mut world = world(script);
    add(&mut world, forge());
    world.call(1, REPO, locate());
    world.run_for(SECOND);
    world.hold_settlement();
    remove(&mut world);
    add(&mut world, Forge { access: Some(access(TOKEN_B, WRITER, None)), ..forge() });
    world.release_settlement();
    world.call(2, REPO, locate());
    world.run_for(SECOND);
    located(&world, 1);
    located(&world, 2);
    assert_eq!(world.connected_forges(), &[FORGE, FORGE]);
    assert_eq!(world.stopped(), None);
    assert!(world.script_done());
    assert!(world.close());
}

#[test]
fn a_removed_github_forge_drops_its_installations_tokens_and_puller() {
    let limits = skein_forge_connection::Limits { log_poll: LOG_POLL, ..fixtures::limits() };
    let mut world = Machine::github(
        SEED,
        limits,
        fixtures::github_settings(),
        GITHUB_START.into_iter().map(Answer::served).collect(),
    );
    world.action(Request::Installations { forge: FORGE, accepted: Box::from([1]) });
    world.action(Request::Connect { forge: FORGE, access: fixtures::github_access() });
    world.run_for(SECOND);
    let wall = world.wall();
    world.github_fake_mut().release(
        skein_forge::github::delivery::Delivery::Issue {
            repository: 2,
            number: 1,
            body: skein_forge::forgejo::document::Body::Kept(Box::from([])),
        },
        wall,
    );
    world.log_requests(vec!["GET /app/hook/deliveries?per_page=8".into()]);
    world.run_for(LOG_POLL);
    let before = (
        world.connected_forges().len(),
        world.installation_events().len(),
        world.git_tokens().len(),
        world.facts().len(),
        world.requests().len(),
    );
    remove(&mut world);
    world.action(Request::Installations { forge: FORGE, accepted: Box::from([1]) });
    world.run_for(AFTER_RENEWAL);
    assert_eq!(
        (
            world.connected_forges().len(),
            world.installation_events().len(),
            world.git_tokens().len(),
            world.facts().len(),
            world.requests().len()
        ),
        before
    );
    assert_eq!(world.start(FORGE), None);
    assert!(world.addresses().is_empty());
    assert!(world.script_done());
    assert!(world.close());
}

#[test]
fn a_webhook_delivery_to_a_removed_forge_is_answered_not_found() {
    let mut world = Listener::new(SEED, fixtures::hook_limits(), fixtures::hook_settings());
    // A body whose authentication began before removal must not produce a new hint.
    let unfinished = world.accept();
    let delivery = deliveries::push(b"main", 7);
    world.write(unfinished, &delivery.head);
    world.request(Request::Remove { forge: FORGE });
    assert!(world.peer(unfinished).closed);
    let peer = world.accept();
    world.send(peer, &delivery);
    assert_eq!(world.status(peer), Some(404));
    assert!(world.hints().is_empty());
}

#[test]
fn close_after_a_removal_says_closed_once() {
    let mut world = world(locate_script().into_iter().take(CALL_REQUEST).collect());
    add(&mut world, forge());
    world.hold_settlement();
    remove(&mut world);
    world.action(Request::Close);
    assert_eq!(world.closed_events(), 0, "removed sockets still belong to io");
    add(&mut world, forge());
    remove(&mut world);
    world.release_settlement();
    assert!(world.close());
    world.action(Request::Close);
    world.action(Request::Abort);
    add(&mut world, forge());
    remove(&mut world);
    world.run_for(SECOND);
    assert_eq!(world.closed_events(), 1);
    assert_eq!(world.connected_forges(), &[FORGE]);
    assert_eq!(world.requests().len(), CALL_REQUEST);
}
