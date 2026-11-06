//! The chat follows jig-client's thread in chronological order.
use crate::{cards::P, look};
use crate::{
    frame::Bindings,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::{
    contract as c,
    thread::{Line, Thread, Words as WordState},
};
use skein_lib::List;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{Act, chat::Chat, delegation},
};
fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
fn control(b: &mut Builder, bindings: &mut Bindings, role: Role, name: &[u8], act: Act) {
    b.open(role);
    b.name(name);
    b.bind(bindings.insert(act));
    b.close();
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    rows: u32,
    text_limit: u32,
    times: &mut Times,
) {
    b.open(Role::Main);
    b.style(super::document::mode(root.pages().chat()));
    let configuring = configuring(root);
    b.style(look::SESSION);
    if let Some(chat) = root.pages().chat()
        && let Some(task) = chat.task(root.client())
    {
        let title = words.say(
            Word::ChatTitle,
            &[Hole::Number(chat.task), Hole::Name(skein_lib::bytes::copy_of(task.chip.title.bytes()))],
        );
        super::session::top(
            b,
            root,
            words,
            bindings,
            Some(task.project),
            if configuring { super::session::Path::Settings } else { super::session::Path::Chats },
            super::session::Heading::Chat { whole: &title, title: task.chip.title.bytes() },
        );
        super::session::right(b);
        if configuring {
            super::configure::byline(b, root, words, task.project);
        } else {
            crate::choice::byline(b, root, Some(task), words, bindings);
        }
        super::document::seg(b, chat, words, bindings);
        super::chat_sheet::bar(b, root, words, bindings);
        if chat.draft.is_none()
            && !chat.writing
            && task.result.is_none()
            && !super::chats::ended(&task.chip)
            && temper_client_domain::pages::drafts_offered(root, task.project)
        {
            super::document::add(b, words, bindings, Act::AddDraft);
        }
        if configuring && chat.draft.is_some() && task.result.is_none() && !super::chats::ended(&task.chip) {
            b.open(Role::Button);
            b.name(&words.say(Word::StartOver, &[]));
            b.style(look::START_OVER);
            b.bind(bindings.insert(Act::StartOver));
            b.close();
        }
        bar_status(b, root, task, words, times);
        bar_actions(b, task, words, bindings);
        b.close();
        b.close();
        b.open(Role::Region);
        b.name(&words.say_card(P::ConversationLog, &[]));
        b.style(look::CONVO);
        b.open(Role::Slot);
        b.style(look::THREAD);
        let person = speakers(task, root.pages().me(root.client()));
        conversation_with_times(
            b,
            &chat.thread,
            words,
            (rows, text_limit),
            person,
            (Drawn::ChatPage { request: Some(&task.spec), root }, Some(times)),
            bindings,
        );
        b.close();
        if task.result.is_none() && !super::chats::ended(&task.chip) && !configuring {
            crate::choice::above_composer(b, root, Some(task), words, bindings, times);
        }
        session_composer(b, root, chat, task, words, bindings);
        b.close();
        super::document::pane(b, root, chat, task, words, times);
    }
    crate::confirm::main(b, root, words, bindings, rows);
    b.close();
}
fn conversation_head(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings) {
    let chat = root.pages().chat().expect("chat page");
    let task = chat.task(root.client()).expect("chat task");
    context(b, root, task.project, words);
    if chat.thread.lines().is_empty() && (chat.draft.is_some() || chat.writing) {
        let name = match &chat.draft {
            Some(draft) => words
                .say(Word::DraftFile, &[Hole::Name(temper_client_domain::pages::chat::document::slug(&draft.title))]),
            None => words.say(Word::DraftName, &[]),
        };
        b.open(Role::Paragraph);
        b.style(look::META_LINE);
        b.text(Run::Plain, &words.say(Word::ChatWrites, &[Hole::Name(name)]));
        b.close();
    }
    if chat.offers_older() {
        b.open(Role::Button);
        b.style(look::CHATS_MORE);
        b.name(&words.say(Word::Older, &[]));
        b.bind(bindings.insert(Act::Older));
        b.close();
    }
}
fn conversation_tail(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    rows: u32,
    times: &mut Times,
) {
    let chat = root.pages().chat().expect("chat page");
    let task = chat.task(root.client()).expect("chat task");
    let mut unanchored = false;
    for card in &chat.cards {
        if proposal_anchor(chat, card.object, rows).is_none() {
            unanchored = true;
        }
    }
    if unanchored || !chat.departures().is_empty() {
        b.open(Role::List);
        b.style(look::CARDS);
    }
    for card in &chat.cards {
        if proposal_anchor(chat, card.object, rows).is_none() {
            proposal_card(b, root, card, words, bindings, (rows, Some(times)), Role::Item);
        }
    }
    for object in chat.departures() {
        b.open(Role::Item);
        b.style(look::CARD);
        b.key(super::inbox::card_key(*object));
        words.status(b, P::CardLeft, &[]);
        b.close();
    }
    if unanchored || !chat.departures().is_empty() {
        b.close();
    }
    if let Some(result) = &task.result {
        let clock = end_clock(task, words, times);
        result_text(b, result, task.requester, words, clock.as_deref());
    }
    act_status(b, root, chat, words);
}
/// The delivered chat's chip decides its path and current navigation entry (client/README.md, 3.5).
pub(crate) fn configuring(root: &Domain) -> bool {
    let Some(chat) = root.pages().chat() else { return false };
    let Some(task) = chat.task(root.client()) else { return false };
    let Some(me) = root.pages().me(root.client()) else { return false };
    temper_client_domain::pages::chats::configuring(me, task.project, &task.chip.executor)
}
fn bar_status(b: &mut Builder, root: &Domain, task: &c::Task, words: &Words, times: &mut Times) {
    if task.result.is_some() || super::chats::ended(&task.chip) {
        closed_status(b, task, words, times);
    } else if let Some(me) = root.pages().me(root.client()) {
        let (token, name) = super::chats::status_words(
            &task.chip,
            (
                &task.waits_on,
                Some(root.pages().chat().expect("chat page").waiting_delegates(root.client())),
                task.run_wait.as_ref(),
            ),
            (task.project, task.requester),
            Some(&task.wake),
            me,
            words,
        );
        b.open(Role::Status);
        b.style(token);
        if token == look::STATUS_YOURS || token == look::STATUS_WAITING {
            b.style(look::STATE_NEEDS);
        }
        b.name(&name);
        b.text(Run::Plain, &name);
        b.close();
    }
}
fn bar_actions(b: &mut Builder, task: &c::Task, words: &Words, bindings: &mut Bindings) {
    if temper_client_domain::pages::chat::allowed(task, c::ActKind::Stop) {
        control(b, bindings, Role::Button, &words.say(Word::Stop, &[]), Act::Stop);
    }
    if temper_client_domain::pages::chat::allowed(task, c::ActKind::Cancel) {
        control(b, bindings, Role::Button, &words.say(Word::Close, &[]), Act::Close);
    }
    spend(b, task, words);
}
fn session_composer(
    b: &mut Builder,
    root: &Domain,
    chat: &Chat,
    task: &c::Task,
    words: &Words,
    bindings: &mut Bindings,
) {
    if task.result.is_some()
        || super::chats::ended(&task.chip)
        || !temper_client_domain::pages::chat::allowed(task, c::ActKind::Message)
    {
        return;
    }
    super::session::foot(b, &words.say(Word::Message, &[]));
    b.open(Role::TextArea);
    b.name(&words.say(Word::Message, &[]));
    b.placeholder(&words.say(Word::MessagePlaceholder, &[Hole::Number(task.chip.task)]));
    if crate::choice::can_send(root) {
        b.state(State::SubmitsOnEnter);
    }
    if chat.composer_focus != 0 && root.pages().switcher().renaming().is_none() {
        b.focus(chat.composer_focus);
    }
    b.value(&chat.words, temper_client_domain::pages::shown(root.pages()));
    b.bind(bindings.insert(Act::MessageWords { words: Box::default() }));
    b.close();
    b.open(Role::Button);
    b.name(&words.say(Word::Send, &[]));
    if chat.words.is_empty() || !crate::choice::can_send(root) {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(Act::Send));
    b.close();
    super::session::hint(b, words, chat.draft.is_some() || chat.writing);
    if chat.outcome_kind == Some(temper_client_domain::pages::chat::Sending::Message)
        && let Some(outcome) = &chat.outcome
        && !super::chats::accepted(outcome)
    {
        super::chats::outcome_text(b, words, outcome, 0);
    }
    b.close();
}
fn closed_status(b: &mut Builder, task: &c::Task, words: &Words, times: &mut Times) {
    let clock = end_clock(task, words, times);
    let name = match clock {
        Some(clock) => words.say(Word::TimeClosed, &[Hole::Name(clock)]),
        None => words.say(Word::ChatClosed, &[]),
    };
    b.open(Role::Status);
    b.style(look::STATUS_QUIET);
    b.name(&name);
    b.text(Run::Plain, &name);
    b.close();
}
fn act_status(b: &mut Builder, root: &Domain, chat: &Chat, words: &Words) {
    for (key, kind) in &chat.pending {
        match kind {
            temper_client_domain::pages::chat::Sending::Message | temper_client_domain::pages::chat::Sending::Seen => {}
            temper_client_domain::pages::chat::Sending::Stop | temper_client_domain::pages::chat::Sending::Close => {
                if let Some(pending) = root.client().pending(*key) {
                    crate::confirm::progress_text(b, words, pending.progress);
                }
            }
        }
    }
    if chat.outcome_kind != Some(temper_client_domain::pages::chat::Sending::Message)
        && let Some(outcome) = &chat.outcome
    {
        super::chats::outcome_text(b, words, outcome, 0);
    }
}
/// The signed-in person and whether they requested this chat.
pub(crate) struct Speakers {
    pub(crate) person: Option<u64>,
    pub(crate) requester: bool,
}

