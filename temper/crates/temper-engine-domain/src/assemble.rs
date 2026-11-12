//! Assignments and briefs assembled from their owners' parts (jig's domain/root.md, 7).
use crate::Decision;
use crate::boundary::{BriefBody, BriefConnector, BriefKind, BriefSection, ForgeBriefKind, Payload, Work};
use crate::domain::Domain;
use crate::limits::{BriefBudgets, Limits};
use crate::route::forge_route::unheld_run_branch;
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_core::{Document, PlannedDocument};
use skein_lib::{Env, Id, List, Queue, Token, Writer};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as forge_change;

#[expect(clippy::too_many_lines, reason = "assembly preserves the section owner's permanent failure reason")]
pub(crate) fn brief_outputs(
    domain: &mut Domain,
    env: &Env<Limits>,
    _decision: &mut Decision,
    out: &mut Queue<brief::GatherRequest>,
) {
    for _ in 0..out.len() {
        match out.pop().expect("brief output count") {
            brief::GatherRequest::Gather { connector: _, section, budget } => {
                let Some(connector) = domain.brief_connectors.get(Id::from_token(section)) else { continue };
                if !connector.active {
                    continue;
                }
                if !connector.gatherable {
                    domain.work.push(Work::Brief(brief::GatherEvent::Missing {
                        brief: Token::new(connector.task),
                        section,
                        why: brief::GatherFailure {
                            fate: brief::GatherFate::Permanent,
                            section: forge_brief_name(&connector.kind),
                            words: Box::from(b"the connector cannot hold this section".as_slice()),
                        },
                    }));
                    continue;
                }
                domain.work.push(Work::Forge(forge::Event::GatherPlanned {
                    task: connector.task,
                    section,
                    parts: env.limits.brief.parts,
                    bytes: budget,
                    ci_budget: env.limits.brief.budgets.ci,
                    deadline: connector.deadline,
                }));
            }
            brief::GatherRequest::CutTo { connector: _, section, size } => {
                if let Some(row) = domain.brief_connectors.get_mut(Id::from_token(section))
                    && row.active
                {
                    row.cutting = true;
                    domain.work.push(Work::Forge(forge::Event::CutBrief { section, bytes: size }));
                }
            }
            brief::GatherRequest::Drop { connector: _, section } => {
                let id = Id::from_token(section);
                if let Some(row) = domain.brief_connectors.get(id)
                    && row.active
                {
                    let gatherable = row.gatherable;
                    retire_connector(domain, id);
                    if gatherable {
                        domain.work.push(Work::Forge(forge::Event::DropBrief { section }));
                    }
                }
            }
            brief::GatherRequest::Complete { brief, order } => {
                let task = brief.raw();
                let mut sections = List::with_capacity(u32::try_from(order.len()).expect("bounded section count"));
                let mut missing_owner = false;
                for placed in order {
                    let section = match placed {
                        brief::GatherPlaced::Core { kind, text } => {
                            BriefSection { kind: BriefKind::Core(kind), body: BriefBody::Text(text) }
                        }
                        brief::GatherPlaced::CoreMissing { kind, why } => {
                            BriefSection { kind: BriefKind::Core(kind), body: BriefBody::Missing(why) }
                        }
                        brief::GatherPlaced::Connector { token, size, .. } => {
                            let id = Id::from_token(token);
                            let Some(row) = domain.brief_connectors.get(id) else {
                                missing_owner = true;
                                continue;
                            };
                            let kind = row.kind.clone();
                            retire_connector(domain, id);
                            let bytes = domain.forge.take_brief(token);
                            match bytes {
                                Some(bytes) if bytes.len() == usize::try_from(size).expect("bounded section") => {
                                    BriefSection { kind: BriefKind::Forge(kind), body: BriefBody::Text(bytes) }
                                }
                                Some(_) | None => {
                                    missing_owner = true;
                                    continue;
                                }
                            }
                        }
                        brief::GatherPlaced::Missing { kind, why, .. } => BriefSection {
                            kind: BriefKind::Forge(placed_brief_kind(domain, task, kind)),
                            body: BriefBody::Missing(why),
                        },
                    };
                    sections.push(section).expect("bounded brief sections");
                }
                drop(domain.brief_document_titles.remove(&task));
                if missing_owner {
                    domain.work.push(Work::Core(jig_domain_core::Event::BriefAssembled {
                        task,
                        failure: Some(brief::GatherFailure {
                            fate: brief::GatherFate::Permanent,
                            section: b"brief",
                            words: Box::from(b"a section's owner no longer holds it".as_slice()),
                        }),
                    }));
                    continue;
                }
                // Text still being gathered in the core or forge is not counted;
                // completed sections enter the root's ownership here.
                domain.hold_bytes(crate::budget::section_bytes(sections.as_slice()));
                crate::link::keep_sections(domain, task, sections.into_boxed());
                domain.work.push(Work::Core(jig_domain_core::Event::BriefAssembled { task, failure: None }));
            }
            brief::GatherRequest::Failed { .. } | brief::GatherRequest::Refused { .. } => {
                unreachable!("the core abandons refused brief preparation")
            }
        }
    }
}

