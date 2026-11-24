use hearth::Form;
use hearth_builder_world::{
    fixtures::{self, File, SIMULATION_SEED, Scenario, diffing, edited},
    protocol,
};

const WITH_VIM: File = File {
    path: "tasks/base/task.json",
    body:
        br#"{"packages": {"curl": {}, "tzdata": {}, "vim": {}, "firmware-linux": {"component": "non-free-firmware"}}}"#,
    mode: 0o644,
};
const WITHOUT_QUIET: File = File {
    path: "tasks/params/task.json",
    body:
        br#"{"cmdline": {"console": {"param": "console=ttyS0,115200n8", "order": 900}, "audit": {"param": "audit=0"}}}"#,
    mode: 0o644,
};

fn diff_json(files: &[File], change: File) -> protocol::Run {
    let later = edited(files, &[change], &[]);
    protocol::run(&Scenario { form: Form::JsonLines, ..diffing(files, &later) }, SIMULATION_SEED)
}

#[test]
fn an_added_package_rebuilds_from_packages_and_exits_1() {
    let run = diff_json(&fixtures::packages_files(true), WITH_VIM);
    let stdout = std::str::from_utf8(&run.stdout).expect("JSON lines");
    assert!(stdout.contains(r#""added":["vim"],"removed":[]"#), "{stdout}");
    assert!(stdout.contains(r#""kind":"from_packages""#), "{stdout}");
    assert_eq!(run.code, Some(1));
}

#[test]
fn a_changed_command_line_needs_only_a_new_slot() {
    let run = diff_json(&fixtures::thin_with(fixtures::CMDLINE), WITHOUT_QUIET);
    let stdout = std::str::from_utf8(&run.stdout).expect("JSON lines");
    assert!(stdout.contains(r#""kind":"slot_only""#), "{stdout}");
    assert_eq!(run.code, Some(1));
}
