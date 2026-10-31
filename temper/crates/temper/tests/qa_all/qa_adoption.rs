//! Init listeners and an empty box (temper/docs/design/draft/playground-reset.md, 3 and 6.1–6.3).
use crate::support;

use std::{fs, net::TcpListener, path::PathBuf, process::Command};

use skein_io::kernel::Exit;

#[test]
fn init_names_a_fixed_listener_without_a_forge() {
    let scratch = support::Scratch::startup("qa-adoption-init");
    let flags = support::configuration::flags(scratch.path());
    let output =
        Command::new(skein_scratch::paths::binary("temper")).arg("init").args(&flags).output().expect("run init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let document_path = flags
        .iter()
        .map(PathBuf::from)
        .map(|directory| directory.join("temper.json"))
        .find(|path| path.is_file())
        .expect("init's document in its explicit configuration directory");
    let bytes = fs::read(&document_path).expect("read init's document");
    let document: serde_json::Value = serde_json::from_slice(&bytes).expect("document JSON");
    let listener = document["application"]["webhooks"]["listener"]["socket"]
        .as_str()
        .expect("init names the webhooks listener")
        .parse::<std::net::SocketAddr>()
        .expect("listener socket");
    assert!(listener.ip().is_loopback());
    assert_ne!(listener.port(), 0, "Adopt needs the document's fixed hook address");
    assert!(document["application"].get("forges").is_none(), "forges are made as records when chosen");
    temper::operations_in(&support::configuration::directories(scratch.path()), &bytes)
        .unwrap_or_else(|why| panic!("serve must accept init's wiring: {why}"));
    scratch.remove();
}

#[test]
fn occupied_defaults_still_give_adopt_a_distinct_fixed_listener() {
    let scratch = support::Scratch::startup("qa-adoption-fallback");
    let _occupied: Vec<_> = (8367..=8371).filter_map(|port| TcpListener::bind(("127.0.0.1", port)).ok()).collect();
    let output = Command::new(skein_scratch::paths::binary("temper"))
        .arg("init")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("run init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(scratch.path().join("temper.json")).expect("document")).expect("JSON");
    let app = &document["application"];
    let addresses: Vec<std::net::SocketAddr> = [
        &app["clients"]["listener"]["socket"],
        &app["machine"]["owned"]["listen"]["socket"],
        &app["machine"]["fake"]["listen"]["socket"],
        &app["demo"]["llm"]["listen"]["socket"],
        &app["webhooks"]["listener"]["socket"],
    ]
    .iter()
    .map(|value| value.as_str().expect("socket").parse().expect("address"))
    .collect();
    for (index, address) in addresses.iter().enumerate() {
        assert!(address.ip().is_loopback());
        assert_ne!(address.port(), 0, "init writes the assigned port, never port zero");
        assert!(!(8367..=8371).contains(&address.port()), "occupied default reused: {address}");
        assert!(!addresses[..index].contains(address), "two listeners share {address}");
    }
    scratch.remove();
}

#[test]
fn the_default_box_hands_over_the_link_without_starting_a_forge() {
    // Section 6.1 makes the playground only when chosen.
    let scratch = support::Scratch::startup("qa-adoption-default-box");
    let output = Command::new(skein_scratch::paths::binary("temper"))
        .arg("init")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("run init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    support::own_listener_files(scratch.path());
    let outcome = support::boxed::run(scratch.path(), None, support::boxed::Plan::Stop, support::boxed::BOX_DEADLINE);
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&outcome.stderr));
    assert!(support::printed_link(&outcome.stdout).is_some(), "the empty box hands over the sign-in link");
    assert_eq!(outcome.left.as_deref(), Some([].as_slice()));
    assert!(!scratch.path().join("demo/forge").exists(), "no playground before it is chosen");
    assert!(!scratch.path().join("forge-1").exists(), "no owned forge before it is chosen");
    scratch.remove();
}
