//! Typed settings handoffs within the root's decision
//! (temper/docs/design/domain/routing.md, 6;
//! temper/docs/design/protocol/configuration/settings.md, 2).
use alloc::boxed::Box;
use jig_domain_core::{Event, effects::connector, settings::application};
use skein_lib::{Decimal, Env, List, Queue};
use temper_engine_domain_forge::settings;

use crate::{Domain, Limits, Work};

#[expect(clippy::wildcard_enum_match_arm, reason = "the forge dispatcher selects only the settings asks")]
pub(super) fn ask(domain: &mut Domain, env: &Env<Limits>, number: u16, ask: connector::Ask) {
    if number != domain.config.forge_connector {
        return;
    }
    let mut out = Queue::uncapped();
    match ask {
        connector::Ask::JudgeSettings { owner, revision: _, proposal, items } => {
            let bounds = bounds(&env.limits);
            let mut judged = List::uncapped();
            for item in &items {
                let answer = judge(domain, &bounds, env.limits.authority.roles, owner, proposal, item);
                judged.push(answer).expect("one judgement per asked item");
            }
            domain.work.push(Work::Core(Event::EffectConnector(connector::Event::SettingsJudged {
                owner,
                judged: judged.into_boxed(),
            })));
        }
        connector::Ask::KeepSettings { revision, proposal, items } => {
            let asked = typed_items(domain, proposal, &items);
            domain.forge.keep_settings(&super::environment_forge(env), revision, proposal, &asked, &mut out);
            for item in &asked {
                if item.kind == 12 || item.kind == 13 { temper_engine_domain_forge::findings::mark_findings(item.scope, &mut out); }
            }
            let projects = domain.core.projects.clone();
            for &project in &projects {
                let mut affected = false;
                for item in &asked {
                    affected |= item.scope == 0 || item.scope == project;
                }
                if affected {
                    super::ways::mark(domain, project);
                }
            }
        }
        connector::Ask::KeepProposalSettings { proposal, items } => {
            let asked = typed_items(domain, None, &items);
            domain.forge.keep_proposal_settings(proposal, &asked, &mut out);
        }
        connector::Ask::RestoreSettings { owner, proposal, items } => {
            let answers = restore(domain, &bounds(&env.limits), env.limits.authority.roles, proposal, &items, &mut out);
            let mut judged = List::uncapped();
            for answer in answers {
                let answer = judgement(answer);
                judged.push(answer).expect("one judgement per restored item");
            }
            domain.work.push(Work::Core(Event::EffectConnector(connector::Event::SettingsRestored {
                owner,
                judged: judged.into_boxed(),
            })));
        }
        connector::Ask::ForgetReplaced { before } => domain.forge.forget_replaced(before, &mut out),
        connector::Ask::DropProposalSettings { proposal } => domain.forge.drop_proposal_settings(proposal, &mut out),
        connector::Ask::SaveSettingsDefaults { revision } => save_defaults(domain, revision, &mut out),
        _ => unreachable!("the forge routes only its settings asks here"),
    }
    if !out.is_empty() {
        domain.work.push(Work::ForgeSettings(out));
    }
}

fn restore(
    domain: &mut Domain,
    bounds: &settings::Bounds,
    owner_roles: u32,
    proposal: u64,
    items: &[application::Restored],
    out: &mut Queue<temper_engine_domain_forge::Request>,
) -> Box<[settings::Judged]> {
    let mut answers = domain.forge.restore_settings(bounds, proposal, items, out);
    let kept = domain.forge.proposal_settings(proposal);
    let mut accepted = List::uncapped();
    let mut refused = false;
    for (key, edit) in kept {
        let asked = settings::Asked { kind: key.kind, scope: key.scope, number: key.number, edit };
        if way_refusal(domain, None, &asked).is_some()
            || owner_refusal(domain, &settings::Bounds { roles: owner_roles, ..*bounds }, &asked).is_some()
        {
            for (item, answer) in items.iter().zip(answers.iter_mut()) {
                if (item.kind, item.scope, item.number) == (key.kind, key.scope, key.number) {
                    *answer = settings::Judged::Refuse(
                        owner_refusal(domain, &settings::Bounds { roles: owner_roles, ..*bounds }, &asked)
                            .unwrap_or(settings::Refusal::Way),
                    );
                }
            }
            refused = true;
        } else {
            accepted.push(asked).expect("one retained edit per accepted restore");
        }
    }
    if refused {
        domain.forge.drop_proposal_settings(proposal, out);
        domain.forge.keep_proposal_settings(proposal, &accepted.into_boxed(), out);
    }
    answers
}

