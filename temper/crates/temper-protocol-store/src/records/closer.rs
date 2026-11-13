//! Goal closer rows (temper/docs/design/domain/forge/branches.md, 2).
use super::{FITS, w};
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge_vocabulary::CloserRow;
use temper_engine_domain_forge_closer as c;

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
    Some(
        w::ClosingRun::new(&w::CEILINGS, w::ClosingRunParts { step: encode_step(run.step), change: run.change })
            .expect(FITS),
    )
}
fn decode_run(run: Option<w::ClosingRun>) -> Option<c::Running> {
    let w::ClosingRunParts { step, change } = run?.into_parts();
    Some(c::Running { step: decode_step(step), change })
}
pub(super) fn encode(row: &CloserRow) -> w::Closer {
    let closed = match &row.closer.closed {
        Some(tasks) => Some(
            w::ClosingTasks::new(&w::CEILINGS, w::ClosingTasksParts { tasks: List::from_boxed(tasks.clone()) })
                .expect(FITS),
        ),
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
    w::Closer::new(
        &w::CEILINGS,
        w::CloserParts {
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
        },
    )
    .expect(FITS)
}
pub(super) fn decode(row: w::Closer) -> CloserRow {
    let w::CloserParts {
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
    } = row.into_parts();
    let closed = match closed {
        Some(tasks) => Some(tasks.into_parts().tasks.into_boxed()),
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
                Stands::Asked { question, answer } => w::ClosingStands::Asked(
                    w::ClosingAsked::new(
                        &w::CEILINGS,
                        w::ClosingAskedParts { question: *question, answer: answer.clone() },
                    )
                    .expect(FITS),
                ),
            };
            Some(
                w::ClosingQuestion::new(
                    &w::CEILINGS,
                    w::ClosingQuestionParts {
                        asker: open.asker,
                        number: open.number,
                        step: encode_step(open.step),
                        words: open.words.clone(),
                        stands,
                    },
                )
                .expect(FITS),
            )
        }
    };
    let mut replies = List::with_capacity(u32::try_from(relay.replies.len()).expect(FITS));
    for message in &relay.replies {
        let form = match message.form {
            Form::Words => w::ClosingForm::Words,
            Form::Question => w::ClosingForm::Question,
            Form::Answer { question } => {
                w::ClosingForm::Answer(w::Number::new(&w::CEILINGS, w::NumberParts { number: question }).expect(FITS))
            }
        };
        replies
            .push(
                w::ClosingMessage::new(
                    &w::CEILINGS,
                    w::ClosingMessageParts { target: message.target, form, words: message.words.clone() },
                )
                .expect(FITS),
            )
            .expect(FITS);
    }
    Some(w::ClosingRelay::new(&w::CEILINGS, w::ClosingRelayParts { open, replies }).expect(FITS))
}
fn decode_relay(relay: Option<w::ClosingRelay>) -> c::relay::Relay {
    use c::relay::{Form, Message, Question, Relay, Stands};
    let Some(relay) = relay else { return Relay::new() };
    let w::ClosingRelayParts { open, replies } = relay.into_parts();
    let open = match open {
        None => None,
        Some(open) => {
            let w::ClosingQuestionParts { asker, number, step, words, stands } = open.into_parts();
            let stands = match stands {
                w::ClosingStands::Taken => Stands::Taken,
                w::ClosingStands::Asked(asked) => {
                    let w::ClosingAskedParts { question, answer } = asked.into_parts();
                    Stands::Asked { question, answer }
                }
            };
            Some(Question { asker, number, step: decode_step(step), words, stands })
        }
    };
    let mut messages = List::with_capacity(replies.len());
    for message in replies.into_boxed() {
        let w::ClosingMessageParts { target, form, words } = message.into_parts();
        let form = match form {
            w::ClosingForm::Words => Form::Words,
            w::ClosingForm::Question => Form::Question,
            w::ClosingForm::Answer(question) => Form::Answer { question: question.into_parts().number },
        };
        messages.push(Message { target, form, words }).expect(FITS);
    }
    Relay { open, replies: messages.into_boxed() }
}

fn encode_selections(selections: &[c::PullSelection]) -> List<w::ClosingSelection> {
    let mut rows = List::with_capacity(u32::try_from(selections.len()).expect(FITS));
    for selection in selections {
        let head = match selection.head {
            Some(commit) => Some(w::Commit::new(&w::CEILINGS, w::CommitParts { commit }).expect(FITS)),
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
                w::ClosingSelectionProgress::Jobs(
                    w::ClosingJobs::new(&w::CEILINGS, w::ClosingJobsParts { attempts: jobs, next: *next }).expect(FITS),
                )
            }
            c::SelectionProgress::Done { line } => w::ClosingSelectionProgress::Done(
                w::ClosingLine::new(&w::CEILINGS, w::ClosingLineParts { line: line.clone() }).expect(FITS),
            ),
        };
        rows.push(
            w::ClosingSelection::new(&w::CEILINGS, w::ClosingSelectionParts { pull: selection.pull, head, progress })
                .expect(FITS),
        )
        .expect(FITS);
    }
    rows
}

fn decode_selections(selections: List<w::ClosingSelection>) -> Box<[c::PullSelection]> {
    let mut rows = List::with_capacity(selections.len());
    for selection in selections.into_boxed() {
        let w::ClosingSelectionParts { pull, head, progress } = selection.into_parts();
        let head = match head {
            Some(head) => Some(head.into_parts().commit),
            None => None,
        };
        let progress = match progress {
            w::ClosingSelectionProgress::Pull => c::SelectionProgress::Pull,
            w::ClosingSelectionProgress::Checks => c::SelectionProgress::Checks,
            w::ClosingSelectionProgress::Jobs(jobs) => {
                let w::ClosingJobsParts { attempts, next } = jobs.into_parts();
                let mut jobs = List::with_capacity(attempts.len());
                for attempt in attempts.into_boxed() {
                    jobs.push(super::answers::decode_job_attempt(attempt)).expect(FITS);
                }
                c::SelectionProgress::Jobs { attempts: jobs.into_boxed(), next }
            }
            w::ClosingSelectionProgress::Done(line) => c::SelectionProgress::Done { line: line.into_parts().line },
        };
        rows.push(c::PullSelection { pull, head, progress }).expect(FITS);
    }
    rows.into_boxed()
}
