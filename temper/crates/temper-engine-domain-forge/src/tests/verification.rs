//! Landing verification stories (temper/docs/design/domain/after-landing.md, 5.3, 6 and 10).
use super::*;
use crate::*;
use skein_lib::{Time, Wall};
use temper_engine_domain_forge_tiers as policy;
const REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 2 };
const PROJECT: u32 = 5;
const CHANGE: u64 = 51;
const SECOND: u64 = 52;
const OLDER_REPAIR: u64 = 53;
const NEWER_REPAIR: u64 = 54;
const TIER_TASK: u64 = 61;
const OPEN: u64 = 11;
const HEAD: skein_forge::api::Commit = [3; 32];
const FIRST: skein_forge::api::Commit = [6; 32];
const LAST: skein_forge::api::Commit = [7; 32];
const REPAIR: skein_forge::api::Commit = [8; 32];
const TIER: &[u8] = b"browser";
const BOUND: Duration = Duration::from_secs(60);
const CLIENT: skein_forge_client::Limits = skein_forge_client::Limits {
    pending: 4,
    calls: 2,
    rate: 100,
    reserve: 0,
    window: Duration::from_secs(10),
    op_bytes: 512,
    answer_bytes: 4096,
    rows: 8,
    inbox: 16,
    read_attempts: 3,
    backoff: Duration::from_secs(1),
    backoff_max: Duration::from_secs(4),
    entries: 8,
    write_attempts: 3,
    lifetime: Duration::from_secs(10),
    clock_margin: Duration::from_secs(1),
    resources: 8,
    repositories: 2,
    poll: Duration::from_secs(5),
    poll_max: Duration::from_secs(30),
    hinted: Duration::from_secs(1),
    slow: Duration::from_secs(40),
    settle: Duration::from_secs(30),
    facts: 16,
};
const LIMITS: Limits = Limits {
    forges: 4,
    repositories: 2,
    tasks: 8,
    holds: 8,
    subscriptions: 8,
    entries: 8,
    named_calls: 16,
    changes: 8,
    issues: 8,
    resources_per_task: 4,
    paths_per_subscription: 4,
    // forge/host/owner/repository/branch/temper/root/marker-task.
    authority_segments: 8,
    name_bytes: 128,
    output: 80,
    facts: 16,
    adoptions: 2,
    applications: 4,
    bindings: 4,
    collaborators: 8,
    landings: 8,
    landing_members: 2,
    gates: 2,
    receipts_per_gate: 4,
    ignored_paths: 4,
    key_entries: 16,
    key_path_bytes: 1024,
    trees_in_flight: 2,
    tiers: TierLimits { tiers: 2, failing_tests: 16, test_id_bytes: 256 },
    judge_projects: 2,
    judge_criteria: 8,
    brief_sections: 32,
    brief_bytes: 4096,
    issue_policy: temper_engine_domain_forge_issues::Limits {
        plan_items: 8,
        milestones: 8,
        title_bytes: 128,
        body_bytes: 512,
        comment_bytes: 256,
        interval: Duration::from_secs(5),
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        repairs: 3,
        resolutions: 3,
        updates: 3,
        stall: Duration::from_secs(30),
        gates: 8,
        clean_heads: 8,
    },
    queue_window: Duration::from_secs(30),
    client: CLIENT,
};

