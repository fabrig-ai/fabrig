//! A goal's reds (temper/docs/design/domain/forge/reds.md, 2, 4 and 6).
use crate::{Domain, Key, Limits, RecordRed, Red, RedRecorded, RedRefusal, Request, Stored};
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
