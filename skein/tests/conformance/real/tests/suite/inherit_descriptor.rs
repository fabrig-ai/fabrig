//! The world's standard descriptors are its own, not the test runner's.

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn inherited_standard_descriptors_survive_exec_in_the_services_process() {
    let mut service = Command::new(skein_scratch::paths::binary("inherit_fixture"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the service starts in a process of its own");
    service.stdin.take().expect("service stdin").write_all(b"hi\n").expect("feed the inherited stdin");
    let output = service.wait_with_output().expect("reap the service and its child");
    assert!(output.status.success(), "service failed: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"hi", "the child reads service stdin and writes service stdout");
    assert_eq!(output.stderr, b"error", "the child writes service stderr");
}
