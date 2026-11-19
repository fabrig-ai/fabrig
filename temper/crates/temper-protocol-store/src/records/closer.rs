//! Goal closer rows (temper/docs/design/domain/forge/branches.md, 2).
use super::{FITS, w};
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge_closer as c;
use temper_engine_domain_forge_vocabulary::CloserRow;

fn encode_step(step: c::Step) -> w::ClosingStep {
    match step {
        c::Step::Close => w::ClosingStep::Close,
        c::Step::Land => w::ClosingStep::Land,
    }
}
fn decode_step(step: w::ClosingStep) -> c::Step {
    match step {
        w::ClosingStep::Close => c::Step::Close,
        w::ClosingStep::Land => c::Step::Land,
    }
}
fn encode_run(run: Option<c::Running>) -> Option<w::ClosingRun> {
    let run = run?;
    Some(w::ClosingRun { step: encode_step(run.step), change: run.change })
}
fn decode_run(run: Option<w::ClosingRun>) -> Option<c::Running> {
    let w::ClosingRun { step, change } = run?;
    Some(c::Running { step: decode_step(step), change })
}
pub(super) fn encode(row: &CloserRow) -> w::Closer {
    let closed = match &row.closer.closed {
        Some(tasks) => Some(w::ClosingTasks { tasks: List::from_boxed(tasks.clone()) }),
        None => None,
    };
    let hold = match row.closer.hold {
        Some(c::Hold::ClosingStep { step, change }) => encode_run(Some(c::Running { step, change })),
        None => None,
    };
    let retry = match row.closer.retry {
        Some(step) => Some(encode_step(step)),
        None => None,
    };
    let making = match row.making {
        Some(step) => Some(encode_step(step)),
        None => None,
    };
    w::Closer {
        task: row.task,
        goal: row.goal,
        closed,
        running: encode_run(row.closer.running),
        landing: row.closer.landing,
        hold,
        retry,
        making,
        close_changes: List::from_boxed(row.close_changes.clone()),
        landing_changes: List::from_boxed(row.landing_changes.clone()),
        selections: encode_selections(&row.closer.selections),
        done: row.done,
        relay: encode_relay(&row.relay),
    }
}
pub(super) fn decode(row: w::Closer) -> CloserRow {
    let w::Closer {
        task,
        goal,
        closed,
        running,
        landing,
        hold,
        retry,
        making,
        close_changes,
        landing_changes,
        selections,
        done,
        relay,
    } = row;
    let closed = match closed {
        Some(tasks) => Some(tasks.tasks.into_boxed()),
        None => None,
    };
    let hold = match decode_run(hold) {
        Some(c::Running { step, change }) => Some(c::Hold::ClosingStep { step, change }),
        None => None,
    };
    let retry = match retry {
        Some(step) => Some(decode_step(step)),
        None => None,
    };
    let making = match making {
        Some(step) => Some(decode_step(step)),
        None => None,
    };
    CloserRow {
        task,
        goal,
        closer: c::Closer {
            closed,
            running: decode_run(running),
            landing,
            hold,
            retry,
            selections: decode_selections(selections),
        },
        making,
        close_changes: close_changes.into_boxed(),
        landing_changes: landing_changes.into_boxed(),
        relay: decode_relay(relay),
        done,
    }
}