fn placed_brief_kind(domain: &Domain, task: u64, kind: u16) -> ForgeBriefKind {
    let Some(index) = kind.checked_sub(DOCUMENT_KINDS) else { return numbered_brief_kind(kind) };
    let titles = domain.brief_document_titles.get(&task).expect("planned document titles");
    ForgeBriefKind::Document(titles.get(usize::from(index)).expect("document kind index").clone())
}

pub(crate) fn numbered_brief_kind(kind: u16) -> ForgeBriefKind {
    match kind {
        1 => ForgeBriefKind::Ci,
        2 => ForgeBriefKind::Reviews,
        3 => ForgeBriefKind::Pull,
        4 => ForgeBriefKind::Home,
        _ => unreachable!("temper's forge connector uses four section kinds"),
    }
}

pub(crate) fn take_payload(domain: &mut Domain, token: Token) -> Option<Payload> {
    domain.answer_parts.remove(&token);
    domain.refused_calls.remove(&token);
    domain.hub_link.take_payload(token)
}

/// Charter document kinds keep their order apart from the forge's fixed kinds
/// (temper/docs/design/protocol/configuration/settings.md, 6).
pub(crate) const DOCUMENT_KINDS: u16 = 16;

fn document_source(document: &Document) -> Option<forge::BriefDocument> {
    let [prefix, host, owner, name, branch, segments @ ..] = document.name.segments.as_ref() else { return None };
    if prefix.as_ref() != b"forge" || branch.as_ref() != b"branch" || segments.is_empty() {
        return None;
    }
    let mut bytes = segments.len().checked_sub(1).expect("branch has a segment");
    for segment in segments {
        bytes = bytes.checked_add(segment.len()).expect("bounded branch name");
    }
    let mut branch = Writer::new(bytes);
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            branch.put(b"/").expect("branch separator counted");
        }
        branch.put(segment).expect("branch segment counted");
    }
    Some(forge::BriefDocument {
        host: host.clone(),
        owner: owner.clone(),
        name: name.clone(),
        branch: branch.finish(),
        path: document.path.clone(),
        heading: document.heading.clone(),
    })
}

fn document_title(document: &Document, source: Option<&forge::BriefDocument>) -> Box<[u8]> {
    let mut bytes = document.path.len();
    if let Some(source) = source {
        bytes = bytes
            .checked_add(source.owner.len())
            .expect("bounded owner")
            .checked_add(source.name.len())
            .expect("bounded repository")
            .checked_add(2)
            .expect("title separators");
        if let Some(heading) = &document.heading {
            bytes =
                bytes.checked_add(heading.len()).expect("bounded heading").checked_add(4).expect("heading punctuation");
        }
    }
    let mut title = Writer::new(bytes);
    if let Some(source) = source {
        title.put(&source.owner).expect("owner counted");
        title.put(b"/").expect("separator counted");
        title.put(&source.name).expect("repository counted");
        title.put(b" ").expect("separator counted");
    }
    title.put(&document.path).expect("path counted");
    if source.is_some()
        && let Some(heading) = &document.heading
    {
        title.put(b", \"").expect("heading punctuation counted");
        title.put(heading).expect("heading counted");
        title.put(b"\"").expect("heading punctuation counted");
    }
    title.finish()
}

