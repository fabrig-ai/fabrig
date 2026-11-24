//! A stopped checkpoint held, restored and removed (protocol/checkpoints.md, 5).
use crate::support;
use skein_io::kernel::Exit;
use std::{
    path::Path,
    process::{Command, Output},
};
use support::ctl::{Call, Step, Story, json};

fn stopped(root: &Path, verb: &str, arguments: &[&str]) -> Output {
    let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
    command
        .arg(verb)
        .args(support::configuration::flags(root))
        .args(arguments)
        .current_dir(root)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("KEPT", "checkpoint-child")
        .env("TEMPER_CHECKPOINT", "must-be-replaced");
    support::story_command::standalone(&mut command, "stopped checkpoint or restore command", support::WITHIN)
}

fn projects(root: &Path, mut steps: Vec<Step>) -> serde_json::Value {
    steps.push(Step::Run(Call::new(&["me", "--json"])));
    let seen = support::ctl::run(Story::new(root, steps));
    assert_eq!(seen.serve.exit, Some(Exit::Code(0)));
    for call in &seen.calls {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
    }
    json(seen.calls.last().expect("me after any commit"))["projects"].clone()
}

#[test]
fn a_stopped_checkpoint_is_held_restored_and_removed() {
    let scratch = support::Scratch::new("stopped-checkpoint");
    let root = scratch.path();
    support::configuration(root, &support::Forge::Unreached);
    // Keep the signed-in directory's address usable across each restart.
    let [clients] = support::free_addresses::<1>(&[]);
    let path = root.join(temper_config_shell::FILE);
    let mut file = support::read_file(&path, temper_config_shell::Shape::Standalone);
    file.listen = Some(clients);
    support::write_file(&path, &file, temper_config_shell::Shape::Standalone);
    let before = projects(root, vec![Step::Run(Call::sign_in())]);
    let taken = stopped(root, "checkpoint", &[]);
    assert!(taken.status.success(), "{}", String::from_utf8_lossy(&taken.stderr));
    let line = String::from_utf8(taken.stdout).expect("checkpoint line");
    let (name, cut) = line
        .strip_prefix("temper-box checkpoint: ")
        .expect("box checkpoint prefix")
        .split_once(" at commit ")
        .expect("checkpoint cut");
    assert!(jig_protocol_store::CheckpointName::parse(name.as_bytes()).is_some());
    assert!(cut.split_once(',').expect("commit cut and secrets").0.parse::<u64>().is_ok(), "{line}");
    let directory = root.join("state/checkpoints").join(name);
    assert!(directory.join("checkpoint").is_file());
    let held = stopped(
        root,
        "checkpoint",
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

    let later = projects(
        root,
        vec![Step::Run(Call::new(&["project", "make", "After checkpoint", "--key", "after-checkpoint"]))],
    );
    assert_ne!(before, later, "a committed project distinguishes the states");
    let restored = stopped(root, "restore", &[name]);
    assert!(restored.status.success(), "{}", String::from_utf8_lossy(&restored.stderr));
    let restored = String::from_utf8(restored.stdout).expect("restore line");
    let (generation, _) = restored
        .strip_prefix(&format!("temper-box restore: restored {name} as generation "))
        .and_then(|line| line.split_once(": "))
        .expect("source and generation");
    assert!(generation.parse::<u64>().is_ok(), "{restored}");
    let safety = restored
        .split_once("; the state before it is checkpoint ")
        .and_then(|(_, line)| line.split_once(", and restore "))
        .expect("safety checkpoint")
        .0;
    assert!(jig_protocol_store::CheckpointName::parse(safety.as_bytes()).is_some());
    assert!(restored.ends_with(&format!(", and restore {safety} brings it back\n")));
    assert_eq!(projects(root, vec![]), before, "a start finds the checkpoint's state");

    let removed = stopped(root, "checkpoint", &["--remove", name]);
    assert!(removed.status.success(), "{}", String::from_utf8_lossy(&removed.stderr));
    assert_eq!(String::from_utf8_lossy(&removed.stdout), format!("temper-box checkpoint: removed {name}\n"));
    assert!(!directory.exists());
    scratch.remove();
}
