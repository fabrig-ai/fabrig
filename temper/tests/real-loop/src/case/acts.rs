//! The person's acts, with documents identified by the moment they were read.
use super::{DIRECTOR, PERSON, PROJECT};
use crate::{
    argv,
    calls::{self, D1, join, number, sign_in, start},
};

fn until(at: calls::Dir, argv: calls::Argv, holds: &'static [u8], early: &'static [u8]) -> calls::Step {
    calls::until(at, argv, holds, early).whole_activity()
}

fn call(at: calls::Dir, argv: calls::Argv) -> calls::Step {
    calls::call(at, argv).whole_activity()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Read {
    H1Held,
    H1ReviewerHeld,
    BHeld,
    H1Ended,
    DirectorEnded,
    AEnded,
    DirectorActivity,
    DirectorTimer,
    BenchmarkWaiting,
    BenchmarkEnded,
    BEnded,
    ShapeEnded,
    MoverEnded,
    X3Produced,
    X3Ended,
    StoryLanded,
    DirectionBHeld,
    DirectionBPending,
    DirectionBEnded,
    // 6.4
    RedChangeEnded,
    RedReviewEnded,
    RedRepairEnded,
    RedRepairCoderEnded,
    RedRepairReviewEnded,
    RedRootsPending,
    RedUpdatedReviewEnded,
    QaSubownerEnded,
    QaTesterEnded,
    QaChangeEnded,
    QaCoderEnded,
    QaReviewerEnded,
}
// Provisioning's key call does not count as a story act. The director's
// ended wait starts first, before its delegation replies, then is joined.
const DIRECTOR_ENDED: usize = 3;
// Sign-in, adoption, chat, the live ended watch and the placement barrier.
const OPENER_CALLS: usize = 5;

#[must_use]
pub fn step_4_1() -> Vec<calls::Step> {
    vec![
        sign_in(D1),
        // Leave out the goal's issue: recovery of its edit holds a closing goal
        // (M1's finding; forge_route.rs project_goal projects only in a home).
        call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--no-home",
                "--key",
                "adopt-kit"
            ],
        ),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "schedule", PERSON]),
        // Keep these watches live: a fresh wait on an ended delegate exits
        // unknown. jig's crates/jig-domain-core/src/routing/views.rs:304 maps
        // documents/small.rs:23-28's missing/ended live row to that refusal.
        start(D1, argv!["task", "show", number(DIRECTOR), "--until", "ended", "--json"]),
        call(D1, argv!["task", "show", number(DIRECTOR), "--until", "replied"]),
        join(DIRECTOR_ENDED),
    ]
}

// Call positions within the signed-in check suffix; non-call steps consume none.
const CHECK_DIRECTOR_ENDED: usize = 1;
const CHECK_BENCHMARK_WAITING: usize = 4;
const CHECK_BENCHMARK_ENDED: usize = 5;
const CHECK_DIRECTOR_ACTIVITY: usize = 6;
const CHECK_DIRECTOR_TIMER: usize = 7;
pub const READS_4_3: &[(Read, usize)] = &[
    (Read::BenchmarkWaiting, CHECK_BENCHMARK_WAITING),
    (Read::BenchmarkEnded, CHECK_BENCHMARK_ENDED),
    (Read::DirectorActivity, CHECK_DIRECTOR_ACTIVITY),
    (Read::DirectorTimer, CHECK_DIRECTOR_TIMER),
];