fn env() -> Env<Limits> {
    Env { now: Time::from_nanos(100_000_000_000), wall: Wall::from_nanos(100_000_000_000), limits: LIMITS }
}
fn domain() -> Domain {
    Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config {
            namespace: Box::from(&b"test"[..]),
            writers: Box::new([skein_forge_client::Writer { forge: 1, author: 7 }]),
        },
        &[GitIdentity { forge: 1, credential: 1 }],
        &configured()[..1],
        false,
    )
    .expect("valid connector")
}
fn configured() -> [Configured; 2] {
    [
        Configured {
            admin_connect: false,
            bot: Box::new([]),
            owned: None,
            git: Box::new([]),
            web: None,
            read_only: false,
            number: 1,
            kind: Kind::Forgejo,
            mode: Mode::Owned,
            asked: true,
        },
        Configured {
            admin_connect: false,
            bot: Box::new([]),
            owned: None,
            git: Box::new([]),
            web: None,
            read_only: false,
            number: 2,
            kind: Kind::GitHub,
            mode: Mode::Owned,
            asked: true,
        },
    ]
}
fn adopted() -> Repository {
    Repository {
        project: 5,
        home: true,
        provider: REPO,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: Box::from(&b"main"[..]),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}
fn opening_row() -> ChangeRow {
    use temper_engine_domain_forge_change as change;
    ChangeRow {
        task: CHANGE,
        repository: REPO,
        branch: Box::from(b"temper/51".as_slice()),
        base: Box::from(b"main".as_slice()),
        title: Box::from(b"Small fix".as_slice()),
        body: Box::new([]),
        priority: 1,
        ci: None,
        change: change::Change {
            way: temper_engine_domain_forge_change::Way::Queue,
            landing: None,
            task: CHANGE,
            state: change::State::Opening { requested: true },
            gates: Box::new([]),
            clean: Box::new([]),
            gate_failures: Box::new([]),
            released: None,
            repairs: 0,
            resolutions: 0,
            updates: 0,
            since: env().wall,
            ready_since: None,
            owns_turn: false,
            last_head: Some(HEAD),
        },
        pull: None,
        pending: Some(OPEN),
        effect: change::EffectResult::Pending,
        delegate: None,
        delegate_status: change::Status::Passed,
        verdicts: Box::new([]),
        result: ChangeResult::empty(),
        gate_remarks: Box::new([]),
        drift: None,
        base_repair: false,
        queue_repair: None,
        queue_repairs: 0,
        producing_charter: 2,
        review_charter: 4,
        effect_account: None,
        producer_failure: None,
        report: Box::new([]),
        repairs: None,
    }
}

fn configured_domain(names: &[(&[u8], Option<&[u8]>)]) -> Domain {
    let mut d = domain();
    d.repositories.insert(REPO, adopted()).expect("adoption");
    let mut tiers = List::uncapped();
    for (name, branch) in names {
        tiers
            .push(Tier {
                name: Box::from(*name),
                workflow: Box::from(b"tier.yml".as_slice()),
                ignored: Box::new([]),
                artifact: Box::from(b"report".as_slice()),
                format: policy::Format::Lines,
                follows: match branch {
                    Some(branch) => Follows::Branch(Box::from(*branch)),
                    None => Follows::Landing,
                },
                span: None,
            })
            .expect("fixture tiers");
    }
    assert!(d.project_landing(
        PROJECT,
        change::Way::Queue,
        Box::new([RepositoryGates { repository: REPO, gates: Box::new([]), tiers: tiers.into_boxed() }]),
        AfterLanding { verdict: policy::Settings { retries: 2, suspects: policy::Suspects::Bisect }, bound: BOUND }
    ));
    d
}

fn add_change(d: &mut Domain, task: u64, base: &[u8]) {
    let mut row = opening_row();
    row.task = task;
    row.change.task = task;
    row.base = Box::from(base);
    row.pending = None;
    row.change.state = change::State::Landing { head: HEAD };
    d.changes.insert(task, row).expect("fixture change");
}

fn merged(d: &mut Domain, task: u64, commit: skein_forge::api::Commit) -> Queue<Request> {
    let mut out = Queue::uncapped();
    outcome_facts(
        d,
        &env(),
        OPEN,
        task,
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(commit), found: false },
        &mut out,
    );
    out
}

fn verifying() -> Domain {
    let mut d = configured_domain(&[(TIER, None)]);
    add_change(&mut d, CHANGE, b"main");
    add_change(&mut d, SECOND, b"main");
    merged(&mut d, CHANGE, FIRST);
    merged(&mut d, SECOND, LAST);
    d
}

fn heard(cancelled: bool) -> change::Heard {
    change::Heard {
        lacking: None,
        spend: None,
        delegate: change::Status::Unknown,
        effect: change::EffectResult::None,
        released: false,
        cancelled,
    }
}

