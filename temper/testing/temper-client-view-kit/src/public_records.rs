//! Public wire records used by the anonymous view (temper/docs/design/client/web/public.md, 4).
use jig_client_protocol_wire::public_v2 as j;
use skein_lib::{List, Queue, Time, Writer};
use temper_client_domain::{
    address::Public,
    public::{Doc, Heard, PublicEvent, PublicLimits, Reader},
};
use temper_client_protocol_wire::public_v1 as f;
use temper_client_protocol_wire::v1::Record;
pub const HEAD: [u8; 32] = [
    0x78, 0x9a, 0xda, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
#[must_use]
pub fn line(goal: bool) -> j::PublicLine {
    let counts = j::PublicCounts { working: 0, waiting: 0, held: 0, closing: 0, ended: 0 };
    j::PublicLine {
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
    }
}
#[must_use]
pub fn card() -> Box<[u8]> {
    card_line(line(false))
}
#[must_use]
pub fn card_line(own: j::PublicLine) -> Box<[u8]> {
    card_with(own, List::from_boxed(Box::new([PROJECT])), line(true))
}
#[must_use]
pub fn card_with(own: j::PublicLine, lineage: List<u64>, delegate: j::PublicLine) -> Box<[u8]> {
    let card = j::PublicCard { line: own, lineage, delegates: List::from_boxed(Box::new([delegate])), folded: 0 };
    let mut writer = Writer::new(
        usize::try_from(card.measure(&jig_client_protocol_wire::public_v2::CEILINGS).expect("bounded fixture"))
            .expect("small card"),
    );
    card.write(&jig_client_protocol_wire::public_v2::CEILINGS, &mut writer).expect("card");
    writer.finish()
}
#[must_use]
pub fn used() -> f::PublicUsed {
    f::PublicUsed { used: 0, limit: 2 }
}
#[must_use]
pub fn change() -> Box<[u8]> {
    change_commits(Some(HEAD), None)
}
#[must_use]
pub fn change_commits(head: Option<[u8; 32]>, merge: Option<[u8; 32]>) -> Box<[u8]> {
    change_with_step(f::PublicStep::Checking, head, merge)
}
#[must_use]
pub fn change_with_step(step: f::PublicStep, head: Option<[u8; 32]>, merge: Option<[u8; 32]>) -> Box<[u8]> {
    let gate =
        f::PublicGate { number: 3, kind: f::PublicGateKind::Review, blocks: true, verdict: f::PublicVerdict::Approved };
    let change = f::PublicChange {
        task: 0,
        head,
        step,
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
    };
    let mut writer =
        Writer::new(usize::try_from(change.measure(&f::CEILINGS).expect("within ceilings")).expect("small change"));
    change.write(&f::CEILINGS, &mut writer).expect("change");
    application(writer.finish())
}
#[must_use]
pub fn application(record: Box<[u8]>) -> Box<[u8]> {
    let app = j::PublicApplication { tasks: List::from_boxed(Box::new([TASK, PROJECT])), record };
    let mut writer = Writer::new(
        usize::try_from(app.measure(&jig_client_protocol_wire::public_v2::CEILINGS).expect("bounded fixture"))
            .expect("small application"),
    );
    app.write(&jig_client_protocol_wire::public_v2::CEILINGS, &mut writer).expect("application");
    writer.finish()
}
#[must_use]
pub fn landings() -> Box<[u8]> {
    landings_with(true, true)
}
#[must_use]
pub fn landings_with(queue_present: bool, landed: bool) -> Box<[u8]> {
    landings_record(queue_present, landed, if landed { Some(0) } else { None }, 0)
}

/// A stock queue may have a waiting member and a recent merge, but does not hold the turn.
#[must_use]
pub fn stock_landings(queue_present: bool, landed: bool) -> Box<[u8]> {
    landings_record(queue_present, landed, None, 1)
}

fn landings_record(queue_present: bool, landed: bool, turn: Option<u16>, member: u16) -> Box<[u8]> {
    let queued = f::PublicQueued { task: 0, goal: 1, ready_since: 0 };
    let queue = f::PublicQueue { changes: List::from_boxed(Box::new([queued])) };
    let line = f::PublicLandingLine {
        members: List::from_boxed(Box::new([member])),
        phase: f::PublicLandingPhase::Landed,
        landed: List::from_boxed(Box::new([HEAD])),
    };
    let landings = f::PublicLandings {
        turn,
        landings: if landed { List::from_boxed(Box::new([line])) } else { List::with_capacity(0) },
        queues: if queue_present { List::from_boxed(Box::new([queue])) } else { List::with_capacity(0) },
    };
    let mut writer =
        Writer::new(usize::try_from(landings.measure(&f::CEILINGS).expect("within ceilings")).expect("small landings"));
    landings.write(&f::CEILINGS, &mut writer).expect("landings");
    application(writer.finish())
}
// A stock queue merge has no turn, preparation, candidates or gate proofs.
#[must_use]
pub fn queue_merge() -> (Box<[u8]>, Box<[u8]>) {
    let list = stock_landings(false, true);
    let member = f::PublicMember {
        head: None,
        left: None,
        judgement: None,
        task: 1,
        base: None,
        candidate: None,
        landed: Some(HEAD),
        gates: List::with_capacity(0),
    };
    let landing = f::PublicLanding {
        search: None,
        combined: None,
        step: None,
        phase: f::PublicLandingPhase::Landed,
        hold: None,
        preparation: 0,
        members: List::from_boxed(Box::new([member])),
        stale_proofs: 0,
        holds_turn: false,
    };
    let mut writer =
        Writer::new(usize::try_from(landing.measure(&f::CEILINGS).expect("within ceilings")).expect("small landing"));
    landing.write(&f::CEILINGS, &mut writer).expect("landing");
    (list, application(writer.finish()))
}
#[must_use]
pub fn landing() -> Box<[u8]> {
    landing_commits(None, None, Some(HEAD))
}
#[must_use]
pub fn landing_commits(base: Option<[u8; 32]>, candidate: Option<[u8; 32]>, landed: Option<[u8; 32]>) -> Box<[u8]> {
    let gate = f::PublicGateProof {
        faults: 0,
        number: 3,
        kind: f::PublicGateKind::Check,
        blocks: false,
        proof: f::PublicProof::Passed,
    };
    let member = f::PublicMember {
        head: None,
        left: None,
        judgement: None,
        task: 0,
        base,
        candidate,
        landed,
        gates: List::from_boxed(Box::new([gate])),
    };
    let landing = f::PublicLanding {
        search: None,
        combined: None,
        step: None,
        phase: f::PublicLandingPhase::Landed,
        hold: None,
        preparation: 0,
        members: List::from_boxed(Box::new([member])),
        stale_proofs: 2,
        holds_turn: true,
    };
    let mut writer =
        Writer::new(usize::try_from(landing.measure(&f::CEILINGS).expect("within ceilings")).expect("small landing"));
    landing.write(&f::CEILINGS, &mut writer).expect("landing");
    application(writer.finish())
}
#[must_use]
pub fn searching_landing() -> Box<[u8]> {
    let member = f::PublicMember {
        task: 0,
        head: None,
        base: None,
        candidate: None,
        landed: None,
        gates: List::with_capacity(0),
        judgement: None,
        left: Some(f::PublicLeave::Blamed(f::PublicBlamed { gate: 3, red: [0x33; 32], green: [0x22; 32] })),
    };
    let landing = f::PublicLanding {
        phase: f::PublicLandingPhase::Proving,
        hold: None,
        preparation: 1,
        members: List::from_boxed(Box::new([member])),
        stale_proofs: 0,
        holds_turn: false,
        combined: None,
        step: Some(f::PublicLandingStep::Probing),
        search: Some(f::PublicSearch {
            green: f::PublicSearchBound { member: 0, prefix: [0x22; 32] },
            red: f::PublicSearchBound { member: 3, prefix: [0x33; 32] },
            probe: Some([0x11; 32]),
        }),
    };
    let mut writer = Writer::new(usize::try_from(landing.measure(&f::CEILINGS).expect("within ceilings")).unwrap());
    landing.write(&f::CEILINGS, &mut writer).unwrap();
    application(writer.finish())
}
#[must_use]
pub fn projects() -> Box<[u8]> {
    projects_with(true)
}
#[must_use]
pub fn projects_with(present: bool) -> Box<[u8]> {
    projects_named(if present { &[(PROJECT, PROJECT_NAME)] } else { &[] })
}
#[must_use]
pub fn projects_named(named: &[(u64, &[u8])]) -> Box<[u8]> {
    let mut projects = List::with_capacity(u32::try_from(named.len()).expect("bounded project list"));
    for &(project, name) in named {
        projects.push(j::PublicProject { project, name: name.into() }).expect("one row per named project");
    }
    let projects = j::PublicProjects { projects, max_age: 8 };
    let mut writer = Writer::new(
        usize::try_from(projects.measure(&jig_client_protocol_wire::public_v2::CEILINGS).expect("bounded fixture"))
            .expect("small projects"),
    );
    projects.write(&jig_client_protocol_wire::public_v2::CEILINGS, &mut writer).expect("projects");
    writer.finish()
}

pub const PROJECT: u64 = 0x3f2a_9c01_7654_3210;
pub const PROJECT_NAME: &[u8] = b"A public project";

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
    board_with(line(true), line(false))
}
#[must_use]
pub fn board_with(line: j::PublicLine, delegate: j::PublicLine) -> Box<[u8]> {
    let goal = j::PublicGoal { line, tree: List::from_boxed(Box::new([delegate])), folded: 2 };
    let board = j::PublicBoard {
        project: PROJECT,
        name: PROJECT_NAME.into(),
        live: List::from_boxed(Box::new([goal])),
        live_folded: 0,
        ended: List::with_capacity(0),
    };
    let mut writer = Writer::new(
        usize::try_from(board.measure(&jig_client_protocol_wire::public_v2::CEILINGS).expect("bounded fixture"))
            .expect("small board"),
    );
    board.write(&jig_client_protocol_wire::public_v2::CEILINGS, &mut writer).expect("board");
    writer.finish()
}