/// The check follows the opener's sign-in; `base` is its first recorded call.
#[must_use]
pub fn step_4_3(base: usize) -> Vec<calls::Step> {
    use super::{B, BENCHMARK, CHECK, CHECK_PERSON};
    vec![
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "check", CHECK_PERSON]),
        start(D1, argv!["task", "show", number(CHECK), "--until", "ended", "--json"]),
        calls::watching(base + CHECK_DIRECTOR_ENDED),
        // Each replied wait follows a completed delegation, admitting the
        // next task's watch without racing a not-yet-created task number.
        call(D1, argv!["task", "show", number(CHECK), "--until", "replied"]),
        call(D1, argv!["task", "show", number(B), "--until", "replied"]),
        call(D1, argv!["task", "show", number(BENCHMARK), "--until", "phase:due", "--json"]),
        calls::add_worker("s2"),
        join(base + CHECK_DIRECTOR_ENDED),
        call(D1, argv!["task", "show", number(BENCHMARK), "--activity", "32", "--json"]),
        call(D1, argv!["task", "show", number(CHECK), "--activity", "32", "--json"]),
        // The latest page starts after the timer: follow its supplied cursor.
        call(
            D1,
            argv![
                "task",
                "activity",
                number(CHECK),
                "--before",
                calls::activity_before(base + CHECK_DIRECTOR_ACTIVITY),
                "--json"
            ],
        ),
    ]
}

/// Run the landing before the check in the signed-in directory.
#[must_use]
pub fn step_4_1_4_3() -> Vec<calls::Step> {
    let mut steps = step_4_1();
    // The fleet picks by workspace, free slots and channel order, not by name:
    // s1 links only once the director's waiting run holds ws's slot.
    let placed = steps
        .iter()
        .position(|step| {
            matches!(step, calls::Step::Call { argv, .. }
                if matches!(argv.0.as_slice(), [calls::Arg::Lit("task"), calls::Arg::Lit("show"),
                    calls::Arg::Number(DIRECTOR), calls::Arg::Lit("--until"), calls::Arg::Lit("replied")]))
        })
        .expect("director's reply wait")
        + 1;
    steps.insert(placed, calls::add_worker("s1"));
    let base = OPENER_CALLS;
    steps.extend(step_4_3(base));
    steps
}

#[must_use]
pub fn reads_4_1_4_3() -> Vec<(Read, usize)> {
    let base = OPENER_CALLS;
    READS_4_3.iter().map(|&(read, call)| (read, base + call)).collect()
}

const HANG_ENDED: usize = 2;
/// The restart seam is observed through worker lives and model input, not documents.
pub const READS_4_6: &[(Read, usize)] = &[];

/// Sign in and run the restart story; `base` names its first recorded call.
#[must_use]
pub fn step_4_6(base: usize) -> Vec<calls::Step> {
    use super::{HANG, HANG_PERSON, HANG_READ};
    vec![
        sign_in(D1),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "hang", HANG_PERSON]),
        start(D1, argv!["task", "show", number(HANG), "--until", "ended", "--json"]),
        calls::watching(base + HANG_ENDED),
        // ctl's predicates distinguish phase/replied, but cannot select a
        // committed turn or its text. Activity contains only committed turns;
        // chat/run can also contain the live writing, so cannot be this barrier.
        until(D1, argv!["task", "activity", number(HANG), "--json"], HANG_READ.as_bytes(), b"run_began"),
        // Once the read is committed, await the following in-run wait.
        // Its withheld turn is sealed when the worker stops, as A's parked waits are.
        call(D1, argv!["task", "show", number(HANG), "--until", "replied", "--json"]),
        calls::stop_worker("ws"),
        call(D1, argv!["inbox", "--until", "waiting"]),
        call(
            D1,
            argv![
                "task",
                "answer",
                number(HANG),
                number(super::PHASE_QUESTION_NUMBER),
                super::PHASE_ANSWER,
                "--key",
                "phase-answer"
            ],
        ),
        join(base + HANG_ENDED),
        // The ended chat is the cheapest event proving its retry cannot use ws.
        calls::start_worker("ws"),
    ]
}