fn bind_documents(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    wanted: &mut List<brief::Planned>,
    documents: Box<[PlannedDocument]>,
) {
    if documents.is_empty() {
        return;
    }
    let mut titles = List::with_capacity(domain.limits.tasks.charter_documents);
    for (index, planned) in documents.into_iter().enumerate() {
        let source = if planned.document.connector == domain.config.forge_connector {
            document_source(&planned.document)
        } else {
            None
        };
        let title = document_title(&planned.document, source.as_ref());
        titles.push(title.clone()).expect("charter document limit");
        let token = domain
            .brief_connectors
            .insert(BriefConnector {
                task,
                kind: ForgeBriefKind::Document(title),
                gatherable: source.is_some(),
                active: true,
                cutting: false,
                deadline: env.now.saturating_add(domain.limits.brief.gather),
            })
            .expect("reserved document section room")
            .token();
        let mut bound = false;
        for at in 0..wanted.len() {
            let section = wanted.get_mut(at).expect("planned section");
            if let brief::Planned::Connector { connector, token: core_token, kind, limit, .. } = section
                && *connector == planned.document.connector
                && *core_token == planned.token
            {
                *core_token = token;
                *kind = DOCUMENT_KINDS
                    .checked_add(u16::try_from(index).expect("bounded document index"))
                    .expect("document kind");
                *limit = domain.limits.brief.budgets.document;
                bound = true;
                break;
            }
        }
        assert!(bound, "each planned document has a matching section");
        if let Some(document) = source {
            domain.work.push(Work::Forge(forge::Event::PlanDocument { section: token, document: Box::new(document) }));
        }
    }
    assert!(domain.brief_document_titles.insert(task, titles.into_boxed()) == Ok(None), "one document plan per task");
}

fn retire_connector(domain: &mut Domain, id: Id<BriefConnector>) {
    let row = domain.brief_connectors.get_mut(id).expect("retiring a bound section");
    row.active = false;
    if let ForgeBriefKind::Document(title) = &mut row.kind {
        *title = Box::new([]);
    }
    domain.brief_connectors.retire(id);
}

fn abandon_plan(domain: &mut Domain, task: u64) {
    loop {
        let mut section = None;
        for (id, row) in &domain.brief_connectors {
            if row.active && row.task == task {
                section = Some((id, row.gatherable));
                break;
            }
        }
        let Some((id, gatherable)) = section else { break };
        retire_connector(domain, id);
        if gatherable {
            domain.work.push(Work::Forge(forge::Event::DropBrief { section: id.token() }));
        }
    }
    drop(domain.brief_document_titles.remove(&task));
}

/// A core batch routes all drops and its completion together. Titles without
/// connector rows after that batch belong to an abandoned brief (domain/root.md, 6.2).
pub(crate) fn retire_document_titles(domain: &mut Domain) {
    loop {
        let mut abandoned = None;
        for (task, _) in &domain.brief_document_titles {
            let mut rows = false;
            for (_, row) in &domain.brief_connectors {
                if row.active && row.task == *task {
                    rows = true;
                    break;
                }
            }
            if !rows {
                abandoned = Some(*task);
                break;
            }
        }
        let Some(task) = abandoned else { break };
        drop(domain.brief_document_titles.remove(&task));
    }
}

fn optional_core(section: &brief::Planned) -> bool {
    match section {
        brief::Planned::Core { required, .. } => !required,
        brief::Planned::Connector { .. } => false,
    }
}