fn judgement(answer: settings::Judged) -> connector::Judged {
    match answer {
        settings::Judged::Accept { size, charter } => connector::Judged::Accept {
            size,
            charters: match charter {
                Some(charter) => Box::new([charter]),
                None => Box::default(),
            },
        },
        settings::Judged::Same => connector::Judged::Same,
        settings::Judged::Refuse(why) => connector::Judged::Refuse { why: why.words() },
    }
}

fn judge(
    domain: &Domain,
    bounds: &settings::Bounds,
    owner_roles: u32,
    owner: skein_lib::Token,
    proposal: Option<u64>,
    item: &application::Item,
) -> connector::Judged {
    let Some(asked) = typed(domain, proposal, item) else {
        return connector::Judged::Refuse {
            why: match item.kind {
                12 => b"areas: value does not decode".as_slice().into(),
                13 => b"triager: value does not decode".as_slice().into(),
                _ => settings::Refusal::Value.words(),
            },
        };
    };
    let answers = domain.forge.judge_settings(bounds, core::slice::from_ref(&asked));
    let answer = answers.first().expect("one forge judgement per item").clone();
    match answer {
        settings::Judged::Refuse(_) => judgement(answer),
        settings::Judged::Accept { .. } | settings::Judged::Same => {
            match way_refusal(domain, Some(owner), &asked).or(approval_refusal(domain, &asked)).or(owner_refusal_words(
                domain,
                &settings::Bounds { roles: owner_roles, ..*bounds },
                &asked,
            )) {
                Some(why) => connector::Judged::Refuse { why },
                None => judgement(answer),
            }
        }
    }
}

fn way_refusal(domain: &Domain, owner: Option<skein_lib::Token>, asked: &settings::Asked) -> Option<Box<[u8]>> {
    let value = match &asked.edit {
        settings::Edit::Set(value) => &value.value,
        settings::Edit::Remove | settings::Edit::Reset => return None,
    };
    let ways = match value {
        settings::Value::Ways(ways) => ways,
        settings::Value::Rule(_)
        | settings::Value::Charter(_)
        | settings::Value::Goal(_)
        | settings::Value::Questions(_)
        | settings::Value::CharterWords(_)
        | settings::Value::Classes(_)
        | settings::Value::ProjectAreas(_)
        | settings::Value::ProjectTriager(_)
        | settings::Value::ProjectWords(_) => return None,
    };
    if ways.way == 0 {
        return None;
    }
    let templates = match owner {
        Some(owner) => domain.core.settings_project_templates(owner, asked.scope),
        None => domain.core.project_templates(asked.scope),
    };
    for template in templates {
        if template.number == ways.way {
            return None;
        }
    }
    let number = Decimal::of(u64::from(ways.way));
    let mut words = List::uncapped();
    for part in [b"way ".as_slice(), number.as_bytes(), b" is not a template the project offers after this decision"] {
        for &byte in part {
            words.push(byte).expect("bounded way refusal");
        }
    }
    Some(words.into_boxed())
}

// temper/docs/design/domain/forge/change/gates.md, 3: eligibility joins the forge's rule with the core's people.
fn approval_refusal(domain: &Domain, asked: &settings::Asked) -> Option<Box<[u8]>> {
    approval_for_projects(domain, asked, domain.core.projects.as_slice())
}

