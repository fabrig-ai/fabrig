//! The forge arm's brief bindings and the assignment handoff
//! (jig/docs/design/domain/engine/briefs.md, 1.4; jig/docs/design/domain/root.md, 7).
use crate::boundary::{BriefBody, BriefConnector, BriefKind, BriefSection, ForgeBriefKind, Payload, Work};
use crate::domain::Domain;
use crate::limits::{BriefBudgets, Limits};
use crate::route::{forge_route, forge_up};
use alloc::boxed::Box;
use jig_domain_brief::{GatherMissing, GatherPlaced};
use jig_domain_contract::{Up, ops, parts};
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Token, Writer};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as forge_change;

pub(crate) fn gather(
    domain: &mut Domain,
    limits: &Limits,
    task: u64,
    section: Token,
    budget: u32,
    deadline: skein_lib::Time,
) {
    let Some(row) = domain.brief_connectors.get(&section) else { return };
    if row.task != task {
        return;
    }
    if row.gatherable {
        domain.work.push(Work::Forge(forge::Event::GatherPlanned {
            task,
            section,
            parts: limits.brief.parts,
            bytes: budget,
            ci_budget: limits.brief.budgets.ci,
            deadline,
        }));
    } else {
        let failure = parts::Failure {
            fate: parts::Fate::Permanent,
            section: forge_brief_name(row.kind),
            words: Box::from(b"the connector cannot hold this section".as_slice()),
        };
        forge_up(domain, Up::Parts(parts::Up::Failed { task, section, failure }));
    }
}

pub(crate) fn cut(domain: &mut Domain, task: u64, section: Token, size: u32) {
    if let Some(row) = domain.brief_connectors.get(&section)
        && row.task == task
    {
        domain.work.push(Work::Forge(forge::Event::CutBrief { section, bytes: size }));
    }
}

pub(crate) fn drop_section(domain: &mut Domain, task: u64, section: Token) {
    retire_connector(domain, task, section);
    domain.work.push(Work::Forge(forge::Event::DropBrief { section }));
}

fn retire_connector(domain: &mut Domain, task: u64, section: Token) {
    if let Some(row) = domain.brief_connectors.get(&section)
        && row.task == task
    {
        domain.brief_connectors.remove(&section);
    }
}

/// Turn the engine's ordered brief into owned assignment sections, charging it once.
pub(crate) fn take_sections(domain: &mut Domain, task: u64, brief: Box<[GatherPlaced]>) -> Box<[BriefSection]> {
    let mut sections = List::with_capacity(u32::try_from(brief.len()).expect("bounded brief sections"));
    for placed in brief {
        let section = match placed {
            GatherPlaced::Core { kind, text } => {
                BriefSection { kind: BriefKind::Core(kind), body: BriefBody::Text(text) }
            }
            GatherPlaced::CoreMissing { kind, why } => {
                BriefSection { kind: BriefKind::Core(kind), body: BriefBody::Missing(why) }
            }
            GatherPlaced::Connector { connector, kind, token, size } => {
                assert_eq!(connector, domain.config.forge_connector, "only the forge serves sections");
                retire_connector(domain, task, token);
                let body = match domain.forge.take_brief(token) {
                    Some(bytes) if skein_lib::len_u64(bytes.len()) == u64::from(size) => BriefBody::Text(bytes),
                    Some(_) | None => BriefBody::Missing(GatherMissing::Failed),
                };
                BriefSection { kind: BriefKind::Forge(placed_brief_kind(domain, task, kind)), body }
            }
            GatherPlaced::Missing { connector, kind, why } => {
                assert_eq!(connector, domain.config.forge_connector, "only the forge serves sections");
                BriefSection {
                    kind: BriefKind::Forge(placed_brief_kind(domain, task, kind)),
                    body: BriefBody::Missing(why),
                }
            }
        };
        sections.push(section).expect("one section per placed input");
    }
    drop(domain.brief_document_titles.remove(&task));
    domain.hold_bytes(crate::budget::section_bytes(sections.as_slice()));
    sections.into_boxed()
}

fn placed_brief_kind(domain: &Domain, task: u64, kind: u16) -> ForgeBriefKind {
    let Some(index) = kind.checked_sub(DOCUMENT_KINDS) else { return numbered_brief_kind(kind) };
    let titles = domain.brief_document_titles.get(&task).expect("planned document titles");
    ForgeBriefKind::Document(titles.get(usize::from(index)).expect("document kind index").clone())
}

