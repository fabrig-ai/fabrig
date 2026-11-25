//! Independent record and carried-part fixtures for the worker's derived layouts.
use crate::layouts::{Peer, Plans};
use crate::{Carried, Root, v22 as wire};
use skein_codec::{layout::Mode, schema::Schema};
use skein_lib::{Buffer, Duration, List, Rng, Secret};
use wire::{FromEngine, FromWorker};

use super::super::fixtures as current;
use super::super::layouts::samples::{ROOM, Shim, draw, parse, replace, richer, stated};

const APP: &str = "family convert-test-app 1
    enum Shape { common, engine_only }
    record Down { shape: Shape number: u16 engine_note: u32 = 7 }
    record Up { shape: Shape number: u16 engine_note: u64 = 7 }";
const TEXTS: [&str; 5] =
    [crate::SCHEMA, smith_channel::PIECES_SCHEMA, smith_charter::SCHEMA, smith_transcript::SCHEMA, APP];

pub(super) const LIMITS: wire::Limits = wire::Limits {
    assignment_charter: ROOM,
    assignment_transcript: ROOM,
    assignment_parts: ROOM,
    kept_part_body: ROOM,
    turn_body: ROOM,
    part_body: ROOM,
    ended_outcome: ROOM,
    ask_bytes: 16,
    ask_bytes_bytes: 16,
    ..current::LIMITS
};

pub(super) struct Layouts {
    pub engine: [Schema; 5],
    pub worker: [Schema; 5],
}

impl Layouts {
    pub fn new() -> Self {
        let frames = replace(&richer(crate::SCHEMA), "    fresh: bool = false\n", "");
        let charter = replace(TEXTS[2], "    check_timeout: option duration", "    worker_note: u16 = 6");
        let charter = replace(&charter, "    label: option text 64", "    worker_note: u16 = 8");
        let charter = replace(
            &charter,
            "enum Form { report, verdict, change, failure }",
            "enum Form { worker_only, report, verdict, change }",
        );
        let transcript = replace(TEXTS[3], "    system: option text 1048576", "    worker_note: u16 = 9");
        let transcript =
            replace(&transcript, "enum TurnKind { ordinary, window }", "enum TurnKind { worker_only, ordinary }");
        let app = replace(APP, "engine_note: u32 = 7", "worker_note: u8 = 8");
        let app = replace(&app, "engine_note: u64 = 7", "worker_note: u16 = 8");
        let app = replace(&app, "enum Shape { common, engine_only }", "enum Shape { worker_only, common }");
        Self {
            engine: TEXTS.map(parse),
            // smith-pieces is kept as written (wires.md, 3).
            worker: [parse(&frames), parse(TEXTS[1]), parse(&charter), parse(&transcript), parse(&app)],
        }
    }

    pub fn plans() -> Plans {
        let mut roots = List::with_capacity(9);
        for (carried, declaration) in [
            (Carried::Items, "Down"),
            (Carried::JobSpec, "Down"),
            (Carried::Grant, "Down"),
            (Carried::Answer, "Down"),
            (Carried::Left, "Up"),
            (Carried::Saved, "Up"),
            (Carried::Preparation, "Up"),
            (Carried::JobResult, "Up"),
            (Carried::Ask, "Up"),
        ] {
            roots.push(Root { carried, wire: "convert-test-app", declaration }).expect("one root per part");
        }
        Plans::new(&TEXTS, &roots).expect("engine texts and roots")
    }

    pub fn peer(&self) -> Peer {
        Self::plans().judge(&self.worker.each_ref().map(stated)).expect("worker layouts plan")
    }

    pub fn pieces_change() -> Schema {
        let text = replace(TEXTS[1], "marker: option Marker explanation:", "worker_note: u16 = 8 explanation:");
        parse(&replace(&text, "failed: DeliveryFailure, stale", "failed: DeliveryFailure, worker_only"))
    }
}

// Both sides of each fixture are built from the same values. The worker's
// bytes are filled by an independent whole-value plan before the records are
// encoded; the frame plan sees only the resulting opaque fields.
pub(super) struct Payloads {
    pub charter: Box<[u8]>,
    pub turns: [Box<[u8]>; 2],
    pub outcome: Box<[u8]>,
    pub items: Box<[u8]>,
    pub spec: Box<[u8]>,
    pub grant: Box<[u8]>,
    pub answer: Box<[u8]>,
    pub left: Box<[u8]>,
    pub saved: Box<[u8]>,
    pub resource: Box<[u8]>,
    pub result: Box<[u8]>,
    pub ask: Box<[u8]>,
}