// Act 2, step 5.1 (testing/reference-case.md)
const RULES_ENDED: usize = 3;
#[must_use]
pub fn step_5_1() -> Vec<calls::Step> {
    use super::{NEW_RULES, RULES, RULES_FILE, RULES_HOLD_REVISION, RULES_PERSON, SCOPE};
    vec![
        sign_in(D1),
        call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--role",
                "context",
                "--no-home",
                "--key",
                "rules-kit"
            ],
        ),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "rules", RULES_PERSON]),
        start(D1, argv!["task", "show", number(RULES), "--until", "ended", "--json"]),
        // The committed delegation creates A before its opening turn is watched.
        call(D1, argv!["task", "show", number(RULES), "--until", "replied"]),
        // A's completed opening run witnesses the old checkout before Stop.
        call(D1, argv!["task", "show", number(SCOPE), "--until", "ended"]),
        call(D1, argv!["task", "stop", number(RULES), "--key", "rules-stop"]),
        calls::push(RULES_FILE.as_bytes(), NEW_RULES.as_bytes()),
        call(
            D1,
            argv!["task", "release", number(RULES), "--hold", number(RULES_HOLD_REVISION), "--key", "rules-release"],
        ),
        join(RULES_ENDED),
    ]
}

// Act 2, step 5.3 (testing/reference-case.md)
const SHAPE_ENDED: usize = 3;
pub const READS_5_3: &[(Read, usize)] = &[(Read::MoverEnded, 6), (Read::ShapeEnded, SHAPE_ENDED)];

#[must_use]
pub fn step_5_3() -> Vec<calls::Step> {
    use super::{MOVER, SHAPE, SHAPE_PERSON};
    vec![
        sign_in(D1),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "shape", SHAPE_PERSON]),
        call(D1, argv!["task", "show", number(SHAPE), "--until", "replied"]),
        start(D1, argv!["task", "show", number(SHAPE), "--until", "ended", "--json"]),
        until(D1, argv!["task", "show", number(MOVER), "--json"], super::NEW_BRIEF.as_bytes(), b"\"task\""),
        call(
            D1,
            argv![
                "task",
                "answer",
                number(SHAPE),
                number(super::MOVER_QUESTION_NUMBER),
                "Continue with the changed instructions.",
                "--key",
                "mover-answer"
            ],
        ),
        call(D1, argv!["task", "show", number(MOVER), "--until", "ended", "--json"]),
        join(SHAPE_ENDED),
    ]
}

// Act 2, step 5.2 (testing/reference-case.md)
pub const READS_5_2: &[(Read, usize)] =
    &[(Read::BHeld, 9), (Read::H1Held, 10), (Read::H1ReviewerHeld, 11), (Read::H1Ended, 14)];

#[must_use]
pub fn step_5_2() -> Vec<calls::Step> {
    use super::{Agent, B_HOLD, BOWNER, H1, H1_REVIEWER, PAUSE, PAUSE_PERSON, PROJECT, REVIEW_HOLD};
    vec![
        sign_in(D1),
        call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--no-home",
                "--key",
                "adopt-kit"
            ],
        ),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "pause", PAUSE_PERSON]),
        call(D1, argv!["task", "show", number(PAUSE), "--until", "replied"]),
        call(D1, argv!["task", "show", number(BOWNER), "--until", "replied"]),
        until(D1, argv!["task", "show", number(H1), "--json"], Agent::Review.cue(), b"\"task\""),
        calls::writing_task(D1, H1_REVIEWER),
        call(D1, argv!["task", "show", number(BOWNER), "--until", "phase:idle"]),
        call(D1, argv!["task", "stop", number(BOWNER), "--key", "stop-b"]),
        until(D1, argv!["task", "show", number(BOWNER), "--json"], b"\"phase\":\"held\"", b"\"task\""),
        until(D1, argv!["task", "show", number(H1), "--json"], b"\"phase\":\"held\"", b"\"task\""),
        until(D1, argv!["task", "show", number(H1_REVIEWER), "--json"], b"\"phase\":\"held\"", b"\"task\""),
        // Drain the cancelled response before the new attempt uses the peer.
        calls::release_hold(REVIEW_HOLD),
        call(D1, argv!["task", "release", number(BOWNER), "--hold", number(B_HOLD), "--key", "release-b"]),
        calls::writing_task(D1, H1_REVIEWER),
        calls::release_hold(REVIEW_HOLD),
        until(D1, argv!["task", "show", number(H1), "--json"], b"\"phase\":\"ended\"", b"\"task\""),
        call(D1, argv!["task", "show", number(BOWNER), "--until", "ended"]),
    ]
}

