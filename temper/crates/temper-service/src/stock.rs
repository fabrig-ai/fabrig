//! temper's stock settings, the binary's defaults (protocol/configuration/stock.md and protocol/configuration/settings.md, 5).
#![expect(
    clippy::disallowed_macros,
    reason = "stock instructions are held in the binary (protocol/configuration/stock.md)"
)]
mod kinds;
pub use kinds::kinds;
mod ways;
pub use ways::ways;

use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core::{
    Charter, CharterPart, ConfiguredPolicy, Model, RunPolicy, Template, ToolFamilies,
    settings::{CatalogueModel, Defaults, ModelSource, Setting},
};
use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
use jig_domain_tasks::{Contract, EndedBy, Flags, Reasons, ResultsWake, Retention, WakePolicy, WakeRule};
use skein_lib::{Duration, List, Map, bytes};

/// Settings defaults supplied at a start (protocol/configuration/stock.md).
/// The binary uses Stock; an in-process test world may pass its own values.
#[derive(Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::large_enum_variant, reason = "worlds supply jig and forge defaults together at startup")
)]
pub enum Given {
    Stock,
    Own {
        jig: Defaults,
        forge: Box<[temper_engine_domain_forge::settings::Stock]>,
        ways: Box<[temper_engine_domain_ways::StockWay]>,
    },
}
impl Given {
    pub(crate) fn defaults(&self, llm: &jig_protocol_config::Llm, result_bytes: u32) -> Option<Defaults> {
        match self {
            Self::Stock => checked(llm, result_bytes),
            Self::Own { jig: own, .. } => {
                let mut policies = Map::with_capacity(own.policies.len());
                for (number, policy) in &own.policies {
                    policies.insert(*number, policy.clone()).expect("counted policies");
                }
                Some(Defaults {
                    models: own.models.clone(),
                    charters: own.charters.clone(),
                    templates: own.templates.clone(),
                    policies,
                    rules: own.rules.clone(),
                })
            }
        }
    }
}

/// The numbered defaults (temper/docs/design/protocol/configuration/stock.md).
struct StockModel {
    number: u32,
    provider: u32,
    name: &'static [u8],
    shown: Option<&'static [u8]>,
    effort: Option<&'static [u8]>,
    efforts: &'static [&'static [u8]],
    input: u64,
    output: u64,
    unit: u32,
}
const MODELS: &[StockModel] = &[
    StockModel {
        number: 1,
        provider: 2,
        name: b"claude-opus-5-5",
        shown: Some(b"Opus 5.5"),
        effort: Some(b"high"),
        efforts: &[b"low", b"medium", b"high", b"xhigh", b"max"],
        input: 4_000,
        output: 20_000,
        unit: 1_000_000,
    },
    StockModel {
        number: 2,
        provider: 2,
        name: b"claude-sonnet-5-5",
        shown: Some(b"Sonnet 5.5"),
        effort: Some(b"high"),
        efforts: &[b"low", b"medium", b"high", b"xhigh", b"max"],
        input: 2_000,
        output: 10_000,
        unit: 1_000_000,
    },
    StockModel {
        number: 3,
        provider: 2,
        name: b"claude-fable-5-1",
        shown: Some(b"Fable 5.1"),
        effort: Some(b"high"),
        efforts: &[b"low", b"medium", b"high", b"xhigh", b"max"],
        input: 10_000,
        output: 50_000,
        unit: 1_000_000,
    },
    StockModel {
        number: 4,
        provider: 1,
        name: b"gpt-6.1-sol",
        shown: Some(b"GPT-6.1 Sol"),
        effort: Some(b"high"),
        efforts: &[b"low", b"medium", b"high", b"xhigh", b"max"],
        input: 2_000,
        output: 10_000,
        unit: 1_000_000,
    },
    StockModel {
        number: 5,
        provider: 1,
        name: b"gpt-6-astra",
        shown: Some(b"GPT-6 Astra"),
        effort: Some(b"high"),
        efforts: &[b"low", b"medium", b"high", b"xhigh", b"max"],
        input: 10_000,
        output: 50_000,
        unit: 1_000_000,
    },
    StockModel {
        number: 6,
        provider: 1,
        name: b"gpt-6.1-terra",
        shown: Some(b"GPT-6.1 Terra"),
        effort: Some(b"medium"),
        efforts: &[b"none", b"low", b"medium", b"high", b"xhigh", b"max"],
        input: 1_000,
        output: 5_000,
        unit: 1_000_000,
    },
    StockModel {
        number: 7,
        provider: 1,
        name: b"gpt-6-luna",
        shown: Some(b"GPT-6 Luna"),
        effort: Some(b"medium"),
        efforts: &[b"none", b"low", b"medium", b"high", b"xhigh", b"max"],
        input: 100,
        output: 500,
        unit: 1_000_000,
    },
    StockModel {
        number: 8,
        provider: 3,
        name: b"demo",
        shown: None,
        effort: None,
        efforts: &[],
        input: 1,
        output: 1,
        unit: 1_000,
    },
];

