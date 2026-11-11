//! The box and its running steps end together (temper/docs/design/protocol/setup/testing.md, 1).
use crate::box_link;
use crate::support::{boxed, parent};
use std::{fs, process::Stdio};

fn listening(boxed: &mut box_link::Boxed) {
    let link = boxed.link();
    boxed.wait(|boxed| boxed::serve_listening(&boxed.root().join("data")));
    let mut sign_in = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("ctl")
        .arg(boxed.root().join("client"))
        .args(["sign-in", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sign in owner");
    {
        use std::io::Write;
        sign_in.stdin.take().expect("sign-in stdin").write_all(&link).expect("private link on stdin");
    }
    let signed_in = sign_in.wait_with_output().expect("sign-in outcome");
    assert!(signed_in.status.success(), "{}", String::from_utf8_lossy(&signed_in.stderr));
    let setup = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("ctl")
        .arg(boxed.root().join("client"))
        .args(["forge", "setup", "--playground", "--key", "lifetime"])
        .stdin(Stdio::null())
        .output()
        .expect("start playground steps");
    assert!(setup.status.success(), "{}", String::from_utf8_lossy(&setup.stderr));
}
fn exited(boxed: &mut box_link::Boxed) -> std::process::ExitStatus {
    let mut status = None;
    boxed.wait(|boxed| {
        status = boxed.guard.child.as_mut().expect("box child").try_wait().expect("box status");
        status.is_some()
    });
    status.expect("box exited")
}

#[test]
fn a_hangup_stops_a_box_and_leaves_nothing_running() {
    let mut boxed = box_link::Boxed::start("hangup", None);
    listening(&mut boxed);
    #[cfg(target_os = "linux")]
    let observed = parent::Guard::observing_test();
    parent::signal(boxed.guard.child.as_mut().expect("box child"), "HUP");
    assert!(exited(&mut boxed).success(), "a hangup drains the box successfully");
    #[cfg(target_os = "linux")]
    assert!(observed.all_exited(), "nothing outlives the box's hangup");
    let log = boxed::box_log(&boxed.root().join("data"));
    assert!(log.lines().last().expect("last log line").ends_with("Stopped"), "the box logs its settled stop");

    // The same retained configuration must bind again, including its owned listener.
    fs::remove_file(boxed.root().join("data/box/logs/serve.log")).expect("remove the old readiness marker");
    let child = box_link::command(boxed.root(), "")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("restart box");
    boxed.guard.child = Some(child);
    boxed.wait(|boxed| boxed::serve_listening(&boxed.root().join("data")));
    boxed.stop();
}

// Parent-death signalling and retained pidfd observations are Linux's.
#[cfg(target_os = "linux")]
#[test]
fn a_killed_box_takes_its_steps_with_it() {
    let mut boxed = box_link::Boxed::start("killed", None);
    listening(&mut boxed);
    let pid = boxed.guard.child.as_ref().expect("box child").id();
    // The fixture owns cleanup even if the no-survivor assertion fails.
    boxed.guard.record(pid, std::process::id());
    boxed.guard.descendants(pid);
    let mut observed = parent::Guard::observing_test();
    assert!(observed.has_descendants(), "the observation includes a running step");
    parent::signal(boxed.guard.child.as_mut().expect("box child"), "KILL");
    assert!(!exited(&mut boxed).success(), "the box was killed");
    observed.wait_all();
    assert!(observed.all_exited(), "every retained step exits after its box dies");
    boxed.scratch.remove();
}
