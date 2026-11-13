pub(crate) use crate::test_configuration as configuration;

pub(crate) fn worker_document(directory: &std::path::Path) -> (Vec<u8>, serde_json::Value) {
    let (agent, input) = configuration::documents(directory, "/missing/temper");
    let mut input: serde_json::Value = serde_json::from_slice(&input).expect("fixture document");
    input.as_object_mut().expect("document object").remove("engine");
    input["workers"][0]["engine"] = serde_json::json!({"socket":"127.0.0.1:7100"});
    input["application"] =
        serde_json::json!({"workers":input["application"]["workers"],"limits":input["application"]["limits"]});
    (agent, input)
}

#[test]
fn a_programs_owner_refusal_identifies_the_directory_ownership_conflict() {
    use smith_agent_document::program::{Refusal, Unsafe};
    let other = skein_shell::effective_user().wrapping_add(1);
    let why = super::agent::program_directory_refusal(&Refusal::Unsafe {
        path: "/programs".into(),
        found: Unsafe::Owner { found: other },
    });
    assert_eq!(
        why,
        format!("workers[0].agent.configuration: programs.claude_code.directory: /programs: owned by user {other}")
    );
}

#[test]
fn the_dump_directory_is_set_by_a_value_and_made_absolute() {
    use std::path::PathBuf;
    let working = std::env::current_dir().expect("test working directory");
    for (row, value, expected) in [
        ("absent", None, None),
        ("empty", Some(""), None),
        ("relative", Some("request-dumps"), Some(working.join("request-dumps"))),
        ("absolute", Some("/request-dumps"), Some(PathBuf::from("/request-dumps"))),
    ] {
        let environment: Vec<_> = value
            .into_iter()
            .map(|value| (Box::from(b"TEMPER_INTERNAL_DUMP_LLM".as_slice()), Box::from(value.as_bytes())))
            .collect();
        assert_eq!(super::dump_llm(&environment), expected, "{row}");
    }
}

#[test]
fn workspace_reuse_reads_nonempty_values_from_the_given_environment() {
    for (value, expected) in [(None, false), (Some("1"), true), (Some(""), false), (Some("on"), true)] {
        let environment: Vec<_> = value
            .into_iter()
            .map(|value| (Box::from(b"TEMPER_INTERNAL_WORKSPACE_REUSE".as_slice()), Box::from(value.as_bytes())))
            .collect();
        assert_eq!(super::workspace_reuse(&environment), expected, "{value:?}");
    }
}

mod pin;
