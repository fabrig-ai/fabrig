//! The mixed workload's drawn rounds and their transcript referee (temper/docs/design/testing/workloads.md, 5).
use super::{Script, Store, tasks};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Bounds, Finish, Line, Menu, Slot, Turn};
use smith_domain_session::{
    llm::{Block, Returned},
    record,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

/// Drawn answers played in each stretch.
pub const ROUNDS: u32 = 2;
// Smith's host referee checks one executable operation in a completion.
pub(super) const CALLS: u32 = 1;
const UNISSUED: u64 = 999_999;
const TOOLS: [&str; 3] = ["read_task", "message", "mark_read"];

pub(super) fn turn() -> Turn {
    Turn {
        lines: Box::new([Line::Drawn { rounds: ROUNDS, tools: TOOLS.map(|name| name.as_bytes().into()).into() }]),
        finish: Finish::ToolCalls,
        tokens: 0,
    }
}

fn bodies() -> [Value; 5] {
    let body = |task, target, form, question: Value| {
        json!({"task":task, "target":target, "form":form, "question":question,
            "words":"A drawn message in the composed workload.", "messages":["{{#sent}}"]})
    };
    [
        body("{{#delegate}}", "{{#delegate}}", "words", json!(UNISSUED)),
        body("{{#read}}", "{{#delegate}}", "question", json!(UNISSUED)),
        body("{{#asker}}", "{{#asker}}", "answer", json!("{{#question}}")),
        body("{{#requester}}", "{{#delegate}}", "words", json!(UNISSUED)),
        body("{{#requester}}", "{{#asker}}", "answer", json!("{{#question}}")),
    ]
}

struct Pattern {
    name: &'static str,
    before: &'static str,
    after: &'static str,
    with: Option<&'static str>,
    weight: u32,
    foreign: u32,
}

const PATTERNS: [Pattern; 6] = [
    Pattern { name: "delegate", before: "Delegated tasks: ", after: "", with: None, weight: 100, foreign: 0 },
    Pattern { name: "read", before: "Task ", after: ", spec revision", with: None, weight: 200, foreign: 0 },
    Pattern { name: "requester", before: "[Requested by task ", after: "]", with: None, weight: 200, foreign: 0 },
    Pattern { name: "asker", before: "message target=", after: ",", with: None, weight: 100, foreign: 0 },
    Pattern { name: "question", before: "question=", after: ")", with: Some("asker"), weight: 100, foreign: 200 },
    Pattern { name: "sent", before: "Message ", after: " sent", with: None, weight: 200, foreign: 0 },
];

pub(super) fn menu() -> Menu {
    Menu {
        arguments: bodies().map(|body| serde_json::to_vec(&body).expect("menu body").into()).into(),
        invalid: Box::new([]),
        slots: PATTERNS
            .map(|pattern| Slot {
                name: pattern.name.as_bytes().into(),
                before: pattern.before.as_bytes().into(),
                after: pattern.after.as_bytes().into(),
                with: pattern.with.unwrap_or("").as_bytes().into(),
                unissued: UNISSUED.to_string().into_bytes().into(),
                weight: pattern.weight,
                foreign: pattern.foreign,
                numeric: Some(Bounds { low: 1, high: u64::MAX }),
            })
            .into(),
    }
}

/// The same menu and one stretch in the standalone fake's document form.
#[must_use]
pub fn document() -> (Value, Value) {
    let slots = PATTERNS.map(|pattern| {
        json!({"name":pattern.name, "before":pattern.before, "after":pattern.after,
        "unissued":UNISSUED.to_string(), "weight":pattern.weight, "foreign":pattern.foreign, "with":pattern.with,
        "numeric":{"low":1,"high":u64::MAX}})
    });
    (
        json!({"arguments":bodies(), "invalid":[], "slots":slots, "calls":CALLS}),
        json!({"lines":[{"drawn":{"rounds":ROUNDS,"tools":TOOLS}}], "finish":"tool_calls", "tokens":0}),
    )
}

pub(super) fn plan(script: &Script) -> Vec<Range<usize>> {
    let mut start = 0;
    let mut ranges = Vec::new();
    for turn in &script.turns {
        if let [Line::Drawn { rounds, .. }] = turn.lines.as_ref() {
            let end = start + usize::try_from(*rounds).expect("stretch length");
            ranges.push(start..end);
            start = end;
        } else {
            start += 1;
        }
    }
    ranges
}

fn no_message_reference(text: &[u8], detail: &[u8]) -> bool {
    detail.starts_with(b"no reference")
        || (text.starts_with(b"Authority refused: ") && skein_lib::bytes::find(text, b"task reference").is_some())
}

/// A refusal's product decision, independent of where the model got its target.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Refusal {
    /// A task read refused for lack of a reference.
    ReadReference,
    /// A message refused for lack of a reference to its recipient.
    MessageReference,
    /// A message to a person delegate that had already ended.
    EndedRecipient,
    /// An answer to a question the product had already received an answer to.
    AnsweredQuestion,
    /// A mark-read call that names no unrecorded words.
    UnknownMessage,
    /// Another refusal, retaining the product's explanation.
    Other(String),
}