/// Acceptance's approval refusal, judged for one project at the call
/// (temper/docs/design/domain/agent/ask-temper.md, 2; domain/forge/change/gates.md, 3).
pub(super) fn call_approval_refusal(domain: &Domain, project: u32, kinds: &[crate::KindValue]) -> Option<Box<[u8]>> {
    for value in kinds {
        let asked = settings::Asked {
            kind: value.kind,
            scope: value.scope,
            number: value.number,
            edit: settings::Edit::Set(value.value.clone()),
        };
        if let Some(why) = approval_for_projects(domain, &asked, core::slice::from_ref(&project)) {
            return Some(why);
        }
    }
    None
}
fn approval_for_projects(domain: &Domain, asked: &settings::Asked, projects: &[u32]) -> Option<Box<[u8]>> {
    let value = match &asked.edit {
        settings::Edit::Set(value) => &value.value,
        settings::Edit::Remove | settings::Edit::Reset => return None,
    };
    let rule = match value {
        settings::Value::Rule(rule) => rule,
        settings::Value::Charter(_)
        | settings::Value::Questions(_)
        | settings::Value::Classes(_)
        | settings::Value::Goal(_)
        | settings::Value::CharterWords(_)
        | settings::Value::ProjectAreas(_)
        | settings::Value::ProjectTriager(_)
        | settings::Value::ProjectWords(_)
        | settings::Value::Ways(_) => return None,
    };
    if !rule.enforce || rule.approvals.is_empty() {
        return None;
    }
    for &project in projects {
        if asked.scope != 0 && asked.scope != project {
            continue;
        }
        let repositories = domain.forge.writable_repositories(project);
        let capacity = u32::try_from(repositories.len()).expect("bounded repositories");
        let mut seen = skein_lib::Set::with_capacity(capacity);
        for repository in repositories {
            let forge = repository.provider.forge;
            if !covers_repository(rule, repository) || !seen.insert(forge).expect("one entry per covered forge") {
                continue;
            }
            if let Some(configured) = domain.forge.configured(forge)
                && configured.mode == temper_engine_domain_forge::Mode::Fake
            {
                return Some(fake_approval_words(project));
            }
            for approval in &rule.approvals {
                let count = eligible_people(domain, project, forge, approval.role);
                if count < approval.people {
                    return Some(approval_words(project, &repository.host, approval, count));
                }
            }
        }
    }
    None
}

pub(super) fn covers_repository(
    rule: &settings::LandingRule,
    repository: &temper_engine_domain_forge::Repository,
) -> bool {
    use jig_domain_authority::{Last, Pattern, pattern_at_most};
    use temper_engine_domain_forge::GateLast;
    if let Some(first) = rule.pattern.segments.first()
        && first.as_ref() == b"branch"
    {
        return true;
    }
    let pattern = Pattern {
        segments: rule.pattern.segments.clone(),
        last: match &rule.pattern.last {
            GateLast::Exact(last) => Last::Exact(last.clone()),
            GateLast::Open(last) => Last::Open(last.clone()),
        },
    };
    let branches = Pattern {
        segments: Box::new([
            Box::from(b"forge".as_slice()),
            repository.host.clone(),
            repository.owner.clone(),
            repository.name.clone(),
            Box::from(b"branch".as_slice()),
        ]),
        last: Last::Open(Box::new([])),
    };
    pattern_at_most(&pattern, &branches) || pattern_at_most(&branches, &pattern)
}

/// The project's members with `role` who count as people on `forge` (domain/forge/change/gates.md, 3).
pub(super) fn eligible_people(domain: &Domain, project: u32, forge: u16, role: u32) -> u32 {
    let mut count = 0_u32;
    let Some(members) = domain.core.people.project_members(project) else { return count };
    for member in members {
        if member.role.number() == role && super::forge_route::forge_human(domain, project, forge, member.person) {
            count = count.saturating_add(1);
        }
    }
    count
}

fn approval_words(project: u32, host: &[u8], approval: &settings::RuleApproval, count: u32) -> Box<[u8]> {
    let project = Decimal::of(u64::from(project));
    let role = Decimal::of(u64::from(approval.role));
    let required = Decimal::of(u64::from(approval.people));
    let count = Decimal::of(u64::from(count));
    let mut words = List::uncapped();
    for part in [
        b"project ".as_slice(), project.as_bytes(), b" role ", role.as_bytes(), b" on forge ", host, b" requires ",
        required.as_bytes(), b" distinct known forge people; currently ", count.as_bytes(),
        b" eligible. Before accepting, add the person as a collaborator on the forge, give that role the permission in this project's policy if needed and adopt the repository again; ",
        b"then propose afresh against the newest revision, or remove the approval requirement.",
    ] {
        for &byte in part {
            words.push(byte).expect("measured approval refusal");
        }
    }
    words.into_boxed()
}

