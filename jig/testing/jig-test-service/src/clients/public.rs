//! The public key and root handoffs (jig/docs/design/protocol/client.md, 19).
use super::*;
#[test]
fn public_names_are_kept_as_a_key_without_an_expiry() {
    let value = kit::Kept { digest: skein_lib::Secret::new(Box::new([17; 32])), expires: Wall::from_nanos(u64::MAX) };
    let (_, request) =
        secret_request(kit::SecretsRequest::Keep { owner: kit::SecretOwner::PublicNames, value: value.clone() });
    let jig_protocol_store::Secrets::Keep { owner, number, value: stored } = request else {
        panic!("keep public key");
    };
    assert_eq!(owner, jig_protocol_store::SecretOwner::PublicNames);
    assert_eq!(number, 0);
    let jig_protocol_store::SecretValue::Key(_) = &stored else {
        panic!("a key without expiry");
    };
    let restored = kept(stored).expect("key record");
    assert_eq!(restored.digest, value.digest);
    let (_, request) = secret_request(kit::SecretsRequest::Get { owner: kit::SecretOwner::PublicNames });
    let jig_protocol_store::Secrets::Get { owner: jig_protocol_store::SecretOwner::PublicNames, number: 0 } = request
    else {
        panic!("get the public names row");
    };
}
#[test]
fn public_reads_go_to_the_root_and_both_terminals_come_back_as_public() {
    let read = core::PublicRead { owner: 0, kind: 2, version: 1, project: Some(7), task: None };
    let request =
        kit::Request::Core(Box::new(core::Event::PublicRead { reply: skein_lib::ReplyTo::new(Token::new(1)), read }));
    let Routed::Root(event) = to_root(request) else {
        panic!("root public read");
    };
    let root::Event::Core(core::Event::PublicRead { read: mapped, .. }) = *event else {
        panic!("public read at root");
    };
    assert_eq!(mapped, read);
    let held =
        core::Held::PublicRead { reply: skein_lib::ReplyTo::new(Token::new(1)), answer: core::PublicAnswer::Unknown };
    let Some(kit::Event::Public { answer: core::PublicAnswer::Unknown, .. }) = to_component(held) else {
        panic!("held public terminal");
    };
    let busy = translation::starting(core::Event::PublicRead { reply: skein_lib::ReplyTo::new(Token::new(2)), read });
    let Some(kit::Event::Public { answer: core::PublicAnswer::Busy, .. }) = busy else {
        panic!("busy public terminal");
    };
}

#[test]
fn the_public_key_lookup_waits_for_the_stores_durable_startup_mark() {
    use crate::tests::fixtures as f;
    let mut service = f::started(&f::engine_document());
    let engine = f::engine(&mut service);
    let (config, _) = crate::startup::fixture_connectors(&f::engine_document());
    let limits = crate::limits::profile_limits(config::Profile::Small);
    let mut clients = Clients::new(
        &config,
        io::Addr::Ip((::core::net::Ipv4Addr::LOCALHOST, 7100).into()),
        &f::seeds(),
        &limits.server,
    )
    .expect("clients fixture");
    clients.start(Time::ZERO, Wall::EPOCH).expect("clients start");
    clients.advance(&mut engine.events, &engine.env.limits.connector, engine.env.limits.core.documents.rows, &mut None);
    clients.secrets_step(&mut engine.store);
    assert!(clients.public_names.is_some(), "the public key lookup remains held at the startup mark");
    assert!(clients.secret_owners.is_empty(), "nothing was sent to the unopened secrets store");
    assert!(clients.secrets.is_empty(), "ordinary exchanges keep their secret request queue");
}
