//! Jig objects shown by an application's page remain jig's working-set objects.
//! One window names them; their chips and deciding documents remain live through
//! jig's watches and reads. No application value is held here (root.md, section 7).
use crate::{
    Request, Ui,
    state::{
        handles::Holder,
        objects::{Body, Source},
    },
};
use jig_client::contract;
use jig_client::contract::{ApplicationPart, Delivery, ObjectKey, Read, ReadResult, ReadValue, Subject};
use skein_lib::{Env, List, Queue};
pub(crate) const WINDOW: u16 = u16::MAX;
fn source(_domain: &Ui) -> Source {
    Source::Window { number: WINDOW }
}
pub(crate) fn show(
    domain: &mut Ui,
    _env: &Env<crate::Config>,
    objects: List<ObjectKey>,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    if domain.confirmed.is_none() || domain.link != jig_client::Link::Live {
        return;
    }
    let source = source(domain);
    let mut unchanged = true;
    let mut count = 0_u32;
    for (key, _) in domain.objects.ids() {
        if domain.objects.named(*key, source) {
            count = count.checked_add(1).expect("named objects fit their capacity");
            if !objects.as_slice().contains(key) {
                unchanged = false;
            }
        }
    }
    if unchanged && count == objects.len() {
        return;
    }
    close(domain, out);
    let missed = domain.objects.replace(source, objects.as_slice());
    if missed > 0 {
        crate::state::page::routing::refuse(domain, contract::Limit::Objects);
    }
    let mut tasks = List::with_capacity(lent.domain_limits().rows);
    let mut deciding = false;
    for key in &objects {
        if !domain.objects.named(*key, source) {
            continue;
        }
        read(domain, *key, out);
        match key {
            ObjectKey::Task(task) => {
                if tasks.push(*task).is_err() {
                    crate::state::page::routing::refuse(domain, contract::Limit::Objects);
                }
            }
            ObjectKey::Proposal(_)
            | ObjectKey::Escalation(_)
            | ObjectKey::Question { .. }
            | ObjectKey::PersonTask(_)
            | ObjectKey::Result(_)
            | ObjectKey::Note { .. } => deciding = true,
        }
    }
    if !tasks.is_empty() {
        open(domain, Subject::Chips { tasks }, out);
    }
    if deciding {
        open(domain, Subject::Inbox { window: lent.domain_limits().rows }, out);
    }
    domain.changed();
}
fn open(domain: &mut Ui, subject: Subject, out: &mut Queue<Request>) {
    if domain.hold_watch(subject, Holder::Application, out).is_none() {
        crate::state::page::routing::refuse(domain, contract::Limit::Watches);
    }
}
fn close(domain: &mut Ui, out: &mut Queue<Request>) {
    for _ in 0..domain.handles.watches.len() {
        let mut found = None;
        for (handle, held) in &domain.handles.watches {
            if held.owner == Holder::Application {
                found = Some(*handle);
                break;
            }
        }
        let Some(handle) = found else {
            break;
        };
        domain.release(handle, out);
    }
}
fn read(domain: &mut Ui, key: ObjectKey, out: &mut Queue<Request>) {
    let what = crate::state::page::routing::document_read(key);
    if !domain.handles.pending(&what) && domain.handles.read(source(domain), what, out).is_none() {
        crate::state::page::routing::refuse(domain, contract::Limit::Reads);
    }
}
pub(crate) fn delivery(
    domain: &mut Ui,
    env: &Env<crate::Config>,
    delivery: Delivery,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    match delivery {
        Delivery::Proposal(proposal) => {
            let key = ObjectKey::Proposal(proposal.key);
            if domain.objects.named(key, source(domain))
                && put(domain, env, key, Body::Proposal(proposal), out, lent, false)
            {
                domain.changed();
            }
        }
        Delivery::Chips(chips) => {
            for chip in chips.into_boxed() {
                let key = ObjectKey::Task(chip.task);
                if domain.objects.named(key, source(domain))
                    && put(domain, env, key, Body::Chip(chip), out, lent, false)
                {
                    domain.changed();
                }
            }
        }
        Delivery::Inbox(_) => {
            let mut keys = List::with_capacity(env.limits.objects);
            for (key, _) in domain.objects.ids() {
                assert!(keys.push(*key).is_ok(), "objects fit their registry");
            }
            for key in &keys {
                if domain.objects.named(*key, source(domain)) {
                    read(domain, *key, out);
                }
            }
        }
        Delivery::Me(_)
        | Delivery::Project(_)
        | Delivery::Task(_)
        | Delivery::Tree(_)
        | Delivery::Tasks(_)
        | Delivery::Run(_)
        | Delivery::RunState(_)
        | Delivery::Settings(_)
        | Delivery::Deployment(_)
        | Delivery::Accounts(_) => {}
    }
}
pub(crate) fn read_result(
    domain: &mut Ui,
    env: &Env<crate::Config>,
    what: Read,
    result: ReadResult,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let Some(key) = key(&what) else {
        return;
    };
    if !domain.objects.named(key, source(domain)) {
        return;
    }
    match result {
        ReadResult::None(why) => match why {
            contract::ReadNone::SignedOut => crate::state::ui::signed_out(domain, out, lent),
            contract::ReadNone::TooLinked { joins } => {
                crate::tree::notices::kept::show(domain, crate::tree::notices::kept::Kind::TooLinked { joins });
            }
            contract::ReadNone::TooLarge { .. }
            | contract::ReadNone::Malformed
            | contract::ReadNone::Unknown
            | contract::ReadNone::Gone
            | contract::ReadNone::Refused
            | contract::ReadNone::Busy
            | contract::ReadNone::Unreachable => {
                crate::state::page::shown::status(domain, None, None, true);
                crate::state::page::routing::refuse(domain, contract::Limit::Reads);
            }
        },
        ReadResult::Value(value) => {
            let stored = match value {
                ReadValue::Task(document) => put(domain, env, key, Body::Task(document), out, lent, true),
                ReadValue::Ended(document) => put(domain, env, key, Body::Ended(document), out, lent, true),
                ReadValue::Proposal(document) => put(domain, env, key, Body::Proposal(document), out, lent, true),
                ReadValue::Escalation(document) => put(domain, env, key, Body::Escalation(document), out, lent, true),
                ReadValue::Question(document) => put(domain, env, key, Body::Question(document), out, lent, true),
                ReadValue::PersonTask(document) => put(domain, env, key, Body::PersonTask(document), out, lent, true),
                ReadValue::Result(document) => put(domain, env, key, Body::Result(document), out, lent, true),
                ReadValue::Tasks(_)
                | ReadValue::Me(_)
                | ReadValue::History(_)
                | ReadValue::Activity(_)
                | ReadValue::Turn(_)
                | ReadValue::Inbox(_)
                | ReadValue::EndedPage(_)
                | ReadValue::Spend(_)
                | ReadValue::Project(_)
                | ReadValue::Notes(_)
                | ReadValue::Settings(_)
                | ReadValue::Setting(_)
                | ReadValue::Revision(_)
                | ReadValue::Revisions(_)
                | ReadValue::Deployment(_) => false,
            };
            if stored {
                domain.changed();
            }
        }
    }
}
fn key(what: &Read) -> Option<ObjectKey> {
    match what {
        Read::Task { task } => Some(ObjectKey::Task(*task)),
        Read::Proposal { proposal } => Some(ObjectKey::Proposal(*proposal)),
        Read::Escalation { task } => Some(ObjectKey::Escalation(*task)),
        Read::Question { task, message } => Some(ObjectKey::Question { task: *task, message: *message }),
        Read::PersonTask { task } => Some(ObjectKey::PersonTask(*task)),
        Read::Result { task } => Some(ObjectKey::Result(*task)),
        Read::Tasks { .. }
        | Read::Me
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Turn { .. }
        | Read::Inbox { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Notes { .. }
        | Read::Settings
        | Read::Setting(_)
        | Read::Revision(_)
        | Read::Revisions(_)
        | Read::Deployment => None,
    }
}
/// Stores the whole source and registers its bounded supplied parts only if accepted.
/// The caller marks its accepted body change; a refused body leaves prior facts intact.
#[must_use]
pub(crate) fn put(
    domain: &mut Ui,
    _env: &Env<crate::Config>,
    key: ObjectKey,
    body: Body,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
    retain: bool,
) -> bool {
    let parts = parts(&body, lent.domain_limits().parts);
    if parts.full {
        crate::state::page::routing::refuse(domain, contract::Limit::Objects);
    }
    let project = match &body {
        Body::Task(task) => Some(task.project),
        Body::Escalation(value) => Some(value.task.project),
        Body::Proposal(proposal) => Some(proposal.project),
        Body::PersonTask(person) => person.project(),
        Body::Note { project, .. } => Some(*project),
        Body::Chip(chip) => {
            let contract::ChipPhaseFacts { details, .. } = chip.details.as_ref();
            Some(details.project)
        }
        Body::Ended(_) | Body::Question(_) | Body::Result(_) => object_project(domain, key, lent),
    };
    if (!retain && domain.objects.find(key).is_some()) || (retain && domain.objects.put(key, body)) {
        for part in parts.values.into_boxed() {
            if let Some(project) = project {
                let scope = owner_scope(&part, contract::PartScope::Project(project));
                out.push(Request::ShowPart { part, scope });
            } else if let Some(scope) = scope(domain, &part, lent) {
                out.push(Request::ShowPart { part, scope });
            }
        }

        true
    } else {
        false
    }
}
struct Parts {
    values: List<ApplicationPart>,
    full: bool,
}
fn parts(body: &Body, limit: u32) -> Parts {
    let mut parts = Parts { values: List::with_capacity(limit), full: false };
    match body {
        Body::Chip(chip) => chip_parts(&mut parts, chip),
        Body::Task(task) => {
            chip_parts(&mut parts, &task.chip);
            if let Some(context) = &task.context {
                append(&mut parts, context.parts.as_slice());
            }
        }
        Body::Escalation(value) => {
            // Native accounts already belong to the shared task parts list.
            if owner_hold(&value.task.chip).is_none() {
                chip_parts(&mut parts, &value.task.chip);
            }
            append(
                &mut parts,
                crate::state::page::task::parts(
                    &value.task,
                    crate::state::address::TaskSection::Overview,
                    limit.saturating_add(1),
                )
                .as_slice(),
            );
        }
        Body::Proposal(proposal) => match &proposal.action {
            contract::ProposalAction::Effect { part, .. } | contract::ProposalAction::ProcedureEffect { part, .. } => {
                if let Some(part) = owner_effect(part) {
                    push(&mut parts, part);
                }
            }
            contract::ProposalAction::Settings(_)
            | contract::ProposalAction::Batch(_)
            | contract::ProposalAction::Amend { .. }
            | contract::ProposalAction::Widen { .. }
            | contract::ProposalAction::Release { .. } => {}
        },
        Body::Question(question) => chip_parts(&mut parts, &question.task),
        Body::PersonTask(person) => chip_parts(&mut parts, &person.task),
        Body::Result(result) => {
            chip_parts(&mut parts, &result.task);
            match &result.value {
                Some(contract::ResultValue::Connector { part, .. }) => push(&mut parts, part.clone()),
                Some(
                    contract::ResultValue::Report(_)
                    | contract::ResultValue::Verdict { .. }
                    | contract::ResultValue::OwnerVerdict { .. }
                    | contract::ResultValue::Change { .. }
                    | contract::ResultValue::Choice { .. }
                    | contract::ResultValue::Approved { .. }
                    | contract::ResultValue::Failure(_)
                    | contract::ResultValue::Words(_),
                )
                | None => {}
            }
        }
        Body::Ended(_) | Body::Note { .. } => {}
    }
    parts
}
fn chip_parts(parts: &mut Parts, chip: &contract::Chip) {
    {
        let contract::ChipPhaseFacts { .. } = chip.details.as_ref();

        if let Some(part) = owner_hold(chip) {
            push(parts, part);
        }
    }
}
// The account is an explicit owner reference, independent of the hold's origin
// (client/domain/contract.md, 5.1; objects.md, 3).
pub(crate) fn owner_hold(chip: &contract::Chip) -> Option<ApplicationPart> {
    let details = {
        let contract::ChipPhaseFacts { details, .. } = chip.details.as_ref();
        details
    };
    let account = details.hold.as_ref()?.account.as_ref()?;
    match account.kind {
        contract::PartKind::Hold => match &account.key {
            contract::PartKey::Number(task) => {
                Some(ApplicationPart::Hold { connector: account.connector, task: *task })
            }
            contract::PartKey::Path(_) => None,
        },
        contract::PartKind::Resource
        | contract::PartKind::Pool
        | contract::PartKind::Result
        | contract::PartKind::Procedure
        | contract::PartKind::Effect
        | contract::PartKind::Outbox => None,
    }
}
// The connector's effect number is independent of the jig proposal maker
// (client/domain/root.md, 6; objects.md, 4).
/// Preserve the raw effect subject namespace for a supplied owner part.
/// Maker-keyed effects remain distinct; other owner part shapes do not
/// acquire a guessed subject.
#[must_use]
pub fn owner_effect(part: &contract::Part) -> Option<ApplicationPart> {
    match part.kind {
        contract::PartKind::Effect => match &part.key {
            contract::PartKey::Number(proposal) => {
                Some(ApplicationPart::OwnerEffect { connector: part.connector, proposal: *proposal })
            }
            contract::PartKey::Path(_) => None,
        },
        contract::PartKind::Resource
        | contract::PartKind::Pool
        | contract::PartKind::Result
        | contract::PartKind::Procedure
        | contract::PartKind::Outbox
        | contract::PartKind::Hold => None,
    }
}

fn push(parts: &mut Parts, part: ApplicationPart) {
    if parts.values.push(part).is_err() {
        parts.full = true;
    }
}
fn append(parts: &mut Parts, values: &[ApplicationPart]) {
    for part in values {
        push(parts, part.clone());
    }
}

fn object_project(domain: &Ui, key: ObjectKey, lent: &jig_client::Client) -> Option<u32> {
    domain.borrow(lent).objects().get(domain.objects.find(key)?)?.project
}
/// An outbox reference names its owner; other parts retain their enclosing document scope.
pub(crate) fn owner_scope(part: &ApplicationPart, enclosing: contract::PartScope) -> contract::PartScope {
    match part {
        ApplicationPart::Outbox { scope, .. } => *scope,
        ApplicationPart::Connector { .. }
        | ApplicationPart::Resource { .. }
        | ApplicationPart::Kind { .. }
        | ApplicationPart::Panel { .. }
        | ApplicationPart::Result { .. }
        | ApplicationPart::Effect { .. }
        | ApplicationPart::OwnerEffect { .. }
        | ApplicationPart::Hold { .. }
        | ApplicationPart::Project { .. }
        | ApplicationPart::Deployment { .. } => enclosing,
    }
}
#[expect(clippy::manual_map, reason = "step code matches optional values explicitly")]
pub(crate) fn scope(domain: &Ui, part: &ApplicationPart, lent: &jig_client::Client) -> Option<contract::PartScope> {
    match part {
        ApplicationPart::Outbox { scope, .. } => Some(*scope),
        ApplicationPart::Project { project, .. } => Some(contract::PartScope::Project(*project)),
        ApplicationPart::Deployment { .. }
        | ApplicationPart::Connector { .. }
        | ApplicationPart::Resource { .. }
        | ApplicationPart::Kind { .. } => Some(contract::PartScope::Deployment),
        ApplicationPart::Panel { task, .. }
        | ApplicationPart::Result { task, .. }
        | ApplicationPart::Hold { task, .. } => {
            let project = match object_project(domain, ObjectKey::Task(*task), lent) {
                Some(project) => Some(project),
                None => current_project(domain, lent),
            };
            match project {
                Some(project) => Some(contract::PartScope::Project(project)),
                None => None,
            }
        }
        ApplicationPart::Effect { proposal, .. } => {
            let project = match object_project(domain, ObjectKey::Proposal(*proposal), lent) {
                Some(project) => Some(project),
                None => current_project(domain, lent),
            };
            match project {
                Some(project) => Some(contract::PartScope::Project(project)),
                None => None,
            }
        }
        ApplicationPart::OwnerEffect { .. } => match current_project(domain, lent) {
            Some(project) => Some(contract::PartScope::Project(project)),
            None => None,
        },
    }
}
fn current_project(domain: &Ui, lent: &jig_client::Client) -> Option<u32> {
    if let Some(project) = domain.shown_project() {
        return Some(project);
    }
    if let Some(task) = domain.shown_task() {
        return object_project(domain, ObjectKey::Task(task), lent);
    }
    domain.shown_inbox().or(domain.shown_conversations()).flatten()
}
