//! GitHub's failed check is read at its Actions job, repaired and merged once
//! (temper/docs/design/protocol/forge/README.md, 9).
use super::*;
use skein_fake_forge_domain::api::{Check, Cue};
use skein_forge::api::{Answer, LogPart, Read};
use skein_forge_kit::joint;

const BRANCH: &[u8] = b"temper/51";
const DIAGNOSTIC: &[u8] = b"Missing changes/1.md with the line: opened 2026-11-02T14:01:43Z";
const REQUIRED: &[u8] = b"opened 2026-11-02T14:01:43Z\n";
const FAILED_END: &[u8] = b"\n2026-11-02T14:01:45Z ##[error]Process completed with exit code 1.\n";

fn repair_log() -> Box<[u8]> {
    const LOG_BYTES: usize = 4_096;
    let mut log = vec![b'\n'; LOG_BYTES - 2_200];
    log.extend_from_slice(DIAGNOSTIC);
    log.extend_from_slice(FAILED_END);
    log.extend_from_slice(b"2026-11-02T14:01:46Z Post job cleanup.\n");
    while log.len() < LOG_BYTES {
        log.extend_from_slice(b"git config --local --get-regexp http.extraheader\n");
    }
    log.truncate(LOG_BYTES);
    log.into_boxed_slice()
}

#[test]
fn a_github_change_whose_check_run_fails_is_repaired() {
    let mut checks = joint::checks(0);
    checks.cue = Some(Cue::PullFile {
        path: Box::from(b"changes/1.md".as_slice()),
        no_pull: Check::Passed,
        step: Box::from(b"changes".as_slice()),
        log: repair_log(),
    });
    let mut world = Protocol::github_with_check_log(73, checks);
    world.push_file(b"main", b"README.md", b"# app\n");
    let main = world.branch(b"main");
    world.run_for(1);
    world.adopt();
    world.event(top::Event::Change { row: change_row() });
    assert_eq!(
        change_step(&mut world, 1, change::Status::Unknown, Box::new([])),
        change::Decision::Delegate(change::Delegate::Produce)
    );
    // The producer's checkout has no gate timestamp.
    world.produce_file(BRANCH, b"README.md", b"# change\n");
    let first = world.branch(BRANCH);
    assert_eq!(
        change_step(&mut world, 2, change::Status::Passed, Box::new([])),
        change::Decision::Effect(change::Effect::Open)
    );
    assert_eq!(
        change_step(&mut world, 3, change::Status::Passed, Box::new([])),
        change::Decision::Delegate(change::Delegate::Repair(change::Repair::Ci))
    );
    github_head_checks(&mut world, skein_forge::api::Check::Failed);
    read_failed_job(&mut world, first);
    assert_eq!(world.tally().routes.get(&Route::Merge), None);

    let repairer = (52, 1);
    world.push_attempt(repairer, BRANCH, b"changes/1.md", REQUIRED);
    let second = world.branch(BRANCH);
    assert_ne!(first, second, "repair pushes a new head");
    let mut finished = false;
    for entry in 4..20 {
        if let change::Decision::Finish { .. } = change_step(&mut world, entry, change::Status::Passed, Box::new([])) {
            finished = true;
            break;
        }
    }
    assert!(finished, "the repaired change lands");
    check_heads_and_merge(world.observations(), main, first, second);
    assert_eq!(world.tally().routes.get(&Route::Merge), Some(&1));
    assert_eq!(world.tally().routes.get(&Route::PostStatus), None);
    let head = temper_engine_forge_world::translate::commit(world.branch(b"main"));
    let Answer::File { bytes, truncated: false, .. } =
        commit_read(&mut world, Read::File { head, path: Box::from(b"changes/1.md".as_slice()), max_bytes: 128 })
    else {
        panic!("landed repair file")
    };
    assert_eq!(bytes.as_ref(), REQUIRED);
}

fn read_failed_job(world: &mut Protocol, first: u64) {
    let commit = temper_engine_forge_world::translate::commit(first);
    let Answer::Checks(checks) = commit_read(world, Read::Checks { commit }) else { panic!("failed check") };
    let [check] = checks.as_ref() else { panic!("one check: {checks:?}") };
    assert_eq!(check.check, skein_forge::api::Check::Failed);
    let attempt = check.job.expect("the check resolves its Actions job");
    assert_eq!(attempt.head, commit);
    let Answer::Job { attempt: read, step, log, truncated } =
        commit_read(world, Read::Job { part: LogPart::End, attempt, max_bytes: 4_096 })
    else {
        panic!("failed job log")
    };
    assert_eq!(read, attempt);
    assert_eq!(step.expect("failed step").name.as_ref(), b"changes");
    assert!(log.windows(DIAGNOSTIC.len()).any(|part| part == DIAGNOSTIC));
    assert!(log.ends_with(FAILED_END), "the failed step's end precedes cleanup");
    let failed_bytes = 4_096 - 2_200 + DIAGNOSTIC.len() + FAILED_END.len();
    assert!(log.ends_with(&repair_log()[..failed_bytes]), "the complete failed step fits");
    assert!(truncated, "omitted post-job cleanup is reported");
    let target = format!("GET /repos/acme/widgets/actions/jobs/{}/logs ", attempt.job);
    assert!(world.requests().any(|(_, line)| line.starts_with(target.as_bytes())), "the failed Actions job's log");
    let blob = format!("GET /logs/2/{}/{}", attempt.job, attempt.attempt);
    assert!(world.requests().any(|(_, line)| line.starts_with(blob.as_bytes())), "the layer's signed log origin");
}

fn check_heads_and_merge(records: &[Observation], main: u64, first: u64, second: u64) {
    assert!(
        records.iter().any(|row| matches!(row,
            Observation::Reported { commit, state: Check::Passed, .. } if *commit == main
        )),
        "seeded main passed its CI"
    );
    let heads: Vec<_> = records
        .iter()
        .filter_map(|row| {
            let Observation::Moved { branch, from: Some(_), to, .. } = row else { return None };
            (branch.as_ref() == BRANCH).then_some(*to)
        })
        .collect();
    assert_eq!(heads, [first, second], "exactly two change pushes");
    let failed_at = records
        .iter()
        .position(|row| {
            matches!(row,
                Observation::Reported { commit, state: Check::Failed, .. } if *commit == first
            )
        })
        .expect("pull-specific CI failed");
    // Opening the pull reruns the content cue; its no-pull verdict also seeds green main.
    let reports: Vec<_> = records
        .iter()
        .enumerate()
        .skip(failed_at.saturating_sub(1))
        .filter_map(|(at, row)| {
            let Observation::Reported { commit, state, .. } = row else { return None };
            (*commit == first || *commit == second).then_some((at, *commit, *state))
        })
        .collect();
    let [
        (pending_a, a, Check::Pending),
        (failed, failed_head, Check::Failed),
        (pending_b, b, Check::Pending),
        (passed, passed_head, Check::Passed),
    ] = reports.as_slice()
    else {
        panic!("failed then passed at two heads: {reports:?}")
    };
    assert_eq!((*a, *failed_head, *b, *passed_head), (first, first, second, second));
    assert!(pending_a < failed && failed < pending_b && pending_b < passed);
    let merges: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, row)| {
            let Observation::Merged { head, .. } = row else { return None };
            Some((at, *head))
        })
        .collect();
    let [(merged_at, merged_head)] = merges.as_slice() else { panic!("one merge: {merges:?}") };
    assert_eq!(*merged_head, second);
    assert!(passed < merged_at, "CI passed before merge");
}
