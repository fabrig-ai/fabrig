//! The person's acts, with documents identified by the moment they were read.
use super::{A, DESIGN, DIRECTOR, PERSON, PROJECT};
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
    BEndedEarlier,
    ImplementerRunning,
    ImplementerHeld,
    H1Held,
    H1ReviewerHeld,
    H1Running,
    HelperEnded,
    BHeld,
    BStillHeld,
    ImplementerEnded,
    H1Ended,
    Ended,
    PauseEnded,
    DirectorEnded,
    AEnded,
    DesignEnded,
    AWaiting,
    Refused,
    Limited,
    PaceAsked,
    Ranked,
    SurveyEnded,
    CheckEnded,
    DirectorActivity,
    DirectorTimer,
    BenchmarkWaiting,
    BenchmarkEnded,
    BEnded,
    RulesWaiting,
    RulesHeld,
    ScopeHeld,
    RulesQuestionWaiting,
    ScopeEnded,
    RulesEnded,
    /// The director's whole activity once ended: its runs' endings.
    RulesActivity,
    /// The first fresh director brief after Release, read from the scripted model's received system text.
    RulesFreshBrief,
    /// A's whole activity once ended: its runs' endings.
    ScopeActivity,
    ShapeEnded,
    MoverEnded,
    PlanAnswerPending,
    PlannerParked,
    PlannerEnded,
    PlannerEarlier,
    PlannerFirst,
    PlanEnded,
    SubownerEnded,
    X3Open,
    X3Produced,
    X3Ended,
    PlannedRunning,
    PlannedCoderRunning,
    PlannedHeld,
    PlannedCoderHeld,
    PlannedEnded,
    StoryLanded,
    DirectionBHeld,
    DirectionBPending,
    DirectionBEnded,
    DirectionBFirst,
    StoryOwnerEnded,
    DirectionEnded,
    StoryOwnerEarlier,
    StoryOwnerFirst,
    StoryOwnerInitial,
    DirectionNoteWritten,
    DirectionFirst,
    // Act 3, step 6.5 (testing/reference-case.md)
    BudgetAsked,
    BudgetInboxAnswered,
    BudgetMedium,
    BudgetHigh,
    BudgetA5Ended,
    BudgetAdvisorEnded,
    BudgetSubownerEnded,
    BudgetOwnerEnded,
    BudgetDirectorEnded,
    BudgetSubownerFirst,
    BudgetOwnerFirst,
    BudgetDirectorFirst,
    // 6.6
    WaitingE1Pending,
    WaitingRepinPending,
    WaitingReviewWriting,
    WaitingDirectorEnded,
    WaitingOwnerEnded,
    WaitingE1Ended,
    WaitingRepinEnded,
    WaitingE1CoderEnded,
    WaitingE1ReviewerEnded,
    WaitingRepinCoderEnded,
    WaitingRepinReviewerEnded,
    WaitingRootsEnded,
    // 6.4
    RedReviewWriting,
    RedChangeOpen,
    RedRepairWriting,
    RedRepairPending,
    RedTimerArmed,
    RedChangePaused,
    RedDirectorEnded,
    RedOwnerEnded,
    RedOwnerEarlier,
    RedChangeEnded,
    RedCoderEnded,
    RedReviewEnded,
    RedRepairEnded,
    RedRepairCoderEnded,
    RedRepairReviewEnded,
    RedRootsEnded,
    RedRootsPending,
    RedUpdatedReviewEnded,
    QaSubownerBefore,
    QaNextWriting,
    QaSubownerLive,
    QaTesterLive,
    QaDirectorEnded,
    QaOwnerEnded,
    QaSubownerEnded,
    QaTesterEnded,
    QaNextEnded,
    QaChangeEnded,
    QaCoderEnded,
    QaReviewerEnded,
    QaInboxEnded,
    QaRootsEnded,
}
// Provisioning's key call does not count as a story act. The director's
// ended wait starts first; its delegates' ended documents are read before
// their requesters end, then the director's wait is joined.
const DIRECTOR_ENDED: usize = 3;
const A_ENDED: usize = 9;
const DESIGN_ENDED: usize = 6;
pub const READS_4_1: &[(Read, usize)] =
    &[(Read::DirectorEnded, DIRECTOR_ENDED), (Read::AEnded, A_ENDED), (Read::DesignEnded, DESIGN_ENDED)];

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
        call(D1, argv!["task", "show", number(A), "--until", "replied"]),
        call(D1, argv!["task", "show", number(DESIGN), "--until", "ended", "--json"]),
        call(D1, argv!["inbox", "--until", "waiting"]),
        call(
            D1,
            argv![
                "task",
                "answer",
                number(DIRECTOR),
                number(super::QUESTION),
                super::GREEN_LIGHT,
                "--key",
                "schedule-answer"
            ],
        ),
        call(D1, argv!["task", "show", number(A), "--until", "ended", "--json"]),
        join(DIRECTOR_ENDED),
    ]
}

// Refusal keeps its phase wait and fresh read; the limit retains its matching read.
const HELD_A_ENDED: usize = 3;
const A_WAITING: usize = 5;
const HELD_ACCOUNTS: usize = 6;
const REFUSED_DIRECTOR_ENDED: usize = 8;
const LIMITED_A_WAITING: usize = 4;
const LIMITED_ACCOUNTS: usize = 5;
const LIMITED_DIRECTOR_ENDED: usize = 6;
pub const READS_4_5_REFUSED: &[(Read, usize)] = &[
    (Read::DirectorEnded, REFUSED_DIRECTOR_ENDED),
    (Read::AWaiting, A_WAITING),
    (Read::Refused, HELD_ACCOUNTS),
    (Read::AEnded, HELD_A_ENDED),
];
pub const READS_4_5_LIMITED: &[(Read, usize)] = &[
    (Read::DirectorEnded, LIMITED_DIRECTOR_ENDED),
    (Read::AWaiting, LIMITED_A_WAITING),
    (Read::Limited, LIMITED_ACCOUNTS),
    (Read::AEnded, HELD_A_ENDED),
];