fn plan_home_access(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    parent: Option<u64>,
    wanted: &mut List<brief::Planned>,
) -> bool {
    let Some(row) = domain.core.tasks.task(task) else { return true };
    let Some(home) = domain.forge.home(row.project) else { return true };
    let design = match domain.core.tasks.run_context(task) {
        Some(context) => match crate::route::forge_route::run_workspace(domain, env, &context, 0) {
            Some(workspace) => !workspace.designs.is_empty(),
            None => false,
        },
        None => false,
    };
    if !design && unheld_run_branch(domain, &env.limits, task, home) != Some(true) {
        return true;
    }
    let inherited = match parent {
        Some(parent) => match domain.forge.change(parent) {
            Some(change) => match change.delegate {
                Some((child, _)) => child == task,
                None => false,
            },
            None => false,
        },
        None => false,
    };
    if inherited {
        return true;
    }
    // Required context takes a slot before optional core context. If every
    // slot is already required, preparation fails rather than omitting Home.
    let mut optional = None;
    for (index, section) in wanted.iter().enumerate() {
        if optional_core(section) {
            optional = Some(index);
        }
    }
    if wanted.room() == 0 && optional.is_none() {
        return false;
    }
    let source = forge::BriefSource::Home { repository: home.provider, design };
    let kind = forge_brief_kind(source);
    let token = domain
        .brief_connectors
        .insert(BriefConnector {
            task,
            kind: kind.clone(),
            gatherable: true,
            active: true,
            cutting: false,
            deadline: env.now.saturating_add(domain.limits.brief.gather),
        })
        .expect("reserved connector section room")
        .token();
    domain.work.push(Work::Forge(forge::Event::PlanBrief { section: token, source, partner: None }));
    let section = brief::Planned::Connector {
        name: forge_brief_name(&kind),
        connector: 0,
        kind: forge_kind_number(&kind),
        token,
        size: 0,
        limit: forge_brief_budget(&kind, &domain.limits.brief.budgets),
        priority: 5,
        required: true,
    };
    if wanted.room() > 0 {
        wanted.push(section).expect("home section room");
    } else {
        *wanted.get_mut(u32::try_from(optional.expect("optional slot")).expect("section index")).expect("section") =
            section;
    }
    true
}

/// Required forge context replaces trailing optional context, preserving the plan's order.
fn make_forge_room(wanted: &mut List<brief::Planned>, needed: u32) {
    for _ in 0..needed {
        if wanted.room() >= needed {
            break;
        }
        let mut optional = None;
        for (index, section) in wanted.iter().enumerate() {
            if optional_core(section) {
                optional = u32::try_from(index).ok();
            }
        }
        let index = optional.expect("forge section room checked");
        for at in index..wanted.len().saturating_sub(1) {
            let next = wanted.get(at.checked_add(1).expect("bounded section index")).expect("next section").clone();
            *wanted.get_mut(at).expect("section") = next;
        }
        wanted.truncate(wanted.len().saturating_sub(1));
    }
}

#[expect(clippy::too_many_arguments, reason = "a paired section names its partner alongside its source and budget")]
fn plan_forge_section(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    source: forge::BriefSource,
    partner: Option<Token>,
    priority: u16,
    limit: u32,
    wanted: &mut List<brief::Planned>,
) -> Token {
    let kind = forge_brief_kind(source);
    let token = domain
        .brief_connectors
        .insert(BriefConnector {
            task,
            kind: kind.clone(),
            gatherable: true,
            active: true,
            cutting: false,
            deadline: env.now.saturating_add(domain.limits.brief.gather),
        })
        .expect("reserved connector section room")
        .token();
    domain.work.push(Work::Forge(forge::Event::PlanBrief { section: token, source, partner }));
    wanted
        .push(brief::Planned::Connector {
            name: forge_brief_name(&kind),
            connector: 0,
            kind: forge_kind_number(&kind),
            token,
            size: 0,
            limit,
            priority,
            required: true,
        })
        .expect("required connector section room");
    token
}

