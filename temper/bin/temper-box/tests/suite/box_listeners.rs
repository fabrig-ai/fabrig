//! The real box's first block and held socket (temper/docs/design/protocol/setup/box.md, 2, and temper/docs/design/protocol/setup/testing.md, 4).
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

/// Open a spent link through its wire route and inspect the reason, not a transport refusal.
fn used(link: &[u8]) -> std::io::Result<bool> {
    use crate::support::{
        self,
        story_socket::{Scope, Socket},
    };
    use jig_client_protocol_wire as wire;
    use skein_lib::{
        Reader, Writer,
        base64::{self, Alphabet},
    };
    let text = std::str::from_utf8(link).expect("fixture URL");
    let secret = base64::decode(Alphabet::Url, text.split_once("secret=").expect("link secret").1.as_bytes(), 32)
        .expect("fixture encoding");
    let limits = jig_protocol_config::profile::profile(jig_protocol_config::profile::Profile::Small).clients.wire;
    let record = wire::LinkSignIn::new(&limits, wire::LinkSignInParts { secret: skein_lib::Secret::new(secret) })
        .expect("link record");
    let mut out = Writer::new(usize::try_from(record.measure()).expect("record length"));
    record.encode(&mut out).expect("measured record");
    let body = out.finish();
    let address = box_link::address(link);
    let mut socket = Socket::connect("spent link", support::WITHIN, address, Scope::Owned)?;
    let header = format!(
        "POST /jig/v1/sign-in/link HTTP/1.0\r\nHost: {address}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    socket.write_all("spent link request", support::WITHIN, header.as_bytes())?;
    socket.write_all("spent link body", support::WITHIN, &body)?;
    let answer = socket.read_to_end("spent link answer", support::WITHIN, 4096)?;
    let start = skein_lib::bytes::find(&answer, b"\r\n\r\n").expect("HTTP answer") + 4;
    let mut reader = Reader::new(&answer[start..]);
    Ok(matches!(wire::SignInAnswer::decode(&limits, &mut reader), Ok(wire::SignInAnswer::LinkUsed))
        && reader.remaining() == 0)
}

#[test]
fn a_fresh_box_holds_serves_clients_port() {
    let root = scratch("first");
    let mut child = command(&root, "").stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("box");
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
    let local = kept_link(&mut boxed);
    assert!(local != link, "the file's link differs from the first printed link");
    assert!(box_link::signs_in(boxed.root(), "first-client", &link), "the block's very first link signs in as owner");
    match used(&link) {
        Ok(used) => assert!(used, "the spent link is answered used"),
        Err(why) => {
            boxed.stop();
            crate::support::story_socket::finish(why);
        }
    }
    let fresh = boxed.link();
    assert!(fresh != link, "a spent link draws a fresh one");
    let path = box_link::link_path(boxed.root());
    assert!(local != fresh, "the file's link differs from the fresh printed link");
    assert!(
        fs::read(&path).expect("kept local link") == [local.as_slice(), b"\n"].concat(),
        "the file's link stays unchanged"
    );
    let (stdout, stderr, _) = boxed.stop();
    assert!(stdout == [link.as_slice(), b"\n", fresh.as_slice(), b"\n"].concat(), "exactly the first and fresh links");
    assert!(
        stderr == [box_link::PRELUDE, box_link::footer(&link).as_slice(), b"\n"].concat(),
        "fresh link has only its separator"
    );
}

fn kept_link(boxed: &mut box_link::Boxed) -> Vec<u8> {
    let path = box_link::link_path(boxed.root());
    boxed.wait(|_| fs::read(&path).is_ok_and(|bytes| bytes.ends_with(b"\n")));
    let bytes = fs::read(&path).expect("the file's link");
    let link = bytes.strip_suffix(b"\n").expect("one final line feed");
    assert!(!link.is_empty() && !link.contains(&b'\n'), "the file holds one link and a line feed");
    link.to_vec()
}

mod lifetime;
