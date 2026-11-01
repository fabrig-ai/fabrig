//! Machine members beside the deployment's seeded forges (contracts, 6).
#![expect(clippy::disallowed_methods, clippy::disallowed_macros, reason = "ordinary configuration fixture editing")]
use super::fixtures as fixture;
use crate::{
    settings,
    startup::{self, application},
};
use jig_protocol_config as config;
use temper_forge_setup::setup::{Mode, Want};

const OWNED: &str = r#"{"directory":"forge-3","bot":{"login":"temper","email":"temper@localhost"},"mode":{"owned":{"forgejo":"fetched","runner":null,"listen":{"socket":"127.0.0.1:3000"},"person":{"login":"person","email":"person@localhost"}}}}"#;
fn document(owned: Option<&str>, fake: bool) -> Box<[u8]> {
    let application = fixture::demo_application();
    let bytes = fixture::with(&fixture::document(), b"application", &application);
    let mut text =
        core::str::from_utf8(&bytes).unwrap().replace("$forge", "127.0.0.1:9").replace("$llm", "127.0.0.1:8370");
    if let Some(owned) = owned {
        text = text.replacen("\"application\":{", &format!("\"application\":{{\"owned\":{owned},"), 1);
    }
    if fake {
        let setup = fixture::value(&application, b"setup[0]");
        let setup = core::str::from_utf8(&setup).unwrap();
        let fake = setup.replacen("\"forge\":1,", "", 1).replace("$forge", "127.0.0.1:9");
        text = text.replacen("\"demo\":{", &format!("\"demo\":{{\"fake\":{fake},"), 1);
    }
    text.into_bytes().into_boxed_slice()
}
fn parse(bytes: &[u8]) -> Result<settings::Engine, startup::Problems> {
    let d = config::document(bytes, &config::DocumentLimits { bytes: 1 << 20, tokens: 16384, depth: 32 }).unwrap();
    let directories = config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    let mut root = config::read::Object::new(&d, 0).unwrap();
    let engine = startup::engine(&d, root.member(&d, b"engine").unwrap(), &directories).unwrap();
    let mut ps = startup::Problems::empty();
    let members = application::parse(&d, root.member(&d, b"application").unwrap(), &engine, &directories, &mut ps);
    if !ps.is_empty() {
        return Err(ps);
    }
    let members = members.unwrap();
    Ok(settings::Engine { owned: members.owned, demo: members.demo, ..super::fixtures::engine() })
}
#[test]
fn the_machine_members_reach_the_engines_settings() {
    let engine = parse(&document(Some(OWNED), true)).unwrap();
    for (template, want, forge, directory) in [
        (engine.owned.as_ref().unwrap(), Want::Owned, 7, b"/data/forge-3".as_slice()),
        (engine.demo.as_ref().unwrap().fake.as_ref().unwrap(), Want::Fake, 9, b"/data/demo/forge".as_slice()),
    ] {
        let entry = template.entry(forge);
        assert_eq!(entry.forge, forge);
        assert_eq!(entry.directory.as_ref(), directory);
        assert_eq!(entry.asked, want == Want::Owned);
        let mode = match entry.mode {
            Mode::Owned(_) => Want::Owned,
            Mode::Fake(_) => Want::Fake,
            Mode::Managed { .. } => panic!("machine template"),
        };
        assert_eq!(mode, want);
    }
}
#[test]
fn a_document_without_them_reads_none() {
    for owned in [None, Some("null")] {
        let engine = parse(&document(owned, false)).unwrap();
        assert!(engine.owned.is_none());
        assert!(engine.demo.unwrap().fake.is_none());
    }
}
#[test]
fn a_wrong_mode_is_refused_at_its_path() {
    let bytes = document(None, false);
    let text =
        core::str::from_utf8(&bytes).unwrap().replacen("\"demo\":{", &format!("\"demo\":{{\"fake\":{OWNED},"), 1);
    for (bytes, path) in [
        (
            document(Some(r#"{"bot":{"login":"temper","email":"temper@localhost"},"mode":"managed"}"#), false),
            b"application.owned.mode: invalid form\n".as_slice(),
        ),
        (text.into_bytes().into_boxed_slice(), b"application.demo.fake.mode: invalid form\n".as_slice()),
    ] {
        assert_eq!(startup::lines(&parse(&bytes).unwrap_err()).as_ref(), path);
    }
}
