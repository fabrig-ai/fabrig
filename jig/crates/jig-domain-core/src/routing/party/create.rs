//! Party roots checked whole, then made together (domain/people/acts.md, 1).
use crate::{
    Core, Limits,
    routing::{
        party::{dispatch::Routed, inputs as party_inputs},
        refusals::pool_refused,
    },
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};
/// Authenticated creation envelope.
pub(crate) struct Creation {
    pub request: Token,
    pub person: u64,
    pub project: u32,
    pub role: people::Role,
    pub batch: Box<[people::Member]>,
}
pub(crate) fn create_request(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, mut creation: Creation) {
    let request = creation.request;
    if core.fill >= env.limits.fill_mark {
        crate::routing::refusals::pool_again(&mut routed.work, request, people::Again::Busy);
        return;
    }
    match checked(core, env, &mut creation, false) {
        Ok((members, given)) => {
            party_inputs::begin(
                core,
                env,
                routed,
                party_inputs::Inputs {
                    person: creation.person,
                    project: creation.project,
                    role: creation.role,
                    symbols: symbols(&creation.batch),
                    members,
                    given,
                    proposal: None,
                    ids: Box::new([]),
                    at: 0,
                    row: None,
                    stubs: List::with_capacity(0),
                },
                request,
            );
        }
        Err(why) => pool_refused(&mut routed.work, request, why),
    }
}
#[expect(clippy::too_many_lines, reason = "one party batch is checked whole before any connector ask")]
pub(crate) fn checked(
    core: &mut Core,
    env: &Env<Limits>,
    creation: &mut Creation,
    proposing: bool,
) -> Result<(Box<[tasks::New]>, Option<people::Given>), people::Refusal> {
    let allowed = if proposing { None } else { budget(core, creation)? };
    let Creation { person, project, role, batch, .. } = &*creation;
    well_formed(batch)?;
    let first = core
        .counters
        .deployment()
        .tasks
        .checked_add(1)
        .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
    check_circle(core, env, batch, first)?;
    if !proposing && !core.authority.escalations_held(*project, role.number()) {
        return Err(people::Refusal::NoEscalationHolder);
    }
    let mut delegates = List::with_capacity(env.limits.people.batch);
    let mut members = List::with_capacity(env.limits.people.batch);
    for member in batch {
        for symbol in &member.symbols {
            if !core.connectors.contains(&symbol.connector) {
                return Err(people::Refusal::Limit { which: people::LimitKind::HoldKind });
            }
        }
        let new = new_task(core, env, creation, member, first, members.len())?;
        let executor = core.delegation_executor(*project, new.executor).ok_or(people::Refusal::Unknown)?;
        delegates
            .push(authority::Delegate {
                executor,
                authority: crate::admission::deadlines::authority(
                    &crate::admission::amendments::granted(
                        member.authority.clone(),
                        new.authority.models,
                        &env.limits.tasks,
                    )
                    .expect("member authority checked"),
                    new.deadline,
                    env.wall,
                ),
                symbolic: crate::symbolic_grants(&named(member, env)?, core.authority.limits().grants)
                    .ok_or(people::Refusal::Limit { which: people::LimitKind::Authority })?,
            })
            .expect("admitted batch");
        members.push(new).expect("admitted batch");
    }
    if !core.authority.escalations_held(*project, role.number()) {
        return Err(people::Refusal::NoEscalationHolder);
    }
    if proposing {
        super::propose::allowed(core, *project, delegates.into_boxed())?;
    } else {
        let (_, pool) = core.person_pool(*project, *person);
        let ask = authority::PersonAsk {
            project: *project,
            role: role.number(),
            pool: crate::translate::authority_numbers(pool),
            tasks_left: env.limits.tasks.tree_tasks,
            request: authority::PersonRequest::Create(delegates.into_boxed()),
        };
        let mut findings = Queue::uncapped();
        let checked = authority::check_request(&core.authority, &ask, &mut findings);
        if checked.answer != authority::Answer::Allow {
            let mut refusal = core.request_refusal(&ask, checked.answer, &mut findings);
            if let people::Refusal::Authority { lacking: people::Lack::TaskKind { executor }, .. } = &mut refusal
                && let Some(policy) = core.authority.role(*project, role.number())
            {
                for (member, delegate) in batch.iter().zip(delegates_for(&ask)) {
                    if !policy.authority.delegation.kinds.contains(&delegate.executor) {
                        *executor = Some(member.executor);
                        break;
                    }
                }
            }
            return Err(refusal);
        }
        let policy = core.authority.policy(*project).expect("checked project");
        if core.period_budget(*project) > policy.period_spend {
            return Err(people::Refusal::Authority {
                lacking: people::Lack::Budget { needs: core.period_budget(*project), left: policy.period_spend },
                proposable: None,
            });
        }
    }
    let mut members = members.into_boxed();
    if !proposing {
        crate::admission::deadlines::resolve(&mut members, env.wall);
    }
    let accounts = if proposing {
        core.proposed_accounts(&members)
    } else {
        core.funded_accounts(
            &members,
            tasks::Funder::Pool { project: *project, person: *person, period: core.period(*project) },
        )
    };
    if let Err(why) = accounts {
        return Err(core.account_refusal(why));
    }
    for _ in &members {
        crate::fresh(&mut core.counters, crate::Family::Task)
            .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
    }
    let mut given = None;
    if let Some((at, amount)) = allowed {
        given = Some(people::Given { task: first.checked_add(u64::from(at)).expect("admitted task number"), amount });
    }
    Ok((members, given))
}
fn new_task(
    core: &Core,
    env: &Env<Limits>,
    creation: &Creation,
    member: &people::Member,
    first: u64,
    at: u32,
) -> Result<tasks::New, people::Refusal> {
    let Creation { person, project, role, batch, .. } = creation;
    let executor = executor(core, member.executor)?;
    let models = given_models(core, *project, *role);
    let Some(given) = crate::admission::amendments::granted(member.authority.clone(), models, &env.limits.tasks) else {
        return Err(people::Refusal::Limit { which: people::LimitKind::Authority });
    };
    let spec = crate::admission::amendments::spec(
        people::Spec {
            asks: false,
            words: member.words.clone(),
            parameters: member.parameters.clone(),
            inputs: member.inputs.clone(),
        },
        &env.limits.tasks,
    )
    .ok_or(people::Refusal::Limit { which: people::LimitKind::Spec })?;
    let mut dependencies = List::with_capacity(env.limits.tasks.dependencies);
    for dependency in &member.dependencies {
        let number = match *dependency {
            people::Dependency::Batch(at) => {
                if skein_lib::length::usize_of(at) >= batch.len() {
                    return Err(people::Refusal::Unknown);
                }
                first
                    .checked_add(u64::from(at))
                    .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?
            }
            people::Dependency::Existing(task) => {
                if !core.reaches(
                    crate::admission::compound::PartyTask {
                        person: *person,
                        role: Some(creation.role),
                        project: *project,
                        task,
                    },
                    env.limits.tasks.depth,
                    &tasks::CompoundAdmission::new(&core.tasks, 0),
                ) {
                    return Err(people::Refusal::Unknown);
                }
                task
            }
        };
        if dependencies.push(number).is_err() {
            return Err(people::Refusal::Limit { which: people::LimitKind::Dependencies });
        }
    }
    let number =
        first.checked_add(u64::from(at)).ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
    let given = crate::resolved_delegate_authority(&given, &named(member, env)?, number, core.authority.limits())
        .ok_or(people::Refusal::Limit { which: people::LimitKind::Authority })?;
    let flags = tasks::Flags {
        ended_by: match member.flags.ended_by {
            people::EndedBy::Result => tasks::EndedBy::Result,
            people::EndedBy::Requester => tasks::EndedBy::Requester,
            people::EndedBy::Standing => tasks::EndedBy::Standing,
            people::EndedBy::Closer(closer) => {
                tasks::EndedBy::Closer(tasks::Closer { connector: closer.connector, code: closer.code })
            }
        },
        conversation: member.flags.conversation,
        listed: member.flags.listed,
        retention: match member.flags.retention {
            people::Retention::Kept => tasks::Retention::Kept,
            people::Retention::ForgottenOnceRead => tasks::Retention::ForgottenOnceRead,
        },
    };
    if let Some(closer) = crate::runs::delegation::missing_closer(flags, &given) {
        return Err(people::Refusal::Authority {
            proposable: None,
            lacking: people::Lack::TaskKind {
                executor: Some(people::TaskExecutor::Procedure { connector: closer.connector, code: closer.code }),
            },
        });
    }
    Ok(tasks::New {
        deadline: match member.deadline {
            people::Deadline::Named => tasks::Deadline::Named,
            people::Deadline::Span(span) => tasks::Deadline::Span(span),
        },
        number,
        project: *project,
        executor,
        llm: crate::runs::llm::choice(member.llm.as_ref()),
        spec,
        contract: contract(&member.contract),
        authority: given,
        numbers: tasks::Numbers { budget: member.authority.spend, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Pool { project: *project, person: *person, period: core.period(*project) },
        dependencies: dependencies.into_boxed(),
        holdings: Box::new([]),
        wake: match member.wake {
            Some(wake) => crate::admission::amendments::wake(wake),
            None => crate::admission::amendments::default_wake(core, &executor),
        },
        recurring: None,
        tracked: member.tracked,
        label: member.label,
        flags,
    })
}

fn contract(value: &people::Contract) -> tasks::Contract {
    match value {
        people::Contract::Change { connector, kind, words } => {
            tasks::Contract::Change { connector: *connector, kind: *kind, words: *words }
        }
        people::Contract::Report { words } => tasks::Contract::Report { words: *words },
        people::Contract::Verdict { choices } => {
            let mut rows = List::with_capacity(u32::try_from(choices.len()).expect("bounded choices"));
            for choice in choices {
                let mut fields = List::with_capacity(u32::try_from(choice.fields.len()).expect("bounded fields"));
                for field in &choice.fields {
                    fields.push(tasks::FieldRule { name: field.name.clone(), max: field.max }).expect("field count");
                }
                rows.push(tasks::Verdict {
                    code: choice.code,
                    words: choice.words,
                    followups: choice.followups,
                    fields: fields.into_boxed(),
                })
                .expect("choice count");
            }
            tasks::Contract::Verdict { choices: rows.into_boxed() }
        }
    }
}
fn check_circle(
    core: &mut Core,
    env: &Env<Limits>,
    batch: &[people::Member],
    first: u64,
) -> Result<(), people::Refusal> {
    if let Some(circle) = cycle(batch, first, env.limits.tasks.circle)? {
        // A refused batch's named tasks must never be reused by later work.
        for _ in batch {
            crate::fresh(&mut core.counters, crate::Family::Task)
                .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
        }
        return Err(people::Refusal::Circle { tasks: circle.tasks, cut: circle.cut });
    }
    Ok(())
}

// The party batch still has local references here. Reserve the prospective
// numbers only on refusal, and translate the exact circle in wait order
// (client/domain/contract/README.md, 3; domain/tasks/holds.md, 7).
fn cycle(batch: &[people::Member], first: u64, bound: u32) -> Result<Option<tasks::Circle>, people::Refusal> {
    let count = u32::try_from(batch.len()).expect("bounded batch");
    for start in 0..count {
        let mut pending = List::with_capacity(count);
        let mut parents = List::with_capacity(count);
        for _ in batch {
            parents.push(None).expect("one parent per member");
        }
        *parents.get_mut(start).expect("member") = Some(start);
        pending.push(start).expect("first member");
        for _ in batch {
            let Some(last) = pending.len().checked_sub(1) else { break };
            let current = *pending.get(last).expect("pending member");
            pending.truncate(last);
            let member = batch.get(skein_lib::length::usize_of(current)).expect("batch member");
            for dependency in &member.dependencies {
                let people::Dependency::Batch(next) = *dependency else { continue };
                if next == start {
                    let mut reverse = List::with_capacity(count);
                    let mut at = current;
                    for _ in batch {
                        reverse.push(at).expect("bounded simple circle");
                        if at == start {
                            break;
                        }
                        at = parents.get(at).expect("member").expect("visited parent");
                    }
                    let mut names = List::with_capacity(bound);
                    for number in reverse.as_slice().iter().rev() {
                        if names.len() == bound {
                            break;
                        }
                        names
                            .push(
                                first
                                    .checked_add(u64::from(*number))
                                    .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?,
                            )
                            .expect("bounded circle names");
                    }
                    return Ok(Some(tasks::Circle { tasks: names.into_boxed(), cut: reverse.len() > bound }));
                }
                let Some(parent) = parents.get_mut(next) else { continue };
                if parent.is_none() {
                    *parent = Some(current);
                    pending.push(next).expect("each member once");
                }
            }
        }
    }
    Ok(None)
}

pub(crate) fn well_formed(batch: &[people::Member]) -> Result<(), people::Refusal> {
    let mut allowed = false;
    for (at, member) in batch.iter().enumerate() {
        let agent = match member.executor {
            people::TaskExecutor::Agent { .. } => true,
            people::TaskExecutor::Procedure { .. } | people::TaskExecutor::Person(_) => false,
        };
        let as_allowed = match member.budget {
            people::Budget::Named => false,
            people::Budget::AsAllowed { .. } => true,
        };
        let why = if !people::shaped(member) {
            Some(people::Malformed::EndedBy)
        } else if member.flags.conversation && !agent {
            Some(people::Malformed::Conversation)
        } else if as_allowed {
            if allowed {
                Some(people::Malformed::AsAllowedTwice)
            } else {
                allowed = true;
                None
            }
        } else {
            None
        };
        if let Some(why) = why {
            return Err(people::Refusal::NotWellFormed { at: u32::try_from(at).expect("bounded member"), why });
        }
    }
    Ok(())
}
fn executor(core: &Core, executor: people::TaskExecutor) -> Result<tasks::Executor, people::Refusal> {
    Ok(match executor {
        people::TaskExecutor::Agent { charter } => {
            if core.charter(charter).is_none() {
                return Err(people::Refusal::NotOffered);
            }
            tasks::Executor::Agent { charter }
        }
        people::TaskExecutor::Procedure { connector, code } => {
            if code == 0 || !core.connectors.contains(&connector) {
                return Err(people::Refusal::NotOffered);
            }
            tasks::Executor::Procedure { connector, code }
        }
        people::TaskExecutor::Person(address) => tasks::Executor::Person(match address {
            people::PersonAddress::Person(person) => tasks::PersonAddress::Person(person),
            people::PersonAddress::Role(role) => tasks::PersonAddress::Role(role),
        }),
    })
}
/// The models a party's created task takes: its creator's role's in the project, whatever runs it
/// (domain/authority.md, sections 3 and 6). A procedure keeps them too, though it names no model,
/// so that its agent delegates may: as with tools, only the goal judge empties a procedure's.
fn given_models(core: &Core, project: u32, role: people::Role) -> tasks::Models {
    match core.authority.models(project, role.number()) {
        Some(models) => tasks::Models::from_parts(models.bits(), models.is_all()),
        None => tasks::Models::NONE,
    }
}
fn named(member: &people::Member, env: &Env<Limits>) -> Result<Box<[tasks::Grant]>, people::Refusal> {
    let mut authority = member.authority.clone();
    authority.grants.clone_from(&member.named);
    match crate::admission::amendments::granted(authority, tasks::Models::NONE, &env.limits.tasks) {
        Some(value) => Ok(value.grants),
        None => Err(people::Refusal::Limit { which: people::LimitKind::Authority }),
    }
}
fn delegates_for(ask: &authority::PersonAsk) -> &[authority::Delegate] {
    let authority::PersonRequest::Create(batch) = &ask.request else { unreachable!("party creation") };
    batch
}

fn budget(core: &Core, creation: &mut Creation) -> Result<Option<(u32, u64)>, people::Refusal> {
    well_formed(&creation.batch)?;
    let mut named = 0_u64;
    for member in &creation.batch {
        if let people::Budget::Named = member.budget {
            named = named
                .checked_add(member.authority.spend)
                .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
        }
    }
    let (_, pool) = core.person_pool(creation.project, creation.person);
    let ask = authority::PersonAsk {
        project: creation.project,
        role: creation.role.number(),
        pool: crate::translate::authority_numbers(pool),
        tasks_left: 0,
        request: authority::PersonRequest::Create(Box::new([])),
    };
    for (at, member) in creation.batch.iter_mut().enumerate() {
        if let people::Budget::AsAllowed { cap } = member.budget {
            let amount = authority::as_allowed(&core.authority, &ask, named, cap);
            if amount == 0 {
                return Err(people::Refusal::Authority {
                    lacking: people::Lack::Budget {
                        needs: named.saturating_add(1),
                        left: authority::as_allowed(&core.authority, &ask, 0, None),
                    },
                    proposable: None,
                });
            }
            member.authority.spend = amount;
            return Ok(Some((u32::try_from(at).expect("bounded member"), amount)));
        }
    }
    Ok(None)
}

/// Per-member symbols stay in flights until their connector resolves the holdings
/// (jig/docs/design/domain/tasks/holds.md, 2).
pub(crate) fn symbols(batch: &[people::Member]) -> Box<[Box<[people::Symbol]>]> {
    let mut symbols = List::with_capacity(u32::try_from(batch.len()).expect("bounded party batch"));
    for member in batch {
        symbols.push(member.symbols.clone()).expect("one symbol list per member");
    }
    symbols.into_boxed()
}
