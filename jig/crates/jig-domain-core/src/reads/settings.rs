//! Dense revision reads (domain/engine/settings.md, 7; client/domain/contract/README.md, section 4).
use crate::{
    Core, CoreRecord, Limits, Record, Request,
    reads::{
        Answer, Range, RevisionAnswer, RevisionEdit, RevisionItemHead, RevisionRow, RevisionsPage, Row, Scope,
        SettingDocument, Why, terminal,
    },
    settings,
};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Map, Queue, ReplyTo};

#[must_use]
pub fn oldest(core: &Core, limits: &Limits) -> u32 {
    core.current.oldest().max(core.current.newest().saturating_sub(limits.settings.history)).max(1)
}
/// Whole charter values are available while their revision is held.
#[must_use]
pub fn may_read(core: &Core, revision: u32) -> bool {
    core.held(revision)
}

/// Run reads use the same documents and load slots as party reads (domain/engine/executors.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SettingsRead {
    Inventory,
    Setting { setting: settings::Setting, revision: u32 },
    Revision { number: u32 },
    Revisions { before: Option<u32> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SettingsAnswer {
    Inventory(crate::documents::Settings),
    Setting(SettingDocument),
    Revision(RevisionAnswer),
    Revisions(RevisionsPage),
    Unknown,
}

/// One proposal of settings, as a run reads it (domain/engine/executors.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
    target_pointer_width = "64",
    expect(clippy::large_enum_variant, reason = "the proposal read contract keeps both documents inline")
)]
pub enum ProposalRead {
    /// Waiting, with its review, as a party's live proposal read gives it, with no acts.
    Waiting(crate::documents::Proposal),
    /// Decided, withdrawn or replaced, as its history row keeps it.
    Decided(crate::reads::Ended),
    /// None, not settings, or a task's of another project.
    Unknown,
}

pub(crate) fn run_proposal(
    core: &mut Core,
    env: &Env<Limits>,
    to: ReplyTo,
    key: crate::CallKey,
    proposer: people::Proposer,
    number: u64,
) -> Queue<Request> {
    if let Some(part) = core.authorize_tool(key, crate::ToolKind::Settings) {
        return refused(to, key, part);
    }
    if !proposal_scope(core, key.task, proposer) {
        return proposal_answer(to, key, Answer::None(Why::Unknown));
    }
    match crate::documents::run_proposal(core, &env.limits, proposer, number, env.wall) {
        Ok(value) => {
            return proposal_answer(
                to,
                key,
                Answer::Document(Box::new(crate::documents::Document::Proposal(Box::new(value)))),
            );
        }
        Err(crate::documents::Missing::NotAllowed) => return proposal_answer(to, key, Answer::None(Why::Unknown)),
        Err(crate::documents::Missing::TooLarge) => return refused(to, key, crate::CallPart::Unavailable),
        Err(crate::documents::Missing::Ended | crate::documents::Missing::Unknown) => {}
    }
    if core.reads.is_full() || core.reads.len() >= env.limits.load_slots {
        return refused(to, key, crate::CallPart::Unavailable);
    }
    let range = Range {
        // The range's party selects a physical owner key; run visibility is checked separately.
        party: match proposer {
            people::Proposer::Party(party) => party,
            people::Proposer::Task(_) => 0,
        },
        scope: Scope::Proposal { proposer, proposal: number },
        after: None,
        backwards: false,
        rows: env.limits.documents.rows,
        bytes: env.limits.load_bytes,
    };
    let pending = super::Pending {
        destination: super::Destination::Proposal { to, key },
        sign_in: 0,
        party: 0,
        range: range.clone(),
        tasks: None,
    };
    let load = core.reads.insert(pending).expect("proposal read room checked").token();
    let mut out = Queue::uncapped();
    out.push(Request::Held(Box::new(crate::Held::ReadLoad { load, range: Box::new(range) })));
    out
}

fn proposal_scope(core: &Core, caller: u64, proposer: people::Proposer) -> bool {
    let Some(reader) = core.tasks.task(caller) else { return false };
    match proposer {
        people::Proposer::Task(task) => match core.tasks.task(task) {
            Some(owner) => owner.project == reader.project,
            // A retained owner is checked with the history row when it loads.
            None => true,
        },
        people::Proposer::Party(_) => true,
    }
}

fn proposal_answer(to: ReplyTo, key: crate::CallKey, value: Answer) -> Queue<Request> {
    let answer = match value {
        Answer::Document(document) => {
            let crate::documents::Document::Proposal(value) = *document else { unreachable!("proposal document") };
            ProposalRead::Waiting(*value)
        }
        Answer::Ended(value) => ProposalRead::Decided(*value),
        Answer::None(Why::Busy) => return refused(to, key, crate::CallPart::Unavailable),
        Answer::None(_) => ProposalRead::Unknown,
        Answer::Setting(_)
        | Answer::Revision(_)
        | Answer::Revisions(_)
        | Answer::TaskText(_)
        | Answer::Tasks(_)
        | Answer::Page(_) => unreachable!("proposal read answer"),
    };
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(crate::Now::ProposalRead { to, key, answer: Box::new(answer) })));
    out
}

pub(super) fn proposal_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    name: skein_lib::Id<super::Pending>,
    result: super::Loaded,
) -> Queue<Request> {
    let pending = core.reads.get_mut(name).expect("pending proposal load");
    let super::Destination::Proposal { to, key } =
        core::mem::replace(&mut pending.destination, super::Destination::Done)
    else {
        unreachable!("proposal destination")
    };
    let range = pending.range.clone();
    core.reads.retire(name);
    if !core.current_proof(key.task, key.attempt) {
        return Queue::uncapped();
    }
    if let Some(part) = core.authorize_tool(key, crate::ToolKind::Settings) {
        return refused(to, key, part);
    }
    let Scope::Proposal { proposer, .. } = range.scope else { unreachable!("proposal scope") };
    if !proposal_scope(core, key.task, proposer) {
        return proposal_answer(to, key, Answer::None(Why::Unknown));
    }
    let project = core.tasks.task(key.task).expect("current reading task").project;
    let value = match result {
        super::Loaded::Busy | super::Loaded::TooLinked { .. } => Answer::None(Why::Busy),
        super::Loaded::Gone | super::Loaded::TurnProblem { .. } => Answer::None(Why::Unknown),
        super::Loaded::Page { rows, next } => {
            super::rows::proposal_from_store(core, &env.limits, &range, rows, next, env.wall, project)
        }
    };
    proposal_answer(to, key, value)
}

struct Recipient {
    destination: super::Destination,
    sign_in: u64,
    party: u64,
}

