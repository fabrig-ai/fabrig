//! The core's held outputs, each released after its decision commits
//! (jig's `domain/root.md`, 4; `testing/jig-test-domain/src/routes.rs`).
use super::{Decision, Delivery, Domain, Env, Limits, Token, emit, forge_route, host_route, relay_call};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;

#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher releases every held core output")]
pub(crate) fn held(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, held: core::Held) {
    let held = match jig_domain_hub_link::down(&mut domain.hub_link, held) {
        Ok((channel, down)) => {
            emit(decision, &env.limits, Delivery::Link { channel, down });
            return;
        }
        Err(held) => held,
    };
    match held {
        core::Held::NeedArrived { party, project, needs, .. } => {
            let name = domain.engine.core().project_name(project).expect("a need belongs to a project");
            let (topic, payload) = crate::push::words(project, needs, name);
            for push in domain.forge.pushes(party, &topic, &payload) {
                emit(decision, &env.limits, Delivery::Push(push));
            }
        }
        core::Held::PublicRead { .. } => {}
        core::Held::Restarting { wait } => emit(decision, &env.limits, Delivery::Restarting { wait }),
        core::Held::PublicWait { read } => emit(decision, &env.limits, Delivery::PublicWait { read }),
        core::Held::ReadUsage { channel, read, source } => {
            emit(decision, &env.limits, Delivery::Link { channel, down: host::LinkDown::ReadUsage { read, source } });
        }
        core::Held::ReadList { channel, read, endpoint, grant } => {
            emit(
                decision,
                &env.limits,
                Delivery::Link { channel, down: host::LinkDown::ReadList { read, endpoint, grant } },
            );
        }
        core::Held::Sources { channel, sources } => {
            emit(decision, &env.limits, Delivery::Link { channel, down: host::LinkDown::Sources { sources } });
        }
        core::Held::PauseWorker { channel, mode } => {
            emit(decision, &env.limits, Delivery::Link { channel, down: host::LinkDown::Pause { mode } });
        }
        core::Held::CheckSource { channel, name } => {
            emit(decision, &env.limits, Delivery::Link { channel, down: host::LinkDown::Check { name } });
        }
        core::Held::SignOutSource { channel, name } => {
            emit(decision, &env.limits, Delivery::Link { channel, down: host::LinkDown::SignOut { name } });
        }
        core::Held::SourceSignedOut { to, channel, name, result } => {
            emit(decision, &env.limits, Delivery::SourceSignedOut { to, channel, name, result });
        }

        core::Held::SettledCall { to, call, .. } => {
            host_route::settled(domain, &env.limits, decision, to, call);
        }
        core::Held::Relayed { channel, run, attempt, call, answer } => {
            let delivery = host_route::relayed(domain, channel, run, attempt, call, answer);
            emit(decision, &env.limits, delivery);
        }
        core::Held::Inbound { channel, run, attempt, message } => {
            let delivery =
                crate::link::delivery(domain, jig_domain_hub_link::Event::Inbound { channel, run, attempt, message });
            emit(decision, &env.limits, delivery);
        }
        core::Held::Relay { task, attempt, previous, word } => {
            emit(decision, &env.limits, Delivery::Relay { task, attempt, previous, word });
        }
        core::Held::PeopleReply { to, reply } => {
            let token = to.into_token();
            let to = skein_lib::ReplyTo::new(token);
            if domain.playground_replies.contains_key(&token) {
                super::playground::reply(domain, env, decision, token, reply);
            } else {
                emit(decision, &env.limits, Delivery::WebReply { to, reply });
            }
        }
        core::Held::CarriedUnused { number } => emit(decision, &env.limits, Delivery::CarriedUnused { number }),
        core::Held::CallAnswer { to, key, part } => {
            let answer = forge_route::effect_answer(domain, key, &part);
            relay_call(domain, &env.limits, decision, to, answer);
        }
        core::Held::Result { person, task, words } => {
            emit(decision, &env.limits, Delivery::Result { person, task, words });
        }
        core::Held::Assign { channel, run, attempt, .. } => {
            let delivery = crate::link::delivery(domain, jig_domain_hub_link::Event::Assign { channel, run, attempt });
            emit(decision, &env.limits, delivery);
        }
        core::Held::Grant { channel, run, attempt, grant } => {
            emit(
                decision,
                &env.limits,
                Delivery::Link { channel, down: host::LinkDown::Grant { run, attempt, grant } },
            );
        }
        core::Held::Acknowledge { channel, run, attempt } => {
            emit(decision, &env.limits, Delivery::Link { channel, down: host::LinkDown::Acknowledge { run, attempt } });
        }
        core::Held::TaskTerminalAcknowledged { task, attempt } => {
            emit(
                decision,
                &env.limits,
                Delivery::Fleet(fleet::Event::Acknowledge { run: Token::new(task), attempt: Token::new(attempt) }),
            );
        }
        core::Held::TaskPartKept { key } => emit(
            decision,
            &env.limits,
            Delivery::Fleet(fleet::Event::PartKept {
                run: Token::new(key.task),
                attempt: Token::new(key.attempt),
                part: key.number,
            }),
        ),
        core::Held::AcknowledgePart { channel, run, attempt, part } => emit(
            decision,
            &env.limits,
            Delivery::Link { channel, down: host::LinkDown::PartKept { run, attempt, part } },
        ),
        core::Held::PartBusy { channel, run, attempt, part } => {
            emit(decision, &env.limits, Delivery::PartBusy { channel, task: run.raw(), attempt: attempt.raw(), part });
        }
        core::Held::TaskTurnKept { task, attempt, turn } => {
            emit(
                decision,
                &env.limits,
                Delivery::Fleet(fleet::Event::TurnKept { run: Token::new(task), attempt: Token::new(attempt), turn }),
            );
        }
        core::Held::AcknowledgeTurn { channel, run, attempt, turn } => {
            emit(
                decision,
                &env.limits,
                Delivery::Link { channel, down: host::LinkDown::TurnKept { run, attempt, turn } },
            );
        }
        core::Held::Cancel { channel, run, attempt } => {
            emit(decision, &env.limits, Delivery::Link { channel, down: host::LinkDown::Cancel { run, attempt } });
        }
        core::Held::Refuse { channel, hello } => emit(decision, &env.limits, Delivery::Refuse { channel, hello }),
        core::Held::TurnBusy { channel, run, attempt, turn } => {
            emit(decision, &env.limits, Delivery::TurnBusy { channel, task: run.raw(), attempt: attempt.raw(), turn });
        }
        core::Held::StopRun { task, attempt } => {
            emit(decision, &env.limits, Delivery::Fleet(core::Core::stop_run(task, attempt)));
        }
        core::Held::Store(_) => {
            unreachable!("store loads cross the engine port (jig/docs/design/domain/engine/vocabulary.md, 2.4)")
        }
        core::Held::Read { reply, answer } => super::kinds::revision_answer(domain, env, decision, reply, answer),
        core::Held::Delivery { watcher, document } => {
            emit(decision, &env.limits, Delivery::Document { watcher, document });
        }
        core::Held::WatchEnded { watcher, why } | core::Held::ApplicationWatchEnded { watcher, why, .. } => {
            emit(decision, &env.limits, Delivery::WatchEnded { watcher, why });
        }
        core::Held::Drained => emit(decision, &env.limits, Delivery::Drained),
        core::Held::Applied { to, token, .. } => {
            let right = to.into_token();
            let to = skein_lib::ReplyTo::new(right);
            if domain.playground_replies.contains_key(&right) {
                let outcome = domain.forge.application_answer(token);
                super::playground::adopted(domain, right, outcome);
            } else {
                emit(
                    decision,
                    &env.limits,
                    Delivery::Application { to, outcome: super::applications::answer(domain, token) },
                );
            }
        }
        core::Held::ApplicationDelivery { .. } => {
            unreachable!("application delivery is taken where it is made (panels.rs)")
        }
        core::Held::NotesWritten { .. } | core::Held::NotesDeleted { .. } => {
            unreachable!("the current application has no note caller")
        }
        core::Held::Connector(_, _) => unreachable!("connector releases use the engine port"),
        core::Held::Acted { to, outcome } => {
            let token = to.into_token();
            if domain.playground_replies.contains_key(&token) {
                super::playground::acted(domain, env, decision, token, outcome);
            } else {
                unreachable!("act reply belongs to the playground");
            }
        }
    }
}
