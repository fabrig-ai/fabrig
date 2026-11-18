//! A stopped checkpoint, a real held child, and removal (protocol/checkpoints.md, 5).
use crate::support;
use std::{
    path::Path,
    process::{Command, Output},
};
use support::ctl::{Call, Step, Story};

fn checkpoint(root: &Path, arguments: &[&str]) -> Output {
    let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
    command
        .arg("checkpoint")
        .args(support::configuration::flags(root))
        .args(arguments)
        .current_dir(root)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("KEPT", "checkpoint-child")
        .env("TEMPER_CHECKPOINT", "must-be-replaced");
    support::story_command::standalone(&mut command, "stopped checkpoint command", support::WITHIN)
}

#[test]
fn a_stopped_checkpoint_is_listed_under_its_hold_then_removed() {
    let scratch = support::Scratch::new("stopped-checkpoint");
    let root = scratch.path();
    support::configuration(root, &support::Forge::Unreached);
    let served =
        support::ctl::run(Story::new(root, vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"]))]));
    assert_eq!(served.serve.exit, Some(skein_io::kernel::Exit::Code(0)));
    let taken = checkpoint(root, &[]);
    assert!(taken.status.success(), "{}", String::from_utf8_lossy(&taken.stderr));
    let line = String::from_utf8(taken.stdout).expect("checkpoint line");
    let (name, _) = line
        .strip_prefix("temper-box checkpoint: ")
        .expect("box checkpoint prefix")
        .split_once(" at commit ")
        .expect("checkpoint cut");
    let directory = root.join("state/checkpoints").join(name);
    assert!(directory.join("checkpoint").is_file());
    let held = checkpoint(
        root,
        &[
            "--hold",
            name,
            "--",
            "/bin/sh",
            "-c",
            "test \"$PWD\" = \"$1\" && test \"$KEPT\" = checkpoint-child && test \"$TEMPER_CHECKPOINT\" = \"$2\" && test -f \"$TEMPER_CHECKPOINT/checkpoint\" && ls \"$TEMPER_CHECKPOINT\" || exit 9; exit 3",
            "checkpoint-child",
            root.to_str().expect("scratch path"),
            directory.to_str().expect("checkpoint path"),
        ],
    );
    assert_eq!(held.status.code(), Some(3), "{}", String::from_utf8_lossy(&held.stderr));
    assert!(String::from_utf8_lossy(&held.stdout).lines().any(|line| line == "checkpoint"));
    let removed = checkpoint(root, &["--remove", name]);
    assert!(removed.status.success(), "{}", String::from_utf8_lossy(&removed.stderr));
    assert_eq!(String::from_utf8_lossy(&removed.stdout), format!("temper-box checkpoint: removed {name}\n"));
    assert!(!directory.exists());
    scratch.remove();
}
