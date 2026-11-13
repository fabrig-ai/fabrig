//! Public kinds, key ownership and glue (temper/docs/design/client/forge/README.md, 9).
use super::*;
use jig_domain_core as core;
use skein_lib::{ReplyTo, Token};
#[test]
fn startup_names_the_public_kinds_and_private_parts() {
    let config = crate::tests::fixtures::engine();
    let seeds = crate::tests::fixtures::seeds();
    let core = crate::system::core_config(&config.jig, &seeds);
    assert_eq!(core.settings.private_tools.as_ref(), [Box::from(b"remember".as_slice())]);
    let clients = crate::tests::fixtures::clients();
    let private = &clients.initial.as_ref().expect("startup settings").private;
    assert_eq!(private, &temper_engine_smith::private());
    assert_eq!(private.title.as_ref(), b"Kept about this person");
    assert_eq!(private.tools.as_slice(), [Box::from(b"remember".as_slice())]);
    let settings = config::clients::settings(
        &config.jig,
        &application(&config.jig, None, List::uncapped()),
        seeds.crypto.clone(),
        seeds.seed,
    );
    let kinds = settings.public.as_slice();
    assert_eq!(kinds.len(), 6);
    let forge = kinds.get(3..).expect("forge kinds after jig");
    let expected = forge_public_kinds();
    assert_eq!(forge, &expected);
    assert_eq!(
        expected.map(|kind| (kind.owner, kind.kind, kind.first, kind.last, kind.task)),
        [(2, 1, 2, 2, true), (2, 2, 2, 2, false), (2, 3, 2, 2, true)]
    );
    assert_eq!(expected.map(|kind| kind.bound), [822, 67_405, 8716]);
}
#[test]
fn a_public_read_and_its_answer_cross_the_root_boundary() {
    let reply = ReplyTo::new(Token::new(9));
    let read = core::PublicRead { owner: 2, kind: 1, version: 2, project: Some(7), task: Some(8) };
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

/// A declared kind must fit the answer ceiling (jig/docs/design/protocol/limits.md).
#[test]
fn a_public_kind_above_the_ceiling_names_answer_bytes() {
    let mut config = crate::tests::fixtures::engine();
    config.jig.limits.clients.public = kit::PublicLimits {
        reads: 4,
        rate: 8,
        rate_burst: 8,
        bytes: 524_288,
        bytes_burst: 524_288,
        answer_bytes: 67_404,
        max_age: 2,
    };
    let Err(problems) = Clients::new(
        &config::Directories::working(),
        &config.jig,
        &config.clients,
        &crate::tests::fixtures::seeds(),
        &config.limits.root.journal,
        None,
        None,
        &[],
    ) else {
        panic!("landings exceeds the ceiling by one byte")
    };
    assert_eq!(problems.problems.as_slice().len(), 1);
    assert_eq!(problems.problems.as_slice().first().expect("one refusal").what, What::Bound);
    assert_eq!(
        problems.problems.as_slice().first().expect("one refusal").path,
        Path::root().member(b"clients").member(b"public").member(b"answer_bytes")
    );
}