fn advance(d: &mut Domain, task: u64, wall: Wall, cancelled: bool) -> Queue<Request> {
    let mut out = Queue::uncapped();
    let mut environment = env();
    environment.wall = wall;
    step_change(
        d,
        &environment,
        StepInput {
            task,
            entry: OPEN,
            heard: heard(cancelled),
            gates: Box::new([]),
            queue_repair_active: false,
            kept: false,
        },
        &mut out,
    );
    out
}

fn tier_run(d: &mut Domain, tier: &[u8], number: u64, commits: &[skein_forge::api::Commit]) -> policy::Run {
    let mut landings = List::uncapped();
    for commit in commits {
        landings.push(policy::Landing { commit: *commit, key: *commit }).expect("range");
    }
    let tip = *commits.last().expect("run tip");
    let run =
        policy::Run { number, tip, key: tip, landings: landings.into_boxed(), part: policy::Part::Whole, faults: 0 };
    d.tiers
        .insert(
            (REPO, Box::from(tier)),
            TierRow {
                observations: Box::new([]),
                repository: REPO,
                tier: Box::from(tier),
                task: None,
                requested: false,
                cancelled: false,
                purpose: None,
                pending: Box::new([]),
                baseline: None,
                branch: TierBranch::Absent,
                probe: None,
                flight: None,
                held: None,
                result: None,
                received: false,
                goals: Box::new([]),
                goal: None,
                started: None,
                record: policy::Record {
                    sequence: number,
                    judged: None,
                    failing: policy::Failures::Tests(Box::new([])),
                    flakes: Box::new([]),
                    run: Some(run.clone()),
                },
            },
        )
        .expect("tier");
    run
}

fn judge(
    d: &mut Domain,
    tier: &[u8],
    number: u64,
    commits: &[skein_forge::api::Commit],
    blames: Box<[policy::Blame]>,
    failing: policy::Failures,
) -> Queue<Request> {
    let run = tier_run(d, tier, number, commits);
    let tip = run.tip;
    let verdict = policy::Verdict { number, tip, key: tip, failing, landings: blames, flakes: Box::new([]) };
    let mut out = Queue::uncapped();
    let row = d.tiers.get_mut(&(REPO, Box::from(tier))).expect("tier");
    row.record = policy::judged(&row.record, &verdict, &policy::Limits { failing_tests: 16, test_id_bytes: 256 });
    verification::judged(d, REPO, tier, &verdict, &run.landings, &mut out);
    out
}

fn green(d: &mut Domain, number: u64, commits: &[skein_forge::api::Commit]) -> Queue<Request> {
    let run = tier_run(d, TIER, number, commits);
    let row = d.tiers.get_mut(&(REPO, Box::from(TIER))).expect("tier");
    row.task = Some(TIER_TASK);
    row.result = Some(Receipt {
        failures: None,
        passed: true,
        run: ReceiptRun { id: number, number, url: Box::new([]) },
        candidate: run.tip,
        ended: env().wall,
    });
    let mut out = Queue::uncapped();
    tiers::step(d, &env(), TIER_TASK, OPEN, TierHeard { released: false, cancelled: false }, &mut out);
    out
}

fn red(d: &mut Domain) {
    judge(
        d,
        TIER,
        1,
        &[FIRST, LAST],
        Box::new([policy::Blame::Cleared, policy::Blame::Owns(policy::Failures::Many)]),
        policy::Failures::Many,
    );
}

#[test]
fn merge_saves_verification_with_landed_and_preserves_observation_time() {
    let mut d = configured_domain(&[(TIER, None), (b"release", Some(b"release"))]);
    add_change(&mut d, CHANGE, b"main");
    let out = merged(&mut d, CHANGE, FIRST);
    let expected = Verification {
        repository: REPO,
        commit: FIRST,
        changes: Box::new([CHANGE]),
        tiers: Box::new([TierVerification { tier: Box::from(TIER), verdict: None, repaired: None }]),
    };
    let mut records = List::uncapped();
    for request in &out {
        if let Request::Save { record, .. } = request {
            records.push(record.clone()).expect("saves");
        }
    }
    assert!(records.as_slice().contains(&Stored::Landed { commit: FIRST, task: CHANGE }));
    assert!(records.as_slice().contains(&Stored::Verification(expected)));
    assert_eq!(d.change_verification(CHANGE), change::Verification::Verifying);
    assert_eq!(
        d.change(CHANGE).expect("change").change.state,
        change::State::Landed { merge: FIRST, since: env().wall }
    );
}