pub(crate) fn finish_brief_plan(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    parent: Option<u64>,
    mut wanted: List<brief::Planned>,
    documents: Box<[PlannedDocument]>,
) {
    bind_documents(domain, env, task, &mut wanted, documents);
    if !plan_home_access(domain, env, task, parent, &mut wanted) {
        abandon_plan(domain, task);
        domain.work.push(Work::Core(jig_domain_core::Event::BriefAssembled {
            task,
            failure: Some(brief::GatherFailure {
                fate: brief::GatherFate::Permanent,
                section: b"home",
                words: Box::from(b"no room for its required section".as_slice()),
            }),
        }));
        return;
    }
    if let Some(parent) = parent
        && let Some(row) = domain.forge.change(parent)
        && !row.terminal()
        && let Some((child, _)) = row.delegate
        && child == task
        && let Some((number, _, _, head)) = row.review_target()
    {
        let item = forge::BriefItem { repository: row.repository.repository, number };
        let pull = forge::BriefSource::Pull { item, head: forge::BriefCommit(head) };
        let (why, reason_first) = match row.delegate.expect("matched delegate").1 {
            forge_change::Delegate::Repair(forge_change::Repair::Review { head: reviewed }) => (
                Some(forge::BriefSource::Request {
                    item,
                    head: forge::BriefCommit(head),
                    reviewed: forge::BriefCommit(reviewed),
                }),
                false,
            ),
            forge_change::Delegate::Repair(forge_change::Repair::Gate(_)) => {
                (Some(forge::BriefSource::Reviews { item, head: forge::BriefCommit(head) }), false)
            }
            forge_change::Delegate::Repair(forge_change::Repair::Ci | forge_change::Repair::Landing { .. }) => {
                (Some(forge::BriefSource::Ci { item, head: forge::BriefCommit(head) }), false)
            }
            forge_change::Delegate::Repair(forge_change::Repair::Semantic) => {
                (Some(forge::BriefSource::Ci { item, head: forge::BriefCommit(head) }), true)
            }
            forge_change::Delegate::Resolve { .. }
            | forge_change::Delegate::Merge { .. }
            | forge_change::Delegate::Produce
            | forge_change::Delegate::Close
            | forge_change::Delegate::Gate { .. } => (None, false),
        };
        let needed = if why.is_some() { 2 } else { 1 };
        let mut room = wanted.room();
        for section in &wanted {
            if optional_core(section) {
                room = room.saturating_add(1);
            }
        }
        if room < needed {
            abandon_plan(domain, task);
            domain.work.push(Work::Core(jig_domain_core::Event::BriefAssembled {
                task,
                failure: Some(brief::GatherFailure {
                    fate: brief::GatherFate::Permanent,
                    section: b"brief",
                    words: Box::from(b"no room for its required forge sections".as_slice()),
                }),
            }));
            return;
        }
        make_forge_room(&mut wanted, needed);
        let mut pull_limit = domain.limits.brief.budgets.pull;
        if let Some(why) = why {
            // The reason keeps its section budget. Pull contents yield before
            // the retained core context, including its optional sections.
            let reason_limit = forge_brief_budget(&forge_brief_kind(why), &domain.limits.brief.budgets);
            let mut left = domain.limits.brief.brief_bytes.saturating_sub(reason_limit);
            for section in &wanted {
                left = left.saturating_sub(section.size().min(section.limit()));
            }
            // If the declared reason leaves no room, its actual size and the
            // section priorities decide the fit after gathering.
            if left > 0 {
                pull_limit = pull_limit.min(left);
            }
        }
        if reason_first && let Some(why) = why {
            let limit = forge_brief_budget(&forge_brief_kind(why), &domain.limits.brief.budgets);
            let reason = plan_forge_section(domain, env, task, why, None, 5, limit, &mut wanted);
            plan_forge_section(domain, env, task, pull, Some(reason), 6, pull_limit, &mut wanted);
        } else {
            let pull = plan_forge_section(domain, env, task, pull, None, 6, pull_limit, &mut wanted);
            if let Some(why) = why {
                let limit = forge_brief_budget(&forge_brief_kind(why), &domain.limits.brief.budgets);
                plan_forge_section(domain, env, task, why, Some(pull), 5, limit, &mut wanted);
            }
        }
    }
    domain.work.push(Work::Brief(brief::GatherEvent::Plan {
        brief: Token::new(task),
        budget: domain.limits.brief.brief_bytes,
        deadline: env.now.saturating_add(domain.limits.brief.gather),
        sections: wanted.into_boxed(),
    }));
}

