//! Push tools through both roots (client/ctl/tools.md, 2, 3; attention.md, 7.1).
use jig_client_kit::harness::{ClientId, Settings, World};
use temper_client_world::{
    app::{Event, Temper},
    fixtures,
    front::Joint,
};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

// RFC 8291 Appendix A's user-agent public key and authentication secret.
const FILE: &[u8] = br#"{"endpoint":"http://push.test:4000/push/alice-phone","keys":{"p256dh":"BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4","auth":"BTBZMqHH6r4Tts7J_aSIgg"},"expirationTime":null,"ignored":1234567890123456789012345678901234567890}"#;
type JointWorld = World<Temper, Joint>;
fn world(push: bool) -> JointWorld {
    let mut front = Joint::new(fixtures::SEED, fixtures::domain_limits());
    let limits = front.engine.env.limits;
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(fixtures::SEED);
        config.core.connectors = Box::new([fixtures::FORGE]);
        if push {
            config.forge.push = Some(Box::new([b"http://push.test:4000".as_slice().into()]));
        }
        config
    });
    let forge = front.engine.forge.take();
    front.engine = Driver::configured(Store::with_limits(&limits), configs, &limits);
    front.engine.forge = forge;
    front.engine.settle();
    let mut world = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), iterations: 4096, ..Settings::default() },
        front,
    );
    world.randomness(2, 0, 0);
    world
}
fn add(world: &mut JointWorld, client: ClientId, file: &[u8]) -> temper_client_tools::jig::Printed {
    // Supply precisely the host's loaded FILE, not an external path or an endpoint argument.
    let Event::Call { call, json, .. } = fixtures::call(&[b"push", b"add", b"subscription.json"]) else {
        panic!("ctl call")
    };
    let number = world.root(client).agent().calls() + 1;
    world.event(client, Event::Call { number, call, json, value: Some(skein_lib::Secret::new(file.into())) });
    fixtures::answered(world, client, number)
}
fn call(world: &mut JointWorld, client: ClientId, words: &[&[u8]]) -> temper_client_tools::jig::Printed {
    let number = fixtures::give(world, client, words);
    fixtures::answered(world, client, number)
}
#[test]
fn ctl_registers_lists_only_the_origin_and_removes_a_device() {
    let mut world = world(true);
    let client = fixtures::agent(&mut world);
    // Wait for the party watch as an ordinary ctl call does.
    assert_eq!(call(&mut world, client, &[b"push", b"list"]).status, 0);
    let added = add(&mut world, client, FILE);
    assert_eq!(added.status, 0, "{added:?}");
    assert_eq!(added.stdout.as_ref(), b"device 1: added\n");
    let listing = call(&mut world, client, &[b"push", b"list"]);
    assert_eq!(listing.status, 0, "{listing:?}");
    let listing = String::from_utf8(listing.stdout.into_vec()).unwrap();
    assert!(listing.contains("device 1: temper ctl"), "{listing}");
    assert!(listing.contains("http://push.test:4000"), "{listing}");
    assert!(!listing.contains("alice-phone"), "{listing}");
    let json = call(&mut world, client, &[b"push", b"list", b"--json"]);
    assert!(!String::from_utf8(json.stdout.into_vec()).unwrap().contains("alice-phone"));
    // Device rows are durable in the joint store, not a client-side list.
    assert!(world.front().engine.store.rows.values().any(|row| matches!(row, engine::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Device(device) if device.number == 1 && device.label.as_ref() == b"temper ctl"))));
    let removed = call(&mut world, client, &[b"push", b"remove", b"1"]);
    assert_eq!(removed.status, 0, "{removed:?}");
    assert_eq!(removed.stdout.as_ref(), b"device 1: removed\n");
    assert!(call(&mut world, client, &[b"push", b"list"]).stdout.is_empty());
    let unknown = call(&mut world, client, &[b"push", b"remove", b"1"]);
    assert_eq!(unknown.status, 2);
    assert_eq!(unknown.stderr.as_ref(), b"No device 1.\n");
}
#[test]
fn ctl_reports_push_off_and_rejects_a_non_subscription_without_an_act() {
    let mut world = world(false);
    let client = fixtures::agent(&mut world);
    assert_eq!(call(&mut world, client, &[b"push", b"list"]).status, 0);
    let refused = add(&mut world, client, FILE);
    assert_eq!(refused.status, 2, "{refused:?}");
    assert_eq!(refused.stderr.as_ref(), b"This deployment doesn't push.\n");
    let before = world.front().engine.store.rows.len();
    let invalid = add(&mut world, client, br#"{"not":"a subscription"}"#);
    assert_eq!(invalid.status, 2);
    assert_eq!(invalid.stderr.as_ref(), b"Not a push subscription's JSON.\n");
    assert_eq!(world.front().engine.store.rows.len(), before, "no application sent for invalid JSON");
}
