//! The shell starts from the re-exported document readers.

use crate::config::{Configuration, decode, from_value, parse};
use smith_agent_service as service;

const README: &str = include_str!("../../README.md");

fn json_blocks(text: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut remaining = text;
    while let Some((_, contents)) = remaining.split_once("```json\n") {
        let (block, rest) = contents.split_once("\n```").expect("JSON fence has a closing fence");
        blocks.push(block);
        remaining = rest;
    }
    blocks
}

#[test]
fn the_readme_configuration_decodes() {
    let blocks = json_blocks(README);
    assert!(!blocks.is_empty(), "the README shows a configuration");
    for block in blocks {
        decode(block.as_bytes()).expect("the README's configuration decodes");
    }
}

fn shaped_document() -> serde_json::Value {
    serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":256,"read_window":256}},
        "endpoints":[{"name":"local","number":0,"dialect":3,"provider":"openai",
            "address":"127.0.0.1:8080","transport":"plaintext","connect":10000,"handshake":10000,
            "limits": {"window":65536,"output":512,"reasoning_item":64,"head":60000,"idle":30000}}],
        "environment":[]
    })
}

fn parse_value(value: &serde_json::Value) -> Result<Configuration, String> {
    parse(&serde_json::to_vec(value).expect("configuration JSON"))
}

fn program_document() -> serde_json::Value {
    let mut document = shaped_document();
    document["programs"] = serde_json::json!({"claude_code": {
        "path": "/usr/local/bin/claude", "directory": "/home/u/.local/state/smith/programs"
    }});
    document
}

#[test]
fn named_endpoint_resolves_and_its_tls_trust_builds_before_start() {
    let mut document = shaped_document();
    let endpoint = &mut document["endpoints"][0];
    endpoint["transport"] = "tls".into();
    endpoint["server_name"] = "example.test".into();
    endpoint["trust_der"] =
        skein_scratch::paths::tree().join("tests/protocol-llm/fixtures/root.der").display().to_string().into();
    let configuration = from_value(document).expect("resolved endpoint");
    service::Service::new(configuration.service, 1).expect("LLM and channel endpoints");
}

#[test]
fn plaintext_endpoints_admit_loopback_addresses() {
    for address in ["127.0.0.1:8080", "[::1]:8080"] {
        let mut document = shaped_document();
        document["endpoints"][0]["address"] = address.into();
        let configuration = parse_value(&document).expect("loopback plaintext configuration");
        service::Service::new(configuration.service, 1).expect("loopback plaintext service");
    }
}

#[test]
fn a_declared_memory_of_one_byte_starts() {
    let mut document = shaped_document();
    document["profile"]["declared"]["memory"] = 1.into();
    let configuration = from_value(document).expect("one declared byte is accepted");
    assert_eq!(configuration.memory, 1);
    assert_eq!(configuration.service.limits.memory, 1);
    crate::Agent::new(configuration, super::resources(), Box::new(std::io::sink()))
        .expect("one declared byte starts through prepare");
}

#[test]
fn an_unbound_program_root_is_closed_by_the_shell() {
    use skein_io::kernel::{Complete, Done, Fd, Op};
    use skein_lib::{Secret, Time, Wall};
    use skein_shell::Host;

    let configuration = from_value(program_document()).expect("unbound declaration");
    let mut agent = crate::Agent::new(
        configuration,
        crate::Resources {
            programs: Some(crate::ProgramResources {
                relay: "/smith-agent".into(),
                entropy: Secret::new(Box::new([7; 32])),
                directory: Some(Fd::new(3)),
            }),
            ..super::resources()
        },
        Box::new(std::io::sink()),
    )
    .expect("unbound declaration starts with shell resources");
    let close = agent.submissions().pop().expect("unused root closes immediately");
    assert_eq!(close.kind, Op::Close { fd: Fd::new(3) });
    assert!(agent.submissions().is_empty());
    agent.completions().push(Complete { op: close.op, kind: close.kind, result: Ok(Done::Nothing) });
    agent.iterate(Time::ZERO, Wall::EPOCH);
    assert!(!agent.submissions().iter().any(|submit| matches!(submit.kind, Op::Close { fd } if fd == Fd::new(3))));
}

#[test]
fn an_unsafe_program_directory_logs_its_startup_refusal_once() {
    let scratch = skein_scratch::Scratch::new("program-diagnostic");
    let directory = scratch.path().join("not-a-directory");
    std::fs::write(&directory, b"fixture").expect("non-directory fixture");
    let mut document = program_document();
    document["programs"]["claude_code"]["directory"] = directory.to_str().expect("fixture path").into();
    let path = scratch.path().join("configuration.json");
    std::fs::write(&path, serde_json::to_vec(&document).expect("configuration")).expect("configuration file");
    let diagnostic = scratch.path().join("diagnostic");
    #[expect(clippy::disallowed_types, reason = "the test reads the owned diagnostic file after run returns")]
    let errors = std::fs::File::create(&diagnostic).expect("diagnostic file");
    let why = crate::run(&path, Box::new(errors)).expect_err("unsafe directory refused");
    assert!(why.contains("not a directory"), "the directory refusal identifies its cause: {why}");
    assert_eq!(std::fs::read_to_string(diagnostic).expect("diagnostic text"), format!("smith: {why}\n"));
}
