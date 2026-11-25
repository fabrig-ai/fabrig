//! Account-use acceptance for the open provider failures (domain/testing-fakes.md, 1.3).
use jig_core_world::fixtures::{PROJECT, ask};
use jig_core_world::world::{Setup, World};
use jig_domain_accounts as accounts;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

const SEED: u64 = 76_001;
const PROVIDER: u32 = 1;

fn service(world: &mut World, key: u8, name: &[u8]) -> u64 {
    ask(world, key, people::Ask::MakeService { project: PROJECT, name: name.into(), role: people::Role::Owner });
    let Some(people::Reply::Outcome(people::Outcome::ServiceMade { person })) = world.people_answers.last() else {
        panic!("service made: {:?}", world.people_answers.last());
    };
    *person
}

fn give_use(world: &mut World, key: u8, account: u32, party: u64) {
    ask(
        world,
        key,
        people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::SetUses { account, uses: Some(Box::new([people::Use::Party(party)])) },
        },
    );
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::AccountChanged)));
}

fn serves(world: &World, account: u32, party: u64) -> bool {
    world
        .domain
        .core()
        .account_coverage(tasks::Payer::Party(party), Some(PROJECT))
        .iter()
        .any(|coverage| coverage.account == account && coverage.serves)
}

/// P76: granting a new service the plan account must preserve the earlier service's
/// run coverage, without keeping a list by hand. `SetUses` replaces that list today,
/// so B loses coverage. The fixing work may give the grant step another request,
/// keeping the two coverage assertions.
#[test]
#[ignore = "P76, open: provider failures (jig/docs/design/domain/testing-fakes.md, 1.3)"]
fn a_new_party_gets_account_uses() {
    let (config, mut limits) = jig_core_world::world::fixture(SEED, false);
    limits.core.people.people = 4;
    limits.core.people.holdings = 4;
    limits.core.accounts.accounts = 2;
    jig_core_world::peers::journal(&mut limits);
    let mut world = World::new(Setup { seed: SEED, configuration: Some(config), limits, ..Setup::default() });
    ask(&mut world, 40, people::Ask::AccountSignIn { holder: people::AccountHolder::Own, provider: PROVIDER });
    let Some(people::Reply::Outcome(people::Outcome::AccountSignInStarted { source, .. })) =
        world.people_answers.last()
    else {
        panic!("plan sign-in: {:?}", world.people_answers.last());
    };
    let account = world.domain.accounts().source(*source).expect("signed-in source").account;
    let plan = world.domain.accounts().account(account).expect("holder's account");
    assert!(plan.plan());
    assert!(matches!(plan.holder(), accounts::Holder::Party(_)));

    let b = service(&mut world, 41, b"B");
    give_use(&mut world, 42, account, b);
    assert!(serves(&world, account, b), "B's run is covered before the new service exists");
    let s = service(&mut world, 43, b"S");
    give_use(&mut world, 44, account, s);

    assert!(serves(&world, account, s), "P76: the plan account pays for S's run");
    assert!(serves(&world, account, b), "P76: giving S the account's use must keep B's run covered");
}
