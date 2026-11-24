//! A worker's startup values and its bounded end (configuration.md, 4; protocol/channel/worker-link.md, 1.1).
use crate::{Address, Directories, Directory, Limits, Path, Problems, What, check, overrides, read, stock};
use alloc::boxed::Box;
use skein_json::Document;
use skein_lib::{Duration, List};

#[derive(Clone, Debug)]
pub struct Worker {
    pub engine: Address,
    pub name: Box<[u8]>,
    /// The private file holding the worker's secret, never the secret itself.
    pub secret: Box<[u8]>,
    pub slots: u32,
    pub graces: Graces,
    pub stop_bound: Duration,
    pub agent: Option<Agent>,
    /// Positive interval between local source-login observations.
    pub check_interval: Duration,
    pub limits: Option<Limits>,
    pub memory: Option<u64>,
}
/// Members an application fixes (protocol/configuration-templates.md, 4).
#[derive(Clone, Copy, Debug)]
pub struct FixedWorker {
    pub graces: Option<Graces>,
    pub check_interval: Option<Duration>,
}
impl FixedWorker {
    pub const NONE: Self = Self { graces: None, check_interval: None };
}
const DEFAULT_DRAIN: Duration = Duration::from_millis(180_000);
/// Calls in flight at a drained answer get ten seconds unless configured otherwise.
pub const DEFAULT_DRAIN_GRACE: Duration = Duration::from_millis(10_000);

/// Worker stop deadlines (jig/docs/design/domain/hosts/hub.md, 6 and 7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Graces {
    pub grace: Duration,
    /// How long a worker told to stop lets runs end their turns before cancelling them.
    pub drain: Duration,
    /// Time for calls in flight at a parts-telling run's drained answer.
    pub drain_grace: Duration,
    pub agent_stop: Duration,
    pub delivery: Duration,
    pub save: Duration,
}
/// The program and the file smith's startup parses; jig does not read that document.
#[derive(Clone, Debug)]
pub struct Agent {
    pub program: Box<[u8]>,
    pub arguments: Box<[Box<[u8]>]>,
    pub configuration: Box<[u8]>,
}
/// The bound stated at hello, measured from the engine's last possible frame.
#[must_use]
pub fn stop_bound(graces: &Graces, ping: Duration, silence: Duration) -> Option<Duration> {
    ping.checked_add(silence)?
        .checked_add(graces.grace)?
        .max(graces.drain)
        .checked_add(graces.agent_stop.max(graces.delivery))?
        .checked_add(graces.save)
}
/// Parse jig's worker member before startup constructs any component.
pub fn worker(
    document: &Document,
    at: u32,
    directories: &Directories,
    fixed: &FixedWorker,
) -> Result<Worker, Problems> {
    worker_over(document, at, directories, fixed, &stock())
}

