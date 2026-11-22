use std::{fs, os::unix::fs::PermissionsExt};
use temper_worker_shell::{testing::launch, worker};

#[test]
fn pin_replaces_the_previous_agent_with_private_byte_copies() {
    let scratch = skein_scratch::Scratch::new("worker-pin");
    let root = scratch.path().join("worker");
    fs::create_dir_all(root.join("agent")).expect("pin fixture");
    fs::write(root.join("agent/old"), b"stale").expect("pin fixture");
    let program = scratch.path().join("temper-agent");
    let configuration = scratch.path().join("agent.json");
    fs::write(&program, b"a").expect("pin fixture");
    let pinned = worker::pin(&root, &launch(&program, &configuration, &[]), b"checked").expect("pin fixture");
    let mut names: Vec<_> = fs::read_dir(root.join("agent"))
        .expect("pin fixture")
        .map(|entry| entry.expect("pin fixture").file_name())
        .collect();
    names.sort();
    assert_eq!(names, ["agent.json", "temper-agent"]);
    for (path, bytes, mode) in [
        (crate::path(&pinned.program), b"a".as_slice(), 0o700),
        (crate::path(&pinned.configuration), b"checked".as_slice(), 0o600),
    ] {
        assert_eq!(fs::read(path).expect("pin fixture"), bytes);
        assert_eq!(fs::metadata(path).expect("pin fixture").permissions().mode() & 0o777, mode);
    }
    assert_eq!(fs::metadata(root.join("agent")).expect("pin fixture").permissions().mode() & 0o777, 0o700);
}
