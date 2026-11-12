//! Public source-table tests (temper/docs/design/client/forge/README.md, 9.4).
use crate::*;
use skein_lib::{Duration, Env, Time, Wall};
#[test]
fn unknown_is_one_return() {
    let d = setup();
    let public = handed();
    for (kind, version, project, task) in [
        (0, 1, 5, None),
        (w::PUBLIC_CHANGE, 2, 5, Some(CHANGE)),
        (w::PUBLIC_CHANGE, 1, 5, None),
        (w::PUBLIC_LANDING, 1, 5, None),
        (w::PUBLIC_LANDINGS, 1, 5, Some(CHANGE)),
        (w::PUBLIC_CHANGE, 1, 6, Some(CHANGE)),
        (w::PUBLIC_LANDING, 1, 6, Some(CHANGE)),
        (w::PUBLIC_CHANGE, 1, 5, Some(LANDING)),
    ] {
        let mut ask = ask(kind, task, &public);
        ask.version = version;
        ask.project = project;
        assert!(d.public(&ask, &LIMITS).is_none(), "{kind} {version} {project} {task:?}");
    }
}

const REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 2 };
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
    fallback: Duration::ZERO,
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
    makings: 2,
    applications: 4,
    bindings: 4,
    collaborators: 8,
    landings: 8,
    landing_members: 2,
    landings_kept: 16,
    changes_rows: 32,
    changes_landed: 8,
    gates: 2,
    receipts_per_gate: 4,
    ignored_paths: 4,
    key_entries: 16,
    key_path_bytes: 1024,
    trees_in_flight: 2,
    tiers: TierLimits { tiers: 2, failing_tests: 16, test_id_bytes: 256 },
    judge_projects: 2,
    decided_per_project: 64,
    judge_criteria: 8,
    landing_rules: 4,
    roles: 4,
    setting_proposals: 4,
    setting_history: 4,
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
        questions: 4,
        tasks: 6,
        stall: Duration::from_secs(30),
        gates: 8,
        clean_heads: 8,
    },
    queue_window: Duration::from_secs(30),
    sitting: SITTING,
    client: CLIENT,
};

fn env() -> Env<Limits> {
    Env { now: Time::from_nanos(100_000_000_000), wall: Wall::from_nanos(100_000_000_000), limits: LIMITS }
}

