//! The forge's edge on the engine's connector port (jig/docs/design/domain/engine/vocabulary.md, 2.5).
use alloc::boxed::Box;
use jig_domain_contract::{
    Ask, Released, Up, effects, lifecycle, parts, procedures, projections, requests, resources, settings, topics,
};
use skein_lib::{Env, List, Queue};

use super::{Domain, Event, Limits, ReleaseEnding, Request, adoption, committed, decide};
use crate::settings as kinds;

pub(super) fn ask(d: &mut Domain, env: &Env<Limits>, ask: Ask, out: &mut Queue<Request>) {
    match ask {
        Ask::Lifecycle(lifecycle::Ask::CatchUp) => decide(d, env, Event::CatchUp, out),
        Ask::Lifecycle(lifecycle::Ask::ReleaseProject { project }) => adoption::release(d, env, project, out),
        Ask::Lifecycle(lifecycle::Ask::Adopt { .. }) => unreachable!("the root seeds an adoption's parties"),
        Ask::Resources(ask) => resource(d, env, ask, out),
        Ask::Effects(ask) => effect(d, env, ask, out),
        Ask::Topics(topics::Ask::Ended { task, subscription }) => {
            if let Some(topic) = d.subscription(task, subscription) {
                super::unsubscribe(d, env, task, topic, out);
            }
        }
        Ask::Topics(
            topics::Ask::Subscribed { .. } | topics::Ask::Unsubscribed { .. } | topics::Ask::Refused { .. },
        ) => {
            unreachable!("the root keeps subscription flights and replies")
        }
        Ask::Parts(
            parts::Ask::DropDocument { .. }
            | parts::Ask::Plan(_)
            | parts::Ask::Abandon { .. }
            | parts::Ask::Gather { .. }
            | parts::Ask::Cut { .. }
            | parts::Ask::Drop { .. },
        ) => {}
        Ask::Parts(
            parts::Ask::Document { .. } | parts::Ask::Read { .. } | parts::Ask::Lines(_) | parts::Ask::Public { .. },
        ) => {
            unreachable!("the root assembles the answer and owns its delivery")
        }
        Ask::Procedures(procedures::Ask::WaitsRefused { .. }) => {
            unreachable!("wait refusal is routed before the drain")
        }
        Ask::Procedures(
            procedures::Ask::Start { .. } | procedures::Ask::StepMade { .. } | procedures::Ask::StepRefused { .. },
        ) => {
            unreachable!("a procedure's admission and step read the root's core")
        }
        Ask::Projections(projections::Ask::Forget { goal }) => decide(d, env, Event::ForgetProjection { goal }, out),
        Ask::Projections(projections::Ask::Project { .. }) => {
            unreachable!("a projection draws on the root's plan and counters")
        }
        Ask::Settings(ask) => setting(d, env, ask, out),
        Ask::Requests(requests::Ask::PartAnswered { request, outcome }) => {
            adoption::answered(d, env, request, outcome, out);
        }
        Ask::Requests(requests::Ask::Decide { .. } | requests::Ask::Drop { .. }) => {
            unreachable!("the root keeps prepared applications")
        }
    }
}

fn resource(d: &mut Domain, env: &Env<Limits>, ask: resources::Ask, out: &mut Queue<Request>) {
    match ask {
        resources::Ask::Unname { task } => decide(d, env, Event::Unnamed { task }, out),
        resources::Ask::Hold { task, resource, entry } => {
            if let Some(name) = crate::routing::forge_name(resource.connector, &resource, env.limits.name_bytes) {
                let from = match d.hold(&name) {
                    Some(row) if row.task != task => Some(row.task),
                    Some(_) | None => None,
                };
                super::hold(d, env, entry, task, name, from, out);
            }
        }
        resources::Ask::Lost { task, attempt, .. } => super::lost(d, env, task, attempt, out),
        resources::Ask::Holdings { .. }
        | resources::Ask::DelegateHoldings { .. }
        | resources::Ask::ProcedureHoldings { .. } => {
            unreachable!("holdings use the root's ancestry, roles and tracked tasks")
        }
        resources::Ask::RepairRefused { .. } | resources::Ask::DelegateRefused { .. } => {
            unreachable!("refusals use the root's admission flights and task holds")
        }
    }
}