impl Payloads {
    pub fn both(layouts: &Layouts) -> (Self, Self) {
        let mut rng = Rng::new(0x2205_0801);
        let charter = draw_root(&layouts.engine[2], "Charter", &mut rng);
        let mut outcome = draw_root(&layouts.engine[2], "RunResult", &mut rng);
        *outcome.get_mut(2).expect("form follows the version") = 0;
        let turns = [turn(&layouts.engine[3], &mut rng, 0), turn(&layouts.engine[3], &mut rng, 0)];
        let ours = Self {
            charter,
            turns,
            outcome,
            items: down(11),
            spec: down(12),
            grant: down(13),
            answer: down(14),
            left: up(21),
            saved: up(22),
            resource: up(23),
            result: up(24),
            ask: up(25),
        };
        let charter = |root, bytes: &[u8]| crossing(root, bytes, &layouts.engine[2], &layouts.worker[2]);
        let turn = |bytes: &[u8]| crossing("Turn", bytes, &layouts.engine[3], &layouts.worker[3]);
        let app = |root, bytes: &[u8]| crossing(root, bytes, &layouts.engine[4], &layouts.worker[4]);
        let theirs = Self {
            charter: charter("Charter", &ours.charter),
            turns: [turn(&ours.turns[0]), turn(&ours.turns[1])],
            outcome: charter("RunResult", &ours.outcome),
            items: app("Down", &ours.items),
            spec: app("Down", &ours.spec),
            grant: app("Down", &ours.grant),
            answer: app("Down", &ours.answer),
            left: app("Up", &ours.left),
            saved: app("Up", &ours.saved),
            resource: app("Up", &ours.resource),
            result: app("Up", &ours.result),
            ask: app("Up", &ours.ask),
        };
        (ours, theirs)
    }

    pub fn assignment(&self) -> wire::Assignment {
        let mut record = current::assignment_resumed();
        record.workspace = Some(self.workspace());
        record.charter = self.charter.clone();
        record.transcript = window(&self.turns);
        record.grants = list(self.grant());
        record.parts = parts(&self.turns);
        record
    }

    pub fn engine(&self) -> [FromEngine; 5] {
        [
            FromEngine::Assignment(self.assignment()),
            FromEngine::Job(wire::Job {
                task: 2,
                attempt: 3,
                kind: 4,
                spec: self.spec.clone(),
                deadline: Duration::from_secs(12),
                workspace: Some(self.workspace()),
                grants: list(self.grant()),
            }),
            FromEngine::Grant(wire::Granted { task: 4, attempt: 5, grant: self.grant() }),
            FromEngine::ReadList(wire::ReadList { read: 6, endpoint: 7, grant: self.grant() }),
            FromEngine::Asked(wire::Asked {
                run: 6,
                attempt: 7,
                number: 8,
                answer: wire::AskAnswer::Bytes(wire::AskBytes { bytes: self.answer.clone() }),
            }),
        ]
    }

    pub fn worker(&self) -> [FromWorker; 8] {
        [
            FromWorker::Turn(wire::Turn {
                task: 2,
                attempt: 3,
                number: 4,
                spent: 5,
                read: Some(6),
                opens_window: true,
                refused: List::uncapped(),
                body: self.turns[0].clone(),
            }),
            FromWorker::Part(wire::Part {
                task: 4,
                attempt: 5,
                number: 6,
                turn: 7,
                thread: 8,
                call: 9,
                kind: wire::PartKind::Result,
                body: self.turns[1].clone(),
            }),
            Self::run_answer(wire::Ending::Ended(wire::Ended { outcome: self.outcome.clone(), work: self.work() })),
            Self::run_answer(wire::Ending::Parked(wire::Parked { work: self.work() })),
            Self::run_answer(wire::Ending::Failed(wire::Failed {
                failure: wire::Failure::Unprepared(self.preparation()),
                detail: Box::from(b"failed".as_slice()),
                work: self.work(),
                model: None,
            })),
            Self::job_answer(wire::JobEnding::Finished(wire::JobFinished {
                result: self.result.clone(),
                work: self.work(),
            })),
            Self::job_answer(wire::JobEnding::Failed(wire::JobFailed {
                failure: wire::JobFailure::Unprepared(self.preparation()),
                detail: Box::from(b"failed".as_slice()),
                work: self.work(),
            })),
            FromWorker::Ask(wire::Ask { run: 8, attempt: 9, number: 10, bytes: self.ask.clone() }),
        ]
    }

