//! Public wire records used by the anonymous view (temper/docs/design/client/web/public.md, 4).
use self as records;
use jig_client_protocol_wire::public_v1 as j;
use skein_lib::{List, Queue, Time, Writer};
use temper_client_domain::{
    address::Public,
    public::{Doc, Heard, PublicEvent, PublicLimits, Reader},
};
use temper_client_protocol_wire::public_v1 as f;
pub const HEAD: [u8; 32] = [
    0x78, 0x9a, 0xda, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
#[must_use]
pub fn line(goal: bool) -> j::PublicLine {
    let counts = j::PublicCounts::new(
        &j::CEILINGS,
        j::PublicCountsParts { working: 0, waiting: 0, held: 0, closing: 0, ended: 0 },
    )
    .expect("counts");
    j::PublicLine::new(
        &j::CEILINGS,
        j::PublicLineParts {
            task: TASK,
            title: b"A public goal title".as_slice().into(),
            title_cut: false,
            parent: None,
            goal,
            priority: None,
            executor: j::PublicExecutor::Agent,
            phase: j::PublicPhase::Running,
            hold: None,
            made: 0,
            changed: 0,
            active: None,
            ended: None,
            spent: if goal { Some(12) } else { None },
            delegates: counts,
        },
    )
    .expect("line")
}
#[must_use]
pub fn card() -> Box<[u8]> {
    card_line(line(false))
}
#[must_use]
pub fn card_line(own: j::PublicLine) -> Box<[u8]> {
    let card = j::PublicCard::new(
        &j::CEILINGS,
        j::PublicCardParts {
            line: own,
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
#[must_use]
pub fn used() -> f::PublicUsed {
    f::PublicUsed::new(&f::CEILINGS, f::PublicUsedParts { used: 0, limit: 2 }).expect("counters")
}
#[must_use]
pub fn change() -> Box<[u8]> {
    change_commits(Some(HEAD), None)
}
#[must_use]
pub fn change_commits(head: Option<[u8; 32]>, merge: Option<[u8; 32]>) -> Box<[u8]> {
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
            head,
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
#[must_use]
pub fn application(record: Box<[u8]>) -> Box<[u8]> {
    let app = j::PublicApplication::new(
        &j::CEILINGS,
        j::PublicApplicationParts { tasks: List::from_boxed(Box::new([TASK, PROJECT])), record },
    )
    .expect("application");
    let mut writer = Writer::new(usize::try_from(app.measure()).expect("small application"));
    app.encode(&mut writer).expect("application");
    writer.finish()
}
#[must_use]
pub fn landings() -> Box<[u8]> {
    landings_with(true, true)
}
#[must_use]
pub fn landings_with(queue_present: bool, landed: bool) -> Box<[u8]> {
    let queued =
        f::PublicQueued::new(&f::CEILINGS, f::PublicQueuedParts { task: 0, goal: 1, ready_since: 0 }).expect("queued");
    let queue =
        f::PublicQueue::new(&f::CEILINGS, f::PublicQueueParts { changes: List::from_boxed(Box::new([queued])) })
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
            turn: if landed { Some(0) } else { None },
            landings: if landed { List::from_boxed(Box::new([line])) } else { List::with_capacity(0) },
            queues: if queue_present { List::from_boxed(Box::new([queue])) } else { List::with_capacity(0) },
        },
    )
    .expect("landings");
    let mut writer = Writer::new(usize::try_from(landings.measure()).expect("small landings"));
    landings.encode(&mut writer).expect("landings");
    application(writer.finish())
}
// A stock queue merge has no turn, preparation, candidates or gate proofs.
#[must_use]
pub fn queue_merge() -> (Box<[u8]>, Box<[u8]>) {
    let line = f::PublicLandingLine::new(
        &f::CEILINGS,
        f::PublicLandingLineParts {
            members: List::from_boxed(Box::new([1])),
            phase: f::PublicLandingPhase::Landed,
            landed: List::from_boxed(Box::new([HEAD])),
        },
    )
    .expect("queue merge line");
    let landings = f::PublicLandings::new(
        &f::CEILINGS,
        f::PublicLandingsParts {
            turn: None,
            landings: List::from_boxed(Box::new([line])),
            queues: List::with_capacity(0),
        },
    )
    .expect("queue merge list");
    let mut writer = Writer::new(usize::try_from(landings.measure()).expect("small landings"));
    landings.encode(&mut writer).expect("landings");
    let list = application(writer.finish());
    let member = f::PublicMember::new(
        &f::CEILINGS,
        f::PublicMemberParts {
            head: None,
            left: None,
            judgement: None,
            task: 1,
            base: None,
            candidate: None,
            landed: Some(HEAD),
            gates: List::with_capacity(0),
        },
    )
    .expect("queue merge member");
    let landing = f::PublicLanding::new(
        &f::CEILINGS,
        f::PublicLandingParts {
            search: None,
            combined: None,
            step: None,
            phase: f::PublicLandingPhase::Landed,
            hold: None,
            preparation: 0,
            members: List::from_boxed(Box::new([member])),
            stale_proofs: 0,
            holds_turn: false,
        },
    )
    .expect("queue merge landing");
    let mut writer = Writer::new(usize::try_from(landing.measure()).expect("small landing"));
    landing.encode(&mut writer).expect("landing");
    (list, application(writer.finish()))
}
#[must_use]
pub fn landing() -> Box<[u8]> {
    landing_commits(None, None, Some(HEAD))
}
#[must_use]
pub fn landing_commits(base: Option<[u8; 32]>, candidate: Option<[u8; 32]>, landed: Option<[u8; 32]>) -> Box<[u8]> {
    let gate = f::PublicGateProof::new(
        &f::CEILINGS,
        f::PublicGateProofParts {
            faults: 0,
            number: 3,
            kind: f::PublicGateKind::Check,
            blocks: false,
            proof: f::PublicProof::Passed,
        },
    )
    .expect("proof");
    let member = f::PublicMember::new(
        &f::CEILINGS,
        f::PublicMemberParts {
            head: None,
            left: None,
            judgement: None,
            task: 0,
            base,
            candidate,
            landed,
            gates: List::from_boxed(Box::new([gate])),
        },
    )
    .expect("member");
    let landing = f::PublicLanding::new(
        &f::CEILINGS,
        f::PublicLandingParts {
            search: None,
            combined: None,
            step: None,
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
#[must_use]
pub fn searching_landing() -> Box<[u8]> {
    let member = f::PublicMember::new(
        &f::CEILINGS,
        f::PublicMemberParts {
            task: 0,
            head: None,
            base: None,
            candidate: None,
            landed: None,
            gates: List::with_capacity(0),
            judgement: None,
            left: Some(f::PublicLeave::Blamed(
                f::PublicBlamed::new(
                    &f::CEILINGS,
                    f::PublicBlamedParts { gate: 3, red: [0x33; 32], green: [0x22; 32] },
                )
                .unwrap(),
            )),
        },
    )
    .unwrap();
    let landing = f::PublicLanding::new(
        &f::CEILINGS,
        f::PublicLandingParts {
            phase: f::PublicLandingPhase::Proving,
            hold: None,
            preparation: 1,
            members: List::from_boxed(Box::new([member])),
            stale_proofs: 0,
            holds_turn: false,
            combined: None,
            step: Some(f::PublicLandingStep::Probing),
            search: Some(
                f::PublicSearch::new(
                    &f::CEILINGS,
                    f::PublicSearchParts {
                        green: f::PublicSearchBound::new(
                            &f::CEILINGS,
                            f::PublicSearchBoundParts { member: 0, prefix: [0x22; 32] },
                        )
                        .unwrap(),
                        red: f::PublicSearchBound::new(
                            &f::CEILINGS,
                            f::PublicSearchBoundParts { member: 3, prefix: [0x33; 32] },
                        )
                        .unwrap(),
                        probe: Some([0x11; 32]),
                    },
                )
                .unwrap(),
            ),
        },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(landing.measure()).unwrap());
    landing.encode(&mut writer).unwrap();
    application(writer.finish())
}
#[must_use]
pub fn projects() -> Box<[u8]> {
    projects_with(true)
}
#[must_use]
pub fn projects_with(present: bool) -> Box<[u8]> {
    let project = j::PublicProject::new(
        &j::CEILINGS,
        j::PublicProjectParts { project: PROJECT, name: b"A public project".as_slice().into() },
    )
    .expect("project");
    let projects = j::PublicProjects::new(
        &j::CEILINGS,
        j::PublicProjectsParts {
            projects: if present { List::from_boxed(Box::new([project])) } else { List::with_capacity(0) },
            max_age: 8,
        },
    )
    .expect("projects");
    let mut writer = Writer::new(usize::try_from(projects.measure()).expect("small projects"));
    projects.encode(&mut writer).expect("projects");
    writer.finish()
}

pub const PROJECT: u64 = 0x3f2a_9c01_7654_3210;

pub const TASK: u64 = 0x9abc_def0_7654_3210;

#[must_use]
pub fn reader(page: Public) -> Reader {
    let limits = PublicLimits { jig: j::CEILINGS, forge: temper_client_protocol_wire::public_v1::CEILINGS };
    let mut reader = Reader::new(&limits);
    reader.step(Time::ZERO, PublicEvent::Open { page }, &mut Queue::uncapped());
    reader
}

pub fn answer(reader: &mut Reader, doc: Doc, heard: Heard) {
    reader.step(Time::ZERO, PublicEvent::Heard { doc, heard }, &mut Queue::uncapped());
}

#[must_use]
pub fn board() -> Box<[u8]> {
    let line = line(true);
    let goal = j::PublicGoal::new(
        &j::CEILINGS,
        j::PublicGoalParts { line, tree: List::from_boxed(Box::new([records::line(false)])), folded: 2 },
    )
    .expect("goal");
    let board = j::PublicBoard::new(
        &j::CEILINGS,
        j::PublicBoardParts {
            project: PROJECT,
            name: b"A public project".as_slice().into(),
            live: List::from_boxed(Box::new([goal])),
            live_folded: 0,
            ended: List::with_capacity(0),
        },
    )
    .expect("board");
    let mut writer = Writer::new(usize::try_from(board.measure()).expect("small board"));
    board.encode(&mut writer).expect("board");
    writer.finish()
}
