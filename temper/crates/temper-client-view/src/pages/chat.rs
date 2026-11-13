//! A bound conversation reads at either edge; Latest returns to its end (client/pages/chat/conversation.md, 2).
use crate::{
    bindings::Bindings,
    words::{Shell, TimeForm, Times, Word, Words},
};
use crate::{cards::P, look};
use alloc::boxed::Box;
use jig_client::{
    contract as c,
    thread::{Line, Thread, Words as WordState},
};
use skein_lib::List;
use skein_ui::{Builder, Key, Role, Run, State, phrases::Hole};
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
#[expect(clippy::too_many_lines, reason = "the page branches between its browser bar and terminal frame")]
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    rows: u32,
    text_limit: u32,
    times: &mut Times,
) {
    let terminal = super::terminal::shown(root, words);
    b.open(Role::Main);
    if terminal {
        super::terminal::style(b, root.pages().look());
    }
    b.style(super::document::mode(root.pages().chat()));
    let asking = asking(root);
    b.style(look::SESSION);
    if let Some(chat) = root.pages().chat()
        && let Some(task) = chat.task(root.client())
    {
        let short = crate::shared::heading(task.chip.title.bytes());
        let whole = words.say(
            Word::ChatTitle,
            &[Hole::Number(chat.task), Hole::Name(skein_lib::bytes::copy_of(task.chip.title.bytes()))],
        );
        let numbered =
            words.say(Word::ChatTitle, &[Hole::Number(chat.task), Hole::Name(skein_lib::bytes::copy_of(&short))]);
        let hint = if short.as_ref() == task.chip.title.bytes() { None } else { Some(task.chip.title.bytes()) };
        if terminal {
            let draft = if chat.draft.is_some() || chat.writing {
                if chat.mode == temper_client_domain::pages::chat::document::Mode::Chat {
                    super::terminal::Draft::Hidden
                } else {
                    super::terminal::Draft::Shown
                }
            } else if task.result.is_none()
                && !super::chats::ended(&task.chip)
                && temper_client_domain::pages::chat_offers_draft(root, task)
            {
                super::terminal::Draft::Add
            } else {
                super::terminal::Draft::None
            };
            super::terminal::top(
                b,
                root,
                words,
                bindings,
                super::terminal::Top {
                    project: Some(task.project),
                    heading: &short,
                    name: &whole,
                    hint,
                    number: Some(task.chip.task),
                    draft,
                },
            );
        } else {
            super::session::top(
                b,
                root,
                words,
                bindings,
                Some(task.project),
                if asking { super::session::Path::Ask } else { super::session::Path::Chats },
                super::session::Heading::Chat { whole: &whole, short: &short, numbered: &numbered, hint },
            );
            super::session::right(b);
            if task.result.is_some() || super::chats::ended(&task.chip) {
                let line = crate::choice::ran_line(root, task, &chat.thread, words);
                super::session::closed_number(b, words, task.chip.task, line.as_deref());
            } else {
                super::session::number(b, words, task.chip.task);
            }
            if asking {
                b.open(Role::Paragraph);
                b.style(look::CHATS_TAG);
                words.text(b, Word::ToolSeparator, &[]);
                words.text(b, Word::Asking, &[]);
                b.close();
            }
            super::chat_sheet::bar(b, root, words, bindings);
            if asking && task.result.is_none() && !super::chats::ended(&task.chip) {
                b.open(Role::Button);
                b.name(&words.say(Word::StartOver, &[]));
                b.style(look::START_OVER);
                b.bind(bindings.insert(Act::StartOver));
                b.close();
            }
            bar_actions(b, root, task, words, bindings);
            if super::document::offers_group(root, chat, task) {
                super::document::seg(b, chat, words, bindings);
            }
            spend(b, task, words);
            b.close();
            b.close();
        }
        b.open(Role::Region);
        b.name(&words.say_card(P::ConversationLog, &[]));
        b.style(look::CONVO);
        b.open(Role::Slot);
        b.style(look::THREAD);
        let person = speakers(task, root.pages().me(root.client()));
        if terminal {
            open_log(b, &chat.thread, words, bindings);
            conversation_lines(
                b,
                &chat.thread,
                words,
                (rows, text_limit),
                person,
                (Drawn::ChatPage { request: Some(&task.spec), root }, Some(times)),
                bindings,
            );
            super::terminal::notes(b, root, words);
            b.close();
        } else {
            conversation_with_times(
                b,
                &chat.thread,
                words,
                (rows, text_limit),
                person,
                (Drawn::ChatPage { request: Some(&task.spec), root }, Some(times)),
                bindings,
            );
        }
        latest(b, &chat.thread, words, bindings);
        b.close();
        if task.result.is_none()
            && !super::chats::ended(&task.chip)
            && (!terminal || !super::terminal::commands::typing(root))
        {
            crate::choice::above_composer(b, root, Some(task), words, bindings, times);
        }
        session_composer(b, root, chat, task, words, bindings);
        b.close();
        super::document::pane(b, root, chat, task, words, times, bindings);
    }
    if !terminal || !super::terminal::confirms(root) {
        crate::confirm::main(b, root, words, bindings, rows);
    }
    b.close();
}
/// A chat's heading is at most this many characters, its ellipsis included.
pub(crate) const SHORT_TITLE: usize = 72;