fn domain_with_limits(limits: &Limits) -> Domain {
    Domain::new(
        limits,
        4,
        Config {
            push: None,
            namespace: Box::from(&b"test"[..]),
            writers: Box::new([skein_forge_client::Writer { forge: 1, author: 7 }]),
        },
        &[GitIdentity { forge: 1, credential: 1 }],
        &configured()[..1],
        false,
    )
    .expect("valid connector")
}
fn adopted() -> Repository {
    Repository {
        workflows: Box::default(),
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
/// A commit status write: a set, made without a find.
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

fn change_row() -> ChangeRow {
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
            early: change::Early::None,
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
        goal: None,
        ending: None,
        repair_tasks: Box::new([]),
        resolution_tasks: Box::new([]),
        result: ChangeResult::empty(),
        gate_remarks: Box::new([]),
        drift: None,
        base_repair: false,
        queue_repair: None,
        queue_repairs: 0,
        producing_charter: 2,
        review_charter: 4,
        producing_llm: None,
        review_llm: None,
        foreman_charter: 6,
        questions: temper_engine_domain_forge_change::Questions::default(),
        effect_account: None,
        producer_failure: None,
        report: Box::new([]),
        repairs: None,
        catch_up: None,
        close: None,
        merge_result: None,
    }
}

// Distinct non-wire identifiers make accidental disclosure visible in bytes and decimal text.
const GOAL: u64 = 0x5EC0_0000_0000_0100;
const CHANGE: u64 = 0x5EC0_0000_0000_0101;
const SECOND: u64 = 0x5EC0_0000_0000_0102;
const QUEUED: u64 = 0x5EC0_0000_0000_0103;
const LANDING: u64 = 0x5EC0_0000_0000_0110;
const OPEN: u64 = 11;
const HEAD: [u8; 32] = [3; 32];
const BASE: [u8; 32] = [4; 32];
const CANDIDATE: [u8; 32] = [5; 32];
const MERGE: [u8; 32] = [6; 32];
const RUN: &[u8] = b"https://forge.example/org/repo/actions/runs/7";
const MARKERS: [&[u8]; 7] = [
    b"secret-branch",
    b"secret-title",
    b"secret-body",
    b"secret-gate",
    b"secret-reason",
    b"secret-tier",
    b"secret-question",
];
use skein_lib::{List, Reader};
use temper_client_protocol_wire::public_v1 as w;
use temper_engine_domain_forge_change as c;
use temper_engine_domain_forge_landing as policy;
fn handed() -> [PublicTask; 4] {
    [
        PublicTask { task: GOAL, goal: Some(GOAL) },
        PublicTask { task: CHANGE, goal: Some(GOAL) },
        PublicTask { task: SECOND, goal: Some(GOAL) },
        PublicTask { task: QUEUED, goal: Some(GOAL) },
    ]
}
fn ask(kind: u16, task: Option<u64>, public: &[PublicTask]) -> PublicAsk<'_> {
    PublicAsk { kind, version: w::PUBLIC_VERSION, project: 5, task, public, now: env().wall }
}
fn member(task: u64) -> policy::Member {
    policy::Member {
        change: task,
        repository: policy::Repository { forge: REPO.forge, repository: REPO.repository },
        pull: 1,
        head: Some(HEAD),
        base: Some(BASE),
        candidate: Some(CANDIDATE),
        landed: Some(MERGE),
        gates: Box::new([policy::GateProof {
            proof: policy::Proof::Failed { key: [7; 32] },
            posted: policy::Posted::Verdict,
            faults: 1,
        }]),
        asked: None,
    }
}
fn setup() -> Domain {
    setup_with_limits(&LIMITS)
}
fn setup_with_limits(limits: &Limits) -> Domain {
    let mut d = domain_with_limits(limits);
    d.repositories.insert(REPO, adopted()).expect("repository");
    for task in [CHANGE, SECOND, QUEUED] {
        let mut row = change_row();
        row.task = task;
        row.change.task = task;
        row.pull = Some(1);
        row.branch = Box::from(MARKERS[0]);
        row.title = Box::from(MARKERS[1]);
        row.body = Box::from(MARKERS[2]);
        row.change.gates = Box::new([c::Gate {
            number: 9,
            kind: c::GateKind::Agent,
            blocking: true,
            freshness: c::Freshness::Clean,
            eager: false,
        }]);
        row.verdicts = Box::new([c::GateReport { number: 9, head: HEAD, status: c::Status::Failed }]);
        row.change.repairs = 2;
        if task == QUEUED {
            row.change.state = c::State::Queued { head: HEAD, ready_since: Wall::from_nanos(90_000_000_000) };
        }
        d.changes.insert(task, row).expect("change");
    }
    d.landing
        .projects
        .insert(
            5,
            (
                c::Way::Turn,
                Box::new([RepositoryGates {
                    repository: REPO,
                    gates: Box::new([policy::Gate {
                        name: Box::from(MARKERS[3]),
                        workflow: Box::from(b"gate.yml".as_slice()),
                        ignored: Box::new([]),
                        blocking: true,
                    }]),
                    tiers: Box::new([]),
                }]),
                AfterLanding {
                    verdict: temper_engine_domain_forge_tiers::Settings {
                        retries: 2,
                        suspects: temper_engine_domain_forge_tiers::Suspects::Bisect,
                    },
                    bound: Duration::from_secs(30),
                },
                None,
                None,
            ),
        )
        .expect("project");
    d.landing
        .rows
        .insert(
            LANDING,
            LandingRow {
                landing: policy::Landing {
                    task: LANDING,
                    project: 5,
                    phase: policy::Phase::Turn { at: env().wall },
                    members: Box::new([member(CHANGE), member(SECOND)]),
                    preparation: 2,
                    stale_proofs: 3,
                },
                asks: Box::new([None, None]),
                entries: Box::new([]),
            },
        )
        .expect("landing");
    d.landing
        .receipts
        .insert(
            ReceiptKey { project: 5, repository: REPO, check: Check::Gate(Box::from(MARKERS[3])), key: [7; 32] },
            Receipt {
                failures: None,
                passed: false,
                run: ReceiptRun { id: 7, number: 7, url: Box::from(RUN) },
                candidate: CANDIDATE,
                ended: env().wall,
            },
        )
        .expect("receipt");
    d
}
fn made(d: &Domain, kind: u16, task: Option<u64>, public: &[PublicTask]) -> PublicMade {
    d.public(&ask(kind, task, public), &LIMITS).expect("public answer")
}
fn change(d: &Domain, public: &[PublicTask]) -> w::PublicChange {
    let m = made(d, w::PUBLIC_CHANGE, Some(CHANGE), public);
    w::PublicChange::decode(&w::CEILINGS, &mut Reader::new(&m.record)).expect("change")
}
fn landings(d: &Domain, public: &[PublicTask]) -> w::PublicLandings {
    let m = made(d, w::PUBLIC_LANDINGS, None, public);
    w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&m.record)).expect("landings")
}
fn list<T, const N: usize>(items: [T; N]) -> List<T> {
    let mut list = List::with_capacity(u32::try_from(N).expect("fixture list"));
    for item in items {
        list.push(item).ok().expect("fixture room");
    }
    list
}
fn used(used: u32) -> w::PublicUsed {
    w::PublicUsed::new(&w::CEILINGS, w::PublicUsedParts { used, limit: 3 }).expect("used")
}
fn expected_change() -> w::PublicChange {
    w::PublicChange::new(
        &w::CEILINGS,
        w::PublicChangeParts {
            task: 0,
            head: Some(HEAD),
            step: w::PublicStep::Opening,
            repair: None,
            merge: None,
            tiers: 0,
            hold: None,
            ci: w::PublicCi::Unknown,
            gates: list([w::PublicGate::new(
                &w::CEILINGS,
                w::PublicGateParts {
                    number: 9,
                    kind: w::PublicGateKind::Review,
                    blocks: true,
                    verdict: w::PublicVerdict::Changes,
                },
            )
            .expect("gate")]),
            repairs: used(2),
            resolutions: used(0),
            updates: used(0),
            way: w::PublicWay::Queue,
            in_landing: true,
        },
    )
    .expect("expected change")
}
fn expected_landings() -> w::PublicLandings {
    let l = &w::CEILINGS;
    w::PublicLandings::new(
        l,
        w::PublicLandingsParts {
            turn: Some(0),
            landings: list([w::PublicLandingLine::new(
                l,
                w::PublicLandingLineParts {
                    members: list([0, 1]),
                    phase: w::PublicLandingPhase::Turn,
                    landed: list([MERGE, MERGE]),
                },
            )
            .expect("line")]),
            queues: list([w::PublicQueue::new(
                l,
                w::PublicQueueParts {
                    changes: list([w::PublicQueued::new(
                        l,
                        w::PublicQueuedParts { task: 2, goal: 3, ready_since: 90 },
                    )
                    .expect("queued")]),
                },
            )
            .expect("queue")]),
        },
    )
    .expect("expected landings")
}
fn expected_landing() -> w::PublicLanding {
    let l = &w::CEILINGS;
    let mut members = List::with_capacity(2);
    for task in [0, 1] {
        members
            .push(
                w::PublicMember::new(
                    l,
                    w::PublicMemberParts {
                        task,

                        base: Some(BASE),
                        candidate: Some(CANDIDATE),
                        landed: Some(MERGE),
                        gates: list([w::PublicGateProof::new(
                            l,
                            w::PublicGateProofParts {
                                number: 0,
                                kind: w::PublicGateKind::Check,
                                blocks: true,
                                proof: w::PublicProof::Failed,
                            },
                        )
                        .expect("proof")]),
                    },
                )
                .expect("member"),
            )
            .expect("members");
    }
    w::PublicLanding::new(
        l,
        w::PublicLandingParts {
            phase: w::PublicLandingPhase::Turn,
            hold: None,
            preparation: 2,
            members,
            stale_proofs: 3,
            holds_turn: true,
        },
    )
    .expect("expected landing")
}
#[test]
fn each_repair_reason_projects_to_the_public_step() {
    for (why, expected) in [
        (c::Repair::Ci, w::PublicRepair::Ci),
        (c::Repair::Semantic, w::PublicRepair::Semantic),
        (c::Repair::Review { head: [7; 32] }, w::PublicRepair::Review),
    ] {
        let mut d = setup();
        d.changes.get_mut(&CHANGE).expect("change").change.state = c::State::Repairing { requested: true, why };
        let got = change(&d, &handed());
        assert_eq!(*got.step(), w::PublicStep::Repairing);
        assert_eq!(*got.repair(), Some(expected));
    }
}