fn fake_approval_words(project: u32) -> Box<[u8]> {
    let project = Decimal::of(u64::from(project));
    let mut words = List::uncapped();
    for part in [
        b"project ".as_slice(), project.as_bytes(),
        b": the playground's forge has no accounts, so no one can approve there. Name a branch pattern or a project that leaves its repository out, or remove the approval requirement.",
    ] {
        for &byte in part {
            words.push(byte).expect("measured fake forge approval refusal");
        }
    }
    words.into_boxed()
}

fn bounds(limits: &Limits) -> settings::Bounds {
    settings::Bounds {
        landing_rules: limits.forge.landing_rules,
        gates: limits.forge.change_policy.gates,
        roles: limits.forge.roles,
        segments: limits.authority.segments,
        segment_bytes: limits.authority.segment_bytes.min(limits.forge.name_bytes),
        charters: limits.tasks.charters,
        templates: limits.templates,
    }
}

fn typed(domain: &Domain, proposal: Option<u64>, item: &application::Item) -> Option<settings::Asked> {
    let edit = match &item.edit {
        application::Edit::Set(_) => {
            let mut kept = None;
            if let Some(proposal) = proposal {
                for (key, edit) in &domain.forge.proposal_settings(proposal) {
                    if (key.kind, key.scope, key.number) == (item.kind, item.scope, item.number) {
                        kept = Some(edit.clone());
                        break;
                    }
                }
            }
            match kept {
                Some(edit) => edit,
                None => {
                    let mut supplied = None;
                    for value in &domain.kinds {
                        if (value.kind, value.scope, value.number) == (item.kind, item.scope, item.number) {
                            supplied = Some(&value.value);
                            break;
                        }
                    }
                    settings::Edit::Set(supplied?.clone())
                }
            }
        }
        application::Edit::Remove => settings::Edit::Remove,
        application::Edit::Reset => settings::Edit::Reset,
    };
    Some(settings::Asked { kind: item.kind, scope: item.scope, number: item.number, edit })
}

fn typed_items(domain: &Domain, proposal: Option<u64>, items: &[application::Item]) -> Box<[settings::Asked]> {
    let mut asked = List::uncapped();
    for item in items {
        asked
            .push(typed(domain, proposal, item).expect("kept items have their judged typed values"))
            .expect("one typed value per kept item");
    }
    asked.into_boxed()
}

fn save_defaults(domain: &mut Domain, revision: Option<u64>, out: &mut Queue<temper_engine_domain_forge::Request>) {
    if let Some(revision) = revision {
        domain.forge.remember_settings_defaults(revision, out);
    }
    domain.forge.save_settings_defaults(out);
}

/// Settings reads require membership in their requested scope (temper/docs/design/client/forge/served.md).
pub(super) fn read(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut crate::Decision,
    reply: skein_lib::ReplyTo,
    sign_in: u64,
    scope: jig_domain_people::ApplicationScope,
    query: crate::ForgeRead,
) {
    use jig_domain_core::reads::Answer;
    let why = read_refusal(domain, env, sign_in, scope, &query);
    if why.is_none()
        && let crate::ForgeRead::RevisionSetting { .. } = query
    {
        revision_begin(domain, env, decision, reply, sign_in, scope, query);
        return;
    }
    if why.is_none() {
        match query {
            crate::ForgeRead::View { project, address } => {
                super::ways::view::read(domain, env, decision, reply, sign_in, project, address);
                return;
            }
            crate::ForgeRead::Setting { .. }
            | crate::ForgeRead::RevisionSetting { .. }
            | crate::ForgeRead::ProposalItems { .. }
            | crate::ForgeRead::Finding { .. }
            | crate::ForgeRead::ProgramEnvironment => {}
        }
    }
    let delivery = match why {
        Some(why) => crate::Delivery::Read { reply, answer: Answer::None(why) },
        None => {
            let answer = match query {
                crate::ForgeRead::View { .. } => unreachable!("view reads have their own answer"),
                crate::ForgeRead::Finding { project, number } => crate::ForgeReadAnswer::Finding(super::findings::read(domain, project, number)),
                crate::ForgeRead::ProgramEnvironment => crate::ForgeReadAnswer::ProgramEnvironment,
                crate::ForgeRead::Setting { kind, scope, number } => {
                    crate::ForgeReadAnswer::Setting(domain.forge.setting(kind, scope, number).cloned())
                }
                crate::ForgeRead::RevisionSetting { .. } => unreachable!("revision reads load their metadata"),
                crate::ForgeRead::ProposalItems { key } => {
                    let items = domain.forge.proposal_settings(key);
                    crate::ForgeReadAnswer::ProposalItems {
                        owned_approval: super::approvals::owned_approval(domain, &items),
                        items,
                    }
                }
            };
            crate::Delivery::ForgeRead { reply, answer }
        }
    };
    super::emit(decision, &env.limits, delivery);
}

