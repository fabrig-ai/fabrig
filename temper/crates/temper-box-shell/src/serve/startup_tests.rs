use crate::{box_command, box_error, serve::operations_with_copies, setup};
use std::path::Path;
use temper_box_kit::{Forge, configuration::directories, recipe};
use temper_service::stock::Given;
use temper_service_shell::{jig_directories, serve_error};

const ROOT: &str = "/fixture/startup";

#[test]
fn problems_reach_stderr_one_line_each() {
    assert_eq!(
        operations_with_copies("temper", &directories(Path::new(ROOT)), b"{}", &Given::Stock, None, Ok(None), Ok(None))
            .expect_err("refused"),
        "temper serve: engine: missing\ntemper serve: workers: missing\ntemper serve: application: missing\n"
    );
}

fn refused_serve(edit: impl FnOnce(&mut serde_json::Value)) -> String {
    let root = Path::new(ROOT);
    let mut document = recipe(root, &Forge::Unreached).start_document();
    edit(&mut document);
    operations_with_copies(
        "temper",
        &directories(root),
        &serde_json::to_vec(&document).expect("start JSON"),
        &Given::Stock,
        None,
        Ok(None),
        Ok(None),
    )
    .expect_err("start refusal")
}

#[test]
fn a_document_charter_is_refused_naming_temper_ctl_settings() {
    let why = refused_serve(|document| document["engine"]["charters"] = serde_json::json!([]));
    assert_eq!(
        why,
        "temper serve: engine.charters: settings live in the store; read them with temper ctl DIR settings show\n"
    );
}

#[test]
fn an_inline_seed_policy_is_refused_naming_its_path() {
    let original: serde_json::Value =
        serde_json::from_slice(&jig_channel_kit::fixtures::engine_document()).expect("jig fixture");
    let why = refused_serve(|document| document["engine"]["projects"] = original["engine"]["projects"].clone());
    assert_eq!(
        why,
        "temper serve: engine.projects: moved: projects are made at runtime; make one from the client or temper ctl\n"
    );
}

#[test]
fn a_setup_limit_past_its_check_refuses_temper_serve_naming_both_limits() {
    let root = Path::new(ROOT);
    let mut recipe = recipe(root, &Forge::Unreached);
    recipe.configuration.temper_over("setup.backoff_ms", 6000);
    // Serve calls this check with typed limits before constructing a worker or opening its store.
    let why = setup::check_over(
        &recipe.read_start().document,
        &jig_directories(&directories(root)),
        &recipe.given,
        Some((&recipe.configuration.jig, &recipe.configuration.limits)),
    )
    .expect_err("configuration refused");
    assert_eq!(
        serve_error("temper", &why),
        "temper serve: application.limits.overrides.setup.backoff_ms: must agree with application.limits.overrides.setup.backoff_max_ms\n"
    );
}

#[test]
fn a_setup_naming_no_forge_refuses_the_box_before_it_starts() {
    let root = Path::new(ROOT);
    let mut document = recipe(root, &Forge::Unreached).start_document();
    document["application"]["setup"] = serde_json::json!([{ "forge":7 }]);
    document["application"]["setup"][0]["forge"] = 7.into();
    // Without a plan no step can reach the box's machine, including creating forge-7.
    let why = box_command::configuration_steps(
        &directories(root),
        &serde_json::to_vec(&document).expect("start JSON"),
        Box::new([]),
    )
    .expect_err("box refused before building steps");
    assert_eq!(box_error(&why), "temper-box: application.setup: unknown member\n");
}