#[test]
fn each_kind_projects_from_the_source_table() {
    let d = setup();
    let public = handed();
    assert_eq!(change(&d, &public), expected_change());
    let m = made(&d, w::PUBLIC_CHANGE, Some(CHANGE), &public);
    assert_eq!(m.tasks.as_ref(), &[CHANGE]);
    let m = made(&d, w::PUBLIC_LANDINGS, None, &public);
    assert_eq!(m.tasks.as_ref(), &[CHANGE, SECOND, QUEUED, GOAL]);
    assert_eq!(
        w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&m.record)).expect("decode"),
        expected_landings()
    );
    for task in [CHANGE, SECOND] {
        let m = made(&d, w::PUBLIC_LANDING, Some(task), &public);
        assert_eq!(m.tasks.as_ref(), &[CHANGE, SECOND]);
        assert_eq!(
            w::PublicLanding::decode(&w::CEILINGS, &mut Reader::new(&m.record)).expect("decode"),
            expected_landing()
        );
    }
}
#[test]
fn the_meaning_check_catches_a_field_from_the_wrong_source() {
    let d = setup();
    let public = handed();
    let got = change(&d, &public);
    let mut expected = expected_change().into_parts();
    expected.head = Some(BASE);
    assert_ne!(got, w::PublicChange::new(&w::CEILINGS, expected).expect("mutation"));
    let mut expected = expected_change().into_parts();
    expected.task = 1;
    assert_ne!(got, w::PublicChange::new(&w::CEILINGS, expected).expect("mutation"));
}
#[test]
fn forbidden_sources_never_reach_the_bytes() {
    let mut d = setup();
    let public = handed();
    install_log(&mut d, Box::new([logged(MERGE, 100, &[QUEUED])]));
    let row = d.changes.get_mut(&CHANGE).expect("change");
    row.producer_failure = Some(Box::from(MARKERS[4]));
    row.questions.open = Box::new([c::Open {
        asker: SECOND,
        number: 1,
        line: Box::from(MARKERS[6]),
        words: Box::from(MARKERS[6]),
        stands: c::Stands::Taken,
    }]);
    row.change.state = c::State::Held {
        was: Box::new(c::State::Red { merge: MERGE, since: env().wall }),
        why: c::Hold::Authority(c::AuthorityHold {
            recovery: c::AuthorityRecovery { task: CHANGE, spec: SECOND, hold: GOAL, tasks: QUEUED },
            lack: c::AuthorityLack::Tasks,
            had: LANDING,
            released: false,
        }),
    };
    d.verifications
        .insert(
            (REPO, MERGE),
            Verification {
                repository: REPO,
                commit: MERGE,
                changes: Box::new([CHANGE]),
                tiers: Box::new([TierVerification { tier: Box::from(MARKERS[5]), verdict: None, repaired: None }]),
            },
        )
        .expect("verification");
    for (kind, task) in [
        (w::PUBLIC_CHANGE, Some(CHANGE)),
        (w::PUBLIC_LANDINGS, None),
        (w::PUBLIC_LANDING, Some(SECOND)),
        (w::PUBLIC_LANDING, Some(QUEUED)),
    ] {
        let m = made(&d, kind, task, &public);
        for marker in MARKERS {
            assert!(skein_lib::bytes::find(&m.record, marker).is_none(), "{kind}: {marker:?}");
        }
        for task in [GOAL, CHANGE, SECOND, QUEUED, LANDING, LOGGED_PULL] {
            assert!(skein_lib::bytes::find(&m.record, &task.to_be_bytes()).is_none(), "{kind}: big-endian task");
            assert!(skein_lib::bytes::find(&m.record, &task.to_le_bytes()).is_none(), "{kind}: internal task");
            let decimal = skein_lib::Decimal::of(task);
            assert!(skein_lib::bytes::find(&m.record, decimal.as_bytes()).is_none(), "{kind}: decimal task");
        }
    }
}
#[test]
fn a_landing_with_a_member_not_public_is_absent_and_unknown() {
    let d = setup();
    let public = [handed()[0], handed()[1], handed()[3]];
    assert!(landings(&d, &public).landings().is_empty());
    assert_eq!(landings(&d, &public).turn(), &None);
    for task in [CHANGE, SECOND] {
        assert!(d.public(&ask(w::PUBLIC_LANDING, Some(task), &public), &LIMITS).is_none());
    }
    assert!(!change(&d, &public).in_landing());
}
#[test]
fn a_task_not_handed_is_never_named() {
    let d = setup();
    let public = &handed()[1..];
    assert!(landings(&d, public).queues().is_empty());
    for kind in [w::PUBLIC_CHANGE, w::PUBLIC_LANDING] {
        assert!(d.public(&ask(kind, Some(CHANGE), &[]), &LIMITS).is_none());
    }
}
#[test]
fn filtered_before_ordered_or_counted() {
    let mut d = setup();
    let public = handed();
    let mut private = d.changes.get(&QUEUED).expect("queue").clone();
    private.task = GOAL.checked_sub(1).expect("fixture task");
    private.change.task = private.task;
    private.priority = -10_i32;
    private.change.state = c::State::Queued { head: HEAD, ready_since: Wall::from_nanos(1) };
    d.changes.insert(private.task, private).expect("private room");
    assert_eq!(landings(&d, &public), expected_landings());
    assert_eq!(landings(&d, &public).landings().as_slice().first().expect("landing").members().len(), 2);
}

