//! The one event writer (temper/docs/design/domain/landing-history.md, 3.1).
use crate::{Domain, Key, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Wall};
pub use temper_engine_domain_forge_vocabulary::history::*;

impl Domain {
    /// Save an event and its indexes in the caller's decision. No event is retained.
    #[cfg_attr(not(test), expect(dead_code, reason = "landing history step callers are added after the stored writer"))]
    pub fn write_event(&mut self, wall: Wall, change: u64, step: Step, out: &mut Queue<Request>) -> Option<u64> {
        let mut row = self.changes.get(&change)?.clone();
        let project = self.repository(row.repository)?.project;
        let mut history = match self.history_projects.get(&project) {
            Some(row) => row.clone(),
            None => Project {
                project,
                next: 1,
                kept: 0,
                whole_since: None,
                marks: Box::new([]),
                sweeping: None,
                removing: false,
            },
        };
        let number = history.next;
        history.next = number.checked_add(1).expect("history event numbers fit u64");
        history.kept = history.kept.checked_add(1).expect("history event count fits u64");
        let hour =
            Wall::from_nanos(wall.as_nanos().checked_div(HOUR_NS).expect("nonzero hour").saturating_mul(HOUR_NS));
        let mut marked = false;
        for mark in &history.marks {
            marked |= mark.hour == hour;
        }
        if !marked {
            let mut marks = List::with_capacity(MARKS);
            let full = history.marks.len() >= usize::try_from(MARKS).expect("marks bound fits usize");
            for (index, mark) in history.marks.iter().enumerate() {
                if full && index == 0 {
                    continue;
                }
                marks.push(*mark).expect("bounded history marks");
            }
            marks.push(Mark { hour, first: number }).expect("room for hour mark");
            history.marks = marks.into_boxed();
        }
        let Step { what, landing, head, base, operation, attempt, source_at } = step;
        let (operation, attempt) = match operation {
            Operation::Own => (number, 0),
            Operation::Of(operation) => (operation, attempt),
        };
        save(
            out,
            Stored::HistoryEvent(Event {
                project,
                number,
                at: wall,
                source_at,
                change,
                repository: row.repository,
                landing,
                head,
                base,
                operation,
                attempt,
                what,
            }),
        );
        save(
            out,
            Stored::HistoryChange(ChangeIndex {
                project,
                change,
                number,
                landing: match landing {
                    Some(landing) => Some(landing.task),
                    None => None,
                },
            }),
        );
        if let Some(landing) = landing {
            save(out, Stored::HistoryLanding(LandingIndex { project, landing: landing.task, number, change }));
        }
        if let Some(previous) = row.history {
            out.push(Request::Erase { key: Key::HistoryLast { project, number: previous.last, change } });
        }
        save(out, Stored::HistoryLast(Last { project, number, change, at: wall }));
        save(out, Stored::HistoryProject(history.clone()));
        self.history_projects.insert(project, history).expect("one history row per project");
        let mut changed = row.history.unwrap_or(ChangeHistory {
            last: number,
            ready: None,
            queued: None,
            paused: None,
            held: None,
            landing_held: None,
        });
        changed.last = number;
        row.history = Some(changed);
        self.changes.insert(change, row.clone()).expect("existing change row");
        crate::domain::save_change(self, out, row);
        Some(number)
    }
}

#[cfg_attr(not(test), expect(dead_code, reason = "used by the event writer before its step callers land"))]
fn save(out: &mut Queue<Request>, record: Stored) {
    out.push(Request::Save { record, release: None, read_afresh: None });
}
