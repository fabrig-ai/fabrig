//! The box holds serve's clients port before any forge is asked to start.
use skein_world::{Deadline, Expired};
use std::{
    fs,
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("temper-box-listeners-{}-{name}", std::process::id()));
    fs::create_dir(&root).expect("fresh scratch");
    root
}
fn command(root: &Path, action: &str) -> Command {
    let mut command = Command::new(skein_scratch::paths::binary("temper"));
    command.arg(action);
    for name in ["config", "data", "cache"] {
        command.arg(format!("--{name}")).arg(root.join(name));
    }
    command
}
fn stop(child: &mut Child) {
    #[cfg(target_os = "linux")]
    let observed = crate::support::parent::Guard::observing_test();
    crate::support::parent::end_child(child);
    #[cfg(target_os = "linux")]
    assert!(observed.all_exited(), "no process outlived the fixture box");
}
fn clients(document: &serde_json::Value) -> SocketAddr {
    document["application"]["clients"]["listener"]["socket"]
        .as_str()
        .expect("clients listener")
        .parse()
        .expect("socket")
}

#[test]
fn a_fresh_box_holds_serves_clients_port() {
    let root = scratch("first");
    let mut child = command(&root, "box").stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("box");
    let document = root.join("config/temper.json");
    let mut address = None;
    let mut owned = None;
    let mut deadline = Deadline::arm("the box's listener document", skein_lib::Duration::from_secs(3));
    deadline.watch_process(child.id());
    loop {
        if let Ok(bytes) = fs::read(&document)
            && let Ok(document) = serde_json::from_slice(&bytes)
        {
            address = Some(clients(&document));
            owned = Some(
                document["application"]["setup"][1]["mode"]["owned"]["listen"]["socket"]
                    .as_str()
                    .expect("owned listener")
                    .parse::<SocketAddr>()
                    .expect("socket"),
            );
            break;
        }
        if child.try_wait().expect("status").is_some() {
            break;
        }
        if deadline.passed() {
            let expired = deadline.expired();
            stop(&mut child);
            fs::remove_dir_all(&root).expect("cleanup after unanswered listener wait");
            match expired {
                Expired::Starved(starved) => starved.pass(),
                Expired::Failed(figures) => panic!("the configured listener did not arrive: {figures}"),
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    // Init is over and the host retains the reservation even while the group stays dormant.
    std::thread::sleep(Duration::from_millis(100));
    let running = child.try_wait().expect("box status").is_none();
    let held = address.map(|address| TcpListener::bind(address).is_err());
    let owned_held = owned.map(|address| TcpListener::bind(address).is_err());
    let dormant = !root.join("data/forge-3").exists();
    stop(&mut child);
    fs::remove_dir_all(&root).expect("cleanup");
    assert!(running, "box stays up");
    assert_eq!(held, Some(true), "the configured clients port stays reserved");
    assert!(dormant, "the owned forge has not been prepared");
    assert_eq!(owned_held, Some(cfg!(target_os = "linux")), "Linux holds the dormant owned forge's port");
}

#[test]
fn a_later_box_refuses_a_taken_port_naming_the_port_and_document() {
    let root = scratch("taken");
    let output = command(&root, "init").output().expect("init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let file = root.join("config/temper.json");
    let bytes = fs::read(&file).expect("document");
    let document = serde_json::from_slice(&bytes).expect("JSON");
    let previous = clients(&document);
    let holder = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("held port");
    let address = holder.local_addr().expect("address");
    let text = String::from_utf8(bytes).expect("UTF-8").replace(&previous.to_string(), &address.to_string());
    fs::write(&file, text).expect("configured taken port");
    let output = command(&root, "box").output().expect("box");
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "taken listener refuses the box");
    assert!(diagnostic.contains(&address.port().to_string()), "{diagnostic}");
    assert!(diagnostic.contains(file.to_str().expect("UTF-8 scratch path")), "{diagnostic}");
    assert!(diagnostic.contains("cannot listen"), "{diagnostic}");
    assert!(output.stdout.is_empty(), "refused before steps start");
    fs::remove_dir_all(&root).expect("cleanup");
}
