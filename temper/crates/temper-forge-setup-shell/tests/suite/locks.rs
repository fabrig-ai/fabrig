use crate::support;
use std::fs;
use temper_forge_setup_shell::process;
#[test]
fn a_child_holds_the_lock_as_its_stdin_after_the_parent_lets_go() {
    let scratch = support::Scratch::new("lock_child");
    let path = scratch.path().join("lock");
    let spec = support::script(scratch.path(), "exec sleep 60");
    let lock = process::lock(&path).expect("first lock");
    let mut child = process::command(&spec, lock).spawn().expect("child");
    #[cfg(target_os = "linux")]
    assert_eq!(fs::read_link(format!("/proc/{}/fd/0", child.id())).expect("stdin"), path);
    #[cfg(not(target_os = "linux"))]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!(stdin_inode(child.id()), fs::metadata(&path).expect("lock metadata").ino());
    }
    assert!(matches!(process::lock(&path), Err(process::Refused::Held)));
    child.kill().and_then(|()| child.wait()).expect("child reaped");
    process::lock(&path).expect("released lock");
    scratch.remove();
}

#[cfg(not(target_os = "linux"))]
fn stdin_inode(pid: u32) -> u64 {
    let output = std::process::Command::new("/usr/sbin/lsof")
        .args(["-a", "-p", &pid.to_string(), "-d", "0", "-F", "i"])
        .output()
        .expect("child stdin metadata");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout)
        .expect("lsof fields")
        .lines()
        .find_map(|line| line.strip_prefix('i'))
        .expect("stdin inode field")
        .parse()
        .expect("stdin inode")
}