pub(crate) fn models(llm: &jig_protocol_config::Llm) -> Box<[CatalogueModel]> {
    let mut models = List::with_capacity(u32::try_from(MODELS.len()).expect("stock model count"));
    for row in MODELS {
        for provider in &llm.providers {
            if provider.number != row.provider {
                continue;
            }
            for endpoint in &llm.endpoints {
                if provider.endpoint.as_deref() != Some(endpoint.name.as_ref()) {
                    continue;
                }
                let mut efforts = List::with_capacity(u32::try_from(row.efforts.len()).expect("stock efforts"));
                for effort in row.efforts {
                    efforts.push(bytes::copy_of(effort)).expect("stock effort count");
                }
                models
                    .push(CatalogueModel {
                        first_read: None,
                        number: row.number,
                        model: Model {
                            name: bytes::copy_of(row.name),
                            provider: row.provider,
                            endpoint: endpoint.number,
                            dialect: endpoint.dialect,
                            program: endpoint.program.clone(),
                            cache_life_ms: if row.number == 8 { 0 } else { 300_000 },
                            window: 1_048_576,
                            output: 8192,
                            input_price: row.input,
                            cached_price: row.input,
                            cache_write_price: None,
                            output_price: row.output,
                            price_unit: row.unit,
                            effort: match row.effort {
                                Some(effort) => Some(bytes::copy_of(effort)),
                                None => None,
                            },
                            efforts: efforts.into_boxed(),
                        },
                        shown: match row.shown {
                            Some(shown) => Some(bytes::copy_of(shown)),
                            None => None,
                        },
                        description: None,
                        picker: true,
                        withdrawn: false,
                        source: ModelSource::Stock,
                        estimated: false,
                    })
                    .expect("stock model count");
            }
        }
    }
    models.into_boxed()
}

const FORGE: u16 = 1;
const FIRST_CHOICES: &[&[u8]] = &[b"claude-opus-5-5", b"gpt-6.1-sol"];
const ASK_FIRST_CHOICES: &[&[u8]] = &[b"claude-opus-5-5", b"gpt-6-astra", b"gpt-6.1-sol"];

/// Each project's funding span, from its first period (protocol/configuration/stock.md).
pub const PERIOD_SPAN: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Delegate (`change` and `delegate`), Message (delegates' questions and answers)
/// Control (a goal's `change` names its plan's members; temper/docs/design/domain/agent/tools.md, 1),
/// Note and Recall (a chat keeps its document, a goal writes its report; temper/docs/design/domain/agent/agents.md, 3),
/// Read (the forge's reads), Settings (Ask temper proposes)
/// and Ask (Ask temper reads by address), jig's families (jig's `domain/authority.md`);
/// protocol/configuration/stock.md.
const CHAT_TOOLS: authority::Tools = authority::Tools(1 | 2 | 4 | 128 | 256 | 512 | 1024 | ToolFamilies::JIG.ask.0);

const CHAT_CHARTER_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(1023);
// A chat's families without delegate (temper/docs/design/protocol/configuration/stock.md).
const DRAFTER_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(1022);
const CODING_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(514);
const ASK_TOOLS: jig_domain_tasks::Tools =
    jig_domain_tasks::Tools(ToolFamilies::JIG.ask.0 | ToolFamilies::JIG.settings);
const REVIEWING_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(512);
const FOREMAN_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(512);
const STATUS_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(0);
const TRIAL_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(0);

/// One stock value past a limit the document set (protocol/configuration/stock.md).
/// `limit` is its override name under `engine.limits`; `setting` needs `needed`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Floor {
    pub limit: &'static [u8],
    pub configured: u64,
    pub needed: u64,
    pub setting: Setting,
}