pub(crate) fn speakers(task: &c::Task, me: Option<&c::Me>) -> Speakers {
    let person = match me {
        Some(me) => Some(me.party.number),
        None => None,
    };
    let requester = match task.requester {
        c::Requester::Person(number) => Some(number) == person,
        c::Requester::Task(_) | c::Requester::Deployment { .. } => false,
    };
    Speakers { person, requester }
}

pub(crate) fn context(b: &mut Builder, root: &Domain, project: u32, words: &Words) {
    let Some(playground) = root.forge().playground() else { return };
    if Some(project) != playground.project || playground.state != temper_client_domain::forge::PlaygroundState::Running
    {
        return;
    }
    let Some(me) = root.pages().me(root.client()) else { return };
    let project = match super::project_name(Some(me), project) {
        Some(name) => name,
        None => words.say(Word::Project, &[Hole::Number(u64::from(project))]),
    };
    b.open(Role::Slot);
    b.style(look::CONTEXT);
    b.style(look::META_LINE);
    let repository = words
        .say(Word::PlaygroundRepository, &[Hole::Name(playground.owner.clone()), Hole::Name(playground.name.clone())]);
    b.open(Role::Paragraph);
    b.text(Run::Plain, &words.say(Word::PlaygroundContextProject, &[Hole::Name(project)]));
    b.open(Role::Text);
    b.style(look::REPOSITORY);
    b.run(Run::Plain, &repository);
    b.close();
    words.text(b, Word::PlaygroundRepositoryForge, &[]);
    b.close();
    let mut owner = false;
    for project in &me.projects {
        if Some(project.project) == playground.project && project.role == c::HeldRole::Owner {
            owner = true;
            break;
        }
    }
    let signed = if me.deployment_role == Some(c::HeldRole::Owner) {
        words.say(Word::SignedDeploymentOwner, &[])
    } else if owner {
        words.say(Word::SignedProjectOwner, &[])
    } else {
        words.say(Word::SignedName, &[Hole::Name(skein_lib::bytes::copy_of(me.party.name.bytes()))])
    };
    text(b, Role::Paragraph, &signed);
    b.close();
}

