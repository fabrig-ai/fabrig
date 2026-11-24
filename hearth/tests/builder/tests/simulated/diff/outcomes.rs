//! Diff output and exit codes (hearth/docs/design/client/README.md, 3.2 and 3.3).
use hearth::{Call, Form};
use hearth_builder_world::{
    fixtures::{self, File, MOTD, MOTD_KIND, SIMULATION_SEED, Scenario, diffing, edited},
    protocol,
};

/// One change record for each target in the MOTD fixture.
const MOTD_TARGETS: usize = 4;
const CHANGE_RECORD: &str = r#"{"record":"change""#;

fn diff_motd(changes: &[File], form: Form) -> protocol::Run {
    let later = edited(MOTD, changes, &[]);
    protocol::run(&Scenario { form, ..diffing(MOTD, &later) }, SIMULATION_SEED)
}

#[test]
fn hearth_diff_names_each_changed_target_and_its_rebuild() {
    let run = diff_motd(&[MOTD_KIND], Form::Text);
    assert_eq!(run.stdout, fixtures::MOTD_DIFF_TEXT.as_bytes());
    assert_eq!(run.code, Some(1));
}

fn diff_motd_with_itself(named: Box<[Box<[u8]>]>) -> protocol::Run {
    let mut scenario = diffing(MOTD, MOTD);
    let Call::Diff { from, to, targets, .. } = &mut scenario.call else { unreachable!("diffing supplies a diff call") };
    from.clone_from(to);
    *targets = named;
    protocol::run(&scenario, SIMULATION_SEED)
}

#[test]
fn a_diff_of_a_commit_with_itself_exits_0() {
    let run = diff_motd_with_itself(Box::default());
    assert_eq!(run.code, Some(0), "same spec digests");
}

#[test]
fn a_named_target_neither_revision_has_exits_64() {
    let run = diff_motd_with_itself(Box::new([b"unknown".as_slice().into()]));
    assert_eq!(run.code, Some(64), "target absent on both sides");
}

#[test]
fn a_document_fixed_in_the_later_revision_fails_its_target_from() {
    let files = edited(MOTD, &[fixtures::BAD_X1_LAB], &[]);
    let later = edited(&files, &[fixtures::FIXED_X1_LAB], &[]);
    let mut scenario = Scenario { form: Form::JsonLines, ..diffing(&files, &later) };
    let Call::Diff { targets, .. } = &mut scenario.call else { unreachable!("diffing supplies a diff call") };
    *targets = Box::new([b"x1-lab".as_slice().into()]);
    let run = protocol::run(&scenario, SIMULATION_SEED);
    let stdout = std::str::from_utf8(&run.stdout).expect("JSON");
    assert_eq!(
        stdout.lines().find(|line| line.starts_with(CHANGE_RECORD)),
        Some(r#"{"record":"change","target":"x1-lab","kind":"failed","failed":{"side":"from"}}"#),
        "the failed earlier document"
    );
}

#[test]
fn a_failure_beats_every_difference_and_exits_2() {
    let run = diff_motd(&[MOTD_KIND, fixtures::BAD_WEST_LAB], Form::Text);
    assert_eq!(run.code, Some(2), "configuration failure outranks differences");
}

#[test]
fn a_later_failure_names_its_side_before_the_outcome() {
    let run = diff_motd(&[MOTD_KIND, fixtures::BAD_WEST_LAB], Form::JsonLines);
    let stdout = std::str::from_utf8(&run.stdout).expect("JSON");
    assert_eq!(
        stdout.lines().find(|line| line.starts_with(CHANGE_RECORD)),
        Some(r#"{"record":"change","target":"west-lab","kind":"failed","failed":{"side":"to"}}"#),
        "the later document failed"
    );
    let mut lines = stdout.lines();
    assert_eq!(lines.next_back().and_then(|line| line.split(',').next()), Some(r#"{"record":"outcome""#));
    assert_eq!(
        lines.next_back(),
        Some(
            r#"{"record":"error","side":"to","kind":"decode","decode":"unexpected","class":"configuration","at":{"column":2,"line":1,"path":"hosts/west/west-lab.json"}}"#
        )
    );
    assert_eq!(lines.filter(|line| line.starts_with(CHANGE_RECORD)).count(), MOTD_TARGETS);
}

#[test]
fn json_lines_give_each_targets_change_then_the_tally_in_the_outcome() {
    let run = diff_motd(&[MOTD_KIND], Form::JsonLines);
    let stdout = std::str::from_utf8(&run.stdout).expect("JSON");
    // The fake repository's `before` and `HEAD` commits take the place of real Git's commit IDs.
    assert_eq!(
        stdout.lines().last(),
        Some(concat!(
            r#"{"record":"outcome","outcome":"negative","#,
            r#""from":"381e59dd694f55f40872e25b45f6f8331dc2743a","to":"0bc48aa50a54e412512ce3c1a6b1017ed119e340","#,
            r#""targets":4,"changed":2,"unchanged":2,"left_out":0}"#,
        )),
    );
    assert_eq!(
        stdout.lines().filter(|line| line.starts_with(CHANGE_RECORD)).count(),
        MOTD_TARGETS,
        "one change per target including same"
    );
}