fn floor(limit: &'static [u8], configured: u64, needed: u64, setting: Setting) -> Result<(), Floor> {
    if configured < needed { Err(Floor { limit, configured, needed, setting }) } else { Ok(()) }
}
fn count(value: usize) -> u64 {
    u64::try_from(value).expect("stock value fits u64")
}

/// Each floor the stock sets under `limits`, the first past it named.
/// # Errors
/// The first stock value past its limit. Byte prices are jig's own prices.
/// # Panics
/// If the stock's bounded byte prices overflow or its policy cannot be encoded.
pub fn floors(defaults: &Defaults, limits: &jig_protocol_config::Limits) -> Result<(), Floor> {
    let core = &limits.core;
    floor(b"tasks.charters", u64::from(core.tasks.charters), count(defaults.charters.len()), Setting::Charter(1))?;
    floor(b"core.templates", u64::from(core.templates), count(defaults.templates.len()), Setting::Template(1))?;
    floor(
        b"authority.projects",
        u64::from(core.authority.projects),
        u64::from(defaults.policies.len()),
        Setting::Policy(1),
    )?;
    let mut resolution = *core;
    // Measure the complete catalogue resolution before this byte bound can omit models.
    resolution.run_bytes = u32::MAX;
    for charter in &defaults.charters {
        let setting = Setting::Charter(charter.number);
        let resolved = jig_domain_core::settings::charters::resolved(charter, &defaults.models, &resolution)
            .expect("stock charter resolves against its catalogue");
        let needed = jig_domain_core::charter_owned_bytes(&resolved).expect("bounded resolved stock charter");
        floor(b"core.run_bytes", u64::from(core.run_bytes), needed, setting)?;
        floor(
            b"core.settings.alternatives",
            u64::from(core.settings.alternatives.min(jig_domain_tasks::MODELS)),
            count(charter.policy.alternatives.len()),
            setting,
        )?;
    }
    for template in &defaults.templates {
        let setting = Setting::Template(template.number);
        floor(b"documents.text_bytes", u64::from(core.documents.text_bytes), count(template.name.len()), setting)?;
        if let Contract::Report { words } = template.contract {
            floor(b"tasks.result_bytes", u64::from(core.tasks.result_bytes), u64::from(words), setting)?;
        }
        if let Some(grace) = template.grace {
            floor(b"tasks.grace_ms", core.tasks.grace.as_nanos() / 1_000_000, grace.as_nanos() / 1_000_000, setting)?;
        }
    }
    authority_floors(&defaults.rules.ceiling, &core.authority, Setting::Rules)?;
    for (&number, policy) in &defaults.policies {
        let setting = Setting::Policy(number);
        let snapshot = jig_domain_core::policy_snapshot(&policy.policy, &policy.permissions, &policy.templates)
            .expect("stock policy snapshot");
        let needed = jig_domain_people::policy_bytes(&snapshot).expect("bounded stock policy encoding");
        floor(b"core.policy_bytes", core.policy_bytes, needed, setting)?;
        floor(
            b"core.settings.permissions",
            u64::from(core.settings.permissions),
            count(policy.permissions.len()),
            setting,
        )?;
        floor(b"authority.roles", u64::from(core.authority.roles), count(policy.policy.roles.len()), setting)?;
        authority_floors(&policy.policy.ceiling, &core.authority, setting)?;
        grant_floors(&policy.policy.projections, &core.authority, setting)?;
        for role in &policy.policy.roles {
            authority_floors(&role.authority, &core.authority, setting)?;
        }
    }
    Ok(())
}
fn authority_floors(value: &authority::Authority, limits: &authority::Limits, setting: Setting) -> Result<(), Floor> {
    grant_floors(&value.grants, limits, setting)?;
    floor(b"authority.executors", u64::from(limits.executors), count(value.delegation.kinds.len()), setting)
}
fn grant_floors(grants: &[authority::Grant], limits: &authority::Limits, setting: Setting) -> Result<(), Floor> {
    floor(b"authority.grants", u64::from(limits.grants), count(grants.len()), setting)?;
    for grant in grants {
        floor(b"authority.segments", u64::from(limits.segments), count(grant.pattern.segments.len()), setting)?;
        for segment in &grant.pattern.segments {
            floor(b"authority.segment_bytes", u64::from(limits.segment_bytes), count(segment.len()), setting)?;
        }
        let (authority::Last::Exact(last) | authority::Last::Open(last)) = &grant.pattern.last;
        floor(b"authority.segment_bytes", u64::from(limits.segment_bytes), count(last.len()), setting)?;
    }
    Ok(())
}

