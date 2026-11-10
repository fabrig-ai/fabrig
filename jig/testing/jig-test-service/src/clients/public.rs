//! The public key and root handoffs (jig/docs/design/protocol/client/public-reads.md).
use super::*;
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
    assert!(clients.bridge.idle(), "nothing was sent to the unopened secrets store");
    assert!(clients.secrets.is_empty(), "ordinary exchanges keep their secret request queue");
}
