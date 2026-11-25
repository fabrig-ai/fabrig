//! Whole owner facts encoded without clipping (protocol/client/wire.md, 1).
use super::{chips, owner, refusals};
use super::{pieces, problem, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::acts;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::List;
pub fn link(value: &d::TaskLink, limits: &w::Limits) -> Result<w::TaskLink, w::Problem> {
    mapped::task_link_to_record(value, &(&[], limits))
}
#[must_use]
pub fn proposal(value: &d::ProposalRef) -> w::ProposalKey {
    mapped::proposal_key_to_record(value)
}
#[must_use]
pub fn act(value: acts::Act) -> w::ActKind {
    mapped::act_kind_to_record(&value)
}
#[must_use]
pub fn recipient(value: &acts::Recipient) -> w::Recipient {
    mapped::recipient_to_record(value)
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
                rows.push(mapped::create_procedure_to_record(item)).expect("supplied goal procedures");
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
                pairs.push(mapped::kind_implication_to_record(item)).expect("supplied implications");
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
    // A task can retain more subscriptions after restart than this client can show.
    let subscriptions =
        owner::subscriptions(&value.subscriptions, limits.subscriptions_entries.min(w::CEILINGS.subscriptions_entries));
    let cut = value.cut || subscriptions.entries.len() < value.subscriptions.len();
    Ok(w::Task {
        concluded: match value.concluded {
            Some(value) => Some(mapped::concluded_to_record(&value)),
            None => None,
        },
        talk: match value.talk {
            Some(talk) => Some(mapped::talk_to_record(&talk)),
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
        waits_on: super::queries::waits(&value.waits_on)?,
        wait: super::waits::first(value.wait.as_ref())?,
        nearest: super::waits::nearest(value.nearest),
        waits: super::waits::list(&value.waits)?,
        wait_count: value.wait_count,
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
        subscriptions,
        result_contract: owner::contract(&value.contract, limits)?,
        dependencies,
        references,
        cut,
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
        delegate_rows.push(mapped::live_delegate_to_record(item, &(&[], limits))?).expect("supplied delegates");
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
    mapped::effect_to_record(value)
}

fn settings(value: Option<d::TaskSettings>) -> Option<w::TaskSettings> {
    match value {
        Some(value) => Some(mapped::task_settings_to_record(&value)),
        None => None,
    }
}