#[test]
fn two_landings_in_one_green_run_finish_cleared() {
    let mut d = verifying();
    green(&mut d, 1, &[FIRST, LAST]);
    let mut wakes = Queue::uncapped();
    // The watcher may consume a resume before the bounded change wakes are emitted.
    for _ in 0..=LIMITS.changes {
        resume(&mut d, &env(), &mut wakes);
        if d.procedure_wakes.is_empty() {
            break;
        }
    }
    for (task, commit) in [(CHANGE, FIRST), (SECOND, LAST)] {
        assert!(contains(&wakes, &Request::WakeProcedure { task }));
        let out = advance(&mut d, task, env().wall, false);
        assert!(contains(
            &out,
            &Request::ChangeDecision {
                task,
                decision: change::Decision::Finish { merge: commit },
                entry: None,
                evidence: None
            }
        ));
        assert_eq!(d.change(task).expect("change").change.state, change::State::Cleared { merge: commit });
    }
}

#[test]
fn blame_reddens_only_the_second_landing() {
    let mut d = verifying();
    red(&mut d);
    advance(&mut d, CHANGE, env().wall, false);
    advance(&mut d, SECOND, env().wall, false);
    assert_eq!(d.change(CHANGE).expect("first").change.state, change::State::Cleared { merge: FIRST });
    assert_eq!(d.change(SECOND).expect("second").change.state, change::State::Red { merge: LAST, since: env().wall });
}

#[test]
fn later_green_repairs_red_without_replacing_its_verdict() {
    let mut d = verifying();
    red(&mut d);
    advance(&mut d, SECOND, env().wall, false);
    green(&mut d, 2, &[REPAIR]);
    advance(&mut d, SECOND, env().wall, false);
    let tier = &d.verifications.get(&(REPO, LAST)).expect("row").tiers[0];
    assert_eq!(tier.repaired, Some(TierRepair { run: 2, tip: REPAIR, by: None }));
    assert_eq!(tier.verdict, Some(TierVerdict { run: 1, blame: policy::Blame::Owns(policy::Failures::Many) }));
    assert_eq!(d.change(SECOND).expect("change").change.state, change::State::Cleared { merge: LAST });
    green(&mut d, 3, &[HEAD]);
    assert_eq!(
        d.verifications.get(&(REPO, LAST)).expect("row").tiers[0].repaired,
        Some(TierRepair { run: 2, tip: REPAIR, by: None })
    );
}

fn repairing_range(older: Option<skein_forge::api::Commit>, newer: Option<skein_forge::api::Commit>) -> Domain {
    let mut d = verifying();
    red(&mut d);
    for (task, commit, repairs) in [(OLDER_REPAIR, REPAIR, older), (NEWER_REPAIR, HEAD, newer)] {
        add_change(&mut d, task, b"main");
        d.changes.get_mut(&task).expect("repair change").repairs = repairs;
        merged(&mut d, task, commit);
    }
    green(&mut d, 2, &[REPAIR, HEAD]);
    d
}

#[test]
fn later_run_attributes_repair_to_the_landing_naming_red() {
    let d = repairing_range(None, Some(LAST));
    assert_eq!(
        d.verifications.get(&(REPO, LAST)).expect("red landing").tiers[0].repaired,
        Some(TierRepair { run: 2, tip: HEAD, by: Some(HEAD) })
    );
}

#[test]
fn later_run_alone_repairs_when_no_change_names_red() {
    let d = repairing_range(None, None);
    assert_eq!(
        d.verifications.get(&(REPO, LAST)).expect("red landing").tiers[0].repaired,
        Some(TierRepair { run: 2, tip: HEAD, by: None })
    );
}

