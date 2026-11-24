//! Scripted openings over prepared scratch deployments.
use serde_json::Value;
use smith_domain_run as run;
use temper_box_kit::Scratch;
use temper_box_kit::{
    cluster,
    fake_opening::{answer, calls, read, says},
    github_land,
};

fn cluster_fixture(name: &str, source: &str, configure: bool) -> Value {
    let scratch = Scratch::new(name);
    let directory = skein_scratch::paths::package().join(source);
    let mut description = cluster::Cluster::lay_description_from(scratch.path(), &directory);
    description.bind("proc.forge.ready.address", "127.0.0.1:12345");
    description.prepare("llm");
    let path = description.dir("llm").join("llm.json");
    if configure {
        github_land::configure_llm(&path);
    }
    let value = read(&path);
    scratch.remove();
    value
}

#[test]
fn github_coder_fixture_keeps_its_finite_system_cue() {
    let value = cluster_fixture("github_coder_opening", "tests/fixtures/github", false);
    calls(&answer(&value, "@coding Fix", run::Mode::Finite), "read");
}

#[test]
fn github_chat_fixture_selects_its_delegate_from_the_opening_user_message() {
    let value = cluster_fixture("github_opening", "tests/fixtures/github", false);
    calls(&answer(&value, "@forge_chat Update README", run::Mode::Open), "delegate");
}

#[test]
fn github_landing_fixture_selects_its_change_from_the_opening_user_message() {
    let value = cluster_fixture("github_landing_opening", "tests/fixtures/github", true);
    calls(&answer(&value, "@forge_chat Update README", run::Mode::Open), "change");
}

#[test]
fn cluster_chat_fixtures_select_their_opening_requests() {
    let value = cluster_fixture("cluster_opening", "tests/fixtures/cluster", false);
    for (words, expected) in
        [("cluster-first-chat", "First worker replied."), ("cluster-second-chat", "Second worker replied.")]
    {
        says(&answer(&value, words, run::Mode::Open), expected);
    }
    calls(&answer(&value, "Add a file NOTICE to widget", run::Mode::Open), "delegate");
}