// Act 2, step 5.4 (testing/reference-case.md)
pub const READS_5_4: &[(Read, usize)] = &[
    (Read::X3Produced, 7),
    (Read::DirectionBHeld, 12),
    (Read::X3Ended, 15),
    (Read::StoryLanded, 17),
    (Read::DirectionBPending, 18),
    (Read::DirectionBEnded, 22),
];
#[must_use]
pub fn step_5_4() -> Vec<calls::Step> {
    use super::{
        Agent, B_DELEGATE_HOLD, DIRECTION, DIRECTION_MESSAGE, DIRECTION_PERSON, OVERLAP_HOLD_REVISION, OVERLAP_OWNER,
        PLANNED, PLANNED_CODER, PLANNED_CODING_HOLD, STORY_CHANGE, STORY_OWNER, X3, X3_CODER, X3_REVIEW_HOLD,
        X3_REVIEWER,
    };
    vec![
        sign_in(D1),
        call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--no-home",
                "--key",
                "adopt-kit"
            ],
        ),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "direction", DIRECTION_PERSON]),
        call(D1, argv!["task", "show", number(DIRECTION), "--until", "replied"]),
        call(D1, argv!["task", "show", number(STORY_OWNER), "--until", "replied"]),
        until(D1, argv!["task", "show", number(X3), "--json"], Agent::Review.cue(), b"\"task\""),
        until(D1, argv!["task", "show", number(X3_REVIEWER), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        call(D1, argv!["task", "show", number(X3_CODER), "--json"]),
        calls::release_hold(B_DELEGATE_HOLD),
        until(D1, argv!["task", "show", number(OVERLAP_OWNER), "--json"], Agent::PlannedWriter.cue(), b"\"task\""),
        until(D1, argv!["task", "show", number(PLANNED_CODER), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        call(D1, argv!["task", "show", number(OVERLAP_OWNER), "--until", "phase:idle"]),
        call(D1, argv!["task", "stop", number(OVERLAP_OWNER), "--key", "stop-b"]),
        until(D1, argv!["task", "show", number(OVERLAP_OWNER), "--json"], b"\"phase\":\"held\"", b"\"task\""),
        call(D1, argv!["task", "message", number(STORY_OWNER), "--key", "new-direction", DIRECTION_MESSAGE]),
        call(D1, argv!["task", "show", number(X3), "--until", "phase:closing"]),
        calls::release_hold(X3_REVIEW_HOLD),
        call(D1, argv!["task", "show", number(X3), "--until", "ended", "--json"]),
        until(D1, argv!["task", "show", number(STORY_OWNER), "--json"], Agent::StoryWriter.cue(), b"\"task\""),
        call(D1, argv!["task", "show", number(STORY_CHANGE), "--until", "ended", "--json"]),
        call(D1, argv!["task", "show", number(OVERLAP_OWNER), "--json"]),
        call(
            D1,
            argv![
                "task",
                "release",
                number(OVERLAP_OWNER),
                "--hold",
                number(OVERLAP_HOLD_REVISION),
                "--key",
                "release-b"
            ],
        ),
        call(D1, argv!["task", "show", number(PLANNED), "--until", "phase:closing"]),
        calls::release_hold(PLANNED_CODING_HOLD),
        call(D1, argv!["task", "show", number(OVERLAP_OWNER), "--until", "ended"]),
        call(D1, argv!["task", "show", number(OVERLAP_OWNER), "--activity", "32", "--json"]),
        call(D1, argv!["task", "show", number(DIRECTION), "--until", "ended"]),
    ]
}

// Act 3, step 6.4 (testing/reference-case.md)
pub const READS_6_4: &[(Read, usize)] = &[
    (Read::RedRootsPending, 3),
    (Read::RedChangeEnded, 5),
    (Read::RedRepairEnded, 6),
    (Read::RedReviewEnded, 7),
    (Read::RedRepairCoderEnded, 8),
    (Read::RedRepairReviewEnded, 9),
    (Read::RedUpdatedReviewEnded, 10),
];
#[must_use]
pub fn step_6_4() -> Vec<calls::Step> {
    use super::{
        Agent, PROJECT, RED_BROKEN, RED_CHANGE, RED_FILE, RED_REPAIR, RED_REPAIR_CODER, RED_REPAIR_HOLD,
        RED_REPAIR_REVIEW, RED_REVIEW, RED_REVIEW_HOLD, RED_UPDATED_REVIEW,
    };
    let mut steps = vec![
        sign_in(D1),
        call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--no-home",
                "--key",
                "red-kit"
            ],
        ),
        call(
            D1,
            argv![
                "change",
                "open",
                "--project",
                number(PROJECT),
                "--repository",
                "acme/app",
                "--branch",
                "a4",
                "--title",
                "a4",
                "--budget",
                "0.06",
                "--key",
                "red-a4"
            ],
        ),
        calls::held(RED_REVIEW_HOLD),
        calls::push(RED_FILE.as_bytes(), RED_BROKEN.as_bytes()),
        // The first exact-head review waits until the person's red push.
        calls::release_hold(RED_REVIEW_HOLD),
        until(
            D1,
            argv!["task", "list", "--project", number(PROJECT), "--json"],
            Agent::RedRepairCoder.cue(),
            b"\"tasks\"",
        ),
        calls::held(RED_REPAIR_HOLD),
        calls::release_hold(RED_REPAIR_HOLD),
        call(D1, argv!["task", "show", number(RED_CHANGE), "--until", "ended"]),
    ];
    for task in [RED_CHANGE, RED_REPAIR, RED_REVIEW, RED_REPAIR_CODER, RED_REPAIR_REVIEW, RED_UPDATED_REVIEW] {
        steps.push(call(D1, argv!["task", "show", number(task), "--activity", "32", "--json"]));
    }
    steps
}

