//! A worker codec behind the derived layouts (jig/docs/design/protocol/channel/layouts.md, 8).
use jig_protocol_worker_wire::{self as wire, Application, Carried, Root, Spoken, v22 as w};
use skein_channel::{Control, Frame, Term};
use skein_codec::{
    layout::{Mode, Plan, Workspace},
    schema::{Schema, parse},
};
use skein_lib::{Buffer, List, Reader, Secret, Writer};
use std::sync::OnceLock;
use w::Record;

const UP: u16 = 0x0100;
const DOWN: u16 = 0x0180;
const ROOM: u32 = 8192;
const APP: &str = "test-worker-wire";
const APP_TEXT: &str = "family test-worker-wire 3
    enum Form { specimen, engine_only }
    record Test { form: Form value: u32 note: u8 = 0 }";

/// The missing optional kinds are tested separately from the common kinds.
#[derive(Clone, Copy)]
pub enum Profile {
    Derived,
    MissingKinds,
}

struct Texts {
    frames: String,
    charter: String,
    transcript: String,
    application: String,
}

fn edit(text: &str, old: &str, new: &str) -> String {
    assert_eq!(text.matches(old).count(), 1, "one instance of {old}");
    text.replace(old, new)
}

fn derived(profile: Profile) -> &'static Texts {
    static COMMON: OnceLock<Texts> = OnceLock::new();
    static MISSING: OnceLock<Texts> = OnceLock::new();
    let cell = match profile {
        Profile::Derived => &COMMON,
        Profile::MissingKinds => &MISSING,
    };
    cell.get_or_init(|| {
        let frames = edit(
            wire::SCHEMA,
            "enum FromWorker {\n    hello: Hello",
            "record WorkerOnly { value: u8 }\nenum FromWorker {\n    worker_only: WorkerOnly, hello: Hello",
        );
        let frames = edit(&frames, "    assignment: Assignment", "    worker_only: WorkerOnly, assignment: Assignment");
        let frames = edit(&frames, "    refusal: option text 24 = none\n", "");
        let frames = edit(&frames, "agents: Agents }", "agents: Agents worker_mark: u16 = 21 }");
        let frames = edit(
            &frames,
            "words: text 1048576 finish_requested: bool }\nrecord RelayText",
            "words: text 12 finish_requested: bool }\nrecord RelayText",
        );
        let frames = match profile {
            Profile::Derived => frames,
            Profile::MissingKinds => edit(&edit(&frames, ", part: Part", ""), ", part_busy: PartBusy", ""),
        };
        let charter = edit(
            smith_charter::SCHEMA,
            "check_timeout: option duration",
            "dropped check_timeout: option duration = none\n    worker_mark: u16 = 19",
        );
        let charter =
            edit(&charter, "label: option text 64", "dropped label: option text 64 = none\n    worker_mark: u8 = 5");
        let charter = edit(
            &charter,
            "enum Form { report, verdict, change, failure }",
            "enum Form { worker_only, report, verdict, change }",
        );
        let transcript = edit(
            smith_transcript::SCHEMA,
            "system: option text 1048576",
            "dropped system: option text 1048576 = none\n    worker_mark: u16 = 11",
        );
        let transcript =
            edit(&transcript, "enum TurnKind { ordinary, window }", "enum TurnKind { worker_only, ordinary }");
        let application = edit(APP_TEXT, "note: u8 = 0", "worker_mark: u16 = 31");
        let application = edit(&application, "specimen, engine_only", "worker_only, specimen");
        Texts { frames, charter, transcript, application }
    })
}

fn spoken(text: &'static str) -> Spoken {
    let schema = parse(text.as_bytes()).expect("derived text");
    let name = core::str::from_utf8(schema.spelled(schema.family())).expect("family name");
    let name = text.split_ascii_whitespace().nth(1).filter(|declared| *declared == name).expect("family header");
    Spoken { name, version: schema.version(), layout: schema.fingerprints().family().digest(), text }
}

