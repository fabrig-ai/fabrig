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
    /// The carrier payload fixed at this engine start.
    pub payload: u32,
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
        let sources = limit_sources(agent);
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
    /// The deployment's opening/replay base, never either override list.
    pub fn store_base(&self) -> Result<(config::Limits, limits::Limits), Problems> {
        let typed = match &self.base {
            Some((jig, own)) => Some((jig, own)),
            None => None,
        };
        match limits::store_base(typed, self.sources) {
            Ok(base) => Ok(base),
            Err(found) => Err(Problems::jig(single(found), &Path::root())),
        }
    }
    /// Runtime defaults remain at the stock's live goals. A typed world
    /// supplies its own stock.
    pub fn stock(&self) -> Result<(config::Limits, limits::Limits), Problems> {
        match self.base {
            Some(base) => Ok(base),
            None => stock(),
        }
    }
}
fn limit_sources(agent: Option<&settings::Agent>) -> Sources {
    match agent {
        Some(agent) => Sources::agent(agent),
        None => derived::STOCK,
    }
}
fn stock() -> Result<(config::Limits, limits::Limits), Problems> {
    base(derived::LIVE_GOALS, derived::STOCK.payload, derived::STOCK)
}
fn base(goals: u32, payload: u32, source: Sources) -> Result<(config::Limits, limits::Limits), Problems> {
    match limits::base(goals, payload, source) {
        Ok(base) => Ok(base),
        Err(found) => Err(Problems::jig(single(found), &Path::root())),
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
/// Resolve in the design's order and check the entire result against the
/// start's facts and the store's stock envelope. Unknown names are retained as
/// observations, not applied. No engine or IO is constructed here.
pub fn resolve_limits(inputs: &LimitInputs, jig: &[Override], temper: &[Override]) -> Result<Resolved, Problems> {
    checked_limits(inputs, jig, temper, true, None)
}

/// A carrier that falls short of the derivation, with the figure it needs.
#[derive(Clone, Copy, Debug)]
struct CarrierFloor {
    name: &'static [u8],
    needs: u64,
    has: u64,
    of: &'static [u8],
}

type CarrierMinimum = (LimitKind, &'static [u8], u64);

/// Carrier shortfalls for an untyped deployment. Report them in dependency
/// order, using the raised figure for any carrier downstream of a shortfall.
/// This is not admission: use `resolve_limits` to check the whole set.
pub fn carrier_floors(
    agent: Option<&settings::Agent>,
    jig: &[Override],
    temper: &[Override],
) -> Result<List<CarrierMinimum>, Problems> {
    let source = limit_sources(agent);
    let (base, own_base, _) = derived_base(&stock()?, source, false, jig, temper)?;
    let jig_floors = jig_floors(&base, jig, true);
    let resume = effective(&jig_floors, b"core.resume_bytes", jig_figure(&base, jig, b"core.resume_bytes"));
    let result = effective(&jig_floors, b"tasks.result_bytes", jig_figure(&base, jig, b"tasks.result_bytes"));
    let own = own_overrides(own_base, temper)?;
    let mut own_floors = derived_floors(&own_base, &own, resume);
    let op = effective(&own_floors, b"forge.client.op_bytes", u64::from(own.forge.client.op_bytes));
    // An absent body override already follows the operation figure.
    let op = forge_op_floor(&mut own_floors, op, result);
    if let Some(body) = supplied(temper, b"forge_component.body_bytes") {
        forge_body_floor(&mut own_floors, body, op);
    }
    let mut floors = List::uncapped();
    for (kind, shortfalls) in [(LimitKind::Jig, jig_floors), (LimitKind::Temper, own_floors)] {
        for floor in &shortfalls {
            floors.push((kind, floor.name, floor.needs)).expect("one row per carrier shortfall");
        }
    }
    Ok(floors)
}

/// The immutable stock and store envelope, prepared once for related mutations.
pub(crate) struct Preflight {
    stock: (config::Limits, limits::Limits),
    store: jig_protocol_store::Limits,
}

impl Preflight {
    pub(crate) fn new(inputs: &LimitInputs) -> Result<Self, Problems> {
        let stock = inputs.stock()?;
        let (mut store_jig, store_own) = inputs.store_base()?;
        let stock_root = super::engine_limits(&mut store_jig, &store_own, None)?;
        let store = super::checked_store(&stock_root, &store_jig.store, None)?;
        Ok(Self { stock, store: *store.limits() })
    }

    /// Use the same preflight as startup; composed relations retain the full start.
    #[cfg(test)]
    pub(crate) fn check(
        &self,
        inputs: &LimitInputs,
        jig: &[Override],
        temper: &[Override],
        full_start: bool,
    ) -> Result<Resolved, Problems> {
        checked_limits(inputs, jig, temper, full_start, Some(self))
    }
}

fn checked_limits(
    inputs: &LimitInputs,
    jig: &[Override],
    temper: &[Override],
    full_start: bool,
    prepared: Option<&Preflight>,
) -> Result<Resolved, Problems> {
    let capacity = u32::try_from(jig.len().saturating_add(temper.len())).expect("checked override counts");
    let mut left_out = List::with_capacity(capacity);
    let jig = known(jig, LimitKind::Jig, &mut left_out);
    let temper = known(temper, LimitKind::Temper, &mut left_out);
    let result = match prepared {
        Some(prepared) => preflight(inputs, &jig, &temper, left_out, prepared),
        None => match Preflight::new(inputs) {
            Ok(prepared) => preflight(inputs, &jig, &temper, left_out, &prepared),
            Err(problems) => Err(problems),
        },
    };
    let result = match result {
        Ok(resolved) if full_start => complete(inputs, resolved),
        result => result,
    };
    match result {
        Ok(resolved) => Ok(resolved),
        Err(problems) => Err(named(problems, &jig, &temper)),
    }
}
fn preflight(
    inputs: &LimitInputs,
    jig: &[Override],
    temper: &[Override],
    left_out: List<LeftOut>,
    prepared: &Preflight,
) -> Result<Resolved, Problems> {
    let (base, own_base, payload) = derived_base(&prepared.stock, inputs.sources, inputs.base.is_some(), jig, temper)?;
    let mut early = Problems::empty();
    report_floors(&jig_floors(&base, jig, false), &mut early);
    if !early.is_empty() {
        return Err(early);
    }
    let mut jig = match config::with_overrides(&base, jig) {
        Ok(jig) => jig,
        Err(found) => return Err(Problems::jig(found, &Path::root())),
    };
    let mut own = own_overrides(own_base, temper)?;
    let store = prepared.store;
    if inputs.is_box && inputs.base.is_none() {
        let document = match config::document(
            &inputs.document,
            &config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 },
        ) {
            Ok(document) => document,
            Err(found) => return Err(Problems::jig(single(found), &Path::root())),
        };
        if let Some(slots) = derived::slots(&document) {
            if supplied(temper, b"checkout.workspaces").is_none() {
                own.checkout.workspaces = slots;
            }
            if supplied(temper, b"git_component.operations").is_none() {
                own.git_component.operations = own.checkout.workspaces;
            }
        }
    }
    let followers = inputs.base.is_none() || !temper.is_empty();
    if followers && supplied(temper, b"forge_component.body_bytes").is_none() {
        own.forge_component.body_bytes = derived::forge_body(own.forge.client.op_bytes).ok_or(super::problem(
            Path::root().member(b"forge_component.body_bytes"),
            What::Jig(config::What::Bound),
        ))?;
    }
    if followers && supplied(temper, b"forge_component.forges").is_none() {
        own.forge_component.forges = own.forge.forges;
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
    let mut carriers = derived_floors(&own_base, &own, u64::from(jig.core.resume_bytes));
    let op = u64::from(own.forge.client.op_bytes);
    forge_op_floor(&mut carriers, op, u64::from(jig.core.tasks.result_bytes));
    forge_body_floor(&mut carriers, u64::from(own.forge_component.body_bytes), op);
    report_floors(&carriers, &mut floors);
    related_limits(&own, &mut floors);
    super::application::tier_bounds(&own, &mut floors);
    if !floors.is_empty() {
        return Err(floors);
    }
    let root = super::engine_limits(&mut jig, &own, None)?;
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
    jig.store = store;
    Ok(Resolved { jig, temper: own, root, left_out, store, payload })
}

fn derived_base(
    stock: &(config::Limits, limits::Limits),
    source: Sources,
    typed: bool,
    jig: &[Override],
    temper: &[Override],
) -> Result<(config::Limits, limits::Limits, u32), Problems> {
    let mut figures = stock.1;
    for item in temper {
        if item.name.as_ref() == b"live_goals" && item.value > u64::from(derived::STORE_LIVE_GOALS) {
            return Err(super::problem(
                Path::root().member(b"live_goals"),
                What::Half(super::EngineWhat::StoreLiveGoals { count: item.value, bound: derived::STORE_LIVE_GOALS }),
            ));
        }
        if (item.name.as_ref() == b"live_goals" || item.name.as_ref() == b"tool_payload")
            && let Err(why) = limits::apply(&mut figures, &item.name, item.value)
        {
            return Err(super::problem(Path::root().member(&item.name), What::Override(why)));
        }
    }
    let payload = source.payload.max(figures.tool_payload);
    let (mut jig_base, own) =
        if typed && figures.live_goals == stock.1.live_goals && figures.tool_payload == stock.1.tool_payload {
            *stock
        } else {
            base(figures.live_goals, figures.tool_payload, source)?
        };
    if !typed {
        jig_base.core.fleet.grace = super::worker_fleet_grace(&jig_base);
    }
    if !jig.is_empty() {
        derived::follow_task_overrides(&mut jig_base, jig);
    }
    Ok((jig_base, own, payload))
}

fn own_overrides(mut own: limits::Limits, temper: &[Override]) -> Result<limits::Limits, Problems> {
    for item in temper {
        if let Err(why) = limits::apply(&mut own, &item.name, item.value) {
            return Err(super::problem(Path::root().member(&item.name), What::Override(why)));
        }
    }
    Ok(own)
}

fn complete(inputs: &LimitInputs, resolved: Resolved) -> Result<Resolved, Problems> {
    let Resolved { mut jig, temper: own, left_out, store, payload, .. } = resolved;
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
        Some(&store),
    )?;
    jig = start.engine.jig.limits;
    let mut root = super::engine_limits(&mut jig, &own, None)?;
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
    Ok(Resolved { jig, temper: own, root, left_out, store, payload })
}

fn jig_floors(base: &config::Limits, overrides: &[Override], repairs: bool) -> List<CarrierFloor> {
    let mut floors = List::uncapped();
    for name in [
        b"tasks.message_bytes".as_slice(),
        b"tasks.result_bytes",
        b"tasks.inbox_bytes",
        b"hub.turn_bytes",
        b"hub.event_bytes",
        b"core.resume_bytes",
        b"hub.transcript_bytes",
        b"hub.turn_queue_bytes",
        b"io.output",
        b"hub.outcome_bytes",
        b"core.call_answer_bytes",
    ] {
        if let Some(has) = supplied(overrides, name) {
            carrier(&mut floors, name, has, jig_figure(base, &[], name), b"the source's derivation");
        }
    }
    // These two carriers also follow explicitly overridden same-set sources.
    let message = jig_figure(base, overrides, b"tasks.message_bytes");
    let result = jig_figure(base, overrides, b"tasks.result_bytes");
    let resume = jig_figure(base, overrides, b"core.resume_bytes");
    let transcript = jig_figure(base, overrides, b"hub.transcript_bytes");
    let (result, resume) = if repairs {
        (effective(&floors, b"tasks.result_bytes", result), effective(&floors, b"core.resume_bytes", resume))
    } else {
        (result, resume)
    };
    carrier(&mut floors, b"tasks.message_bytes", message, result, b"tasks.result_bytes");
    let needs =
        resume.checked_add(12_u64.checked_mul(resume / 4).expect("u32 resume carrier")).expect("u32 resume window");
    carrier(&mut floors, b"hub.transcript_bytes", transcript, needs, b"core.resume_bytes");
    floors
}

fn jig_figure(base: &config::Limits, overrides: &[Override], name: &[u8]) -> u64 {
    let row = config::names::find(name).expect("derived jig name");
    supplied(overrides, name).unwrap_or((row.value)(base))
}

fn derived_floors(own_base: &limits::Limits, own: &limits::Limits, resume: u64) -> List<CarrierFloor> {
    let mut floors = List::uncapped();
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
        let (needs, source) = if name == b"root.journal.transcript_bytes" {
            (needs.max(resume), b"the source's derivation (core.resume_bytes)".as_slice())
        } else {
            (needs, b"the source's derivation".as_slice())
        };
        carrier(&mut floors, name, has, needs, source);
    }
    floors
}

fn floor(problems: &mut Problems, name: &[u8], has: u64, needs: u64, source: &'static [u8]) {
    if has < needs {
        problems.push(Path::root().member(name), What::Short { bytes: has, of: source, needs });
    }
}
fn forge_op_floor(floors: &mut List<CarrierFloor>, has: u64, result: u64) -> u64 {
    carrier(floors, b"forge.client.op_bytes", has, result, b"the source's derivation (tasks.result_bytes)");
    has.max(result)
}

fn forge_body_floor(floors: &mut List<CarrierFloor>, has: u64, op: u64) {
    let needs = match u32::try_from(op) {
        Ok(op) => derived::forge_body(op).unwrap_or(u32::MAX),
        Err(_) => u32::MAX,
    };
    carrier(
        floors,
        b"forge_component.body_bytes",
        has,
        u64::from(needs),
        b"the source's derivation (forge.client.op_bytes)",
    );
}

fn carrier(floors: &mut List<CarrierFloor>, name: &'static [u8], has: u64, needs: u64, of: &'static [u8]) {
    if has < needs {
        floors.push(CarrierFloor { name, needs, has, of }).expect("one row per carrier check");
    }
}

fn effective(floors: &List<CarrierFloor>, name: &[u8], mut value: u64) -> u64 {
    for floor in floors {
        if floor.name == name {
            value = value.max(floor.needs);
        }
    }
    value
}

fn report_floors(floors: &List<CarrierFloor>, problems: &mut Problems) {
    for entry in floors {
        floor(problems, entry.name, entry.has, entry.needs, entry.of);
    }
}

/// Name related settings before whole-domain or codec validity can hide the relation.
fn related_limits(own: &limits::Limits, problems: &mut Problems) {
    for (name, has, needs, source) in [
        (
            b"root.brief.read_bytes".as_slice(),
            u64::from(own.root.brief.read_bytes),
            u64::from(own.root.brief.budgets.task),
            b"the source's derivation (root.brief.budgets.task)".as_slice(),
        ),
        (
            b"forge.brief_bytes",
            u64::from(own.forge.brief_bytes),
            u64::from(own.root.brief.read_bytes),
            b"the source's derivation (root.brief.read_bytes)",
        ),
        (
            b"root.journal.result_bytes",
            u64::from(own.root.journal.result_bytes),
            u64::from(own.root.brief.brief_bytes),
            b"the source's derivation (root.brief.brief_bytes)",
        ),
        (
            b"root.loads.reply_bytes",
            u64::from(own.root.loads.reply_bytes),
            u64::from(own.root.loads.bytes),
            b"root.loads.bytes",
        ),
    ] {
        floor(problems, name, has, needs, source);
    }
    for (name, has, needs, source) in [
        (
            b"forge.client.repositories".as_slice(),
            own.forge.client.repositories,
            own.forge.repositories,
            b"forge.repositories".as_slice(),
        ),
        (b"forge.client.entries", own.forge.client.entries, own.forge.entries, b"forge.entries"),
        (b"forge.client.resources", own.forge.client.resources, own.forge.holds, b"forge.holds"),
        (b"forge_component.string_bytes", own.forge_component.string_bytes, own.forge.name_bytes, b"forge.name_bytes"),
    ] {
        if has < needs {
            problems.push(Path::root().member(name), What::Against { other: source });
        }
    }
    for (name, has, bound, source) in [
        (
            b"forge.client.answer_bytes".as_slice(),
            own.forge.client.answer_bytes,
            temper_protocol_store::v25::CEILINGS.answer_name_name,
            b"the store schema's answer byte ceiling".as_slice(),
        ),
        (
            b"forge.holds",
            own.forge.holds,
            temper_protocol_store::v25::CEILINGS.tier_row_goals,
            b"the store schema's tier goals ceiling",
        ),
    ] {
        if has > bound {
            problems.push(
                Path::root().member(name),
                What::Within { bytes: u64::from(has), limit: source, bound: u64::from(bound) },
            );
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
        if let What::Jig(config::What::Capacity { requires, .. }) = problem.what
            && supplied(jig, requires).is_some()
        {
            name = Box::from(requires);
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
            let mut other = None;
            for item in temper {
                if item.name.as_ref() != b"live_goals" {
                    other = Some(item);
                    break;
                }
            }
            name = if supplied(temper, b"forge.entries").is_some() {
                Box::from(b"forge.entries".as_slice())
            } else if let Some(item) = jig.first() {
                item.name.clone()
            } else if let Some(item) = other {
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