fn read_refusal(
    domain: &Domain,
    env: &Env<Limits>,
    sign_in: u64,
    scope: jig_domain_people::ApplicationScope,
    query: &crate::ForgeRead,
) -> Option<jig_domain_core::reads::Why> {
    use jig_domain_core::reads::Why;
    use jig_domain_people::{ApplicationScope, DEPLOYMENT};
    let party = domain.core.people.person(sign_in, env.now, env.wall);
    match party {
        None => Some(if domain.core.people.ready() { Why::SignedOut } else { Why::Unknown }),
        Some(party) => {
            let permitted = match scope {
                ApplicationScope::Deployment => match query {
                    crate::ForgeRead::View { project, .. } | crate::ForgeRead::Finding { project, .. } => domain.core.people.role(party, *project).is_some(),
                    crate::ForgeRead::ProgramEnvironment => {
                        domain.core.people.role(party, DEPLOYMENT) == Some(jig_domain_people::Role::Owner)
                    }
                    crate::ForgeRead::Setting { .. }
                    | crate::ForgeRead::RevisionSetting { .. }
                    | crate::ForgeRead::ProposalItems { .. } => domain.core.people.role(party, DEPLOYMENT).is_some(),
                },
                ApplicationScope::Project(project) => {
                    let own_kind = match query {
                        crate::ForgeRead::View { project: requested, .. } | crate::ForgeRead::Finding { project: requested, .. } => *requested == project,
                        crate::ForgeRead::Setting { kind, scope, .. }
                        | crate::ForgeRead::RevisionSetting { kind, scope, .. } => {
                            (*kind == 2
                                || *kind == 6
                                || *kind == 7
                                || *kind == 8
                                || *kind == 10
                                || *kind == 12
                                || *kind == 13)
                                && *scope == project
                        }
                        crate::ForgeRead::ProposalItems { .. } | crate::ForgeRead::ProgramEnvironment => false,
                    };
                    project != DEPLOYMENT && own_kind && domain.core.people.role(party, project).is_some()
                }
                ApplicationScope::Own => false,
            };
            if permitted { None } else { Some(Why::Unknown) }
        }
    }
}

fn revision_begin(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut crate::Decision,
    reply: skein_lib::ReplyTo,
    sign_in: u64,
    scope: jig_domain_people::ApplicationScope,
    query: crate::ForgeRead,
) {
    let crate::ForgeRead::RevisionSetting { revision, .. } = query else { unreachable!("revision query") };
    let token = reply.into_token();
    let reply = skein_lib::ReplyTo::new(token);
    if domain.revision_reads.insert(token, query).is_err() {
        super::emit(
            decision,
            &env.limits,
            crate::Delivery::Read {
                reply,
                answer: jig_domain_core::reads::Answer::None(jig_domain_core::reads::Why::Busy),
            },
        );
        return;
    }
    match scope {
        jig_domain_people::ApplicationScope::Project(project) => {
            let requests = jig_domain_core::reads::settings::revision_metadata(
                &mut domain.core,
                &super::environment_core(env),
                reply,
                sign_in,
                project,
                revision,
            );
            super::route_core_requests(domain, env, decision, requests);
        }
        jig_domain_people::ApplicationScope::Deployment => domain.work.push(Work::Core(Event::Read {
            reply,
            sign_in,
            read: jig_domain_core::reads::Read::Revision { number: revision },
        })),
        jig_domain_people::ApplicationScope::Own => unreachable!("settings reads require a shared scope"),
    }
}