pub fn worker_wires(profile: Profile) -> List<Spoken> {
    let texts = derived(profile);
    let mut list = List::with_capacity(4);
    for current in &super::fixtures::spoken() {
        let changed = match current.name {
            "jig-worker-wire" => spoken(&texts.frames),
            "smith-charter" => spoken(&texts.charter),
            "smith-transcript" => spoken(&texts.transcript),
            // The opening requires smith-pieces unchanged (wires.md, 3).
            "smith-pieces" => current.clone(),
            _ => panic!("unknown jig wire"),
        };
        list.push(changed).expect("jig's four wires");
    }
    list
}

pub fn application(profile: Option<Profile>) -> Application {
    let mut app = Application::opaque(match profile {
        None => List::from_boxed(Box::new([spoken(APP_TEXT)])),
        Some(profile) => List::from_boxed(Box::new([spoken(&derived(profile).application)])),
    });
    for carried in [
        Carried::Items,
        Carried::Left,
        Carried::Saved,
        Carried::Preparation,
        Carried::JobSpec,
        Carried::JobResult,
        Carried::Grant,
        Carried::Ask,
        Carried::Answer,
    ] {
        app.roots.push(Root { carried, wire: APP, declaration: "Test" }).expect("one root per carried part");
    }
    app
}

struct Pass {
    plan: Plan,
    work: Workspace,
}
impl Pass {
    fn new(mode: Mode, from: &Schema, to: &Schema, root: &str) -> Self {
        let old = from.root(root.as_bytes(), None).expect("source root");
        let new = to.root(root.as_bytes(), None).expect("target root");
        let plan = Plan::between(mode, from, old, &[], to, new, &[]).expect("shim plan");
        let mut work = Workspace::new();
        assert!(work.fit(&plan));
        Self { plan, work }
    }
    fn value(&mut self, bytes: &[u8]) -> Box<[u8]> {
        let mut out = Buffer::with_capacity(ROOM);
        self.plan.convert(bytes, &mut out, &mut self.work).expect("representable shim value");
        out.to_boxed()
    }
    fn tag(&self, kind: u16, base: u16) -> Option<u16> {
        let tag = u8::try_from(kind.checked_sub(base)?).ok()?;
        Some(base + u16::from(self.plan.variant_tag(tag).ok()?))
    }
}

fn schemas(current: &str, derived: &str) -> (Schema, Schema) {
    (parse(current.as_bytes()).expect("current schema"), parse(derived.as_bytes()).expect("derived schema"))
}

/// Only the test transport sees this adapter. The worker component itself
/// still runs its own codec; the engine sees the derived credential and bytes.
pub struct Shim {
    limits: w::Limits,
    current: skein_channel::Schema,
    frame_up: Pass,
    frame_down: Pass,
    up_term: Pass,
    down_term: Pass,
    charter: Pass,
    result: Pass,
    turn_up: Pass,
    turn_down: Pass,
    app_up: Pass,
    app_down: Pass,
    worker_turn_only: bool,
}
impl Shim {
    pub fn new(profile: Profile, limits: &w::Limits) -> Self {
        let text = derived(profile);
        let (ours, theirs) = schemas(wire::SCHEMA, &text.frames);
        let (charter, other_charter) = schemas(smith_charter::SCHEMA, &text.charter);
        let (transcript, other_transcript) = schemas(smith_transcript::SCHEMA, &text.transcript);
        let (app, other_app) = schemas(APP_TEXT, &text.application);
        Self {
            limits: *limits,
            current: wire::schema(limits).expect("current fixture table"),
            frame_up: Pass::new(Mode::Keep, &ours, &theirs, "FromWorker"),
            frame_down: Pass::new(Mode::Read, &theirs, &ours, "FromEngine"),
            up_term: Pass::new(Mode::Read, &theirs, &ours, "FromWorker"),
            down_term: Pass::new(Mode::Keep, &ours, &theirs, "FromEngine"),
            charter: Pass::new(Mode::Read, &other_charter, &charter, "Charter"),
            result: Pass::new(Mode::Keep, &charter, &other_charter, "RunResult"),
            turn_up: Pass::new(Mode::Keep, &transcript, &other_transcript, "Turn"),
            turn_down: Pass::new(Mode::Read, &other_transcript, &transcript, "Turn"),
            app_up: Pass::new(Mode::Keep, &app, &other_app, "Test"),
            app_down: Pass::new(Mode::Read, &other_app, &app, "Test"),
            worker_turn_only: false,
        }
    }

