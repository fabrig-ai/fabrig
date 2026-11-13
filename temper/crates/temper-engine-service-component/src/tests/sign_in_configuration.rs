#![expect(clippy::disallowed_types, reason = "ordinary tests build configuration documents")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary tests construct and inspect protocol fixtures"
)]
use super::fixtures;
use crate::startup::{Problems, application};
use jig_protocol_config as config;
const SIGN_IN: &str = r#"{"google":{"provider":3,"client":"temper-test","secret":"google.secret","endpoints":{"issuer":"http://127.0.0.1:31999","authorization":"http://127.0.0.1:31999/auth","token":"http://127.0.0.1:31999/token","keys":"http://127.0.0.1:31999/keys","transport":"plaintext"}},"members":[{"address":"Sam@example.com","login":"sam"}]}"#;
fn jig() -> config::Engine {
    let mut jig = fixtures::engine().jig;
    let mut providers = skein_lib::List::with_capacity(3);
    for provider in jig.providers {
        providers.push(provider).unwrap();
    }
    providers.push(config::IdentityProvider { number: 3, kind: config::ProviderKind::Application }).unwrap();
    jig.providers = providers.into_boxed();
    jig.owners[0].identity =
        jig_domain_people::IdentityKey { provider: 3, subject: b"Sam@example.com".as_slice().into() };
    jig
}

fn parse(input: &str, jig: &config::Engine) -> (Option<application::Members>, Problems) {
    let document =
        config::document(input.as_bytes(), &config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 32 }).unwrap();
    let mut problems = Problems::empty();
    let members = application::parse(&document, 0, jig, &config::Directories::working(), &mut problems);
    (members, problems)
}

fn input(sign_in: &str) -> String {
    core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("\"sign_in\":null", &format!("\"sign_in\":{sign_in}"))
}

fn provider(jig: &mut config::Engine) {
    jig.clients.provider = Some(config::clients::Provider {
        key: b"provider.key".as_slice().into(),
        addresses: Box::new([3]),
        clients: Box::new([
            config::clients::ProviderClient {
                id: b"forgejo".as_slice().into(),
                secret: Some(b"forgejo.secret".as_slice().into()),
                redirect_uri: b"http://127.0.0.1:9/user/oauth2/temper/callback".as_slice().into(),
            },
            config::clients::ProviderClient {
                id: b"second".as_slice().into(),
                secret: Some(b"second.secret".as_slice().into()),
                redirect_uri: b"http://127.0.0.1:9/user/oauth2/temper/callback".as_slice().into(),
            },
        ]),
    });
}

fn with_forges(rows: &str) -> String {
    input(&SIGN_IN.replacen('{', &format!("{{\"forges\":{rows},"), 1))
}

#[test]
fn filling_the_bundle_sets_the_ways_and_changes_its_etag() {
    for google in [false, true] {
        let mut bundle = fixtures::bundle::bundle();
        assert!(bundle.fill_sign_in(google).is_none(), "missing head refused");
        bundle.page = b"<!doctype html><html><head><title>t</title></head><body></body></html>".as_slice().into();
        let before = bundle.etag.clone();
        bundle.fill_sign_in(google).unwrap();
        let meta = if google {
            b"<meta name=\"temper-sign-in\" content=\"google\"></head>".as_slice()
        } else {
            b"<meta name=\"temper-sign-in\" content=\"\"></head>".as_slice()
        };
        assert!(bundle.page.windows(meta.len()).any(|part| part == meta));
        assert_ne!(bundle.etag, before);
        let hash = bundle
            .page
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3));
        assert_eq!(bundle.etag.as_ref(), format!("\"{hash:016x}\"").as_bytes());
    }
}