fn effect(d: &mut Domain, env: &Env<Limits>, ask: effects::Ask, out: &mut Queue<Request>) {
    match ask {
        effects::Ask::Resolve { entry, again } => super::resolve(d, env, entry, again, out),
        effects::Ask::DropProposal { proposal } => decide(d, env, Event::DropProposedEffect { number: proposal }, out),
        effects::Ask::Close { task, root, ending } => {
            let ending = match ending {
                jig_domain_tasks::Ending::Done(_) => ReleaseEnding::Done,
                jig_domain_tasks::Ending::Failed { .. } => ReleaseEnding::Failed,
                jig_domain_tasks::Ending::Cancelled { .. } => ReleaseEnding::Cancelled,
            };
            super::settle_effects(d, env, task, root, ending, out);
        }
        effects::Ask::Describe { .. }
        | effects::Ask::DescribeProposal { .. }
        | effects::Ask::Judge { .. }
        | effects::Ask::Keep { .. }
        | effects::Ask::KeepProposal { .. }
        | effects::Ask::Drop { .. }
        | effects::Ask::Release { .. } => unreachable!("the root supplies the effect's core context"),
    }
}

pub(super) fn not_made(entry: u64, task: u64) -> Request {
    Request::Up(Up::Effects(effects::Up::Outbox { entry, task, outcome: effects::OutboxOutcome::NotMade }))
}

pub(super) fn released(d: &mut Domain, env: &Env<Limits>, released: Released, out: &mut Queue<Request>) {
    match released {
        Released::Make { entry } => committed(d, env, entry, out),
        Released::Answered { .. } => {} // The application's own acts use reply tokens instead.
    }
}

fn setting(d: &mut Domain, env: &Env<Limits>, ask: settings::Ask, out: &mut Queue<Request>) {
    match ask {
        settings::Ask::Judge { owner, items, .. } => {
            let mut judged = List::uncapped();
            let bounds = setting_bounds(&env.limits);
            for item in &items {
                let answer = match item.edit {
                    settings::Edit::Remove => {
                        let asked = kinds::Asked {
                            kind: item.kind,
                            scope: item.scope,
                            number: item.number,
                            edit: kinds::Edit::Remove,
                        };
                        let answers = d.judge_settings(&bounds, core::slice::from_ref(&asked));
                        judgement(answers.first().expect("one forge judgement per item").clone())
                    }
                    settings::Edit::Set(_) | settings::Edit::Reset => {
                        unreachable!("the root supplies prepared values and checks reset policy")
                    }
                };
                judged.push(answer).expect("one judgement per item");
            }
            out.push(Request::Up(Up::Settings(settings::Up::Judged { owner, judged: judged.into_boxed() })));
        }
        settings::Ask::ForgetReplaced { before } => d.forget_replaced(before, out),
        settings::Ask::DropProposal { proposal } => d.drop_proposal_settings(proposal, out),
        settings::Ask::SaveDefaults { revision } => {
            if let Some(revision) = revision {
                d.remember_settings_defaults(revision, out);
            }
            d.save_settings_defaults(out);
        }
        settings::Ask::Keep { .. } | settings::Ask::KeepProposal { .. } | settings::Ask::Restore { .. } => {
            unreachable!("the root owns prepared settings and their core checks and deliveries")
        }
    }
}

fn setting_bounds(limits: &Limits) -> kinds::Bounds {
    // A removal has no setting value: it uses the kind's scope and number bounds only.
    kinds::Bounds {
        landing_members: limits.landing_members,
        rerun_processes: 0,
        repositories: limits.repositories,
        landing_gates: limits.gates,
        tiers: limits.tiers.tiers,
        ignored_paths: limits.ignored_paths,
        landing_rules: limits.landing_rules,
        gates: limits.change_policy.gates,
        roles: limits.roles,
        segments: limits.authority_segments,
        segment_bytes: limits.name_bytes,
        follows: limits.follows,
        dependencies: limits.dependencies,
        follow_paths: limits.follow_paths,
        name_bytes: limits.name_bytes,
    }
}

fn judgement(answer: kinds::Judged) -> settings::Judged {
    match answer {
        kinds::Judged::Accept { size, charter } => settings::Judged::Accept {
            size,
            charters: match charter {
                Some(charter) => Box::new([charter]),
                None => Box::default(),
            },
        },
        kinds::Judged::Same => settings::Judged::Same,
        kinds::Judged::Refuse(why) => settings::Judged::Refuse { why: why.words() },
    }
}
