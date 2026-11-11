//! The report's ordered CI reads (temper/docs/design/domain/forge/branches.md, 2).
use crate::{Closer, LOG_START_BYTES, selection_line};
use alloc::boxed::Box;
use skein_forge::api::{Answer, Commit, Error, JobAttempt, LogPart, Read};
use skein_lib::List;

pub const SELECTION_JOBS: u32 = 8;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PullSelection {
    pub pull: u64,
    pub head: Option<Commit>,
    pub progress: SelectionProgress,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SelectionProgress {
    Pull,
    Checks,
    Jobs { attempts: Box<[JobAttempt]>, next: u32 },
    Done { line: Option<Box<[u8]>> },
}

impl PullSelection {
    #[must_use]
    pub fn new(pull: u64, head: Option<Commit>) -> Self {
        let progress = if head.is_some() { SelectionProgress::Checks } else { SelectionProgress::Pull };
        Self { pull, head, progress }
    }
}

#[must_use]
pub fn selection_read(closer: &Closer) -> Option<Read> {
    for selection in &closer.selections {
        match &selection.progress {
            SelectionProgress::Pull => return Some(Read::Pull { number: selection.pull, ci: false }),
            SelectionProgress::Checks => {
                return Some(Read::Checks { commit: selection.head.expect("checks pin a head") });
            }
            SelectionProgress::Jobs { attempts, next } => {
                return Some(Read::Job {
                    attempt: *attempts.get(usize::try_from(*next).expect("bounded job index")).expect("pending job"),
                    max_bytes: LOG_START_BYTES,
                    part: LogPart::Start,
                });
            }
            SelectionProgress::Done { .. } => {}
        }
    }
    None
}

pub fn selection_answer(closer: &mut Closer, result: Result<Answer, Error>) {
    for selection in &mut closer.selections {
        match &selection.progress {
            SelectionProgress::Done { .. } => continue,
            SelectionProgress::Pull | SelectionProgress::Checks | SelectionProgress::Jobs { .. } => {}
        }
        let answer = match result {
            Ok(answer) => observation(answer),
            Err(_) => Observed::Other,
        };
        selection.progress = advance(selection, answer);
        return;
    }
}

fn advance(selection: &mut PullSelection, answer: Observed) -> SelectionProgress {
    let unknown = SelectionProgress::Done { line: None };
    match &selection.progress {
        SelectionProgress::Pull => match answer {
            Observed::Head(head) => {
                selection.head = Some(head);
                SelectionProgress::Checks
            }
            Observed::Checks(_) | Observed::Job { .. } | Observed::Other => unknown,
        },
        SelectionProgress::Checks => match answer {
            Observed::Checks(rows) => {
                let mut attempts = List::with_capacity(SELECTION_JOBS);
                for row in rows {
                    if let Some(attempt) = row.job {
                        attempts.push(attempt).expect("selection job room checked");
                        if attempts.room() == 0 {
                            break;
                        }
                    }
                }
                if attempts.is_empty() {
                    unknown
                } else {
                    SelectionProgress::Jobs { attempts: attempts.into_boxed(), next: 0 }
                }
            }
            Observed::Head(_) | Observed::Job { .. } | Observed::Other => unknown,
        },
        SelectionProgress::Jobs { attempts, next } => match answer {
            Observed::Job { attempt, log }
                if attempts.get(usize::try_from(*next).expect("bounded job index")) == Some(&attempt) =>
            {
                if let Some(line) = selection_line(&log) {
                    return SelectionProgress::Done { line: Some(line.into()) };
                }
                let next = next.checked_add(1).expect("eight selection jobs");
                if usize::try_from(next).expect("bounded job index") == attempts.len() {
                    unknown
                } else {
                    SelectionProgress::Jobs { attempts: attempts.clone(), next }
                }
            }
            Observed::Head(_) | Observed::Checks(_) | Observed::Job { .. } | Observed::Other => unknown,
        },
        SelectionProgress::Done { .. } => unreachable!("only unsettled selections receive answers"),
    }
}

#[derive(Debug)]
enum Observed {
    Head(Commit),
    Checks(Box<[skein_forge::api::Status]>),
    Job { attempt: JobAttempt, log: Box<[u8]> },
    Other,
}

fn observation(answer: Answer) -> Observed {
    match answer {
        Answer::Pull(pull) => Observed::Head(pull.commit),
        Answer::Checks(rows) => Observed::Checks(rows),
        Answer::Job { attempt, log, .. } => Observed::Job { attempt, log },
        Answer::Located { .. }
        | Answer::Items { .. }
        | Answer::Item { .. }
        | Answer::Reviews { .. }
        | Answer::Statuses { .. }
        | Answer::Remarks { .. }
        | Answer::Commit(_)
        | Answer::Branches(_)
        | Answer::PullFiles { .. }
        | Answer::Compare { .. }
        | Answer::File { .. }
        | Answer::Directory { .. }
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators { .. }
        | Answer::Permission(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        | Answer::Queued
        | Answer::Branch(_)
        | Answer::Runs { .. }
        | Answer::Run(_)
        | Answer::Tree { .. }
        | Answer::Commits { .. }
        | Answer::Done => Observed::Other,
    }
}

#[cfg(test)]
mod tests;