#[test]
fn newest_landing_naming_red_is_its_repairer() {
    let d = repairing_range(Some(LAST), Some(LAST));
    assert_eq!(
        d.verifications.get(&(REPO, LAST)).expect("red landing").tiers[0].repaired,
        Some(TierRepair { run: 2, tip: HEAD, by: Some(HEAD) })
    );
}

#[test]
fn repairer_precedes_newer_landings_that_do_not_name_red() {
    for newer in [None, Some(FIRST)] {
        let d = repairing_range(Some(LAST), newer);
        assert_eq!(
            d.verifications.get(&(REPO, LAST)).expect("red landing").tiers[0].repaired,
            Some(TierRepair { run: 2, tip: HEAD, by: Some(REPAIR) }),
            "newer landing repairs {newer:?}"
        );
    }
}

#[test]
fn no_verdict_at_the_project_bound_finishes_uncleared() {
    let mut d = verifying();
    let deadline = Wall::from_nanos(env().wall.as_nanos() + BOUND.as_nanos());
    let before = Wall::from_nanos(deadline.as_nanos() - 1);
    let out = advance(&mut d, CHANGE, before, false);
    assert!(contains(
        &out,
        &Request::ChangeDecision {
            task: CHANGE,
            decision: change::Decision::Wait { until: Some(deadline) },
            entry: None,
            evidence: None
        }
    ));
    d.wait_change(&env(), CHANGE, Some(deadline));
    let mut environment = env();
    environment.wall = deadline;
    let mut out = Queue::uncapped();
    // Restore-stage timers are enabled by the restart script.
    d.restore_phase = RestorePhase::Active;
    fire(&mut d, &environment, &mut out);
    assert!(contains(&out, &Request::WakeProcedure { task: CHANGE }));
    advance(&mut d, CHANGE, deadline, false);
    assert_eq!(d.change(CHANGE).expect("change").change.state, change::State::Uncleared { merge: FIRST });
}

#[test]
fn cancel_while_verifying_finishes_uncleared() {
    let mut d = verifying();
    advance(&mut d, CHANGE, env().wall, true);
    assert_eq!(d.change(CHANGE).expect("change").change.state, change::State::Uncleared { merge: FIRST });
}

#[test]
fn an_unfollowed_goal_branch_finishes_landed_at_once() {
    let mut d = configured_domain(&[(TIER, None)]);
    add_change(&mut d, CHANGE, b"temper/goal");
    merged(&mut d, CHANGE, FIRST);
    assert!(d.verifications.is_empty());
    assert_eq!(d.change_verification(CHANGE), change::Verification::Unfollowed);
    let out = advance(&mut d, CHANGE, env().wall, false);
    assert!(contains(
        &out,
        &Request::ChangeDecision {
            task: CHANGE,
            decision: change::Decision::Finish { merge: FIRST },
            entry: None,
            evidence: None
        }
    ));
    assert_eq!(
        d.change(CHANGE).expect("change").change.state,
        change::State::Landed { merge: FIRST, since: env().wall }
    );
}

#[test]
fn a_cold_restart_keeps_red_rows_and_changes() {
    let mut before = verifying();
    red(&mut before);
    let row = before.verifications.get(&(REPO, LAST)).expect("verification").clone();
    let change = before.change(SECOND).expect("change").clone();
    let mut d = configured_domain(&[(TIER, None)]);
    d.restore_begin(Wall::EPOCH);
    restore(&mut d, &env(), Stored::Verification(row.clone()));
    restore(&mut d, &env(), Stored::Change(change));
    assert_eq!(d.verifications.get(&(REPO, LAST)), Some(&row));
    assert_eq!(d.change_verification(SECOND), change::Verification::Red);
}