    pub fn next_turn_worker_only(&mut self) {
        self.worker_turn_only = true;
    }

    pub fn up(&mut self, kind: u16, body: &[u8]) -> Frame {
        if kind < UP {
            return self.control(kind, body, true);
        }
        let value = tagged(UP, kind, body);
        let mut record = w::FromWorker::decode(&self.limits, &mut Reader::new(&value)).expect("current worker frame");
        self.parts_up(&mut record);
        let value = record.encode(&self.limits).expect("worker record with derived parts");
        from_tagged(UP, &self.frame_up.value(&value))
    }

    pub fn down(&mut self, kind: u16, body: &[u8]) -> Frame {
        if kind < UP {
            return self.control(kind, body, false);
        }
        let value = self.frame_down.value(&tagged(DOWN, kind, body));
        let mut record = w::FromEngine::decode(&self.limits, &mut Reader::new(&value)).expect("current engine frame");
        self.parts_down(&mut record);
        from_tagged(DOWN, &record.encode(&self.limits).expect("engine record with current parts"))
    }

    fn control(&self, kind: u16, body: &[u8], up: bool) -> Frame {
        use skein_channel::{control_frame, decode_control};
        let limits = super::fixtures::channel_limits();
        if let Control::Terms { entries } = decode_control(kind, body, &limits).expect("channel control") {
            let mut mapped = List::with_capacity(entries.len() + 1);
            if up {
                // The derived worker advertises its own extra receive kind.
                mapped.push(Term { kind: DOWN, largest: 1 }).expect("extra worker kind");
            }
            for term in &entries {
                let kind = if up { self.down_term.tag(term.kind, DOWN) } else { self.up_term.tag(term.kind, UP) };
                if let Some(kind) = kind {
                    // Only Figures acquired extra bytes in a receive term.
                    let grows = up && term.kind == wire::Kind::Figures.number();
                    let largest = if up {
                        term.largest + if grows { 2 } else { 0 }
                    } else {
                        self.current.kind(kind).expect("mapped worker kind").largest
                    };
                    mapped.push(Term { kind, largest }).expect("one per receive term");
                }
            }
            control_frame(&Control::Terms { entries: mapped }, &limits).expect("mapped terms")
        } else {
            super::fixtures::frame(kind, body)
        }
    }

    fn parts_up(&mut self, record: &mut w::FromWorker) {
        match record {
            w::FromWorker::Turn(turn) => {
                turn.body = self.turn_up.value(&turn.body);
                if std::mem::take(&mut self.worker_turn_only) {
                    // The versioned transcript begins with its version and TurnKind.
                    turn.body[2] = 0;
                }
            }
            w::FromWorker::Part(part) => part.body = self.turn_up.value(&part.body),
            w::FromWorker::Ask(ask) => ask.bytes = self.app_up.value(&ask.bytes),
            w::FromWorker::Answer(answer) => match &mut answer.ending {
                w::Ending::Ended(ended) => {
                    ended.outcome = self.result.value(&ended.outcome);
                    self.work(&mut ended.work);
                }
                w::Ending::Parked(parked) => self.work(&mut parked.work),
                w::Ending::Failed(failed) => {
                    self.work(&mut failed.work);
                    if let w::Failure::Unprepared(value) = &mut failed.failure {
                        self.resource(&mut value.value);
                    }
                }
                w::Ending::Refused(_) => {}
            },
            w::FromWorker::JobAnswer(answer) => match &mut answer.ending {
                w::JobEnding::Finished(finished) => {
                    finished.result = self.app_up.value(&finished.result);
                    self.work(&mut finished.work);
                }
                w::JobEnding::Failed(failed) => {
                    self.work(&mut failed.work);
                    if let w::JobFailure::Unprepared(value) = &mut failed.failure {
                        self.resource(&mut value.value);
                    }
                }
                w::JobEnding::Refused(_) => {}
            },
            _ => {}
        }
    }