#[test]
fn the_public_queue_follows_aging_and_uses_only_public_ties() {
    let mut d = setup();
    let public = handed();
    let row = d.changes.get_mut(&SECOND).expect("second");
    row.priority = 0_i32;
    row.change.state = c::State::Queued { head: HEAD, ready_since: Wall::from_nanos(95_000_000_000) };
    let early = made(&d, w::PUBLIC_LANDINGS, None, &public);
    let early = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&early.record)).expect("early");
    assert_eq!(early.queues().get(0).expect("queue").changes().get(0).expect("first").task(), 1);
    let mut late = ask(w::PUBLIC_LANDINGS, None, &public);
    late.now = Wall::from_nanos(121_000_000_000);
    let late = d.public(&late, &LIMITS).expect("late");
    let late = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&late.record)).expect("late record");
    assert_eq!(late.queues().get(0).expect("queue").changes().get(0).expect("first").task(), 2);
}
#[test]
fn the_largest_answers_fit_their_declared_bounds() {
    let (mut d, limits, public) = maximal();
    let bounds = temper_client_protocol_wire::Bounds {
        segments: 3,
        segment_bytes: 8,
        gates: 2,
        approvals: 1,
        field_bytes: 16,
        checks: 2,
        contexts: 2,
        refusal_bytes: 64,
    };
    let declared = temper_client_protocol_wire::public_bounds(&bounds);
    assert_eq!(declared, [822, 67_405, 1562]);
    for ((kind, task), bound) in
        [(w::PUBLIC_CHANGE, Some(CHANGE)), (w::PUBLIC_LANDINGS, None), (w::PUBLIC_LANDING, Some(CHANGE))]
            .into_iter()
            .zip(declared)
    {
        let made = d.public(&ask(kind, task, &public), &limits).expect("answer at bounds");
        assert!(made.record.len() <= usize::try_from(bound).expect("bound"));
    }
    let made = d.public(&ask(w::PUBLIC_LANDINGS, None, &public), &limits).expect("bounded list");
    let record = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("bounded record");
    assert_eq!(record.landings().len(), 64);
    assert_eq!(record.landings().get(0).expect("line").members().len(), 8);
    assert_eq!(record.queues().get(0).expect("queue").changes().len(), 64);
    for offset in 8..72_u64 {
        let task = CHANGE.checked_add(offset).expect("task");
        d.changes.get_mut(&task).expect("queued").base = Box::from(skein_lib::Decimal::of(offset).as_bytes());
    }
    let made = d.public(&ask(w::PUBLIC_LANDINGS, None, &public), &limits).expect("maximum queues");
    assert!(made.record.len() <= usize::try_from(declared[1]).expect("list bound"));
    let record = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("queues");
    assert_eq!(record.queues().len(), 64);
    let row = d.changes.get_mut(&CHANGE).expect("change");
    let mut gates = List::with_capacity(65);
    for gate in &row.change.gates {
        gates.push(*gate).expect("gate");
    }
    gates
        .push(c::Gate {
            number: 65,
            kind: c::GateKind::Agent,
            blocking: true,
            freshness: c::Freshness::Clean,
            eager: false,
        })
        .expect("extra");
    row.change.gates = gates.into_boxed();
    assert!(d.public(&ask(w::PUBLIC_CHANGE, Some(CHANGE), &public), &limits).is_none());
}

