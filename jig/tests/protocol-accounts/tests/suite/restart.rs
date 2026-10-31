use jig_domain_accounts::{Failure, FromComponent, Kind, Place, Unread};
use jig_protocol_accounts::{party, table::Scheme};
use jig_protocol_accounts_world::{fixtures, key_world::World, world::Story};
use jig_protocol_store::{SecretOwner, SecretValue, Secrets};
use skein_lib::{List, Map, Secret, Time};
use skein_oauth_accounts as driver;

fn signed_in_value() -> Secret {
    let mut saved = fixtures::kept();
    saved.key = 7;
    saved.issued_client_id = Some(b"issued-client".as_slice().into());
    saved.granted_scopes = Some(b"openid read".as_slice().into());
    saved.subject = Some(b"subject".as_slice().into());
    Secret::new(skein_oauth::encode_record(&saved, &fixtures::limits().client.document).expect("saved source"))
}
fn signed_in_world() -> World {
    let registration = fixtures::party_registration(Story::SignIn);
    let template = party::party_account(
        &registration,
        driver::Endpoint { address: ([127, 0, 0, 1], 31000).into(), transport: driver::Transport::Plaintext },
        List::with_capacity(0),
        &[5; 16],
    );
    let mut providers = Map::with_capacity(1);
    providers.insert(1, template).ok().expect("provider");
    World::configured(fixtures::limits(), providers)
}
#[test]
fn a_restart_loads_a_kept_key() {
    let mut world = World::new();
    let carried = world.carry(&[7; 32]);
    world.keep(7, carried, false);
    world.restart();
    world.restore(7, 1, Kind::Key);
    world.want(7);
    assert!(matches!(world.heard.as_slice(), [FromComponent::Granted { account: 7, generation: 1, .. }]));
    assert_eq!(world.table.value(7, 1, Time::ZERO).expect("restored key").token.expose(), &[7; 32]);
    assert_eq!(world.get(SecretOwner::Source, 7), Some(Secret::new(Box::new([7; 32]))));
}
#[test]
fn a_restart_re_adds_a_party_signed_in_source() {
    let mut world = signed_in_world();
    world.secret(Secrets::Keep {
        owner: SecretOwner::Source,
        number: 7,
        value: SecretValue::Record(signed_in_value()),
    });
    world.restore(7, 1, Kind::SignIn);
    world.want(7);
    assert!(matches!(world.heard.as_slice(), [FromComponent::Granted { account: 7, generation: 1, .. }]));
    let value = world.table.value(7, 1, Time::ZERO).expect("restored source grant");
    assert_eq!(value.scheme, Scheme::Bearer);
    assert_eq!(value.token.expose(), fixtures::TOKEN_FIRST);
}
#[test]
fn the_restart_erases_only_unnamed_source_rows() {
    let mut world = signed_in_world();
    for number in [1, 8, 9, 10] {
        world.secret(Secrets::Keep {
            owner: SecretOwner::Source,
            number,
            value: SecretValue::Key(Secret::new(Box::new([7; 32]))),
        });
    }
    world.secret(Secrets::Keep {
        owner: SecretOwner::Source,
        number: 7,
        value: SecretValue::Record(signed_in_value()),
    });
    let carried = world.carry(&[9; 32]);
    world.restore(1, 1, Kind::Key);
    world.restore(7, 1, Kind::SignIn);
    assert!(world.swept.is_empty() && world.loads == 0, "nothing swept before Restored");
    world.restored();
    world.restored();
    assert_eq!(world.swept, [8, 9, 10]);
    assert_eq!(world.loads, 3, "one bounded page at a time");
    for number in [1, 7] {
        assert!(world.get(SecretOwner::Source, number).is_some());
    }
    for number in [8, 9, 10] {
        assert!(world.get(SecretOwner::Source, number).is_none());
    }
    assert!(world.get(SecretOwner::Application, carried).is_some(), "carried application rows survive");
    world.secret(Secrets::Keep {
        owner: SecretOwner::Source,
        number: 11,
        value: SecretValue::Key(Secret::new(Box::new([7; 32]))),
    });
    world.restored();
    assert_eq!(world.swept, [8, 9, 10], "the restart sweeps only once");
    assert!(world.get(SecretOwner::Source, 11).is_some());
}
#[test]
fn a_restart_reports_missing_and_oversized_llm_keys_as_unloaded() {
    for value in [None, Some(Secret::new(Box::new([7; 33])))] {
        let mut world = World::new();
        if let Some(value) = value {
            world.secret(Secrets::Keep { owner: SecretOwner::Source, number: 7, value: SecretValue::Key(value) });
        }
        world.restore(7, 1, Kind::Key);
        world.want(7);
        assert_eq!(
            world.heard,
            [FromComponent::Failed {
                account: 7,
                failure: Failure::Unloaded { at: Place::File, why: Unread::Unreadable }
            }]
        );
        assert!(world.table.value(7, 1, Time::ZERO).is_none());
    }
}
