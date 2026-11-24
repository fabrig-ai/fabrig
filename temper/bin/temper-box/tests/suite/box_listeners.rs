//! The real box's first block and held socket (temper/docs/design/protocol/setup/box-supervision.md, 1, and temper/docs/design/protocol/setup/testing.md, 4).
use box_link::command;
use skein_world::{Deadline, Expired};
use std::{
    fs,
    net::{SocketAddr, TcpListener},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    process::{Child, Stdio},
    time::Duration,
};
use temper_box_kit::box_link;

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("temper-box-listeners-{}-{name}", std::process::id()));
    fs::create_dir(&root).expect("fresh scratch");
    root
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
    let mut child = command(&root, "").stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("box");
    let dirs = temper_agent_fixtures::directories(&root.join("config"));
    let mut address = None;
    let mut deadline = Deadline::arm("the box's listener document", skein_lib::Duration::from_secs(3));
    deadline.watch_process(child.id());
    loop {
        if let Ok(read) = temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone) {
            let document = serde_json::from_slice(&read.document).expect("start document");
            address = Some(clients(&document));
            assert!(document["application"].get("machine").is_none());
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
    // Init is over and the host retains its deployment listeners.
    std::thread::sleep(Duration::from_millis(100));
    let running = child.try_wait().expect("box status").is_none();
    let held = address.map(|address| TcpListener::bind(address).is_err());
    let dormant = !root.join("data/forge-3").exists();
    stop(&mut child);
    fs::remove_dir_all(&root).expect("cleanup");
    assert!(running, "box stays up");
    assert_eq!(held, Some(true), "the configured clients port stays reserved");
    assert!(dormant, "the owned forge has not been prepared");
}

#[test]
fn a_later_box_refuses_a_taken_port_naming_the_port_and_document() {
    let root = scratch("taken");
    let output = command(&root, "init").arg("--box").output().expect("init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let file = root.join("config").join(temper_config_shell::FILE);
    let dirs = temper_agent_fixtures::directories(&root.join("config"));
    let mut configuration =
        temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file").file;
    let holder = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("held port");
    let address = holder.local_addr().expect("address");
    configuration.listen = Some(address);
    fs::write(&file, temper_config_shell::render(&configuration, temper_config_shell::Shape::Standalone))
        .expect("configured taken port");
    let output = command(&root, "").output().expect("box");
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
    let path = box_link::link_path(boxed.root());
    let local = kept_link(&mut boxed);
    assert!(local != link, "the file's link differs from the printed link");
    assert_eq!(box_link::address(&local), box_link::address(&link), "both links name the same listener");
    assert_eq!(fs::metadata(&path).expect("link metadata").mode() & 0o777, 0o600);
    assert_eq!(fs::metadata(path.parent().unwrap()).expect("parent metadata").mode() & 0o777, 0o700);
    let (stdout, stderr, log) = boxed.stop();
    let secret = std::str::from_utf8(&local).expect("the file's URL").split_once("secret=").expect("local secret").1;
    assert!(!String::from_utf8_lossy(&stdout).contains(secret), "the file's secret is absent from stdout");
    assert!(!String::from_utf8_lossy(&stderr).contains(secret), "the file's secret is absent from stderr");
    assert!(!log.contains(secret), "the file's secret is absent from box.log");
    assert!(!path.exists(), "removed at stop");
    assert!(stdout.strip_suffix(b"\n") == Some(link.as_slice()), "stdout is exactly the first link and its line feed");
    assert!(
        stderr == [box_link::PRELUDE, box_link::footer(&link).as_slice()].concat(),
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

fn kept_link(boxed: &mut box_link::Boxed) -> Vec<u8> {
    let path = box_link::link_path(boxed.root());
    boxed.wait(|_| fs::read(&path).is_ok_and(|bytes| bytes.ends_with(b"\n")));
    let bytes = fs::read(&path).expect("the file's link");
    let link = bytes.strip_suffix(b"\n").expect("one final line feed");
    assert!(!link.is_empty() && !link.contains(&b'\n'), "the file holds one link and a line feed");
    link.to_vec()
}

#[path = "../support/listeners.rs"]
pub(super) mod inherited;
mod lifetime;
