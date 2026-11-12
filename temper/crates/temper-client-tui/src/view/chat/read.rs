//! The browser terminal's conversation walk (temper/docs/design/client/pages/README.md and temper/docs/design/client/tui/pages/README.md).
use super::{Entry, Piece, Words, printed, shared};
use alloc::boxed::Box;
use jig_client::{
    contract as c,
    thread::{Line, window::Edge},
};
use skein_lib::{List, Wall};
use skein_ui::phrases::Hole;
use temper_client_domain::{
    Domain,
    pages::{
        chat::{Chat, document},
        delegation,
    },
    views::{self, Part},
};
use temper_client_view::words::Word;

fn flush(run: &mut shared::Run<'_>, entries: &mut List<Entry>, words: &Words) {
    if let Some(line) = run.line(None, false, words.view(), true) {
        entries.push(Entry::Run(line)).expect("conversation entries fit memory");
    }
}
fn at(line: &Line<'_>) -> Option<Wall> {
    match line {
        Line::Words { at, .. } => *at,
        Line::Turn { at, .. } | Line::Undecoded { at, .. } => Some(*at),
        Line::Activity { line } => Some(line.source.at),
        Line::Provisional { .. } => None,
    }
}
fn document_write(call: &c::Call) -> bool {
    if call.failed || !document::writes(call.tool.bytes()) {
        return false;
    }
    match &call.outcome {
        Some(outcome) => document::tool_write(call.tool.bytes(), outcome.bytes()).is_some(),
        None => false,
    }
}
fn card_call(root: &Domain, chat: &Chat, call: &c::Call) -> bool {
    for card in &chat.cards {
        if call.objects.as_slice().contains(&card.object) {
            return true;
        }
    }
    let mut tasks = List::uncapped();
    delegation::delegated(call, &mut tasks);
    for task in &tasks {
        if chat.delegation(root.client(), *task).is_some() {
            return true;
        }
    }
    false
}
#[expect(clippy::too_many_arguments, reason = "the ordered block walk shares its run, time and card context")]
fn committed<'a>(
    root: &Domain,
    chat: &Chat,
    turn: &'a c::Turn,
    at: Wall,
    mut since: Option<Wall>,
    run: &mut shared::Run<'a>,
    entries: &mut List<Entry>,
    words: &Words,
) {
    for message in &turn.messages {
        for block in &message.blocks {
            match block {
                c::Block::Text(text) => {
                    if message.speaker == c::Speaker::Assistant && !text.bytes().is_empty() {
                        flush(run, entries, words);
                        entries.push(agent(text.bytes(), chat)).expect("conversation entries fit memory");
                    }
                }
                c::Block::Call(call) => {
                    if message.speaker != c::Speaker::Assistant {
                        continue;
                    }
                    if document_write(call) || card_call(root, chat, call) {
                        flush(run, entries, words);
                    } else {
                        run.call(call, at, since.take());
                    }
                }
                c::Block::Result { call, summary, .. } => {
                    if run.has_call(*call) {
                        run.result(*call, summary);
                    } else {
                        let mut tasks = List::uncapped();
                        delegation::results(summary.bytes(), &mut tasks);
                        for task in &tasks {
                            if chat.delegation(root.client(), *task).is_some() {
                                flush(run, entries, words);
                                break;
                            }
                        }
                    }
                }
                c::Block::Opaque { .. } => {}
            }
        }
    }
}
#[expect(
    clippy::disallowed_methods,
    reason = "the protocol validated committed UTF-8; the view splitter borrows it as str"
)]
fn agent(bytes: &[u8], chat: &Chat) -> Entry {
    let text = core::str::from_utf8(bytes).expect("committed text is UTF-8");
    let parts = views::parts(text);
    let mut viewed = false;
    for part in &parts {
        match part {
            Part::Text(_) => {}
            Part::View(_) => viewed = true,
        }
    }
    if !viewed {
        return Entry::Agent { words: bytes.into(), growing: false };
    }
    let mut pieces = List::with_capacity(parts.len());
    for part in parts.into_boxed() {
        let piece = match part {
            Part::Text(text) => Piece::Text(text.as_bytes().into()),
            Part::View(address) => Piece::View(printed::read(address, chat.view(address))),
        };
        pieces.push(piece).expect("one piece per text part");
    }
    Entry::Viewed(pieces.into_boxed())
}
fn cancelling_party(ending: &c::ChipEnding) -> Option<c::Requester> {
    match ending {
        c::ChipEnding::Cancelled { by, .. } => Some(*by),
        c::ChipEnding::Done(_) | c::ChipEnding::Failed(_) => None,
    }
}
fn closed_by_requester(root: &Domain, chat: &Chat) -> bool {
    let Some(task) = chat.task(root.client()) else { return false };
    let by = match &task.result {
        Some(c::TaskResult::Ending(ending)) => cancelling_party(ending),
        Some(c::TaskResult::Document(result)) => match &result.ending {
            c::Ending::Task(ending) => cancelling_party(ending),
            c::Ending::Cancelled { by: c::Actor::Party(party), .. } => Some(c::Requester::Person(*party)),
            c::Ending::Cancelled { by: c::Actor::Task(task), .. } => Some(c::Requester::Task(*task)),
            c::Ending::Cancelled { by: c::Actor::ProjectDeployment { project }, .. } => {
                Some(c::Requester::Deployment { project: *project })
            }
            c::Ending::Cancelled { by: c::Actor::Deployment, .. }
            | c::Ending::Failed { .. }
            | c::Ending::Withdrawn { .. }
            | c::Ending::Done
            | c::Ending::Deleted
            | c::Ending::Decided { .. } => None,
        },
        None => None,
    };
    by == Some(task.requester)
}
fn owner(root: &Domain, chat: &Chat, line: &c::Activity, words: &Words) -> Option<Entry> {
    match line {
        c::Activity::Words { by, words, .. } => Some(match by {
            c::Requester::Person(_) | c::Requester::Deployment { .. } => Entry::Said(words.bytes().into()),
            c::Requester::Task(task) => Entry::Delegate { task: *task, words: words.bytes().into() },
        }),
        c::Activity::Heard(word) | c::Activity::Notified { word, .. } => {
            if let Some(task) = delegation::heard(line)
                && chat.delegation(root.client(), task).is_some()
            {
                return None;
            }
            let bytes = match word.kind {
                c::MessageKind::QuestionAnswered { question } => {
                    words.view().say(Word::QuestionAnswered, &[Hole::Number(question)])
                }
                c::MessageKind::Words
                | c::MessageKind::QuestionOwed
                | c::MessageKind::NothingAwaited
                | c::MessageKind::SettingsDecision { .. }
                | c::MessageKind::Escalation { .. }
                | c::MessageKind::Proposal { .. }
                | c::MessageKind::PersonStalled { .. }
                | c::MessageKind::ProposalDecision { .. }
                | c::MessageKind::QuestionEnded { .. }
                | c::MessageKind::EffectNotMade { .. }
                | c::MessageKind::Amendment { .. }
                | c::MessageKind::Question
                | c::MessageKind::Answer { .. }
                | c::MessageKind::Notice { .. }
                | c::MessageKind::Timer { .. }
                | c::MessageKind::News { .. }
                | c::MessageKind::Result(_)
                | c::MessageKind::Met { .. } => word.words.bytes().into(),
            };
            Some(Entry::Said(bytes))
        }
        c::Activity::Decided { proposal, .. } => {
            if closed_by_requester(root, chat) {
                return None;
            }
            if let Some(proposal) = proposal {
                if proposal.made {
                    return None;
                }
                for card in &chat.cards {
                    let standing = match card.object {
                        c::ObjectKey::Proposal(key) => key.number == proposal.number,
                        c::ObjectKey::Task(_)
                        | c::ObjectKey::Escalation(_)
                        | c::ObjectKey::Question { .. }
                        | c::ObjectKey::PersonTask(_)
                        | c::ObjectKey::Result(_)
                        | c::ObjectKey::Note { .. } => false,
                    };
                    if standing {
                        return None;
                    }
                }
            }
            Some(Entry::Agent { words: words.view().say(Word::DecisionMade, &[]), growing: false })
        }
        c::Activity::NowWritten { line, .. } => Some(Entry::Agent {
            words: skein_lib::bytes::concat(&[&words.view().say(Word::NowWritten, &[]), line.bytes()]),
            growing: false,
        }),
        c::Activity::Stood { .. }
        | c::Activity::TalkOpened { .. }
        | c::Activity::TalkEnded { .. }
        | c::Activity::SessionEnded { .. }
        | c::Activity::Kept { .. } => {
            Some(Entry::Agent { words: words.view().say(Word::DependencyParked, &[]), growing: false })
        }
        c::Activity::Turn { .. } => {
            Some(Entry::Agent { words: words.view().say(Word::UnreadTurn, &[]), growing: false })
        }
        c::Activity::RunBegan { .. } | c::Activity::RunEnded { .. } | c::Activity::Milestone { .. } => None,
    }
}
/// A talk begins at its newest opening; pending words survive an unread task.
fn talk_lines(chat: &Chat) -> List<Line<'_>> {
    let lines = chat.thread.lines();
    let Some(talk) = chat.talk() else { return lines };
    let mut after = talk.open().is_some() && talk.from().is_none();
    let mut kept = List::with_capacity(lines.len());
    for line in lines.into_boxed() {
        match &line {
            Line::Activity { line } => {
                if Some(line.source.position) == talk.from() {
                    after = true;
                    continue;
                }
            }
            Line::Words { .. } => {
                kept.push(line).expect("pending words fit");
                continue;
            }
            Line::Turn { .. } | Line::Undecoded { .. } | Line::Provisional { .. } => {}
        }
        if after { kept.push(line).expect("talk lines are a subset"); }
    }
    kept
}
/// The newest committed turn's last assistant text, fenced by later words or live text.
pub(crate) fn last_reply(root: &Domain) -> Option<(c::TurnName, Box<[u8]>)> {
    let chat = root.pages().chat()?;
    if chat.thread.lines.edges.before != Edge::End || chat.thread.loaded < chat.thread.committed {
        return None;
    }
    let mut reply = None;
    for line in talk_lines(chat).into_boxed() {
        match line {
            Line::Turn { name, turn, .. } => {
                reply = None;
                for message in &turn.messages {
                    if message.speaker != c::Speaker::Assistant {
                        continue;
                    }
                    for block in &message.blocks {
                        match block {
                            c::Block::Text(text) => reply = Some((name, text.bytes().into())),
                            c::Block::Call(_) | c::Block::Result { .. } | c::Block::Opaque { .. } => {}
                        }
                    }
                }
            }
            Line::Words { .. } | Line::Undecoded { .. } => reply = None,
            Line::Provisional { text, .. } => {
                if !text.is_empty() {
                    reply = None;
                }
            }
            Line::Activity { line } => match &line.source.line {
                c::Activity::Words { .. } | c::Activity::Turn { .. } => reply = None,
                c::Activity::Heard(_)
                | c::Activity::Notified { .. }
                | c::Activity::Decided { .. }
                | c::Activity::NowWritten { .. }
                | c::Activity::Stood { .. }
                | c::Activity::TalkOpened { .. }
                | c::Activity::TalkEnded { .. }
                | c::Activity::SessionEnded { .. }
                | c::Activity::Kept { .. }
                | c::Activity::RunBegan { .. }
                | c::Activity::RunEnded { .. }
                | c::Activity::Milestone { .. } => {}
            },
        }
    }
    reply
}

