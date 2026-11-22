//! Startup fixtures shared by the reference binary's tests (tools/crates/design.md, 3.2).

use skein_io::kernel::Fd;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn prepare(bytes: &[u8], owned: &mut Vec<Fd>) -> Result<jig_test_service::Service, String> {
    super::prepare(bytes, owned)
}

pub fn read_startup_file(name: &[u8], max: u32, user: u32) -> Result<skein_lib::Secret, String> {
    super::read_startup_file(name, max, user)
}

#[must_use]
pub fn scratch(case: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("jig-test-{}-{case}", std::process::id()));
    fs::create_dir(&path).expect("scratch directory");
    path
}

#[must_use]
pub fn agent_document(directory: &Path, program: bool) -> PathBuf {
    agent_endpoint_document(directory, program, false)
}
pub(crate) fn agent_endpoint_document(directory: &Path, program: bool, program_endpoint: bool) -> PathBuf {
    fs::write(directory.join("root.der"), skein_tls_kit::pki::ROOT).expect("public test root");
    let mut value = serde_json::json!({
        "profile":{"name":"standard","declared":{"tool_payload":512,"read_window":256},
            "policy":{"max_turns":5,"group_stop":10}},
        "endpoints":[{"name":"fake","number":0,"dialect":3,"provider":"openai",
            "address":"127.0.0.1:34443","transport":"tls","server_name":"skein.test","trust_der":directory.join("root.der"),"connect":1000,"handshake":1000,
            "limits": {"window":1_048_576,"output":1024,"reasoning_item":64,"head":60000,"idle":30000}}],
        "environment":[{"name":"JIG_SOURCE_CHECK","value":"configured"}]
    });
    if program {
        value["programs"] = serde_json::json!({"claude_code":{
            "path":directory.join("private-claude"),"directory":directory.join("private-programs")
        }});
    }
    if program_endpoint {
        value["endpoints"][0]["program"] = serde_json::json!("claude_code");
    }
    let path = directory.join("agent.json");
    fs::write(&path, serde_json::to_vec(&value).expect("agent fixture JSON")).expect("agent configuration");
    path
}
