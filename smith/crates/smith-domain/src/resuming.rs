//! Cut turns are selected by main's answer and rebuilt before any completion.
//! No provider input is parsed here. Contract: domain/turn-parts.md, sections 5 and 6.
use crate::{AnsweredCall, Limits};
use alloc::boxed::Box;
use skein_lib::List;
use smith_domain_session::{
    llm::{Block, Returned, Role},
    record::{self, PartKind},
};

#[derive(Debug)]
pub(crate) struct Parts {
    held: Box<[record::Resumed]>,
    pub(crate) fallback_answers: Box<[u8]>,
    sequence: u32,
    results: Box<[Block]>,
    pub(crate) deliveries: List<(u32, Box<crate::run::Delivery>)>,
}

pub(crate) fn select(
    history: Option<&record::Transcript>,
    held: Box<[record::Resumed]>,
    answered: Box<[AnsweredCall]>,
    limits: &Limits,
) -> (Option<Parts>, Box<[AnsweredCall]>) {
    let Some(history) = history else { return (None, answered) };
    let next = match history.turns.last() {
        Some(turn) => turn.sequence.checked_add(1),
        None => Some(1),
    };
    let Some(sequence) = next else { return (None, answered) };
    let mut found = false;
    let mut bytes = history.owned_bytes().unwrap_or(u64::MAX);
    for resumed in &held {
        bytes = bytes.saturating_add(resumed.part.body.owned_bytes().unwrap_or(u64::MAX));
        if resumed.part.thread == 0 && resumed.part.kind == PartKind::Answer {
            if !found && resumed.part.body.sequence != sequence {
                return (None, answered);
            }
            found = true;
        }
    }
    if !found
        || bytes > limits.session.history_bytes
        || held.len() > usize::try_from(limits.session.history_messages).expect("u32 fits")
    {
        return (None, answered);
    }
    if answered.len() > usize::try_from(limits.run.answered_calls).expect("u32 fits") {
        return (None, answered);
    }
    let Some(fallback_answers) = crate::waking::render(answered.clone(), limits) else { return (None, answered) };
    let mut parts = Parts {
        held,
        fallback_answers,
        sequence,
        results: Box::default(),
        deliveries: List::with_capacity(limits.run.answered_calls),
    };
    let Some(answer) = parts.main_answer() else { return (None, answered) };
    if !record::cut_follows(history, answer, &limits.session) {
        return (None, answered);
    }
    let Some(message) = answer.messages.last().cloned() else { return (None, answered) };
    if message.role != Role::Assistant {
        return (None, answered);
    }
    let mut results = List::with_capacity(limits.run.answered_calls);
    let mut remaining = List::with_capacity(limits.run.answered_calls);
    for answered in answered {
        let mut position = None;
        for resumed in &parts.held {
            if parts.group(resumed)
                && resumed.part.thread == 0
                && resumed.part.kind == PartKind::Answer
                && answered.name.activation == resumed.activation
                && answered.name.completion == resumed.turn
            {
                position = Some(answered.name.position);
            }
        }
        let call = match position {
            Some(position) => match message.content.get(usize::try_from(position).expect("u32 fits")) {
                Some(block) => block.tool_call(),
                None => None,
            },
            None => None,
        };
        if let Some(call) = call
            && call.name == answered.tool
        {
            let given = match answered.answer {
                crate::Answered::Delivery(delivery) if call.name.as_ref() == b"finish" => {
                    parts.deliveries.push((answered.name.position, delivery)).expect("bounded host answers");
                    continue;
                }
                answer @ (crate::Answered::Host(_) | crate::Answered::TooLarge | crate::Answered::Delivery(_)) => {
                    answer
                }
            };
            let feedback =
                crate::waking::answer(given, limits).expect("compatible feedback bound checked at admission");
            results
                .push(Block::ToolResult {
                    id: call.id.clone(),
                    result: Returned::Text { text: feedback.text, error: feedback.error, replay: None },
                })
                .expect("one result per bounded host answer");
        } else {
            remaining.push(answered).expect("bounded host answers");
        }
    }
    parts.results = results.into_boxed();
    (Some(parts), remaining.into_boxed())
}

