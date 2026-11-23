use hearth::{Call, Form};
use hearth_builder_world::{fixtures, protocol};

fn diff_motd_bodies(old: &'static [u8], new: &'static [u8], turns: u32) -> protocol::Run {
    let files = fixtures::edited(fixtures::MOTD, &[fixtures::File { body: old, ..fixtures::MOTD_KIND }], &[]);
    let later = fixtures::edited(fixtures::MOTD, &[fixtures::File { body: new, ..fixtures::MOTD_KIND }], &[]);
    let mut scenario = fixtures::Scenario { form: Form::JsonLines, ..fixtures::diffing(&files, &later) };
    let Call::Diff { targets, .. } = &mut scenario.call else { unreachable!("diffing supplies a diff call") };
    *targets = Box::new([b"x1-prod".as_slice().into()]);
    protocol::with_turns(&scenario, fixtures::SIMULATION_SEED, turns)
}

/// Each side has 300 differing lines in a single hunk.
const MOTD_LINES_PER_SIDE: usize = 300;
/// The hunk header uses one of the 200 retained lines; removals come first.
const REMOVALS_RETAINED: usize = 199;
/// Reading 600 diff lines and printing 200 takes about 20,300 turns at this seed.
const LONG_HUNK_TURNS: u32 = 24_000;

#[test]
fn json_counts_the_hunk_lines_left_out_past_200() {
    let run = diff_motd_bodies(
        b"old\n".repeat(MOTD_LINES_PER_SIDE).leak(),
        b"new\n".repeat(MOTD_LINES_PER_SIDE).leak(),
        LONG_HUNK_TURNS,
    );
    let stdout = std::str::from_utf8(&run.stdout).expect("JSON");
    assert_eq!(stdout.matches("\"kind\":\"removed\"").count(), REMOVALS_RETAINED);
    // The rest of the 300 removals and all 300 additions are left out.
    assert!(stdout.contains("\"hunks\":{\"binary\":false,\"left_out\":401,\"lines\":["));
}

#[test]
fn a_binary_file_answers_binary_with_no_hunk_line() {
    let run = diff_motd_bodies(b"old\0\n", b"new\0\n", fixtures::PROTOCOL_TURNS);
    let stdout = std::str::from_utf8(&run.stdout).expect("JSON");
    assert!(stdout.contains("\"hunks\":{\"binary\":true,\"left_out\":0,\"lines\":[]}"));
}

/// One change record for each target in the MOTD fixture.
const MOTD_TARGETS: usize = 4;

#[test]
fn the_diffs_errors_follow_its_changes_and_precede_the_outcome() {
    let later = fixtures::edited(fixtures::MOTD, &[fixtures::MOTD_KIND, fixtures::BAD_WEST_LAB], &[]);
    let scenario = fixtures::Scenario { form: Form::JsonLines, ..fixtures::diffing(fixtures::MOTD, &later) };
    let run = protocol::run(&scenario, fixtures::SIMULATION_SEED);
    let stdout = std::str::from_utf8(&run.stdout).expect("JSON text");
    let mut lines = stdout.lines();
    assert_eq!(lines.next_back().and_then(|line| line.split(',').next()), Some("{\"record\":\"outcome\""));
    assert_eq!(
        lines.next_back(),
        Some(
            "{\"record\":\"error\",\"side\":\"to\",\"kind\":\"decode\",\"decode\":\"unexpected\",\"class\":\"configuration\",\"at\":{\"column\":2,\"line\":1,\"path\":\"hosts/west/west-lab.json\"}}"
        )
    );
    assert_eq!(lines.filter(|line| line.starts_with("{\"record\":\"change\"")).count(), MOTD_TARGETS);
}