// Act 3, step 6.6 observes the forge, not task journals.
pub const READS_6_6: &[(Read, usize)] = &[];

#[must_use]
pub fn step_6_6() -> Vec<calls::Step> {
    use super::waiting::{E1, PERSON, REPIN, REPIN_HOLD, STARTER};
    let mut steps = vec![sign_in(D1)];
    for name in ["app", "kit"] {
        steps.push(call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                name,
                "--landing",
                "main",
                "--no-home",
                "--key",
                name
            ],
        ));
    }
    steps.extend([
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "waiting", PERSON]),
        call(D1, argv!["task", "show", number(STARTER), "--until", "replied"]),
        call(D1, argv!["task", "show", number(E1), "--until", "ended"]),
        calls::release_hold(REPIN_HOLD),
        call(D1, argv!["task", "show", number(REPIN), "--until", "ended"]),
        call(D1, argv!["task", "show", number(STARTER), "--until", "ended"]),
    ]);
    steps
}

// 6.8: only the tester-to-change landing evidence is read.
const QA_FINAL: usize = 4; // sign-in, adoption, chat start, director-ended wait.
pub const READS_6_8: &[(Read, usize)] = &[
    (Read::QaSubownerEnded, QA_FINAL),
    (Read::QaTesterEnded, QA_FINAL + 1),
    (Read::QaChangeEnded, QA_FINAL + 2),
    (Read::QaCoderEnded, QA_FINAL + 3),
    (Read::QaReviewerEnded, QA_FINAL + 4),
];

#[must_use]
pub fn step_6_8() -> Vec<calls::Step> {
    use super::qa::{A1, C, CODER, DIRECTOR, PERSON, REVIEWER, T};
    let mut steps = vec![
        sign_in(D1),
        call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--no-home",
                "--key",
                "qa-adopt"
            ],
        ),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "qa", PERSON]),
        call(D1, argv!["task", "show", number(DIRECTOR), "--until", "ended"]),
    ];
    for task in [A1, T, C, CODER, REVIEWER] {
        steps.push(call(D1, argv!["task", "show", number(task), "--activity", "32", "--json"]));
    }
    steps
}
