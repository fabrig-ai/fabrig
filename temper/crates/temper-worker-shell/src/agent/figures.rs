//! Smith's document from figures and this machine (protocol/configuration/figures.md, 6).
use jig_protocol_worker_wire::{Figures, Oversized, Transport};
use serde_json::{Value, json};
use std::{os::unix::ffi::OsStrExt, path::Path};
use temper_shell::directories::EnvironmentEntry;
fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|why| format!("the engine's figures: {why}"))
}
/// `claude` is the entry's explicit program or the one found on PATH; none selects fetched.
pub fn configuration(
    figures: &Figures,
    claude: Option<&Path>,
    data: &Path,
    environment: &[EnvironmentEntry],
) -> Result<Box<[u8]>, String> {
    let mut profile = json!({"name":"standard", "declared":{}, "policy":{}});
    for limit in figures.agents().profile() {
        let name = text(limit.name())?;
        let (group, member) = name.split_once('.').ok_or_else(|| format!("the engine's figures: {name}: no group"))?;
        let object = profile
            .get_mut(group)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| format!("the engine's figures: {name}: unknown profile group"))?;
        object.insert(member.into(), limit.value().into());
    }
    let mut endpoints = Vec::new();
    for endpoint in figures.agents().endpoints() {
        let limits = endpoint.limits();
        let host = text(endpoint.host())?;
        let address = if host.contains(':') {
            format!("[{host}]:{}", endpoint.port())
        } else {
            format!("{host}:{}", endpoint.port())
        };
        // Smith names the protocol provider; engine providers may have local names such as demo.
        let provider = match endpoint.dialect() {
            3 => "openai",
            2 => "anthropic",
            dialect => return Err(format!("the engine's figures: unsupported endpoint dialect {dialect}")),
        };
        let mut value = json!({
            "name":text(endpoint.name())?, "number":endpoint.number(), "dialect":endpoint.dialect(),
            "provider":provider, "address":address,
            "connect":endpoint.connect().as_millis(), "handshake":endpoint.handshake().as_millis(),
            "limits": { "window":limits.window(), "output":limits.output(), "reasoning_item":limits.reasoning_item(),
                "head":limits.head().as_millis(), "idle":limits.idle().as_millis(),
                "oversized_reasoning":match limits.oversized() { Oversized::Fail => "fail", Oversized::Drop => "drop" }
            }
        });
        if let Some(program) = endpoint.program() {
            value["program"] = text(program)?.into();
        }
        match endpoint.transport() {
            Transport::Plaintext => value["transport"] = "plaintext".into(),
            Transport::Tls(tls) => {
                value["transport"] = "tls".into();
                value["server_name"] = text(tls.server_name())?.into();
                if let Some(trust) = tls.trust() {
                    value["trust_der"] = text(trust)?.into();
                }
            }
        }
        let mut headers = Vec::new();
        for header in endpoint.headers() {
            headers.push(json!({"name":text(header.name())?, "value":text(header.value())?}));
        }
        value["headers"] = headers.into();
        endpoints.push(value);
    }
    let program = match claude {
        Some(path) => text(path.as_os_str().as_bytes())?,
        None => "fetched",
    };
    let directory = data.join("programs");
    let mut variables = Vec::new();
    for (name, value) in environment {
        variables.push(json!({"name":text(name)?, "value":text(value)?}));
    }
    serde_json::to_vec(&json!({"profile":profile, "endpoints":endpoints,
        "programs":{"claude_code":{"path":program,"directory":text(directory.as_os_str().as_bytes())?}},
        "environment":variables}))
    .map(Vec::into_boxed_slice)
    .map_err(|why| why.to_string())
}