impl Parts {
    fn group(&self, resumed: &record::Resumed) -> bool {
        for main in &self.held {
            if main.part.thread == 0
                && main.part.kind == PartKind::Answer
                && main.part.body.sequence == self.sequence
                && main.activation == resumed.activation
                && main.turn == resumed.turn
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn identity(&self) -> Option<crate::run::Identity> {
        match &self.main_answer()?.kind {
            record::Kind::First(opener) | record::Kind::Window(opener) => {
                Some(crate::run::Identity::new(opener.affinity.key))
            }
            record::Kind::Ordinary => None,
        }
    }

    fn main_answer(&self) -> Option<&record::Turn> {
        for resumed in &self.held {
            if !self.group(resumed) {
                continue;
            }
            let part = &resumed.part;
            if part.kind == PartKind::Answer && part.thread == 0 {
                return Some(&part.body);
            }
        }
        None
    }

    /// A compaction can be the child's last part when the drain stops its next completion.
    pub(crate) fn window(&self, position: u32) -> Option<(record::Transcript, u32)> {
        let mut window = None;
        let mut highest = 0_u32;
        let mut answers = 0_u32;
        for resumed in &self.held {
            if !self.group(resumed) || resumed.part.thread == 0 || resumed.part.call != position {
                continue;
            }
            let part = &resumed.part;
            match part.kind {
                PartKind::Answer => {
                    if part.body.sequence > highest {
                        highest = part.body.sequence;
                        answers = answers.checked_add(1)?;
                        window = None;
                    }
                }
                PartKind::Window => window = Some(&part.body),
                PartKind::Result => {}
            }
        }
        let window = window?;
        Some((
            record::Transcript {
                version: record::VERSION,
                endpoint: window.endpoint,
                dialect: window.dialect,
                turns: Box::new([window.clone()]),
            },
            answers,
        ))
    }

    pub(crate) fn build(
        &self,
        position: Option<u32>,
        history: Option<record::Transcript>,
        limits: &Limits,
    ) -> Option<(record::Transcript, record::Cut)> {
        let child = position.is_some();
        let position = position.unwrap_or(0);
        let mut turns = List::with_capacity(limits.session.history_messages);
        if let Some(history) = history {
            for turn in history.turns {
                turns.push(turn).ok()?;
            }
        }
        let previous = turns.len();
        let mut last = None;
        let mut answers = 0_u32;
        let mut highest_answer = 0_u32;
        for resumed in &self.held {
            if !self.group(resumed) {
                continue;
            }
            let part = &resumed.part;
            if if child { part.thread == 0 || part.call != position } else { part.thread != 0 } {
                continue;
            }
            match part.kind {
                PartKind::Window if child => {
                    turns.clear();
                    turns.push(part.body.clone()).ok()?;
                    last = None;
                }
                PartKind::Answer => {
                    if let Some(previous) = last
                        && part.body.sequence <= previous
                    {
                        continue;
                    }
                    if !child && part.body.sequence != self.sequence {
                        continue;
                    }
                    if part.body.sequence > highest_answer {
                        answers = answers.checked_add(1)?;
                        highest_answer = part.body.sequence;
                    }
                    turns.push(part.body.clone()).ok()?;
                    last = Some(part.body.sequence);
                }
                PartKind::Result | PartKind::Window => {}
            }
        }
        let answer = turns.last()?;
        let sequence = answer.sequence;
        let usage = answer.usage;
        let messages = u32::try_from(answer.messages.len()).ok()?;
        let content = answer.messages.last()?.content.clone();
        if content.len() > usize::try_from(limits.session.completion_blocks).expect("u32 fits") {
            return None;
        }
        if answer.messages.last()?.role != Role::Assistant {
            return None;
        }
        let results = self.results_for(&content, child, position, sequence, limits)?;
        let mut jobs = answer.jobs;
        for resumed in &self.held {
            if self.group(resumed)
                && if child {
                    resumed.part.thread != 0 && resumed.part.call == position
                } else {
                    resumed.part.thread == 0
                }
            {
                jobs = jobs.max(resumed.part.body.jobs);
            }
        }
        let last = turns.len().checked_sub(1)?;
        turns.get_mut(last)?.jobs = jobs;
        let first = turns.get(0)?;
        let system = match &first.kind {
            record::Kind::First(opener) | record::Kind::Window(opener) => opener.system.clone(),
            record::Kind::Ordinary => None,
        };
        let history = record::Transcript {
            version: record::VERSION,
            endpoint: first.endpoint,
            dialect: first.dialect,
            turns: turns.into_boxed(),
        };
        Some((
            history,
            record::Cut { previous, usage, system, answers, waking_in_result: false, messages, content, results },
        ))
    }
    fn results_for(
        &self,
        content: &[Block],
        child: bool,
        position: u32,
        sequence: u32,
        limits: &Limits,
    ) -> Option<Box<[Block]>> {
        let mut results = List::with_capacity(limits.session.completion_blocks);
        // Parts arrive by activation then number; the last result wins over host answers.
        for block in content {
            let Some(call) = block.tool_call() else { continue };
            let mut found = None;
            for resumed in &self.held {
                let part = &resumed.part;
                if !self.group(resumed)
                    || part.kind != PartKind::Result
                    || part.body.sequence != sequence
                    || if child { part.thread == 0 || part.call != position } else { part.thread != 0 }
                {
                    continue;
                }
                if let Some(result) = one_result(&part.body) {
                    match result {
                        Block::ToolResult { id, .. } if *id == call.id => {
                            found = Some(result.clone());
                        }
                        Block::ToolResult { .. }
                        | Block::ToolCall(_)
                        | Block::Text { .. }
                        | Block::Refusal { .. }
                        | Block::Opaque { .. } => {}
                    }
                }
            }
            if !child && found.is_none() {
                for result in &self.results {
                    match result {
                        Block::ToolResult { id, .. } if *id == call.id => {
                            found = Some(result.clone());
                            break;
                        }
                        Block::ToolResult { .. }
                        | Block::ToolCall(_)
                        | Block::Text { .. }
                        | Block::Refusal { .. }
                        | Block::Opaque { .. } => {}
                    }
                }
            }
            if let Some(result) = found {
                results.push(result).ok()?;
            }
        }
        Some(results.into_boxed())
    }
}

fn one_result(turn: &record::Turn) -> Option<&Block> {
    if turn.messages.len() != 1 {
        return None;
    }
    let message = turn.messages.first()?;
    if message.role != Role::User || message.content.len() != 1 {
        return None;
    }
    let block = message.content.first()?;
    match block {
        Block::ToolResult { .. } => Some(block),
        Block::ToolCall(_) | Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } => None,
    }
}