// A proposal keeps one keyed card at its first visible call, or in the tail.
fn proposal_card(
    b: &mut Builder,
    root: &Domain,
    card: &temper_client_domain::pages::inbox::Card,
    words: &Words,
    bindings: &mut Bindings,
    (rows, times): (u32, Option<&mut Times>),
    role: Role,
) {
    b.open(role);
    b.name(&super::inbox::card_name(root, card.object, Word::Proposal, words));
    if crate::cards::proposal::settings_proposal(root, card).is_none() {
        b.style(look::CARD);
        b.style(look::KIND_PROPOSAL);
    }
    if role == Role::Item {
        b.key(super::inbox::card_key(card.object));
    }
    if let Some(page) = &card.page
        && let Some(since) = super::inbox::waiting_since(page)
        && let Some(times) = times
    {
        proposal_wait(b, words, times, since);
    }
    let replaced = settings_replaced(root, card.object, rows);
    if !crate::cards::proposal::settings(b, root, card, words, bindings, replaced) {
        crate::confirm::card(b, root, card, words, bindings, rows);
    }
    b.close();
}

// Only assistant calls inside the displayed row bounds can stand for a card.
fn proposal_anchor(chat: &Chat, object: c::ObjectKey, rows: u32) -> Option<(c::TurnName, usize, usize)> {
    let bound = usize::try_from(rows).expect("row bound fits");
    for line in chat.thread.lines().into_boxed() {
        if let Line::Turn { name, turn, .. } = line {
            for (message_at, message) in turn.messages.iter().take(bound).enumerate() {
                if message.speaker != c::Speaker::Assistant {
                    continue;
                }
                for (block_at, block) in message.blocks.iter().take(bound).enumerate() {
                    if let c::Block::Call(call) = block
                        && call.objects.as_slice().contains(&object)
                    {
                        return Some((name, message_at, block_at));
                    }
                }
            }
        }
    }
    None
}

// A proposal's wait, muted beside its first line as the inbox's.
fn proposal_wait(b: &mut Builder, words: &Words, times: &mut Times, since: skein_lib::Wall) {
    let said = words.say(Word::TimeWaitingChat, &[Hole::Name(times.say(words, TimeForm::Span, since))]);
    b.open(Role::Paragraph);
    b.name(&said);
    b.style(look::META);
    b.text(Run::Plain, &said);
    b.close();
}
fn end_clock(task: &c::Task, words: &Words, times: &mut Times) -> Option<Box<[u8]>> {
    let at = task.chip.details.details.ended_at;
    match at {
        Some(at) => Some(times.say(words, TimeForm::Clock, at)),
        None => None,
    }
}
fn reason_bytes(reason: Option<&c::Text>) -> Option<&[u8]> {
    match reason {
        Some(reason) => Some(reason.bytes()),
        None => None,
    }
}
// A chat owns one end line; goal/result cards keep their independent drawing.
fn result_text(b: &mut Builder, result: &c::TaskResult, requester: c::Requester, words: &Words, clock: Option<&[u8]>) {
    match result {
        c::TaskResult::Ending(ending) => owner_end(b, ending, requester, words, clock),
        c::TaskResult::Document(result) => match &result.ending {
            c::Ending::Task(ending) => owner_end(b, ending, requester, words, clock),
            c::Ending::Failed { reason } => {
                end_reason(b, Word::ChatFailed, reason_bytes(reason.as_ref()), words, clock);
            }
            c::Ending::Cancelled { reason, by, .. } => {
                let by = match by {
                    c::Actor::Party(number) => Some(c::Requester::Person(*number)),
                    c::Actor::Task(number) => Some(c::Requester::Task(*number)),
                    c::Actor::ProjectDeployment { project } => Some(c::Requester::Deployment { project: *project }),
                    c::Actor::Deployment => None,
                };
                if let Some(by) = by
                    && closed_by_requester(requester, by)
                {
                    end_closed(b, words, clock);
                    return;
                }
                end_reason(b, Word::ChatCancelled, reason_bytes(reason.as_ref()), words, clock);
            }
            c::Ending::Withdrawn { reason } => end_reason(b, Word::ChatCancelled, Some(reason.bytes()), words, clock),
            c::Ending::Done | c::Ending::Deleted | c::Ending::Decided { .. } => {
                end_report(b, result.value.as_ref(), words, clock);
            }
        },
    }
}
fn owner_end(b: &mut Builder, ending: &c::ChipEnding, requester: c::Requester, words: &Words, clock: Option<&[u8]>) {
    match ending {
        c::ChipEnding::Done(value) => end_report(b, Some(value), words, clock),
        c::ChipEnding::Failed(reason) => {
            end_reason(b, Word::ChatFailed, Some(reason.bytes()), words, clock);
        }
        c::ChipEnding::Cancelled { reason, by, .. } => {
            if closed_by_requester(requester, *by) {
                end_closed(b, words, clock);
            } else {
                end_reason(b, Word::ChatCancelled, Some(reason.bytes()), words, clock);
            }
        }
    }
}
pub(crate) fn closed_by_requester(requester: c::Requester, by: c::Requester) -> bool {
    requester == by
}
fn end_closed(b: &mut Builder, words: &Words, clock: Option<&[u8]>) {
    let said = match clock {
        Some(clock) => words.say(Word::TimeClosed, &[crate::cards::name(clock)]),
        None => words.say(Word::ChatClosed, &[]),
    };
    end_line(b, &said);
}
fn end_reason(b: &mut Builder, word: Word, reason: Option<&[u8]>, words: &Words, clock: Option<&[u8]>) {
    let reason = match reason {
        Some(reason) => reason,
        None => b"",
    };
    let said = match clock {
        Some(clock) => words.say(
            if word == Word::ChatFailed { Word::TimeFailed } else { Word::TimeCancelled },
            &[crate::cards::name(clock), crate::cards::name(reason)],
        ),
        None => words.say(word, &[crate::cards::name(reason)]),
    };
    end_line(b, &said);
}
fn end_report(b: &mut Builder, value: Option<&c::ResultValue>, words: &Words, clock: Option<&[u8]>) {
    let said = match value {
        Some(c::ResultValue::Report(report)) if !report.bytes().is_empty() => match clock {
            Some(clock) => {
                words.say(Word::TimeEndedReport, &[crate::cards::name(clock), crate::cards::name(report.bytes())])
            }
            None => words.say(Word::ChatEndedReport, &[crate::cards::name(report.bytes())]),
        },
        _ => match clock {
            Some(clock) => words.say(Word::TimeEnded, &[crate::cards::name(clock)]),
            None => words.say(Word::ChatEnded, &[]),
        },
    };
    end_line(b, &said);
}
fn end_line(b: &mut Builder, value: &[u8]) {
    b.open(Role::Paragraph);
    b.style(look::ENDED);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
fn spend(b: &mut Builder, task: &c::Task, words: &Words) {
    text(
        b,
        Role::Paragraph,
        &words.say(
            Word::ChatbarSpend,
            &[Hole::Number(task.budget.numbers.spent), Hole::Number(task.budget.numbers.budget)],
        ),
    );
}
/// The chat projects a request and assistant messages; an agent keeps every work line.
pub(crate) enum Drawn<'a> {
    ChatPage {
        request: Option<&'a c::SpecificationDocument>,
        root: &'a Domain,
    },
    #[cfg(test)]
    Chat {
        request: Option<&'a c::SpecificationDocument>,
        delegations: Option<(&'a Chat, &'a jig_client::Client)>,
    },
    Lines,
}

