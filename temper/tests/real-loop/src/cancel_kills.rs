//! A cancel's kill is asked for (temper/docs/design/testing/real-loop.md).
use std::{collections::BTreeMap, fmt::Write};

use jig_domain_tasks::Party;
use skein_io::kernel::{Exit, Reason};
use skein_lib::Time;

#[derive(Default)]
pub(crate) struct History {
    pub tasks: BTreeMap<u64, Task>,
    pub cancellations: BTreeMap<u64, Cancellation>,
}
pub(crate) struct Task {
    pub requester: Option<u64>,
    pub agent: bool,
}
pub(crate) struct Cancellation {
    pub by: Party,
    pub at: Time,
}
pub(crate) struct Note {
    pub root: usize,
    pub at: Time,
    pub run: u64,
    pub attempt: u64,
}
pub(crate) struct Kill {
    pub root: Option<usize>,
    pub at: Time,
    pub reason: Reason,
    pub exit: Exit,
}
pub(crate) struct Judged {
    /// One suffix per kill, including kills no cancel accounts for.
    pub details: Vec<String>,
    pub refusal: Option<String>,
}

impl History {
    fn cancelled_with(&self, mut task: u64, at: Time) -> Option<u64> {
        // Requesters form a tree; the bound also refuses malformed plain records.
        for _ in 0..self.tasks.len() {
            if let Some(cancel) = self.cancellations.get(&task)
                && cancel.at <= at
            {
                let story = match cancel.by {
                    Party::Person(_) => true,
                    Party::Task(by) => self.tasks.get(&by).is_some_and(|task| task.agent),
                    Party::Deployment { .. } => false,
                };
                if story {
                    return Some(task);
                }
            }
            task = self.tasks.get(&task)?.requester?;
        }
        None
    }
}

fn account(kill: &Kill, notes: &[Note], history: &History) -> (bool, String) {
    if kill.reason != Reason::Cancelled {
        return (false, String::new());
    }
    let Some(root) = kill.root else { return (false, "; no cancel noted".into()) };
    let latest = notes.iter().filter(|note| note.root == root && note.at <= kill.at).map(|note| note.at).max();
    let Some(latest) = latest else { return (false, "; no cancel noted".into()) };
    let mut accepted = kill.exit == Exit::Signal(9);
    let mut detail = String::new();
    for note in notes.iter().filter(|note| note.root == root && note.at == latest) {
        write!(detail, "; stands for run {} attempt {}, ", note.run, note.attempt).expect("string writer");
        if let Some(task) = history.cancelled_with(note.run, kill.at) {
            write!(detail, "cancelled with task {task}").expect("string writer");
        } else {
            accepted = false;
            detail.push_str("not the story's cancel");
        }
    }
    (accepted, detail)
}

pub(crate) fn judge(kills: &[Kill], notes: &[Note], history: &History, requested: usize) -> Judged {
    let mut accounted = 0;
    let mut details = Vec::new();
    for kill in kills {
        let (accepted, detail) = account(kill, notes, history);
        accounted += usize::from(accepted);
        details.push(detail);
    }
    let refusal = if kills.len().saturating_sub(accounted) != requested
        || kills.iter().any(|kill| kill.exit != Exit::Signal(9))
    {
        Some(format!(
            "only requested kills: {} recorded, {accounted} accounted for by the story's cancels, for {requested} Kill steps",
            kills.len()
        ))
    } else {
        None
    };
    Judged { details, refusal }
}

#[cfg(test)]
mod tests;