#[must_use]
pub fn step_4_5_refused() -> Vec<calls::Step> {
    use super::{FIRST_ACCOUNT, SECOND_KEY_FILE};
    held(
        "refused",
        Some(call(D1, argv!["account", "replace-key", number(FIRST_ACCOUNT), calls::file(SECOND_KEY_FILE)])),
    )
}

#[must_use]
pub fn step_4_5_limited() -> Vec<calls::Step> {
    // Only reads and waits follow the chat: the engine lifts the limit.
    held("limited", None)
}

fn held(key: &'static str, act: Option<calls::Step>) -> Vec<calls::Step> {
    let mut steps = vec![
        sign_in(D1),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", key, super::HELD_PERSON]),
        call(D1, argv!["task", "show", number(DIRECTOR), "--until", "replied"]),
        // A fresh wait after the requester ends exits unknown: jig's
        // routing/views.rs admission and documents/small.rs require live rows.
        // Keep A's watch across the act until jig supports ended waits.
        start(D1, argv!["task", "show", number(A), "--until", "ended", "--json"]),
    ];
    if act.is_none() {
        steps.push(calls::until_matching(
            D1,
            argv!["task", "show", number(A), "--json"],
            calls::ReadCondition::LimitedTask { task: A, account: super::FIRST_ACCOUNT, source: 0 },
            b"\"phase\":\"ended\"",
        ));
    } else {
        steps.push(call(D1, argv!["task", "show", number(A), "--until", "phase:due"]));
        steps.push(call(D1, argv!["task", "show", number(A), "--json"]));
    }
    steps.push(call(D1, argv!["account", "list", "--json"]));
    steps.extend(act);
    steps.push(join(HELD_A_ENDED));
    // The director can already be ended too: a fresh wait exits unknown.
    steps.push(until(
        D1,
        argv!["task", "show", number(DIRECTOR), "--json"],
        b"\"phase\":\"ended\"",
        b"\"phase\":\"running\"",
    ));
    steps
}

const PACE_DIRECTOR_ENDED: usize = 3;
const PACE_ASKED: usize = 6;
const RANKED: usize = 8;
const SURVEY_ENDED: usize = 10;
const PACE_A_ENDED: usize = 11;
pub const READS_4_4: &[(Read, usize)] = &[
    (Read::DirectorEnded, PACE_DIRECTOR_ENDED),
    (Read::PaceAsked, PACE_ASKED),
    (Read::Ranked, RANKED),
    (Read::SurveyEnded, SURVEY_ENDED),
    (Read::AEnded, PACE_A_ENDED),
];

#[must_use]
pub fn step_4_4() -> Vec<calls::Step> {
    use super::{PACE_ANSWER, PACE_PERSON, PACE_QUESTION_MESSAGE, SECOND_ACCOUNT, SECOND_KEY_FILE, SURVEY};
    vec![
        sign_in(D1),
        call(D1, argv!["account", "add-key", "1", calls::file(SECOND_KEY_FILE), "--deployment", "--key", "second-llm"]),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "pace", PACE_PERSON]),
        start(D1, argv!["task", "show", number(DIRECTOR), "--until", "ended", "--json"]),
        call(D1, argv!["inbox", "--until", "waiting"]),
        // A's question makes the director due. Wait for its acknowledgement
        // run to park before the person reads the question: it must outlive
        // that acknowledgement (jig's domain/tasks/messages.md, 2). Polling chat
        // instead races its live run watch with that run's end and can retry.
        call(D1, argv!["task", "show", number(DIRECTOR), "--until", "phase:idle"]),
        call(D1, argv!["inbox", "--kind", "questions", "--json"]),
        call(D1, argv!["account", "rank", number(SECOND_ACCOUNT), "1"]),
        call(D1, argv!["account", "list", "--json"]),
        call(
            D1,
            argv![
                "task",
                "answer",
                number(DIRECTOR),
                number(PACE_QUESTION_MESSAGE),
                PACE_ANSWER,
                "--key",
                "pace-answer"
            ],
        ),
        // The director reports after A, which reports after the survey. Its
        // existing live watch is the completion barrier; read their ended
        // documents once afterwards, without polling or opening ended watches.
        join(PACE_DIRECTOR_ENDED),
        call(D1, argv!["task", "show", number(SURVEY), "--json"]),
        call(D1, argv!["task", "show", number(A), "--json"]),
    ]
}

// Call positions within the signed-in check suffix; non-call steps consume none.
const CHECK_DIRECTOR_ENDED: usize = 1;
const CHECK_BENCHMARK_WAITING: usize = 4;
const CHECK_BENCHMARK_ENDED: usize = 5;
const CHECK_B_ENDED: usize = 6;
const CHECK_DIRECTOR_ACTIVITY: usize = 7;
const CHECK_DIRECTOR_TIMER: usize = 8;
pub const READS_4_3: &[(Read, usize)] = &[
    (Read::CheckEnded, CHECK_DIRECTOR_ENDED),
    (Read::BenchmarkWaiting, CHECK_BENCHMARK_WAITING),
    (Read::BenchmarkEnded, CHECK_BENCHMARK_ENDED),
    (Read::BEnded, CHECK_B_ENDED),
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
        call(D1, argv!["task", "show", number(B), "--activity", "32", "--json"]),
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

/// Preserve every opener act, then run the check in the signed-in directory.
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
    let base = READS_4_1.iter().map(|(_, call)| call).max().expect("opener calls") + 1;
    steps.extend(step_4_3(base));
    steps
}

