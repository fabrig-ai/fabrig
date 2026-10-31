//! Machine setup through the shipped binary (temper/docs/design/draft/playground-reset.md, 3).
use crate::{listeners, support};

use std::{collections::BTreeSet, fs, net::TcpListener, path::Path, process::Command};

use serde_json::Value;
use skein_io::kernel::Exit;

fn init(directory: &Path) -> (Value, String) {
    let output = Command::new(skein_scratch::paths::binary("temper"))
        .arg("init")
        .args(support::configuration::flags(directory))
        .env_clear()
        .output()
        .expect("run init");
    assert!(output.status.success(), "init succeeds");
    assert!(output.stdout.is_empty(), "init has no stdout");
    let value = serde_json::from_slice(&fs::read(directory.join("temper.json")).expect("document")).expect("JSON");
    (value, String::from_utf8(output.stderr).expect("stderr text"))
}

fn boxed(directory: &Path) {
    support::own_listener_files(directory);
    let ended = support::boxed::run(directory, None, support::boxed::Plan::Stop, support::boxed::BOX_DEADLINE);
    assert!(support::printed_link(&ended.stdout).is_some(), "box reaches sign-in");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "box stops cleanly");
    assert!(ended.left.as_ref().is_some_and(Vec::is_empty), "no surviving children");
}

#[test]
fn init_wires_the_owned_machine_without_a_forge_record() {
    let scratch = support::Scratch::startup("qa-setup-members");
    let (value, _) = init(scratch.path());
    let application = &value["application"];
    assert!(application.get("forges").is_none());
    assert!(application.get("setup").is_none());
    let owned = &application["machine"]["owned"];
    assert_eq!(owned["remote"], format!("http://{}", owned["listen"]["socket"].as_str().expect("socket")));
    assert_eq!(owned["forgejo"], "fetched");
    assert_eq!(owned["runner"]["binary"], "fetched");
    assert_eq!(owned["runner"]["labels"], serde_json::json!(["temper-host:host"]));
    assert_eq!(owned["runner"]["capacity"], 1);
    assert!(owned["runner"]["path"].as_str().is_some_and(|path| !path.is_empty()));
    let fake = &application["machine"]["fake"];
    assert_eq!(fake["remote"], format!("http://{}", fake["listen"]["socket"].as_str().expect("socket")));
    assert!(!scratch.path().join("forge-1").exists());
    scratch.remove();
}

#[test]
fn init_uses_the_effective_uids_login_or_names_its_fallback() {
    let scratch = support::Scratch::startup("qa-setup-person");
    let (value, stderr) = init(scratch.path());
    let uid = Command::new("/usr/bin/id").arg("-u").output().expect("effective uid");
    assert!(uid.status.success());
    let uid = String::from_utf8(uid.stdout).expect("uid text");
    let passwd = fs::read_to_string("/etc/passwd").expect("passwd");
    let found = passwd.lines().find_map(|line| {
        let fields: Vec<_> = line.split(':').collect();
        (fields.get(2) == Some(&uid.trim())).then(|| fields[0])
    });
    let person = &value["application"]["machine"]["owned"]["person"];
    let login = person["login"].as_str().expect("person login");
    if let Some(found) = found {
        if login == found {
            assert!(!stderr.contains("cannot be a Forgejo login"));
        } else {
            assert_eq!(login, "person");
            assert!(stderr.contains(&format!("init: {found} cannot be a Forgejo login: forge 3's person is person")));
        }
    } else {
        assert_eq!(login, "person");
        assert!(
            stderr
                .contains(&format!("init: uid {} has no login in /etc/passwd: forge 3's person is person", uid.trim()))
        );
    }
    assert_eq!(person["email"], format!("{login}@localhost"));
    assert!(person.get("orgs").is_none());
    scratch.remove();
}

