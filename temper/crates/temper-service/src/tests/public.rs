//! Public kinds, key ownership and glue (temper/docs/design/client/forge.md, 11).
use super::*;
use jig_domain_core as core;
use skein_lib::{ReplyTo, Token};
#[test]
fn the_forge_declares_its_three_public_kinds() {
    let config = crate::tests::fixtures::engine();
    let settings =
        client_settings(&config.jig, &crate::tests::fixtures::seeds(), None, List::uncapped(), List::uncapped(), 0);
    let kinds = settings.public.as_slice();
    assert_eq!(kinds.len(), 6);
    let forge = kinds.get(3..).expect("forge kinds after jig");
    let expected = forge_public_kinds();
    assert_eq!(forge, &expected);
    assert_eq!(
        expected.map(|kind| (kind.owner, kind.kind, kind.first, kind.last, kind.task)),
        [(2, 1, 1, 1, true), (2, 2, 1, 1, false), (2, 3, 1, 1, true)]
    );
    assert_eq!(expected.map(|kind| kind.bound), [4927, 198_733, 165_794]);
}
#[test]
fn the_public_names_key_is_kept_under_its_owner() {
    assert_eq!(place(kit::SecretOwner::PublicNames), (jig_protocol_store::SecretOwner::PublicNames, 0));
}
#[test]
fn a_public_read_and_its_answer_cross_the_root_boundary() {
    let reply = ReplyTo::new(Token::new(9));
    let read = core::PublicRead { owner: 2, kind: 1, version: 1, project: Some(7), task: Some(8) };
    let input = translation::to_root(core::Event::PublicRead { reply, read }).expect("public input");
    let root::PartyInput::PublicRead { reply, read: mapped } = input else { panic!("public read") };
    assert_eq!(mapped, read);
    let output = translation::to_component(root::PartyRequest::Deliver(root::Delivery::Public {
        reply,
        answer: core::PublicAnswer::Unknown,
        admitted: true,
    }));
    let Some(kit::Event::Public { answer, .. }) = output else { panic!("public answer") };
    assert_eq!(answer, core::PublicAnswer::Unknown);
    let output = translation::not_ready(root::PartyInput::PublicRead { reply: ReplyTo::new(Token::new(10)), read });
    let Some(kit::Event::Public { answer, .. }) = output else { panic!("public refusal") };
    assert_eq!(answer, core::PublicAnswer::Busy);
}