/// Target classes distinguished by the workload's referee.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Target {
    /// The menu's sentinel, never allocated by these worlds.
    NeverIssued,
    /// A person task delegated by this producer, now in the ended store.
    EndedPersonDelegate,
    /// Any other task or message number found in the conversation.
    Conversation,
}

/// Rounds that reached the process boundary and the product's refusals within them.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Audit {
    planned: BTreeMap<u64, Vec<Range<usize>>>,
    played: BTreeSet<(u64, usize)>,
    calls: u64,
    refusals: BTreeMap<(Refusal, Target), u64>,
}

impl Audit {
    pub(super) fn plan(&mut self, task: u64, ranges: Vec<Range<usize>>) {
        match self.planned.entry(task) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(ranges);
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                assert_eq!(*entry.get(), ranges, "task {task} resumed the same stretches");
            }
        }
    }

    pub(super) fn observe(&mut self, task: u64, index: usize, turn: &record::Turn, store: &Store) {
        if !self.planned.get(&task).is_some_and(|ranges| ranges.iter().any(|range| range.contains(&index))) {
            return;
        }
        assert!(self.played.insert((task, index)), "task {task} repeated drawn round {index}");
        let blocks = || turn.messages.iter().flat_map(|message| &message.content);
        let mut results = BTreeMap::new();
        for block in blocks() {
            if let Block::ToolResult { id, result } = block {
                assert!(results.insert(id.as_ref(), result).is_none(), "task {task}: duplicate result {id:?}");
            }
        }
        let mut calls = 0;
        for call in blocks().filter_map(Block::tool_call) {
            assert!(
                TOOLS.iter().any(|tool| tool.as_bytes() == call.name.as_ref()),
                "task {task}: not a stretch tool: {:?}",
                call.name
            );
            let result = results.remove(call.id.as_ref()).expect("one answer per drawn call");
            calls += 1;
            self.count(task, &call.name, &call.input, result, store);
        }
        assert!((1..=CALLS).contains(&calls), "task {task}: {calls} drawn calls at round {index}");
        assert!(results.is_empty(), "task {task}: results with no drawn call: {results:?}");
        self.calls += u64::from(calls);
    }

    fn count(&mut self, task: u64, tool: &[u8], input: &[u8], result: &Returned, store: &Store) {
        let Returned::Text { text, error, .. } = result else {
            panic!("task {task}: drawn call got an unexpected terminal: {result:?}");
        };
        let args: Value = serde_json::from_slice(input).expect("drawn body");
        let target = match tool {
            b"read_task" => &args["task"],
            b"message" => &args["target"],
            b"mark_read" => &args["messages"][0],
            _ => unreachable!("tool was checked"),
        }
        .as_u64()
        .expect("numeric target");
        let class = if target == UNISSUED {
            Target::NeverIssued
        } else if tool != b"mark_read"
            && store.rows.get(&super::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(target)))).is_some_and(
                |row| {
                    matches!(row, super::engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))
                if row.requester == tasks::Party::Task(task) && matches!(row.executor, tasks::Executor::Person(_)))
                },
            )
        {
            Target::EndedPersonDelegate
        } else {
            Target::Conversation
        };
        if tool == b"read_task" && class == Target::NeverIssued {
            assert!(*error, "task {task}: read of never-issued task succeeded");
            assert_eq!(text.as_ref(), b"Task read refused: no reference", "task {task}: read of never-issued task");
        }
        let decision = format!("Refused: task {target} ");
        let detail = text.strip_prefix(decision.as_bytes()).unwrap_or(text.as_ref());
        let refusal = if tool == b"mark_read" && text.starts_with(b"named none of your unrecorded words: ") {
            Refusal::UnknownMessage
        } else if !*error {
            return;
        } else if text.as_ref() == b"Task read refused: no reference" {
            Refusal::ReadReference
        } else if detail.starts_with(b"question ") && skein_lib::bytes::find(detail, b" was answered by ").is_some() {
            Refusal::AnsweredQuestion
        } else if tool == b"message"
            && class == Target::EndedPersonDelegate
            && (no_message_reference(text, detail) || matches!(detail, b"unknown" | b"task state"))
        {
            // An ended person task may already have lost its live delegation reference.
            Refusal::EndedRecipient
        } else if tool == b"message" && no_message_reference(text, detail) {
            Refusal::MessageReference
        } else {
            Refusal::Other(String::from_utf8_lossy(text).into_owned())
        };
        *self.refusals.entry((refusal, class)).or_default() += 1;
    }

    /// Every registered stretch ran all its rounds, once each.
    pub fn check(&self) -> Result<(), String> {
        if self.planned.is_empty() {
            return Err("no producer stretches planned".into());
        }
        for (task, ranges) in &self.planned {
            for round in ranges.iter().flat_map(Clone::clone) {
                if !self.played.contains(&(*task, round)) {
                    return Err(format!("task {task} missed drawn round {round}"));
                }
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn refusals(&self) -> &BTreeMap<(Refusal, Target), u64> {
        &self.refusals
    }

    /// Counts of settled rounds and calls, not an extra seed or shape count.
    #[must_use]
    pub fn played(&self) -> (usize, u64) {
        (self.played.len(), self.calls)
    }
}