#[test]
fn closing_the_last_change_erases_the_row_and_late_verdict_ignores_it() {
    let mut d = verifying();
    let mut out = Queue::uncapped();
    end_change(&mut d, &env(), CHANGE, &mut out);
    end_change(&mut d, &env(), SECOND, &mut out);
    assert!(contains(&out, &Request::Erase { key: Key::Verification { repository: REPO, commit: LAST } }));
    assert!(d.verifications.is_empty());
    let out = judge(
        &mut d,
        TIER,
        1,
        &[FIRST, LAST],
        Box::new([policy::Blame::Cleared, policy::Blame::Owns(policy::Failures::Many)]),
        policy::Failures::Many,
    );
    assert!(out.is_empty());
    assert!(d.procedure_wakes.is_empty());
    assert_eq!(d.change_verification(SECOND), change::Verification::Unfollowed);
}

#[test]
fn a_verdict_waits_for_all_tiers_and_red_takes_precedence() {
    const OTHER: &[u8] = b"fuzzy";
    let mut d = configured_domain(&[(TIER, None), (OTHER, None)]);
    add_change(&mut d, CHANGE, b"main");
    merged(&mut d, CHANGE, FIRST);
    green(&mut d, 1, &[FIRST]);
    assert_eq!(d.change_verification(CHANGE), change::Verification::Verifying);
    assert!(d.procedure_wakes.contains_key(&CHANGE));
    judge(
        &mut d,
        OTHER,
        1,
        &[FIRST],
        Box::new([policy::Blame::Suspect(policy::Failures::Many)]),
        policy::Failures::Many,
    );
    assert_eq!(d.change_verification(CHANGE), change::Verification::Red);
    judge(&mut d, OTHER, 2, &[REPAIR], Box::new([policy::Blame::Cleared]), policy::Failures::Tests(Box::new([])));
    assert_eq!(d.change_verification(CHANGE), change::Verification::Cleared);
}

#[test]
fn a_shared_row_lives_until_its_last_change_closes() {
    let mut d = verifying();
    verification::released(&mut d, SECOND, &mut Queue::uncapped());
    merged(&mut d, SECOND, FIRST);
    let mut out = Queue::uncapped();
    end_change(&mut d, &env(), CHANGE, &mut out);
    assert_eq!(d.verifications.get(&(REPO, FIRST)).expect("shared row").changes.as_ref(), &[SECOND]);
    end_change(&mut d, &env(), SECOND, &mut out);
    assert!(d.verifications.is_empty());
}

#[test]
fn a_replayed_merge_keeps_its_first_verdict_and_clock() {
    let mut d = verifying();
    red(&mut d);
    advance(&mut d, SECOND, env().wall, false);
    let expected = d.verifications.get(&(REPO, LAST)).expect("row").clone();
    let mut environment = env();
    environment.wall = Wall::from_nanos(env().wall.as_nanos() + BOUND.as_nanos());
    outcome_facts(
        &mut d,
        &environment,
        OPEN,
        SECOND,
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(LAST), found: true },
        &mut Queue::uncapped(),
    );
    assert_eq!(d.verifications.get(&(REPO, LAST)), Some(&expected));
    assert_eq!(d.change(SECOND).expect("change").change.state, change::State::Red { merge: LAST, since: env().wall });
}

#[test]
fn a_late_red_stays_on_the_tier_after_uncleared_closes() {
    let mut d = verifying();
    advance(&mut d, SECOND, Wall::from_nanos(env().wall.as_nanos() + BOUND.as_nanos()), false);
    end_change(&mut d, &env(), SECOND, &mut Queue::uncapped());
    red(&mut d);
    assert_eq!(d.change(SECOND).expect("closed change").change.state, change::State::Uncleared { merge: LAST });
    assert!(!d.verifications.contains_key(&(REPO, LAST)));
    assert!(!d.procedure_wakes.contains_key(&SECOND));
    assert_eq!(d.tiers.get(&(REPO, Box::from(TIER))).expect("tier").record.failing, policy::Failures::Many);
}

#[test]
fn a_repository_without_tiers_creates_no_row() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"main");
    merged(&mut d, CHANGE, FIRST);
    assert!(d.verifications.is_empty());
    assert_eq!(d.change_verification(CHANGE), change::Verification::Unfollowed);
}

#[test]
fn a_tier_follows_its_explicit_branch() {
    let mut d = configured_domain(&[(TIER, Some(b"release"))]);
    add_change(&mut d, CHANGE, b"release");
    merged(&mut d, CHANGE, FIRST);
    assert_eq!(d.change_verification(CHANGE), change::Verification::Verifying);
}