    fn work(&mut self, work: &mut w::Work) {
        for part in [&mut work.left, &mut work.saved].into_iter().flatten() {
            *part = self.app_up.value(part);
        }
    }
    fn resource(&mut self, value: &mut w::Preparation) {
        if let w::Preparation::Permanent(resource) = value
            && let Some(part) = &mut resource.resource
        {
            *part = self.app_up.value(part);
        }
    }

    fn parts_down(&mut self, record: &mut w::FromEngine) {
        match record {
            w::FromEngine::Assignment(assignment) => {
                assignment.charter = self.charter.value(&assignment.charter);
                let mut turns = Reader::new(&assignment.transcript);
                let mut window = Vec::new();
                while let Some(turn) = skein_codec::piece::split(&mut turns).expect("bounded turn") {
                    let turn = self.turn_down.value(&turn);
                    window.extend(u32::try_from(turn.len()).expect("fixture turn length").to_be_bytes());
                    window.extend_from_slice(&turn);
                }
                assignment.transcript = window.into_boxed_slice();
                let mut kept = wire::parts::decode(&assignment.parts, &self.limits).expect("bounded kept parts");
                for index in 0..kept.len() {
                    let part = kept.get_mut(index).expect("kept part");
                    part.body = self.turn_down.value(&part.body);
                }
                assignment.parts = wire::parts::encode(kept.as_slice(), &self.limits).expect("current kept parts");
                self.assignment_parts(assignment.workspace.as_mut(), &mut assignment.grants);
            }
            w::FromEngine::Job(job) => {
                job.spec = self.app_down.value(&job.spec);
                self.assignment_parts(job.workspace.as_mut(), &mut job.grants);
            }
            w::FromEngine::Grant(grant) => self.grant(&mut grant.grant),
            w::FromEngine::Asked(asked) => {
                if let w::AskAnswer::Bytes(bytes) = &mut asked.answer {
                    bytes.bytes = self.app_down.value(&bytes.bytes);
                }
            }
            _ => {}
        }
    }

    fn assignment_parts(&mut self, workspace: Option<&mut w::Workspace>, grants: &mut List<w::Grant>) {
        if let Some(workspace) = workspace {
            workspace.items = self.app_down.value(&workspace.items);
        }
        for index in 0..grants.len() {
            self.grant(grants.get_mut(index).expect("grant"));
        }
    }
    fn grant(&mut self, grant: &mut w::Grant) {
        if let w::GrantValue::Application(value) = &mut grant.value {
            value.secret = Secret::new(self.app_down.value(value.secret.expose()));
        }
    }
}

fn tagged(base: u16, kind: u16, body: &[u8]) -> Box<[u8]> {
    let tag = u8::try_from(kind.checked_sub(base).expect("direction base")).expect("one-byte tag");
    let mut bytes = Writer::new(body.len() + 1);
    bytes.put(&[tag]).expect("tag room");
    bytes.put(body).expect("body room");
    bytes.finish()
}
fn from_tagged(base: u16, tagged: &[u8]) -> Frame {
    let (&tag, body) = tagged.split_first().expect("record tag");
    super::fixtures::frame(base + u16::from(tag), body)
}