#[expect(clippy::wildcard_enum_match_arm, reason = "non-revision answers are forwarded as the core returned them")]
pub(super) fn revision_answer(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut crate::Decision,
    reply: skein_lib::ReplyTo,
    answer: jig_domain_core::reads::Answer,
) {
    let token = reply.into_token();
    let reply = skein_lib::ReplyTo::new(token);
    if domain.view_reads.contains_key(&token) {
        super::ways::view::answered(domain, env, decision, reply, answer);
        return;
    }
    let Some(query) = domain.revision_reads.remove(&token) else {
        super::emit(decision, &env.limits, crate::Delivery::Read { reply, answer });
        return;
    };
    let crate::ForgeRead::RevisionSetting { kind, scope, number, revision } = query else {
        unreachable!("only revision reads await metadata")
    };
    let value = match &answer {
        jig_domain_core::reads::Answer::Revision(answer) => match answer.as_ref() {
            jig_domain_core::reads::RevisionAnswer::Revision { revision: record, .. } => revision_value(
                domain,
                record,
                (kind, scope, number),
                revision,
                jig_domain_core::reads::settings::oldest(&domain.core, &super::environment_core(env).limits),
            ),
            jig_domain_core::reads::RevisionAnswer::Unknown
            | jig_domain_core::reads::RevisionAnswer::Forgotten { .. } => None,
        },
        _ => {
            super::emit(decision, &env.limits, crate::Delivery::Read { reply, answer });
            return;
        }
    };
    super::emit(
        decision,
        &env.limits,
        crate::Delivery::ForgeRead { reply, answer: crate::ForgeReadAnswer::Setting(value) },
    );
}

fn revision_value(
    domain: &Domain,
    record: &jig_domain_core::settings::Revision,
    selector: (u32, u32, u32),
    revision: u32,
    oldest: u32,
) -> Option<settings::SizedValue> {
    if record.head.number != revision || revision < oldest {
        return None;
    }
    for item in &record.applications {
        if (item.kind, item.scope, item.number) == selector
            && let application::SizedEdit::Set { .. } = item.edit
        {
            return domain.forge.revision_setting(item.kind, item.scope, item.number, revision).cloned();
        }
    }
    None
}

fn owner_refusal_words(domain: &Domain, bounds: &settings::Bounds, asked: &settings::Asked) -> Option<Box<[u8]>> {
    match owner_refusal(domain, bounds, asked) {
        Some(why) => Some(why.words()),
        None => None,
    }
}
fn owner_refusal(domain: &Domain, bounds: &settings::Bounds, asked: &settings::Asked) -> Option<settings::Refusal> {
    let value = match &asked.edit {
        settings::Edit::Set(value) => &value.value,
        settings::Edit::Remove | settings::Edit::Reset => return None,
    };
    match value {
        settings::Value::ProjectAreas(value) => {
            for area in &value.areas {
                if !owner_of_project(domain, bounds, asked.scope, area.owner) {
                    let mut name = List::uncapped();
                    for part in [b"area ".as_slice(), area.name.as_ref()] {
                        for &byte in part {
                            name.push(byte).expect("bounded area name");
                        }
                    }
                    return Some(settings::Refusal::Owner { name: name.into_boxed(), owner: area.owner });
                }
            }
            None
        }
        settings::Value::ProjectTriager(value) => {
            if owner_of_project(domain, bounds, asked.scope, value.owner) {
                None
            } else {
                Some(settings::Refusal::Owner { name: b"triager".as_slice().into(), owner: value.owner })
            }
        }
        settings::Value::Rule(_)
        | settings::Value::Charter(_)
        | settings::Value::Questions(_)
        | settings::Value::Ways(_)
        | settings::Value::Classes(_)
        | settings::Value::Goal(_)
        | settings::Value::CharterWords(_)
        | settings::Value::ProjectWords(_) => None,
    }
}
fn owner_of_project(domain: &Domain, bounds: &settings::Bounds, project: u32, owner: settings::AreaOwner) -> bool {
    match owner {
        settings::AreaOwner::Task(number) => {
            let Some(task) = domain.core.tasks.task(number) else { return false };
            if task.project != project {
                return false;
            }
            match task.phase {
                jig_domain_tasks::Phase::Ended(_) => false,
                jig_domain_tasks::Phase::Closing(_)
                | jig_domain_tasks::Phase::Waiting
                | jig_domain_tasks::Phase::Active(_)
                | jig_domain_tasks::Phase::Held { .. } => true,
            }
        }
        settings::AreaOwner::Party(number) => domain.core.people.role(number, project).is_some(),
        settings::AreaOwner::Role(number) => number < bounds.roles,
        settings::AreaOwner::Charter(number) => number > 0 && number <= bounds.charters,
    }
}