/// The defaults temper passes jig's builder at each start. Each charter offers every
/// stock model, with the stock's first choices before the catalogue's rest.
/// # Panics
/// If `llm` offers no model or its budgets overflow; startup checks both before calling this.
#[must_use]
pub fn defaults(llm: &jig_protocol_config::Llm, result_bytes: u32) -> Defaults {
    checked(llm, result_bytes).expect("startup checked the default model and its stock budgets")
}

/// Stock charters and budgets over an explicitly supplied world's catalogue.
/// The caller passes these as `Given::Own` (protocol/configuration/stock.md).
/// # Panics
/// If the supplied catalogue is empty or its budgets overflow.
#[must_use]
pub fn with_models(models: Box<[CatalogueModel]>, result_bytes: u32) -> Defaults {
    checked_with(models, result_bytes).expect("world's catalogue and budgets")
}

/// Checked stock construction lets startup refuse unrepresentable model-based budgets.
pub(crate) fn checked(llm: &jig_protocol_config::Llm, result_bytes: u32) -> Option<Defaults> {
    checked_with(models(llm), result_bytes)
}

pub(crate) fn checked_with(models: Box<[CatalogueModel]>, result_bytes: u32) -> Option<Defaults> {
    models.first()?;
    let mut charters = Box::new([
        charter(1, CHAT_TURNS, CHAT_TIME, include_bytes!("stock/chat.txt"), &models, FIRST_CHOICES),
        charter(2, WORK_TURNS, WORK_TIME, include_bytes!("stock/coding.txt"), &models, FIRST_CHOICES),
        reviewer(&models),
        coordinator(&models),
        standing_role(&models),
        foreman(&models),
        status(&models),
        trial(&models),
        ask_charter(&models),
        drafter(&models),
    ]);
    // jig prices the costliest completion; one reserve turn plus one is 2C.
    let mut completion = 0;
    for model in &models {
        completion = completion.max(jig_domain_core::most_costly(&model.model)?);
    }
    let minimum_run = completion.checked_mul(2)?;
    let mut run = minimum_run.checked_mul(2)?; // K starts at 4C.
    // K's floor is four units for each of the chat's 64 run turns, 256 units.
    run = run.max(u64::from(charters.first()?.policy.turns).checked_mul(4)?);
    let chat_spend = run.checked_mul(24)?;
    // Eighty chats or goals funded at once from one pool.
    let pool_spend = chat_spend.checked_mul(80)?;
    // One chat's more in the period, which no pool reserves, for what the deployment
    // itself starts (protocol/configuration/stock.md).
    let period_spend = chat_spend.checked_mul(81)?;
    for charter in charters.iter_mut() {
        charter.spend = run;
    }
    let templates = Box::new([
        chat(result_bytes),
        goal(result_bytes),
        draft_goal(result_bytes),
        role(result_bytes),
        ask(result_bytes),
        design(result_bytes),
    ]);
    let mut policies = Map::with_capacity(1);
    policies.insert(1, project_policy(period_spend, pool_spend, chat_spend)).expect("one stock policy");
    Some(Defaults {
        models,
        charters,
        templates,
        policies,
        rules: authority::Rules {
            ceiling: ceiling(pool_spend),
            period_spend,
            minimum_run_spend: minimum_run,
            maximum_run_spend: run,
            implies: authority::Implies::new(Box::default(), 0).expect("empty stock implications"),
            requirements: Box::default(),
        },
    })
}

