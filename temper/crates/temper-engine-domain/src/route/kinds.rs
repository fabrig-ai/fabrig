//! Typed settings handoffs within the root's decision
//! (temper/docs/design/domain/root.md, 5.6;
//! temper/docs/design/protocol/configuration.md, 13.2).
use alloc::boxed::Box;
use jig_domain_core::{Event, effects::connector, settings::application};
use skein_lib::{Env, List, Queue};
use temper_engine_domain_forge::settings;

use crate::{Domain, Limits, Work};

#[expect(clippy::wildcard_enum_match_arm, reason = "the forge dispatcher selects only the five settings asks")]
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
                let answer = match typed(domain, proposal, item) {
                    Some(asked) => {
                        let answers = domain.forge.judge_settings(&bounds, core::slice::from_ref(&asked));
                        match *answers.first().expect("one forge judgement per item") {
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
                    None => connector::Judged::Refuse { why: settings::Refusal::Value.words().into() },
                };
                judged.push(answer).expect("one judgement per asked item");
            }
            domain.work.push(Work::Core(Event::EffectConnector(connector::Event::SettingsJudged {
                owner,
                judged: judged.into_boxed(),
            })));
        }
        connector::Ask::KeepSettings { revision: _, proposal, items } => {
            let asked = typed_items(domain, proposal, &items);
            domain.forge.keep_settings(&super::environment_forge(env), proposal, &asked, &mut out);
        }
        connector::Ask::KeepProposalSettings { proposal, items } => {
            let asked = typed_items(domain, None, &items);
            domain.forge.keep_proposal_settings(proposal, &asked, &mut out);
        }
        connector::Ask::DropProposalSettings { proposal } => domain.forge.drop_proposal_settings(proposal, &mut out),
        connector::Ask::SaveSettingsDefaults { revision } => save_defaults(revision),
        _ => unreachable!("the forge routes only its settings asks here"),
    }
    if !out.is_empty() {
        domain.work.push(Work::ForgeSettings(out));
    }
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

fn save_defaults(_revision: Option<u64>) {
    // kinds-start gives the defaults' save its body.
}