fn encode_relay(relay: &c::relay::Relay) -> Option<w::ClosingRelay> {
    use c::relay::{Form, Stands};
    if relay.open.is_none() && relay.replies.is_empty() {
        return None;
    }
    let open = match &relay.open {
        None => None,
        Some(open) => {
            let stands = match &open.stands {
                Stands::Taken => w::ClosingStands::Taken,
                Stands::Asked { question, answer } => {
                    w::ClosingStands::Asked(w::ClosingAsked { question: *question, answer: answer.clone() })
                }
            };
            Some(w::ClosingQuestion {
                asker: open.asker,
                number: open.number,
                step: encode_step(open.step),
                words: open.words.clone(),
                stands,
            })
        }
    };
    let mut replies = List::with_capacity(u32::try_from(relay.replies.len()).expect(FITS));
    for message in &relay.replies {
        let form = match message.form {
            Form::Words => w::ClosingForm::Words,
            Form::Question => w::ClosingForm::Question,
            Form::Answer { question } => w::ClosingForm::Answer(w::Number { number: question }),
        };
        replies.push(w::ClosingMessage { target: message.target, form, words: message.words.clone() }).expect(FITS);
    }
    Some(w::ClosingRelay { open, replies })
}
fn decode_relay(relay: Option<w::ClosingRelay>) -> c::relay::Relay {
    use c::relay::{Form, Message, Question, Relay, Stands};
    let Some(relay) = relay else { return Relay::new() };
    let w::ClosingRelay { open, replies } = relay;
    let open = match open {
        None => None,
        Some(open) => {
            let w::ClosingQuestion { asker, number, step, words, stands } = open;
            let stands = match stands {
                w::ClosingStands::Taken => Stands::Taken,
                w::ClosingStands::Asked(asked) => {
                    let w::ClosingAsked { question, answer } = asked;
                    Stands::Asked { question, answer }
                }
            };
            Some(Question { asker, number, step: decode_step(step), words, stands })
        }
    };
    let mut messages = List::with_capacity(replies.len());
    for message in replies.into_boxed() {
        let w::ClosingMessage { target, form, words } = message;
        let form = match form {
            w::ClosingForm::Words => Form::Words,
            w::ClosingForm::Question => Form::Question,
            w::ClosingForm::Answer(question) => Form::Answer { question: question.number },
        };
        messages.push(Message { target, form, words }).expect(FITS);
    }
    Relay { open, replies: messages.into_boxed() }
}

fn encode_selections(selections: &[c::PullSelection]) -> List<w::ClosingSelection> {
    let mut rows = List::with_capacity(u32::try_from(selections.len()).expect(FITS));
    for selection in selections {
        let head = match selection.head {
            Some(commit) => Some(w::Commit { commit }),
            None => None,
        };
        let progress = match &selection.progress {
            c::SelectionProgress::Pull => w::ClosingSelectionProgress::Pull,
            c::SelectionProgress::Checks => w::ClosingSelectionProgress::Checks,
            c::SelectionProgress::Jobs { attempts, next } => {
                let mut jobs = List::with_capacity(u32::try_from(attempts.len()).expect(FITS));
                for attempt in attempts {
                    jobs.push(super::answers::encode_job_attempt(attempt)).expect(FITS);
                }
                w::ClosingSelectionProgress::Jobs(w::ClosingJobs { attempts: jobs, next: *next })
            }
            c::SelectionProgress::Done { line } => {
                w::ClosingSelectionProgress::Done(w::ClosingLine { line: line.clone() })
            }
        };
        rows.push(w::ClosingSelection { pull: selection.pull, head, progress }).expect(FITS);
    }
    rows
}

fn decode_selections(selections: List<w::ClosingSelection>) -> Box<[c::PullSelection]> {
    let mut rows = List::with_capacity(selections.len());
    for selection in selections.into_boxed() {
        let w::ClosingSelection { pull, head, progress } = selection;
        let head = match head {
            Some(head) => Some(head.commit),
            None => None,
        };
        let progress = match progress {
            w::ClosingSelectionProgress::Pull => c::SelectionProgress::Pull,
            w::ClosingSelectionProgress::Checks => c::SelectionProgress::Checks,
            w::ClosingSelectionProgress::Jobs(jobs) => {
                let w::ClosingJobs { attempts, next } = jobs;
                let mut jobs = List::with_capacity(attempts.len());
                for attempt in attempts.into_boxed() {
                    jobs.push(super::answers::decode_job_attempt(attempt)).expect(FITS);
                }
                c::SelectionProgress::Jobs { attempts: jobs.into_boxed(), next }
            }
            w::ClosingSelectionProgress::Done(line) => c::SelectionProgress::Done { line: line.line },
        };
        rows.push(c::PullSelection { pull, head, progress }).expect(FITS);
    }
    rows.into_boxed()
}