pub(crate) fn run(
    core: &mut Core,
    env: &Env<Limits>,
    to: ReplyTo,
    key: crate::CallKey,
    read: SettingsRead,
) -> Queue<Request> {
    if let Some(part) = core.authorize_tool(key, crate::ToolKind::Settings) {
        return refused(to, key, part);
    }
    let recipient = Recipient { destination: super::Destination::Settings { to, key }, sign_in: 0, party: 0 };
    match read {
        SettingsRead::Inventory => answer(
            recipient.destination,
            Answer::Document(Box::new(crate::documents::Document::Settings(Box::new(
                crate::documents::settings_inventory(core),
            )))),
        ),
        SettingsRead::Setting { setting, revision } => {
            answer(recipient.destination, setting_value(core, setting, revision))
        }
        SettingsRead::Revision { number } => revision_read(core, env, recipient, number),
        SettingsRead::Revisions { before } => revisions_read(core, env, recipient, before),
    }
}

fn refused(to: ReplyTo, key: crate::CallKey, part: crate::CallPart) -> Queue<Request> {
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(crate::Now::EffectAnswer { to, key, part })));
    out
}

pub(super) fn answer(destination: super::Destination, value: Answer) -> Queue<Request> {
    let super::Destination::Settings { to, key } = destination else {
        let reply = match destination {
            super::Destination::Read(reply)
            | super::Destination::RevisionMetadata { reply, .. }
            | super::Destination::ProjectProposal { reply, .. } => reply,
            super::Destination::Watch(_)
            | super::Destination::Erase
            | super::Destination::Restore
            | super::Destination::Proposal { .. }
            | super::Destination::Settings { .. }
            | super::Destination::Person { .. }
            | super::Destination::RunTask { .. }
            | super::Destination::Done => unreachable!("settings read destination"),
        };
        return terminal(reply, value);
    };
    let value = match value {
        Answer::Document(document) => {
            let crate::documents::Document::Settings(value) = *document else { unreachable!("settings inventory") };
            SettingsAnswer::Inventory(*value)
        }
        Answer::Setting(value) => SettingsAnswer::Setting(*value),
        Answer::Revision(value) => SettingsAnswer::Revision(*value),
        Answer::Revisions(value) => SettingsAnswer::Revisions(*value),
        Answer::None(Why::Busy) => return refused(to, key, crate::CallPart::Unavailable),
        Answer::None(_) => SettingsAnswer::Unknown,
        Answer::TaskText(_) | Answer::Tasks(_) | Answer::Ended(_) | Answer::Page(_) => {
            unreachable!("settings reads answer settings documents")
        }
    };
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(crate::Now::SettingsAnswer { to, key, answer: Box::new(value) })));
    out
}

pub(super) fn run_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    name: skein_lib::Id<super::Pending>,
    result: super::Loaded,
) -> Queue<Request> {
    let pending = core.reads.get_mut(name).expect("pending settings load");
    let destination = core::mem::replace(&mut pending.destination, super::Destination::Done);
    let range = pending.range.clone();
    core.reads.retire(name);
    let value = match result {
        super::Loaded::TooLinked { .. } | super::Loaded::Busy => Answer::None(Why::Busy),
        super::Loaded::Gone => Answer::None(Why::Gone),
        super::Loaded::Page { rows, next } => super::rows::from_store(core, &env.limits, &range, rows, next, env.wall),
        super::Loaded::TurnProblem { .. } => Answer::None(Why::Unknown),
    };
    answer(destination, value)
}