fn project_policy(period_spend: u64, pool_spend: u64, task_spend: u64) -> ConfiguredPolicy {
    ConfiguredPolicy {
        policy: authority::Policy {
            escalation_role: Some(0),
            ceiling: ceiling(pool_spend),
            projections: grants(),
            period_spend,
            roles: Box::new([
                project_role(0, pool_spend, task_spend, 511, 31),
                project_role(1, pool_spend, task_spend, 255, 31),
                project_role(2, task_spend, task_spend, 185, 0),
                project_role(3, 0, 0, 128, 0),
            ]),
            requirements: Box::default(),
        },
        permissions: Box::new([
            jig_domain_people::PermissionRole { connector: FORGE, permission: 3, role: 1 },
            jig_domain_people::PermissionRole { connector: FORGE, permission: 2, role: 2 },
            jig_domain_people::PermissionRole { connector: FORGE, permission: 1, role: 3 },
        ]),
        templates: Box::default(),
    }
}

/// A role delegates to its own role alone (domain/agent/agents.md, 2).
fn project_role(number: u32, period_spend: u64, spend: u64, requests: u16, decides: u8) -> authority::Role {
    let mut authority = ceiling(spend);
    if spend == 0 {
        authority.tools = authority::Tools(0);
        authority.models = authority::Models::NONE;
        authority.grants = Box::default();
        authority.delegation = authority::Delegation { kinds: Box::default(), tasks: 0, depth: 0 };
        authority.notes = authority::Scopes(0);
    } else {
        let mut kinds = List::with_capacity(u32::try_from(authority.delegation.kinds.len()).expect("stock kinds fit"));
        for kind in &authority.delegation.kinds {
            let keep = match *kind {
                authority::Executor::Role(role) => role == number,
                authority::Executor::Job(_) | authority::Executor::Charter(_) | authority::Executor::Procedure(_) => {
                    true
                }
            };
            if keep {
                kinds.push(*kind).expect("narrowed stock kinds fit");
            }
        }
        authority.delegation.kinds = kinds.into_boxed();
    }
    authority::Role {
        number,
        authority,
        period_spend,
        requests: authority::Requests(requests),
        decides: authority::Proposals(decides),
    }
}

// Chat, Ask temper and drafter charter rows (temper/docs/design/protocol/configuration/stock.md).
const CHAT_TURNS: u32 = 64;
// Chat, Ask temper and drafter charter rows: one hour (temper/docs/design/protocol/configuration/stock.md).
const CHAT_TIME: Duration = Duration::from_secs(3_600);
// Coding, reviewer and foreman rows (temper/docs/design/protocol/configuration/stock.md).
const WORK_TURNS: u32 = 800;
// Coding, reviewer and foreman rows: eight hours (temper/docs/design/protocol/configuration/stock.md).
const WORK_TIME: Duration = Duration::from_secs(28_800);