/// Parse jig's worker member over an application's base (protocol/limits.md, the stock).
pub fn worker_over(
    document: &Document,
    at: u32,
    directories: &Directories,
    fixed: &FixedWorker,
    base: &Limits,
) -> Result<Worker, Problems> {
    let path = Path::root();
    let mut problems = Problems::empty();
    let Some(mut object) = check::object(document, at, &path, &mut problems, "worker") else {
        return Err(problems);
    };
    let mut limits = *base;
    let mut hub_slots_overridden = false;
    let has_limits = object.member(document, b"limits");
    if let Some(at) = has_limits {
        let (parsed, overridden) = overrides::read_over(document, at, &path.member(b"limits"), base);
        hub_slots_overridden = overridden.hub_slots;
        limits = parsed.limits;
        for problem in &parsed.problems.problems {
            problems.push(problem.path.clone(), problem.what);
        }
        problems.past = problems.past.saturating_add(parsed.problems.past);
    }
    let engine = check::address(&mut object, document, b"engine", &path, &mut problems);
    if let Some(engine) = &engine {
        check::local(engine, &path.member(b"engine"), &mut problems);
    }
    let name = check::text(&mut object, document, b"name", &path, &mut problems);
    if let Some(name) = &name
        && name.len() > skein_lib::length::usize_of(limits.link.name)
    {
        problems.push(path.member(b"name"), What::Bound);
    }
    let secret = check::path(&mut object, document, b"secret", &path, Directory::Data, directories, &mut problems);
    let slots = check::count(&mut object, document, b"slots", &path, &mut problems);
    if let Some(slots) = slots {
        if !hub_slots_overridden {
            limits.hub.slots = slots;
        } else if slots > limits.hub.slots {
            problems.push(path.member(b"slots"), What::SlotsPastHub);
        }
    }
    let mut graces = fixed.graces;
    if let Some(at) =
        crate::fixed::member(&mut object, document, b"graces", fixed.graces.is_some(), &path, &mut problems)
    {
        graces = parse_graces(document, at, &path.member(b"graces"), &mut problems);
    }
    let mut check_interval = fixed.check_interval;
    if let Some(at) =
        crate::fixed::member(&mut object, document, b"programs", fixed.check_interval.is_some(), &path, &mut problems)
    {
        let path = path.member(b"programs");
        if let Some(mut programs) = check::object(document, at, &path, &mut problems, "programs") {
            check_interval = check::duration(&mut programs, document, b"check_interval_ms", &path, &mut problems);
            programs.finish(document, &path, &mut problems);
        }
    }
    if check_interval == Some(Duration::ZERO) {
        problems.push(path.member(b"programs").member(b"check_interval_ms"), What::Bound);
    }
    if let Some(interval) = check_interval {
        limits.hub.source_interval = interval;
    }
    let mut agent = None;
    if let Some(at) = object.member(document, b"agent") {
        agent = parse_agent(document, at, &path.member(b"agent"), directories, &mut problems);
    }
    let memory = check::optional_integer(&mut object, document, b"memory_bytes", &path, &mut problems);
    object.finish(document, &path, &mut problems);
    let bound = check::worker(&limits, graces, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(Worker {
        engine: engine.expect("required member checked"),
        name: name.expect("required member checked"),
        secret: secret.expect("required member checked"),
        slots: slots.expect("required member checked"),
        graces: graces.expect("required member checked"),
        stop_bound: bound.expect("duration sum checked"),
        agent,
        check_interval: check_interval.expect("required member checked"),
        limits: has_limits.and(Some(limits)),
        memory,
    })
}
fn parse_graces(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Graces> {
    let mut object = check::object(document, at, path, problems, "graces")?;
    let grace = check::duration(&mut object, document, b"grace_ms", path, problems);
    let mut drain = Some(DEFAULT_DRAIN);
    if let Some(at) = object.member(document, b"drain_ms") {
        drain = match read::duration(document, at) {
            Ok(value) => Some(value),
            Err(what) => {
                problems.push(path.member(b"drain_ms"), what);
                None
            }
        };
    }
    let mut drain_grace = Some(DEFAULT_DRAIN_GRACE);
    if let Some(at) = object.member(document, b"drain_grace_ms") {
        drain_grace = match read::duration(document, at) {
            Ok(value) => Some(value),
            Err(what) => {
                problems.push(path.member(b"drain_grace_ms"), what);
                None
            }
        };
    }
    let agent_stop = check::duration(&mut object, document, b"agent_stop_ms", path, problems);
    let delivery = check::duration(&mut object, document, b"delivery_ms", path, problems);
    let save = check::duration(&mut object, document, b"save_ms", path, problems);
    object.finish(document, path, problems);
    Some(Graces {
        grace: grace?,
        drain: drain?,
        drain_grace: drain_grace?,
        agent_stop: agent_stop?,
        delivery: delivery?,
        save: save?,
    })
}
fn parse_agent(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Agent> {
    let mut object = check::object(document, at, path, problems, "agent")?;
    let program = check::text(&mut object, document, b"program", path, problems);
    let configuration =
        check::path(&mut object, document, b"configuration", path, Directory::Config, directories, problems);
    let mut arguments = List::with_capacity(document.len());
    if let Some(items) = check::list(&mut object, document, b"arguments", document.len(), path, problems) {
        for (index, at) in items.iter().enumerate() {
            match read::text(document, *at, 4_096) {
                Ok(value) => arguments.push(value).expect("arguments bounded by document tokens"),
                Err(what) => problems.push(check::index(&path.member(b"arguments"), index), what),
            }
        }
    }
    object.finish(document, path, problems);
    Some(Agent { program: program?, arguments: arguments.into_boxed(), configuration: configuration? })
}
