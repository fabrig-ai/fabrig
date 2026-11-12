//! Exact source inputs, receipts, reports and parity (domain/forge/follows.md, 6–7).
#![expect(clippy::disallowed_methods, reason = "ordinary step tests inspect bounded workflow actions")]
use super::*;
use crate::follow_check::{self as check, Action, ReadKind};
use skein_forge::api;
use skein_lib::List;
use temper_engine_domain_forge_change as c;
use temper_engine_domain_forge_tiers as p;
const PROJECT: u32 = 2;
const SOURCE: api::Repository = api::Repository { forge: 1, repository: 3 };
const RUN: u64 = 71;
const URL: &[u8] = b"https://forge.example/org/repo/actions/runs/71";
const FAILED: &[u8] = b"tests/handoff_1\ntests/handoff_2\n";
fn sha1(byte: u8) -> api::Commit {
    let mut commit = [0; 32];
    commit[..20].fill(byte);
    commit
}
fn configured() -> (Domain, ChangeRow) {
    let mut d = domain();
    d.writers.insert(1, 1).unwrap();
    let mut target = adopted();
    target.project = PROJECT;
    outputs(&mut d, Event::Restore { record: Stored::Repository(target) });
    let mut source = adopted();
    source.provider = SOURCE;
    source.project = PROJECT;
    source.role = Role::Context;
    source.owner = b"org".as_slice().into();
    source.name = b"vsr".as_slice().into();
    outputs(&mut d, Event::Restore { record: Stored::Repository(source) });
    assert!(d.project_landing(
        PROJECT,
        c::Way::Queue,
        Box::new([]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        AfterLanding {
            verdict: p::Settings { retries: 2, suspects: p::Suspects::Bisect },
            bound: Duration::from_secs(600),
        }
    ));
    let mut follow = settings::follow();
    follow.source.repository = SOURCE;
    follow.target.repository = REPO;
    follow.check = crate::settings::FollowCheck {
        workflow: b".forgejo/workflows/follow.yml".as_slice().into(),
        artifact: b"results".as_slice().into(),
        format: p::Format::Lines,
    };
    d.keep_settings(
        &env(),
        1,
        None,
        &[crate::settings::Asked {
            kind: 12,
            scope: PROJECT,
            number: 1,
            edit: crate::settings::Edit::Set(crate::settings::SizedValue {
                value: crate::settings::Value::Follow(follow),
                size: 128,
            }),
        }],
        &mut Queue::uncapped(),
    );
    let mut row = fixtures::opening_row();
    row.change.state = c::State::Gating { head: fixtures::HEAD, asked: Box::new([c::FOLLOW_CHECK_GATE]) };
    row.follows = Some(ChangeFollows { follow: 1, commits: Box::new([sha1(0xaa), sha1(0xbb)]) });
    assert!(check::select(&d, &mut row, &LIMITS));
    (d, row)
}
fn advance(d: &mut Domain, row: &mut ChangeRow) -> Action {
    check::advance(d, &env(), row, fixtures::HEAD, &mut Queue::uncapped())
}
fn keyed(d: &mut Domain, row: &mut ChangeRow) {
    assert!(matches!(advance(d, row), Action::Read { kind: ReadKind::Tree(1), .. }));
    let action = check::answered(
        d,
        &env(),
        row,
        ReadKind::Tree(1),
        Ok(api::Answer::Tree {
            entries: Box::new([api::TreeEntry {
                name: b"src".as_slice().into(),
                object: [9; 32],
                mode: 0o040_000,
                kind: api::TreeEntryKind::Tree,
            }]),
            more: false,
        }),
        &mut List::with_capacity(LIMITS.key_entries),
        &mut Queue::uncapped(),
    );
    assert!(action.is_none());
}
fn sent(d: &Domain, row: &mut ChangeRow) -> api::Write {
    let write = check::write(d, row, fixtures::OPEN).unwrap();
    let flight = row.follow_check.as_mut().unwrap().flight.as_mut().unwrap();
    flight.waiting = false;
    flight.start = Some(env().now);
    flight.since = Some(env().now);
    write
}
fn workflow(row: &ChangeRow, passed: bool) -> api::Run {
    api::Run {
        id: RUN,
        number: 7,
        workflow: b".forgejo/workflows/follow.yml".as_slice().into(),
        commit: row.follow_check.as_ref().unwrap().head,
        actor: 1,
        event: api::Event::Dispatch,
        state: api::RunState::Completed,
        conclusion: Some(if passed { api::Conclusion::Success } else { api::Conclusion::Failure }),
        created: env().now,
        url: URL.into(),
        jobs: Box::new([api::RunJob {
            id: 81,
            name: b"follow".as_slice().into(),
            attempt: 1,
            state: api::RunState::Completed,
            conclusion: Some(if passed { api::Conclusion::Success } else { api::Conclusion::Failure }),
            step: if passed { None } else { Some(api::Step { number: 1, name: b"test".as_slice().into() }) },
            steps: 3,
        }]),
    }
}
fn completed(d: &mut Domain, row: &mut ChangeRow, passed: bool) -> Queue<Request> {
    let run = workflow(row, passed);
    // Rebuilding the caller-owned follower for every read models its restart path.
    let mut out = Queue::uncapped();
    check::answered(
        d,
        &env(),
        row,
        ReadKind::Run,
        Ok(api::Answer::Runs { runs: Box::new([run.clone()]), more: false }),
        &mut List::with_capacity(0),
        &mut out,
    );
    check::answered(d, &env(), row, ReadKind::Run, Ok(api::Answer::Run(run)), &mut List::with_capacity(0), &mut out);
    out
}
fn artifact(d: &mut Domain, row: &mut ChangeRow, bytes: &[u8]) {
    check::answered(
        d,
        &env(),
        row,
        ReadKind::Report,
        Ok(api::Answer::File {
            head: fixtures::HEAD,
            path: b"results".as_slice().into(),
            bytes: bytes.into(),
            truncated: false,
        }),
        &mut List::with_capacity(0),
        &mut Queue::uncapped(),
    );
}
#[test]
fn the_follow_source_is_in_dispatch_and_canonical_key_and_receipts_answer_only_that_source() {
    let (mut d, mut row) = configured();
    keyed(&mut d, &mut row);
    assert!(matches!(advance(&mut d, &mut row), Action::Write));
    let key = row.follow_check.as_ref().unwrap().key.unwrap();
    let check_row = row.follow_check.as_ref().unwrap();
    // Independent canonical bytes: version, repository, gate, definition, padded source, tree.
    let mut bytes = vec![1];
    bytes.extend_from_slice(&1_u64.to_be_bytes());
    bytes.extend_from_slice(&2_u64.to_be_bytes());
    bytes.extend_from_slice(&8_u16.to_be_bytes());
    bytes.extend_from_slice(b"follow.1");
    bytes.extend_from_slice(&u16::try_from(check_row.definition.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(&check_row.definition);
    bytes.extend_from_slice(&sha1(0xbb));
    bytes.extend_from_slice(&3_u16.to_be_bytes());
    bytes.extend_from_slice(b"src");
    bytes.extend_from_slice(&0o040_000_u32.to_be_bytes());
    bytes.extend_from_slice(&[9; 32]);
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(&bytes);
    assert_eq!(key, hash.finish());
    let api::Write::Dispatch { workflow, branch, inputs } = sent(&d, &mut row) else { panic!("dispatch") };
    assert_eq!(workflow.as_ref(), b".forgejo/workflows/follow.yml");
    assert_eq!(branch, row.branch);
    assert_eq!(
        inputs.as_ref(),
        &[
            api::Input { name: b"temper_commit".as_slice().into(), value: commit_text(&fixtures::HEAD) },
            api::Input {
                name: b"temper_key".as_slice().into(),
                value: key.iter().flat_map(|byte| hex(*byte)).collect()
            },
            api::Input { name: b"temper_follows".as_slice().into(), value: Box::from([b'b'; 40]) },
        ]
    );
    let out = completed(&mut d, &mut row, true);
    assert!(out.iter().any(|request| matches!(request, Request::Save { record: Stored::Follow(_), .. })));
    assert_eq!(
        d.follow_row(PROJECT, 1).unwrap().parity,
        Some(FollowParity {
            source: sha1(0xbb),
            head: fixtures::HEAD,
            verdict: FollowVerdict::Passed,
            run: URL.into()
        })
    );
    assert!(matches!(advance(&mut d, &mut row), Action::Done));
    let mut reused = row.clone();
    reused.follow_check = None;
    reused.verdicts = Box::new([]);
    keyed(&mut d, &mut reused);
    assert!(matches!(advance(&mut d, &mut reused), Action::Done), "the same key reuses the receipt without dispatch");
    reused.follows.as_mut().unwrap().commits = Box::new([sha1(0xaa)]);
    assert!(check::select(&d, &mut reused, &LIMITS));
    keyed(&mut d, &mut reused);
    assert_ne!(reused.follow_check.as_ref().unwrap().key, Some(key));
    assert!(matches!(advance(&mut d, &mut reused), Action::Write), "equal trees at a different source have no receipt");
}
#[test]
fn a_follow_report_names_source_failures_or_many_and_sets_parity() {
    for many in [false, true] {
        let (mut d, mut row) = configured();
        let mut environment = env();
        environment.limits.tiers.failing_tests = if many { 1 } else { 2 };
        keyed(&mut d, &mut row);
        sent(&d, &mut row);
        completed(&mut d, &mut row, false);
        assert!(matches!(
            advance(&mut d, &mut row),
            Action::Read { kind: ReadKind::Report, read: api::Read::Artifact { run: RUN, .. } }
        ));
        check::answered(
            &mut d,
            &environment,
            &mut row,
            ReadKind::Report,
            Ok(api::Answer::File {
                head: fixtures::HEAD,
                path: b"results".as_slice().into(),
                bytes: FAILED.into(),
                truncated: false,
            }),
            &mut List::with_capacity(0),
            &mut Queue::uncapped(),
        );
        let expected = if many {
            b"The follow's check failed at 030303030303 against org/vsr bbbbbbbbbbbb: many of the source's tests fail on the target.\nmany\nRun: https://forge.example/org/repo/actions/runs/71".as_slice()
        } else {
            b"The follow's check failed at 030303030303 against org/vsr bbbbbbbbbbbb: 2 of the source's tests fail on the target.\ntests/handoff_1\ntests/handoff_2\nRun: https://forge.example/org/repo/actions/runs/71".as_slice()
        };
        assert_eq!(row.gate_remarks[0].words.as_ref(), expected);
        assert_eq!(row.verdicts[0].status, c::Status::Failed);
        assert_eq!(
            d.follow_row(PROJECT, 1).unwrap().parity.as_ref().unwrap().verdict,
            if many { FollowVerdict::Many } else { FollowVerdict::Failing(2) }
        );
        assert!(matches!(
            check::advance(&mut d, &env(), &mut row, [8; 32], &mut Queue::uncapped()),
            Action::Read { kind: ReadKind::Tree(1), .. }
        ));
        assert!(row.verdicts.is_empty(), "a repair's new head has no verdict");
    }
}
#[test]
fn empty_or_missing_follow_reports_retry_then_hold_without_receipts_or_parity() {
    for missing in [false, true] {
        let (mut d, mut row) = configured();
        keyed(&mut d, &mut row);
        for faults in 1..=3 {
            sent(&d, &mut row);
            completed(&mut d, &mut row, false);
            if missing {
                check::answered(
                    &mut d,
                    &env(),
                    &mut row,
                    ReadKind::Report,
                    Err(api::Error::Missing),
                    &mut List::with_capacity(0),
                    &mut Queue::uncapped(),
                );
            } else {
                artifact(&mut d, &mut row, b"");
            }
            assert_eq!(row.follow_check.as_ref().unwrap().faults, faults);
            assert!(d.landing.receipts.is_empty());
            assert!(row.verdicts.is_empty());
            assert!(d.follow_row(PROJECT, 1).unwrap().parity.is_none());
            if faults < 3 {
                assert!(matches!(advance(&mut d, &mut row), Action::Write));
            }
        }
        assert!(matches!(advance(&mut d, &mut row), Action::Held));
        assert_eq!(
            row.follow_check.as_ref().unwrap().held.as_deref(),
            Some(b"follow check held after 3 runner faults, last: artifact report".as_slice())
        );
    }
}

#[test]
fn a_follow_job_fault_retries_its_failed_attempt_and_an_authority_veto_keeps_the_retry() {
    let (mut d, mut row) = configured();
    keyed(&mut d, &mut row);
    sent(&d, &mut row);
    let mut run = workflow(&row, false);
    run.jobs[0].step.as_mut().unwrap().number = 0;
    let mut out = Queue::uncapped();
    check::answered(
        &mut d,
        &env(),
        &mut row,
        ReadKind::Run,
        Ok(api::Answer::Runs { runs: Box::new([run.clone()]), more: false }),
        &mut List::with_capacity(0),
        &mut out,
    );
    check::answered(
        &mut d,
        &env(),
        &mut row,
        ReadKind::Run,
        Ok(api::Answer::Run(run)),
        &mut List::with_capacity(0),
        &mut out,
    );
    assert_eq!(row.follow_check.as_ref().unwrap().faults, 1);
    let retry = check::write(&d, &mut row, 12).expect("job retry");
    assert_eq!(
        retry,
        api::Write::Rerun { attempt: api::JobAttempt { head: fixtures::HEAD, run: RUN, job: 81, attempt: 1 } }
    );
    check::veto(&mut row, 12, &retry);
    assert_eq!(check::write(&d, &mut row, 13), Some(retry));
    assert!(row.verdicts.is_empty());
    assert!(d.landing.receipts.is_empty());
}

#[test]
fn a_follow_workflow_judges_the_train_prefix_on_its_landing_branch() {
    let (mut d, mut row) = configured();
    let prefix = sha1(0xcc);
    row.change.state = c::State::Judging {
        head: fixtures::HEAD,
        ready_since: env().wall,
        target: c::Judgement {
            landing: 61,
            round: 1,
            pull: 2,
            branch: b"temper/1/l61".as_slice().into(),
            base: b"main".as_slice().into(),
            prefix,
        },
        asked: Box::new([c::FOLLOW_CHECK_GATE]),
    };
    assert_eq!(check::asked(&row), Some(prefix));
    assert!(matches!(check::advance(&mut d, &env(), &mut row, prefix, &mut Queue::uncapped()),
        Action::Read { kind: ReadKind::Tree(1), read: api::Read::Tree { object, .. } } if object == prefix));
    assert!(
        check::answered(
            &mut d,
            &env(),
            &mut row,
            ReadKind::Tree(1),
            Ok(api::Answer::Tree { entries: Box::new([]), more: false }),
            &mut List::with_capacity(LIMITS.key_entries),
            &mut Queue::uncapped()
        )
        .is_none()
    );
    assert!(matches!(check::advance(&mut d, &env(), &mut row, prefix, &mut Queue::uncapped()), Action::Write));
    let api::Write::Dispatch { branch, inputs, .. } = sent(&d, &mut row) else { panic!("dispatch") };
    assert_eq!(branch.as_ref(), b"temper/1/l61");
    assert_eq!(inputs[0].value, commit_text(&prefix));
    assert_eq!(inputs[2].value.as_ref(), &[b'b'; 40]);
}