#[must_use]
pub fn reads_4_1_4_3() -> Vec<(Read, usize)> {
    let base = READS_4_1.iter().map(|(_, call)| call).max().expect("opener calls") + 1;
    READS_4_1.iter().copied().chain(READS_4_3.iter().map(|&(read, call)| (read, base + call))).collect()
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
pub const READS_5_1: &[(Read, usize)] = &[
    (Read::RulesWaiting, 5),
    (Read::RulesHeld, 8),
    (Read::ScopeHeld, 9),
    (Read::RulesQuestionWaiting, 12),
    (Read::RulesEnded, RULES_ENDED),
    (Read::ScopeEnded, 14),
    (Read::RulesActivity, 15),
    (Read::ScopeActivity, 16),
];

#[must_use]
pub fn step_5_1() -> Vec<calls::Step> {
    use super::{
        NEW_RULES, RULES, RULES_FILE, RULES_HOLD_REVISION, RULES_PERSON, SCOPE, SCOPE_QUESTION, SCOPE_QUESTION_HOLD,
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
                "--role",
                "context",
                "--no-home",
                "--key",
                "rules-kit"
            ],
        ),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "rules", RULES_PERSON]),
        // Keep the ended watch across Stop and Release instead of polling the
        // full task document through the director's answer and both reports.
        start(D1, argv!["task", "show", number(RULES), "--until", "ended", "--json"]),
        call(D1, argv!["task", "show", number(RULES), "--until", "replied"]),
        until(D1, argv!["task", "show", number(RULES), "--json"], b"\"work\":\"waiting\"", b"\"phase\":\"running\""),
        // Hold the owner's response, not its checkout preparation: stopping
        // during setup would exercise cancellation before it can ask anything.
        calls::writing_task(D1, SCOPE),
        call(D1, argv!["task", "stop", number(RULES), "--key", "rules-stop"]),
        until(D1, argv!["task", "show", number(RULES), "--json"], b"\"placed\"", b"\"phase\":\"running\""),
        until(D1, argv!["task", "show", number(SCOPE), "--json"], b"\"phase\":\"held\"", b"\"task\""),
        // Both old responses and the resumed response have their own fake hold.
        calls::release_hold(SCOPE_QUESTION_HOLD),
        calls::push(RULES_FILE.as_bytes(), NEW_RULES.as_bytes()),
        call(
            D1,
            argv!["task", "release", number(RULES), "--hold", number(RULES_HOLD_REVISION), "--key", "rules-release"],
        ),
        calls::writing_task(D1, SCOPE),
        calls::release_hold(SCOPE_QUESTION_HOLD),
        until(
            D1,
            argv!["task", "show", number(RULES), "--activity", "32", "--json"],
            SCOPE_QUESTION.as_bytes(),
            b"\"task\"",
        ),
        call(
            D1,
            argv![
                "task",
                "answer",
                number(RULES),
                number(super::SCOPE_QUESTION_NUMBER),
                super::SCOPE_ANSWER,
                "--key",
                "scope-answer"
            ],
        ),
        join(RULES_ENDED),
        call(D1, argv!["task", "show", number(SCOPE), "--json"]),
        // Every run's ending: a fresh start refused and retried would pass unseen.
        call(D1, argv!["task", "show", number(RULES), "--activity", "32", "--json"]),
        call(D1, argv!["task", "show", number(SCOPE), "--activity", "32", "--json"]),
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
const B_ENDED_5_2: usize = 22;
pub const READS_5_2: &[(Read, usize)] = &[
    (Read::ImplementerRunning, 5),
    (Read::H1Running, 7),
    (Read::HelperEnded, 8),
    (Read::BHeld, 11),
    (Read::ImplementerHeld, 12),
    (Read::H1Held, 13),
    (Read::H1ReviewerHeld, 14),
    (Read::BStillHeld, 15),
    (Read::ImplementerEnded, 20),
    (Read::H1Ended, 21),
    (Read::BEnded, B_ENDED_5_2),
    (Read::BEndedEarlier, 23),
    (Read::Ended, 24),
    (Read::PauseEnded, 25),
];

