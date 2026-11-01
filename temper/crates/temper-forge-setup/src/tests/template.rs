//! Machine templates share setup's entry reader and directory resolution.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary Rust fixture loading and JSON editing"
)]
use super::fixtures;
use crate::{
    limits, problem,
    setup::{Mode, Setup, Template, Want},
};
use jig_protocol_config::{self as config, Path};

fn fixture(forge: u16) -> String {
    let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let source = std::fs::read_to_string(root.join("src/tests/seeded-forges.json")).unwrap();
    let source = source.split_once("\"setup\": [").unwrap().1;
    let marker = format!("\"forge\": {forge},");
    let member = source.find(&marker).unwrap();
    let start = source[..member].rfind('{').unwrap();
    let mut depth = 0_u32;
    let mut end = start;
    for (offset, byte) in source.as_bytes()[start..].iter().enumerate() {
        match byte {
            b'{' => depth = depth.checked_add(1).unwrap(),
            b'}' => {
                depth = depth.checked_sub(1).unwrap();
                if depth == 0 {
                    end = start.checked_add(offset).unwrap().checked_add(1).unwrap();
                    break;
                }
            }
            _ => {}
        }
    }
    source[start..end]
        .replace(&marker, "")
        .replace("\"start\": \"asked\",", "")
        .replace("$forge", "127.0.0.1:8369")
        .replace("$owned", "127.0.0.1:3000")
        .replace("$login", "person")
        .replace("$email", "person@localhost")
}
fn read(bytes: &[u8], want: Want, ps: &mut config::Problems) -> Option<Template> {
    Template::read(
        &fixtures::document(bytes),
        0,
        want,
        &limits::small(),
        &config::Directories::given(b"/config", b"/data", b"/cache").unwrap(),
        &Path::root().member(b"application").member(b"machine"),
        ps,
    )
}
fn equivalent(want: Want, seed: u16, forge: u16) -> Template {
    let source = fixture(seed);
    let template = read(source.as_bytes(), want, &mut config::Problems::empty()).unwrap();
    let start = if want == Want::Owned { ",\"start\":\"asked\"" } else { "" };
    let entry = format!("[{{\"forge\":{forge}{start},{}]", &source[1..]);
    let setup = Setup::read(
        &fixtures::document(entry.as_bytes()),
        0,
        &limits::small(),
        &config::Directories::given(b"/config", b"/data", b"/cache").unwrap(),
        &mut problem::Problems::empty(),
    )
    .unwrap();
    assert_eq!(template.entry(forge), *setup.entry(forge).unwrap());
    template
}
#[test]
fn an_owned_template_is_its_entry_less_forge_and_start() {
    let template = equivalent(Want::Owned, 3, 7);
    assert_eq!(template.directory.as_ref(), b"/data/forge-3");
    let Mode::Owned(owned) = template.mode else { panic!("owned template") };
    assert_eq!(
        owned.forgejo.as_ref(),
        format!("/cache/forgejo/{}/forgejo", skein_forge::forgejo::pin::VERSION).as_bytes()
    );
    assert_eq!(
        owned.runner.unwrap().binary.as_ref(),
        format!("/cache/forgejo-runner/{}/forgejo-runner", skein_forge::forgejo::pin::RUNNER_VERSION).as_bytes()
    );
}
#[test]
fn a_fake_template_is_its_entry_less_forge() {
    let template = equivalent(Want::Fake, 1, 9);
    assert!(!template.entry(9).asked);
    assert_eq!(template.directory.as_ref(), b"/data/demo/forge");
    let Mode::Fake(fake) = template.mode else { panic!("fake template") };
    assert_eq!(fake.git.as_ref(), b"/data/demo/bin/git");
}
#[test]
fn a_template_refuses_forge_start_and_the_other_mode() {
    for (source, want, member) in [
        (fixture(3).replacen('{', "{\"forge\":7,", 1), Want::Owned, b"forge".as_slice()),
        (fixture(3).replacen('{', "{\"start\":\"asked\",", 1), Want::Owned, b"start".as_slice()),
        (fixture(1), Want::Owned, b"mode".as_slice()),
        (fixture(3), Want::Fake, b"mode".as_slice()),
        (
            "{\"bot\":{\"login\":\"temper\",\"email\":\"temper@localhost\"},\"mode\":\"managed\"}".into(),
            Want::Owned,
            b"mode".as_slice(),
        ),
    ] {
        let mut ps = config::Problems::empty();
        assert!(read(source.as_bytes(), want, &mut ps).is_none());
        assert_eq!(ps.problems.len(), 1);
        assert_eq!(
            ps.problems.get(0).unwrap().path,
            Path::root().member(b"application").member(b"machine").member(member)
        );
        assert_eq!(ps.problems.get(0).unwrap().what, config::What::Form);
    }
}