#[test]
fn forge_metas_escape_attributes_publish_modes_and_change_the_etag() {
    use crate::clients::forge_sign_in::{Entries, refresh};
    use temper_engine_domain_forge::Mode;
    let mut jig = jig();
    provider(&mut jig);
    let (members, problems) = parse(&with_forges(r#"[{"forge":1,"client":"forgejo"}]"#), &jig);
    assert!(problems.is_empty());
    let mut members = members.unwrap();
    let sign_in = members.sign_in.as_mut().unwrap();
    sign_in.forges[0].client = b"forge&\"'< >".as_slice().into();
    let mut forges = fixtures::forges();
    forges[0].git = b"https://forge.example/?x=\"&y='<>".as_slice().into();
    let mut clients = jig.clients.provider.take().unwrap().clients;
    clients[0].id = sign_in.forges[0].client.clone();
    clients[0].redirect_uri = b"https://forge.example/?x=\"&y='<>/user/oauth2/temper/callback".as_slice().into();
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    engine.workers.forges = fixtures::forge_table(&forges);
    let mut bundle = fixtures::bundle::bundle();
    bundle.page = b"<head></head>".as_slice().into();
    bundle.fill_sign_in(true).unwrap();
    let before = bundle.etag.clone();
    engine.clients.adopt_bundle(bundle);
    engine.clients.forge_sign_in = Entries::new(&sign_in.forges, &clients, None, None, 65_536);
    refresh(engine);
    let bundle = engine.clients.bundle.as_ref().unwrap();
    assert_eq!(bundle.page.as_ref(), b"<head><meta name=\"temper-sign-in\" content=\"google\"><meta name=\"temper-forge-sign-in\" content=\"1 forge&amp;&quot;&#39;&lt; &gt; https://forge.example/?x=&quot;&amp;y=&#39;&lt;&gt; other\"></head>");
    assert_ne!(bundle.etag, before);
    engine.workers.forges.get_mut(1).unwrap().mode = Mode::Owned;
    let mut bundle = fixtures::bundle::bundle();
    bundle.page = b"<head></head>".as_slice().into();
    engine.clients.adopt_bundle(bundle);
    refresh(engine);
    assert!(engine.clients.bundle.as_ref().unwrap().page.ends_with(b" owned\"></head>"));
    for (port, origin) in [(443, "https://git.example.com"), (8443, "https://git.example.com:8443")] {
        let public = temper_forge_setup::setup::Origin { host: b"git.example.com".as_slice().into(), port };
        clients[0].redirect_uri = format!("{origin}/user/oauth2/temper/callback").into_bytes().into();
        engine.clients.forge_sign_in = Entries::new(&sign_in.forges, &clients, Some(&public), None, 65_536);
        refresh(engine);
        assert_eq!(engine.clients.bundle.as_ref().unwrap().page.as_ref(), format!("<head><meta name=\"temper-forge-sign-in\" content=\"1 forge&amp;&quot;&#39;&lt; &gt; {origin} owned\"></head>").as_bytes());
    }
    engine.clients.forge_sign_in = Entries::new(&sign_in.forges, &clients, None, None, 1);
    refresh(engine);
    assert_eq!(engine.clients.bundle.as_ref().unwrap().page.as_ref(), b"<head></head>");
}

#[test]
fn recording_meta_follows_sign_in_and_survives_forge_metas_with_its_own_etag() {
    let mut bundle = fixtures::bundle::bundle();
    assert!(bundle.fill_record().is_none(), "missing head refused");
    bundle.page = b"<html><head></head><body></body></html>".as_slice().into();
    bundle.fill_sign_in(false).unwrap();
    let without = bundle.etag.clone();
    assert_eq!(
        bundle.page.as_ref(),
        b"<html><head><meta name=\"temper-sign-in\" content=\"\"></head><body></body></html>"
    );
    bundle.fill_record().unwrap();
    assert_eq!(bundle.page.as_ref(), b"<html><head><meta name=\"temper-sign-in\" content=\"\"><meta name=\"temper-record\" content=\"on\"></head><body></body></html>");
    assert_ne!(bundle.etag, without);
    let hash = bundle
        .page
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3));
    assert_eq!(bundle.etag.as_ref(), format!("\"{hash:016x}\"").as_bytes());
    for metas in [
        b"<meta name=\"temper-forge-sign-in\" content=\"one\">".as_slice(),
        b"<meta name=\"temper-forge-sign-in\" content=\"two\">",
        b"",
    ] {
        bundle.set_forge_metas(metas).unwrap();
        let expected = format!(
            "<html><head><meta name=\"temper-sign-in\" content=\"\"><meta name=\"temper-record\" content=\"on\">{}</head><body></body></html>",
            core::str::from_utf8(metas).unwrap()
        );
        assert_eq!(bundle.page.as_ref(), expected.as_bytes());
    }
}
