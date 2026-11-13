//! Public wire records used by the anonymous view (temper/docs/design/client/web/public.md, 4).
use super::{PROJECT, TASK};
use jig_client_protocol_wire::public_v1 as j;
use skein_lib::{List, Writer};
use temper_client_protocol_wire::public_v1 as f;
pub(super) const HEAD: [u8; 32] = [
    0x78, 0x9a, 0xda, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub(super) fn line(goal: bool) -> j::PublicLine {
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
pub(super) fn card() -> Box<[u8]> {
    card_line(line(false))
}
pub(super) fn card_line(own: j::PublicLine) -> Box<[u8]> {
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
fn used() -> f::PublicUsed {
    f::PublicUsed::new(&f::CEILINGS, f::PublicUsedParts { used: 0, limit: 2 }).expect("counters")
}
pub(super) fn change() -> Box<[u8]> {
    change_commits(Some(HEAD), None)
}
pub(super) fn change_commits(head: Option<[u8; 32]>, merge: Option<[u8; 32]>) -> Box<[u8]> {
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
    landings_with(true, true)
}
pub(super) fn landings_with(queue_present: bool, landed: bool) -> Box<[u8]> {
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
pub(super) fn queue_merge() -> (Box<[u8]>, Box<[u8]>) {
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
pub(super) fn landing() -> Box<[u8]> {
    landing_commits(None, None, Some(HEAD))
}
pub(super) fn landing_commits(
    base: Option<[u8; 32]>,
    candidate: Option<[u8; 32]>,
    landed: Option<[u8; 32]>,
) -> Box<[u8]> {
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
pub(super) fn projects() -> Box<[u8]> {
    projects_with(true)
}
pub(super) fn projects_with(present: bool) -> Box<[u8]> {
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
