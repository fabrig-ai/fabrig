//! Attempts follow the phase that names them (domain/tasks/lifecycle.md, 12.1).
use crate::{
    Active, AttemptRow, ClaimSources, Domain, Guide, Limits, OpenAttempt, Phase, Request, RunSources, Stored,
    TaskRecord,
};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Wall, usize_of};

pub(crate) fn names(phase: &Phase) -> Option<u64> {
    crate::run_attempt(phase)
}

pub(crate) fn changed(record: &mut TaskRecord, at: Wall, prepared: Option<ClaimSources>, out: &mut Queue<Request>) {
    let keeps_suspension = match record.phase {
        Phase::Active(Active::Due | Active::Preparing) => true,
        Phase::Waiting
        | Phase::Active(Active::Idle | Active::Claimed { .. } | Active::Running { .. } | Active::BackingOff { .. })
        | Phase::Held { .. }
        | Phase::Closing(_)
        | Phase::Ended(_) => false,
    };
    if !keeps_suspension && let Some(suspension) = record.suspension.take() {
        record.suspended = record.suspended.saturating_add(at.as_nanos().saturating_sub(suspension.since.as_nanos()));
    }
    let next = names(&record.phase);
    let closes = match &record.open_attempt {
        Some(open) => Some(open.attempt) != next,
        None => false,
    };
    if closes {
        let open = record.open_attempt.take().expect("open attempt closes");
        let phases = if open.from == 0 {
            None
        } else {
            Some(
                u32::try_from(record.milestone.checked_sub(open.from).expect("attempt positions ordered"))
                    .expect("attempt phases fit"),
            )
        };
        save(record.number, &open, Some(at), phases, out);
    }
    let running = match record.phase {
        Phase::Active(Active::Running { .. }) => true,
        Phase::Waiting
        | Phase::Active(
            Active::Idle | Active::Due | Active::Preparing | Active::Claimed { .. } | Active::BackingOff { .. },
        )
        | Phase::Held { .. }
        | Phase::Closing(_)
        | Phase::Ended(_) => false,
    };
    if let Some(attempt) = next {
        match &mut record.open_attempt {
            None => {
                let (sources, guide_limit) = match prepared {
                    Some(prepared) => (
                        Some(RunSources {
                            build: prepared.build,
                            settings: prepared.settings,
                            charters: record.charters,
                            source: prepared.source,
                            guides: None,
                        }),
                        prepared.guide_limit,
                    ),
                    None => (None, 0),
                };
                let open = OpenAttempt {
                    attempt,
                    claimed: Some(at),
                    started: if running { Some(at) } else { None },
                    from: record.milestone,
                    end: None,
                    sources,
                    guide_limit,
                };
                save(record.number, &open, None, None, out);
                record.open_attempt = Some(open);
            }
            Some(open) => {
                if running && open.started.is_none() && open.from != 0 {
                    open.started = Some(at);
                    save(record.number, open, None, None, out);
                }
            }
        }
    }
}

pub(crate) fn report(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    guides: Box<[Guide]>,
    out: &mut Queue<Request>,
) {
    if !domain.ready() {
        return;
    }
    let Some(live) = crate::domain::task_mut(domain, task) else { return };
    let Some(open) = &mut live.record.open_attempt else { return };
    if open.attempt != attempt || guides.len() > usize_of(open.guide_limit.min(crate::GUIDES)) {
        return;
    }
    let Some(sources) = &mut open.sources else { return };
    if sources.guides.is_some() {
        return;
    }
    for guide in &guides {
        if guide.workspace.len() > crate::GUIDE_TEXT_BYTES
            || guide.commit.len() > crate::GUIDE_TEXT_BYTES
            || guide.path.len() > crate::GUIDE_TEXT_BYTES
        {
            return;
        }
    }
    sources.guides = Some(guides);
    save(task, open, None, None, out);
    crate::domain::save_live(domain, env, task, out);
}

fn save(task: u64, open: &OpenAttempt, ended: Option<Wall>, phases: Option<u32>, out: &mut Queue<Request>) {
    out.push(Request::Save {
        record: Stored::Attempt(AttemptRow {
            task,
            attempt: open.attempt,
            claimed: open.claimed,
            started: open.started,
            ended,
            end: open.end,
            phases,
            sources: open.sources.clone(),
        }),
    });
}
