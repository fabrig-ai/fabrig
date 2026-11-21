//! Repair admission and words need no engine root (domain/red-repairs.md, 4.2–4.4).
use super::*;
use crate::settings::{RepairGoal, Repairs};

fn repairs() -> Repairs {
    Repairs {
        goals: 2,
        attempts: 2,
        deadline_ms: 86_400_000,
        goal: RepairGoal { charter: 1, llm: None, priority: 7, budget: 20 },
    }
}
fn row(number: u32) -> Finding {
    let mut row = blamed_red(number, FindingState::New);
    row.signature = Some(FindingSignature {
        source: b"tier-a".as_slice().into(),
        ids: Box::new([format!("test {number}").into_bytes().into()]),
        ..signature()
    });
    row
}
fn fixture(rows: &[Finding]) -> Domain {
    let mut d = domain();
    d.take_project_repairs(1, Ok(repairs()));
    for row in rows {
        d.commit_finding(&LIMITS, FindingWrite { row: row.clone(), erase: None }, &mut Queue::uncapped());
    }
    d
}
fn act(more: &[u32]) -> RepairAct {
    RepairAct { more: more.into(), words: b"Fix these together.".as_slice().into() }
}
fn refusal(d: &Domain, more: &[u32]) -> Box<[u8]> {
    d.prepare_repair_findings(1, 1, &act(more)).expect_err("repair refused").words(1, 1, b"")
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary test assertions collect and compare the prepared rows and exact words"
)]
fn repairs_settings_refuse_first_and_admission_returns_unchanged_oldest_findings() {
    let mut older = row(2);
    older.at = Wall::EPOCH;
    let d = fixture(&[row(1), older.clone()]);
    let writes = d.prepare_repair_findings(1, 1, &act(&[2])).expect("repair fixture");
    assert_eq!(writes.iter().map(|write| &write.row).collect::<Vec<_>>(), [&older, &row(1)]);
    assert_eq!(d.finding(1, 1), Some(&row(1)), "preparation writes nothing");
    let mut d = domain();
    assert_eq!(refusal(&d, &[99]).as_ref(), b"repairs are off in project 1");
    for field in [b"goal.charter".as_slice(), b"goal.llm", b"tasks.spec_bytes"] {
        d.take_project_repairs(1, Err(field));
        assert_eq!(refusal(&d, &[99]).as_ref(), [b"repairs are off in project 1: ", field].concat());
    }
}

