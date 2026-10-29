//! Sender attribution through the shell and io on the Linux kernel.
#![cfg(target_os = "linux")]

use super::given_listener::deadlines::Patience;
use skein_world::Expired;

fn service(sender: &str) {
    for backend in ["ring", "readiness"] {
        let output = std::process::Command::new(skein_scratch::paths::binary("signal_sender_fixture"))
            .args([backend, sender])
            .output()
            .expect("the single-threaded signal fixture starts");
        assert!(
            output.status.success(),
            "{backend} signal from {sender}: {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        forward_starved(&output.stderr);
    }
}

fn forward_starved(stderr: &[u8]) {
    if std::env::var_os("NEXTEST").is_some()
        && let Some(line) =
            String::from_utf8_lossy(stderr).lines().find(|line| line.starts_with("skein-world: starved:"))
    {
        eprintln!("{line}");
        std::process::exit(0);
    }
}

#[test]
fn a_spawned_childs_sigterm_names_its_io_token() {
    service("child");
}

#[test]
fn a_self_sigterm_has_no_child_token() {
    service("self");
}

// Recording senders never signal their selected target; teardown ends only the owned sender.
fn recorded_orphan(selection: &str) -> String {
    use skein_io::kernel::Exit;
    use std::io::{BufRead, Write};
    use std::process::{Command, Stdio};

    skein_shell::make_subreaper().expect("adopt recording sender");
    let scratch = skein_scratch::Scratch::new("recorded-orphan");
    let record = scratch.path().join("target");
    let mut parent = Command::new(skein_scratch::paths::binary("signal_sender_fixture"))
        .arg("delayed-parent")
        .arg(&record)
        .arg(selection)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start recording sender's parent");
    let mut line = String::new();
    std::io::BufReader::new(parent.stdout.take().expect("parent output")).read_line(&mut line).expect("sender PID");
    let pid: u32 = line.trim().parse().expect("sender PID");
    let sender = skein_shell::open_pidfd(pid).expect("keep recording sender");
    let mut barrier = parent.stdin.take().expect("sender barrier stays open across parent wait");
    assert!(parent.wait().expect("reap intended target before sender runs").success());
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).expect("sender identity");
    let adopter: u32 = stat
        .rsplit_once(") ")
        .expect("proc stat")
        .1
        .split_whitespace()
        .nth(1)
        .expect("parent field")
        .parse()
        .expect("parent PID");
    assert_eq!(adopter, std::process::id(), "prove adoption before releasing the recording sender");
    let mut deadline = Patience::new("the recording sender exiting", skein_lib::Duration::from_secs(3));
    deadline.watch_process(pid);
    barrier.write_all(b"go\n").expect("release recording sender");
    let exit = loop {
        if let Some(exit) = skein_shell::poll_child(sender, true).expect("wait recording sender") {
            break exit;
        }
        if deadline.passed() {
            let expired = deadline.expired();
            drop(barrier);
            let _killed =
                skein_shell::signal_kept_child(sender, skein_io::kernel::Signal::Kill, skein_io::kernel::Target::Child);
            skein_shell::abandon_binary(Some(sender), &[sender]);
            drop(parent);
            drop(scratch);
            match expired {
                Expired::Starved(starved) => starved.pass(),
                Expired::Failed(figures) => panic!("recording sender must settle promptly: {figures}"),
            }
        }
        std::thread::sleep(std::time::Duration::from_nanos(
            deadline.remaining().min(skein_lib::Duration::from_millis(1)).as_nanos(),
        ));
    };
    skein_shell::close_keeper_fd(sender);
    assert_eq!(exit, Exit::Code(0), "recording sender exits without a signal");
    std::fs::read_to_string(record).expect("recorded selection")
}

#[test]
fn an_orphan_sender_records_its_adopter_without_sending_a_signal() {
    assert_eq!(recorded_orphan("legacy"), std::process::id().to_string());
}

#[test]
fn a_sender_released_after_its_parent_is_reaped_refuses_the_adopter() {
    assert_eq!(recorded_orphan("checked"), "refused");
}
