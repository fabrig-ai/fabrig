use jig_domain_people::{AccountChange, AccountHolder, Ask, InitialOwner, Outcome, Refusal, Reply, Role, USES, Use};
use jig_people_world::{
    Settings, World,
    fixtures::{OWNER, PROJECT, SECOND_SIGN_IN, SIGN_IN},
    identity,
};
const SERVICE_SIGN_IN: u64 = 12;

fn change(holder: AccountHolder, change: AccountChange) -> Ask {
    Ask::AccountChange { holder, change }
}
fn llm_key(digest: u8) -> Ask {
    change(AccountHolder::Own, AccountChange::AddKey { provider: 3, join: None, digest: [digest; 32] })
}
fn owners() -> World {
    let mut world = World::with_owners(
        Settings::calm(16),
        Box::new([InitialOwner { project: PROJECT, identity: identity(OWNER).key }]),
    );
    world.roles(PROJECT, Box::new([]));
    world.commit_all();
    world.sign_in_owner();
    world.sign_in_second();
    world.commit_all();
    world
}
#[test]
fn an_account_change_is_admitted_for_its_holder_with_no_role() {
    let mut world = owners();
    let changes = [
        AccountChange::AddKey { provider: 3, join: None, digest: [8; 32] },
        AccountChange::ReplaceKey { source: 1, digest: [9; 32] },
        AccountChange::MoveOut { account: 1 },
        AccountChange::Written { number: 2 },
        AccountChange::MoveIn { digest: [7; 32] },
        AccountChange::RemoveKey { source: 1 },
        AccountChange::Resume { source: 1 },
        AccountChange::Remove { account: 1 },
        AccountChange::MoveAccount { account: 1, place: 2 },
        AccountChange::MoveSource { account: 1, source: 1, place: 2 },
        AccountChange::SetUses { account: 1, uses: Some(Box::new([Use::Party(1), Use::Project(PROJECT)])) },
    ];
    for (at, value) in changes.into_iter().enumerate() {
        let expected = match value {
            AccountChange::LoginProgram { .. } => Outcome::Refused(Refusal::NoTerminal),
            AccountChange::CancelLogin { .. } | AccountChange::LoginEnded { .. } => {
                Outcome::Refused(Refusal::NotLending)
            }
            AccountChange::ReadLists { .. } => Outcome::AccountReading { count: 0 },
            AccountChange::AddKey { .. } | AccountChange::ReplaceKey { .. } => Outcome::AccountKeyKept { source: 1 },
            AccountChange::DeclareProgram { ref name, .. } => {
                Outcome::AccountProgramDeclared { source: 1, name: name.clone() }
            }

            AccountChange::Remove { .. } => Outcome::AccountRemoved { confirmed: true, failed_hosts: Box::new([]) },
            AccountChange::MoveOut { account } => Outcome::AccountMovingOut { account, number: 2, source: 1 },
            AccountChange::FinishSignIn { source, .. } => Outcome::AccountFinishing { source },
            AccountChange::MoveIn { .. } => Outcome::AccountMovedIn { account: 1, renewed: false },
            AccountChange::Written { .. }
            | AccountChange::CheckProgram { .. }
            | AccountChange::RemoveKey { .. }
            | AccountChange::Resume { .. }
            | AccountChange::MoveAccount { .. }
            | AccountChange::MoveSource { .. }
            | AccountChange::SetUses { .. } => Outcome::AccountChanged,
        };
        let call = world.request(
            SECOND_SIGN_IN,
            [u8::try_from(at).expect("eight account changes"); 16],
            change(AccountHolder::Own, value),
        );
        world.commit_all();
        assert_eq!(world.reply(call), Some(Reply::Outcome(expected)));
    }
    world.assert_settled();
}
#[test]
fn a_deployment_account_change_is_for_its_owners_only() {
    let mut world = owners();
    for (sign_in, expected) in
        [(SIGN_IN, Outcome::AccountChanged), (SECOND_SIGN_IN, Outcome::Refused(Refusal::Role { held: None }))]
    {
        let call =
            world.request(sign_in, [8; 16], change(AccountHolder::Deployment, AccountChange::Resume { source: 1 }));
        world.commit_all();
        assert_eq!(world.reply(call), Some(Reply::Outcome(expected)));
    }
    world.assert_settled();
}
#[test]
fn a_service_may_not_change_an_account() {
    let mut world = owners();
    let service = world.request(
        SIGN_IN,
        [9; 16],
        Ask::MakeService { project: PROJECT, name: b"build".to_vec().into(), role: Role::Member },
    );
    world.commit_all();
    let Some(Reply::Outcome(Outcome::ServiceMade { person })) = world.reply(service) else { panic!("service made") };
    world.signin_service(SERVICE_SIGN_IN, person);
    world.commit_all();
    for (key, holder) in [(0, AccountHolder::Own), (1, AccountHolder::Deployment)] {
        let call = world.request(SERVICE_SIGN_IN, [key; 16], change(holder, AccountChange::Remove { account: 1 }));
        world.commit_all();
        assert_eq!(world.reply(call), Some(Reply::Outcome(Outcome::Refused(Refusal::Role { held: None }))));
    }
    world.assert_settled();
}
#[test]
fn a_key_ask_replays_its_answer_for_the_same_digest() {
    let mut world = owners();
    let first = world.request(SECOND_SIGN_IN, [10; 16], llm_key(7));
    world.commit_all();
    let routes = world.stats().routes;
    world.restart();
    let replay = world.request(SECOND_SIGN_IN, [10; 16], llm_key(7));
    world.commit_all();
    assert_eq!(world.reply(first), world.reply(replay));
    assert_eq!(world.reply(replay), Some(Reply::Outcome(Outcome::AccountKeyKept { source: 1 })));
    assert_eq!(world.stats().routes, routes);
    world.assert_settled();
}
#[test]
fn a_key_ask_with_another_digest_is_a_key_conflict() {
    let mut world = owners();
    world.request(SECOND_SIGN_IN, [10; 16], llm_key(7));
    world.commit_all();
    let routes = world.stats().routes;
    let conflict = world.request(SECOND_SIGN_IN, [10; 16], llm_key(8));
    world.commit_all();
    assert_eq!(world.reply(conflict), Some(Reply::Refused(Refusal::KeyConflict)));
    assert_eq!(world.stats().routes, routes);
    world.assert_settled();
}