struct Drawing<'a> {
    projected: bool,
    request: Option<&'a c::SpecificationDocument>,
    delegations: Option<(&'a Chat, &'a jig_client::Client)>,
    proposals: Option<&'a Domain>,
}
fn drawing(drawn: Drawn<'_>) -> Drawing<'_> {
    match drawn {
        Drawn::ChatPage { request, root } => Drawing {
            projected: true,
            request,
            delegations: Some((root.pages().chat().expect("chat page"), root.client())),
            proposals: Some(root),
        },
        #[cfg(test)]
        Drawn::Chat { request, delegations } => Drawing { projected: true, request, delegations, proposals: None },
        Drawn::Lines => Drawing { projected: false, request: None, delegations: None, proposals: None },
    }
}

pub(crate) fn conversation(
    b: &mut Builder,
    thread: &Thread,
    words: &Words,
    limits: (u32, u32),
    speakers: Speakers,
    drawn: Drawn<'_>,
    bindings: &mut Bindings,
) {
    conversation_with_times(b, thread, words, limits, speakers, (drawn, None), bindings);
}

// Agent task pages keep their own treatment of moments (client/README.md, 5.12).
#[expect(clippy::too_many_lines, reason = "the conversation walk interleaves its run, text and card anchors")]
pub(crate) fn conversation_with_times(
    b: &mut Builder,
    thread: &Thread,
    words: &Words,
    limits: (u32, u32),
    speakers: Speakers,
    (drawn, mut times): (Drawn<'_>, Option<&mut Times>),
    bindings: &mut Bindings,
) {
    let (rows, text_limit) = limits;
    let Drawing { projected, request, delegations, proposals } = drawing(drawn);
    let bound = match delegations {
        Some((chat, _)) => chat.delegations.capacity(),
        None => 0,
    };
    let mut tasks = List::with_capacity(bound);
    let mut shown = List::with_capacity(bound);
    b.open(Role::Log);
    b.name(&words.say_card(P::ConversationLog, &[]));
    if let Some(root) = proposals {
        b.open(Role::Slot);
        b.style(look::THREAD_INNER);
        conversation_head(b, root, words, bindings);
    }
    let lines = thread.lines().into_boxed();
    let mut run = super::chat_line::Run::with_capacity(
        rows.checked_mul(rows)
            .expect("bounded blocks")
            .checked_mul(temper_client_domain::pages::chat::LINES)
            .expect("window calls fit"),
    );
    let mut started = super::chat_sheet::started(proposals);
    let mut day = None;
    // A chat started with no words opens on its first words line (3.5).
    if thread.lines.edges.after == jig_client::thread::window::Edge::End
        && let Some(request) = request
        && !request.specification.words.bytes().is_empty()
    {
        let clock = turn_time(b, first_time(&lines), &mut times, &mut day, words);
        request_message(b, request, speakers.requester, words, clock.as_deref());
    }
    for line in lines {
        if let Some((at, _)) = started
            && let Some(time) = match &line {
                Line::Activity { line } => Some(line.source.at),
                Line::Words { .. }
                | Line::Turn { .. }
                | Line::Undecoded { .. }
                | Line::Provisional { .. }
                | Line::Left { .. } => line_time(&line),
            }
            && time > skein_lib::Wall::from_nanos(at)
        {
            run.flush(b, None, false, words);
        }
        super::chat_sheet::before(b, &mut started, Some(&line), words, bindings);
        if hidden_line(&line, projected, delegations) {
            if let Line::Activity { line: event } = &line
                && let Some(task) = delegation::heard(&event.source.line)
                && !shown.as_slice().contains(&task)
            {
                run.flush(b, None, false, words);
            }
            heard_card(b, &line, delegations, (&mut tasks, &mut shown), words, bindings);
            continue;
        }
        if let Line::Turn { name, turn, at, .. } = line {
            committed(
                b,
                turn,
                words,
                rows,
                Committed {
                    delegations,
                    proposals: match proposals {
                        Some(root) => Some((root, name)),
                        None => None,
                    },
                    times: times.as_deref_mut(),
                    day: &mut day,
                    at,
                    projected,
                    requester: speakers.requester,
                },
                (&mut tasks, &mut shown),
                bindings,
                &mut run,
            );
            continue;
        }
        run.flush(b, None, false, words);
        let clock = turn_time(b, line_time(&line), &mut times, &mut day, words);
        b.open(Role::Region);
        b.style(look::TURN);
        match line {
            Line::Words { by, words: message, state, .. } => {
                if projected {
                    person_words(b, by, message, state, speakers.person, words, clock.as_deref());
                } else {
                    person_lines(b, by, message, state, speakers.person, words, clock.as_deref());
                }
            }
            Line::Turn { .. } => unreachable!("turn drawn above"),
            Line::Provisional { text, .. } => provisional(b, text, words, text_limit),
            Line::Undecoded { why, .. } => undecoded(b, why, words, clock.as_deref()),
            Line::Left { lines } => {
                b.style(look::MARKER);
                text(b, Role::Status, &words.say(Word::LinesLeft, &[Hole::Number(u64::from(lines))]));
            }
            Line::Activity { line } => {
                owner_line(b, &line.source.line, projected, speakers.person, words, clock.as_deref());
            }
        }
        b.close();
    }
    let running = match proposals {
        Some(root) => super::chats::working(root, words, thread),
        None => false,
    };
    let current = if running {
        match proposals {
            Some(root) => match root.pages().chat() {
                Some(chat) => chat.current_call(),
                None => None,
            },
            None => None,
        }
    } else {
        None
    };
    let running_run = running && (!run.is_empty() || current.is_some());
    run.flush(b, current, running_run, words);
    super::chat_sheet::before(b, &mut started, None, words, bindings);
    // The live delegates anchor follows the window's last line (client/README.md, 5.7).
    live_delegates(b, &mut tasks, delegations, &mut shown, words, bindings);
    if let Some(root) = proposals {
        conversation_tail(b, root, words, bindings, rows, times.expect("chat page times"));
        if words.shell == crate::words::Shell::Browser && running && !running_run {
            b.open(Role::Slot);
            b.style(look::TYPING);
            b.name(&words.say(Word::ChatWorking, &[]));
            for _ in 0_u32..3 {
                b.open(Role::Slot);
                b.close();
            }
            b.close();
        }
        b.close();
    }
    b.close();
}
fn undecoded(b: &mut Builder, why: c::TurnProblem, words: &Words, clock: Option<&[u8]>) {
    match why {
        c::TurnProblem::Version { found } => {
            words.status(b, P::TurnUnreadVersion, &[Hole::Number(u64::from(found))]);
        }
        c::TurnProblem::Malformed => words.status(b, P::TurnUnreadMalformed, &[]),
        c::TurnProblem::TooLarge { .. } => words.status(b, P::TurnUnreadTooLarge, &[]),
    }
    if clock.is_some() {
        b.open(Role::Slot);
        b.style(look::WHO);
        turn_clock(b, clock);
        b.close();
    }
}
fn request_message(
    b: &mut Builder,
    request: &c::SpecificationDocument,
    requester: bool,
    words: &Words,
    clock: Option<&[u8]>,
) {
    b.open(Role::Region);
    b.style(look::TURN);
    let label = if requester { words.say_card(P::You, &[]) } else { words.say_card(P::Person, &[]) };
    person_message(b, &label, request.specification.words.bytes(), WordState::Read { message: 0 }, words, clock);
    b.close();
}
fn first_time(lines: &[Line<'_>]) -> Option<skein_lib::Wall> {
    for line in lines {
        if let Some(at) = line_time(line) {
            return Some(at);
        }
    }
    None
}
fn line_time(line: &Line<'_>) -> Option<skein_lib::Wall> {
    match line {
        Line::Words { at, .. } => *at,
        Line::Turn { at, .. } | Line::Undecoded { at, .. } => Some(*at),
        Line::Activity { line } => match line.source.line {
            c::Activity::Words { .. } => Some(line.source.at),
            c::Activity::Heard(_)
            | c::Activity::Notified { .. }
            | c::Activity::RunBegan { .. }
            | c::Activity::RunEnded { .. }
            | c::Activity::Turn { .. }
            | c::Activity::Decided { .. }
            | c::Activity::Milestone { .. }
            | c::Activity::Kept { .. } => None,
        },
        Line::Provisional { .. } | Line::Left { .. } => None,
    }
}
fn turn_time(
    b: &mut Builder,
    at: Option<skein_lib::Wall>,
    times: &mut Option<&mut Times>,
    day: &mut Option<i64>,
    words: &Words,
) -> Option<Box<[u8]>> {
    let at = at?;
    let times = times.as_mut()?;
    let date = words.calendar_day(at);
    if *day != Some(date) {
        let said = times.say(words, TimeForm::Day, at);
        b.open(Role::Paragraph);
        b.style(look::DAY);
        b.name(&said);
        b.text(Run::Plain, &said);
        b.close();
        *day = Some(date);
    }
    Some(words.turn_clock(at))
}
fn turn_clock(b: &mut Builder, clock: Option<&[u8]>) {
    if let Some(clock) = clock {
        text(b, Role::Time, clock);
    }
}
/// A delegate's heard result anchors its card, as a result block does (client/README.md, 5.7).
fn heard_card(
    b: &mut Builder,
    line: &Line<'_>,
    delegations: Option<(&Chat, &jig_client::Client)>,
    (tasks, shown): (&mut List<u64>, &mut List<u64>),
    words: &Words,
    bindings: &mut Bindings,
) {
    if let Line::Activity { line } = line
        && let Some(task) = delegation::heard(&line.source.line)
        && !shown.as_slice().contains(&task)
    {
        tasks.clear();
        tasks.push(task).expect("one heard delegate fits");
        b.open(Role::Region);
        b.style(look::TURN);
        cards(b, tasks, delegations, shown, words, bindings);
        b.close();
    }
}
fn provisional(b: &mut Builder, message: &[u8], words: &Words, text_limit: u32) {
    b.open(Role::Slot);
    b.style(look::MESSAGE);
    b.style(look::BY_AGENT);
    b.open(Role::Slot);
    b.style(look::WHO);
    words.card_text(b, P::Temper, &[]);
    b.close();
    b.open(Role::Slot);
    b.style(look::SAYS);
    markdown(b, message, text_limit);
    writing_end(b, words);
    b.close();
    b.close();
}
/// The words being written end in a mark (temper/docs/design/client/README.md, 3.5).
fn writing_end(b: &mut Builder, words: &Words) {
    match words.shell {
        crate::words::Shell::Browser => {
            b.open(Role::Text);
            b.style(look::WRITING_END);
            b.close();
        }
        crate::words::Shell::Terminal => text(b, Role::Status, "▌".as_bytes()),
    }
}
fn owner_line(
    b: &mut Builder,
    line: &c::Activity,
    projected: bool,
    person: Option<u64>,
    words: &Words,
    clock: Option<&[u8]>,
) {
    if projected && let c::Activity::Heard(c::Word { kind: c::MessageKind::QuestionAnswered { question }, .. }) = line {
        text(b, Role::Paragraph, &words.say(Word::QuestionAnswered, &[Hole::Number(*question)]));
        return;
    }
    if let c::Activity::Words { by, words: message, .. } = line {
        let (label, side) = match by {
            c::Requester::Person(by) => (
                if Some(*by) == person { words.say_card(P::You, &[]) } else { words.say_card(P::Person, &[]) },
                look::BY_PERSON,
            ),
            c::Requester::Task(task) => (
                words.say(Word::TaskWho, &[Hole::Number(*task)]),
                if projected { look::BY_DELEGATE } else { look::BY_AGENT },
            ),
            c::Requester::Deployment { .. } => {
                super::task::owner_activity(b, line, words);
                return;
            }
        };
        words_message(b, &label, message.bytes(), WordState::Read { message: 0 }, side, words, clock);
        return;
    }
    super::task::owner_activity(b, line, words);
}
fn hidden_line(line: &Line<'_>, projected: bool, delegations: Option<(&Chat, &jig_client::Client)>) -> bool {
    match line {
        Line::Activity { line } => hidden(line, projected, delegations),
        Line::Words { .. }
        | Line::Turn { .. }
        | Line::Undecoded { .. }
        | Line::Provisional { .. }
        | Line::Left { .. } => false,
    }
}

fn live_delegates(
    b: &mut Builder,
    tasks: &mut List<u64>,
    delegations: Option<(&Chat, &jig_client::Client)>,
    shown: &mut List<u64>,
    words: &Words,
    bindings: &mut Bindings,
) {
    tasks.clear();
    if let Some((chat, client)) = delegations
        && let Some(document) = chat.task(client)
    {
        for delegate in &document.delegates {
            let task = delegate.chip.task;
            if !shown.as_slice().contains(&task) && chat.delegation(client, task).is_some() && tasks.push(task).is_err()
            {
                break;
            }
        }
    }
    if !tasks.is_empty() {
        b.open(Role::Region);
        b.style(look::TURN);
        cards(b, tasks, delegations, shown, words, bindings);
        b.close();
    }
}
fn person_words(
    b: &mut Builder,
    by: Option<&c::Party>,
    message: &[u8],
    state: WordState,
    person: Option<u64>,
    words: &Words,
    clock: Option<&[u8]>,
) {
    let label = match by {
        Some(by) if Some(by.number) != person => skein_lib::bytes::copy_of(by.name.bytes()),
        Some(_) | None => words.say_card(P::You, &[]),
    };
    person_message(b, &label, message, state, words, clock);
}
fn person_message(
    b: &mut Builder,
    label: &[u8],
    message: &[u8],
    state: WordState,
    words: &Words,
    clock: Option<&[u8]>,
) {
    words_message(b, label, message, state, look::BY_PERSON, words, clock);
}
fn words_message(
    b: &mut Builder,
    label: &[u8],
    message: &[u8],
    state: WordState,
    side: skein_ui::Style,
    words: &Words,
    clock: Option<&[u8]>,
) {
    b.open(Role::Slot);
    b.style(look::MESSAGE);
    b.style(side);
    b.open(Role::Slot);
    b.style(look::WHO);
    text(b, Role::Paragraph, label);
    turn_clock(b, clock);
    match state {
        WordState::BeingSent => words.status(b, P::WordsBeingSent, &[]),
        WordState::Sent { .. } => words.status(b, P::WordsSent, &[]),
        WordState::Read { .. } => {}
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::SAYS);
    let mut start = 0;
    for (end, byte) in message.iter().enumerate() {
        if *byte == b'\n' {
            text(b, Role::Paragraph, message.get(start..end).expect("paragraph range"));
            start = end.checked_add(1).expect("next paragraph within message");
        }
    }
    text(b, Role::Paragraph, message.get(start..).expect("last paragraph"));
    b.close();
    b.close();
}

fn person_lines(
    b: &mut Builder,
    by: Option<&c::Party>,
    message: &[u8],
    state: WordState,
    person: Option<u64>,
    words: &Words,
    clock: Option<&[u8]>,
) {
    b.open(Role::Slot);
    b.style(look::MESSAGE);
    b.style(look::BY_PERSON);
    b.open(Role::Slot);
    b.style(look::WHO);
    match by {
        Some(by) if Some(by.number) != person => text(b, Role::Paragraph, by.name.bytes()),
        Some(_) | None => text(b, Role::Paragraph, &words.say_card(P::You, &[])),
    }
    turn_clock(b, clock);
    words.status(
        b,
        match state {
            WordState::BeingSent => P::WordsBeingSent,
            WordState::Sent { .. } => P::WordsSent,
            WordState::Read { .. } => P::WordsRead,
        },
        &[],
    );
    b.close();
    b.open(Role::Slot);
    b.style(look::SAYS);
    text(b, Role::Paragraph, message);
    b.close();
    b.close();
}

// Decoded turns already share the receiving turn_text budget; both drawings
// retain their text whole, independently of provisional/field text.
struct Committed<'a> {
    delegations: Option<(&'a Chat, &'a jig_client::Client)>,
    proposals: Option<(&'a Domain, c::TurnName)>,
    times: Option<&'a mut Times>,
    day: &'a mut Option<i64>,
    at: skein_lib::Wall,
    projected: bool,
    requester: bool,
}
#[expect(clippy::too_many_arguments, reason = "the block walk shares its drawing state and run with the conversation")]
#[expect(clippy::too_many_lines, reason = "one ordered walk keeps text, run calls and card anchors at their block")]
fn committed<'a>(
    b: &mut Builder,
    turn: &'a c::Turn,
    words: &Words,
    rows: u32,
    mut drawing: Committed<'_>,
    lists: (&mut List<u64>, &mut List<u64>),
    bindings: &mut Bindings,
    run: &mut super::chat_line::Run<'a>,
) {
    let (tasks, shown) = lists;
    let mut dated = false;
    let bound = usize::try_from(rows).expect("row bound fits");
    for (message_at, message) in turn.messages.iter().take(bound).enumerate() {
        for (block_at, block) in message.blocks.iter().take(bound).enumerate() {
            if let c::Block::Call(call) = block
                && (!drawing.projected || message.speaker == c::Speaker::Assistant)
                && run_call(block, tasks, drawing.delegations)
            {
                if unshown_card(tasks, drawing.delegations, shown) {
                    run.flush(b, None, false, words);
                    let clock = committed_clock(b, &mut drawing, &mut dated, words);
                    b.open(Role::Region);
                    b.style(look::TURN);
                    card_clock(b, clock.as_deref());
                    cards(b, tasks, drawing.delegations, shown, words, bindings);
                    b.close();
                }
                run.call(call, drawing.at);
                continue;
            }
            if let c::Block::Call(call) = block
                && !call_draws(call, &drawing, (message_at, block_at), rows, tasks, shown)
            {
                continue;
            }
            if let c::Block::Result { call, summary, .. } = block {
                tasks.clear();
                delegation::results(summary.bytes(), tasks);
                if !requested(tasks, drawing.delegations) {
                    if run.has_call(*call) {
                        run.result(*call, summary);
                        continue;
                    }
                    if drawing.projected {
                        continue;
                    }
                }
                if requested(tasks, drawing.delegations) {
                    let mut all_shown = true;
                    for task in tasks.iter() {
                        all_shown &= shown.as_slice().contains(task);
                    }
                    if all_shown {
                        continue;
                    }
                }
            }
            if let c::Block::Opaque { .. } = block {
                continue;
            }
            if let c::Block::Text(value) = block
                && value.bytes().is_empty()
            {
                continue;
            }
            if drawing.projected && message.speaker != c::Speaker::Assistant {
                if let c::Block::Result { .. } = block {
                } else {
                    continue;
                }
            }
            run.flush(b, None, false, words);
            let clock = committed_clock(b, &mut drawing, &mut dated, words);
            b.open(Role::Region);
            b.style(look::TURN);
            match block {
                c::Block::Text(value) => {
                    b.open(Role::Slot);
                    b.style(look::MESSAGE);
                    b.style(match message.speaker {
                        c::Speaker::Assistant => look::BY_AGENT,
                        c::Speaker::Person => look::BY_PERSON,
                        c::Speaker::Tool => look::BY_TOOL,
                        c::Speaker::System => look::BY_SYSTEM,
                    });
                    b.open(Role::Slot);
                    b.style(look::WHO);
                    words.card_text(
                        b,
                        match message.speaker {
                            c::Speaker::Assistant => P::Temper,
                            c::Speaker::Person => {
                                if drawing.requester {
                                    P::You
                                } else {
                                    P::Person
                                }
                            }
                            c::Speaker::Tool => P::CallCompleted,
                            c::Speaker::System => P::TurnInstructions,
                        },
                        &[],
                    );
                    turn_clock(b, clock.as_deref());
                    b.close();
                    b.open(Role::Slot);
                    b.style(look::SAYS);
                    skein_ui::markdown::read(value.bytes(), 2, b);
                    b.close();
                    b.close();
                }
                c::Block::Opaque { .. } => unreachable!("opaque blocks are hidden"),
                c::Block::Call(call) => {
                    card_clock(b, clock.as_deref());
                    super::document::tool_line(
                        b,
                        call,
                        match drawing.delegations {
                            Some((chat, _)) => Some(chat),
                            None => None,
                        },
                        words,
                        bindings,
                    );
                    tasks.clear();
                    delegation::delegated(call, tasks);
                    cards(b, tasks, drawing.delegations, shown, words, bindings);
                    if let Some((root, name)) = drawing.proposals {
                        let chat = root.pages().chat().expect("chat page");
                        for card in &chat.cards {
                            if call.objects.as_slice().contains(&card.object)
                                && proposal_anchor(chat, card.object, rows) == Some((name, message_at, block_at))
                            {
                                proposal_card(
                                    b,
                                    root,
                                    card,
                                    words,
                                    bindings,
                                    (rows, drawing.times.as_deref_mut()),
                                    Role::Region,
                                );
                            }
                        }
                    }
                }
                c::Block::Result { summary, failed, .. } => {
                    card_clock(b, clock.as_deref());
                    if requested(tasks, drawing.delegations) {
                        cards(b, tasks, drawing.delegations, shown, words, bindings);
                    } else {
                        b.open(Role::Slot);
                        b.style(look::MESSAGE);
                        b.style(look::BY_TOOL);
                        b.open(Role::Slot);
                        b.style(look::WHO);
                        words.card_text(b, P::CallCompleted, &[]);
                        b.close();
                        b.open(Role::Slot);
                        b.style(look::SAYS);
                        b.open(Role::Slot);
                        b.style(look::WORK);
                        words.card_text(b, if *failed { P::CallFailed } else { P::CallCompleted }, &[]);
                        text(b, Role::Paragraph, summary.bytes());
                        b.close();
                        b.close();
                        b.close();
                    }
                }
            }
            b.close();
        }
    }
}

fn call_draws(
    call: &c::Call,
    drawing: &Committed<'_>,
    at: (usize, usize),
    rows: u32,
    tasks: &mut List<u64>,
    shown: &List<u64>,
) -> bool {
    if super::document::write(call).is_some() {
        return true;
    }
    tasks.clear();
    delegation::delegated(call, tasks);
    if unshown_card(tasks, drawing.delegations, shown) {
        return true;
    }
    let Some((root, name)) = drawing.proposals else { return false };
    let chat = root.pages().chat().expect("chat page");
    for card in &chat.cards {
        if call.objects.as_slice().contains(&card.object)
            && proposal_anchor(chat, card.object, rows) == Some((name, at.0, at.1))
        {
            return true;
        }
    }
    false
}
fn committed_clock(b: &mut Builder, drawing: &mut Committed<'_>, dated: &mut bool, words: &Words) -> Option<Box<[u8]>> {
    if *dated {
        return None;
    }
    *dated = true;
    turn_time(b, Some(drawing.at), &mut drawing.times, drawing.day, words)
}
fn card_clock(b: &mut Builder, clock: Option<&[u8]>) {
    if clock.is_some() {
        b.open(Role::Slot);
        b.style(look::WHO);
        turn_clock(b, clock);
        b.close();
    }
}

fn run_call(block: &c::Block, tasks: &mut List<u64>, delegations: Option<(&Chat, &jig_client::Client)>) -> bool {
    let c::Block::Call(call) = block else { return false };
    if super::document::write(call).is_some() {
        return false;
    }
    if let Some((chat, _)) = delegations {
        for card in &chat.cards {
            if call.objects.as_slice().contains(&card.object) {
                return false;
            }
        }
    }
    tasks.clear();
    delegation::delegated(call, tasks);
    !carded(tasks, delegations)
}
fn unshown_card(tasks: &List<u64>, delegations: Option<(&Chat, &jig_client::Client)>, shown: &List<u64>) -> bool {
    let Some((chat, client)) = delegations else { return false };
    for task in tasks {
        if !shown.as_slice().contains(task) && chat.delegation(client, *task).is_some() {
            return true;
        }
    }
    false
}
fn cards(
    b: &mut Builder,
    tasks: &List<u64>,
    delegations: Option<(&Chat, &jig_client::Client)>,
    shown: &mut List<u64>,
    words: &Words,
    bindings: &mut Bindings,
) {
    let Some((chat, client)) = delegations else { return };
    for task in tasks {
        if !shown.as_slice().contains(task)
            && let Some(document) = chat.delegation(client, *task)
        {
            crate::cards::delegation::card(b, document, words, bindings);
            shown.push(*task).expect("shown delegations fit their bound");
        }
    }
}
fn requested(tasks: &List<u64>, delegations: Option<(&Chat, &jig_client::Client)>) -> bool {
    let Some((chat, client)) = delegations else { return false };
    if tasks.is_empty() {
        return false;
    }
    for task in tasks {
        if chat.requested(client, *task).is_none() {
            return false;
        }
    }
    true
}
fn carded(tasks: &List<u64>, delegations: Option<(&Chat, &jig_client::Client)>) -> bool {
    let Some((chat, client)) = delegations else { return false };
    if tasks.is_empty() {
        return false;
    }
    for task in tasks {
        if chat.delegation(client, *task).is_none() {
            return false;
        }
    }
    true
}
fn hidden(event: &c::DecodedActivityLine, projected: bool, delegations: Option<(&Chat, &jig_client::Client)>) -> bool {
    {
        let c::DecodedActivityLine { source, .. } = event;
        match &source.line {
            c::Activity::Milestone { .. } => return true,
            c::Activity::RunBegan { .. } | c::Activity::RunEnded { .. } => {
                if projected {
                    return true;
                }
            }
            c::Activity::Words { .. }
            | c::Activity::Heard(_)
            | c::Activity::Notified { .. }
            | c::Activity::Turn { .. }
            | c::Activity::Decided { .. }
            | c::Activity::Kept { .. } => {}
        }
    }
    let Some((chat, client)) = delegations else { return false };
    let task = delegation::heard(&event.source.line);
    match task {
        Some(task) => chat.delegation(client, task).is_some(),
        None => false,
    }
}
fn markdown(b: &mut Builder, bytes: &[u8], most: u32) {
    let most = bytes.len().min(usize::try_from(most).expect("text bound fits"));
    let mut end = most;
    if end < bytes.len() {
        // The protocol validated UTF-8; align a display window to the preceding character.
        for index in most.saturating_sub(3)..=most {
            if let Some(byte) = bytes.get(index)
                && byte & 0xc0 != 0x80
            {
                end = index;
            }
        }
    }
    skein_ui::markdown::read(bytes.get(..end).expect("UTF-8 prefix"), 2, b);
}

// Only a later settings proposal anchored in this conversation replaces an ended one.
fn settings_replaced(root: &Domain, object: c::ObjectKey, rows: u32) -> bool {
    let Some(chat) = root.pages().chat() else { return false };
    let Some(anchor) = proposal_anchor(chat, object, rows) else { return false };
    let mut reached = false;
    for line in chat.thread.lines().into_boxed() {
        if let Line::Turn { name, turn, .. } = line {
            for (message, value) in turn.messages.iter().enumerate() {
                for (block, value) in value.blocks.iter().enumerate() {
                    if (name, message, block) == anchor {
                        reached = true;
                        continue;
                    }
                    if reached && let c::Block::Call(call) = value {
                        for card in &chat.cards {
                            if call.objects.as_slice().contains(&card.object)
                                && card.object != object
                                && proposal_anchor(chat, card.object, rows) == Some((name, message, block))
                                && crate::cards::proposal::settings_proposal(root, card).is_some()
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}