pub(super) fn entries(root: &Domain, words: &Words) -> Box<[Entry]> {
    let chat = root.pages().chat().expect("chat page");
    let mut entries = List::uncapped();
    if chat.thread.lines.edges.after == Edge::End
        && let Some((request, _whole)) = chat.request(root.client())
        && !request.is_empty()
    {
        entries.push(Entry::Said(request.into())).expect("conversation entries fit memory");
    }
    let rows = root.client().domain_limits().rows;
    let mut run = shared::Run::with_capacity(rows.saturating_mul(rows).saturating_mul(chat.thread.limits.lines));
    let lines = talk_lines(chat);
    let mut before = None;
    for line in lines.into_boxed() {
        let since = before;
        if let Some(at) = at(&line) {
            before = Some(at);
        }
        match line {
            Line::Turn { turn, at, .. } => committed(root, chat, turn, at, since, &mut run, &mut entries, words),
            Line::Words { words: bytes, .. } => {
                flush(&mut run, &mut entries, words);
                entries.push(Entry::Said(bytes.into())).expect("conversation entries fit memory");
            }
            Line::Provisional { text, .. } => {
                if !text.is_empty() {
                    flush(&mut run, &mut entries, words);
                    entries
                        .push(Entry::Agent { words: text.into(), growing: true })
                        .expect("conversation entries fit memory");
                }
            }
            Line::Activity { line } => {
                if let Some(entry) = owner(root, chat, &line.source.line, words) {
                    flush(&mut run, &mut entries, words);
                    entries.push(entry).expect("conversation entries fit memory");
                } else if let Some(task) = delegation::heard(&line.source.line)
                    && chat.delegation(root.client(), task).is_some()
                {
                    flush(&mut run, &mut entries, words);
                }
            }
            Line::Undecoded { .. } => {
                flush(&mut run, &mut entries, words);
                entries
                    .push(Entry::Agent { words: words.view().say(Word::UnreadTurn, &[]), growing: false })
                    .expect("conversation entries fit memory");
            }
        }
    }
    let working = shared::working(root, words.view());
    let current = if working { chat.current_call() } else { None };
    let running = working && (!run.is_empty() || current.is_some());
    if let Some(line) = run.line(current, running, words.view(), true) {
        entries.push(Entry::Run(line)).expect("conversation entries fit memory");
    }
    if working && !running {
        entries.push(Entry::Dots).expect("conversation entries fit memory");
    }
    for delegation in &chat.delegations {
        if let Some(task) = chat.delegation(root.client(), delegation.task) {
            entries
                .push(Entry::Delegation(shared::delegation_line(task, words.view())))
                .expect("conversation entries fit memory");
        }
    }
    for card in &chat.cards {
        if let Some(title) = shared::proposal_line(card, words.view()) {
            entries.push(Entry::Proposal(title)).expect("conversation entries fit memory");
        }
    }
    entries.into_boxed()
}