#[test]
fn a_move_out_replays_its_answer_without_routing_again() {
    let mut world = owners();
    let ask = change(AccountHolder::Own, AccountChange::MoveOut { account: 1 });
    let first = world.request(SECOND_SIGN_IN, [20; 16], ask.clone());
    world.commit_all();
    let routes = world.stats().routes;
    world.restart();
    let replay = world.request(SECOND_SIGN_IN, [20; 16], ask);
    world.commit_all();
    assert_eq!(world.reply(first), world.reply(replay));
    assert_eq!(
        world.reply(replay),
        Some(Reply::Outcome(Outcome::AccountMovingOut { account: 1, number: 2, source: 1 }))
    );
    assert_eq!(world.stats().routes, routes);
    world.assert_settled();
}

#[test]
fn a_move_in_with_another_digest_conflicts_under_the_same_key() {
    let mut world = owners();
    let first =
        world.request(SECOND_SIGN_IN, [21; 16], change(AccountHolder::Own, AccountChange::MoveIn { digest: [7; 32] }));
    world.commit_all();
    assert_eq!(world.reply(first), Some(Reply::Outcome(Outcome::AccountMovedIn { account: 1, renewed: false })));
    let routes = world.stats().routes;
    world.restart();
    let conflict =
        world.request(SECOND_SIGN_IN, [21; 16], change(AccountHolder::Own, AccountChange::MoveIn { digest: [8; 32] }));
    world.commit_all();
    assert_eq!(world.reply(conflict), Some(Reply::Refused(Refusal::KeyConflict)));
    assert_eq!(world.stats().routes, routes);
    world.assert_settled();
}

#[test]
fn a_non_owner_cannot_request_the_deployments_account_moves() {
    let mut world = owners();
    for (key, value) in [
        (22, AccountChange::MoveOut { account: 1 }),
        (23, AccountChange::Written { number: 2 }),
        (24, AccountChange::MoveIn { digest: [7; 32] }),
    ] {
        let call = world.request(SECOND_SIGN_IN, [key; 16], change(AccountHolder::Deployment, value));
        world.commit_all();
        assert_eq!(world.reply(call), Some(Reply::Outcome(Outcome::Refused(Refusal::Role { held: None }))));
    }
    world.assert_settled();
}
#[test]
fn account_uses_are_bounded_before_admission() {
    let mut world = owners();
    let call = world.request(
        SECOND_SIGN_IN,
        [11; 16],
        change(
            AccountHolder::Own,
            AccountChange::SetUses {
                account: 1,
                uses: Some(vec![Use::Party(1); skein_lib::length::usize_of(USES) + 1].into()),
            },
        ),
    );
    world.commit_all();
    assert_eq!(world.reply(call), Some(Reply::Refused(Refusal::Limit { which: jig_domain_people::LimitKind::Words })));
    world.assert_settled();
}

#[test]
fn another_program_name_under_a_retained_key_conflicts() {
    let mut world = owners();
    let declare = |name: &[u8]| {
        change(AccountHolder::Own, AccountChange::DeclareProgram { provider: 3, join: None, name: name.into() })
    };
    world.request(SECOND_SIGN_IN, [31; 16], declare(b"first"));
    world.commit_all();
    let routes = world.stats().routes;
    let conflict = world.request(SECOND_SIGN_IN, [31; 16], declare(b"second"));
    world.commit_all();
    assert_eq!(world.reply(conflict), Some(Reply::Refused(Refusal::KeyConflict)));
    assert_eq!(world.stats().routes, routes);
    world.assert_settled();
}
#[test]
fn a_program_check_is_admitted_as_an_account_change() {
    let mut world = owners();
    let call =
        world.request(SECOND_SIGN_IN, [32; 16], change(AccountHolder::Own, AccountChange::CheckProgram { source: 1 }));
    world.commit_all();
    assert_eq!(world.reply(call), Some(Reply::Outcome(Outcome::AccountChanged)));
    world.assert_settled();
}
