//! Core dispatch of the in-process connector port (jig/docs/design/domain/engine/vocabulary.md, 2.5).

use super::{
    Work, application_routes, children, effect::procedure::ProcedureCall, party, run::delegation, stopping, task, view,
};
use crate::{Core, Limits, Request, judgements::Subject};
use alloc::boxed::Box;
use jig_domain_contract::{Up, lifecycle, parts, procedures, projections, requests, resources, settings, topics};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{Env, Queue};

#[expect(clippy::too_many_lines, reason = "exhaustive cases route each capability to its owner")]
pub(super) fn step(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Work>, source: u16, up: Up) -> Queue<Request> {
    match up {
        Up::Lifecycle(up) => match up {
            lifecycle::Up::Done { .. } => stopping::empty(), // catch-up reports go to the start
            lifecycle::Up::Released { project } => crate::admission::deletion::released(core, env, source, project),
            lifecycle::Up::Adopted { project, name, role, home } => {
                assert_resource_phase(core, name.connector);
                crate::effects::adopted(core, &env.limits, project, name, role, home);
                stopping::empty()
            }
        },
        Up::Resources(up) => match up {
            resources::Up::Holdings { request, holdings, refused } => {
                core.judgements.answered(source, Subject::Holdings(request));
                party::holdings::route_holdings(core, env, work, request, source, holdings, refused)
            }
            resources::Up::DelegateHoldings { request, holdings, refused } => {
                core.judgements.answered(source, Subject::DelegateHoldings(request));
                delegation::delegate_holdings(core, env, work, request, source, holdings, refused)
            }
            resources::Up::ProcedureHoldings { task, step, holdings, refused } => {
                core.judgements.answered(source, Subject::ProcedureHoldings { task, step });
                if core.stopping || core.settling {
                    return stopping::empty();
                }
                delegation::procedure_holdings(
                    core,
                    env,
                    work,
                    ProcedureCall { task, step, connector: source },
                    holdings,
                    refused,
                )
            }
            resources::Up::Resource { project, name, role, hold } => {
                assert_resource_phase(core, name.connector);
                crate::effects::resource(core, &env.limits, work, project, name, role, hold);
                stopping::empty()
            }
            resources::Up::PoolSlots { name, slots } => {
                work.push(Work::Event(crate::Event::Tasks(tasks::Event::Slots { pool: name, slots })));
                stopping::empty()
            }
            resources::Up::Drift { task, resource } => {
                work.push(Work::Event(crate::Event::Tasks(tasks::Event::HoldFrom {
                    task,
                    why: tasks::Hold::Drift,
                    source: Box::new(tasks::HoldSource::Resource(resource)),
                })));
                stopping::empty()
            }
            resources::Up::ReadAfresh { resource } => {
                task_event(core, env, work, tasks::Event::ReadAfresh { resource })
            }
        },
        Up::Effects(up) => crate::effects::up(core, env, work, source, up),
        Up::Topics(topics::Up::News { task, subscription, class, words, topic: _ }) => {
            if let Some(event) = core.connector_news(task, subscription, class, words, env.wall) {
                work.push(Work::Event(crate::Event::Tasks(event)));
            }
            stopping::empty()
        }
        Up::Parts(up) => match up {
            parts::Up::Made { watcher, outcome } => {
                view::routes::route_view(core, env, views::Event::Made { watcher, outcome })
            }
            parts::Up::Lines { token, parts } => crate::reads::parts::answered(core, env, source, token, parts),
            parts::Up::Public { reply, answer } => {
                let token = reply.into_token();
                core.judgements.answered(source, Subject::Public(token));
                crate::public::reads::answered(core, token, answer)
            }
            parts::Up::Marked { tasks } => {
                view::part_marks::records(core, source, &tasks);
                stopping::empty()
            }
        },
        Up::Procedures(up) => match up {
            procedures::Up::Step { task, step, code, decision } => {
                if core.stopping || core.settling {
                    return stopping::empty();
                }
                delegation::procedure_step(
                    core,
                    env,
                    work,
                    ProcedureCall { task, step, connector: source },
                    code,
                    decision,
                )
            }
            procedures::Up::Waits { task, waits } => crate::runs::waits::report(core, env, source, task, waits),
            procedures::Up::Wake { task } => task_event(core, env, work, tasks::Event::WakeProcedure { task }),
        },
        Up::Projections(up) => match up {
            projections::Up::Settled { goal } => children::route_projection_settled(core, goal, source),
            projections::Up::Outbox { entry, outcome } => {
                crate::effects::projection_outbox(core, work, source, entry, outcome);
                stopping::empty()
            }
        },
        Up::Settings(up) => match up {
            settings::Up::Unfit { .. } => stopping::empty(),
            settings::Up::Judged { owner, judged } => {
                core.judgements.answered(source, Subject::SettingsJudge(owner));
                crate::settings::application::judged(core, env, work, owner, judged)
            }
            settings::Up::Restored { owner, judged } => {
                core.judgements.answered(source, Subject::SettingsRestore(owner));
                super::settings::restore::restored(core, env, work, owner, judged)
            }
            settings::Up::Defaults { defaults, dropped } => {
                crate::settings::application::defaults(core, source, defaults, dropped);
                stopping::empty()
            }
        },
        Up::Requests(up) => match up {
            requests::Up::Decided { request, outcome } => {
                application_routes::finished(core, env, work, request, outcome)
            }
            requests::Up::Marked { subjects } => view::routes::marked(core, source, subjects),
        },
        Up::Act { key, act, carried } => {
            let key = people::ActKey { connector: source, key };
            if core.stopping {
                return party::deployment_acts::answer(
                    None,
                    key,
                    carried,
                    people::Outcome::Again(people::Again::NotReady),
                );
            }
            party::deployment_acts::start(core, env, work, None, key, act, carried)
        }
    }
}

pub(crate) fn task_event(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    event: tasks::Event,
) -> Queue<Request> {
    let mut out = task::route_tasks(core, env, work, event);
    crate::admission::deletion::advance(core, env, &mut out);
    out
}

fn assert_resource_phase(core: &Core, connector: u16) {
    if let Some(phase) = core.start_phase() {
        assert!(super::takes_connector_resource(phase, connector), "connector resources load before Link");
    }
}