#[test]
fn init_chooses_five_distinct_fixed_ports_when_defaults_are_occupied() {
    let held: Vec<_> = (8367..=8371).map(|port| TcpListener::bind(("127.0.0.1", port))).collect();
    assert!(
        held.iter()
            .all(|result| result.as_ref().err().is_none_or(|error| error.kind() == std::io::ErrorKind::AddrInUse))
    );
    let scratch = support::Scratch::startup("qa-setup-ports");
    let (value, _) = init(scratch.path());
    let application = &value["application"];
    let sockets = [
        &application["clients"]["listener"],
        &application["machine"]["owned"]["listen"],
        &application["machine"]["fake"]["listen"],
        &application["demo"]["llm"]["listen"],
        &application["webhooks"]["listener"],
    ];
    let ports: BTreeSet<_> = sockets
        .iter()
        .map(|socket| {
            let address: std::net::SocketAddr = socket["socket"].as_str().expect("socket").parse().expect("address");
            assert!(address.ip().is_loopback());
            assert_ne!(address.port(), 0);
            assert!(!(8367..=8371).contains(&address.port()), "an occupied default is replaced");
            address.port()
        })
        .collect();
    assert_eq!(ports.len(), 5, "all five chosen sockets must be held together");
    scratch.remove();
}

#[test]
fn a_box_on_inits_document_leaves_forges_and_cache_absent() {
    let scratch = support::Scratch::startup("qa-setup-dormant");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    let flags = [
        "--config".into(),
        config.as_os_str().to_owned(),
        "--data".into(),
        data.as_os_str().to_owned(),
        "--cache".into(),
        cache.as_os_str().to_owned(),
    ];
    let output = Command::new(skein_scratch::paths::binary("temper"))
        .arg("init")
        .args(&flags)
        .env_clear()
        .output()
        .expect("init");
    assert!(output.status.success());
    support::own_listener_files(&config);
    let command = skein_world::end_to_end::Command {
        program: skein_scratch::paths::binary("temper"),
        arguments: std::iter::once("box".into()).chain(flags).collect(),
        directory: scratch.path().into(),
        environment: Vec::new(),
    };
    let ended = support::boxed::run_command(command, None, support::boxed::Plan::Stop, support::boxed::BOX_DEADLINE);
    assert!(support::printed_link(&ended.stdout).is_some(), "box reaches sign-in");
    assert_eq!(ended.exit, Some(Exit::Code(0)));
    assert!(ended.left.as_ref().is_some_and(Vec::is_empty));
    assert!(!data.join("forge-1").exists(), "no preparation before an ask");
    assert_eq!(fs::read_dir(cache).expect("cache directory").count(), 0, "no fetched binaries before an ask");
    scratch.remove();
}

#[test]
fn serve_with_box_refuses_without_the_boxs_pipes() {
    let scratch = support::Scratch::startup("qa-setup-no-pipes");
    init(scratch.path());
    boxed(scratch.path());
    let mut arguments = vec!["internal".into(), "serve".into(), "--box".into()];
    arguments.extend(support::configuration::flags(scratch.path()));
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let mut command = listeners::command(listener, "exec \"$@\" 3<&0 4>&- 5>&- 0</dev/null");
    command.args(arguments);
    let ended = command.output().expect("serve");
    assert_eq!(ended.status.code(), Some(1));
    assert!(support::printed_link(&ended.stdout).is_none());
    let stderr = String::from_utf8_lossy(&ended.stderr);
    assert!(stderr.contains("fd 4"), "refusal names the pipe after the listener: {stderr}");
    scratch.remove();
}

#[test]
fn qa_box_drain_does_not_report_its_own_pipe_as_closed() {
    // protocol/forge.md 2.8: pipe failure is named once; serve's own end is the box's to make.
    let scratch = support::Scratch::startup("qa-m2-box-drain");
    init(scratch.path());
    support::own_listener_files(scratch.path());
    let ended = support::boxed::run(scratch.path(), None, support::boxed::Plan::Stop, support::boxed::BOX_DEADLINE);
    assert!(support::printed_link(&ended.stdout).is_some(), "box reaches sign-in");
    assert_eq!(ended.exit, Some(Exit::Code(0)));
    assert!(ended.left.as_ref().is_some_and(Vec::is_empty));
    let stderr = String::from_utf8_lossy(&ended.stderr);
    assert!(!stderr.contains("box closed"), "normal drain must not report box closed: {stderr}");
    assert!(!stderr.contains("box link:"), "an idle link has no diagnostic at drain: {stderr}");
    assert!(!scratch.path().join("forge-1").exists(), "no ask, no owned forge preparation");
    scratch.remove();
}