    fn workspace(&self) -> wire::Workspace {
        wire::Workspace { workstream: 7, items: self.items.clone() }
    }

    fn grant(&self) -> wire::Grant {
        let secret = Secret::new(self.grant.clone());
        current::grant(wire::GrantValue::Application(wire::ApplicationValue { secret }), 5, 9)
    }

    fn work(&self) -> wire::Work {
        wire::Work { left: Some(self.left.clone()), saved: Some(self.saved.clone()) }
    }

    fn preparation(&self) -> wire::PreparationValue {
        wire::PreparationValue {
            value: wire::Preparation::Permanent(wire::Resource { resource: Some(self.resource.clone()) }),
        }
    }

    fn run_answer(ending: wire::Ending) -> FromWorker {
        FromWorker::Answer(wire::Answer {
            task: 10,
            attempt: 11,
            turns: 12,
            spent: 13,
            read: None,
            refused: List::uncapped(),
            ending,
        })
    }

    fn job_answer(ending: wire::JobEnding) -> FromWorker {
        FromWorker::JobAnswer(wire::JobAnswer { task: 14, attempt: 15, ending })
    }
}

fn list(grant: wire::Grant) -> List<wire::Grant> {
    let mut grants = List::with_capacity(1);
    grants.push(grant).expect("one grant");
    grants
}

fn down(number: u8) -> Box<[u8]> {
    Box::new([0, 0, number, 0, 0, 0, 7])
}

fn up(number: u8) -> Box<[u8]> {
    Box::new([0, 0, number, 0, 0, 0, 0, 0, 0, 0, 7])
}

pub(super) fn turn(schema: &Schema, rng: &mut Rng, kind: u8) -> Box<[u8]> {
    let mut turn = draw_root(schema, "Turn", rng);
    *turn.get_mut(2).expect("turn kind follows the version") = kind;
    turn
}

fn draw_root(schema: &Schema, root: &str, rng: &mut Rng) -> Box<[u8]> {
    let mut out = Buffer::with_capacity(ROOM);
    draw(schema, schema.find(root.as_bytes()).expect("test root"), rng, &mut out, true);
    out.to_boxed()
}

fn crossing(root: &str, bytes: &[u8], engine: &Schema, worker: &Schema) -> Box<[u8]> {
    let mut derived = Buffer::with_capacity(ROOM);
    Shim::new(Mode::Keep, engine, worker, root).convert(bytes, &mut derived).expect("part at worker layout");
    assert_ne!(derived.as_slice(), bytes, "{root} must visibly change its layout");
    let mut restored = Buffer::with_capacity(ROOM);
    Shim::new(Mode::Read, worker, engine, root).convert(derived.as_slice(), &mut restored).expect("read derived part");
    assert_eq!(restored.as_slice(), bytes, "{root} arrives as itself");
    derived.to_boxed()
}

pub(super) fn window(turns: &[Box<[u8]>]) -> Box<[u8]> {
    let mut out = Buffer::with_capacity(ROOM);
    for turn in turns {
        out.put(&u32::try_from(turn.len()).expect("small turn").to_be_bytes()).expect("room for turn length");
        out.put(turn).expect("room for turn");
    }
    out.to_boxed()
}

pub(super) fn parts(turns: &[Box<[u8]>]) -> Box<[u8]> {
    let mut items = List::uncapped();
    for (turn, bytes) in turns.iter().enumerate() {
        items
            .push(wire::KeptPart {
                attempt: 4,
                turn: u32::try_from(turn).expect("few test turns"),
                thread: 6,
                call: 7,
                kind: wire::PartKind::Answer,
                body: bytes.clone(),
            })
            .expect("one item per test turn");
    }
    crate::parts::encode(items.as_slice(), &LIMITS).expect("small parts envelope")
}