fn numbered_brief_kind(kind: u16) -> ForgeBriefKind {
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

fn document_source(document: &parts::Document) -> Option<forge::BriefDocument> {
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

fn document_title(document: &parts::Document, source: Option<&forge::BriefDocument>) -> Box<[u8]> {
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

fn bind_documents(domain: &mut Domain, context: &parts::Preparation, plan: &mut List<parts::Section>) {
    if context.documents.is_empty() {
        return;
    }
    // Index titles by the preparation's document indices, even if another connector owns one.
    let mut titles = List::with_capacity(domain.limits.tasks.charter_documents);
    for (index, document) in context.documents.iter().enumerate() {
        if document.connector != domain.config.forge_connector {
            titles.push(Box::<[u8]>::from([])).expect("charter document limit");
            continue;
        }
        let source = document_source(document);
        titles.push(document_title(document, source.as_ref())).expect("charter document limit");
        let index = u16::try_from(index).expect("bounded document index");
        let kind = DOCUMENT_KINDS.checked_add(index).expect("document kind");
        let token = bind(domain, context.task, kind, source.is_some());
        if let Some(document) = source {
            domain.work.push(Work::Forge(forge::Event::PlanDocument { section: token, document: Box::new(document) }));
        }
        plan.push(parts::Section {
            token,
            kind,
            name: b"document",
            size: 0,
            limit: domain.limits.brief.budgets.document,
            priority: parts::DOCUMENT_PRIORITY,
            required: false,
            document: Some(index),
        })
        .expect("one section per charter document");
    }
    drop(domain.brief_document_titles.insert(context.task, titles.into_boxed()).expect("one title list per live task"));
}

/// Missing documents have a kind but no token in the placed brief. Keep their
/// titles even after Drop, until the preparation hands them over or ends.
pub(crate) fn retire_document_titles(domain: &mut Domain) {
    let mut stale = List::with_capacity(domain.brief_document_titles.len());
    for (task, _) in &domain.brief_document_titles {
        match domain.engine.core().tasks.task(*task) {
            Some(row) if row.phase == tasks::Phase::Active(tasks::Active::Preparing) => {}
            Some(_) | None => stale.push(*task).expect("bounded title lists"),
        }
    }
    for task in &stale {
        drop(domain.brief_document_titles.remove(task));
    }
}

fn plan_home_access(
    domain: &mut Domain,
    env: &Env<Limits>,
    context: &parts::Preparation,
    parent: Option<u64>,
    plan: &mut List<parts::Section>,
) {
    let Some(home) = domain.forge.home(context.project) else { return };
    if let Some(parent) = parent
        && let Some(change) = domain.forge.change(parent)
        && let Some((child, _)) = change.delegate
        && child == context.task
    {
        return;
    }
    let design = if let Some(mut run) = domain.engine.core().tasks.run_context(context.task) {
        run.project = context.project;
        run.requester = context.requester;
        match forge_route::run_workspace(domain, env, &run, Some(context.root)) {
            Some(workspace) => !workspace.designs.is_empty(),
            None => false,
        }
    } else {
        false
    };
    if !design && forge_route::unheld_run_branch(domain, &env.limits, context.root, context.task, home) != Some(true) {
        return;
    }
    let source = forge::BriefSource::Home { repository: home.provider, design };
    let limit = forge_brief_budget(&ForgeBriefKind::Home, &domain.limits.brief.budgets);
    plan_forge_section(domain, context.task, source, None, 5, limit, plan);
}

fn bind(domain: &mut Domain, task: u64, kind: u16, gatherable: bool) -> Token {
    let token = domain.hub_link.fresh();
    domain
        .brief_connectors
        .insert(token, BriefConnector { task, kind, gatherable })
        .expect("one binding per planned token");
    token
}

fn plan_forge_section(
    domain: &mut Domain,
    task: u64,
    source: forge::BriefSource,
    partner: Option<Token>,
    priority: u16,
    limit: u32,
    plan: &mut List<parts::Section>,
) -> Token {
    let kind = forge_kind_number(&forge_brief_kind(source));
    let token = bind(domain, task, kind, true);
    domain.work.push(Work::Forge(forge::Event::PlanBrief { section: token, source, partner }));
    plan.push(parts::Section {
        name: forge_brief_name(kind),
        kind,
        token,
        size: 0,
        limit,
        priority,
        required: true,
        document: None,
    })
    .expect("one section per planned source");
    token
}

pub(crate) fn finish_brief_plan(domain: &mut Domain, env: &Env<Limits>, context: &parts::Preparation) {
    let parent = match context.requester {
        ops::Party::Task(parent) => Some(parent),
        ops::Party::Person(_) | ops::Party::Deployment { .. } => None,
    };
    let mut plan =
        List::with_capacity(u32::try_from(context.documents.len()).expect("bounded documents").saturating_add(3));
    plan_home_access(domain, env, context, parent, &mut plan);
    bind_documents(domain, context, &mut plan);
    if let Some(parent) = parent {
        plan_delegate(domain, context, parent, &mut plan);
    }
    forge_up(domain, Up::Parts(parts::Up::Planned { task: context.task, sections: plan.into_boxed() }));
}

fn plan_delegate(domain: &mut Domain, context: &parts::Preparation, parent: u64, plan: &mut List<parts::Section>) {
    let Some(row) = domain.forge.change(parent) else { return };
    let Some((child, delegate)) = row.delegate else { return };
    if row.terminal() || child != context.task {
        return;
    }
    let Some((number, _, _, head)) = row.review_target() else { return };
    let item = forge::BriefItem { repository: row.repository.repository, number };
    let head = forge::BriefCommit(head);
    let pull = forge::BriefSource::Pull { item, head };
    let why = match delegate {
        forge_change::Delegate::Repair(forge_change::Repair::Review { head: reviewed }) => {
            Some(forge::BriefSource::Request { item, head, reviewed: forge::BriefCommit(reviewed) })
        }
        forge_change::Delegate::Repair(forge_change::Repair::Gate(_)) => {
            Some(forge::BriefSource::Reviews { item, head })
        }
        forge_change::Delegate::Repair(
            forge_change::Repair::Ci
            | forge_change::Repair::LandingAt { .. }
            | forge_change::Repair::Landing { .. }
            | forge_change::Repair::Semantic,
        ) => Some(forge::BriefSource::Ci { item, head }),
        forge_change::Delegate::Resolve { .. }
        | forge_change::Delegate::Merge { .. }
        | forge_change::Delegate::Produce
        | forge_change::Delegate::Close
        | forge_change::Delegate::Gate { .. } => None,
    };
    let mut pull_limit = domain.limits.brief.budgets.pull;
    let mut reason = None;
    if let Some(why) = why {
        let limit = forge_brief_budget(&forge_brief_kind(why), &domain.limits.brief.budgets);
        let mut left = context.budget.saturating_sub(limit);
        for claim in &context.core {
            left = left.saturating_sub(claim.size.min(claim.limit));
        }
        // If the claims leave nothing, the sizes and priorities decide after gathering.
        if left > 0 {
            pull_limit = pull_limit.min(left);
        }
        reason = Some(plan_forge_section(domain, context.task, why, None, 5, limit, plan));
    }
    plan_forge_section(domain, context.task, pull, reason, 6, pull_limit, plan);
}

fn forge_brief_kind(source: forge::BriefSource) -> ForgeBriefKind {
    match source {
        forge::BriefSource::Document => unreachable!("documents are planned by PlanDocument"),
        forge::BriefSource::Home { .. } => ForgeBriefKind::Home,
        forge::BriefSource::Ci { .. } => ForgeBriefKind::Ci,
        forge::BriefSource::Reviews { .. } | forge::BriefSource::Request { .. } => ForgeBriefKind::Reviews,
        forge::BriefSource::Pull { .. } => ForgeBriefKind::Pull,
    }
}

pub(crate) fn forge_brief_name(kind: u16) -> &'static [u8] {
    match kind {
        1 => b"ci",
        2 => b"reviews",
        3 => b"pull",
        4 => b"home",
        _ => {
            assert!(kind >= DOCUMENT_KINDS, "forge brief kind");
            b"document"
        }
    }
}

fn forge_kind_number(kind: &ForgeBriefKind) -> u16 {
    match kind {
        ForgeBriefKind::Document(_) => unreachable!("document kinds carry their charter index"),
        ForgeBriefKind::Home => 4,
        ForgeBriefKind::Ci => 1,
        ForgeBriefKind::Reviews => 2,
        ForgeBriefKind::Pull => 3,
    }
}

fn forge_brief_budget(kind: &ForgeBriefKind, budgets: &BriefBudgets) -> u32 {
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