#[must_use]
pub fn step_5_2() -> Vec<calls::Step> {
    use super::{
        Agent, B_HOLD, BOWNER, H1, H1_REVIEWER, HELPER, IMPLEMENTER, IMPLEMENTER_HOLD, OWNER, PAUSE, PAUSE_PERSON,
        PROJECT, REVIEW_HOLD,
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
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "pause", PAUSE_PERSON]),
        call(D1, argv!["task", "show", number(PAUSE), "--until", "replied"]),
        call(D1, argv!["task", "show", number(BOWNER), "--until", "replied"]),
        // The documents establish the case's in-flight work before Stop.
        // The fake Pieces keep each run writing until its release below.
        calls::writing_task(D1, IMPLEMENTER),
        // h1 exists after B's reply; its review is created asynchronously.
        // Wait for its live delegate card before reading the review's document.
        until(D1, argv!["task", "show", number(H1), "--json"], Agent::Review.cue(), b"\"task\""),
        calls::writing_task(D1, H1_REVIEWER),
        until(
            D1,
            argv!["task", "show", number(HELPER), "--activity", "32", "--json"],
            b"\"phase\":\"ended\"",
            b"\"task\"",
        ),
        // replied alone can still be writing; read the parked owner's phase.
        call(D1, argv!["task", "show", number(BOWNER), "--until", "phase:idle"]),
        call(D1, argv!["task", "stop", number(BOWNER), "--key", "stop-b"]),
        until(
            D1,
            argv!["task", "show", number(BOWNER), "--activity", "32", "--json"],
            b"\"phase\":\"held\"",
            b"\"task\"",
        ),
        until(
            D1,
            argv!["task", "show", number(IMPLEMENTER), "--activity", "32", "--json"],
            b"\"phase\":\"held\"",
            b"\"task\"",
        ),
        until(D1, argv!["task", "show", number(H1), "--activity", "32", "--json"], b"\"phase\":\"held\"", b"\"task\""),
        until(
            D1,
            argv!["task", "show", number(H1_REVIEWER), "--activity", "32", "--json"],
            b"\"phase\":\"held\"",
            b"\"task\"",
        ),
        call(D1, argv!["task", "show", number(BOWNER), "--activity", "32", "--json"]),
        // Complete the cancelled model responses before the new attempts use
        // the peer, then release the new responses once they are writing.
        calls::release_hold(IMPLEMENTER_HOLD),
        calls::release_hold(REVIEW_HOLD),
        call(D1, argv!["task", "release", number(BOWNER), "--hold", number(B_HOLD), "--key", "release-b"]),
        // The cancelled response's fake hold is not the resumed response's.
        // Observe both new runs writing before releasing their scripted replies.
        calls::writing_task(D1, IMPLEMENTER),
        calls::writing_task(D1, H1_REVIEWER),
        // Release also activates B. Let that activation park before either
        // delegate finishes, so no result enters its release query mid-read.
        // The world's patience bounds the phase watch and reports its argv.
        call(D1, argv!["task", "show", number(BOWNER), "--until", "phase:idle", "--json"]).succeeds(),
        calls::release_hold(IMPLEMENTER_HOLD),
        calls::release_hold(REVIEW_HOLD),
        until(
            D1,
            argv!["task", "show", number(IMPLEMENTER), "--activity", "32", "--json"],
            b"\"phase\":\"ended\"",
            b"\"task\"",
        ),
        until(D1, argv!["task", "show", number(H1), "--activity", "32", "--json"], b"\"phase\":\"ended\"", b"\"task\""),
        until(
            D1,
            argv!["task", "show", number(BOWNER), "--activity", "32", "--json"],
            b"\"phase\":\"ended\"",
            b"\"task\"",
        ),
        // B's two attempts span two activity pages; retain the supplied cursor.
        call(D1, argv!["task", "activity", number(BOWNER), "--before", calls::activity_before(B_ENDED_5_2), "--json"]),
        call(D1, argv!["task", "show", number(OWNER), "--activity", "32", "--json"]),
        until(
            D1,
            argv!["task", "show", number(PAUSE), "--activity", "32", "--json"],
            b"\"phase\":\"ended\"",
            b"\"task\"",
        ),
    ]
}

// Act 2, step 5.5 (testing/reference-case.md)
const PLAN_ENDED: usize = 2;
pub const READS_5_5: &[(Read, usize)] = &[
    (Read::PlanEnded, PLAN_ENDED),
    (Read::PlannerParked, 4),
    (Read::PlanAnswerPending, 5),
    (Read::PlannerEnded, 7),
    (Read::PlannerEarlier, 8),
    (Read::PlannerFirst, 9),
    (Read::SubownerEnded, 10),
];

#[must_use]
pub fn step_5_5() -> Vec<calls::Step> {
    use super::{PLAN_ANSWER_HOLD, PLAN_ANSWER_PENDING, PLAN_DIRECTOR, PLAN_PERSON, PLANNER, SUBOWNER};
    vec![
        sign_in(D1),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "plan", PLAN_PERSON]),
        start(D1, argv!["task", "show", number(PLAN_DIRECTOR), "--until", "ended", "--json"]),
        // A live stream confirms identity, not creation of the director's delegate.
        calls::until(D1, argv!["task", "show", number(PLAN_DIRECTOR), "--json"], b"\"live_delegates\":1", b"\"task\""),
        call(D1, argv!["task", "show", number(PLANNER), "--until", "phase:idle", "--json"]),
        // Step::Held gates ctl writes. The LLM's emitted piece, read through
        // the live run document, proves its answer is at the fake hold.
        until(
            D1,
            argv!["task", "run", number(PLAN_DIRECTOR), "--json"],
            PLAN_ANSWER_PENDING.as_bytes(),
            b"\"writing\":",
        ),
        calls::release_hold(PLAN_ANSWER_HOLD),
        call(
            D1,
            argv![
                "task",
                "answer",
                number(PLAN_DIRECTOR),
                number(super::PLAN_QUESTION_NUMBER),
                super::PLAN_ANSWER,
                "--key",
                "plan-answer"
            ],
        ),
        join(PLAN_ENDED),
        call(D1, argv!["task", "show", number(PLANNER), "--activity", "32", "--json"]),
        call(D1, argv!["task", "activity", number(PLANNER), "--before", calls::activity_before(7), "--json"]),
        call(D1, argv!["task", "activity", number(PLANNER), "--before", calls::activity_before(8), "--json"]),
        call(D1, argv!["task", "show", number(SUBOWNER), "--json"]),
    ]
}

