//! Attempts follow the phase that names them (domain/tasks/lifecycle.md, 12.1).
use crate::{Active, AttemptRow, OpenAttempt, Phase, Request, Stored, TaskRecord};
use skein_lib::{Queue, Wall};

pub(crate) fn names(phase: &Phase) -> Option<u64> {
    crate::run_attempt(phase)
}

pub(crate) fn changed(record: &mut TaskRecord, at: Wall, out: &mut Queue<Request>) {
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
                let open = OpenAttempt {
                    attempt,
                    claimed: Some(at),
                    started: if running { Some(at) } else { None },
                    from: record.milestone,
                    end: None,
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
        }),
    });
}
