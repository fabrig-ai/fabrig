//! One resolution for startup and settings admission (configuration/limit-settings.md, 3–4).
use super::{Problems, What, checked_start_limits, checks};
use crate::{limits, settings, stock};
use alloc::boxed::Box;
use jig_domain_core::settings::Override;
use jig_protocol_config::{self as config, Path, Segment};
use skein_lib::{List, Writer, bytes};
use temper_service_core::derived::{self, Sources};

/// Immutable startup facts, not either limit setting. The optional typed base
/// is the in-process world's existing seam; production leaves it absent.
#[derive(Debug)]
pub struct LimitInputs {
    pub document: Box<[u8]>,
    pub directories: config::Directories,
    pub files: settings::Files,
    pub agent: Option<settings::Agent>,
    pub seeds: settings::Seeds,
    pub given: stock::Given,
    pub sources: Sources,
    pub is_box: bool,
    pub base: Option<(config::Limits, limits::Limits)>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LimitKind {
    Jig,
    Temper,
}
impl LimitKind {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Jig => b"limits",
            Self::Temper => b"temper-limits",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LeftOut {
    pub kind: LimitKind,
    pub name: Box<[u8]>,
}
#[derive(Clone, Debug)]
pub struct Resolved {
    pub jig: config::Limits,
    pub temper: limits::Limits,
    pub root: temper_engine_domain::Limits,
    pub left_out: List<LeftOut>,
    pub store: jig_protocol_store::Limits,
    pub worker: limits::Limits,
}

impl LimitInputs {
    /// The engine's and the box's shells supply exactly the facts they checked
    /// and read before opening the store.
    #[must_use]
    #[expect(clippy::too_many_arguments, reason = "the pure resolution retains the start's immutable inputs")]
    pub fn new(
        document: &[u8],
        directories: &config::Directories,
        files: &settings::Files,
        agent: Option<&settings::Agent>,
        seeds: &settings::Seeds,
        given: &stock::Given,
        is_box: bool,
        base: Option<(&config::Limits, &limits::Limits)>,
    ) -> Self {
        let sources = match agent {
            Some(agent) => Sources::agent(agent),
            None => derived::STOCK,
        };
        Self {
            document: bytes::copy_of(document),
            directories: directories.clone(),
            files: settings::Files { entries: files.entries.clone() },
            agent: agent.cloned(),
            seeds: seeds.clone(),
            given: given.copied(),
            sources,
            is_box,
            base: match base {
                Some((jig, own)) => Some((*jig, *own)),
                None => None,
            },
        }
    }
    /// Opening/replay figures come from the stock alone, never either override
    /// list or a changed agent declaration. A typed world supplies its own stock.
    pub fn stock(&self) -> Result<(config::Limits, limits::Limits), Problems> {
        match self.base {
            Some(base) => Ok(base),
            None => match limits::base(derived::LIVE_GOALS, derived::STOCK.payload, derived::STOCK) {
                Ok(base) => Ok(base),
                Err(found) => Err(Problems::jig(single(found), &Path::root())),
            },
        }
    }
}
fn single(found: config::Problem) -> config::Problems {
    let mut problems = config::Problems::empty();
    problems.push(found.path, found.what);
    problems
}
fn supplied(overrides: &[Override], name: &[u8]) -> Option<u64> {
    for item in overrides {
        if item.name.as_ref() == name {
            return Some(item.value);
        }
    }
    None
}
fn known(overrides: &[Override], kind: LimitKind, left_out: &mut List<LeftOut>) -> Box<[Override]> {
    let mut kept = List::with_capacity(u32::try_from(overrides.len()).expect("checked override count"));
    for item in overrides {
        let found = match kind {
            LimitKind::Jig => config::names::find(&item.name).is_some(),
            LimitKind::Temper => limits::find(&item.name).is_some(),
        };
        if found {
            kept.push(item.clone()).expect("input count");
        } else {
            left_out.push(LeftOut { kind, name: item.name.clone() }).expect("input counts");
        }
    }
    kept.into_boxed()
}
fn worker(name: &[u8]) -> bool {
    temper_service_core::limits::shared_name(name)
}

/// Resolve in the design's order and check the entire result against the
/// start's facts and the store's stock envelope. Unknown names are retained as
/// observations, not applied. No engine or IO is constructed here.
pub fn resolve_limits(inputs: &LimitInputs, jig: &[Override], temper: &[Override]) -> Result<Resolved, Problems> {
    let capacity = u32::try_from(jig.len().saturating_add(temper.len())).expect("checked override counts");
    let mut left_out = List::with_capacity(capacity);
    let jig = known(jig, LimitKind::Jig, &mut left_out);
    let temper = known(temper, LimitKind::Temper, &mut left_out);
    let result = resolve(inputs, &jig, &temper, left_out);
    match result {
        Ok(resolved) => Ok(resolved),
        Err(problems) => Err(named(problems, &jig, &temper)),
    }
}
#[expect(clippy::too_many_lines, reason = "the six resolution steps retain one checked ordering")]
fn resolve(
    inputs: &LimitInputs,
    jig: &[Override],
    temper: &[Override],
    left_out: List<LeftOut>,
) -> Result<Resolved, Problems> {
    let (stock_jig, stock_own) = inputs.stock()?;
    let stock_root = super::engine_limits(&mut stock_jig.clone(), &stock_own, None)?;
    let store = super::checked_store(&stock_root, &stock_jig.store, None)?;
    let mut sources = stock_own;
    for item in temper {
        if inputs.is_box && worker(&item.name) {
            return Err(super::problem(
                Path::root().member(&item.name),
                What::Jig(config::What::Because(b"this is a worker's limit; it is not taken from temper-limits")),
            ));
        }
        if (item.name.as_ref() == b"live_goals" || item.name.as_ref() == b"tool_payload")
            && let Err(why) = limits::apply(&mut sources, &item.name, item.value)
        {
            return Err(super::problem(Path::root().member(&item.name), What::Override(why)));
        }
    }
    let source = match &inputs.agent {
        Some(_) => inputs.sources,
        None => Sources { payload: sources.tool_payload, ..inputs.sources },
    };
    let (mut base, mut own) = if inputs.base.is_some()
        && sources.live_goals == stock_own.live_goals
        && sources.tool_payload == stock_own.tool_payload
    {
        (stock_jig, stock_own)
    } else {
        match limits::base(sources.live_goals, sources.tool_payload, source) {
            Ok(base) => base,
            Err(found) => return Err(Problems::jig(single(found), &Path::root())),
        }
    };
    let minimum_own = own;
    if inputs.base.is_none() {
        base.core.fleet.grace = super::worker_fleet_grace(&base);
    }
    if !jig.is_empty() {
        derived::follow_task_overrides(&mut base, jig);
    }
    let minimum_jig = base;
    let mut jig = match config::with_overrides(&base, jig) {
        Ok(jig) => jig,
        Err(found) => return Err(Problems::jig(found, &Path::root())),
    };
    for item in temper {
        if let Err(why) = limits::apply(&mut own, &item.name, item.value) {
            return Err(super::problem(Path::root().member(&item.name), What::Override(why)));
        }
    }
    if inputs.is_box {
        let document = match config::document(
            &inputs.document,
            &config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 },
        ) {
            Ok(document) => document,
            Err(found) => return Err(Problems::jig(single(found), &Path::root())),
        };
        if let Some(slots) = derived::slots(&document) {
            own.checkout.workspaces = slots;
            own.git_component.operations = slots;
        }
    }
    let followers = inputs.base.is_none() || !temper.is_empty();
    if followers && supplied(temper, b"forge_component.body_bytes").is_none() {
        own.forge_component.body_bytes = derived::forge_body(own.forge.client.op_bytes).ok_or(super::problem(
            Path::root().member(b"forge_component.body_bytes"),
            What::Jig(config::What::Bound),
        ))?;
    }
    if followers && supplied(temper, b"forge_component.repositories").is_none() {
        own.forge_component.repositories = own.forge.repositories;
    }
    if followers && supplied(temper, b"forge.landing_members").is_none() {
        own.forge.landing_members = own.forge.landing_members.min(own.forge.holds);
    }
    if followers && supplied(temper, b"forge.landings").is_none() {
        own.forge.landings =
            own.forge.landings.min(own.forge.holds.checked_div(own.forge.landing_members).unwrap_or(0));
    }
    let mut floors = Problems::empty();
    derived_floors(&minimum_jig, &minimum_own, &jig, &own, &mut floors);
    if !floors.is_empty() {
        return Err(floors);
    }
    let mut root = super::engine_limits(&mut jig, &own, None)?;
    let application = temper_protocol_store::application(&root.forge, root.tasks.spec_bytes)
        .ok_or(checks::store_refusal(crate::store::Refusal::Map(jig_protocol_store::sizes::SizeRefusal::Ceiling)))?;
    let commits = jig_protocol_store::sizes::Commits { in_flight: root.journal.commits, writes: root.journal.writes };
    if let Err(why) = jig_protocol_store::sizes::check_commits(
        &temper_engine_domain::core_limits(&root),
        &application,
        &commits,
        &store.store,
        store.orphan_erases,
    ) {
        return Err(checks::store_refusal(crate::store::Refusal::Map(why)));
    }
    let start = checked_start_limits(
        &inputs.document,
        &inputs.directories,
        &inputs.files,
        inputs.agent.as_ref(),
        &inputs.seeds,
        List::with_capacity(0),
        &inputs.given,
        Some((&jig, &own)),
        inputs.is_box,
    )?;
    jig = start.engine.jig.limits;
    root = super::engine_limits(&mut jig, &own, None)?;
    root.host_reply_bytes = start.engine.host_reply_bytes;
    let application = temper_protocol_store::application(&root.forge, root.tasks.spec_bytes)
        .ok_or(checks::store_refusal(crate::store::Refusal::Map(jig_protocol_store::sizes::SizeRefusal::Ceiling)))?;
    if let Err(why) = jig_protocol_store::runtime_limits(
        &store,
        &temper_engine_domain::core_limits(&root),
        application,
        root.journal.transcript_bytes.max(root.loads.transcript_bytes),
    ) {
        return Err(checks::store_refusal(crate::store::Refusal::Map(why)));
    }
    let mut worker = match inputs.base {
        Some(_) => stock_own,
        None => match limits::base(derived::LIVE_GOALS, derived::STOCK.payload, inputs.sources) {
            Ok((_, worker)) => worker,
            Err(found) => return Err(Problems::jig(single(found), &Path::root())),
        },
    };
    if inputs.is_box {
        worker.checkout.workspaces = own.checkout.workspaces;
        worker.git_component.operations = own.git_component.operations;
    }
    Ok(Resolved { jig, temper: own, root, left_out, store, worker })
}

fn derived_floors(
    base: &config::Limits,
    own_base: &limits::Limits,
    jig: &config::Limits,
    own: &limits::Limits,
    problems: &mut Problems,
) {
    for name in [
        b"tasks.message_bytes".as_slice(),
        b"tasks.result_bytes",
        b"tasks.inbox_bytes",
        b"hub.turn_bytes",
        b"core.resume_bytes",
        b"hub.transcript_bytes",
        b"hub.turn_queue_bytes",
        b"io.output",
        b"hub.outcome_bytes",
        b"core.call_answer_bytes",
    ] {
        let row = config::names::find(name).expect("derived jig name");
        let (has, needs) = ((row.value)(jig), (row.value)(base));
        if has < needs {
            problems.push(Path::root().member(name), What::Short { bytes: has, of: b"the source's derivation", needs });
        }
    }
    for name in [
        b"root.journal.transcript_bytes".as_slice(),
        b"root.loads.transcript_bytes",
        b"root.loads.bytes",
        b"root.loads.reply_bytes",
        b"root.loads.read_bytes",
        b"checkout.message_bytes",
        b"forge.client.op_bytes",
        b"forge.client.answer_bytes",
    ] {
        let row = limits::find(name).expect("derived temper name");
        let (has, needs) = ((row.value)(own), (row.value)(own_base));
        if has < needs {
            problems.push(Path::root().member(name), What::Short { bytes: has, of: b"the source's derivation", needs });
        }
    }
}

fn dotted(path: &Path) -> Box<[u8]> {
    let mut pieces = List::with_capacity(path.segments.len().saturating_mul(2));
    for segment in &path.segments {
        match segment {
            Segment::Member(name) => {
                if name.as_ref() == b"engine"
                    || name.as_ref() == b"application"
                    || name.as_ref() == b"limits"
                    || name.as_ref() == b"overrides"
                {
                    continue;
                }
                if !pieces.is_empty() {
                    pieces.push(b".".as_slice()).expect("path count");
                }
                pieces.push(name.as_ref()).expect("path count");
            }
            Segment::Index(_) => {}
        }
    }
    bytes::concat(pieces.as_slice())
}
fn named(found: Problems, jig: &[Override], temper: &[Override]) -> Problems {
    let mut problems = Problems::empty();
    for problem in &found.problems {
        let mut name = dotted(&problem.path);
        if name.is_empty() && jig.is_empty() && temper.is_empty() {
            problems.push_file(problem.path.clone(), problem.what, problem.file.clone());
            continue;
        }
        if !name.is_empty()
            && name.as_ref() != b"store"
            && config::names::find(&name).is_none()
            && limits::find(&name).is_none()
        {
            problems.push_file(problem.path.clone(), problem.what, problem.file.clone());
            continue;
        }
        if supplied(jig, b"tasks.tasks").is_some()
            && supplied(jig, &name).is_none()
            && [b"tasks.delegates".as_slice(), b"tasks.exemptions", b"people.operations", b"people.inbox_tasks"]
                .contains(&name.as_ref())
        {
            name = Box::from(b"tasks.tasks".as_slice());
        }
        if name.is_empty() {
            name = if supplied(temper, b"tool_payload").is_some() {
                Box::from(b"tool_payload".as_slice())
            } else if supplied(temper, b"forge.entries").is_some() {
                Box::from(b"forge.entries".as_slice())
            } else if let Some(item) = jig.first() {
                item.name.clone()
            } else if let Some(item) = temper.first() {
                item.name.clone()
            } else {
                Box::from(b"forge.entries".as_slice())
            };
        }
        if name.as_ref() == b"store" {
            name = if supplied(temper, b"live_goals").unwrap_or(0) > u64::from(derived::LIVE_GOALS) {
                Box::from(b"live_goals".as_slice())
            } else if supplied(temper, b"forge.entries").is_some() {
                Box::from(b"forge.entries".as_slice())
            } else if let Some(item) = temper.first() {
                item.name.clone()
            } else if let Some(item) = jig.first() {
                item.name.clone()
            } else {
                Box::from(b"forge.entries".as_slice())
            };
        }
        let source: &[u8] = if supplied(jig, &name).is_some() {
            b" (setting limits)".as_slice()
        } else if supplied(temper, &name).is_some() {
            b" (setting temper-limits)"
        } else {
            b" (the stock)"
        };
        let mut label = Writer::new(name.len().saturating_add(source.len()));
        label.put(&name).expect("measured name");
        label.put(source).expect("measured source");
        problems.push_file(Path::root().member(&label.finish()), problem.what, problem.file.clone());
    }
    problems.past = found.past;
    problems
}
