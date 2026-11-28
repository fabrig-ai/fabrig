//! The engine's start observations, for temper's admission and opening hand-offs.
use super::{Domain, Env, Limits};
use jig_domain_core::{engine::application::Out, start::Phase};

pub(super) fn record(domain: &mut Domain, env: &Env<Limits>, report: Out) {
    match report {
        Out::Numbered { commits } => {
            assert!(domain.restored_commits.replace(commits).is_none(), "one deployment header");
        }
        Out::Start { phase } => {
            domain.start_phase = phase;
            domain.start_report = if phase.is_some() { Some(super::forge::Report::new(env.now)) } else { None };
            if let Some(phase) = phase {
                #[cfg(test)]
                domain.start_phases.push(phase);
                if phase == Phase::Open {
                    calendar(domain, env);
                }
            } else {
                domain.opened = true;
                if domain.rehearsal.is_some() {
                    domain.rehearsal = Some(true);
                }
                super::trials::restarted(domain);
            }
        }
        Out::Refused { phase, failure } => {
            domain.stop_start(crate::StartStop::Core { phase: Some(phase), failure });
        }
    }
}

fn calendar(domain: &mut Domain, env: &Env<Limits>) {
    for index in 0..domain.engine.core().projects.len() {
        let project = *domain.engine.core().projects.get(index).expect("bounded core project");
        let period = domain.engine.core().period(project);
        if let Some(row) = domain.engine.core().tasks.funding(jig_domain_tasks::Funder::Period { project, period })
            && let Some(times) = row.period
        {
            super::calendar::route(
                domain,
                env,
                temper_engine_domain_calendar::Event::Restored {
                    project,
                    period,
                    began: times.began,
                    next: times.next,
                },
                None,
            );
        }
    }
}
