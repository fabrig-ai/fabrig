//! Whole owner facts encoded without clipping (protocol/client/wire.md, 1).
use super::{chips, owner, refusals};
use super::{pieces, problem, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::acts;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;
use skein_lib::List;
pub fn link(value: &d::TaskLink, limits: &w::Limits) -> Result<w::TaskLink, w::Problem> {
    Ok(match value {
        d::TaskLink::Live(value) => w::TaskLink::Live(w::TaskLinkLive { value: chips::chip(value, limits)? }),
        d::TaskLink::Retained { chip, at } => w::TaskLink::Retained(w::TaskLinkRetained {
            chip: chips::chip(chip, limits)?,
            at: match *at {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        }),
        d::TaskLink::Stored { number, status } => w::TaskLink::Stored(w::TaskLinkStored {
            number: *number,
            status: match status {
                Some(t::Status::Done) => Some(w::TaskStatus::Done),
                Some(t::Status::Failed) => Some(w::TaskStatus::Failed),
                Some(t::Status::Cancelled) => Some(w::TaskStatus::Cancelled),
                None => None,
            },
        }),
    })
}
#[must_use]
pub fn proposal(value: &d::ProposalRef) -> w::ProposalKey {
    w::ProposalKey {
        proposer: match value.proposer {
            jig_domain_people::Proposer::Task(number) => w::Proposer::Task(w::ProposerTask { value: number }),
            jig_domain_people::Proposer::Party(number) => w::Proposer::Party(w::ProposerParty { value: number }),
        },
        number: value.number,
    }
}
#[must_use]
pub fn act(value: acts::Act) -> w::ActKind {
    match value {
        acts::Act::Reorder => w::ActKind::Reorder,
        acts::Act::Talk => w::ActKind::Talk,
        acts::Act::Leave => w::ActKind::Leave,
        acts::Act::Message => w::ActKind::Message,
        acts::Act::Answer => w::ActKind::Answer,
        acts::Act::Take => w::ActKind::Take,
        acts::Act::HandBack => w::ActKind::HandBack,
        acts::Act::Accept => w::ActKind::Accept,
        acts::Act::Withdraw => w::ActKind::Withdraw,
        acts::Act::Reject => w::ActKind::Reject,
        acts::Act::PassUp => w::ActKind::PassUp,
        acts::Act::Release => w::ActKind::Release,
        acts::Act::LeaveHeld => w::ActKind::LeaveHeld,
        acts::Act::Amend => w::ActKind::Amend,
        acts::Act::Stop => w::ActKind::Stop,
        acts::Act::Cancel => w::ActKind::Cancel,
        acts::Act::Move => w::ActKind::MoveTask,
        acts::Act::Seen => w::ActKind::Seen,
        acts::Act::EditNote => w::ActKind::EditNote,
        acts::Act::DeleteNote => w::ActKind::DeleteNote,
        acts::Act::Adopt => w::ActKind::Adopt,
        acts::Act::RemoveProject => w::ActKind::RemoveProject,
        acts::Act::Rename => w::ActKind::Rename,
        acts::Act::ChangePolicy => w::ActKind::ChangePolicy,
        acts::Act::SetRoles => w::ActKind::SetRoles,
        acts::Act::MakeService => w::ActKind::MakeService,
        acts::Act::SetPool => w::ActKind::SetPool,
    }
}
#[must_use]
pub fn recipient(value: &acts::Recipient) -> w::Recipient {
    match value {
        acts::Recipient::Task(number) => w::Recipient::Task(w::RecipientTask { value: *number }),
        acts::Recipient::Party(number) => w::Recipient::Party(w::RecipientParty { value: *number }),
        acts::Recipient::Role { project, role } => {
            w::Recipient::Role(w::RecipientRole { project: *project, role: *role })
        }
        acts::Recipient::Roles { project, roles } => {
            w::Recipient::Roles(w::RecipientRoles { project: *project, roles: roles.clone() })
        }
    }
}
pub(super) fn bounds(
    models: &[crate::Model],
    value: &acts::Bounds,
    limits: &w::Limits,
) -> Result<w::Bounds, w::Problem> {
    Ok(match value {
        acts::Bounds::Create {
            authority,
            pool_left,
            charters,
            procedures,
            latest,
            budget,
            maximum_budget,
            maximum_deadline,
        } => {
            let mut rows = List::with_capacity(procedures.capacity());
            for item in procedures {
                rows.push(w::CreateProcedure { connector: item.connector, code: item.code })
                    .expect("supplied goal procedures");
            }
            w::Bounds::Create(w::BoundsCreate {
                authority: owner::authority(models, authority, limits)?,
                pool_left: *pool_left,
                charters: charters.clone(),
                procedures: rows,
                latest: match latest {
                    Some(item) => Some(item.as_nanos()),
                    None => None,
                },
                budget: *budget,
                maximum_budget: *maximum_budget,
                maximum_deadline: match maximum_deadline {
                    Some(item) => Some(item.as_nanos()),
                    None => None,
                },
            })
        }
        acts::Bounds::Amend { budget_left, grants, implications, authority, ceiling } => {
            let mut rows = List::with_capacity(grants.capacity());
            for item in grants {
                rows.push(owner::grant(item)).expect("supplied grants");
            }
            let grants = owner::grants_pieces(
                &rows,
                limits.bounds_amend_grants.min(w::CEILINGS.bounds_amend_grants),
                w::paths::BOUNDS_AMEND_GRANTS,
                limits,
            )?;
            let mut pairs = List::with_capacity(implications.capacity());
            for item in implications {
                pairs
                    .push(w::KindImplication { connector: item.connector, kind: item.kind, implies: item.implies })
                    .expect("supplied implications");
            }
            w::Bounds::Amend(w::BoundsAmend {
                budget_left: *budget_left,
                grants,
                implications: pairs,
                authority: owner::authority(models, authority, limits)?,
                ceiling: owner::authority(models, ceiling, limits)?,
            })
        }
    })
}
fn admission(models: &[crate::Model], value: &acts::Admission, limits: &w::Limits) -> Result<w::Admission, w::Problem> {
    Ok(match value {
        acts::Admission::Allowed => w::Admission::Allowed,
        acts::Admission::Proposed { to } => w::Admission::Proposed(w::AdmissionProposed { to: recipient(to) }),
        acts::Admission::Refused(value) => {
            w::Admission::Refused(w::AdmissionRefused { value: refusals::refusal(value.clone(), limits)? })
        }
        acts::Admission::Bounded { within, beyond } => {
            let beyond = match beyond {
                acts::Beyond::Proposed { to } => w::Beyond::Proposed(w::BeyondProposed { to: recipient(to) }),
                acts::Beyond::Refused(value) => {
                    w::Beyond::Refused(w::BeyondRefused { value: refusals::refusal(value.clone(), limits)? })
                }
            };
            w::Admission::Bounded(Box::new(w::AdmissionBounded { within: bounds(models, within, limits)?, beyond }))
        }
    })
}
pub fn offered(
    models: &[crate::Model],
    value: &List<acts::Offered>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    if value.capacity() > 25 {
        return Err(problem(skein_codec::Reason::Bound));
    }
    let mut rows = List::with_capacity(value.capacity());
    for item in value {
        rows.push(w::Offered {
            effect: match &item.effect {
                Some(value) => Some(effect(value)),
                None => None,
            },
            act: act(item.act),
            admission: admission(models, &item.admission, limits)?,
        })
        .expect("supplied offered acts");
    }
    offered_pieces(&rows, bound.min(16_777_216), path, limits)
}

pub fn offered_pieces(
    rows: &List<w::Offered>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
pub fn chips_pieces(
    rows: &List<w::Chip>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
pub fn links_pieces(
    rows: &List<w::TaskLink>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
pub fn task_resources_pieces(
    rows: &List<w::ResourceLine>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
pub fn delegates_pieces(
    rows: &List<w::LiveDelegate>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
#[expect(clippy::too_many_lines, reason = "one bounded task document translates each supplied field")]
pub fn document(value: &d::Task, models: &[crate::Model], limits: &w::Limits) -> Result<w::Task, w::Problem> {
    // The task document carries its dependency waits.
    if value.project != value.chip.details.project
        || value.executor != value.chip.details.executor
        || value.words != value.spec.words
        || value.parameters != value.spec.parameters
        || value.parameters.capacity() != value.spec.parameters.capacity()
    {
        return Err(problem(skein_codec::Reason::Malformed));
    }
    let mut lineage_rows = List::with_capacity(value.lineage.capacity());
    let mut lineage = List::with_capacity(value.lineage.capacity());
    for item in &value.lineage {
        lineage_rows.push(chips::chip(item, limits)?).expect("supplied lineage chips");
        lineage.push(item.number).expect("supplied lineage");
    }
    let lineage_chips = chips_pieces(
        &lineage_rows,
        limits.task_lineage_chips.min(w::CEILINGS.task_lineage_chips),
        w::paths::TASK_LINEAGE_CHIPS,
        limits,
    )?;
    let inputs =
        link_list(&value.inputs, limits, limits.task_inputs.min(w::CEILINGS.task_inputs), w::paths::TASK_INPUTS)?;
    let dependencies = link_list(
        &value.dependencies,
        limits,
        limits.task_dependencies.min(w::CEILINGS.task_dependencies),
        w::paths::TASK_DEPENDENCIES,
    )?;
    let references = link_list(
        &value.references,
        limits,
        limits.task_references.min(w::CEILINGS.task_references),
        w::paths::TASK_REFERENCES,
    )?;
    let resources = resources(&value.resources, limits)?;
    let mut proposals = List::with_capacity(value.proposals.capacity());
    for item in &value.proposals {
        proposals.push(proposal(item)).expect("supplied proposals");
    }
    Ok(w::Task {
        concluded: match value.concluded {
            Some(value) => Some(w::Concluded { by: value.by, at: value.at.as_nanos() }),
            None => None,
        },
        talk: match value.talk {
            Some(talk) => {
                Some(w::Talk { party: talk.party, opened: talk.opened.as_nanos(), last: talk.last.as_nanos() })
            }
            None => None,
        },
        session: match value.session {
            Some(session) => Some(w::Session { first: session.first, last: session.last.as_nanos() }),
            None => None,
        },
        settings: settings(value.settings),
        dependency_facts: chips::dependencies(&value.dependency_facts),
        unread: unread(&value.unread),
        llm: super::llm::choice(value.llm.as_ref(), models)?,
        kept_account: value.kept_account,
        run_wait: super::task_wait::waiting(value.run_wait),
        label: value.label,
        flags: super::me::flags(value.flags),
        last_activity: value.last_activity.as_nanos(),
        made: value.made.as_nanos(),
        tasks_left: value.tasks_left,
        waits_on: super::queries::waits(value.waits_on.clone())?,
        chip: chips::chip(&value.chip, limits)?,
        goal: value.goal,
        priority: value.priority,
        lineage_chips,
        inputs,
        project: value.project,
        requester: owner::requester(value.requester),
        lineage,
        spec: owner::specification(&value.spec)?,
        spec_revision: w::Revision { value: value.spec_revision },
        person_revision: match value.person_revision {
            Some(item) => Some(w::Revision { value: item }),
            None => None,
        },
        taken_by: value.taken_by,
        instructions: match &value.instructions {
            Some(item) => Some(owner::text(item)),
            None => None,
        },
        wake: owner::wake(value.wake),
        subscriptions: owner::subscriptions(&value.subscriptions),
        result_contract: owner::contract(&value.contract, limits)?,
        dependencies,
        references,
        cut: value.cut,
        resources,
        budget: owner::budget(&value.budget),
        authority: owner::authority(models, &value.authority, limits)?,
        hold: match &value.hold {
            Some(item) => Some(chips::held(item)),
            None => None,
        },
        proposals,
        activity: value.activity,
        changed: value.changed.as_nanos(),
        acts: offered(models, &value.acts, limits, limits.task_acts, w::paths::TASK_ACTS)?,
        result: match &value.result {
            Some(item) => Some(w::TaskResult::Ending(w::TaskResultEnding { value: chips::ending(item, limits)? })),
            None => None,
        },
        delegates: delegates(&value.delegates, limits)?,
        context: None,
    })
}

fn unread(value: &List<d::UnreadWords>) -> List<w::UnreadWords> {
    let mut result = List::with_capacity(value.capacity());
    for word in value {
        result
            .push(w::UnreadWords {
                message: word.message,
                by: owner::requester(word.by),
                at: word.at.as_nanos(),
                seen: word.seen,
                head: word.head.bytes.clone(),
                bytes: word.bytes,
            })
            .expect("supplied unread words");
    }
    result
}

fn link_list(
    value: &List<d::TaskLink>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(value.capacity());
    for item in value {
        rows.push(link(item, limits)?).expect("supplied task links");
    }
    links_pieces(&rows, bound, path, limits)
}

fn resources(value: &List<d::ResourceLine>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut resource_rows = List::with_capacity(value.capacity());
    for item in value {
        resource_rows.push(chips::resource(item)).expect("supplied resources");
    }
    task_resources_pieces(
        &resource_rows,
        limits.task_resources.min(w::CEILINGS.task_resources),
        w::paths::TASK_RESOURCES,
        limits,
    )
}

fn delegates(value: &List<d::LiveDelegate>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut delegate_rows = List::with_capacity(value.capacity());
    for item in value {
        delegate_rows
            .push(w::LiveDelegate { chip: chips::chip(&item.chip, limits)?, takeover_budget: item.takeover_budget })
            .expect("supplied delegates");
    }
    delegates_pieces(
        &delegate_rows,
        limits.task_delegates.min(w::CEILINGS.task_delegates),
        w::paths::TASK_DELEGATES,
        limits,
    )
}

#[must_use]
pub fn effect(value: &acts::Effect) -> w::Effect {
    match value {
        acts::Effect::Release(value) => w::Effect::Release(w::EffectRelease {
            what: match value {
                acts::ReleaseEffect::Judged => w::ReleaseEffect::Judged,
                acts::ReleaseEffect::RunAgain => w::ReleaseEffect::RunAgain,
                acts::ReleaseEffect::Uncertain => w::ReleaseEffect::Uncertain,
            },
        }),
        acts::Effect::Move { reservation } => w::Effect::MoveTask(w::EffectMove { reservation: *reservation }),
        acts::Effect::Cancel(value) => w::Effect::Cancel(w::Impact {
            tasks: w::ImpactCounts {
                ended: 0,
                waiting: value.tasks.waiting,
                running: value
                    .tasks
                    .idle
                    .checked_add(value.tasks.starting)
                    .expect("task bound")
                    .checked_add(value.tasks.running)
                    .expect("task bound")
                    .checked_add(value.tasks.backing_off)
                    .expect("task bound"),
                closing: value.tasks.closing,
                held: value.tasks.held,
                done: value.tasks.done,
                failed: value.tasks.failed,
                cancelled: value.tasks.cancelled,
            },
            live_runs: value.live_runs,
            proposals: value.proposals,
        }),
    }
}

fn settings(value: Option<d::TaskSettings>) -> Option<w::TaskSettings> {
    match value {
        Some(value) => Some(w::TaskSettings { task: value.task, run: value.run, newest: value.newest }),
        None => None,
    }
}
