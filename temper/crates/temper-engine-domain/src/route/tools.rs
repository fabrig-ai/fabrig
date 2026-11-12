//! The decoded engine tools, translated into core or connector vocabulary
//! (domain/routing.md, 2; jig's `domain/engine/executors.md`, 3).
use super::{
    Decision, Domain, Env, EscalationChoice, Limits, MessageForm, ProposalChoice, ProposedAction, ReplyTo, Tool, Work,
    call_answer, current_proof, delegate_call, forge_route, proposals, relay_call, tool_kind,
};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_tasks as tasks;

#[expect(
    clippy::too_many_lines,
    reason = "the root translates every temper host tool into core or connector vocabulary"
)]
pub(crate) fn relay_payload(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    reply_to: ReplyTo,
    key: crate::CallKey,
    body: crate::Call,
) {
    assert!(current_proof(domain, key.task, key.attempt), "fleet only relays a current claim");
    if let Tool::ReadTask { task, field, from } = body.tool {
        let room = crate::limits::task_read_room(&domain.limits, domain.reply_bound(key));
        domain.work.push(Work::Core(core::Event::NamedAction {
            to: reply_to,
            key,
            action: core::NamedAction::ReadTask { task, field, from, room },
        }));
        return;
    }
    // Settings reads enter before replay and admission to the durable call map.
    // The core checks their family and answers denials through its immediate lane.
    if let Tool::Settings { mut read, .. } = body.tool {
        if let core::SettingsRead::Setting { revision, .. } = &mut read
            && *revision == 0
        {
            *revision = domain.core.current.newest();
        }
        domain.work.push(Work::Core(core::Event::NamedAction {
            to: reply_to,
            key,
            action: core::NamedAction::Settings(read),
        }));
        return;
    }
    if let Some(core::CallPart::Effect {
        deadline,
        outcome:
            None
            | Some(
                core::effects::connector::OutboxOutcome::Uncertain
                | core::effects::connector::OutboxOutcome::Held { .. },
            ),
        ..
    }) = domain.core.call_parts.get(&key)
        && env.wall < *deadline
    {
        let deadline = *deadline;
        let owner = domain.core.effect_owner();
        domain.work.push(Work::Core(core::Event::EffectStart {
            owner,
            connector: domain.config.forge_connector,
            origin: core::EffectOrigin::Call { to: reply_to, key, deadline },
        }));
        return;
    }
    let reading = crate::translate::is_core_read(&body.tool);
    let replay = if reading { None } else { call_answer(domain, key) };
    match replay {
        Some(answer) => relay_call(domain, &env.limits, decision, reply_to, answer),
        None => {
            if let Some(kind) = tool_kind(&body.tool)
                && let Some(part) = domain.core.authorize_tool(key, kind)
            {
                if reading {
                    super::host_route::read(
                        domain,
                        reply_to,
                        super::CallAnswer::from_core(&part).expect("core tool authorization answer"),
                    );
                } else {
                    domain.work.push(Work::Core(core::Event::NamedAnswer { to: reply_to, key, part }));
                }
                return;
            }
            let tool = if let Tool::ReadNamed { repository, mut read } = body.tool {
                let project = match domain.core.tasks.delegation(key.task) {
                    Some(context) => Some(context.project),
                    None => None,
                };
                let resolved = match project {
                    Some(project) => forge_route::named_repository(domain, project, repository.as_deref()),
                    None => Err(tasks::Refusal::Unknown),
                };
                let Ok(adopted) = resolved else {
                    let repositories = match project {
                        Some(project) => forge_route::read_repositories(domain, project),
                        None => Box::new([]),
                    };
                    super::host_route::read(
                        domain,
                        reply_to,
                        super::CallAnswer::ReadRepositoryRefused { named: repository, repositories },
                    );
                    return;
                };
                if let skein_forge::api::Read::File { max_bytes, .. } | skein_forge::api::Read::Job { max_bytes, .. } =
                    &mut read
                    && *max_bytes == 0
                {
                    *max_bytes = env.limits.forge.client.answer_bytes;
                }
                Tool::ReadForge { repository: adopted.provider, read }
            } else {
                body.tool
            };
            match tool {
                Tool::See { line, from } => super::see::call(domain, env, decision, reply_to, key, line, from),
                Tool::Document { action, body } => {
                    super::document::call(domain, env, decision, reply_to, key, action, body);
                }
                Tool::Remember { action } => super::remember::call(domain, env, decision, reply_to, key, action),
                Tool::Finding(call) => super::finding_tool::call(domain, env, decision, reply_to, key, call),
                Tool::Decided(call) => super::decided::call(domain, env, decision, reply_to, key, call),
                Tool::ReadTask { .. } => unreachable!("task reads enter before replay"),
                Tool::Settings { .. } => unreachable!("settings reads enter before replay"),
                tool @ Tool::ProposeSettingsNote { .. } => {
                    super::host_route::settings_note(
                        domain,
                        env,
                        decision,
                        reply_to,
                        crate::Call { completion: body.completion, position: body.position, tool },
                    );
                }
                tool @ Tool::ProposeSettings { .. } => propose_settings(domain, reply_to, key, tool),
                Tool::ReadNamed { .. } => unreachable!("named reads resolved before deciding"),
                tool @ (Tool::Change { .. } | Tool::Goal { .. }) => {
                    if let Tool::Goal { advisor, agree, .. } = &tool
                        && let Some(words) = super::goal::refusal(domain, key.task, *advisor, *agree)
                    {
                        super::host_route::rendered(
                            domain,
                            reply_to,
                            core::SettledAnswer::Host { error: true, body: words.into() },
                        );
                        return;
                    }
                    let expanded = match domain.core.tasks.delegation(key.task) {
                        Some(context) => {
                            if let Tool::Change { changes } = tool {
                                forge_route::change_batch(domain, env, key.task, &context, &changes)
                            } else {
                                let Tool::Goal { words, advisor, agree } = tool else { unreachable!("goal or change") };
                                if agree {
                                    super::goal::request_words(domain, env, decision, reply_to, key, words);
                                    return;
                                }
                                match super::goal::goal(domain, env, key.task, &context, words, advisor, None) {
                                    Ok(member) => Ok(Box::from([member])),
                                    Err(why) => Err(forge_route::ChangeRefused::Refusal(why)),
                                }
                            }
                        }
                        None => Err(forge_route::ChangeRefused::Refusal(tasks::Refusal::Unknown)),
                    };
                    let tool = match expanded {
                        Ok(batch) => Tool::Delegate { batch },
                        Err(forge_route::ChangeRefused::Refusal(why)) => Tool::Rejected(why),
                        Err(forge_route::ChangeRefused::Words(body)) => {
                            super::host_route::rendered(
                                domain,
                                reply_to,
                                core::SettledAnswer::Host { error: true, body },
                            );
                            return;
                        }
                    };
                    let expanded = super::checked_call(
                        crate::Call { completion: key.completion, position: key.position, tool },
                        &env.limits,
                    );
                    let Tool::Delegate { batch } = expanded.tool else {
                        let Tool::Rejected(why) = expanded.tool else { unreachable!("expanded delegation shape") };
                        domain.work.push(Work::Core(core::Event::NamedAnswer {
                            to: reply_to,
                            key,
                            part: core::CallPart::DelegationRefused(tasks::Problem::new(None, why)),
                        }));
                        return;
                    };
                    delegate_call(domain, env, decision, reply_to, key, batch, false, Box::new([]));
                }
                Tool::Unavailable => {
                    domain.work.push(Work::Core(core::Event::NamedAnswer {
                        to: reply_to,
                        key,
                        part: core::CallPart::Unavailable,
                    }));
                }
                Tool::RejectedNote(why) => domain.work.push(Work::Core(core::Event::NamedAnswer {
                    to: reply_to,
                    key,
                    part: core::CallPart::NoteRefused(why),
                })),
                Tool::Note { entry, recalled } => domain.work.push(Work::Core(core::Event::NamedAction {
                    to: reply_to,
                    key,
                    action: core::NamedAction::Note { entry, recalled },
                })),
                Tool::NoteLines { entry, recalled, range } => domain.work.push(Work::Core(core::Event::NamedAction {
                    to: reply_to,
                    key,
                    action: core::NamedAction::NoteLines { entry, recalled, range },
                })),
                Tool::Recall { by, page } => domain.work.push(Work::Core(core::Event::NamedAction {
                    to: reply_to,
                    key,
                    action: core::NamedAction::Recall { by, page },
                })),
                Tool::Rejected(why) => domain.work.push(Work::Core(core::Event::NamedAnswer {
                    to: reply_to,
                    key,
                    part: core::CallPart::DelegationRefused(tasks::Problem::new(None, why)),
                })),
                Tool::RejectedMessage(why) => domain.work.push(Work::Core(core::Event::NamedAnswer {
                    to: reply_to,
                    key,
                    part: core::CallPart::MessageRefused(tasks::Problem::new(None, why)),
                })),
                Tool::RejectedControl(why) => domain.work.push(Work::Core(core::Event::NamedAnswer {
                    to: reply_to,
                    key,
                    part: core::CallPart::ControlRefused(tasks::Problem::new(None, why)),
                })),
                Tool::RejectedProposal(why) => domain.work.push(Work::Core(core::Event::NamedAnswer {
                    to: reply_to,
                    key,
                    part: core::CallPart::ProposalRefused(tasks::Problem::new(None, why)),
                })),
                Tool::Delegate { batch } => {
                    delegate_call(domain, env, decision, reply_to, key, batch, false, Box::new([]));
                }
                Tool::Propose { action, reason, as_holder } => match action {
                    ProposedAction::Effect { repository, resource, write } => forge_route::effect_call(
                        domain,
                        env,
                        decision,
                        reply_to,
                        key,
                        repository,
                        resource,
                        *write,
                        Some((reason, as_holder)),
                    ),
                    action @ (ProposedAction::Batch(_)
                    | ProposedAction::Amend { .. }
                    | ProposedAction::Widen { .. }
                    | ProposedAction::Release { .. }) => {
                        proposals::propose_call(domain, reply_to, key, action, reason, as_holder);
                    }
                },
                Tool::Decide { proposer, proposal, decision: choice } => {
                    domain.work.push(Work::Core(core::Event::NamedAction {
                        to: reply_to,
                        key,
                        action: core::NamedAction::DecideProposal {
                            proposer,
                            proposal,
                            choice: match choice {
                                ProposalChoice::Accept => core::ProposalChoice::Accept,
                                ProposalChoice::Reject { reason } => core::ProposalChoice::Reject { reason },
                                ProposalChoice::Pass => core::ProposalChoice::Pass,
                            },
                        },
                    }));
                }
                Tool::Withdraw { proposal } => {
                    domain.work.push(Work::Core(core::Event::NamedAction {
                        to: reply_to,
                        key,
                        action: core::NamedAction::WithdrawProposal { proposal },
                    }));
                }
                Tool::DecideEscalation { task, revision, decision: choice } => {
                    domain.work.push(Work::Core(core::Event::NamedAction {
                        to: reply_to,
                        key,
                        action: core::NamedAction::DecideEscalation {
                            task,
                            revision,
                            choice: match choice {
                                EscalationChoice::Release => core::EscalationChoice::Release,
                                EscalationChoice::Reject { reason } => core::EscalationChoice::Reject { reason },
                                EscalationChoice::Pass => core::EscalationChoice::Pass,
                            },
                        },
                    }));
                }
                Tool::Amend { keep_parameters, target, revision, amendment } => {
                    domain.work.push(Work::Core(core::Event::NamedAction {
                        to: reply_to,
                        key,
                        action: core::NamedAction::Amend { keep_parameters, target, revision, amendment },
                    }));
                }
                Tool::Cancel { target, reason } => {
                    domain.work.push(Work::Core(core::Event::NamedAction {
                        to: reply_to,
                        key,
                        action: core::NamedAction::Control {
                            cancel: tasks::Cancel { task: target, reason, grace: None, message: None },
                        },
                    }));
                }
                Tool::Release { target, revision, again } => {
                    domain.work.push(Work::Core(core::Event::NamedAction {
                        to: reply_to,
                        key,
                        action: core::NamedAction::Release { target, revision, again },
                    }));
                }
                Tool::Message { target, form, words } => {
                    let kind = match form {
                        MessageForm::Words => {
                            if let Some(words) = forge_route::own_agents_words(domain, key.task, target) {
                                super::host_route::rendered(
                                    domain,
                                    reply_to,
                                    core::SettledAnswer::Host { error: true, body: words.into() },
                                );
                                return;
                            }
                            tasks::MessageKind::Words
                        }
                        MessageForm::Question => tasks::MessageKind::Question,
                        MessageForm::Answer { question } => tasks::MessageKind::Answer { question },
                    };
                    domain.work.push(Work::Core(core::Event::NamedAction {
                        to: reply_to,
                        key,
                        action: core::NamedAction::Message { target, kind, words },
                    }));
                }
                Tool::Introduce { left, right } => {
                    domain.work.push(Work::Core(core::Event::NamedAction {
                        to: reply_to,
                        key,
                        action: core::NamedAction::Introduce { left, right },
                    }));
                }
                Tool::Subscribe { kind } => domain.work.push(Work::Core(core::Event::NamedAction {
                    to: reply_to,
                    key,
                    action: core::NamedAction::Subscribe { kind },
                })),
                Tool::SubscribeNamedForge { repository, topic, own_change, paths } => {
                    forge_route::subscribe_named_call(
                        domain, env, decision, reply_to, key, repository, topic, own_change, paths,
                    );
                }
                Tool::SubscribeForge { topic, own_change, paths } => {
                    forge_route::subscribe_call(domain, env, decision, reply_to, key, topic, own_change, paths);
                }
                Tool::ReadForge { repository, read } => {
                    forge_route::read_call(domain, env, decision, reply_to, key, repository, read);
                }
                Tool::EffectForge { repository, resource, write } => {
                    forge_route::effect_call(domain, env, decision, reply_to, key, repository, resource, *write, None);
                }
                Tool::Unsubscribe { subscription } => {
                    let token = reply_to.into_token();
                    if let Some(topic) = domain.forge.subscription(key.task, subscription) {
                        assert!(
                            domain.forge_unsubscribing.insert(token, (key.task, topic)) == Ok(None),
                            "one connector unsubscribe"
                        );
                    }
                    domain.work.push(Work::Core(core::Event::NamedAction {
                        to: ReplyTo::new(token),
                        key,
                        action: core::NamedAction::Unsubscribe { subscription },
                    }));
                }
            }
        }
    }
}

/// The core's successful newest-revision read supplies the document and family check;
/// the connector supplies only its own value (jig's domain/engine/settings.md, 5).
pub(super) fn settings_value(
    domain: &Domain,
    answer: &core::SettingsAnswer,
) -> Option<temper_engine_domain_forge::settings::SizedValue> {
    let core::SettingsAnswer::Setting(document) = answer else { return None };
    let core::settings::Setting::Application { kind, scope, number } = document.setting else { return None };
    if document.revision != domain.core.current.newest() {
        return None;
    }
    domain.forge.setting(kind, scope, number).cloned()
}

fn propose_settings(domain: &mut Domain, to: ReplyTo, key: crate::CallKey, tool: Tool) {
    super::settings_parts::call(domain, to, key, tool);
}