fn charter(
    number: u32,
    turns: u32,
    time: Duration,
    instructions: &[u8],
    models: &[CatalogueModel],
    first_choices: &[&[u8]],
) -> Charter {
    let mut ordered = List::with_capacity(u32::try_from(models.len()).expect("model count"));
    for name in first_choices {
        for model in models {
            if model.model.name.as_ref() == *name {
                ordered.push(model.model.clone()).expect("one per configured model");
            }
        }
    }
    if ordered.is_empty() {
        let mut first = models.first().expect("nonempty catalogue");
        for model in models {
            if model.number < first.number {
                first = model;
            }
        }
        ordered.push(first.model.clone()).expect("first model");
    }
    let mut ordered = ordered.into_boxed().into_iter();
    let model = ordered.next().expect("parsed llm offers at least one model");
    let mut alternatives = List::with_capacity(u32::try_from(ordered.len()).expect("model count"));
    for alternative in ordered {
        alternatives.push(alternative).expect("one per remaining configured model");
    }
    Charter {
        parts: Box::new([]),
        rest: true,
        wake: WakePolicy::DEFAULT,
        documents: Box::new([]),
        number,
        hosts: jig_domain_fleet::Kinds::Workers,
        spend: 0,
        policy: RunPolicy {
            instructions: bytes::copy_of(instructions),
            model,
            alternatives: alternatives.into_boxed(),
            waiting: Duration::from_millis(10_000),
            resume: true,
            turns,
            time,
            reserve_turns: 1,
            reserve_time: Duration::from_millis(60_000),
            inspect: true,
            modify: number != 1,
            shell: false,
            agents: true,
            call_timeout: Duration::from_millis(10_000),
            tools: match number {
                1 | 5 => CHAT_CHARTER_TOOLS,
                2 => CODING_TOOLS,
                4 => REVIEWING_TOOLS,
                6 => FOREMAN_TOOLS,
                9 => STATUS_TOOLS,
                10 => TRIAL_TOOLS,
                11 => ASK_TOOLS,
                12 => DRAFTER_TOOLS,
                13 => jig_domain_tasks::Tools(2047),
                _ => unreachable!("stock charter number"),
            },
        },
    }
}
fn ask_charter(models: &[CatalogueModel]) -> Charter {
    let mut value = charter(11, CHAT_TURNS, CHAT_TIME, include_bytes!("stock/ask.txt"), models, ASK_FIRST_CHOICES);
    value.policy.inspect = false;
    value.policy.modify = false;
    value.policy.shell = false;
    value.policy.agents = false;
    value
}
// The drafter's chat policy (temper/docs/design/protocol/configuration/stock.md;
// temper/docs/design/domain/agent/agents.md, 4).
fn drafter(models: &[CatalogueModel]) -> Charter {
    let mut value = charter(12, CHAT_TURNS, CHAT_TIME, include_bytes!("stock/drafter.txt"), models, FIRST_CHOICES);
    value.policy.inspect = false;
    value.policy.modify = false;
    value.policy.shell = false;
    value.policy.agents = true;
    value
}
fn reviewer(models: &[CatalogueModel]) -> Charter {
    let mut value = charter(4, WORK_TURNS, WORK_TIME, include_bytes!("stock/review.txt"), models, FIRST_CHOICES);
    value.policy.modify = false;
    value.policy.agents = false;
    value
}
// temper/docs/design/protocol/configuration/stock.md; jig/docs/design/domain/tasks/messages.md, 6.
const COORDINATOR_WAKE: WakePolicy = WakePolicy {
    words: WakeRule::Immediate,
    notices: WakeRule::Immediate,
    news: WakeRule::Paced { span: Duration::from_secs(1_800) },
    results: ResultsWake::LastOrFailure,
    questions: true,
    answers: true,
    timers: true,
};

fn coordinator(models: &[CatalogueModel]) -> Charter {
    let (shared, parts) = sections(include_bytes!("stock/coordinator.md"));
    let mut value = charter(5, WORK_TURNS, WORK_TIME, &shared, models, FIRST_CHOICES);
    value.parts = parts;
    value.policy.waiting = Duration::ZERO;
    value.policy.resume = false;
    value.policy.shell = false;
    value.wake = COORDINATOR_WAKE;
    value
}

// temper/docs/design/domain/agent/standing.md, 2: every delegate's result concerns the role.
const ROLE_WAKE: WakePolicy = WakePolicy { results: ResultsWake::Each, ..COORDINATOR_WAKE };

fn standing_role(models: &[CatalogueModel]) -> Charter {
    let (shared, parts) = sections(include_bytes!("stock/role.md"));
    let mut value = charter(13, WORK_TURNS, WORK_TIME, &shared, models, FIRST_CHOICES);
    value.parts = parts;
    value.policy.waiting = Duration::ZERO;
    value.policy.inspect = false;
    value.policy.modify = false;
    value.policy.shell = false;
    value.policy.agents = false;
    value.wake = ROLE_WAKE;
    value
}

#[expect(clippy::disallowed_methods, reason = "compiled stock headings and UTF-8 are held by the parts test")]
fn sections(text: &'static [u8]) -> (Box<[u8]>, Box<[CharterPart]>) {
    let text = core::str::from_utf8(text).expect("stock charter is UTF-8");
    let mut sections = text.strip_prefix("## ").expect("stock section heading").split("\n## ");
    let (heading, shared) = sections.next().expect("stock shared section").split_once('\n').expect("heading line");
    assert_eq!(heading, "shared", "stock shared heading");
    let mut parts = List::with_capacity(4);
    for section in sections {
        let (heading, instructions) = section.split_once('\n').expect("stock part heading line");
        let names = heading.strip_prefix("part, on: ").expect("stock part heading");
        let mut on = Reasons::EMPTY;
        for name in names.split(", ") {
            on = on.union(Reasons::named(name.as_bytes()).expect("known stock part reason"));
        }
        parts
            .push(CharterPart { on, instructions: bytes::copy_of(instructions.trim_matches('\n').as_bytes()) })
            .expect("at most four stock charter parts");
    }
    (bytes::copy_of(shared.trim_matches('\n').as_bytes()), parts.into_boxed())
}