/// A chat's heading shows its title's short form (client/pages/README.md, A chat's heading is short).
pub(crate) fn short_title(title: &[u8]) -> Box<[u8]> {
    let mut start = title.len();
    for (index, byte) in title.iter().enumerate() {
        if !title_space(*byte) {
            start = index;
            break;
        }
    }
    let title = title.get(start..).expect("start is within the title");
    let mut end = title.len();
    for (index, byte) in title.iter().enumerate() {
        if *byte == b'\n' {
            end = index;
            break;
        }
        let next = index.checked_add(1).expect("title offset fits");
        if b".!?".contains(byte)
            && match title.get(next) {
                Some(byte) => title_space(*byte),
                None => true,
            }
        {
            end = next;
            break;
        }
    }
    let words = title.get(..end).expect("sentence ends within the title");
    for byte in words.iter().rev() {
        if !title_space(*byte) {
            break;
        }
        end = end.checked_sub(1).expect("trailing byte is within the words");
    }
    let words = title.get(..end).expect("trimmed words are within the title");
    let mut characters = 0_usize;
    let mut hard_end = 0;
    let mut word_end = 0;
    // Text arrived validated; each non-continuation byte starts one scalar value.
    for (index, byte) in words.iter().enumerate() {
        if byte & 0xc0 != 0x80 {
            characters = characters.checked_add(1).expect("title character count fits");
            if characters > SHORT_TITLE {
                break;
            }
            if characters == SHORT_TITLE {
                hard_end = index;
            }
            if *byte == b' ' || *byte == b'\t' {
                word_end = index;
            }
        }
    }
    if characters <= SHORT_TITLE {
        return skein_lib::bytes::copy_of(words);
    }
    let mut end = word_end;
    for byte in words.get(..word_end).expect("word boundary is within the words").iter().rev() {
        if !title_space(*byte) && !b",;:".contains(byte) {
            break;
        }
        end = end.checked_sub(1).expect("trailing byte is within the prefix");
    }
    if end == 0 {
        end = hard_end;
    }
    let prefix = words.get(..end).expect("cut is at a character boundary within the words");
    let mark = "…".as_bytes();
    let bytes = end.checked_add(mark.len()).expect("short title bytes fit");
    let mut out = skein_lib::Writer::new(bytes);
    out.put(prefix).expect("short title length counted");
    out.put(mark).expect("short title length counted");
    out.finish()
}

fn title_space(byte: u8) -> bool {
    b" \t\r\n".contains(&byte)
}

