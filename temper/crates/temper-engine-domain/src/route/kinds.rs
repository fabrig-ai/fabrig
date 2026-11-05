//! Typed settings handoffs within the root's decision
//! (temper/docs/design/domain/root.md, 5.6;
//! temper/docs/design/protocol/configuration.md, 13.2).
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
                let answer = judge(domain, &bounds, proposal, item);
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
        }
        connector::Ask::KeepProposalSettings { proposal, items } => {
            let asked = typed_items(domain, None, &items);
            domain.forge.keep_proposal_settings(proposal, &asked, &mut out);
        }
        connector::Ask::RestoreSettings { owner, proposal, items } => {
            let answers = restore(domain, &bounds(&env.limits), proposal, &items, &mut out);
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
        if way_refusal(domain, &asked).is_some() {
            for (item, answer) in items.iter().zip(answers.iter_mut()) {
                if (item.kind, item.scope, item.number) == (key.kind, key.scope, key.number) {
                    *answer = settings::Judged::Refuse(settings::Refusal::Way);
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
        settings::Judged::Refuse(why) => connector::Judged::Refuse { why: why.words().into() },
    }
}

fn judge(
    domain: &Domain,
    bounds: &settings::Bounds,
    proposal: Option<u64>,
    item: &application::Item,
) -> connector::Judged {
    let Some(asked) = typed(domain, proposal, item) else {
        return connector::Judged::Refuse { why: settings::Refusal::Value.words().into() };
    };
    let answers = domain.forge.judge_settings(bounds, core::slice::from_ref(&asked));
    let answer = *answers.first().expect("one forge judgement per item");
    match answer {
        settings::Judged::Refuse(_) => judgement(answer),
        settings::Judged::Accept { .. } | settings::Judged::Same => {
            match way_refusal(domain, &asked).or(approval_refusal(domain, &asked)) {
                Some(why) => connector::Judged::Refuse { why },
                None => judgement(answer),
            }
        }
    }
}

fn way_refusal(domain: &Domain, asked: &settings::Asked) -> Option<Box<[u8]>> {
    let value = match &asked.edit {
        settings::Edit::Set(value) => &value.value,
        settings::Edit::Remove | settings::Edit::Reset => return None,
    };
    let ways = match value {
        settings::Value::Ways(ways) => ways,
        settings::Value::Rule(_) | settings::Value::Charter(_) | settings::Value::Questions(_) => return None,
    };
    if ways.way == 0 {
        return None;
    }
    for template in domain.core.project_templates(asked.scope) {
        if template.number == ways.way {
            return None;
        }
    }
    Some(settings::Refusal::Way.words().into())
}

// temper/docs/design/domain/forge.md, 8.9: eligibility joins the forge's rule with the core's people.
fn approval_refusal(domain: &Domain, asked: &settings::Asked) -> Option<Box<[u8]>> {
    let value = match &asked.edit {
        settings::Edit::Set(value) => &value.value,
        settings::Edit::Remove | settings::Edit::Reset => return None,
    };
    let rule = match value {
        settings::Value::Rule(rule) => rule,
        settings::Value::Charter(_) | settings::Value::Questions(_) | settings::Value::Ways(_) => return None,
    };
    if !rule.enforce || rule.approvals.is_empty() {
        return None;
    }
    for &project in &domain.core.projects {
        if (asked.scope != 0 && asked.scope != project) || !covers_project(domain, rule, project) {
            continue;
        }
        for approval in &rule.approvals {
            let count = eligible_people(domain, project, approval.role);
            if count < approval.people {
                return Some(approval_words(project, approval, count));
            }
        }
    }
    None
}

fn covers_project(domain: &Domain, rule: &settings::LandingRule, project: u32) -> bool {
    use jig_domain_authority::{Last, Pattern, pattern_at_most};
    use temper_engine_domain_forge::GateLast;
    let pattern = Pattern {
        segments: rule.pattern.segments.clone(),
        last: match &rule.pattern.last {
            GateLast::Exact(last) => Last::Exact(last.clone()),
            GateLast::Open(last) => Last::Open(last.clone()),
        },
    };
    for repository in domain.forge.writable_repositories(project) {
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
        if pattern_at_most(&pattern, &branches) || pattern_at_most(&branches, &pattern) {
            return true;
        }
    }
    false
}

fn eligible_people(domain: &Domain, project: u32, role: u32) -> u32 {
    let mut count = 0_u32;
    let Some(members) = domain.core.people.project_members(project) else { return count };
    for member in members {
        let Some(identity) = domain.core.people.identity(member.person) else { continue };
        if member.role.number() == role && super::forge_route::human_forge_identity(domain, project, &identity.key) {
            count = count.saturating_add(1);
        }
    }
    count
}

fn approval_words(project: u32, approval: &settings::RuleApproval, count: u32) -> Box<[u8]> {
    let project = Decimal::of(u64::from(project));
    let role = Decimal::of(u64::from(approval.role));
    let required = Decimal::of(u64::from(approval.people));
    let count = Decimal::of(u64::from(count));
    let mut words = List::uncapped();
    for part in [
        b"project ".as_slice(), project.as_bytes(), b" role ", role.as_bytes(), b" requires ",
        required.as_bytes(), b" distinct known forge people; currently ", count.as_bytes(),
        b" eligible. Before accepting, add the human collaborator on the forge; configure this project's project-policy permissions if needed; ",
        b"re-adopt with repository adopt, verify the required role with project show, then submit a fresh proposal against the newest revision. ",
        b"See temper help configure for the approval setup route, or remove the approval requirement.",
    ] {
        for &byte in part {
            words.push(byte).expect("measured approval refusal");
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

/// Settings reads require membership in their requested scope (temper/docs/design/client/forge.md, 9).
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
    let delivery = match why {
        Some(why) => crate::Delivery::Read { reply, answer: Answer::None(why) },
        None => {
            let answer = match query {
                crate::ForgeRead::Setting { kind, scope, number } => {
                    crate::ForgeReadAnswer::Setting(domain.forge.setting(kind, scope, number).cloned())
                }
                crate::ForgeRead::RevisionSetting { .. } => unreachable!("revision reads load their metadata"),
                crate::ForgeRead::ProposalItems { key } => {
                    crate::ForgeReadAnswer::ProposalItems(domain.forge.proposal_settings(key))
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
                ApplicationScope::Deployment => domain.core.people.role(party, DEPLOYMENT).is_some(),
                ApplicationScope::Project(project) => {
                    let own_kind = match query {
                        crate::ForgeRead::Setting { kind, scope, .. }
                        | crate::ForgeRead::RevisionSetting { kind, scope, .. } => {
                            (*kind == 2 || *kind == 6 || *kind == 7) && *scope == project
                        }
                        crate::ForgeRead::ProposalItems { .. } => false,
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