fn contains(out: &Queue<Request>, expected: &Request) -> bool {
    for request in out {
        if request == expected {
            return true;
        }
    }
    false
}

#[test]
fn goal_tiers_leave_goal_landings_unfollowed_and_main_verified() {
    let mut d = configured_domain(&[(TIER, None), (b"g", None)]);
    d.landing.projects.get_mut(&PROJECT).expect("configured project").1[0].tiers[1].follows = Follows::Goals;
    add_change(&mut d, CHANGE, b"temper/1/g1");
    merged(&mut d, CHANGE, FIRST);
    assert!(d.verifications.is_empty());
    assert_eq!(d.change_verification(CHANGE), change::Verification::Unfollowed);
    let out = advance(&mut d, CHANGE, env().wall, false);
    assert!(contains(
        &out,
        &Request::ChangeDecision {
            task: CHANGE,
            decision: change::Decision::Finish { merge: FIRST },
            entry: None,
            evidence: None,
        }
    ));
    assert_eq!(
        d.change(CHANGE).expect("goal landing").change.state,
        change::State::Landed { merge: FIRST, since: env().wall }
    );
    add_change(&mut d, SECOND, b"main");
    merged(&mut d, SECOND, LAST);
    assert_eq!(d.change_verification(SECOND), change::Verification::Verifying);
    let verification = d.verifications.get(&(REPO, LAST)).expect("main verification");
    assert_eq!(verification.tiers.as_ref(), &[TierVerification { tier: TIER.into(), verdict: None, repaired: None }]);
    green(&mut d, 1, &[LAST]);
    advance(&mut d, SECOND, env().wall, false);
    assert_eq!(d.change_verification(SECOND), change::Verification::Cleared);
    assert_eq!(d.change(SECOND).expect("main landing").change.state, change::State::Cleared { merge: LAST });
}

#[test]
fn landing_red_is_unknown_without_a_live_landing_in_the_repository() {
    let d = verifying();
    assert_eq!(d.landing_red(REPO, &REPAIR, b"main"), LandingRed::Unknown);
    assert_eq!(
        d.landing_red(
            skein_forge::api::Repository { forge: REPO.forge, repository: REPO.repository + 1 },
            &FIRST,
            b"main"
        ),
        LandingRed::Unknown
    );
}

#[test]
fn landing_red_names_the_first_live_changes_other_branch() {
    let mut d = verifying();
    red(&mut d);
    d.verifications.get_mut(&(REPO, LAST)).unwrap().changes = Box::new([99, SECOND]);
    assert_eq!(d.landing_red(REPO, &LAST, b"release"), LandingRed::Elsewhere { branch: Box::from(b"main".as_slice()) });
}

#[test]
fn landing_red_is_not_red_after_green_and_after_repair() {
    let mut d = verifying();
    assert_eq!(d.landing_red(REPO, &LAST, b"main"), LandingRed::NotRed);
    green(&mut d, 1, &[FIRST, LAST]);
    assert_eq!(d.landing_red(REPO, &LAST, b"main"), LandingRed::NotRed);
    let mut d = verifying();
    red(&mut d);
    green(&mut d, 2, &[REPAIR]);
    assert_eq!(d.landing_red(REPO, &LAST, b"main"), LandingRed::NotRed);
}

#[test]
fn landing_red_is_red_for_unrepaired_owns_and_suspect_verdicts() {
    let mut d = verifying();
    red(&mut d);
    assert_eq!(d.landing_red(REPO, &FIRST, b"main"), LandingRed::NotRed);
    assert_eq!(d.landing_red(REPO, &LAST, b"main"), LandingRed::Red);
    let mut d = verifying();
    judge(
        &mut d,
        TIER,
        1,
        &[FIRST],
        Box::new([policy::Blame::Suspect(policy::Failures::Many)]),
        policy::Failures::Many,
    );
    assert_eq!(d.landing_red(REPO, &FIRST, b"main"), LandingRed::Red);
}
