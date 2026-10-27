//! A move's completion must remain possible at the retained-answer boundary.
use super::*;

const SEED: u64 = 211;
// The fixture's chat, the account sign-in, and its move out each retain one answer.
const LAST_REQUEST: u32 = 3;
const EXPORT: u8 = 50;
const FRESH_EXPORT: u8 = 51;
const WRITTEN: u8 = 52;
const RETRY: u8 = 53;

fn change(change: people::AccountChange) -> people::Ask {
    people::Ask::AccountChange { holder: people::AccountHolder::Own, change }
}

fn moving(requests: u32) -> (World, u32, u32, u32) {
    let (config, mut limits) = signing_configuration(SEED);
    limits.core.people.requests = requests;
    jig_core_world::peers::journal(&mut limits);
    let mut world = World::configured(SEED, false, config, limits);
    let (account, source) = signed_in(&mut world);
    let number = world.domain.accounts().next_number();
    ask(&mut world, EXPORT, change(people::AccountChange::MoveOut { account }));
    world.take_moved(source).expect("the export received its fake record");
    (world, account, source, number)
}

fn expired_and_restarted(requests: u32) -> (World, u32, u32) {
    let (mut world, account, source, _) = moving(requests);
    // No Written reaches the engine. Fire the shipped move deadline, before answer retention.
    let deadline = signing_configuration(SEED).1.core.accounts.move_deadline;
    let (now, wall) = world.clocks();
    world.fire_due(now.saturating_add(deadline), Wall::from_nanos(wall.as_nanos() + deadline.as_nanos()));
    world.restart_with(signing_configuration(SEED).0);
    let identity = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            root::Record::Core(core::Record::People(people::Stored::Person { identity, .. })) => Some(identity.clone()),
            _ => None,
        })
        .expect("the fixture's person survives restart");
    world.send(root::Event::Core(core::Event::SignIn {
        held: None,
        reply_to: skein_lib::ReplyTo::new(Token::new(9000)),
        identity,
    }));
    (world, account, source)
}

fn export_again(world: &mut World, account: u32, source: u32) {
    let number = world.domain.accounts().next_number();
    assert_eq!(
        ask(world, FRESH_EXPORT, change(people::AccountChange::MoveOut { account })),
        Some(&people::Reply::Outcome(people::Outcome::AccountMovingOut { account, number, source }))
    );
}

#[test]
fn an_unacknowledged_move_can_export_again_after_restart() {
    let (mut world, account, source) = expired_and_restarted(signing_configuration(SEED).1.core.people.requests);
    export_again(&mut world, account, source);
}

#[test]
fn a_last_slot_move_can_export_again_after_restart() {
    let (mut world, account, source) = expired_and_restarted(LAST_REQUEST);
    export_again(&mut world, account, source);
}

#[test]
fn a_last_slot_move_can_acknowledge_its_written_record() {
    let (mut world, _, _, number) = moving(LAST_REQUEST);
    assert_eq!(
        ask(&mut world, WRITTEN, change(people::AccountChange::Written { number })),
        Some(&people::Reply::Outcome(people::Outcome::AccountChanged))
    );
}

#[test]
fn a_last_slot_move_gets_a_terminal_retry_answer() {
    let (mut world, _, source) = expired_and_restarted(LAST_REQUEST);
    // The cancellation refreshed the source; retry must say it is not limited.
    assert_eq!(
        ask(&mut world, RETRY, change(people::AccountChange::Resume { source })),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotOffered)))
    );
}
