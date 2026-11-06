//! temper's stock settings, the binary's defaults (protocol/configuration.md, 13.4 and 13.6).
#![expect(
    clippy::disallowed_macros,
    reason = "stock instructions are held in the binary (protocol/configuration.md, 13.4)"
)]
mod kinds;
pub use kinds::kinds;

use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core::{
    Charter, ConfiguredPolicy, RunPolicy, Template,
    settings::{Defaults, Setting},
};
use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
use jig_domain_tasks::{Contract, EndedBy, Flags, Retention};
use skein_lib::{Duration, List, Map, bytes};

/// Settings defaults supplied at a start (protocol/configuration.md, 13.4).
/// The binary uses Stock; an in-process test world may pass its own values.
#[derive(Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::large_enum_variant, reason = "worlds supply jig and forge defaults together at startup")
)]
pub enum Given {
    Stock,
    Own { jig: Defaults, forge: Box<[temper_engine_domain_forge::settings::Stock]> },
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

const FORGE: u16 = 1;
const FIRST_CHOICES: &[&[u8]] = &[b"claude-opus-5-5", b"gpt-6.1-sol"];
const CONFIGURING_FIRST_CHOICES: &[&[u8]] = &[b"claude-opus-5-5", b"gpt-6-astra", b"gpt-6.1-sol"];

/// Each project's funding span, from its first period (protocol/configuration.md, 13.4).
pub const PERIOD_SPAN: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Delegate (`change` and `delegate`), Message (delegates' questions and answers)
/// Control (a goal's `change` names its plan's members; temper/docs/design/domain/agent.md, 4.3.1),
/// Note and Recall (a chat keeps its document, a goal writes its report; temper/docs/design/domain/agent.md, 8.3),
/// Read (the forge's reads), and Settings (the configuring chat reads settings and
/// proposes), jig's families (jig's `domain/authority.md`);
/// protocol/configuration.md, 13.4.
const CHAT_TOOLS: authority::Tools = authority::Tools(1 | 2 | 4 | 128 | 256 | 512 | 1024);

const CHAT_CHARTER_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(1023);
const CODING_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(514);
const CONFIGURING_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(1920);
const REVIEWING_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(512);
const FOREMAN_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(512);
const STATUS_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(0);

/// One stock value past a limit the document set (protocol/configuration.md, 13.4).
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
    for charter in &defaults.charters {
        let setting = Setting::Charter(charter.number);
        let needed = jig_domain_core::policy_owned_bytes(&charter.policy).expect("bounded stock run policy");
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
/// model `llm` configures, with the stock's first choices before the document's rest.
/// # Panics
/// If `llm` offers no model or its budgets overflow; startup checks both before calling this.
#[must_use]
pub fn defaults(llm: &jig_protocol_config::Llm, result_bytes: u32) -> Defaults {
    checked(llm, result_bytes).expect("startup checked the default model and its stock budgets")
}

/// Checked stock construction lets startup refuse unrepresentable model-based budgets.
pub(crate) fn checked(llm: &jig_protocol_config::Llm, result_bytes: u32) -> Option<Defaults> {
    llm.models.first()?;
    let mut charters = Box::new([
        charter(1, CHAT_TURNS, CHAT_TIME, include_bytes!("stock/chat.txt"), llm, FIRST_CHOICES),
        charter(2, WORK_TURNS, WORK_TIME, include_bytes!("stock/coding.txt"), llm, FIRST_CHOICES),
        configuring_charter(llm),
        reviewer(llm),
        foreman(llm),
        status(llm),
    ]);
    // jig prices the costliest completion; one reserve turn plus one is 2C.
    let mut completion = 0;
    for model in &llm.models {
        completion = completion.max(jig_domain_core::most_costly(model)?);
    }
    let minimum_run = completion.checked_mul(2)?;
    let mut run = minimum_run.checked_mul(2)?; // K starts at 4C.
    // K's floor is four units for each of the chat's 64 run turns, 256 units.
    run = run.max(u64::from(charters.first()?.policy.turns).checked_mul(4)?);
    let chat_spend = run.checked_mul(24)?;
    // Eighty chats or goals funded at once from one pool.
    let pool_spend = chat_spend.checked_mul(80)?;
    // One chat's more in the period, which no pool reserves, for what the deployment
    // itself starts (protocol/configuration.md, 13.4).
    let period_spend = chat_spend.checked_mul(81)?;
    for charter in charters.iter_mut() {
        charter.spend = run;
    }
    let templates = Box::new([chat(result_bytes), goal(result_bytes), configuring(result_bytes)]);
    let mut policies = Map::with_capacity(1);
    policies.insert(1, project_policy(period_spend, pool_spend, chat_spend)).expect("one stock policy");
    Some(Defaults {
        models: Box::default(),
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

fn project_role(number: u32, period_spend: u64, spend: u64, requests: u16, decides: u8) -> authority::Role {
    let mut authority = ceiling(spend);
    if spend == 0 {
        authority.tools = authority::Tools(0);
        authority.models = authority::Models::NONE;
        authority.grants = Box::default();
        authority.delegation = authority::Delegation { kinds: Box::default(), tasks: 0, depth: 0 };
        authority.notes = authority::Scopes(0);
    }
    authority::Role {
        number,
        authority,
        period_spend,
        requests: authority::Requests(requests),
        decides: authority::Proposals(decides),
    }
}

// Chat and configuring charter rows (temper/docs/design/protocol/configuration.md, 13.4).
const CHAT_TURNS: u32 = 64;
// Chat and configuring charter rows: one hour (temper/docs/design/protocol/configuration.md, 13.4).
const CHAT_TIME: Duration = Duration::from_secs(3_600);
// Coding, reviewer and foreman rows (temper/docs/design/protocol/configuration.md, 13.4).
const WORK_TURNS: u32 = 800;
// Coding, reviewer and foreman rows: eight hours (temper/docs/design/protocol/configuration.md, 13.4).
const WORK_TIME: Duration = Duration::from_secs(28_800);

fn charter(
    number: u32,
    turns: u32,
    time: Duration,
    instructions: &[u8],
    llm: &jig_protocol_config::Llm,
    first_choices: &[&[u8]],
) -> Charter {
    let mut ordered = List::with_capacity(u32::try_from(llm.models.len()).expect("model count"));
    for name in first_choices {
        for model in &llm.models {
            if model.name.as_ref() == *name {
                ordered.push(model.clone()).expect("one per configured model");
            }
        }
    }
    for model in &llm.models {
        if !first_choices.contains(&model.name.as_ref()) {
            ordered.push(model.clone()).expect("one per configured model");
        }
    }
    let mut ordered = ordered.into_boxed().into_iter();
    let model = ordered.next().expect("parsed llm offers at least one model");
    let mut alternatives = List::with_capacity(u32::try_from(ordered.len()).expect("model count"));
    for alternative in ordered {
        alternatives.push(alternative).expect("one per remaining configured model");
    }
    Charter {
        rest: false,
        wake: jig_domain_tasks::WakePolicy::DEFAULT,
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
                1 => CHAT_CHARTER_TOOLS,
                2 => CODING_TOOLS,
                3 => CONFIGURING_TOOLS,
                4 => REVIEWING_TOOLS,
                6 => FOREMAN_TOOLS,
                9 => STATUS_TOOLS,
                _ => unreachable!("stock charter number"),
            },
        },
    }
}
fn configuring_charter(llm: &jig_protocol_config::Llm) -> Charter {
    let mut value =
        charter(3, CHAT_TURNS, CHAT_TIME, include_bytes!("stock/configuring.txt"), llm, CONFIGURING_FIRST_CHOICES);
    value.policy.modify = false;
    value.policy.agents = false;
    value
}
fn reviewer(llm: &jig_protocol_config::Llm) -> Charter {
    let mut value = charter(4, WORK_TURNS, WORK_TIME, include_bytes!("stock/review.txt"), llm, FIRST_CHOICES);
    value.policy.modify = false;
    value.policy.agents = false;
    value
}
fn foreman(llm: &jig_protocol_config::Llm) -> Charter {
    let mut value = charter(6, WORK_TURNS, WORK_TIME, include_bytes!("stock/foreman.txt"), llm, FIRST_CHOICES);
    value.policy.modify = false;
    value.policy.agents = false;
    value
}
fn status(llm: &jig_protocol_config::Llm) -> Charter {
    let mut value = charter(9, 4, Duration::from_secs(300), include_bytes!("stock/status.txt"), llm, FIRST_CHOICES);
    value.policy.inspect = false;
    value.policy.modify = false;
    value.policy.agents = false;
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
                authority::Executor::Charter(3),
                authority::Executor::Charter(4),
                authority::Executor::Charter(6),
                authority::Executor::Charter(9),
                authority::Executor::Procedure(2),
                authority::Executor::Procedure(3),
                authority::Executor::Procedure(4),
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
fn configuring(result_bytes: u32) -> Template {
    Template {
        number: 3,
        name: bytes::copy_of(b"configuring"),
        executor: TemplateExecutor::Charter(3),
        ..chat(result_bytes)
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

/// The forge's defaults at each start (temper/docs/design/protocol/configuration.md, 13.4).
#[must_use]
pub fn forge(_limits: &temper_engine_domain::Limits) -> Box<[temper_engine_domain_forge::settings::Stock]> {
    use temper_engine_domain_forge::{self as forge, settings};
    use temper_engine_smith::kinds::size;
    let values = [
        (
            1,
            1,
            settings::Value::Rule(settings::LandingRule {
                pattern: forge::GatePattern {
                    segments: Box::new([b"forge".as_slice().into()]),
                    last: forge::GateLast::Open(Box::default()),
                },
                enforce: true,
                ci: true,
                up_to_date: true,
                gates: Box::new([settings::RuleGate {
                    number: 1,
                    blocking: true,
                    freshness: forge::JudgeFreshness::Clean,
                }]),
                approvals: Box::default(),
            }),
        ),
        (3, 1, settings::Value::Charter(2)),
        (3, 2, settings::Value::Charter(4)),
    ];
    let mut stock = List::with_capacity(3);
    for (kind, number, value) in values {
        stock
            .push(settings::Stock { kind, scope: 0, number, value: settings::SizedValue { size: size(&value), value } })
            .expect("one landing rule and two forge charters");
    }
    stock.into_boxed()
}