// Act 2, step 5.4 (testing/reference-case.md)
const DIRECTION_B_ENDED: usize = 32;
const STORY_OWNER_ENDED: usize = 36;
const DIRECTION_ENDED: usize = 38;
const STORY_OWNER_EARLIER: usize = 39;
const STORY_OWNER_FIRST: usize = 40;
const STORY_OWNER_INITIAL: usize = 41;
pub const READS_5_4: &[(Read, usize)] = &[
    (Read::X3Open, 7),
    (Read::X3Produced, 8),
    (Read::PlannedCoderRunning, 11),
    (Read::PlannedRunning, 12),
    (Read::DirectionBHeld, 15),
    (Read::PlannedHeld, 16),
    (Read::PlannedCoderHeld, 17),
    (Read::DirectionNoteWritten, 20),
    (Read::X3Ended, 22),
    (Read::StoryLanded, 25),
    (Read::DirectionBPending, 26),
    (Read::PlannedEnded, 30),
    (Read::DirectionBEnded, DIRECTION_B_ENDED),
    (Read::StoryOwnerEnded, STORY_OWNER_ENDED),
    (Read::DirectionEnded, DIRECTION_ENDED),
    (Read::StoryOwnerEarlier, STORY_OWNER_EARLIER),
    (Read::StoryOwnerFirst, STORY_OWNER_FIRST),
    (Read::StoryOwnerInitial, STORY_OWNER_INITIAL),
];
#[must_use]
#[expect(clippy::too_many_lines, reason = "the reference story keeps its ordered ctl acts together")]
pub fn step_5_4() -> Vec<calls::Step> {
    use super::{
        Agent, B_DELEGATE_HOLD, DIRECTION, DIRECTION_MESSAGE, DIRECTION_PERSON, OVERLAP_HOLD_REVISION, OVERLAP_OWNER,
        PLANNED, PLANNED_CODER, PLANNED_CODING_HOLD, STORY_CHANGE, STORY_OWNER, STORY_RECALL_HOLD, X3, X3_CODER,
        X3_REVIEW_HOLD, X3_REVIEWER,
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
        // The delegate cue selects x3's review before its writing observation.
        // Fake Pieces keep it writing until ReleaseHold (i2's addendum 2).
        until(D1, argv!["task", "show", number(X3), "--json"], Agent::Review.cue(), b"\"task\""),
        until(D1, argv!["task", "show", number(X3_REVIEWER), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        call(D1, argv!["task", "show", number(X3), "--activity", "32", "--json"]),
        call(D1, argv!["task", "show", number(X3_CODER), "--activity", "32", "--json"]),
        calls::release_hold(B_DELEGATE_HOLD),
        until(D1, argv!["task", "show", number(OVERLAP_OWNER), "--json"], Agent::PlannedWriter.cue(), b"\"task\""),
        until(D1, argv!["task", "show", number(PLANNED), "--json"], b"\"live_delegates\":1", b"\"task\""),
        until(D1, argv!["task", "show", number(PLANNED_CODER), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        call(D1, argv!["task", "show", number(PLANNED), "--activity", "32", "--json"]),
        call(D1, argv!["task", "show", number(OVERLAP_OWNER), "--until", "phase:idle"]),
        call(D1, argv!["task", "stop", number(OVERLAP_OWNER), "--key", "stop-b"]),
        until(
            D1,
            argv!["task", "show", number(OVERLAP_OWNER), "--activity", "32", "--json"],
            b"\"phase\":\"held\"",
            b"\"task\"",
        ),
        call(D1, argv!["task", "show", number(PLANNED), "--json"]),
        call(D1, argv!["task", "show", number(PLANNED_CODER), "--json"]),
        call(D1, argv!["task", "message", number(DIRECTION), "--key", "new-direction", DIRECTION_MESSAGE]),
        call(D1, argv!["task", "show", number(X3), "--until", "phase:closing"]),
        // Commit the note before releasing the review: otherwise the cancel's
        // result can take the note's fresh number between A's two tool calls.
        until(
            D1,
            argv!["task", "show", number(STORY_OWNER), "--activity", "32", "--json"],
            b"\"tool\":\"note\"",
            b"\"task\"",
        ),
        calls::release_hold(X3_REVIEW_HOLD),
        call(D1, argv!["task", "show", number(X3), "--until", "ended"]),
        call(D1, argv!["task", "show", number(X3), "--activity", "32", "--json"]),
        until(D1, argv!["task", "show", number(STORY_OWNER), "--json"], Agent::StoryWriter.cue(), b"\"task\""),
        // a1's review has no hold. Watch endings once, then read their journals,
        // avoiding repeated reads of the same large activity documents.
        call(D1, argv!["task", "show", number(STORY_CHANGE), "--until", "ended"]),
        call(D1, argv!["task", "show", number(STORY_CHANGE), "--activity", "32", "--json"]),
        call(D1, argv!["task", "show", number(OVERLAP_OWNER), "--activity", "32", "--json"]),
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
        call(D1, argv!["task", "show", number(PLANNED), "--until", "ended"]),
        call(D1, argv!["task", "show", number(PLANNED), "--activity", "32", "--json"]),
        call(D1, argv!["task", "show", number(OVERLAP_OWNER), "--until", "ended"]),
        call(D1, argv!["task", "show", number(OVERLAP_OWNER), "--activity", "32", "--json"]),
        // B's batched turns fit its complete journal in this one page.
        // Select the run woken by a1's change result, then await its held
        // writing turn (i2's addendum 2). x3's result was a cancellation.
        until(
            D1,
            argv!["task", "show", number(STORY_OWNER), "--activity", "32", "--json"],
            b"\"kind\":{\"result\":{\"change\":",
            b"\"task\"",
        ),
        until(D1, argv!["task", "show", number(STORY_OWNER), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        calls::release_hold(STORY_RECALL_HOLD),
        call(D1, argv!["task", "show", number(STORY_OWNER), "--until", "ended"]),
        call(D1, argv!["task", "show", number(STORY_OWNER), "--activity", "32", "--json"]),
        call(D1, argv!["task", "show", number(DIRECTION), "--until", "ended"]),
        call(D1, argv!["task", "show", number(DIRECTION), "--activity", "32", "--json"]),
        call(
            D1,
            argv![
                "task",
                "activity",
                number(STORY_OWNER),
                "--before",
                calls::activity_before(STORY_OWNER_ENDED),
                "--json"
            ],
        ),
        call(
            D1,
            argv![
                "task",
                "activity",
                number(STORY_OWNER),
                "--before",
                calls::activity_before(STORY_OWNER_EARLIER),
                "--json"
            ],
        ),
        // Run endings can put the first attempt's beginning on a fourth page;
        // follow that page only when the third supplies a cursor.
        call(
            D1,
            argv![
                "task",
                "activity",
                number(STORY_OWNER),
                "--before",
                calls::activity_before(STORY_OWNER_FIRST),
                "--json"
            ],
        )
        .if_activity_remains(),
    ]
}

// Act 3, step 6.5 (testing/reference-case.md)
pub const READS_6_5: &[(Read, usize)] = &[
    (Read::BudgetAsked, 5),
    (Read::BudgetMedium, 10),
    (Read::BudgetHigh, 11),
    (Read::BudgetA5Ended, 12),
    (Read::BudgetAdvisorEnded, 13),
    (Read::BudgetSubownerEnded, 14),
    (Read::BudgetOwnerEnded, 15),
    (Read::BudgetDirectorEnded, 16),
    (Read::BudgetInboxAnswered, 17),
    (Read::BudgetSubownerFirst, 18),
    (Read::BudgetOwnerFirst, 19),
    (Read::BudgetDirectorFirst, 20),
];

#[must_use]
pub fn step_6_5() -> Vec<calls::Step> {
    use super::{
        Agent, BUDGET_A5, BUDGET_ADVISOR, BUDGET_AMEND_HOLD, BUDGET_DIRECTOR, BUDGET_HIGH_HOLD, BUDGET_OWNER,
        BUDGET_PERSON, BUDGET_PERSON_ANSWER, BUDGET_SUBOWNER, BUDGET_UP_QUESTION_NUMBER, PROJECT,
    };
    let mut steps = vec![
        sign_in(D1),
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "budget", BUDGET_PERSON]),
        start(D1, argv!["task", "show", number(BUDGET_DIRECTOR), "--until", "ended", "--json"]),
        call(D1, argv!["inbox", "--until", "waiting"]),
        call(D1, argv!["task", "show", number(BUDGET_DIRECTOR), "--until", "phase:idle"]),
        call(D1, argv!["inbox", "--kind", "questions", "--json"]),
        call(
            D1,
            argv![
                "task",
                "answer",
                number(BUDGET_DIRECTOR),
                number(BUDGET_UP_QUESTION_NUMBER),
                BUDGET_PERSON_ANSWER,
                "--key",
                "budget-answer"
            ],
        ),
        until(
            D1,
            argv!["task", "show", number(BUDGET_SUBOWNER), "--json"],
            Agent::BudgetInvestigator.cue(),
            b"\"task\"",
        ),
        until(D1, argv!["task", "show", number(BUDGET_SUBOWNER), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        call(D1, argv!["task", "show", number(BUDGET_A5), "--until", "phase:idle"]),
        call(D1, argv!["task", "show", number(BUDGET_A5), "--activity", "32", "--json"]),
        calls::release_hold(BUDGET_AMEND_HOLD),
        until(
            D1,
            argv!["task", "show", number(BUDGET_A5), "--activity", "32", "--json"],
            b"\"work\":\"writing\"",
            b"\"task\"",
        ),
        calls::release_hold(BUDGET_HIGH_HOLD),
        join(2),
    ];
    for task in [BUDGET_A5, BUDGET_ADVISOR, BUDGET_SUBOWNER, BUDGET_OWNER, BUDGET_DIRECTOR] {
        steps.push(call(D1, argv!["task", "show", number(task), "--activity", "32", "--json"]));
    }
    steps.push(call(D1, argv!["inbox", "--kind", "questions", "--json"]));
    // Follow each supplied cursor; never infer a complete journal from a
    // latest page whose first attempts have fallen outside the window.
    for (task, latest) in [(BUDGET_SUBOWNER, 14), (BUDGET_OWNER, 15), (BUDGET_DIRECTOR, 16)] {
        steps.push(call(
            D1,
            argv!["task", "activity", number(task), "--before", calls::activity_before(latest), "--json"],
        ));
    }
    steps
}

// Act 3, step 6.4 (testing/reference-case.md)
const RED_OWNER_ENDED: usize = 19;
pub const READS_6_4: &[(Read, usize)] = &[
    (Read::RedReviewWriting, 8),
    (Read::RedChangeOpen, 9),
    (Read::RedRootsPending, 10),
    (Read::RedRepairWriting, 12),
    // Reuse the creation read; the held coder has its separate document.
    (Read::RedRepairPending, 11),
    (Read::RedTimerArmed, 14),
    (Read::RedChangePaused, 15),
    (Read::RedDirectorEnded, 18),
    (Read::RedOwnerEnded, RED_OWNER_ENDED),
    (Read::RedChangeEnded, 20),
    (Read::RedCoderEnded, 21),
    (Read::RedReviewEnded, 22),
    // The ended wait already returns the complete procedure document.
    (Read::RedRepairEnded, 16),
    (Read::RedRepairCoderEnded, 23),
    (Read::RedRepairReviewEnded, 24),
    (Read::RedUpdatedReviewEnded, 25),
    (Read::RedRootsEnded, 26),
];
#[must_use]
pub fn step_6_4() -> Vec<calls::Step> {
    use super::{
        Agent, PROJECT, RED_BROKEN, RED_CHANGE, RED_CODER, RED_DIRECTOR, RED_FILE, RED_MESSAGE, RED_OWNER, RED_PERSON,
        RED_REPAIR, RED_REPAIR_CODER, RED_REPAIR_HOLD, RED_REPAIR_REVIEW, RED_REVIEW, RED_REVIEW_HOLD,
        RED_UPDATED_REVIEW,
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
        call(D1, argv!["chat", "start", "--project", number(PROJECT), "--key", "red", RED_PERSON]),
        call(D1, argv!["task", "show", number(RED_DIRECTOR), "--until", "replied"]),
        call(D1, argv!["task", "show", number(RED_OWNER), "--until", "replied"]),
        // The producing procedure parks after making its coder; watch that ending.
        call(D1, argv!["task", "show", number(RED_CHANGE), "--until", "phase:idle"]),
        call(D1, argv!["task", "show", number(RED_CODER), "--until", "ended"]),
        // No ctl predicate selects review creation: poll only this later transition.
        until(D1, argv!["task", "show", number(RED_CHANGE), "--json"], Agent::Review.cue(), b"\"task\""),
        until(D1, argv!["task", "show", number(RED_REVIEW), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        // News and forge hints wake a4's procedure for a step that finds nothing to do: a plain read can catch it
        // due. Read it settled: an ended a4 answers with its ended read, which the oracle names.
        call(D1, argv!["task", "show", number(RED_CHANGE), "--until", "phase:idle", "--json"]),
        calls::push(RED_FILE.as_bytes(), RED_BROKEN.as_bytes()),
        // A ready change, after its gate passes, discovers the red base.
        calls::release_hold(RED_REVIEW_HOLD),
        // Until cannot retry an unknown task: first observe the deployment
        // repair among project roots, then its coder's live delegate card.
        until(
            D1,
            argv!["task", "list", "--project", number(PROJECT), "--json"],
            Agent::RedRepairCoder.cue(),
            b"\"tasks\"",
        ),
        until(D1, argv!["task", "show", number(RED_REPAIR), "--json"], Agent::RedRepairCoder.cue(), b"\"task\""),
        until(D1, argv!["task", "show", number(RED_REPAIR_CODER), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        call(D1, argv!["task", "message", number(RED_DIRECTOR), "--key", "red-tip", RED_MESSAGE]),
        until(
            D1,
            argv!["task", "show", number(RED_DIRECTOR), "--activity", "32", "--json"],
            b"\"kind\":{\"timer\":{",
            b"\"task\"",
        ),
        call(D1, argv!["task", "show", number(RED_CHANGE), "--until", "phase:idle", "--json"]),
        calls::release_hold(RED_REPAIR_HOLD),
        call(D1, argv!["task", "show", number(RED_REPAIR), "--until", "ended", "--json"]),
        call(D1, argv!["task", "show", number(RED_DIRECTOR), "--until", "ended"]),
    ];
    for task in [
        RED_DIRECTOR,
        RED_OWNER,
        RED_CHANGE,
        RED_CODER,
        RED_REVIEW,
        RED_REPAIR_CODER,
        RED_REPAIR_REVIEW,
        RED_UPDATED_REVIEW,
    ] {
        steps.push(call(D1, argv!["task", "show", number(task), "--activity", "32", "--json"]));
    }
    steps.push(call(D1, argv!["task", "list", "--project", number(PROJECT), "--ended", "--json"]));
    steps
}

// Act 3, step 6.6. Name every call position: holds and releases make no call.
const WAITING_SIGN_IN: usize = 0;
const WAITING_ADOPT_APP: usize = WAITING_SIGN_IN + 1;
const WAITING_ADOPT_KIT: usize = WAITING_ADOPT_APP + 1;
const WAITING_CHAT: usize = WAITING_ADOPT_KIT + 1;
const WAITING_DIRECTOR_REPLIED: usize = WAITING_CHAT + 1;
const WAITING_OWNER_REPLIED: usize = WAITING_DIRECTOR_REPLIED + 1;
const WAITING_REVIEW_CREATED: usize = WAITING_OWNER_REPLIED + 1;
const WAITING_REVIEW_WRITING: usize = WAITING_REVIEW_CREATED + 1;
const WAITING_E1_PENDING: usize = WAITING_REVIEW_WRITING + 1;
const WAITING_REPIN_PENDING: usize = WAITING_E1_PENDING + 1;
const WAITING_DIRECTOR_ENDED: usize = WAITING_REPIN_PENDING + 3;
const WAITING_ENDED_FIRST: usize = WAITING_DIRECTOR_ENDED + 1;
pub const READS_6_6: &[(Read, usize)] = &[
    (Read::WaitingReviewWriting, WAITING_REVIEW_WRITING),
    (Read::WaitingE1Pending, WAITING_E1_PENDING),
    (Read::WaitingRepinPending, WAITING_REPIN_PENDING),
    (Read::WaitingDirectorEnded, WAITING_ENDED_FIRST),
    (Read::WaitingOwnerEnded, WAITING_ENDED_FIRST + 1),
    (Read::WaitingE1Ended, WAITING_ENDED_FIRST + 2),
    (Read::WaitingRepinEnded, WAITING_ENDED_FIRST + 3),
    (Read::WaitingE1CoderEnded, WAITING_ENDED_FIRST + 4),
    (Read::WaitingE1ReviewerEnded, WAITING_ENDED_FIRST + 5),
    (Read::WaitingRepinCoderEnded, WAITING_ENDED_FIRST + 6),
    (Read::WaitingRepinReviewerEnded, WAITING_ENDED_FIRST + 7),
    (Read::WaitingRootsEnded, WAITING_ENDED_FIRST + 8),
];

#[must_use]
pub fn step_6_6() -> Vec<calls::Step> {
    use super::{
        Agent,
        waiting::{
            A1, DIRECTOR, E1, E1_CODER, E1_REVIEWER, PERSON, REPIN, REPIN_CODER, REPIN_HOLD, REPIN_REVIEWER,
            REVIEW_HOLD,
        },
    };
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
        call(D1, argv!["task", "show", number(DIRECTOR), "--until", "replied"]),
        call(D1, argv!["task", "show", number(A1), "--until", "replied"]),
        until(D1, argv!["task", "show", number(E1), "--json"], Agent::Review.cue(), b"\"task\""),
        until(
            D1,
            argv!["task", "show", calls::delegate(WAITING_REVIEW_CREATED), "--json"],
            b"\"work\":\"writing\"",
            b"\"task\"",
        ),
        call(D1, argv!["task", "show", number(E1), "--json"]),
        call(D1, argv!["task", "show", number(REPIN), "--activity", "32", "--json"]),
        until(
            D1,
            argv!["task", "show", calls::delegate(WAITING_REPIN_PENDING), "--json"],
            b"\"work\":\"writing\"",
            b"\"task\"",
        ),
        calls::release_hold(REVIEW_HOLD),
        // The producer is already running. Its scripted write waits for the
        // kit result so it can pin the landing commit.
        call(D1, argv!["task", "show", number(E1), "--until", "ended"]),
        calls::release_hold(REPIN_HOLD),
        call(D1, argv!["task", "show", number(DIRECTOR), "--until", "ended"]),
    ]);
    for task in [DIRECTOR, A1, E1, REPIN, E1_CODER, E1_REVIEWER, REPIN_CODER, REPIN_REVIEWER] {
        steps.push(call(D1, argv!["task", "show", number(task), "--activity", "32", "--json"]));
    }
    steps.push(call(D1, argv!["task", "list", "--project", number(PROJECT), "--ended", "--json"]));
    steps
}

// 6.8: release steps do not occupy ctl call positions.
const QA_SIGN_IN: usize = 0;
const QA_ADOPT: usize = QA_SIGN_IN + 1;
const QA_CHAT: usize = QA_ADOPT + 1;
const QA_DIRECTOR_REPLIED: usize = QA_CHAT + 1;
const QA_OWNER_REPLIED: usize = QA_DIRECTOR_REPLIED + 1;
const QA_TESTER_WRITING: usize = QA_OWNER_REPLIED + 1;
const QA_OWNER_WRITING: usize = QA_TESTER_WRITING + 1;
const QA_SUBOWNER_BEFORE: usize = QA_OWNER_WRITING + 1;
const QA_NEXT_CREATED: usize = QA_SUBOWNER_BEFORE + 1;
const QA_NEXT_WRITING: usize = QA_NEXT_CREATED + 1;
const QA_SUBOWNER_LIVE: usize = QA_NEXT_WRITING + 1;
const QA_TESTER_LIVE: usize = QA_SUBOWNER_LIVE + 1;
const QA_NEXT_ENDED: usize = QA_TESTER_LIVE + 1;
const QA_DIRECTOR_ENDED: usize = QA_NEXT_ENDED + 1;
const QA_FINAL: usize = QA_DIRECTOR_ENDED + 1;
const QA_INBOX: usize = QA_FINAL + 8;
const QA_ROOTS: usize = QA_INBOX + 1;
pub const READS_6_8: &[(Read, usize)] = &[
    (Read::QaSubownerBefore, QA_SUBOWNER_BEFORE),
    (Read::QaNextWriting, QA_NEXT_WRITING),
    (Read::QaSubownerLive, QA_SUBOWNER_LIVE),
    (Read::QaTesterLive, QA_TESTER_LIVE),
    (Read::QaDirectorEnded, QA_FINAL),
    (Read::QaOwnerEnded, QA_FINAL + 1),
    (Read::QaSubownerEnded, QA_FINAL + 2),
    (Read::QaTesterEnded, QA_FINAL + 3),
    (Read::QaNextEnded, QA_FINAL + 4),
    (Read::QaChangeEnded, QA_FINAL + 5),
    (Read::QaCoderEnded, QA_FINAL + 6),
    (Read::QaReviewerEnded, QA_FINAL + 7),
    (Read::QaInboxEnded, QA_INBOX),
    (Read::QaRootsEnded, QA_ROOTS),
];

#[must_use]
pub fn step_6_8() -> Vec<calls::Step> {
    use super::qa::{A, A1, A2, C, CODER, DIRECTOR, NEXT_HOLD, OWNER_HOLD, PERSON, REVIEWER, T, TESTER_HOLD};
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
        call(D1, argv!["task", "show", number(DIRECTOR), "--until", "replied"]),
        call(D1, argv!["task", "show", number(A1), "--until", "replied"]),
        until(D1, argv!["task", "show", number(T), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        until(D1, argv!["task", "show", number(A), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        call(D1, argv!["task", "show", number(A1), "--activity", "32", "--json"]),
        calls::release_hold(OWNER_HOLD),
        call(D1, argv!["task", "show", number(A), "--until", "phase:idle"]),
        until(D1, argv!["task", "show", number(A2), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        call(D1, argv!["task", "show", number(A1), "--activity", "32", "--json"]),
        call(D1, argv!["task", "show", number(T), "--activity", "32", "--json"]),
        calls::release_hold(NEXT_HOLD),
        call(D1, argv!["task", "show", number(A2), "--until", "ended"]),
        calls::release_hold(TESTER_HOLD),
        call(D1, argv!["task", "show", number(DIRECTOR), "--until", "ended"]),
    ];
    for task in [DIRECTOR, A, A1, T, A2, C, CODER, REVIEWER] {
        steps.push(call(D1, argv!["task", "show", number(task), "--activity", "32", "--json"]));
    }
    steps.push(call(D1, argv!["inbox", "--after", "00", "--json"]));
    steps.push(call(D1, argv!["task", "list", "--project", number(PROJECT), "--ended", "--json"]));
    steps
}
