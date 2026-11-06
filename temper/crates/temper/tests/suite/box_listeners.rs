//! The real box's first block and held socket (temper/docs/design/protocol/setup.md, 7.2 and 13.4).
#[path = "../fake_backends/box_link.rs"]
mod box_link;
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
                document["application"]["machine"]["owned"]["listen"]["socket"]
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
    let output = command(&root, "init").arg("--box").output().expect("init");
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
    assert!(!diagnostic.contains("Open this link") && !diagnostic.contains("It signs you in"), "no block on refusal");
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn a_fresh_box_prints_its_block_and_nothing_else() {
    let mut boxed = box_link::Boxed::start("first-block", None);
    let link = boxed.link();
    boxed.wait(|boxed| crate::support::boxed::serve_listening(&boxed.root().join("data")));
    let (stdout, stderr, log) = boxed.stop();
    assert!(stdout.strip_suffix(b"\n") == Some(link.as_slice()), "stdout is exactly the first link and its line feed");
    assert!(
        stderr == [box_link::PRELUDE, box_link::FOOTER].concat(),
        "the complete sign-in block, with no lifecycle chatter"
    );
    let mut remaining = log.as_str();
    for fact in ["First run: ", "Logs: ", "Ctrl-C stops temper.", "Starting serve", "Stopping", "Stopped"] {
        let entry =
            remaining.lines().find(|line| line.get(10..).is_some_and(|text| text.starts_with(fact))).expect(fact);
        let time = entry.get(..10).expect("UTC timestamp").as_bytes();
        assert!(
            time.iter().enumerate().all(|(at, byte)| match at {
                2 | 5 => *byte == b':',
                8 => *byte == b'Z',
                9 => *byte == b' ',
                _ => byte.is_ascii_digit(),
            }),
            "HH:MM:SSZ prefix"
        );
        remaining = remaining.split_once(entry).expect("ordered lifecycle fact").1;
    }
    assert!(!log.contains("http://") && !log.contains("secret="), "box.log holds no link");
}

#[test]
fn a_box_without_openssl_logs_sign_in_unavailable_once_and_stays_up() {
    let mut boxed = box_link::Boxed::start("no-openssl", Some(""));
    boxed.link();
    boxed.wait(|boxed| crate::support::boxed::serve_listening(&boxed.root().join("data")));
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(boxed.root().join("config/temper.json")).expect("box document"))
            .expect("JSON");
    assert!(document["engine"]["clients"]["provider"].is_null());
    assert!(document["application"]["machine"]["owned"]["sign_in"].is_null());
    let (_, _, log) = boxed.stop();
    let first = log.lines().next().expect("first-run log").get(10..).expect("timestamp");
    assert_eq!(first, "First run: no sign-in through temper: openssl is not on PATH");
    assert_eq!(log.matches("no sign-in through temper:").count(), 1);
}

#[test]
fn the_first_link_opened_at_once_waits_and_signs_in() {
    let mut boxed = box_link::Boxed::start("first-open", None);
    let link = boxed.link();
    let connected = std::net::TcpStream::connect(box_link::address(&link));
    let listening = crate::support::boxed::serve_listening(&boxed.root().join("data"));
    eprintln!("serve.log had listening clients at the first connect: {listening}; early open: {}", !listening);
    let connected = connected.expect("the first link's socket accepts immediately");
    drop(connected);
    assert!(box_link::signs_in(boxed.root(), "first-client", &link), "the block's very first link signs in as owner");
    let (stdout, stderr, _) = boxed.stop();
    assert!(
        stdout.strip_suffix(b"\n") == Some(link.as_slice()),
        "signing in with the first link prints no second link"
    );
    assert!(stderr == [box_link::PRELUDE, box_link::FOOTER].concat(), "no fresh-link separator");
}

mod lifetime;
