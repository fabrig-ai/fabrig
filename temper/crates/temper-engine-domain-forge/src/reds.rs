//! A goal's reds (temper/docs/design/domain/forge/reds.md, 2, 4 and 6).
use crate::{Domain, Key, Limits, RecordRed, Red, RedPass, RedRecorded, RedRefusal, Request, Stored};
use alloc::boxed::Box;
use core::ops::Bound;
use skein_lib::Queue;

impl Domain {
    /// A goal is the task holding a goal branch, not a name inferred from its number.
    #[must_use]
    pub fn holds_goal_branch(&self, task: u64) -> bool {
        for (name, hold) in &self.holds {
            if hold.task == task && super::goal_branch::is_goal(self, name) {
                return true;
            }
        }
        false
    }
    #[must_use]
    pub fn goal_branch_holder(&self, name: &crate::Name) -> Option<u64> {
        if !super::goal_branch::is_goal(self, name) {
            return None;
        }
        Some(self.hold(name)?.task)
    }
    pub fn reds(&self, goal: u64) -> impl DoubleEndedIterator<Item = (&(u64, Box<[u8]>), &Red)> {
        let start = Bound::Included((goal, Box::default()));
        let end = match goal.checked_add(1) {
            Some(next) => Bound::Excluded((next, Box::default())),
            None => Bound::Unbounded,
        };
        self.reds.range((start, end))
    }
    pub(crate) fn red_passes(&self, change: u64) -> impl DoubleEndedIterator<Item = (&(u64, Box<[u8]>), &RedPass)> {
        let start = Bound::Included((change, Box::default()));
        let end = match change.checked_add(1) {
            Some(next) => Bound::Excluded((next, Box::default())),
            None => Bound::Unbounded,
        };
        self.red_passes.range((start, end))
    }
    /// Each key's latest observation; unnamed keys retain their pass (reds.md, 5.7).
    pub fn observe_reds(
        &mut self,
        change: u64,
        goal: u64,
        passed: &[Box<[u8]>],
        failed: &[Box<[u8]>],
        out: &mut Queue<Request>,
    ) {
        // Remove stale keys before inserting, so replacements never need extra room.
        let mut stale = skein_lib::List::with_capacity(self.red_passes.len());
        for (_, pass) in self.red_passes(change) {
            if !self.reds.contains_key(&(goal, pass.key.clone())) {
                stale.push(pass.key.clone()).expect("bounded red passes");
            }
        }
        for key in stale.into_boxed() {
            erase_pass(self, change, key, out);
        }
        for key in passed {
            if self.reds.contains_key(&(goal, key.clone())) {
                let row = RedPass { change, key: key.clone(), generation: self.red_number.unwrap_or(0) };
                self.red_passes.insert((change, key.clone()), row.clone()).expect("passes bounded by goal reds");
                out.push(Request::Save { record: Stored::RedPass(row), release: None, read_afresh: None });
            }
        }
        for key in failed {
            erase_pass(self, change, key.clone(), out);
        }
    }
    /// The caller admits the key, head and why at its decoding boundary (reds.md, 3).
    pub fn record_red(
        &mut self,
        limits: &Limits,
        call: RecordRed,
        out: &mut Queue<Request>,
    ) -> Result<RedRecorded, RedRefusal> {
        if !self.holds_goal_branch(call.goal) {
            return Err(RedRefusal::NoGoal);
        }
        let key = (call.goal, call.key.clone());
        let replaced = self.reds.get(&key).cloned();
        if replaced.is_none() && self.reds(call.goal).count() >= skein_lib::usize_of(limits.branch_reds) {
            return Err(RedRefusal::Full { bound: limits.branch_reds });
        }
        let generation = self.red_number.unwrap_or(0).checked_add(1).expect("red generation exhausted");
        let row = Red {
            goal: call.goal,
            key: call.key,
            head: call.head,
            why: call.why,
            by: call.by,
            at: call.at,
            generation,
        };
        self.reds.insert(key, row.clone()).expect("reds bounded by goal holds");
        self.red_number = Some(generation);
        out.push(Request::Save { record: Stored::Red(row), release: None, read_afresh: None });
        out.push(Request::Save { record: Stored::RedNumber { last: generation }, release: None, read_afresh: None });
        Ok(RedRecorded { generation, replaced })
    }
    pub fn end_red(&mut self, goal: u64, key: &[u8], out: &mut Queue<Request>) -> Result<Red, RedRefusal> {
        if !self.holds_goal_branch(goal) {
            return Err(RedRefusal::NoGoal);
        }
        let row = self.reds.remove(&(goal, Box::from(key))).ok_or(RedRefusal::NoSuch)?;
        out.push(Request::Erase { key: Key::Red { goal, key: row.key.clone() } });
        Ok(row)
    }
}

fn erase_pass(d: &mut Domain, change: u64, key: Box<[u8]>, out: &mut Queue<Request>) {
    if d.red_passes.remove(&(change, key.clone())).is_some() {
        out.push(Request::Erase { key: Key::RedPass { change, key } });
    }
}
pub(crate) fn erase_passes(d: &mut Domain, change: u64, out: &mut Queue<Request>) {
    loop {
        let key = match d.red_passes(change).next() {
            Some((_, row)) => row.key.clone(),
            None => break,
        };
        erase_pass(d, change, key, out);
    }
}
/// Called in each decision first recording a merge into the destination (reds.md, 5.7).
pub(crate) fn landed(d: &mut Domain, limits: &Limits, row: &crate::ChangeRow, out: &mut Queue<Request>) {
    let resource = skein_forge_client::Resource {
        repository: row.repository,
        what: skein_forge_client::What::Branch(row.base.clone()),
    };
    if let Some(name) = crate::domain::from_client(&resource, limits)
        && let Some(goal) = d.goal_branch_holder(&name)
    {
        let mut ended = skein_lib::List::with_capacity(d.red_passes.len());
        for (_, pass) in d.red_passes(row.task) {
            if let Some(red) = d.reds.get(&(goal, pass.key.clone()))
                && red.generation <= pass.generation
            {
                ended.push(pass.key.clone()).expect("bounded red passes");
            }
        }
        for key in ended.into_boxed() {
            d.end_red(goal, &key, out).expect("goal and red retained until landing");
        }
    }
    erase_passes(d, row.task, out);
}

/// Called with the hold's erasure, including when its remote branch is kept.
pub(crate) fn released(d: &mut Domain, task: u64, name: &crate::Name, out: &mut Queue<Request>) {
    if !super::goal_branch::is_goal(d, name) {
        return;
    }
    loop {
        let key = match d.reds(task).next() {
            Some((key, _)) => key.clone(),
            None => break,
        };
        d.reds.remove(&key);
        out.push(Request::Erase { key: Key::Red { goal: task, key: key.1 } });
    }
}