pub(super) fn setting(
    core: &Core,
    reply: ReplyTo,
    party: u64,
    setting: settings::Setting,
    revision: u32,
) -> Queue<Request> {
    if core.people.role(party, people::DEPLOYMENT).is_none() {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    terminal(reply, setting_value(core, setting, revision))
}
fn setting_value(core: &Core, setting: settings::Setting, revision: u32) -> Answer {
    if !may_read(core, revision) {
        return Answer::None(Why::Unknown);
    }
    match setting {
        settings::Setting::ProjectPolicy { project } => return project_policy(core, project, revision),
        settings::Setting::Application { .. } => {
            if revision != core.current.newest() {
                return Answer::None(Why::Unknown);
            }
            return match core.current.applications.get(&setting) {
                Some(state) => Answer::Setting(Box::new(SettingDocument {
                    overlay: None,
                    setting,
                    revision,
                    value: None,
                    application: Some(state.clone()),
                    proposed: None,
                })),
                None => Answer::None(Why::Unknown),
            };
        }
        settings::Setting::Charter(_)
        | settings::Setting::Template(_)
        | settings::Setting::Policy(_)
        | settings::Setting::Model(_)
        | settings::Setting::Rules => {}
    }
    if let settings::Setting::Charter(number) = setting {
        return match core.charter_at(number, revision) {
            Some(charter) => Answer::Setting(Box::new(SettingDocument {
                overlay: match core.overlay_at(number, revision) {
                    Some(overlay) => Some(Box::new(overlay.clone())),
                    None => None,
                },
                setting,
                revision,
                value: Some(settings::Value::Charter(Box::new(charter.clone()))),
                application: None,
                proposed: None,
            })),
            None => Answer::None(Why::Unknown),
        };
    }
    if revision != core.current.newest() {
        return Answer::None(Why::Unknown);
    }
    match core.current.value(setting) {
        Some(value) => Answer::Setting(Box::new(SettingDocument {
            overlay: None,
            setting,
            revision,
            value: Some(value.clone()),
            application: None,
            proposed: None,
        })),
        None => Answer::None(Why::Unknown),
    }
}

// domain/engine/settings.md, 6: current authority is read whole, never as a retained revision.
fn project_policy(core: &Core, project: u32, revision: u32) -> Answer {
    if revision != core.current.newest() {
        return Answer::None(Why::Unknown);
    }
    match core.configured_policy(project) {
        Some(value) => Answer::Setting(Box::new(SettingDocument {
            overlay: None,
            setting: settings::Setting::ProjectPolicy { project },
            revision,
            value: Some(settings::Value::Policy(Box::new(value))),
            application: None,
            proposed: None,
        })),
        None => Answer::None(Why::Unknown),
    }
}

/// The same pending row and audience as the proposal review, without its value cut.
pub(super) fn proposed_setting(
    core: &Core,
    reply: ReplyTo,
    party: u64,
    setting: settings::Setting,
    proposal: (people::Proposer, u64),
) -> Queue<Request> {
    let (proposer, number) = proposal;
    if !pending_settings_review(core, party, proposer, number) {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    let Ok(number) = u32::try_from(number) else {
        return terminal(reply, Answer::None(Why::Unknown));
    };
    let Some(proposal) = core.settings_proposals.get(&(proposer, number)) else {
        return terminal(reply, Answer::None(Why::Unknown));
    };
    for item in &proposal.applications {
        let named = settings::Setting::Application { kind: item.kind, scope: item.scope, number: item.number };
        if named != setting {
            continue;
        }
        return terminal(reply, application_proposal(core, setting, &item.edit));
    }
    for item in &proposal.items {
        if item.setting == setting {
            let value = match &item.edit {
                settings::Edit::Set(value) => Some(value),
                settings::Edit::Removed => None,
                settings::Edit::Reset => core.current.default_value(setting),
            };
            return terminal(
                reply,
                match value {
                    Some(value) => Answer::Setting(Box::new(SettingDocument {
                        overlay: None,
                        setting,
                        revision: core.current.newest(),
                        value: Some(value.clone()),
                        application: None,
                        proposed: None,
                    })),
                    None => Answer::None(Why::Unknown),
                },
            );
        }
    }
    terminal(reply, Answer::None(Why::Unknown))
}

fn pending_settings_review(core: &Core, party: u64, proposer: people::Proposer, number: u64) -> bool {
    match proposer {
        people::Proposer::Party(proposer) => {
            let Some(row) = core.tasks.person_proposal(proposer, number) else { return false };
            if party != proposer && core.people.role(party, row.project).is_none() {
                return false;
            }
            match row.state {
                tasks::PersonProposalState::Pending { .. } => true,
                tasks::PersonProposalState::SettingsAccepted { .. }
                | tasks::PersonProposalState::Accepted { .. }
                | tasks::PersonProposalState::Rejected { .. }
                | tasks::PersonProposalState::Withdrawn { .. } => false,
            }
        }
        people::Proposer::Task(task) => {
            let Some(row) = core.tasks.task(task) else { return false };
            let Some(proposal) = &row.proposal else { return false };
            if proposal.number != number || core.people.role(party, people::DEPLOYMENT) != Some(people::Role::Owner) {
                return false;
            }
            let pending = match proposal.state {
                tasks::ProposalState::Pending { .. } => true,
                tasks::ProposalState::Accepted { .. }
                | tasks::ProposalState::Rejected { .. }
                | tasks::ProposalState::Withdrawn => false,
            };
            let settings_action = match proposal.action {
                tasks::ProposalAction::Settings { .. } => true,
                tasks::ProposalAction::Batch(..)
                | tasks::ProposalAction::Effect { .. }
                | tasks::ProposalAction::ProcedureEffect { .. }
                | tasks::ProposalAction::Amend { .. }
                | tasks::ProposalAction::Widen { .. }
                | tasks::ProposalAction::Release { .. }
                | tasks::ProposalAction::ReleasePlaced { .. } => false,
            };
            pending && settings_action
        }
    }
}

fn application_proposal(core: &Core, setting: settings::Setting, edit: &settings::application::SizedEdit) -> Answer {
    Answer::Setting(Box::new(SettingDocument {
        overlay: None,
        setting,
        revision: core.current.newest(),
        value: None,
        application: core.current.applications.get(&setting).cloned(),
        proposed: Some(edit.clone()),
    }))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    Unknown,
    Forgotten { oldest: u32 },
    Load { first: u32, last: u32 },
}
#[must_use]
pub fn select_revision(newest: u32, oldest: u32, number: u32) -> Selection {
    if number == 0 || number > newest {
        Selection::Unknown
    } else if number < oldest {
        Selection::Forgotten { oldest }
    } else {
        Selection::Load { first: number, last: number }
    }
}
#[must_use]
pub fn page_range(newest: u32, oldest: u32, before: Option<u32>, rows: u32) -> Option<(u32, u32)> {
    let last = match before {
        Some(before) => newest.min(before.saturating_sub(1)),
        None => newest,
    };
    if last < oldest || rows == 0 {
        return None;
    }
    Some((oldest.max(last.saturating_sub(rows.saturating_sub(1))), last))
}

pub(super) fn revision(
    core: &mut Core,
    env: &Env<Limits>,
    reply: ReplyTo,
    sign_in: u64,
    party: u64,
    number: u32,
) -> Queue<Request> {
    if core.people.role(party, people::DEPLOYMENT).is_none() {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    revision_read(core, env, Recipient { destination: super::Destination::Read(reply), sign_in, party }, number)
}

/// A project's member may load its application revision metadata for the application's own reads.
/// Core values and other scopes' items stay private (jig/docs/design/domain/engine/settings.md, 5 and 7).
pub fn revision_metadata(
    core: &mut Core,
    env: &Env<Limits>,
    reply: ReplyTo,
    sign_in: u64,
    project: u32,
    number: u32,
) -> Queue<Request> {
    let Some(party) = core.people.person(sign_in, env.now, env.wall) else {
        return terminal(reply, Answer::None(if core.people.ready() { Why::SignedOut } else { Why::Unknown }));
    };
    if !metadata_allowed(core, party, project) {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    revision_read(
        core,
        env,
        Recipient { destination: super::Destination::RevisionMetadata { reply, project }, sign_in, party },
        number,
    )
}

/// A project's member reads a proposal of settings that touches the project, for the application's own reads
/// (jig/docs/design/domain/engine/settings.md, 6). Other scopes' items and core values stay as the run's read gives them.
/// Decided party proposals have no retained projects and are unknown to this read.
pub fn project_proposal(
    core: &mut Core,
    env: &Env<Limits>,
    reply: ReplyTo,
    sign_in: u64,
    project: u32,
    proposer: people::Proposer,
    number: u64,
) -> Queue<Request> {
    let Some(party) = core.people.person(sign_in, env.now, env.wall) else {
        return terminal(reply, Answer::None(if core.people.ready() { Why::SignedOut } else { Why::Unknown }));
    };
    if !metadata_allowed(core, party, project) || !project_proposal_scope(core, project, proposer, number) {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    match crate::documents::run_proposal(core, &env.limits, proposer, number, env.wall) {
        Ok(value) => {
            return terminal(reply, Answer::Document(Box::new(crate::documents::Document::Proposal(Box::new(value)))));
        }
        Err(crate::documents::Missing::NotAllowed) => return terminal(reply, Answer::None(Why::Unknown)),
        Err(crate::documents::Missing::TooLarge) => return terminal(reply, Answer::None(Why::Busy)),
        Err(crate::documents::Missing::Ended | crate::documents::Missing::Unknown) => {}
    }
    let range = Range {
        party: match proposer {
            people::Proposer::Party(party) => party,
            people::Proposer::Task(_) => 0,
        },
        scope: Scope::Proposal { proposer, proposal: number },
        after: None,
        backwards: false,
        rows: env.limits.documents.rows,
        bytes: env.limits.load_bytes,
    };
    super::load_settings(
        core,
        env,
        super::Pending {
            destination: super::Destination::ProjectProposal { reply, project },
            sign_in,
            party,
            range,
            tasks: None,
        },
    )
}

fn project_proposal_scope(core: &Core, project: u32, proposer: people::Proposer, number: u64) -> bool {
    match proposer {
        people::Proposer::Task(task) => match core.tasks.task(task) {
            Some(task) => task.project == project,
            // The linked owner in the history load establishes the project.
            None => true,
        },
        people::Proposer::Party(_) => {
            let Ok(number) = u32::try_from(number) else { return false };
            match core.settings_proposals.get(&(proposer, number)) {
                Some(proposal) => proposal.projects().as_slice().contains(&project),
                None => false,
            }
        }
    }
}

pub(super) fn project_proposal_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    name: skein_lib::Id<super::Pending>,
    result: super::Loaded,
) -> Queue<Request> {
    let pending = core.reads.get(name).expect("pending project proposal load");
    let super::Destination::ProjectProposal { project, .. } = pending.destination else {
        unreachable!("project proposal destination")
    };
    let Scope::Proposal { proposer, proposal } = pending.range.scope else { unreachable!("project proposal scope") };
    let value = if core.people.person(pending.sign_in, env.now, env.wall) != Some(pending.party) {
        Answer::None(Why::SignedOut)
    } else if !metadata_allowed(core, pending.party, project)
        || !project_proposal_scope(core, project, proposer, proposal)
    {
        Answer::None(Why::Unknown)
    } else {
        match result {
            super::Loaded::Busy | super::Loaded::TooLinked { .. } => Answer::None(Why::Busy),
            super::Loaded::Gone | super::Loaded::TurnProblem { .. } => Answer::None(Why::Unknown),
            super::Loaded::Page { rows, next } => {
                super::rows::proposal_from_store(core, &env.limits, &pending.range, rows, next, env.wall, project)
            }
        }
    };
    let reply = core.reads.get_mut(name).expect("pending project proposal load").take_reply();
    core.reads.retire(name);
    terminal(reply, value)
}

fn metadata_allowed(core: &Core, party: u64, project: u32) -> bool {
    project != people::DEPLOYMENT && core.open_project(project) && core.people.role(party, project).is_some()
}

pub(super) fn metadata_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    name: skein_lib::Id<super::Pending>,
    result: super::Loaded,
) -> Queue<Request> {
    let pending = core.reads.get(name).expect("pending revision metadata load");
    let super::Destination::RevisionMetadata { project, .. } = pending.destination else {
        unreachable!("revision metadata destination")
    };
    let value = if core.people.person(pending.sign_in, env.now, env.wall) != Some(pending.party) {
        Answer::None(Why::SignedOut)
    } else if !metadata_allowed(core, pending.party, project) {
        Answer::None(Why::Unknown)
    } else {
        match result {
            super::Loaded::TooLinked { .. } | super::Loaded::Busy => Answer::None(Why::Busy),
            super::Loaded::Gone => Answer::None(Why::Gone),
            super::Loaded::TurnProblem { .. } => Answer::None(Why::Unknown),
            super::Loaded::Page { rows, next } => {
                metadata(from_store(core, &env.limits, &pending.range, rows, next), project)
            }
        }
    };
    let reply = core.reads.get_mut(name).expect("pending revision metadata load").take_reply();
    core.reads.retire(name);
    terminal(reply, value)
}

fn metadata(answer: Answer, project: u32) -> Answer {
    let Answer::Revision(mut answer) = answer else { return answer };
    match answer.as_mut() {
        RevisionAnswer::Revision { revision, .. } => {
            revision.items = Box::new([]);
            let mut applications = List::uncapped();
            for item in &revision.applications {
                if item.scope == project {
                    applications.push(item.clone()).expect("subset of loaded revision items");
                }
            }
            revision.applications = applications.into_boxed();
        }
        RevisionAnswer::Unknown | RevisionAnswer::Forgotten { .. } => {}
    }
    Answer::Revision(answer)
}

fn revision_read(core: &mut Core, env: &Env<Limits>, recipient: Recipient, number: u32) -> Queue<Request> {
    if core.held(number) {
        return load(core, env, recipient, number, number, true);
    }
    match select_revision(core.current.newest(), oldest(core, &env.limits), number) {
        Selection::Unknown => answer(recipient.destination, Answer::Revision(Box::new(RevisionAnswer::Unknown))),
        Selection::Forgotten { oldest } => {
            answer(recipient.destination, Answer::Revision(Box::new(RevisionAnswer::Forgotten { oldest })))
        }
        Selection::Load { first, last } => load(core, env, recipient, first, last, true),
    }
}
pub(super) fn revisions(
    core: &mut Core,
    env: &Env<Limits>,
    reply: ReplyTo,
    sign_in: u64,
    party: u64,
    before: Option<u32>,
) -> Queue<Request> {
    if core.people.role(party, people::DEPLOYMENT).is_none() {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    revisions_read(core, env, Recipient { destination: super::Destination::Read(reply), sign_in, party }, before)
}
fn revisions_read(core: &mut Core, env: &Env<Limits>, recipient: Recipient, before: Option<u32>) -> Queue<Request> {
    let cutoff = oldest(core, &env.limits);
    let last = match before {
        Some(before) => core.current.newest().min(before.saturating_sub(1)),
        None => core.current.newest(),
    };
    let (oldest, before) = if last < cutoff {
        let mut held = None;
        for (&revision, _) in &core.held.heads {
            if revision <= last {
                held = Some(revision);
            }
        }
        match held {
            Some(revision) => (page_oldest(core, cutoff), revision.checked_add(1)),
            None => (cutoff, before),
        }
    } else {
        (cutoff, before)
    };
    match page_range(core.current.newest(), oldest, before, env.limits.documents.rows) {
        Some((first, last)) => load(core, env, recipient, first, last, false),
        None => answer(
            recipient.destination,
            Answer::Revisions(Box::new(RevisionsPage {
                rows: List::with_capacity(0),
                next: None,
                forgotten: oldest > 1,
            })),
        ),
    }
}
fn load(
    core: &mut Core,
    env: &Env<Limits>,
    recipient: Recipient,
    first: u32,
    last: u32,
    whole: bool,
) -> Queue<Request> {
    let rows = last.checked_sub(first).expect("ordered revision range").checked_add(1).expect("nonzero revision range");
    let Some(bytes) = load_bytes(&env.limits, rows) else {
        return answer(recipient.destination, Answer::None(Why::Busy));
    };
    let range = Range {
        party: recipient.party,
        scope: Scope::SettingsHistory { first, last, newest: core.current.newest(), whole },
        after: None,
        backwards: false,
        rows,
        bytes,
    };
    super::load_settings(
        core,
        env,
        super::Pending {
            destination: recipient.destination,
            sign_in: recipient.sign_in,
            party: recipient.party,
            range,
            tasks: None,
        },
    )
}
/// A selected revision is decoded whole before its values are admitted separately.
#[must_use]
pub fn load_bytes(limits: &Limits, rows: u32) -> Option<u64> {
    settings::revision_bound(limits)?
        .checked_add(u64::try_from(size_of::<Row>()).ok()?)?
        .checked_add(u64::try_from(size_of::<Record>()).ok()?)?
        .checked_mul(u64::from(rows))
}

/// Validate one decoded history record before it crosses a load boundary.
/// Its set values occupy at most one whole bounded settings set.
#[must_use]
pub fn valid_revision(revision: &settings::Revision, limits: &Limits) -> bool {
    let Some(capacity) = settings::capacity(limits) else { return false };
    let Some(capacity) = capacity.checked_mul(2) else { return false };
    let Some(capacity) = capacity.checked_add(1) else { return false };
    let Ok(count) = u32::try_from(revision.items.len()) else { return false };
    if count > capacity {
        return false;
    }
    let mut seen = Map::with_capacity(count);
    let mut charters = 0_u32;
    let mut templates = 0_u32;
    let mut policies = 0_u32;
    for item in &revision.items {
        if seen.insert(item.setting, ()).ok() != Some(None) {
            return false;
        }
        match &item.edit {
            settings::Edit::Set(value) => {
                if settings::check::shape(item.setting, value, limits).is_err() {
                    return false;
                }
                match value {
                    settings::Value::Charter(value) => {
                        charters = charters.saturating_add(1);
                        if charters > limits.tasks.charters
                            || value.policy.alternatives.len()
                                > skein_lib::length::usize_of(limits.settings.alternatives)
                        {
                            return false;
                        }
                    }
                    settings::Value::Overlay(_) => {
                        charters = charters.saturating_add(1);
                        if charters > limits.tasks.charters {
                            return false;
                        }
                    }
                    settings::Value::Template(_) => {
                        templates = templates.saturating_add(1);
                        if templates > limits.templates {
                            return false;
                        }
                    }
                    settings::Value::Policy(_) => {
                        if settings::is_project_policy(item.setting) {
                            continue;
                        }
                        policies = policies.saturating_add(1);
                        if policies > limits.authority.projects {
                            return false;
                        }
                    }
                    settings::Value::Rules(_) | settings::Value::Model(_) => {}
                    settings::Value::Application(_) => unreachable!("an application's setting: apps-core"),
                }
            }
            settings::Edit::Removed | settings::Edit::Reset => {}
        }
    }
    true
}
#[must_use]
pub fn record_revision(value: &Record) -> Option<&settings::Revision> {
    match value {
        Record::Core(CoreRecord::Settings(settings::Record::Revision { revision, .. })) => Some(revision),
        Record::Core(
            CoreRecord::Settings(
                settings::Record::Application { .. }
                | settings::Record::Default { .. }
                | settings::Record::Changed { .. }
                | settings::Record::Charter { .. }
                | settings::Record::Proposal { .. }
                | settings::Record::Extent { .. },
            )
            | CoreRecord::Deployment(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Call(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Projection(_)
            | CoreRecord::EscalationDecision(_)
            | CoreRecord::ProposalDecision(_),
        )
        | Record::People(_)
        | Record::Tasks(_)
        | Record::Notes(_)
        | Record::Accounts(_) => None,
    }
}

fn page_oldest(core: &Core, cutoff: u32) -> u32 {
    match core.held.heads.first() {
        Some((&revision, _)) => cutoff.min(revision),
        None => cutoff,
    }
}

fn selected(scope: &Scope) -> (u32, u32, u32, bool) {
    match scope {
        Scope::SettingsHistory { first, last, newest, whole } => (*first, *last, *newest, *whole),
        Scope::EndedTasks
        | Scope::TaskRows { .. }
        | Scope::Tasks { .. }
        | Scope::TaskText { .. }
        | Scope::Task { .. }
        | Scope::Proposal { .. }
        | Scope::Escalation { .. }
        | Scope::Question { .. }
        | Scope::PersonTask { .. }
        | Scope::Result { .. }
        | Scope::Reply { .. }
        | Scope::Turn { .. }
        | Scope::EndedDelegates { .. }
        | Scope::History { .. }
        | Scope::Activity { .. }
        | Scope::Inbox { .. }
        | Scope::Notes { .. }
        | Scope::ScopedNotes { .. }
        | Scope::Spend { .. } => unreachable!("settings range dispatched by its scope"),
    }
}

pub(super) fn from_store(
    core: &Core,
    limits: &Limits,
    range: &Range,
    raw: Box<[Row]>,
    next: Option<super::Position>,
) -> Answer {
    let (first, last, newest, whole) = selected(&range.scope);
    if next.is_some() || raw.len() > skein_lib::length::usize_of(range.rows) {
        return Answer::None(Why::Busy);
    }
    let oldest = oldest(core, limits);
    if whole && first < oldest && !core.held(first) {
        return Answer::Revision(Box::new(RevisionAnswer::Forgotten { oldest }));
    }
    let mut found = Map::with_capacity(range.rows);
    for row in raw {
        if row.owner.is_some()
            || !row.ancestors.is_empty()
            || row.related.is_some()
            || !row.linked.is_empty()
            || row.inbox.is_some()
            || row.unjoined.is_some()
            || row.ended_read.is_some()
        {
            return Answer::None(Why::Unknown);
        }
        if !super::admits(range.party, &range.scope, &row.value.key()) {
            return Answer::None(Why::Unknown);
        }
        let revision = match row.value {
            Record::Core(CoreRecord::Settings(settings::Record::Revision { revision, .. })) => revision,
            Record::Core(
                CoreRecord::Settings(
                    settings::Record::Application { .. }
                    | settings::Record::Default { .. }
                    | settings::Record::Changed { .. }
                    | settings::Record::Charter { .. }
                    | settings::Record::Proposal { .. }
                    | settings::Record::Extent { .. },
                )
                | CoreRecord::Deployment(_)
                | CoreRecord::RunProof(_)
                | CoreRecord::Call(_)
                | CoreRecord::Terminal(_)
                | CoreRecord::Turn(_)
                | CoreRecord::Projection(_)
                | CoreRecord::EscalationDecision(_)
                | CoreRecord::ProposalDecision(_),
            )
            | Record::People(_)
            | Record::Tasks(_)
            | Record::Notes(_)
            | Record::Accounts(_) => return Answer::None(Why::Unknown),
        };
        if !valid_revision(&revision, limits) {
            return Answer::None(Why::Busy);
        }
        let previous = found.insert(revision.head.number, revision).expect("bounded loaded revisions");
        if previous.is_some() {
            return Answer::None(Why::Unknown);
        }
    }
    if whole {
        return Answer::Revision(Box::new(match found.remove(&first) {
            Some(mut revision) => {
                if first == newest && newest == core.current.newest() {
                    revision.head = core.current.newest_head().clone();
                }
                RevisionAnswer::Revision { revision, value_bytes: limits.load_bytes }
            }
            None => RevisionAnswer::Forgotten { oldest },
        }));
    }
    let mut rows = List::with_capacity(range.rows);
    for number in (first..=last).rev() {
        if number < oldest && !core.held(number) {
            continue;
        }
        let Some(revision) = found.remove(&number) else { continue };
        let total = u32::try_from(
            revision.items.len().checked_add(revision.applications.len()).expect("bounded revision items"),
        )
        .expect("checked item count");
        let items = item_preview(&revision, limits.documents.lines);
        rows.push(RevisionRow { head: revision.head, more: total.saturating_sub(items.len()), items })
            .expect("bounded revision page");
    }
    let next = if first > page_oldest(core, oldest) { Some(first) } else { None };
    Answer::Revisions(Box::new(RevisionsPage { rows, next, forgotten: next.is_none() && oldest > 1 }))
}

fn item_preview(revision: &settings::Revision, lines: u32) -> List<RevisionItemHead> {
    let count = u32::try_from(revision.items.len().checked_add(revision.applications.len()).expect("bounded items"))
        .expect("bounded item count");
    let mut items = List::with_capacity(count.min(lines));
    for item in revision.items.iter().take(skein_lib::length::usize_of(lines)) {
        let edit = match item.edit {
            settings::Edit::Set(_) => RevisionEdit::Set,
            settings::Edit::Removed => RevisionEdit::Removed,
            settings::Edit::Reset => RevisionEdit::Reset,
        };
        items.push(RevisionItemHead { setting: item.setting, edit }).expect("bounded item preview");
    }
    for item in &revision.applications {
        if items.len() == lines {
            break;
        }
        let setting = settings::Setting::Application { kind: item.kind, scope: item.scope, number: item.number };
        let edit = match item.edit {
            settings::application::SizedEdit::Set { .. } => RevisionEdit::Set,
            settings::application::SizedEdit::Remove => RevisionEdit::Removed,
            settings::application::SizedEdit::Reset => RevisionEdit::Reset,
        };
        items.push(RevisionItemHead { setting, edit }).expect("bounded application preview");
    }
    items
}

/// Restore pages use the same range and byte admission as history reads (domain/engine/settings.md, 10).
pub(crate) fn restore_load(
    core: &mut Core,
    env: &Env<Limits>,
    first: u32,
    last: u32,
    party: u64,
) -> Option<Queue<Request>> {
    let rows = last.checked_sub(first)?.checked_add(1)?;
    if core.reads.is_full() || core.reads.len() >= env.limits.load_slots || load_bytes(&env.limits, rows).is_none() {
        return None;
    }
    Some(load(core, env, Recipient { destination: super::Destination::Restore, sign_in: 0, party }, first, last, false))
}
pub(super) fn restore_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<crate::routing::Work>,
    name: skein_lib::Id<super::Pending>,
    result: super::Loaded,
) -> Queue<Request> {
    let pending = core.reads.get_mut(name).expect("pending restore page");
    pending.destination = super::Destination::Done;
    let range = pending.range.clone();
    core.reads.retire(name);
    core.reads.reclaim();
    crate::routing::settings::restore::loaded(core, env, work, range, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const PROJECT: u32 = 1;
    const SIGN_IN: u64 = 10;
    const REPLY: ReplyTo = ReplyTo::new(skein_lib::Token::new(90));
    const REVISION: u32 = 2;
    // The application fixture declares an opaque value of eight bytes.
    const APPLICATION_BYTES: u32 = 8;

    fn metadata_core(role: Option<people::Role>) -> (Core, Env<Limits>) {
        let (mut core, limits) = crate::tests::core();
        core.current.head.number = REVISION;
        let identity = core.people.identity(1).unwrap().clone();
        core.people = people::Domain::new(&limits.people, Box::new([]), 1, Box::new([]));
        core.people.start_restore(skein_lib::Wall::EPOCH);
        let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
        let people_env = env.with(limits.people);
        for record in [
            people::Stored::Person { number: 1, identity },
            people::Stored::SignIn { number: SIGN_IN, person: 1, expires: skein_lib::Wall::from_nanos(100) },
            people::Stored::Roles {
                project: PROJECT,
                holdings: match role {
                    Some(role) => Box::new([people::Holding { person: 1, role }]),
                    None => Box::new([]),
                },
            },
        ] {
            people::step(&mut core.people, &people_env, people::Event::Restore { record }, &mut Queue::uncapped());
        }
        assert!(core.people.start_link());
        core.people.start_sweep();
        assert!(core.people.start_complete(&people_env));
        (core, env)
    }

    fn metadata_load(core: &mut Core, env: &Env<Limits>) -> skein_lib::Token {
        let mut out = revision_metadata(core, env, REPLY, SIGN_IN, PROJECT, REVISION);
        let Some(Request::Held(held)) = out.pop() else { panic!("metadata load") };
        let crate::Held::ReadLoad { load, .. } = *held else { panic!("metadata load") };
        assert!(out.is_empty());
        load
    }

    fn read_answer(mut out: Queue<Request>) -> Answer {
        let Some(Request::Held(held)) = out.pop() else { panic!("metadata answer") };
        let crate::Held::Read { reply, answer } = *held else { panic!("metadata answer") };
        assert_eq!(reply, REPLY);
        assert!(out.is_empty());
        answer
    }

    fn application_item(scope: u32) -> settings::application::RevisionRecorded {
        settings::application::RevisionRecorded {
            kind: 7,
            scope,
            number: 1,
            edit: settings::application::SizedEdit::Set { size: APPLICATION_BYTES },
            replaced: settings::application::Replaced::NotOffered,
        }
    }

    #[test]
    fn project_revision_metadata_keeps_only_its_application_items() {
        let (mut core, env) = metadata_core(Some(people::Role::Member));
        let load = metadata_load(&mut core, &env);
        let revision = settings::Revision {
            head: core.current.head.clone(),
            items: Box::new([settings::RevisionItem {
                setting: settings::Setting::Rules,
                edit: settings::Edit::Reset,
                replaced: settings::Replaced::NotOffered,
            }]),
            applications: Box::new([application_item(0), application_item(PROJECT), application_item(PROJECT + 1)]),
        };
        let rows = Box::new([crate::tests::stored_row(
            Record::Core(CoreRecord::Settings(settings::Record::Revision { history: false, revision })),
            None,
        )]);
        let out = crate::reads::loaded(
            &mut core,
            &env,
            &mut Queue::uncapped(),
            load,
            crate::reads::Loaded::Page { rows, next: None },
        );
        let Answer::Revision(answer) = read_answer(out) else { panic!("revision metadata") };
        let RevisionAnswer::Revision { revision, .. } = *answer else { panic!("revision metadata") };
        assert!(revision.items.is_empty(), "core values stay private");
        assert_eq!(revision.applications.as_ref(), &[application_item(PROJECT)]);
    }

    #[test]
    fn project_revision_metadata_rechecks_membership_and_sign_in() {
        for (role, wall, why) in [
            (None, skein_lib::Wall::EPOCH, Why::Unknown),
            (Some(people::Role::Member), skein_lib::Wall::from_nanos(100), Why::SignedOut),
        ] {
            let (mut core, mut env) = metadata_core(Some(people::Role::Member));
            let load = metadata_load(&mut core, &env);
            let (replacement, _) = metadata_core(role);
            core.people = replacement.people;
            env.wall = wall;
            let out = crate::reads::loaded(&mut core, &env, &mut Queue::uncapped(), load, crate::reads::Loaded::Busy);
            assert_eq!(read_answer(out), Answer::None(why));
        }
    }

    #[test]
    fn project_revision_metadata_admits_only_a_signed_in_projects_member() {
        for (role, sign_in, project, why) in [
            (None, SIGN_IN, PROJECT, Why::Unknown),
            (Some(people::Role::Member), SIGN_IN + 1, PROJECT, Why::SignedOut),
            (Some(people::Role::Member), SIGN_IN, PROJECT + 1, Why::Unknown),
            (Some(people::Role::Member), SIGN_IN, people::DEPLOYMENT, Why::Unknown),
        ] {
            let (mut core, env) = metadata_core(role);
            let out = revision_metadata(&mut core, &env, REPLY, sign_in, project, REVISION);
            assert_eq!(read_answer(out), Answer::None(why), "project {project}, sign in {sign_in}");
        }
    }

    const PROPOSER: u64 = 1;
    const PROPOSAL: u64 = 7;

    fn proposal_core(role: Option<people::Role>) -> (Core, Env<Limits>) {
        let (mut core, env) = metadata_core(Some(people::Role::Owner));
        crate::tests::make_task(&mut core, &env.limits, PROPOSER);
        tasks::step(
            &mut core.tasks,
            &env.with(env.limits.tasks),
            tasks::Event::Propose {
                reply_to: ReplyTo::new(skein_lib::Token::new(91)),
                proposal: tasks::Proposal {
                    number: PROPOSAL,
                    proposer: PROPOSER,
                    project: people::DEPLOYMENT,
                    action: tasks::ProposalAction::Settings {
                        projects: Box::new([PROJECT]),
                        against: REVISION,
                        items: Box::new([]),
                    },
                    reason: b"Review these settings".as_slice().into(),
                    as_holder: false,
                    state: tasks::ProposalState::Pending {
                        holder: tasks::ProposalHolder::Policy {
                            project: people::DEPLOYMENT,
                            kind: tasks::ProposalKind::Settings,
                        },
                        since: env.wall,
                    },
                },
            },
            &mut Queue::uncapped(),
        );
        core.settings_proposals
            .insert(
                (people::Proposer::Task(PROPOSER), u32::try_from(PROPOSAL).unwrap()),
                settings::Proposal {
                    key: Some(9),
                    against: REVISION,
                    items: Box::new([settings::Item {
                        setting: settings::Setting::Charter(1),
                        edit: settings::Edit::Reset,
                    }]),
                    applications: Box::new([]),
                },
            )
            .unwrap();
        core.people = metadata_core(role).0.people;
        (core, env)
    }

    #[test]
    fn project_proposal_members_read_the_same_waiting_review_as_owners() {
        let (mut owner, env) = proposal_core(Some(people::Role::Owner));
        let expected = read_answer(project_proposal(
            &mut owner,
            &env,
            REPLY,
            SIGN_IN,
            PROJECT,
            people::Proposer::Task(PROPOSER),
            PROPOSAL,
        ));
        let Answer::Document(document) = &expected else { panic!("waiting proposal document") };
        let crate::documents::Document::Proposal(proposal) = document.as_ref() else { panic!("proposal") };
        let crate::documents::ProposalAction::Settings { review: Some(review), .. } = &proposal.action else {
            panic!("settings review")
        };
        assert_eq!(review.key, Some(9));
        let (mut member, env) = proposal_core(Some(people::Role::Member));
        assert_eq!(
            read_answer(project_proposal(
                &mut member,
                &env,
                REPLY,
                SIGN_IN,
                PROJECT,
                people::Proposer::Task(PROPOSER),
                PROPOSAL
            )),
            expected
        );
    }

    fn party_proposal(core: &mut Core, env: &Env<Limits>, scope: u32) {
        core.settings_proposals
            .insert(
                (people::Proposer::Party(PROPOSER), u32::try_from(PROPOSAL).unwrap()),
                settings::Proposal {
                    key: Some(9),
                    against: REVISION,
                    items: Box::new([]),
                    applications: Box::new([settings::application::Recorded {
                        kind: 7,
                        scope,
                        number: 1,
                        edit: settings::application::SizedEdit::Remove,
                    }]),
                },
            )
            .unwrap();
        tasks::step(
            &mut core.tasks,
            &env.with(env.limits.tasks),
            tasks::Event::ProposePerson {
                reply_to: ReplyTo::new(skein_lib::Token::new(92)),
                proposal: tasks::PersonProposal {
                    number: PROPOSAL,
                    proposer: PROPOSER,
                    project: people::DEPLOYMENT,
                    action: tasks::PersonAction::Settings { against: REVISION, items: Box::new([]) },
                    reason: Box::new([]),
                    state: tasks::PersonProposalState::Pending { since: env.wall },
                },
            },
            &mut Queue::uncapped(),
        );
    }

    #[test]
    fn project_proposal_admits_only_its_signed_in_projects_members() {
        for (role, sign_in, project, why) in [
            (None, SIGN_IN, PROJECT, Why::Unknown),
            (Some(people::Role::Member), SIGN_IN + 1, PROJECT, Why::SignedOut),
            (Some(people::Role::Member), SIGN_IN, PROJECT + 1, Why::Unknown),
            (Some(people::Role::Member), SIGN_IN, people::DEPLOYMENT, Why::Unknown),
        ] {
            let (mut core, env) = proposal_core(role);
            assert_eq!(
                read_answer(project_proposal(
                    &mut core,
                    &env,
                    REPLY,
                    sign_in,
                    project,
                    people::Proposer::Task(PROPOSER),
                    PROPOSAL
                )),
                Answer::None(why)
            );
        }
        for (scope, expected_document) in [(PROJECT, true), (PROJECT + 1, false)] {
            let (mut core, env) = proposal_core(Some(people::Role::Member));
            party_proposal(&mut core, &env, scope);
            let answer = read_answer(project_proposal(
                &mut core,
                &env,
                REPLY,
                SIGN_IN,
                PROJECT,
                people::Proposer::Party(PROPOSER),
                PROPOSAL,
            ));
            let document = match answer {
                Answer::Document(_) => true,
                Answer::None(Why::Unknown) => false,
                other @ (Answer::Setting(_)
                | Answer::Revision(_)
                | Answer::Revisions(_)
                | Answer::TaskText(_)
                | Answer::Tasks(_)
                | Answer::Ended(_)
                | Answer::Page(_)
                | Answer::None(_)) => panic!("project-scoped proposal answer: {other:?}"),
            };
            assert_eq!(document, expected_document, "scope {scope}");
            tasks::step(
                &mut core.tasks,
                &env.with(env.limits.tasks),
                tasks::Event::DecidePersonProposal {
                    reply_to: ReplyTo::new(skein_lib::Token::new(93)),
                    proposer: PROPOSER,
                    proposal: PROPOSAL,
                    by: tasks::Party::Person(1),
                    message: None,
                    decision: tasks::ProposalDecision::Accept,
                    revision: Some(REVISION),
                },
                &mut Queue::uncapped(),
            );
            core.settings_proposals.remove(&(people::Proposer::Party(PROPOSER), u32::try_from(PROPOSAL).unwrap()));
            // A decided party proposal retains no project list, even if its set touched this project.
            assert_eq!(
                read_answer(project_proposal(
                    &mut core,
                    &env,
                    REPLY,
                    SIGN_IN,
                    PROJECT,
                    people::Proposer::Party(PROPOSER),
                    PROPOSAL
                )),
                Answer::None(Why::Unknown)
            );
        }
    }

    #[test]
    fn project_proposal_load_rechecks_membership_and_sign_in() {
        for (role, wall, why) in [
            (None, skein_lib::Wall::EPOCH, Why::Unknown),
            (Some(people::Role::Member), skein_lib::Wall::from_nanos(100), Why::SignedOut),
        ] {
            let (mut core, mut env) = proposal_core(Some(people::Role::Member));
            let mut out = project_proposal(
                &mut core,
                &env,
                REPLY,
                SIGN_IN,
                PROJECT,
                people::Proposer::Task(PROPOSER),
                PROPOSAL + 1,
            );
            let Some(Request::Held(held)) = out.pop() else { panic!("proposal load") };
            let crate::Held::ReadLoad { load, .. } = *held else { panic!("proposal load") };
            core.people = metadata_core(role).0.people;
            env.wall = wall;
            let out = crate::reads::loaded(&mut core, &env, &mut Queue::uncapped(), load, crate::reads::Loaded::Busy);
            assert_eq!(read_answer(out), Answer::None(why));
        }
    }

    #[test]
    fn ordinary_revision_reads_still_require_a_deployment_role() {
        let (mut core, env) = metadata_core(Some(people::Role::Member));
        let out =
            crate::reads::start(&mut core, &env, REPLY, SIGN_IN, crate::reads::Read::Revision { number: REVISION });
        assert_eq!(read_answer(out), Answer::None(Why::Unknown));
    }

    #[test]
    fn a_lowered_history_bound_hides_rows_the_commit_could_not_erase() {
        let (mut core, mut limits) = crate::tests::core();
        core.current.head.number = 6;
        core.current.oldest = 2;
        limits.settings.history = 2;
        assert_eq!(oldest(&core, &limits), 4);
        core.current.oldest = 5;
        assert_eq!(oldest(&core, &limits), 5);
        core.current.head.number = 1;
        core.current.oldest = 0;
        assert_eq!(oldest(&core, &limits), 1);
    }
    #[test]
    fn revision_selection_names_unknown_forgotten_and_kept_ranges() {
        const NEWEST: u32 = 6;
        const OLDEST: u32 = 4;
        for (number, expected) in [
            (0, Selection::Unknown),
            (1, Selection::Forgotten { oldest: OLDEST }),
            (3, Selection::Forgotten { oldest: OLDEST }),
            (4, Selection::Load { first: 4, last: 4 }),
            (5, Selection::Load { first: 5, last: 5 }),
            (6, Selection::Load { first: 6, last: 6 }),
            (7, Selection::Unknown),
        ] {
            assert_eq!(select_revision(NEWEST, OLDEST, number), expected, "{number}");
        }
    }
    #[test]
    fn dense_pages_select_only_the_requested_predecessors() {
        for (newest, oldest, before, rows, expected) in [
            (6, 4, None, 2, Some((5, 6))),
            (6, 4, Some(5), 2, Some((4, 4))),
            (6, 1, Some(5), 2, Some((3, 4))),
            (6, 1, Some(3), 2, Some((1, 2))),
            (6, 4, None, 8, Some((4, 6))),
            (6, 4, Some(4), 2, None),
            (6, 4, Some(0), 2, None),
            (6, 4, Some(u32::MAX), 2, Some((5, 6))),
            (u32::MAX, 1, None, 2, Some((u32::MAX - 1, u32::MAX))),
        ] {
            assert_eq!(page_range(newest, oldest, before, rows), expected);
        }
    }
    #[test]
    fn loads_admit_only_the_selected_history_and_held_keys() {
        use crate::{CoreKey, Key};
        let scope = Scope::SettingsHistory { first: 4, last: 6, newest: 6, whole: false };
        for (key, expected) in [
            (settings::Key::History(3), false),
            (settings::Key::History(4), true),
            (settings::Key::History(5), true),
            (settings::Key::History(6), false),
            (settings::Key::Held(5), true),
            (settings::Key::Held(6), true),
            (settings::Key::Held(7), false),
            (settings::Key::Extent, false),
        ] {
            assert_eq!(crate::reads::admits(1, &scope, &Key::Core(CoreKey::Settings(key))), expected);
        }
    }
    #[test]
    fn application_reads_keep_current_state_and_proposed_sizes_without_values() {
        use settings::application::{Follows, SizedEdit, State};
        let (mut core, _) = crate::tests::core();
        core.current.applications = Map::with_capacity(1);
        let setting = settings::Setting::Application { kind: 7, scope: 0, number: 2 };
        let state = State { follows: Follows::Default, size: 9, charters: Box::new([1]) };
        core.current.applications.insert(setting, state.clone()).unwrap();
        let Answer::Setting(document) = setting_value(&core, setting, core.current.newest()) else {
            panic!("application setting")
        };
        assert!(document.value.is_none());
        assert_eq!(document.application, Some(state.clone()));
        assert!(document.proposed.is_none());
        assert_eq!(setting_value(&core, setting, core.current.newest() + 1), Answer::None(Why::Unknown));
        for edit in [SizedEdit::Set { size: 13 }, SizedEdit::Remove, SizedEdit::Reset] {
            let Answer::Setting(document) = application_proposal(&core, setting, &edit) else {
                panic!("proposed application setting")
            };
            assert!(document.value.is_none());
            assert_eq!(document.application, Some(state.clone()));
            assert_eq!(document.proposed, Some(edit));
        }
    }
}