#[test]
fn the_newest_landing_and_a_pin_only_member_obey_the_same_filter() {
    let mut d = setup();
    let public = handed();
    install_log(&mut d, Box::new([logged(MERGE, 100, &[CHANGE])]));
    let mut newer = d.landing.rows.get(&LANDING).expect("landing").clone();
    newer.landing.task = LANDING.checked_add(1).expect("newer");
    newer.landing.members.get_mut(1).expect("member").change = 0;
    d.landing.rows.insert(newer.landing.task, newer).expect("newer room");
    assert!(!change(&d, &public).in_landing());
    assert!(d.public(&ask(w::PUBLIC_LANDING, Some(CHANGE), &public), &LIMITS).is_none());
    assert_eq!(landings(&d, &public).landings().len(), 2);
}

fn maximal_proofs(d: &mut Domain) {
    let mut proof_gates = List::with_capacity(8);
    for number in 0..8_u64 {
        let name: Box<[u8]> = Box::from(skein_lib::Decimal::of(number).as_bytes());
        proof_gates
            .push(policy::Gate {
                name: name.clone(),
                workflow: Box::from(b"gate.yml".as_slice()),
                ignored: Box::new([]),
                blocking: true,
            })
            .expect("gate");
        d.landing
            .receipts
            .insert(
                ReceiptKey { project: 5, repository: REPO, check: Check::Gate(name), key: [7; 32] },
                Receipt {
                    failures: None,
                    passed: false,
                    run: ReceiptRun { id: 7, number: 7, url: Box::from(RUN) },
                    candidate: CANDIDATE,
                    ended: env().wall,
                },
            )
            .expect("receipt");
    }
    d.landing.projects.get_mut(&5).expect("project").1.get_mut(0).expect("repo").gates = proof_gates.into_boxed();
}
fn maximal() -> (Domain, Limits, Box<[PublicTask]>) {
    // The panels fix 64 landings, 8 members, 8 proofs, 64 change gates and 64 queued changes.
    let limits = Limits { changes: 72, landings: 65, landing_members: 8, gates: 8, ..LIMITS };
    let mut d = setup_with_limits(&limits);
    maximal_proofs(&mut d);
    for task in [CHANGE, SECOND, QUEUED] {
        d.changes.remove(&task);
    }
    d.landing.rows.remove(&LANDING);
    let mut public = List::with_capacity(73);
    public.push(PublicTask { task: GOAL, goal: Some(GOAL) }).expect("goal");
    let mut members = List::with_capacity(8);
    let mut gates = List::with_capacity(64);
    for number in 0..64 {
        gates
            .push(c::Gate {
                number,
                kind: c::GateKind::Agent,
                blocking: true,
                freshness: c::Freshness::Clean,
                eager: false,
            })
            .expect("gate");
    }
    for offset in 0..72_u64 {
        let task = CHANGE.checked_add(offset).expect("fixture task");
        public.push(PublicTask { task, goal: Some(GOAL) }).expect("public");
        let mut row = change_row();
        row.task = task;
        row.change.task = task;
        row.pull = Some(u64::MAX);
        row.change.gates = gates.to_boxed();
        if offset < 8 {
            let mut member = member(task);
            member.gates = Box::new(
                [policy::GateProof {
                    proof: policy::Proof::Failed { key: [7; 32] },
                    posted: policy::Posted::Verdict,
                    faults: u32::MAX,
                }; 8],
            );
            members.push(member).expect("member");
        } else {
            row.change.state = c::State::Queued { head: HEAD, ready_since: env().wall };
        }
        d.changes.insert(task, row).expect("change");
    }
    let members = members.into_boxed();
    for offset in 0..65_u64 {
        let task = LANDING.checked_add(10_000).expect("landing task").checked_add(offset).expect("fixture task");
        d.landing
            .rows
            .insert(
                task,
                LandingRow {
                    landing: policy::Landing {
                        task,
                        project: 5,
                        phase: policy::Phase::Landed,
                        members: members.clone(),
                        preparation: u16::MAX,
                        stale_proofs: u32::MAX,
                    },
                    asks: Box::new([None; 8]),
                    entries: Box::new([]),
                },
            )
            .expect("landing");
    }
    // The newest private landing must be filtered before the 64-row panel bound.
    let private = LANDING.checked_add(10_064).expect("private landing");
    d.landing.rows.get_mut(&private).expect("private row").landing.members = Box::new([member(0)]);
    let public = public.into_boxed();
    (d, limits, public)
}