fn foreman(models: &[CatalogueModel]) -> Charter {
    let mut value = charter(6, WORK_TURNS, WORK_TIME, include_bytes!("stock/foreman.txt"), models, FIRST_CHOICES);
    value.policy.modify = false;
    value.policy.agents = false;
    value
}
fn status(models: &[CatalogueModel]) -> Charter {
    let mut value = charter(9, 4, Duration::from_secs(300), include_bytes!("stock/status.txt"), models, FIRST_CHOICES);
    value.policy.inspect = false;
    value.policy.modify = false;
    value.policy.agents = false;
    value
}
fn trial(models: &[CatalogueModel]) -> Charter {
    let mut value = charter(10, 2, Duration::from_secs(120), include_bytes!("stock/trial.txt"), models, FIRST_CHOICES);
    value.policy.inspect = false;
    value.policy.modify = false;
    value.policy.agents = false;
    value.policy.resume = false;
    value
}

fn grants() -> Box<[authority::Grant]> {
    let mut grants = List::with_capacity(14);
    for kind in [1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 16, 17] {
        grants
            .push(authority::Grant {
                connector: FORGE,
                kind,
                pattern: authority::Pattern {
                    segments: Box::new([bytes::copy_of(b"forge")]),
                    last: authority::Last::Open(Box::default()),
                },
            })
            .expect("stock forge grants");
    }
    grants.into_boxed()
}
fn ceiling(ceiling_spend: u64) -> authority::Authority {
    authority::Authority {
        tools: CHAT_TOOLS,
        models: authority::Models::ALL,
        grants: grants(),
        delegation: authority::Delegation {
            kinds: Box::new([
                authority::Executor::Charter(1),
                authority::Executor::Charter(2),
                authority::Executor::Charter(4),
                authority::Executor::Charter(5),
                authority::Executor::Charter(6),
                authority::Executor::Charter(9),
                authority::Executor::Charter(11),
                authority::Executor::Charter(12),
                authority::Executor::Charter(13),
                authority::Executor::Procedure(2),
                authority::Executor::Procedure(3),
                authority::Executor::Procedure(4),
                authority::Executor::Procedure(6),
                authority::Executor::Job(3),
                authority::Executor::Job(4),
                authority::Executor::Role(0),
                authority::Executor::Role(1),
                authority::Executor::Role(2),
            ]),
            tasks: 384,
            depth: 32,
        },
        budget: authority::Budget { spend: ceiling_spend, deadline: None },
        notes: authority::Scopes::GOAL,
        note_resources: Box::default(),
    }
}
fn chat(result_bytes: u32) -> Template {
    Template {
        resources: Box::new([]),
        number: 1,
        name: bytes::copy_of(b"chat"),
        executor: TemplateExecutor::Charter(1),
        contract: Contract::Report { words: result_bytes },
        shallower: 1,
        budget: TemplateBudget::AsAllowed { cap: None },
        deadline: TemplateDeadline::None,
        tracked: None,
        flags: Flags { ended_by: EndedBy::Requester, conversation: true, listed: false, retention: Retention::Kept },
        label: 1,
        grace: Some(Duration::from_millis(5000)),
        llm: None,
    }
}
fn ask(result_bytes: u32) -> Template {
    Template { number: 5, name: bytes::copy_of(b"ask"), executor: TemplateExecutor::Charter(11), ..chat(result_bytes) }
}
// A design session is a chat (temper/docs/design/protocol/configuration/stock.md).
fn design(result_bytes: u32) -> Template {
    Template {
        number: 6,
        name: bytes::copy_of(b"design session"),
        executor: TemplateExecutor::Charter(12),
        ..chat(result_bytes)
    }
}
fn draft_goal(result_bytes: u32) -> Template {
    Template {
        number: 4,
        name: bytes::copy_of(b"goal for a draft"),
        executor: TemplateExecutor::Charter(5),
        resources: Box::new([jig_domain_people::Symbol { connector: 1, symbol: 1 }]),
        flags: Flags {
            ended_by: EndedBy::Closer(jig_domain_tasks::Closer { connector: 1, code: 6 }),
            ..goal(result_bytes).flags
        },
        ..goal(result_bytes)
    }
}
fn role(result_bytes: u32) -> Template {
    Template {
        number: 7,
        name: bytes::copy_of(b"standing role"),
        deadline: TemplateDeadline::None,
        tracked: None,
        flags: Flags { ended_by: EndedBy::Standing, conversation: false, listed: true, retention: Retention::Kept },
        label: 3,
        ..goal(result_bytes)
    }
}
fn goal(result_bytes: u32) -> Template {
    Template {
        resources: Box::new([]),
        number: 2,
        name: bytes::copy_of(b"goal"),
        executor: TemplateExecutor::Chosen,
        contract: Contract::Report { words: result_bytes },
        shallower: 1,
        budget: TemplateBudget::Given,
        deadline: TemplateDeadline::Given,
        tracked: Some(1),
        flags: Flags { ended_by: EndedBy::Result, conversation: false, listed: true, retention: Retention::Kept },
        label: 2,
        grace: None,
        llm: None,
    }
}

