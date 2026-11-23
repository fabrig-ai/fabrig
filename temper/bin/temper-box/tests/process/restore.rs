//! Restore a stopped deployment and undo it with its safety checkpoint (protocol/checkpoints.md, 5).
use crate::support;
use skein_io::kernel::Exit;
use std::{path::Path, process::Command};
use support::ctl::{Call, Step, Story, json};

fn stopped(root: &Path, verb: &str, name: Option<&str>) -> String {
    let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
    command.arg(verb).args(name).args(support::configuration::flags(root)).current_dir(root).env_clear();
    let output = support::story_command::standalone(&mut command, "stopped restore command", support::WITHIN);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).expect("command line")
}

fn projects(root: &Path, sign_in: bool, make: Option<&str>) -> serde_json::Value {
    let mut steps = Vec::new();
    if sign_in {
        steps.push(Step::Run(Call::sign_in()));
    }
    if let Some(name) = make {
        steps.push(Step::Run(Call::new(&["project", "make", name, "--key", "after-checkpoint"])));
    }
    steps.push(Step::Run(Call::new(&["me", "--json"])));
    let seen = support::ctl::run(Story::new(root, steps));
    assert_eq!(seen.serve.exit, Some(Exit::Code(0)));
    for call in &seen.calls {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
    }
    json(seen.calls.last().expect("me after any commit"))["projects"].clone()
}

#[test]
fn a_checkpoint_restores_earlier_state_and_its_safety_restores_the_later_state() {
    let scratch = support::Scratch::new("stopped-restore");
    let root = scratch.path();
    support::lay_configuration(root, &support::Forge::Unreached);
    // Keep the signed-in directory's address usable across each restart.
    let [clients] = support::free_addresses::<1>(&[]);
    let path = root.join(temper_config_shell::FILE);
    let mut file = support::read_file(&path, temper_config_shell::Shape::Standalone);
    file.listen = Some(clients);
    support::write_file(&path, &file, temper_config_shell::Shape::Standalone);
    let before = projects(root, true, None);
    let taken = stopped(root, "checkpoint", None);
    let name = taken
        .strip_prefix("temper-box checkpoint: ")
        .and_then(|line| line.split_once(" at commit "))
        .expect("checkpoint name")
        .0;
    let later = projects(root, false, Some("After checkpoint"));
    assert_ne!(before, later, "a committed project distinguishes the states");
    let restored = stopped(root, "restore", Some(name));
    assert!(restored.starts_with(&format!("temper-box restore: restored {name} as generation ")), "{restored}");
    let safety = restored
        .split_once("; the state before it is checkpoint ")
        .and_then(|(_, line)| line.split_once(", and restore "))
        .expect("safety checkpoint")
        .0;
    assert!(jig_protocol_store::CheckpointName::parse(safety.as_bytes()).is_some());
    assert!(restored.ends_with(&format!(", and restore {safety} brings it back\n")));
    assert_eq!(projects(root, false, None), before, "a start finds the checkpoint's state");
    let undone = stopped(root, "restore", Some(safety));
    assert!(undone.starts_with(&format!("temper-box restore: restored {safety} as generation ")), "{undone}");
    assert_eq!(projects(root, false, None), later, "a start finds the safety checkpoint's later state");
    scratch.remove();
}