// Logged words and pull numbers remain private, independently of the change row.
const LOGGED_PULL: u64 = 0x7351_2034_5678_9ABC;
fn logged(commit: [u8; 32], at: u64, tasks: &[u64]) -> LoggedLanding {
    let mut changes = List::with_capacity(u32::try_from(tasks.len()).expect("fixture changes"));
    for task in tasks {
        changes
            .push(LoggedChange { task: *task, pull: Some(LOGGED_PULL), title: Box::from(MARKERS[1]) })
            .expect("logged change");
    }
    LoggedLanding { commit, branch: Box::from(MARKERS[0]), at: Wall::from_nanos(at), changes: changes.into_boxed() }
}
fn install_log(d: &mut Domain, landings: Box<[LoggedLanding]>) {
    d.landing_logs.insert(REPO, LandingLog { repository: REPO, landings }).expect("log");
}
fn queue_history() -> Domain {
    let mut d = setup();
    d.landing.rows.remove(&LANDING);
    for task in [CHANGE, SECOND, QUEUED] {
        d.changes.get_mut(&task).expect("change").change.state = c::State::Cleared { merge: MERGE };
    }
    d
}
fn ended_public() -> [PublicTask; 2] {
    [PublicTask { task: CHANGE, goal: None }, PublicTask { task: SECOND, goal: None }]
}
fn expected_merge_line(members: List<u16>, commit: [u8; 32]) -> w::PublicLandingLine {
    w::PublicLandingLine::new(
        &w::CEILINGS,
        w::PublicLandingLineParts { members, phase: w::PublicLandingPhase::Landed, landed: list([commit]) },
    )
    .expect("merge line")
}
fn expected_merge(tasks: &[u16]) -> w::PublicLanding {
    let mut members = List::with_capacity(u32::try_from(tasks.len()).expect("fixture members"));
    for task in tasks {
        members
            .push(
                w::PublicMember::new(
                    &w::CEILINGS,
                    w::PublicMemberParts {
                        task: *task,
                        base: None,
                        candidate: None,
                        landed: Some(MERGE),
                        gates: list([]),
                    },
                )
                .expect("merge member"),
            )
            .expect("member room");
    }
    w::PublicLanding::new(
        &w::CEILINGS,
        w::PublicLandingParts {
            phase: w::PublicLandingPhase::Landed,
            hold: None,
            preparation: 0,
            members,
            stale_proofs: 0,
            holds_turn: false,
        },
    )
    .expect("merge landing")
}
#[test]
fn a_queue_merge_is_a_landed_line_newest_first() {
    let mut d = queue_history();
    let public = ended_public();
    install_log(&mut d, Box::new([logged(MERGE, 50, &[CHANGE]), logged(HEAD, 100, &[SECOND])]));
    let made = made(&d, w::PUBLIC_LANDINGS, None, &public);
    assert_eq!(made.tasks.as_ref(), &[SECOND, CHANGE]);
    let expected = w::PublicLandings::new(
        &w::CEILINGS,
        w::PublicLandingsParts {
            turn: None,
            landings: list([expected_merge_line(list([0]), HEAD), expected_merge_line(list([1]), MERGE)]),
            queues: list([]),
        },
    )
    .expect("queue history");
    assert_eq!(landings(&d, &public), expected);
}
#[test]
fn a_queue_merge_reads_as_its_landing() {
    let public = ended_public();
    for (tasks, places) in [(&[CHANGE][..], &[0][..]), (&[CHANGE, SECOND][..], &[0, 1][..])] {
        let mut d = queue_history();
        install_log(&mut d, Box::new([logged(MERGE, 100, tasks)]));
        for task in tasks {
            let made = made(&d, w::PUBLIC_LANDING, Some(*task), &public);
            assert_eq!(made.tasks.as_ref(), tasks);
            let record = w::PublicLanding::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("merge");
            assert_eq!(record, expected_merge(places));
        }
        assert_eq!(landings(&d, &public).landings(), &list([expected_merge_line(list_from(places), MERGE)]));
        assert!(!change(&d, &public).in_landing());
    }
}
fn list_from(items: &[u16]) -> List<u16> {
    let mut list = List::with_capacity(u32::try_from(items.len()).expect("fixture list"));
    for item in items {
        list.push(*item).expect("fixture room");
    }
    list
}
#[test]
fn a_logged_landing_not_wholly_public_is_absent_and_unknown() {
    let other = skein_forge::api::Repository { forge: REPO.forge, repository: 3 };
    let public = ended_public();
    for (handed, repository, way, tasks) in [
        (&public[..1], REPO, c::Way::Queue, &[CHANGE, SECOND][..]),
        (&public[..], other, c::Way::Queue, &[CHANGE, SECOND][..]),
        (&public[..], REPO, c::Way::Turn, &[CHANGE, SECOND][..]),
        (&public[..], REPO, c::Way::Queue, &[][..]),
    ] {
        let mut d = queue_history();
        let mut foreign = adopted();
        foreign.project = 6;
        foreign.provider = other;
        d.repositories.insert(other, foreign).expect("other project");
        let row = d.changes.get_mut(&SECOND).expect("second");
        row.repository = repository;
        row.change.way = way;
        install_log(&mut d, Box::new([logged(MERGE, 100, tasks)]));
        let made = made(&d, w::PUBLIC_LANDINGS, None, handed);
        assert!(made.tasks.is_empty(), "{handed:?} {repository:?} {way:?} {tasks:?}");
        assert!(landings(&d, handed).landings().is_empty());
        for task in [CHANGE, SECOND] {
            assert!(d.public(&ask(w::PUBLIC_LANDING, Some(task), handed), &LIMITS).is_none());
        }
    }
}
#[test]
fn landings_by_turn_stand_before_queue_merges_within_the_bound() {
    // One turn landing leaves 63 places in the public panel's 64-line bound.
    let limits = Limits { landings_kept: 65, ..LIMITS };
    let mut d = setup_with_limits(&limits);
    let mut logs = List::with_capacity(limits.landings_kept);
    for at in 0..65_u8 {
        logs.push(logged([at; 32], u64::from(at), &[QUEUED])).expect("log room");
    }
    install_log(&mut d, logs.into_boxed());
    let mut public = handed();
    public[3].goal = None;
    let made = d.public(&ask(w::PUBLIC_LANDINGS, None, &public), &limits).expect("bounded history");
    assert_eq!(made.tasks.as_ref(), &[CHANGE, SECOND, QUEUED]);
    let mut expected = List::with_capacity(w::CEILINGS.public_landings_landings);
    expected.push(expected_landings().landings().get(0).expect("turn line").clone()).expect("turn room");
    for at in (2..65_u8).rev() {
        expected.push(expected_merge_line(list([2]), [at; 32])).expect("merge room");
    }
    let expected = w::PublicLandings::new(
        &w::CEILINGS,
        w::PublicLandingsParts { turn: Some(0), landings: expected, queues: list([]) },
    )
    .expect("bounded landings");
    let record = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("history");
    assert_eq!(record, expected);
}
