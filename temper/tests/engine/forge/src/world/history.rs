//! Scripted root history pages, held independently of the forge's read state.
use super::{World, top};
use skein_lib::{Duration, Wall};
use top::history::{HistoryRange, InLanding, Operation, Query, Reply, Step, What};

impl World {
    /// Configure retention before writing fixture history; restart uses the same limits.
    pub fn history_limits(&mut self, days: u32, events: u64) {
        self.env.limits.history_days = days;
        self.env.limits.history_events = events;
        self.restart();
    }
    /// Deliver the forge's hourly wall deadline without another history write.
    pub fn history_hour(&mut self, wall: Wall) {
        assert!(self.top.next_wall_deadline().is_some_and(|due| due <= wall));
        self.env.now =
            self.env.now.saturating_add(Duration::from_nanos(wall.as_nanos().saturating_sub(self.env.wall.as_nanos())));
        self.env.wall = wall;
        self.forge_env.now = self.env.now;
        top::fire(&mut self.top, &self.env, &mut self.top_out);
        self.take_top();
    }
    pub fn answer_history(&mut self) {
        let (generation, event) = self.history_held.remove(0);
        if generation == self.generation {
            self.event(event);
        }
    }
    pub fn refuse_history(&mut self) {
        let (generation, top::Event::HistoryLoaded { load, .. }) = self.history_held.remove(0) else {
            panic!("held history load")
        };
        assert_eq!(generation, self.generation);
        self.event(top::Event::HistoryUnloaded { load });
    }
    #[must_use]
    pub fn history_deadline(&self) -> Option<Wall> {
        self.top.next_wall_deadline()
    }
    pub fn write_history(&mut self, change: u64, wall: Wall, landing: Option<u64>) -> u64 {
        let number = self
            .top
            .write_event(
                wall,
                change,
                Step {
                    what: What::Ready,
                    landing: landing.map(|task| InLanding { task, round: 1, unit: 1 }),
                    head: None,
                    base: None,
                    operation: Operation::Own,
                    attempt: 0,
                    source_at: None,
                },
                &mut self.top_out,
            )
            .expect("history change adopted");
        self.take_top();
        number
    }
    pub fn evict_history_change(&mut self, task: u64, held: u32) {
        self.env.limits.changes = held;
        self.event(top::Event::DropChange { task });
    }
    pub fn keep_history_landing(&mut self, row: top::LandingRow) {
        self.stored
            .insert(top::Key::Landing(row.landing.task), top::Stored::Landing(row.clone()))
            .expect("landing room");
        self.event(top::Event::Restore { record: top::Stored::Landing(row) });
    }
    pub fn erase_history_row(&mut self, key: &top::Key) {
        self.stored.remove(key).expect("row to erase");
    }
    pub fn sweeping_history(&mut self, change: u64) {
        let Some(top::Stored::HistoryProject(mut row)) =
            self.stored.get(&top::Key::HistoryProject { project: 5 }).cloned()
        else {
            panic!("project history")
        };
        row.sweeping = Some(change);
        row.whole_since = Some(Wall::from_nanos(1));
        self.event(top::Event::Restore { record: top::Stored::HistoryProject(row) });
    }
    pub fn history_rows(&mut self, rows: u32) {
        self.history_page_rows = rows;
    }
    pub fn hold_history(&mut self) {
        self.history_hold = true;
    }
    #[must_use]
    pub fn history_loads(&self) -> &[(HistoryRange, Option<top::Key>, u32)] {
        &self.history_loads
    }
    pub fn release_history(&mut self) {
        self.history_hold = false;
        for event in core::mem::take(&mut self.history_held) {
            self.to_top.push(event);
        }
    }
    pub fn history_reply(&mut self, read: u64, query: Query) -> Reply {
        self.event(top::Event::ReadHistory { read, query });
        self.finish_history(read)
    }
    pub fn finish_history(&mut self, read: u64) -> Reply {
        for _ in 0..1000 {
            for request in self.take_seen() {
                if let top::Request::HistoryPage { read: found, answer } = request {
                    assert_eq!(found, read);
                    return answer;
                }
            }
            assert!(self.iterate(self.env.now), "history must progress without time");
        }
        panic!("bounded history completion")
    }
    pub(super) fn history_load(&mut self, load: u64, range: HistoryRange, after: Option<top::Key>, most: u32) {
        let mut rows = Vec::new();
        let mut next = None;
        let most = most.min(self.history_page_rows);
        for (key, row) in &self.stored {
            if after.as_ref().is_some_and(|after| key <= after) || !admits(&range, key) {
                continue;
            }
            if rows.len() >= usize::try_from(most).expect("page rows") {
                next = rows.last().map(top::stored_key);
                break;
            }
            rows.push(row.clone());
        }
        self.history_loads.push((range, after, most));
        let event = top::Event::HistoryLoaded { load, rows: rows.into_boxed_slice(), next };
        if self.history_hold {
            self.history_held.push((self.generation, event));
        } else {
            self.to_top.push((self.generation, event));
        }
    }
}
fn admits(range: &HistoryRange, key: &top::Key) -> bool {
    match (range, key) {
        (HistoryRange::Events { project }, top::Key::HistoryEvent { project: found, .. })
        | (HistoryRange::Last { project }, top::Key::HistoryLast { project: found, .. }) => project == found,
        (HistoryRange::Change { project, change }, top::Key::HistoryChange { project: found, change: task, .. }) => {
            project == found && change == task
        }
        (
            HistoryRange::Landing { project, landing },
            top::Key::HistoryLanding { project: found, landing: task, .. },
        ) => project == found && landing == task,
        (HistoryRange::Numbers { project, numbers }, top::Key::HistoryEvent { project: found, number }) => {
            project == found && numbers.contains(number)
        }
        _ => false,
    }
}
