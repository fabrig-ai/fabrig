//! Startup fixtures shared by the reference binary's tests (tools/crates/design.md, 3.2).

use skein_io::kernel::Fd;
use skein_shell::{RootBounds, Roots, RootsRefusal, Trust, Unread};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::StartupReads;

type ReadRoots<'a> = dyn FnMut(&Trust, &RootBounds) -> Result<Roots, RootsRefusal> + 'a;

/// The documents, private files and trust-read answers supplied by a startup step test.
pub struct Given<'a> {
    pub files: &'a [GivenFile<'a>],
    pub read_roots: &'a mut ReadRoots<'a>,
}

/// The bytes or refusal for one path named by startup.
pub struct GivenFile<'a> {
    pub path: &'a Path,
    pub contents: Result<&'a [u8], Unread>,
}

impl<'a> Given<'a> {
    pub(super) fn file(&self, path: &Path, max: u32) -> Result<&'a [u8], Unread> {
        let bytes = self.files.iter().find(|file| file.path == path).ok_or(Unread::Absent)?.contents?;
        if skein_lib::len_u64(bytes.len()) > u64::from(max) {
            return Err(Unread::TooLarge { max });
        }
        Ok(bytes)
    }
}

/// Check startup's documents using the supplied answers instead of reading files.
pub fn check_with(configuration: &Path, given: Given<'_>) -> Result<(), String> {
    super::check_with(configuration, &mut StartupReads::Given(given))
}

/// Exercise startup preparation with supplied reads, refusing before native runtime setup if the reads succeed.
pub fn prepare_with(
    configuration: &Path,
    owned: &mut Vec<Fd>,
    given: Given<'_>,
) -> Result<jig_test_service::Service, String> {
    let mut reads = StartupReads::Given(given);
    let bytes = super::read_configuration(configuration, &reads)?;
    super::prepare(&bytes, owned, &mut reads)
}

pub fn prepare(bytes: &[u8], owned: &mut Vec<Fd>) -> Result<jig_test_service::Service, String> {
    super::prepare(bytes, owned, &mut StartupReads::Native)
}

pub fn read_startup_file(name: &[u8], max: u32, user: u32) -> Result<skein_lib::Secret, String> {
    super::read_startup_file(name, max, user, &StartupReads::Native)
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
    let path = directory.join("agent.json");
    fs::write(&path, agent_endpoint_document_bytes(directory, program, program_endpoint)).expect("agent configuration");
    path
}

/// Build the same agent fixture as [`agent_document`] without writing it.
#[must_use]
pub fn agent_document_bytes(directory: &Path, program: bool) -> Vec<u8> {
    agent_endpoint_document_bytes(directory, program, false)
}

fn agent_endpoint_document_bytes(directory: &Path, program: bool, program_endpoint: bool) -> Vec<u8> {
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
    serde_json::to_vec(&value).expect("agent fixture JSON")
}