#[test]
#[expect(clippy::disallowed_types, reason = "ordinary test cases collect independent invalid finding pairs")]
fn repair_refusals_check_kind_state_repository_attempts_then_distinct_live_goal_cap() {
    let first = row(1);
    let second = row(2);
    let mut cases: Vec<(Finding, Finding, &str)> = vec![];
    for kind in [FindingKind::Qa, FindingKind::Flake, FindingKind::Review, FindingKind::Feedback] {
        cases.push((
            Finding { kind, ..first.clone() },
            second.clone(),
            "finding 1/1 is not an uncleared red filed under landed",
        ));
    }
    cases.push((
        Finding { blame: None, ..first.clone() },
        second.clone(),
        "finding 1/1 is not an uncleared red filed under landed",
    ));
    cases.push((
        Finding { cleared: Some(FindingCleared { run: 1, at: Wall::EPOCH }), ..first.clone() },
        second.clone(),
        "finding 1/1 is not an uncleared red filed under landed",
    ));
    for (state, expected) in [
        (FindingState::Routed, "finding 1/1 is routed, not new"),
        (FindingState::Backlog, "finding 1/1 is backlog, not new"),
        (FindingState::Dropped, "finding 1/1 is dropped, not new"),
    ] {
        cases.push((Finding { state, ..first.clone() }, second.clone(), expected));
    }
    let mut other = second.clone();
    other.signature.as_mut().expect("repair fixture").repository.repository += 1;
    cases.push((first.clone(), other.clone(), "findings 1/1 and 1/2 are in two repositories"));
    let repair = FindingRepair {
        goal: 30,
        made: true,
        attempts: 2,
        cancelled: false,
        failing: Box::new([]),
        until: None,
        judging: None,
    };
    cases.push((
        Finding { repair: Some(repair.clone()), ..first.clone() },
        second.clone(),
        "ask before repairing finding 1/1 again",
    ));
    cases.push((
        Finding { repair: Some(FindingRepair { attempts: 1, cancelled: true, ..repair.clone() }), ..first.clone() },
        second.clone(),
        "ask before repairing finding 1/1 again",
    ));
    // Kind beats state, state beats repository, and repository beats attempts, across the whole act.
    cases.push((
        Finding { state: FindingState::Dropped, ..first.clone() },
        Finding { kind: FindingKind::Qa, ..second.clone() },
        "finding 1/2 is not an uncleared red filed under landed",
    ));
    cases.push((
        first.clone(),
        Finding { state: FindingState::Dropped, ..other.clone() },
        "finding 1/2 is dropped, not new",
    ));
    cases.push((
        Finding { repair: Some(repair), ..first.clone() },
        other,
        "findings 1/1 and 1/2 are in two repositories",
    ));
    for (first, second, expected) in cases {
        let d = fixture(&[first.clone(), second.clone()]);
        assert_eq!(refusal(&d, &[2]).as_ref(), expected.as_bytes());
        assert_eq!((d.finding(1, 1), d.finding(1, 2)), (Some(&first), Some(&second)), "refusal writes nothing");
    }
    let live = |goal| FindingRepair {
        goal,
        made: true,
        attempts: 1,
        cancelled: false,
        failing: Box::new([]),
        until: Some(Wall::from_nanos(1)),
        judging: None,
    };
    let d = fixture(&[
        first.clone(),
        second.clone(),
        Finding { repair: Some(live(30)), ..row(3) },
        Finding { repair: Some(live(30)), ..row(4) },
    ]);
    assert!(
        d.prepare_repair_findings(1, 1, &act(&[2])).is_ok(),
        "two findings naming one goal count once, even past the kept deadline"
    );
    let mut rows =
        vec![first, second, Finding { repair: Some(live(30)), ..row(3) }, Finding { repair: Some(live(40)), ..row(4) }];
    assert_eq!(refusal(&fixture(&rows), &[2]).as_ref(), b"project 1 has its 2 repair goals live: goal 30, goal 40");
    rows[3].repair.as_mut().expect("repair fixture").until = None;
    assert!(fixture(&rows).prepare_repair_findings(1, 1, &act(&[2])).is_ok(), "no deadline no cap");
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "ordinary text assertions join independent expectations and count UTF-8 title characters"
)]
fn repair_words_keep_the_act_attempt_evidence_and_whole_findings_with_character_bounded_titles() {
    let first = row(1);
    let mut second = row(2);
    second.repair = Some(FindingRepair {
        goal: 30,
        made: true,
        attempts: 1,
        cancelled: false,
        failing: Box::new([b"test 2".as_slice().into()]),
        until: None,
        judging: None,
    });
    let writes = [FindingWrite { row: first.clone(), erase: None }, FindingWrite { row: second.clone(), erase: None }];
    let header = "Repair 2 tests failing in tier-a\n\nFix these together.\n\nRepository: org/repo. Tier: tier-a.\nName each finding a change repairs in its pull request's body, by its line:\nFixes-finding: 1\nFixes-finding: 2\n";
    let one = "\nFinding 1/1: test fails\nAttempt 1 of 2.\nreproduce\n";
    let two =
        "\nFinding 1/2: test fails\nAttempt 2 of 2.\nStill failing after goal 30: 1 of its tests.\ntest 2\nreproduce\n";
    let full = crate::findings::repair_words(b"org/repo", &writes, &act(&[]).words, 2, 8192).expect("full repair spec");
    assert_eq!(full.as_ref(), [header, one, two].concat().as_bytes());
    let omitted = "\nand more: finding 1/2";
    let bound = u32::try_from(header.len() + one.len() + omitted.len()).expect("repair fixture");
    assert_eq!(
        crate::findings::repair_words(b"org/repo", &writes, &act(&[]).words, 2, bound)
            .expect("one section fits")
            .as_ref(),
        [header, one, omitted].concat().as_bytes()
    );
    let omitted = "\nand more: finding 1/1, finding 1/2";
    assert_eq!(
        crate::findings::repair_words(b"org/repo", &writes, &act(&[]).words, 2, bound - 1)
            .expect("omissions fit")
            .as_ref(),
        [header, omitted].concat().as_bytes()
    );
    let mut many = first.clone();
    many.signature.as_mut().expect("repair fixture").many = true;
    many.signature.as_mut().expect("repair fixture").ids = Box::new([]);
    let tier = "é".repeat(32);
    many.signature.as_mut().expect("repair fixture").source = tier.as_bytes().into();
    let text = crate::findings::repair_words(
        b"org/repo",
        &[FindingWrite { row: many, erase: None }, FindingWrite { row: second, erase: None }],
        b"words",
        2,
        8192,
    )
    .expect("full repair spec");
    let text = core::str::from_utf8(&text).expect("repair fixture");
    let title = text.lines().next().expect("repair fixture");
    assert!(title.starts_with("Repair many tests failing in "));
    assert_eq!(title.chars().count(), 60);
    assert!(title.ends_with('…'));
    assert!(
        text.contains(&format!("Repository: org/repo. Tier: {tier}, tier-a.")),
        "distinct tiers stay whole in oldest order"
    );
    let mut same = first.clone();
    same.number = 3;
    let text = crate::findings::repair_words(
        b"org/repo",
        &[FindingWrite { row: first, erase: None }, FindingWrite { row: same, erase: None }],
        b"words",
        2,
        8192,
    )
    .expect("full repair spec");
    assert!(text.starts_with(b"Repair 1 test failing in tier-a\n"), "ids are distinct and the singular is test");
}

#[test]
fn repair_words_refuse_a_spec_that_cannot_keep_the_header_and_omissions_whole() {
    let writes = [FindingWrite { row: row(1), erase: None }];
    assert!(crate::findings::repair_words(b"org/repo", &writes, &act(&[]).words, 2, 64).is_none());
}