/// The forge's defaults at each start (temper/docs/design/protocol/configuration/stock.md).
#[must_use]
pub fn forge(_limits: &temper_engine_domain::Limits) -> Box<[temper_engine_domain_forge::settings::Stock]> {
    use temper_engine_domain_forge::settings;
    use temper_engine_smith::kinds::size;
    let values = [
        (2, 1, settings::Value::Rule(review_rule())),
        (3, 1, settings::Value::Charter(2)),
        (3, 2, settings::Value::Charter(4)),
        (11, 4, settings::Value::Goal(settings::TemplateGoal { design: false, close_out: false })),
        // Each stock charter's words (temper/docs/design/protocol/configuration/stock.md).
        (9, 1, settings::Value::CharterWords(bytes::copy_of(b"talks with you and asks for the changes"))),
        (
            9,
            2,
            settings::Value::CharterWords(bytes::copy_of(b"makes each change, repairs it and resolves its conflicts")),
        ),
        (
            9,
            4,
            settings::Value::CharterWords(bytes::copy_of(
                b"reviews each change at its head, and holds it until it approves",
            )),
        ),
        (
            9,
            5,
            settings::Value::CharterWords(bytes::copy_of(b"cuts a goal's draft into changes and lands the goal whole")),
        ),
        (
            9,
            6,
            settings::Value::CharterWords(bytes::copy_of(
                b"answers a coding agent's question from the goal's design, or passes it up",
            )),
        ),
        (
            9,
            9,
            settings::Value::CharterWords(bytes::copy_of(
                b"writes the few lines at the top of Home, from what landed and what waits",
            )),
        ),
        (
            9,
            10,
            settings::Value::CharterWords(bytes::copy_of(b"answers one word, so that a new model is known to run")),
        ),
        (
            9,
            11,
            settings::Value::CharterWords(bytes::copy_of(b"explains how work runs here, and proposes changes to it")),
        ),
        (
            9,
            12,
            settings::Value::CharterWords(bytes::copy_of(
                b"writes a draft with you, one question at a time, and starts nothing",
            )),
        ),
        (
            9,
            13,
            settings::Value::CharterWords(bytes::copy_of(b"owns its work for the project's life and keeps its Now")),
        ),
    ];
    let mut stock = List::with_capacity(14);
    for (kind, number, value) in values {
        stock
            .push(settings::Stock { kind, scope: 0, number, value: settings::SizedValue { size: size(&value), value } })
            .expect("one landing rule, two forge charters, ten charter lines and one goal template's work");
    }
    stock.into_boxed()
}

/// Kind 2's stock rule 1, also the Agents review way (protocol/configuration/stock.md and protocol/configuration/settings.md, 8).
fn review_rule() -> temper_engine_domain_forge::settings::LandingRule {
    use temper_engine_domain_forge::{self as forge, settings};
    settings::LandingRule {
        pattern: forge::GatePattern {
            segments: Box::new([b"branch".as_slice().into()]),
            last: forge::GateLast::Open(Box::default()),
        },
        enforce: true,
        ci: true,
        up_to_date: true,
        gates: Box::new([settings::RuleGate { number: 1, blocking: true, freshness: forge::JudgeFreshness::Clean }]),
        approvals: Box::default(),
    }
}
