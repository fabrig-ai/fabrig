//! Shared startup vocabulary, bounded problems and lines (protocol/services.md, 3).
use crate::limits;
use alloc::boxed::Box;
use jig_protocol_config as config;
use skein_lib::{Decimal, List, Writer};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum What<E> {
    Jig(config::What),
    Standalone,
    WorkerAlone,
    Absolute,
    Profile,
    Override(limits::Refused),
    Against {
        other: &'static [u8],
    },
    Checkout {
        conflicts: u32,
        path_bytes: u32,
        repositories: u32,
        stdout: Option<(u32, u32)>,
        stdout_of: &'static [u8],
        agent_conflicts: Option<(u32, u32)>,
        agent_paths: Option<(u32, u32)>,
        saved: Option<(u32, u32)>,
        engine: Option<(u32, u32)>,
        worker: Option<(u32, u32)>,
    },
    Past {
        bytes: u64,
        bound: u64,
    },
    Short {
        bytes: u64,
        of: &'static [u8],
        needs: u64,
    },
    Within {
        bytes: u64,
        limit: &'static [u8],
        bound: u64,
    },
    CallInput {
        bytes: u64,
        input: u64,
        ask: u64,
        reply: u64,
    },
    Half(E),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Problem<W> {
    pub path: config::Path,
    pub what: W,
    pub file: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Problems<W> {
    pub problems: List<Problem<W>>,
    pub past: u32,
}
impl<W> Problems<W> {
    #[must_use]
    pub fn empty() -> Self {
        Self { problems: List::with_capacity(config::PROBLEMS), past: 0 }
    }
    pub fn push(&mut self, path: config::Path, what: W) {
        self.push_file(path, what, None);
    }
    pub fn push_file(&mut self, path: config::Path, what: W, file: Option<Box<[u8]>>) {
        if self.problems.push(Problem { path, what, file }).is_err() {
            self.past = self.past.saturating_add(1);
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.problems.is_empty() && self.past == 0
    }
}
impl<E> Problems<What<E>> {
    #[must_use]
    pub fn jig(found: config::Problems, prefix: &config::Path) -> Self {
        let mut result = Self::empty();
        result.append(found, prefix);
        result
    }
    pub fn append(&mut self, found: config::Problems, prefix: &config::Path) {
        for problem in &found.problems {
            let mut path = prefix.clone();
            for segment in &problem.path.segments {
                path = path.child(segment.clone());
            }
            self.push(path, What::Jig(problem.what));
        }
        self.past = self.past.saturating_add(found.past);
    }
}
fn words<E>(what: &What<E>) -> &'static [u8] {
    match what {
 What::Standalone => b"standalone takes one worker, joined to the engine by the in-process name \"workers\"",
 What::WorkerAlone => b"a worker alone takes one worker dialling a plaintext loopback socket, and only application.workers and application.limits",
 What::Absolute => b"must be an absolute path",
 What::Profile => b"the only profile is \"small\"",
 What::Override(limits::Refused::Unknown) => b"no limit of temper's has this name",
 What::Override(limits::Refused::Range) => b"outside its type",
 What::Against { .. } => b"must agree with ",
 What::Jig(what) => jig_words(*what),
 What::Checkout { .. } | What::Past { .. } | What::Within { .. } | What::Short { .. } | What::CallInput { .. } | What::Half(_) => b"",
 }
}
pub fn put(writer: &mut Writer, length: &mut usize, measuring: bool, bytes: &[u8]) {
    *length = length.checked_add(bytes.len()).expect("bounded diagnostic fits usize");
    if !measuring {
        writer.put(bytes).expect("diagnostic was measured");
    }
}
fn jig_words(what: config::What) -> &'static [u8] {
    match what {
        config::What::Because(words) => words,
        config::What::PolicyEnvelope { .. }
        | config::What::AmendmentEnvelope { .. }
        | config::What::CodecCeiling { .. }
        | config::What::Capacity { .. }
        | config::What::CodecArithmetic
        | config::What::StoreCodec => b"",
        config::What::Json => b"invalid JSON",
        config::What::Stored => b"settings live in the store; read them with temper ctl DIR settings show",
        config::What::Unknown => b"unknown member",
        config::What::Missing => b"missing",
        config::What::Duplicate => b"duplicate",
        config::What::Kind => b"wrong value kind",
        config::What::Bound => b"past its bound",
        config::What::SlotsPastHub => {
            b"past this worker's limits.overrides.hub.slots, which must be at least its slots"
        }
        config::What::Form | config::What::ProviderKey { .. } => b"invalid form",
        config::What::Unique => b"must be unique",
        config::What::Refers => b"unknown reference",
        config::What::Plaintext => b"plaintext requires loopback or in-process",
        config::What::Confidential => b"a confidential registration is refused: only a public one signs in",
        config::What::NoProvider => b"no provider offers this model",
        config::What::Digest => b"invalid digest",
        config::What::Reserve { .. } => b"reserve does not fit charter ",
        config::What::Budget { .. } => b"budget does not fit charter ",
        config::What::Grace { .. } => b"grace does not fit template ",
        config::What::Operations { .. } => b"operations do not fit compound requests: operations ",
        config::What::Message => b"message bounds do not agree",
        config::What::Silence => b"heartbeat does not fit silence bound",
        config::What::StopBound => b"stop bound does not fit grace",
        config::What::Override => b"invalid override",
        config::What::Unreadable => b"not read",
        config::What::Relative => b"relative path must be non-empty with no .. component",
    }
}

pub fn render_path(path: &config::Path, writer: &mut Writer, length: &mut usize, measuring: bool) {
    let mut member = false;
    for segment in &path.segments {
        match segment {
            config::Segment::Member(name) => {
                if member {
                    put(writer, length, measuring, b".");
                }
                put(writer, length, measuring, name);
                member = true;
            }
            config::Segment::Index(index) => {
                put(writer, length, measuring, b"[");
                put(writer, length, measuring, Decimal::of(u64::from(*index)).as_bytes());
                put(writer, length, measuring, b"]");
            }
        }
    }
}
fn codec_words(what: config::What, writer: &mut Writer, length: &mut usize, measuring: bool) {
    let problem = config::Problem { path: config::Path::root(), what };
    let line = problem.codec_message().expect("codec-envelope diagnostic variant");
    let words = line.strip_prefix(b": ").expect("root codec diagnostic prefix");
    put(writer, length, measuring, words);
}

fn operation_counts(
    operations: u32,
    delegates: u32,
    goals: u32,
    writer: &mut Writer,
    length: &mut usize,
    measuring: bool,
) {
    put(writer, length, measuring, Decimal::of(u64::from(operations)).as_bytes());
    put(writer, length, measuring, b", delegates ");
    put(writer, length, measuring, Decimal::of(u64::from(delegates)).as_bytes());
    put(writer, length, measuring, b", goals ");
    put(writer, length, measuring, Decimal::of(u64::from(goals)).as_bytes());
}

/// A capacity's first line is not an override path: all its carriers follow it.
fn checkout<E>(what: &What<E>, writer: &mut Writer, length: &mut usize, measuring: bool) -> bool {
    match what {
        What::Checkout {
            conflicts,
            path_bytes,
            repositories,
            stdout,
            stdout_of,
            agent_conflicts,
            agent_paths,
            saved,
            engine,
            worker,
        } => {
            for (name, value) in [
                (b"checkout.conflicts ".as_slice(), conflicts),
                (b", checkout.path_bytes ", path_bytes),
                (b", checkout.repositories ", repositories),
            ] {
                put(writer, length, measuring, name);
                put(writer, length, measuring, Decimal::of(u64::from(*value)).as_bytes());
            }
            put(writer, length, measuring, b": raise these with them\n");
            for (name, of, unit, short) in [
                (b"git_component.stdout_bytes".as_slice(), *stdout_of, b" bytes".as_slice(), stdout),
                (b"workers[0].agent.conflicts", b"checkout.conflicts", b" paths", agent_conflicts),
                (b"workers[0].agent.path_bytes", b"checkout.path_bytes", b" bytes", agent_paths),
                (b"jig's Work.saved", b"Saved's encoded bound", b" bytes", saved),
                (b"engine.limits.io.output", b"the largest answer a run's saved work makes", b" bytes", engine),
                (b"workers[0].limits.io.output", b"the largest answer a run's saved work makes", b" bytes", worker),
            ] {
                if let Some((has, needs)) = short {
                    put(writer, length, measuring, name);
                    put(writer, length, measuring, b": ");
                    put(writer, length, measuring, Decimal::of(u64::from(*has)).as_bytes());
                    put(writer, length, measuring, unit);
                    put(writer, length, measuring, b", short of ");
                    put(writer, length, measuring, of);
                    put(writer, length, measuring, b", ");
                    put(writer, length, measuring, Decimal::of(u64::from(*needs)).as_bytes());
                    put(writer, length, measuring, unit);
                    put(writer, length, measuring, b"\n");
                }
            }
            true
        }
        What::Jig(_)
        | What::Standalone
        | What::WorkerAlone
        | What::Absolute
        | What::Profile
        | What::Override(_)
        | What::Against { .. }
        | What::Past { .. }
        | What::Short { .. }
        | What::Within { .. }
        | What::CallInput { .. }
        | What::Half(_) => false,
    }
}
/// Render a shared variant after the half has dispatched its own `Half` variant.
#[expect(clippy::too_many_lines, reason = "one exhaustive shared vocabulary for both rendering passes")]
pub fn render_problem<E>(problem: &Problem<What<E>>, writer: &mut Writer, measuring: bool) -> usize {
    let mut length = 0;
    if checkout(&problem.what, writer, &mut length, measuring) {
        return length;
    }
    render_path(&problem.path, writer, &mut length, measuring);
    put(writer, &mut length, measuring, b": ");
    if let Some(file) = &problem.file {
        put(writer, &mut length, measuring, file);
        put(writer, &mut length, measuring, b": ");
    }
    put(writer, &mut length, measuring, words(&problem.what));
    match &problem.what {
        What::Half(_) => unreachable!("half rendered by its service"),
        What::Checkout { .. } => unreachable!("checkout capacity rendered whole"),
        What::Jig(
            what @ (config::What::PolicyEnvelope { .. }
            | config::What::AmendmentEnvelope { .. }
            | config::What::CodecCeiling { .. }
            | config::What::Capacity { .. }
            | config::What::CodecArithmetic
            | config::What::StoreCodec),
        ) => codec_words(*what, writer, &mut length, measuring),
        What::CallInput { bytes, input, ask, reply } => {
            put(writer, &mut length, measuring, Decimal::of(*bytes).as_bytes());
            put(
                writer,
                &mut length,
                measuring,
                b" bytes, short of the agent's call input (profile.declared.tool_payload), ",
            );
            put(writer, &mut length, measuring, Decimal::of(*input).as_bytes());
            put(writer, &mut length, measuring, b" bytes; raise it to at least ");
            put(writer, &mut length, measuring, Decimal::of(*input).as_bytes());
            put(writer, &mut length, measuring, b" bytes, or omit it to follow the agent's call input\n");
            render_path(&problem.path, writer, &mut length, measuring);
            put(writer, &mut length, measuring, b": call input ");
            put(writer, &mut length, measuring, Decimal::of(*ask).as_bytes());
            put(writer, &mut length, measuring, b" bytes; wire answer ");
            put(writer, &mut length, measuring, Decimal::of(*bytes).as_bytes());
            put(writer, &mut length, measuring, b" bytes; smith host_reply_bytes ");
            put(writer, &mut length, measuring, Decimal::of(*reply).as_bytes());
            put(writer, &mut length, measuring, b" bytes");
        }
        What::Against { other } => put(writer, &mut length, measuring, other),
        What::Past { bytes, bound } => {
            put(writer, &mut length, measuring, Decimal::of(*bytes).as_bytes());
            put(writer, &mut length, measuring, b" bytes, past its bound of ");
            put(writer, &mut length, measuring, Decimal::of(*bound).as_bytes());
        }
        What::Short { bytes, of, needs } => {
            put(writer, &mut length, measuring, Decimal::of(*bytes).as_bytes());
            put(writer, &mut length, measuring, b" bytes, short of ");
            put(writer, &mut length, measuring, of);
            put(writer, &mut length, measuring, b", ");
            put(writer, &mut length, measuring, Decimal::of(*needs).as_bytes());
            put(writer, &mut length, measuring, b" bytes");
        }
        What::Within { bytes, limit, bound } => {
            put(writer, &mut length, measuring, Decimal::of(*bytes).as_bytes());
            put(writer, &mut length, measuring, b" bytes, past ");
            put(writer, &mut length, measuring, limit);
            put(writer, &mut length, measuring, b" of ");
            put(writer, &mut length, measuring, Decimal::of(*bound).as_bytes());
        }
        What::Jig(config::What::Operations { operations, delegates, goals }) => {
            operation_counts(*operations, *delegates, *goals, writer, &mut length, measuring);
        }
        What::Jig(
            config::What::Reserve { charter }
            | config::What::Budget { charter }
            | config::What::Grace { template: charter },
        ) => {
            put(writer, &mut length, measuring, Decimal::of(u64::from(*charter)).as_bytes());
        }
        What::Jig(
            config::What::Because(_)
            | config::What::Json
            | config::What::Stored
            | config::What::Unknown
            | config::What::Missing
            | config::What::Duplicate
            | config::What::Kind
            | config::What::Bound
            | config::What::SlotsPastHub
            | config::What::Form
            | config::What::Unique
            | config::What::Refers
            | config::What::Plaintext
            | config::What::Confidential
            | config::What::NoProvider
            | config::What::ProviderKey { .. }
            | config::What::Digest
            | config::What::Message
            | config::What::Silence
            | config::What::StopBound
            | config::What::Override
            | config::What::Unreadable
            | config::What::Relative,
        )
        | What::WorkerAlone
        | What::Standalone
        | What::Absolute
        | What::Profile
        | What::Override(_) => {}
    }
    put(writer, &mut length, measuring, b"\n");
    length
}
pub fn render_past(past: u32, writer: &mut Writer, length: &mut usize, measuring: bool) {
    if past > 0 {
        put(writer, length, measuring, b"and ");
        put(writer, length, measuring, Decimal::of(u64::from(past)).as_bytes());
        put(writer, length, measuring, b" more\n");
    }
}
fn render(problems: &SharedProblems, writer: &mut Writer, measuring: bool) -> usize {
    let mut length: usize = 0;
    for problem in &problems.problems {
        length = length.checked_add(render_problem(problem, writer, measuring)).expect("bounded diagnostics");
    }
    render_past(problems.past, writer, &mut length, measuring);
    length
}
#[must_use]
pub fn lines(problems: &SharedProblems) -> Box<[u8]> {
    let length = render(problems, &mut Writer::new(0), true);
    let mut writer = Writer::new(length);
    assert_eq!(render(problems, &mut writer, false), length, "diagnostic passes measure identically");
    writer.finish()
}
/// A half with no additional startup problems (protocol/services.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SharedOnly {}
pub type SharedWhat = What<SharedOnly>;
pub type SharedProblems = Problems<SharedWhat>;
impl SharedWhat {
    #[must_use]
    pub fn extend<E>(self) -> What<E> {
        match self {
            What::Jig(what) => What::Jig(what),
            What::Standalone => What::Standalone,
            What::WorkerAlone => What::WorkerAlone,
            What::Absolute => What::Absolute,
            What::Profile => What::Profile,
            What::Override(what) => What::Override(what),
            What::Against { other } => What::Against { other },
            What::Checkout {
                conflicts,
                path_bytes,
                repositories,
                stdout,
                stdout_of,
                agent_conflicts,
                agent_paths,
                saved,
                engine,
                worker,
            } => What::Checkout {
                conflicts,
                path_bytes,
                repositories,
                stdout,
                stdout_of,
                agent_conflicts,
                agent_paths,
                saved,
                engine,
                worker,
            },
            What::Past { bytes, bound } => What::Past { bytes, bound },
            What::Short { bytes, of, needs } => What::Short { bytes, of, needs },
            What::Within { bytes, limit, bound } => What::Within { bytes, limit, bound },
            What::CallInput { bytes, input, ask, reply } => What::CallInput { bytes, input, ask, reply },
            What::Half(half) => match half {},
        }
    }
}
