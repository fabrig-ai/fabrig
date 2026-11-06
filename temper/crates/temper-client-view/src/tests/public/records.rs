//! Public wire records used by the anonymous view (temper/docs/design/client/browser.md, 8.4).
use super::{PROJECT, TASK};
use jig_client_protocol_wire::public_v1 as j;
use skein_lib::{List, Writer};
use temper_client_protocol_wire::public_v1 as f;
pub(super) const REPOSITORY: &[u8] = b"https://forge.example/owner/repository";
pub(super) const PULL: &[u8] = b"https://forge.example/owner/repository/pulls/7";
pub(super) const RUN: &[u8] = b"https://forge.example/owner/repository/actions/runs/8";
const FAILED_GOAL: u64 = 20;
const OTHER_FAILED_GOAL: u64 = 21;
const COMPLETED_GOAL: u64 = 22;
const CANCELLED_GOAL: u64 = 23;
const ENDED_GOALS: u32 = 4; // The two failures, completion and cancellation.
/// Synthetic commit from the concatenated forge-link regression.
pub(super) const WALK_HEAD: [u8; 32] = [
    0x78, 0x9a, 0xda, 0x51, 0x5c, 0x1a, 0xf9, 0xd4, 0x49, 0x67, 0xf2, 0x60, 0x49, 0xc4, 0x09, 0x0b, 0x3a, 0xb4, 0x72,
    0xdb, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const HEAD: [u8; 32] = [0x12; 32];
pub(super) fn line(goal: bool) -> j::PublicLine {
    terminal_line(TASK, goal, j::PublicPhase::Running, j::PublicExecutor::Agent)
}

/// Build a public line with the supplied outcome and executor kind.
pub(super) fn terminal_line(
    task: u64,
    goal: bool,
    phase: j::PublicPhase,
    executor: j::PublicExecutor,
) -> j::PublicLine {
    let counts = j::PublicCounts::new(
        &j::CEILINGS,
        j::PublicCountsParts { working: 0, waiting: 0, held: 0, closing: 0, ended: 0 },
    )
    .expect("counts");
    j::PublicLine::new(
        &j::CEILINGS,
        j::PublicLineParts {
            task,
            title: b"A public goal title".as_slice().into(),
            title_cut: false,
            parent: None,
            goal,
            priority: None,
            executor,
            phase,
            hold: None,
            made: 0,
            changed: 0,
            active: None,
            ended: None,
            spent: 12,
            budget: 50,
            delegates: counts,
        },
    )
    .expect("line")
}
pub(super) fn card() -> Box<[u8]> {
    let card = j::PublicCard::new(
        &j::CEILINGS,
        j::PublicCardParts {
            line: line(false),
            lineage: List::from_boxed(Box::new([PROJECT])),
            delegates: List::from_boxed(Box::new([line(true)])),
            folded: 0,
        },
    )
    .expect("card");
    let mut writer = Writer::new(usize::try_from(card.measure()).expect("small card"));
    card.encode(&mut writer).expect("card");
    writer.finish()
}
fn used() -> f::PublicUsed {
    f::PublicUsed::new(&f::CEILINGS, f::PublicUsedParts { used: 0, limit: 2 }).expect("counters")
}
pub(super) fn change() -> Box<[u8]> {
    change_with_commits(HEAD, None)
}
/// Encode a public change with the supplied head and optional merge commit.
pub(super) fn change_with_commits(head: [u8; 32], merge: Option<[u8; 32]>) -> Box<[u8]> {
    let gate = f::PublicGate::new(
        &f::CEILINGS,
        f::PublicGateParts {
            number: 3,
            kind: f::PublicGateKind::Review,
            blocks: true,
            verdict: f::PublicVerdict::Approved,
        },
    )
    .expect("gate");
    let change = f::PublicChange::new(
        &f::CEILINGS,
        f::PublicChangeParts {
            task: 0,
            repository: REPOSITORY.into(),
            pull: Some(PULL.into()),
            head: Some(head),
            step: f::PublicStep::Checking,
            repair: None,
            merge,
            tiers: 0,
            hold: None,
            ci: f::PublicCi::Passed,
            gates: List::from_boxed(Box::new([gate])),
            repairs: used(),
            resolutions: used(),
            updates: used(),
            way: f::PublicWay::Queue,
            in_landing: false,
        },
    )
    .expect("change");
    let mut writer = Writer::new(usize::try_from(change.measure()).expect("small change"));
    change.encode(&mut writer).expect("change");
    application(writer.finish())
}
fn application(record: Box<[u8]>) -> Box<[u8]> {
    let app = j::PublicApplication::new(
        &j::CEILINGS,
        j::PublicApplicationParts { tasks: List::from_boxed(Box::new([TASK, PROJECT])), record },
    )
    .expect("application");
    let mut writer = Writer::new(usize::try_from(app.measure()).expect("small application"));
    app.encode(&mut writer).expect("application");
    writer.finish()
}
pub(super) fn landings() -> Box<[u8]> {
    let queued =
        f::PublicQueued::new(&f::CEILINGS, f::PublicQueuedParts { task: 0, goal: 1, ready_since: 0 }).expect("queued");
    let queue = f::PublicQueue::new(
        &f::CEILINGS,
        f::PublicQueueParts { repository: REPOSITORY.into(), changes: List::from_boxed(Box::new([queued])) },
    )
    .expect("queue");
    let line = f::PublicLandingLine::new(
        &f::CEILINGS,
        f::PublicLandingLineParts {
            members: List::from_boxed(Box::new([0])),
            phase: f::PublicLandingPhase::Landed,
            landed: List::from_boxed(Box::new([HEAD])),
        },
    )
    .expect("landing line");
    let landings = f::PublicLandings::new(
        &f::CEILINGS,
        f::PublicLandingsParts {
            turn: Some(0),
            landings: List::from_boxed(Box::new([line])),
            queues: List::from_boxed(Box::new([queue])),
        },
    )
    .expect("landings");
    let mut writer = Writer::new(usize::try_from(landings.measure()).expect("small landings"));
    landings.encode(&mut writer).expect("landings");
    application(writer.finish())
}
pub(super) fn landing() -> Box<[u8]> {
    landing_with_commits(None, None, HEAD)
}
/// Encode a public landing with the supplied base, candidate and landed commits.
pub(super) fn landing_with_commits(base: Option<[u8; 32]>, candidate: Option<[u8; 32]>, landed: [u8; 32]) -> Box<[u8]> {
    let gate = f::PublicGateProof::new(
        &f::CEILINGS,
        f::PublicGateProofParts {
            number: 3,
            kind: f::PublicGateKind::Check,
            blocks: false,
            proof: f::PublicProof::Passed,
            run: Some(RUN.into()),
        },
    )
    .expect("proof");
    let member = f::PublicMember::new(
        &f::CEILINGS,
        f::PublicMemberParts {
            task: 0,
            repository: REPOSITORY.into(),
            pull: Some(PULL.into()),
            base,
            candidate,
            landed: Some(landed),
            gates: List::from_boxed(Box::new([gate])),
        },
    )
    .expect("member");
    let landing = f::PublicLanding::new(
        &f::CEILINGS,
        f::PublicLandingParts {
            phase: f::PublicLandingPhase::Landed,
            hold: None,
            preparation: 0,
            members: List::from_boxed(Box::new([member])),
            stale_proofs: 2,
            holds_turn: true,
        },
    )
    .expect("landing");
    let mut writer = Writer::new(usize::try_from(landing.measure()).expect("small landing"));
    landing.encode(&mut writer).expect("landing");
    application(writer.finish())
}
pub(super) fn projects() -> Box<[u8]> {
    let project = j::PublicProject::new(
        &j::CEILINGS,
        j::PublicProjectParts { project: PROJECT, name: b"A public project".as_slice().into() },
    )
    .expect("project");
    let projects = j::PublicProjects::new(
        &j::CEILINGS,
        j::PublicProjectsParts { projects: List::from_boxed(Box::new([project])), max_age: 8 },
    )
    .expect("projects");
    let mut writer = Writer::new(usize::try_from(projects.measure()).expect("small projects"));
    projects.encode(&mut writer).expect("projects");
    writer.finish()
}

/// Encode empty projects without a missing or failed document.
pub(super) fn empty_projects() -> Box<[u8]> {
    let projects =
        j::PublicProjects::new(&j::CEILINGS, j::PublicProjectsParts { projects: List::with_capacity(0), max_age: 8 })
            .expect("empty projects");
    let mut writer = Writer::new(usize::try_from(projects.measure()).expect("small projects"));
    projects.encode(&mut writer).expect("projects");
    writer.finish()
}

/// Encode a board whose ended goals keep the walk's two failures and one completion.
pub(super) fn terminal_board(empty: bool) -> Box<[u8]> {
    let mut ended = List::with_capacity(ENDED_GOALS);
    if !empty {
        for (task, phase, executor) in [
            (FAILED_GOAL, j::PublicPhase::Failed, j::PublicExecutor::Agent),
            (OTHER_FAILED_GOAL, j::PublicPhase::Failed, j::PublicExecutor::Job),
            (COMPLETED_GOAL, j::PublicPhase::Done, j::PublicExecutor::Procedure),
            (CANCELLED_GOAL, j::PublicPhase::Cancelled, j::PublicExecutor::Person),
        ] {
            ended
                .push(
                    j::PublicGoal::new(
                        &j::CEILINGS,
                        j::PublicGoalParts {
                            line: terminal_line(task, true, phase, executor),
                            tree: List::with_capacity(0),
                            folded: 0,
                        },
                    )
                    .expect("terminal goal"),
                )
                .expect("four goals fit");
        }
    }
    let board = j::PublicBoard::new(
        &j::CEILINGS,
        j::PublicBoardParts {
            project: PROJECT,
            name: b"A public project".as_slice().into(),
            live: List::with_capacity(0),
            live_folded: 0,
            ended,
        },
    )
    .expect("terminal board");
    let mut writer = Writer::new(usize::try_from(board.measure()).expect("small board"));
    board.encode(&mut writer).expect("board");
    writer.finish()
}

/// Encode lineage links with distinct synthetic identifiers from the walk.
pub(super) fn lineage_card() -> Box<[u8]> {
    let card = j::PublicCard::new(
        &j::CEILINGS,
        j::PublicCardParts {
            line: terminal_line(COMPLETED_GOAL, true, j::PublicPhase::Done, j::PublicExecutor::Procedure),
            lineage: List::from_boxed(Box::new([0x0893_ed2e_0000_0000, 0xbd0d_98ef_0000_0000])),
            delegates: List::with_capacity(0),
            folded: 0,
        },
    )
    .expect("lineage card");
    let mut writer = Writer::new(usize::try_from(card.measure()).expect("small card"));
    card.encode(&mut writer).expect("card");
    writer.finish()
}

/// Encode an available landings document with no queued or retained work.
pub(super) fn empty_landings() -> Box<[u8]> {
    let landings = f::PublicLandings::new(
        &f::CEILINGS,
        f::PublicLandingsParts { turn: None, landings: List::with_capacity(0), queues: List::with_capacity(0) },
    )
    .expect("empty landings");
    let mut writer = Writer::new(usize::try_from(landings.measure()).expect("small landings"));
    landings.encode(&mut writer).expect("landings");
    application(writer.finish())
}

/// Encode an empty landing with zero stale proofs.
pub(super) fn empty_landing() -> Box<[u8]> {
    let landing = f::PublicLanding::new(
        &f::CEILINGS,
        f::PublicLandingParts {
            phase: f::PublicLandingPhase::Waiting,
            hold: None,
            preparation: 0,
            members: List::with_capacity(0),
            stale_proofs: 0,
            holds_turn: false,
        },
    )
    .expect("empty landing");
    let mut writer = Writer::new(usize::try_from(landing.measure()).expect("small landing"));
    landing.encode(&mut writer).expect("landing");
    application(writer.finish())
}
