//! The story owns every listener across restarts (temper/docs/design/testing/process-tier.md).
use crate::support;
use std::{fs, net::SocketAddr, process::Command};

#[test]
fn own_listener_files_moves_each_listener_below_the_ephemeral_range() {
    let scratch = support::Scratch::new("chosen-listener-files");
    let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
    command.args(["init", "--box"]);
    for name in ["config", "data", "cache"] {
        command.arg(format!("--{name}")).arg(scratch.path().join(name));
    }
    let initialized = command.output().expect("initialize story");
    assert!(initialized.status.success(), "{}", String::from_utf8_lossy(&initialized.stderr));
    support::own_listener_files(&scratch.path().join("config"));
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(scratch.path().join("config/temper.json")).expect("deployment"))
            .expect("deployment JSON");
    assert!(document["application"].get("machine").is_none(), "a forge chooses its listener at set-up");
    for pointer in [
        "/application/clients/listener/socket",
        "/engine/llm/endpoints/3/address/socket",
        "/application/webhooks/listener/socket",
    ] {
        let address: SocketAddr =
            document.pointer(pointer).expect("listener").as_str().expect("socket").parse().expect("address");
        assert!(address.port() < temper_fixtures::ports::ephemeral_first(), "{pointer}: {address}");
    }
}
