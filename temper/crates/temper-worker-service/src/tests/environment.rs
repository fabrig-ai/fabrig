//! The agents' environment at the document and figures boundary (configuration/figures.md, 6).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary tests construct JSON entries and inspect refusal lines"
)]
use super::{figures::stock_figures, fixtures};
use crate::startup::{
    self,
    worker::application::{self, EnvironmentEntry},
};
use jig_protocol_config as config;

fn entries(values: &[(&str, &str)]) -> Box<[EnvironmentEntry]> {
    values.iter().map(|(name, value)| (name.as_bytes().into(), value.as_bytes().into())).collect()
}
fn own() -> Box<[EnvironmentEntry]> {
    entries(&[("HOME", "/home/w"), ("OTHER", "x"), ("PATH", "/usr/bin:/bin")])
}
fn from_figures(given: Option<serde_json::Value>) -> startup::WorkerPart {
    let figures = stock_figures();
    let mut input: serde_json::Value = serde_json::from_slice(&fixtures::worker_alone_document()).unwrap();
    input["workers"][0]["slots"] = figures.places().into();
    if let Some(given) = given {
        input["application"]["environment"] = given;
    }
    let mut part =
        startup::worker_part(&serde_json::to_vec(&input).unwrap(), &config::Directories::working(), None, &own())
            .unwrap();
    crate::figures::apply(&mut part, figures).unwrap();
    part
}
#[test]
fn a_worker_from_figures_inherits_only_its_own_home_and_path() {
    let part = from_figures(None);
    assert_eq!(part.environment, entries(&[("HOME", "/home/w"), ("PATH", "/usr/bin:/bin")]));
}
#[test]
fn a_worker_from_figures_keeps_given_entries_then_missing_home() {
    let part = from_figures(Some(serde_json::json!([
        {"name":"CARGO_TARGET_DIR","value":"/srv/target"},
        {"name":"PATH","value":"/opt/cargo/bin:/usr/bin"}
    ])));
    assert_eq!(
        part.environment,
        entries(&[("CARGO_TARGET_DIR", "/srv/target"), ("PATH", "/opt/cargo/bin:/usr/bin"), ("HOME", "/home/w"),])
    );
}
#[test]
fn agents_environment_inherits_only_missing_home_and_path() {
    for (given, process, expected) in [
        (None, own(), entries(&[("HOME", "/home/w"), ("PATH", "/usr/bin:/bin")])),
        (Some(entries(&[])), own(), entries(&[("HOME", "/home/w"), ("PATH", "/usr/bin:/bin")])),
        (Some(entries(&[("HOME", "/given")])), own(), entries(&[("HOME", "/given"), ("PATH", "/usr/bin:/bin")])),
        (
            Some(entries(&[("PATH", "/given/bin"), ("HOME", "/given")])),
            own(),
            entries(&[("PATH", "/given/bin"), ("HOME", "/given")]),
        ),
        (None, entries(&[("OTHER", "x"), ("PATH", "/bin")]), entries(&[("PATH", "/bin")])),
    ] {
        assert_eq!(
            application::agents_environment(given.as_deref(), &process),
            expected,
            "given {given:?}, own {process:?}"
        );
    }
}
#[test]
fn environment_entries_are_refused_at_their_member() {
    for (entry, member, what, words) in [
        (serde_json::json!({"name":"A","value":"two"}), "name", config::What::Unique, "must be unique"),
        (serde_json::json!({"name":"B"}), "value", config::What::Missing, "missing"),
        (serde_json::json!({"name":"B","value":"two","extra":true}), "extra", config::What::Unknown, "unknown member"),
    ] {
        let mut input: serde_json::Value = serde_json::from_slice(&fixtures::worker_alone_document()).unwrap();
        input["application"]["environment"] = serde_json::json!([{"name":"A","value":"one"}, entry]);
        let found =
            startup::worker_part(&serde_json::to_vec(&input).unwrap(), &config::Directories::working(), None, &own())
                .expect_err("invalid entry refused");
        let path = config::Path::root()
            .member(b"application")
            .member(b"environment")
            .child(config::Segment::Index(1))
            .member(member.as_bytes());
        assert_eq!(found.problems.len(), 1);
        assert_eq!(found.problems.get(0).unwrap().path, path);
        assert_eq!(found.problems.get(0).unwrap().what, startup::What::Jig(what));
        let lines = startup::lines(&found);
        let line = core::str::from_utf8(&lines).unwrap();
        assert!(line.contains(&format!("application.environment[1].{member}: {words}")), "{line}");
    }
}
#[test]
fn environment_text_bound_names_the_entry() {
    // Exercise the entry reader's bound independently of the whole file's byte bound.
    for member in ["name", "value"] {
        for length in [1_048_576, 1_048_577] {
            let mut entry = serde_json::json!({"name":"B","value":"two"});
            entry[member] = "x".repeat(length).into();
            let input = serde_json::to_vec(
                &serde_json::json!({"application":{"environment":[{"name":"A","value":"one"},entry]}}),
            )
            .unwrap();
            let document = config::document(
                &input,
                &config::DocumentLimits { bytes: u32::try_from(input.len()).unwrap(), tokens: 128, depth: 8 },
            )
            .unwrap();
            let at = document.member(document.member(0, b"application").unwrap(), b"environment").unwrap();
            let mut problems = startup::Problems::empty();
            let entries = application::environment(
                &document,
                at,
                &config::Path::root().member(b"application").member(b"environment"),
                &mut problems,
            );
            if length == 1_048_576 {
                assert!(problems.is_empty());
                assert_eq!(entries.len(), 2);
            } else {
                assert_eq!(problems.problems.len(), 1);
                assert_eq!(problems.problems.get(0).unwrap().what, startup::What::Jig(config::What::Bound));
                let lines = startup::lines(&problems);
                let line = core::str::from_utf8(&lines).unwrap();
                assert!(line.starts_with(&format!("application.environment[1].{member}: ")), "{line}");
            }
        }
    }
}
