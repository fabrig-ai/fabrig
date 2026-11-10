//! Landing evidence from conversation words and detailed committed activity.
//! See temper/docs/design/client/ctl/tools.md, 2, and temper/docs/design/client/ctl/results.md, 2.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Read conversation replies, then committed call outcomes through task activity.
pub struct Evidence {
    chat: u64,
    ended_at: u64,
    detailed: bool,
    pages: usize,
    /// Committed assistant replies at or after the change ended.
    pub replies: BTreeMap<u64, Value>,
    /// Tools with a committed, non-failed outcome in detailed activity.
    pub tools: BTreeSet<String>,
    /// Tools with a committed failed call in detailed activity.
    pub failed: BTreeSet<String>,
}

impl Evidence {
    /// Start at chat show's first page; each read family allows at most 32 pages.
    #[must_use]
    pub fn new(chat: u64, ended_at: u64) -> Self {
        Self {
            chat,
            ended_at,
            detailed: false,
            pages: 0,
            replies: BTreeMap::new(),
            tools: BTreeSet::new(),
            failed: BTreeSet::new(),
        }
    }

    /// Consume the queued read's JSON and return its continuation, including the
    /// detail fetch once conversation replies are complete. None with no replies
    /// lets the caller wait for a reply; otherwise None ends the evidence reads.
    pub fn read(&mut self, last: &Value) -> Result<Option<Vec<String>>, String> {
        self.pages += 1;
        if self.pages > 32 {
            return Err("bounded conversation evidence pages".to_owned());
        }
        let activity = if self.detailed { last } else { &last["activity"] };
        if activity["cut"] != false {
            return Err("whole activity page".to_owned());
        }
        if self.detailed {
            let rows = activity["rows"].as_array().ok_or("activity rows missing")?;
            for row in rows {
                if let Some(owner) = row.pointer("/event/owner")
                    && owner.pointer("/source/line/turn").is_some()
                    && owner.pointer("/decoded/turn/decoded/cut") != Some(&Value::Bool(false))
                {
                    return Err("whole committed activity turn".to_owned());
                }
            }
            self.tools.extend(super::successful_tools(activity));
            self.failed.extend(super::failed_tools(activity));
        } else {
            self.replies.extend(super::replies_after(activity, self.ended_at)?);
        }
        if let Some(before) = activity["before"]["value"].as_str() {
            let mut words = self.command();
            words.extend(["--before".to_owned(), before.to_owned()]);
            return Ok(Some(words));
        }
        if !self.detailed && !self.replies.is_empty() {
            self.detailed = true;
            self.pages = 0;
            return Ok(Some(self.command()));
        }
        Ok(None)
    }

    fn command(&self) -> Vec<String> {
        let words = if self.detailed { ["task", "activity"] } else { ["chat", "show"] };
        vec![words[0].to_owned(), words[1].to_owned(), self.chat.to_string(), "--json".to_owned()]
    }
}