pub(crate) fn forge_brief_kind(source: forge::BriefSource) -> ForgeBriefKind {
    match source {
        forge::BriefSource::Document => unreachable!("documents are planned by PlanDocument"),
        forge::BriefSource::Home { .. } => ForgeBriefKind::Home,
        forge::BriefSource::Ci { .. } => ForgeBriefKind::Ci,
        forge::BriefSource::Reviews { .. } | forge::BriefSource::Request { .. } => ForgeBriefKind::Reviews,
        forge::BriefSource::Pull { .. } => ForgeBriefKind::Pull,
    }
}

pub(crate) fn forge_brief_name(kind: &ForgeBriefKind) -> &'static [u8] {
    match kind {
        ForgeBriefKind::Pull => b"pull",
        ForgeBriefKind::Ci => b"ci",
        ForgeBriefKind::Reviews => b"reviews",
        ForgeBriefKind::Home => b"home",
        ForgeBriefKind::Document(_) => b"document",
    }
}

pub(crate) fn forge_kind_number(kind: &ForgeBriefKind) -> u16 {
    match kind {
        ForgeBriefKind::Document(_) => unreachable!("document kinds carry their charter index"),
        ForgeBriefKind::Home => 4,
        ForgeBriefKind::Ci => 1,
        ForgeBriefKind::Reviews => 2,
        ForgeBriefKind::Pull => 3,
    }
}

pub(crate) fn forge_brief_budget(kind: &ForgeBriefKind, budgets: &BriefBudgets) -> u32 {
    match kind {
        ForgeBriefKind::Document(_) => budgets.document,
        ForgeBriefKind::Home => forge::HOME_SECTION_BYTES,
        ForgeBriefKind::Ci => budgets.ci,
        ForgeBriefKind::Reviews => budgets.reviews,
        ForgeBriefKind::Pull => budgets.pull,
    }
}

#[cfg(test)]
mod tests {
    use super::{ForgeBriefKind, forge_brief_budget};
    use temper_engine_domain_forge as forge;

    #[test]
    fn the_home_section_has_the_adoption_holds_exact_words() {
        let expected = b"This chat began before org/repo was adopted, so you can read org/repo but not push to it. Tell the person, in your next reply: \"This chat began before org/repo was adopted, so I can read it but not push. Start a new chat to push.\"";
        assert_eq!(
            forge::home_access_reason(b"org", b"repo", forge::HOME_SECTION_BYTES),
            forge::BriefRead::Got { bytes: expected.as_slice().into(), left: 0 },
        );
    }

    #[test]
    fn the_home_section_constant_fits_the_longest_admitted_names() {
        let mut names = skein_lib::List::with_capacity(forge::PANEL_NAME_BYTES);
        for _ in 0..forge::PANEL_NAME_BYTES {
            names.push(b'x').expect("name bound");
        }
        let names = names.into_boxed();
        let forge::BriefRead::Got { bytes, left } =
            forge::home_access_reason(&names, &names, forge::HOME_SECTION_BYTES)
        else {
            panic!("home reason")
        };
        assert_eq!(left, 0);
        assert_eq!(bytes.len(), usize::try_from(forge::HOME_SECTION_BYTES).expect("section bytes"));
        let budgets = crate::BriefBudgets {
            document: 4096,
            task: 1,
            dependencies: 1,
            ci: 1,
            reviews: 1,
            pull: 1,
            attempts: 1,
            plan: 1,
            notes: 1,
            preferences: 1,
            now: 1,
        };
        assert_eq!(forge_brief_budget(&ForgeBriefKind::Home, &budgets), forge::HOME_SECTION_BYTES);
    }
}
