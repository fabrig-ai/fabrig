//! The playground's fake forge before start assigns its number (draft playground-reset.md, 6.1).
#![expect(clippy::disallowed_methods, clippy::disallowed_macros, reason = "ordinary configuration fixture editing")]
use crate::{
    settings,
    startup::{self, application},
};
use jig_protocol_config as config;
use skein_lib::Duration;
use temper_engine_domain_forge::{Kind, record as r};
use temper_forge_setup::setup::Mode;

fn document(fake: bool, forge: bool) -> Box<[u8]> {
    let text = include_str!("../../../temper-init/template/temper.json");
    let mut text = text.to_owned();
    for member in ["forges", "identities", "setup"] {
        let seeded = super::fixtures::value(include_bytes!("seeded-forges.json"), member.as_bytes());
        text = text.replace(
            &format!("\"{member}\": []"),
            &format!("\"{member}\": {}", core::str::from_utf8(&seeded).unwrap()),
        );
    }
    text = text.replacen("\"demo\": {", "\"demo\": {\"forge\": 1,", 1);
    for (from, to) in [
        ("$git", "/usr/bin/git"),
        ("$forge_git", "http://127.0.0.1:8369"),
        ("$owned_git", "http://127.0.0.1:3000"),
        ("$forge", "127.0.0.1:8369"),
        ("$owned", "127.0.0.1:3000"),
        ("$llm", "127.0.0.1:8370"),
        ("$listener", "127.0.0.1:8367"),
        ("$webhooks", "127.0.0.1:8368"),
        ("$login", "person"),
        ("$email", "person@localhost"),
    ] {
        text = text.replace(from, to);
    }
    if !fake {
        let value = super::fixtures::value(text.as_bytes(), b"application.demo.fake");
        text = text.replace(core::str::from_utf8(&value).unwrap(), "null");
    }
    if !forge {
        // Remove only the demo's forge; the setup entry keeps its number.
        let at = text.find("\"demo\": {").unwrap();
        let demo = text[at..].replacen("\"forge\": 1,", "", 1);
        text.truncate(at);
        text.push_str(&demo);
    }
    text.into_bytes().into_boxed_slice()
}
fn parse(bytes: &[u8]) -> Result<settings::Demo, startup::Problems> {
    let d = config::document(bytes, &config::DocumentLimits { bytes: 1 << 20, tokens: 32768, depth: 64 }).unwrap();
    let directories = config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    let mut root = config::read::Object::new(&d, 0).unwrap();
    let engine = startup::engine(&d, root.member(&d, b"engine").unwrap(), &directories).unwrap();
    let mut ps = startup::Problems::empty();
    let app = application::parse(&d, root.member(&d, b"application").unwrap(), &engine, &directories, &mut ps);
    if !ps.is_empty() {
        return Err(ps);
    }
    Ok(app.unwrap().demo.unwrap())
}
#[test]
fn the_templates_fake_makes_the_playgrounds_record() {
    for forge in [true, false] {
        let demo = parse(&document(true, forge)).unwrap();
        let domain = demo.domain(&[], Duration::from_secs(10));
        assert!(domain.within());
        assert_eq!(domain.adoption.forge, u16::from(forge));
        assert_eq!(domain.adoption.project, demo.project);
        assert_eq!(domain.adoption.host.as_ref(), b"127.0.0.1:8369");
        let record = domain.forge.unwrap();
        assert!(record.fits());
        assert_eq!(record.number, 0);
        assert_eq!(record.kind, Kind::Forgejo);
        assert_eq!(record.api, r::Address::Socket("127.0.0.1:8369".parse().unwrap()));
        assert_eq!(record.transport, r::Transport::Plaintext);
        assert_eq!(record.git.as_ref(), b"http://127.0.0.1:8369");
        assert_eq!(record.git_program.as_deref(), Some(b"/data/demo/bin/git".as_slice()));
        assert!(record.web.is_none());
        assert!(!record.read_only);
        let entry = demo.fake.unwrap().entry(0);
        assert_eq!(
            record.identity,
            r::Identity {
                number: 0,
                login: entry.bot.login.clone(),
                name: entry.bot.login.clone(),
                email: entry.bot.email.clone(),
            }
        );
        assert!(record.files.is_none());
        let setup = record.setup.unwrap();
        assert_eq!(setup.directory, entry.directory);
        assert_eq!(setup.bot.login, entry.bot.login);
        assert_eq!(setup.bot.email, entry.bot.email);
        assert_eq!(setup.orgs, entry.orgs);
        assert_eq!(setup.asked, entry.asked);
        let (r::SetupMode::Fake(actual), Mode::Fake(wanted)) = (setup.mode, entry.mode) else { panic!("fake setup") };
        assert_eq!(actual.listen, wanted.listen);
        assert_eq!(actual.git, wanted.git);
        assert_eq!(actual.owner, wanted.owner);
        assert_eq!(actual.repositories.len(), wanted.repositories.len());
        for (actual, wanted) in actual.repositories.iter().zip(&wanted.repositories) {
            assert_eq!(actual.name, wanted.name);
            assert_eq!(actual.landing, wanted.landing);
        }
        assert!(record.first.is_none());
        assert!(record.playground);
        assert!(!record.removing);
    }
}
#[test]
fn a_demo_without_forge_needs_its_fake() {
    let demo = parse(&document(true, false)).unwrap();
    assert_eq!(demo.forge, None);
    assert!(demo.domain(&[], Duration::from_secs(10)).forge.is_some());
    let problems = parse(&document(false, false)).unwrap_err();
    assert_eq!(startup::lines(&problems).as_ref(), b"application.demo.fake: missing\n");
}
#[test]
fn a_demo_without_fake_has_no_record() {
    let demo = parse(&document(false, true)).unwrap();
    assert_eq!(demo.forge, Some(1));
    assert!(demo.domain(&[], Duration::from_secs(10)).forge.is_none());
}