fn conversation_head(b: &mut Builder, root: &Domain, words: &Words) {
    let chat = root.pages().chat().expect("chat page");
    let task = chat.task(root.client()).expect("chat task");
    older(b, &chat.thread, words);
    if chat.thread.lines.edges.reaches(jig_client::thread::window::End::After) {
        context(b, root, task.project, words);
    }
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
}
/// The read's status at the conversation's head (client/pages/chat/conversation.md, 2).
pub(crate) fn older(b: &mut Builder, thread: &Thread, words: &Words) {
    use jig_client::thread::{Reading, window::End};
    let first_failed = thread.lines.rows.is_empty() && thread.read_failed == Some(End::Before);
    if thread.lines.edges.reaches(End::After) && !first_failed {
        return;
    }
    let status = if let Some(Reading::Older) = thread.reading {
        Word::LoadingEarlier
    } else if thread.read_failed == Some(End::After) || first_failed {
        Word::LoadingEarlierFailed
    } else {
        return;
    };
    b.open(Role::Status);
    b.style(look::THREAD_LOADING);
    b.name(&words.say(Word::LoadingEarlierName, &[]));
    b.text(Run::Plain, &words.say(status, &[]));
    b.close();
}
/// Latest returns to the log's end, reading it again where the window let it go (client/pages/chat/conversation.md, 2).
pub(crate) fn latest(b: &mut Builder, thread: &Thread, words: &Words, bindings: &mut Bindings) {
    if words.shell == Shell::Terminal {
        return;
    }
    b.open(Role::Button);
    b.name(&words.say(Word::Latest, &[]));
    b.style(look::LATEST);
    b.state(State::ToLogEnd);
    if !thread.lines.edges.reaches(jig_client::thread::window::End::Before) {
        b.bind(bindings.insert(Act::Latest));
    }
    b.close();
}
fn conversation_tail(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    rows: u32,
    times: &mut Times,
    places: &List<Place>,
) {
    let chat = root.pages().chat().expect("chat page");
    let task = chat.task(root.client()).expect("chat task");
    let mut unanchored = false;
    for place in places {
        if *place == Place::End {
            unanchored = true;
        }
    }
    if unanchored || !chat.departures().is_empty() {
        b.open(Role::Slot);
        b.key(fixed_key(50));
        b.open(Role::List);
        b.style(look::CARDS);
    }
    for (card, place) in chat.cards.iter().zip(places) {
        if *place == Place::End {
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
        b.close();
    }
    if let Some(result) = &task.result {
        b.open(Role::Slot);
        b.key(fixed_key(51));
        let clock = end_clock(task, words, times);
        result_text(b, result, task.requester, words, clock.as_deref());
        b.close();
    }
    act_status(b, root, chat, words);
}
/// The delivered chat's chip decides its path and current navigation entry (client/pages/README.md).
pub(crate) fn asking(root: &Domain) -> bool {
    let Some(chat) = root.pages().chat() else { return false };
    if chat.talk().is_some() {
        return false;
    }
    let Some(task) = chat.task(root.client()) else { return false };
    let Some(me) = root.pages().me(root.client()) else { return false };
    temper_client_domain::pages::chats::asking(me, task.project, &task.chip.executor)
}
fn status_line(b: &mut Builder, root: &Domain, task: &c::Task, words: &Words) {
    let Some(me) = root.pages().me(root.client()) else { return };
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
    let writing = jig_client::facts::chat_status(&task.chip, &task.waits_on, task.run_wait.as_ref(), task.project, me)
        == jig_client::facts::ChatStatus::Writing;
    let name = if token == look::STATUS_ACTIVE && writing { words.say(Word::RunningWork, &[]) } else { name };
    b.open(Role::Status);
    b.style(look::STATUS_LINE);
    if super::chats::active(root, words) {
        b.style(look::STATUS_WORKING);
    }
    b.name(&name);
    b.open(Role::Slot);
    b.style(look::STATUS_STAR);
    b.text(Run::Plain, &crate::shared::say(words, crate::shared::TerminalWord::Star));
    b.close();
    if words.shell == Shell::Terminal {
        b.text(Run::Plain, b" ");
    }
    b.text(Run::Plain, &name);
    b.close();
}
fn bar_actions(b: &mut Builder, root: &Domain, task: &c::Task, words: &Words, bindings: &mut Bindings) {
    if let Some(chat) = root.pages().chat()
        && chat.talk().is_some()
    {
        spend(b, task, words);
        return;
    }
    if is_paused(root, task) {
        if temper_client_domain::pages::chat::allowed(task, c::ActKind::Release) {
            control(b, bindings, Role::Button, &words.say(Word::Resume, &[]), Act::Resume);
        }
    } else if temper_client_domain::pages::chat::allowed(task, c::ActKind::Stop) {
        control(b, bindings, Role::Button, &words.say(Word::Pause, &[]), Act::Stop);
    }
    if temper_client_domain::pages::chat::allowed(task, c::ActKind::Cancel) {
        control(b, bindings, Role::Button, &words.say(Word::Close, &[]), Act::Close);
    }
}
fn session_composer(
    b: &mut Builder,
    root: &Domain,
    chat: &Chat,
    task: &c::Task,
    words: &Words,
    bindings: &mut Bindings,
) {
    let terminal = super::terminal::shown(root, words);
    let ended = task.result.is_some() || super::chats::ended(&task.chip);
    if ended && !terminal {
        return;
    }
    let send = if chat.talk().is_some() { c::ActKind::Talk } else { c::ActKind::Message };
    if !terminal && !temper_client_domain::pages::chat::allowed(task, send) {
        b.open(Role::Slot);
        b.style(look::FOOT_ROW);
        crate::choice::picker(b, root, Some(task), words, bindings);
        b.close();
        return;
    }
    let over = match root.pages().me(root.client()) {
        Some(me) => temper_client_domain::pages::composer::over(&chat.words, chat.beyond, me.words_bytes),
        None => 0,
    };
    if terminal {
        super::session::form(b, &words.say(Word::Message, &[]));
        super::terminal::chat_running(b, root, words);
        super::session::composer(b);
        super::terminal::prompt(b, words);
    } else {
        super::session::form(b, &words.say(Word::Message, &[]));
        status_line(b, root, task, words);
        super::session::composer(b);
    }
    b.open(Role::TextArea);
    b.name(&words.say(Word::Message, &[]));
    let placeholder = if terminal && ended {
        Box::from(b"/new starts another chat".as_slice())
    } else {
        match chat.change_asked {
            Some(_) => words.say(Word::SettingsProposalChangeAsks, &[]),
            None => words.say(Word::MessagePlaceholder, &[Hole::Number(task.chip.task)]),
        }
    };
    b.placeholder(&placeholder);
    if (terminal && (ended || super::terminal::commands::typing(root))) || (over == 0 && crate::choice::can_send(root))
    {
        b.state(State::SubmitsOnEnter);
    } else if over != 0 {
        bindings.block_submissions();
    }
    if chat.composer_focus != 0 && (!terminal || chat.shown.is_none()) && root.pages().switcher().renaming().is_none() {
        b.focus(chat.composer_focus);
    }
    // Opening and sending write the field; an edit keeps the browser's own value.
    b.value(&chat.words, chat.composer_focus);
    b.bind(bindings.insert(Act::MessageWords { words: Box::default() }));
    b.close();
    if !terminal {
        b.open(Role::Button);
        b.name(&words.say(Word::Send, &[]));
        if chat.words.is_empty() || over != 0 || !crate::choice::can_send(root) {
            b.state(State::Disabled);
        }
        b.bind(bindings.insert(Act::Send));
        b.close();
    }
    let hint = if chat.talk().is_none()
        && is_paused(root, task)
        && temper_client_domain::pages::chat::allowed(task, c::ActKind::Release)
    {
        if super::document::offers_group(root, chat, task) { Word::DraftSendResumes } else { Word::ChatSendResumes }
    } else if super::document::offers_group(root, chat, task) {
        Word::DraftSendHint
    } else {
        Word::ChatSendHint
    };
    if terminal {
        b.close();
        super::terminal::under(
            b,
            root,
            Some(task),
            words,
            bindings,
            if is_paused(root, task) { super::terminal::Hint::Paused } else { super::terminal::Hint::Commands },
        );
    } else {
        super::session::foot_row(b, root, Some(task), words, bindings, hint);
    }
    super::session::over(b, words, over);
    super::session::refusals(b, root, words);
    b.close();
}
fn act_status(b: &mut Builder, root: &Domain, chat: &Chat, words: &Words) {
    let sheet = crate::confirm::showing(root, c::ObjectKey::Task(chat.task));
    for (key, kind) in &chat.pending {
        match kind {
            temper_client_domain::pages::chat::Sending::Message | temper_client_domain::pages::chat::Sending::Seen => {}
            temper_client_domain::pages::chat::Sending::Stop
            | temper_client_domain::pages::chat::Sending::Resume
            | temper_client_domain::pages::chat::Sending::Close => {
                if !sheet && let Some(pending) = root.client().pending(*key) {
                    b.open(Role::Slot);
                    b.key(Key { a: u64::from(key.asker.0), b: key.number, ..fixed_key(53) });
                    crate::confirm::progress_text(b, words, pending.progress);
                    b.close();
                }
            }
        }
    }
    if !sheet
        && chat.outcome_kind != Some(temper_client_domain::pages::chat::Sending::Message)
        && let Some(outcome) = &chat.outcome
        && match chat.outcome_kind {
            Some(
                temper_client_domain::pages::chat::Sending::Stop
                | temper_client_domain::pages::chat::Sending::Resume
                | temper_client_domain::pages::chat::Sending::Close,
            ) => !super::chats::accepted(outcome),
            Some(
                temper_client_domain::pages::chat::Sending::Message | temper_client_domain::pages::chat::Sending::Seen,
            )
            | None => true,
        }
    {
        b.open(Role::Slot);
        b.key(fixed_key(54));
        super::chats::outcome_text(b, words, outcome, 0);
        b.close();
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
    if !proposal_draws(card) {
        return;
    }
    b.open(role);
    b.name(&super::inbox::card_name(root, card.object, Word::Proposal, words));
    if !super::terminal::shown(root, words) && crate::cards::proposal::settings_facts(card).is_none() {
        b.style(look::CARD);
        b.style(look::KIND_PROPOSAL);
    }
    if role == Role::Item || role == Role::Slot {
        b.key(super::inbox::card_key(card.object));
    }
    if !super::terminal::shown(root, words)
        && let Some(page) = &card.page
        && let Some(since) = super::inbox::waiting_since(page)
        && let Some(times) = times
    {
        proposal_wait(b, words, times, since);
    }
    let replaced = settings_replaced(root, card.object);
    if !crate::cards::proposal::settings(b, root, card, words, bindings, replaced) {
        crate::confirm::card(b, root, card, words, bindings, rows);
    }
    b.close();
}

fn proposal_draws(card: &temper_client_domain::pages::inbox::Card) -> bool {
    if let Some(c::ReadValue::Ended(_)) = &card.page {
        return crate::cards::proposal::settings_facts(card).is_some();
    }
    card.page.is_some()
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

/// Where a card stands among the window's lines (client/pages/settings.md, 2, *Where a card stands*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Place {
    /// At the call that names it: `proposal_anchor`.
    Call,
    /// After the line at this index, a committed turn.
    After(usize),
    /// At the conversation's end.
    End,
    /// Past the window's end, or out of it: not drawn.
    Outside,
}

fn proposal_places(root: Option<&Domain>, thread: &Thread, lines: &[Line<'_>], rows: u32) -> List<Place> {
    let Some(root) = root else { return List::with_capacity(0) };
    let chat = root.pages().chat().expect("chat page");
    let mut places = List::with_capacity(chat.cards.capacity());
    for card in &chat.cards {
        places.push(proposal_place(chat, card, thread, lines, rows)).expect("places fit the cards");
    }
    places
}
fn proposal_place(
    chat: &Chat,
    card: &temper_client_domain::pages::inbox::Card,
    thread: &Thread,
    lines: &[Line<'_>],
    rows: u32,
) -> Place {
    if !proposal_draws(card) {
        return Place::Outside;
    }
    if proposal_anchor(chat, card.object, rows).is_some() {
        return Place::Call;
    }
    let end =
        if thread.lines.edges.reaches(jig_client::thread::window::End::Before) { Place::End } else { Place::Outside };
    let mut made = None;
    for (index, line) in lines.iter().enumerate() {
        if let Line::Activity { line } = line
            && let c::Activity::Decided { proposal: Some(c::DecidedProposal { number, made: true }), .. } =
                line.source.line
            && card.object == c::ObjectKey::Proposal(c::ProposalKey { proposer: c::Proposer::Task(chat.task), number })
        {
            made = Some(index);
            break;
        }
    }
    if let Some(made) = made {
        for (index, line) in lines.iter().enumerate() {
            if index > made
                && let Line::Turn { .. } = line
            {
                return Place::After(index);
            }
        }
        return end;
    }
    if let Some(c::ReadValue::Ended(_)) = &card.page {
        return Place::Outside;
    }
    if let Some(page) = &card.page
        && let Some(since) = super::inbox::waiting_since(page)
    {
        for (index, line) in lines.iter().enumerate() {
            if let Line::Turn { at, .. } = line
                && *at >= since
            {
                return Place::After(index);
            }
        }
        return end;
    }
    Place::End
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
    /// An agent's page while a run is out (client/pages/task.md, 2).
    AgentRun {
        current: Option<&'a c::CurrentCall>,
    },
    Lines,
}

struct Drawing<'a> {
    projected: bool,
    request: Option<(&'a [u8], bool)>,
    delegations: Option<(&'a Chat, &'a jig_client::Client)>,
    proposals: Option<&'a Domain>,
    agent_run: bool,
    current: Option<&'a c::CurrentCall>,
}
fn drawing(drawn: Drawn<'_>) -> Drawing<'_> {
    match drawn {
        Drawn::ChatPage { request, root } => Drawing {
            projected: true,
            request: match request {
                Some(_) => root.pages().chat().expect("chat page").request(root.client()),
                None => None,
            },
            delegations: Some((root.pages().chat().expect("chat page"), root.client())),
            proposals: Some(root),
            agent_run: false,
            current: None,
        },
        #[cfg(test)]
        Drawn::Chat { request, delegations } => {
            let request = match request {
                Some(request) => Some((request.specification.words.bytes(), !request.specification.words.cut)),
                None => None,
            };
            Drawing { projected: true, request, delegations, proposals: None, agent_run: false, current: None }
        }
        Drawn::AgentRun { current } => {
            Drawing { projected: false, request: None, delegations: None, proposals: None, agent_run: true, current }
        }
        Drawn::Lines => Drawing {
            projected: false,
            request: None,
            delegations: None,
            proposals: None,
            agent_run: false,
            current: None,
        },
    }
}

/// Stable conversation identities, apart from inbox cards (client/pages/chat/conversation.md, 2).
pub(crate) fn line_key(line: &Line<'_>) -> Option<Key> {
    let (kind, a, b) = match line {
        Line::Turn { name, .. } | Line::Undecoded { name, .. } => (40, name.attempt, u64::from(name.number)),
        Line::Provisional { name, .. } => (40, name.attempt, u64::from(name.number)),
        Line::Words { state, .. } => match state {
            WordState::Sent { message } | WordState::Read { message } => (41, *message, 0),
            WordState::BeingSent => return None,
        },
        Line::Activity { line } => (42, line.source.position, 0),
    };
    Some(Key { owner: look::owner(), kind, a, b })
}
pub(super) fn fixed_key(kind: u16) -> Key {
    Key { owner: look::owner(), kind, a: 0, b: 0 }
}
// A turn's later blocks share its name, with a separate high word for their place.
fn block_key(key: Key, place: u32) -> Key {
    Key { b: key.b | (u64::from(place) << 32), ..key }
}
fn start_fact(
    b: &mut Builder,
    started: &mut Option<(u64, &c::Chip)>,
    line: Option<&Line<'_>>,
    words: &Words,
    bindings: &mut Bindings,
) {
    let Some((at, chip)) = *started else { return };
    if let Some(line) = line {
        let time = match line {
            Line::Activity { line } => Some(line.source.at),
            Line::Words { .. } | Line::Turn { .. } | Line::Undecoded { .. } | Line::Provisional { .. } => {
                line_time(line)
            }
        };
        let Some(time) = time else { return };
        if time <= skein_lib::Wall::from_nanos(at) {
            return;
        }
    }
    b.open(Role::Slot);
    b.key(Key { a: chip.task, ..fixed_key(48) });
    super::chat_sheet::before(b, started, None, words, bindings);
    b.close();
}

#[cfg(test)]
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

pub(crate) fn conversation_with_times(
    b: &mut Builder,
    thread: &Thread,
    words: &Words,
    limits: (u32, u32),
    speakers: Speakers,
    reading: (Drawn<'_>, Option<&mut Times>),
    bindings: &mut Bindings,
) {
    open_log(b, thread, words, bindings);
    conversation_lines(b, thread, words, limits, speakers, reading, bindings);
    b.close();
}

/// Opens the conversation's log, bound to read its edges (client/pages/chat/conversation.md, 2).
pub(crate) fn open_log(b: &mut Builder, thread: &Thread, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Log);
    b.name(&words.say_card(P::ConversationLog, &[]));
    if !thread.lines.edges.reaches(jig_client::thread::window::End::After)
        || !thread.lines.edges.reaches(jig_client::thread::window::End::Before)
    {
        b.bind(bindings.reading());
    }
    if !thread.lines.edges.reaches(jig_client::thread::window::End::Before) {
        b.state(State::Continues);
    }
    if thread.reading.is_some() || thread.read_failed.is_some() {
        b.state(State::Busy);
    }
}

/// The log's lines, into a log its caller opened and closes.
// Agent task pages keep their own treatment of moments (client/view.md, 6).
#[expect(clippy::too_many_lines, reason = "the conversation walk interleaves its run, text and card anchors")]
pub(crate) fn conversation_lines(
    b: &mut Builder,
    thread: &Thread,
    words: &Words,
    limits: (u32, u32),
    speakers: Speakers,
    (drawn, mut times): (Drawn<'_>, Option<&mut Times>),
    bindings: &mut Bindings,
) {
    let (rows, _) = limits;
    let Drawing { projected, request, delegations, proposals, agent_run, current: agent_current } = drawing(drawn);
    let terminal = match proposals {
        Some(root) => super::terminal::shown(root, words),
        None => false,
    };
    let bound = match delegations {
        Some((chat, _)) => chat.delegations.capacity(),
        None => 0,
    };
    let mut tasks = List::with_capacity(bound);
    let mut shown = List::with_capacity(bound);
    if let Some(root) = proposals {
        b.end_request(root.pages().chat().expect("chat page").to_end());
        b.open(Role::Slot);
        b.style(look::THREAD_INNER);
        conversation_head(b, root, words);
    }
    let lines = thread.lines().into_boxed();
    let places = proposal_places(proposals, thread, &lines, rows);
    let mut standing = List::with_capacity(places.capacity());
    if let Some(root) = proposals {
        let chat = root.pages().chat().expect("chat page");
        for (card, place) in chat.cards.iter().zip(&places) {
            if *place != Place::Outside
                && let c::ObjectKey::Proposal(key) = card.object
                && key.proposer == c::Proposer::Task(chat.task)
            {
                standing.push(key.number).expect("standing cards fit");
            }
        }
    }
    let mut run = super::chat_line::Run::with_capacity(rows.saturating_mul(rows).saturating_mul(thread.limits.lines));
    let mut started = super::chat_sheet::started(proposals);
    let mut day = None;
    // A chat started with no words opens on its first words line (3.5).
    if thread.lines.edges.after == jig_client::thread::window::Edge::End
        && let Some(request) = request
        && (!request.0.is_empty() || !request.1)
    {
        let clock = turn_time(b, first_time(&lines), &mut times, &mut day, words);
        request_message(b, request, speakers.requester, words, clock.as_deref());
    }
    let mut pending = 0_u64;
    let mut before = None;
    for (index, line) in lines.into_iter().enumerate() {
        let key = match line_key(&line) {
            Some(key) => key,
            None => {
                let key = Key { a: pending, ..fixed_key(43) };
                pending = pending.checked_add(1).expect("pending words fit the window");
                key
            }
        };
        let since = before;
        if let Some(at) = line_at(&line) {
            before = Some(at);
        }
        if let Some((at, _)) = started
            && let Some(time) = match &line {
                Line::Activity { line } => Some(line.source.at),
                Line::Words { .. } | Line::Turn { .. } | Line::Undecoded { .. } | Line::Provisional { .. } => {
                    line_time(&line)
                }
            }
            && time > skein_lib::Wall::from_nanos(at)
        {
            run.flush(b, None, false, words, terminal);
        }
        if let Line::Provisional { text, .. } = &line
            && text.is_empty()
        {
            continue;
        }
        start_fact(b, &mut started, Some(&line), words, bindings);
        if hidden_line(&line, projected, delegations, standing.as_slice()) {
            if let Line::Activity { line: event } = &line
                && let Some(task) = delegation::heard(&event.source.line)
                && !shown.as_slice().contains(&task)
            {
                run.flush(b, None, false, words, terminal);
            }
            heard_card(b, &line, delegations, (&mut tasks, &mut shown), words, bindings, terminal);
            continue;
        }
        if let Line::Turn { name, turn, at, .. } = line {
            committed(
                b,
                turn,
                words,
                rows,
                Committed {
                    key,
                    delegations,
                    proposals: match proposals {
                        Some(root) => Some((root, name)),
                        None => None,
                    },
                    times: times.as_deref_mut(),
                    day: &mut day,
                    at,
                    since,
                    projected,
                    terminal,
                    requester: speakers.requester,
                },
                (&mut tasks, &mut shown),
                bindings,
                &mut run,
            );
            if let Some(root) = proposals {
                let chat = root.pages().chat().expect("chat page");
                for (card, place) in chat.cards.iter().zip(&places) {
                    if *place == Place::After(index) {
                        run.flush(b, None, false, words, terminal);
                        proposal_card(b, root, card, words, bindings, (rows, times.as_deref_mut()), Role::Slot);
                    }
                }
            }
            continue;
        }
        run.flush(b, None, false, words, terminal);
        let clock = turn_time(b, line_time(&line), &mut times, &mut day, words);
        b.open(Role::Slot);
        b.key(key);
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
            Line::Provisional { text, .. } => provisional(b, text, words),
            Line::Undecoded { why, .. } => undecoded(b, why, words, clock.as_deref()),
            Line::Activity { line } => {
                owner_line(b, &line.source.line, projected, speakers.person, words, clock.as_deref());
            }
        }
        b.close();
    }
    let running = if agent_run {
        super::chats::words_not_begun(thread)
    } else {
        match proposals {
            Some(root) => super::chats::working(root, words, thread),
            None => false,
        }
    };
    let current = if running {
        if agent_run {
            agent_current
        } else {
            match proposals {
                Some(root) => match root.pages().chat() {
                    Some(chat) => chat.current_call(),
                    None => None,
                },
                None => None,
            }
        }
    } else {
        None
    };
    let running_run = running && (!run.is_empty() || current.is_some());
    run.flush(b, current, running_run, words, terminal);
    start_fact(b, &mut started, None, words, bindings);
    // The live delegates anchor follows the window's last line (client/view.md, 5).
    live_delegates(b, &mut tasks, delegations, &mut shown, words, bindings, terminal);
    if let Some(root) = proposals {
        conversation_tail(b, root, words, bindings, rows, times.expect("chat page times"), &places);
        if words.shell == Shell::Browser && running && !running_run {
            b.open(Role::Slot);
            b.style(look::TYPING);
            b.key(fixed_key(55));
            b.name(&words.say(Word::ChatWorking, &[]));
            for _ in 0_u32..3 {
                b.open(Role::Slot);
                b.close();
            }
            b.close();
        }
        b.close();
    }
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
fn request_message(b: &mut Builder, request: (&[u8], bool), requester: bool, words: &Words, clock: Option<&[u8]>) {
    b.open(Role::Region);
    b.style(look::TURN);
    let label = if requester { words.say_card(P::You, &[]) } else { words.say_card(P::Person, &[]) };
    let (bytes, whole) = request;
    if whole {
        person_message(b, &label, bytes, WordState::Read { message: 0 }, words, clock);
    } else {
        let mut marked = List::with_capacity(u32::try_from(bytes.len()).expect("request bound fits").saturating_add(3));
        for byte in bytes {
            marked.push(*byte).expect("request and mark fit");
        }
        for byte in "…".as_bytes() {
            marked.push(*byte).expect("request and mark fit");
        }
        person_message(b, &label, marked.as_slice(), WordState::Read { message: 0 }, words, clock);
    }
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
/// When any line happened, drawn or not; `line_time` stays the drawn lines' clock.
fn line_at(line: &Line<'_>) -> Option<skein_lib::Wall> {
    match line {
        Line::Words { at, .. } => *at,
        Line::Turn { at, .. } | Line::Undecoded { at, .. } => Some(*at),
        Line::Activity { line } => Some(line.source.at),
        Line::Provisional { .. } => None,
    }
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
            | c::Activity::NowWritten { .. }
            | c::Activity::Stood { .. }
            | c::Activity::TalkOpened { .. }
            | c::Activity::TalkEnded { .. }
            | c::Activity::SessionEnded { .. }
            | c::Activity::Kept { .. } => None,
        },
        Line::Provisional { .. } => None,
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
        b.open(Role::Slot);
        b.key(Key { a: u64::from_le_bytes(date.to_le_bytes()), b: at.as_nanos(), ..fixed_key(46) });
        b.style(look::DAY);
        b.name(&said);
        b.open(Role::Paragraph);
        b.name(&said);
        b.text(Run::Plain, &said);
        b.close();
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
/// A delegate's heard result anchors its card, as a result block does (client/view.md, 5).
fn heard_card(
    b: &mut Builder,
    line: &Line<'_>,
    delegations: Option<(&Chat, &jig_client::Client)>,
    (tasks, shown): (&mut List<u64>, &mut List<u64>),
    words: &Words,
    bindings: &mut Bindings,
    terminal: bool,
) {
    let key = line_key(line);
    if let Line::Activity { line } = line
        && let Some(task) = delegation::heard(&line.source.line)
        && !shown.as_slice().contains(&task)
    {
        tasks.clear();
        tasks.push(task).expect("one heard delegate fits");
        b.open(Role::Slot);
        b.key(key.expect("activity key"));
        b.style(look::TURN);
        cards(b, tasks, delegations, shown, words, bindings, terminal);
        b.close();
    }
}
// A browser's semantic region passes the flex layout through the keyed turn slot.
fn message_open(b: &mut Builder, words: &Words) {
    b.open(Role::Region);
    if words.shell == Shell::Browser {
        b.style(look::TURN);
    }
    b.open(Role::Slot);
    b.style(look::MESSAGE);
}
fn provisional(b: &mut Builder, message: &[u8], words: &Words) {
    message_open(b, words);
    b.style(look::BY_AGENT);
    b.open(Role::Slot);
    b.style(look::WHO);
    words.card_text(b, P::Temper, &[]);
    b.close();
    b.open(Role::Slot);
    b.style(look::SAYS);
    skein_ui::markdown::read_growing(
        message,
        2,
        |b| {
            b.open(Role::Slot);
            b.style(look::WRITING_END);
            words.status(b, P::Provisional, &[]);
            b.close();
        },
        b,
    );
    b.close();
    b.close();
    b.close();
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
fn hidden_line(
    line: &Line<'_>,
    projected: bool,
    delegations: Option<(&Chat, &jig_client::Client)>,
    standing: &[u64],
) -> bool {
    match line {
        Line::Activity { line } => {
            if let c::Activity::Decided { proposal: Some(proposal), .. } = line.source.line
                && standing.contains(&proposal.number)
            {
                return true;
            }
            if let c::Activity::Decided { .. } = line.source.line
                && let Some((chat, client)) = delegations
                && let Some(task) = chat.task(client)
            {
                let by = match &task.result {
                    Some(c::TaskResult::Ending(ending)) => match ending.as_ref() {
                        c::ChipEnding::Cancelled { by, .. } => Some(*by),
                        c::ChipEnding::Done(_) | c::ChipEnding::Failed(_) => None,
                    },
                    Some(c::TaskResult::Document(result)) => match &result.ending {
                        c::Ending::Task(ending) => match ending.as_ref() {
                            c::ChipEnding::Cancelled { by, .. } => Some(*by),
                            c::ChipEnding::Done(_) | c::ChipEnding::Failed(_) => None,
                        },
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
                if by == Some(task.requester) {
                    return true;
                }
            }
            hidden(line, projected, delegations)
        }
        Line::Words { .. } | Line::Turn { .. } | Line::Undecoded { .. } | Line::Provisional { .. } => false,
    }
}

fn live_delegates(
    b: &mut Builder,
    tasks: &mut List<u64>,
    delegations: Option<(&Chat, &jig_client::Client)>,
    shown: &mut List<u64>,
    words: &Words,
    bindings: &mut Bindings,
    terminal: bool,
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
        b.open(Role::Slot);
        b.key(fixed_key(49));
        b.style(look::TURN);
        cards(b, tasks, delegations, shown, words, bindings, terminal);
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
    message_open(b, words);
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
    message_open(b, words);
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
    b.close();
}

// Decoded turns already share the receiving turn_text budget; both drawings
// retain their text whole, independently of field text.
struct Committed<'a> {
    key: Key,
    delegations: Option<(&'a Chat, &'a jig_client::Client)>,
    proposals: Option<(&'a Domain, c::TurnName)>,
    times: Option<&'a mut Times>,
    day: &'a mut Option<i64>,
    at: skein_lib::Wall,
    since: Option<skein_lib::Wall>,
    projected: bool,
    requester: bool,
    terminal: bool,
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
    let mut place = 0_u32;
    let bound = usize::try_from(rows).expect("row bound fits");
    for (message_at, message) in turn.messages.iter().take(bound).enumerate() {
        for (block_at, block) in message.blocks.iter().take(bound).enumerate() {
            if let c::Block::Call(call) = block
                && (!drawing.projected || message.speaker == c::Speaker::Assistant)
                && run_call(block, tasks, drawing.delegations)
            {
                if unshown_card(tasks, drawing.delegations, shown) {
                    run.flush(b, None, false, words, drawing.terminal);
                    let clock = committed_clock(b, &mut drawing, &mut dated, words);
                    b.open(Role::Slot);
                    b.key(block_key(drawing.key, place));
                    place = place.checked_add(1).expect("turn blocks fit");
                    b.style(look::TURN);
                    card_clock(b, clock.as_deref());
                    cards(b, tasks, drawing.delegations, shown, words, bindings, drawing.terminal);
                    b.close();
                }
                if run.is_empty() {
                    run.key(block_key(drawing.key, place));
                    place = place.checked_add(1).expect("turn blocks fit");
                }
                run.call(call, drawing.at, drawing.since.take());
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
            run.flush(b, None, false, words, drawing.terminal);
            let clock = committed_clock(b, &mut drawing, &mut dated, words);
            b.open(Role::Slot);
            b.key(block_key(drawing.key, place));
            place = place.checked_add(1).expect("turn blocks fit");
            b.style(look::TURN);
            match block {
                c::Block::Text(value) => {
                    message_open(b, words);
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
                    crate::views::read(
                        b,
                        value.bytes(),
                        2,
                        match drawing.delegations {
                            Some((chat, _)) => Some(chat),
                            None => None,
                        },
                        words,
                    );
                    b.close();
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
                    cards(b, tasks, drawing.delegations, shown, words, bindings, drawing.terminal);
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
                        cards(b, tasks, drawing.delegations, shown, words, bindings, drawing.terminal);
                    } else {
                        message_open(b, words);
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
                        text(b, Role::Paragraph, first_line(summary.bytes()));
                        b.close();
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

fn first_line(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    for (end, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            let line = bytes.get(start..end).expect("line lies within summary");
            let line = match line.strip_suffix(b"\r") {
                Some(line) => line,
                None => line,
            };
            if !line.trim_ascii().is_empty() {
                return line;
            }
            start = end.checked_add(1).expect("newline lies within summary");
        }
    }
    let line = bytes.get(start..).expect("last line lies within summary");
    let line = match line.strip_suffix(b"\r") {
        Some(line) => line,
        None => line,
    };
    if line.trim_ascii().is_empty() { &[] } else { line }
}

// Both conversation drawings keep a call's name and outcome as one work line.
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
    terminal: bool,
) {
    let Some((chat, client)) = delegations else { return };
    for task in tasks {
        if !shown.as_slice().contains(task)
            && let Some(document) = chat.delegation(client, *task)
        {
            if terminal {
                crate::cards::delegation::terminal_card(b, document, words, bindings);
            } else {
                crate::cards::delegation::card(b, document, words, bindings);
            }
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
            c::Activity::Milestone { .. }
            | c::Activity::Decided { proposal: Some(c::DecidedProposal { made: true, .. }), .. } => return true,
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
            | c::Activity::NowWritten { .. }
            | c::Activity::Stood { .. }
            | c::Activity::TalkOpened { .. }
            | c::Activity::TalkEnded { .. }
            | c::Activity::SessionEnded { .. }
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
// A later settings proposal in this chat replaces a withdrawn one (client/pages/settings.md, 2).
fn settings_replaced(root: &Domain, object: c::ObjectKey) -> bool {
    let c::ObjectKey::Proposal(key) = object else { return false };
    let Some(chat) = root.pages().chat() else { return false };
    for card in &chat.cards {
        if let Some(proposal) = crate::cards::proposal::settings_facts(card)
            && proposal.key.number > key.number
        {
            return true;
        }
    }
    false
}

fn is_paused(root: &Domain, task: &c::Task) -> bool {
    match root.pages().me(root.client()) {
        Some(me) => temper_client_domain::pages::chat::paused(&task.chip, me),
        None => false,
    }
}
