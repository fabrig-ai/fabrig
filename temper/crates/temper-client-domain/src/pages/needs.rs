//! Session state for what needs the person (temper/docs/design/client/README.md, 3.5, 3.6).
use super::{
    Pages, Request,
    inbox::{Card, Inbox},
};
use crate::{Address, Limits, acts, address::Filter, owner::Owner};
use alloc::boxed::Box;
use jig_client::{
    Client, Handle, Key, Latest, Outcome, State, Told,
    contract::{self, ActKind, ObjectKey, ReadValue, Text},
};
use skein_lib::{Duration, Env, List, Queue, Time, Wall};

pub const HOLD: Duration = Duration::from_secs(5);
pub const TOAST: Duration = Duration::from_millis(4500);
pub const ANSWERED_ON_NEEDS: Duration = Duration::from_secs(600);
pub const ANSWERED_ON_HOME: Duration = Duration::from_secs(120);
/// Session memory holds at most 64 answered lines, Later marks and refusals each.
const CAPACITY: u32 = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Choose { object: ObjectKey, words: Box<[u8]> },
    OpenReply { object: ObjectKey },
    CancelReply,
    Dismiss,
    SendReply { object: ObjectKey },
    Later { object: ObjectKey },
    Undo { object: ObjectKey },
    Open { object: ObjectKey },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Label {
    Choice(Box<[u8]>),
    Reply(Box<[u8]>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(
    clippy::partial_pub_fields,
    reason = "the line is public; its held send and correlation are session bookkeeping"
)]
pub struct Answered {
    pub object: ObjectKey,
    pub project: u32,
    pub title: Box<[u8]>,
    pub label: Label,
    pub at: Wall,
    held: Option<(Text, Time)>,
    key: Option<Key>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Toasted {
    AwayOn { project: u32, until: Option<Wall> },
    AwayOff,
    AwayRefused,
    Overturned,
    OverturnEnded,
    OverturnAlready,
    Answered { object: ObjectKey, label: Label },
    Later,
    NotSent,
    Paused,
    Resumed,
    Proposed { to: contract::Recipient },
    Refused { outcome: Outcome },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Count {
    pub count: u32,
    /// The document was cut: the view says "N+".
    pub cut: bool,
}
#[expect(clippy::derivable_impls, reason = "the programming model uses explicit Default implementations")]
impl Default for Count {
    fn default() -> Self {
        Self { count: 0, cut: false }
    }
}
/// The standing inbox watch, outside the increasing page handle namespace.
#[must_use]
pub fn handle() -> Handle {
    Handle { asker: Owner::Pages.asker(), number: u64::MAX.checked_sub(3).expect("reserved needs handle") }
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the reply's excess count is drawn beside its card's words")]
pub struct Session {
    watching: bool,
    settings: List<contract::Proposal>,
    answered: List<Answered>,
    later: List<ObjectKey>,
    refused: List<(ObjectKey, Outcome)>,
    at: Option<ObjectKey>,
    reply: Option<ObjectKey>,
    pub reply_beyond: u64,
    toast: Option<(Toasted, Time)>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            watching: false,
            settings: List::with_capacity(CAPACITY),
            answered: List::with_capacity(CAPACITY),
            later: List::with_capacity(CAPACITY),
            refused: List::with_capacity(CAPACITY),
            at: None,
            reply: None,
            reply_beyond: 0,
            toast: None,
        }
    }
}
impl Session {
    pub(crate) fn say(&mut self, toasted: Toasted, now: Time) {
        self.toast = Some((toasted, now.saturating_add(TOAST)));
    }
    pub(super) fn configure(&mut self, showing: bool, limits: &Limits, out: &mut Queue<Request>) {
        if showing && !self.watching {
            self.watching = true;
            out.push(Request::Client(jig_client::Event::Watch {
                handle: handle(),
                subject: jig_client::Watchable::Jig(contract::Subject::Inbox { window: limits.client.domain.rows }),
            }));
        } else if !showing && self.watching {
            self.watching = false;
            out.push(Request::Client(jig_client::Event::Unwatch { handle: handle() }));
        }
    }
    #[must_use]
    pub fn count(&self, client: &Client, project: u32) -> Count {
        let Some(document) = client.document(handle()) else { return Count::default() };
        if !self.watching || document.state == State::Ended {
            return Count::default();
        }
        let Some(Latest::Jig(contract::Delivery::Inbox(inbox))) = document.latest else { return Count::default() };
        let mut count = 0_u32;
        for row in &inbox.needs {
            if row.shows_in(project) && self.line(row.card_key()).is_none() {
                count = count.checked_add(1).expect("inbox rows fit");
            }
        }
        Count { count, cut: inbox.next.is_some() }
    }
    #[must_use]
    pub fn open<'a>(&self, inbox: &'a Inbox) -> Box<[&'a Card]> {
        let mut cards = List::with_capacity(inbox.cards.len());
        for later in [false, true] {
            for card in inbox.needs() {
                if self.line(card.object).is_none() && self.later(card.object) == later {
                    cards.push(card).expect("inbox cards fit");
                }
            }
        }
        cards.into_boxed()
    }
    #[must_use]
    pub fn shown<'a>(&self, inbox: &'a Inbox) -> Option<&'a Card> {
        let open = self.open(inbox);
        for card in &open {
            if Some(card.object) == self.at {
                return Some(card);
            }
        }
        open.first().copied()
    }
    #[must_use]
    pub fn later(&self, object: ObjectKey) -> bool {
        self.later.as_slice().contains(&object)
    }
    #[must_use]
    pub fn replying(&self, object: ObjectKey) -> bool {
        self.reply == Some(object)
    }
    #[must_use]
    pub fn held(&self, object: ObjectKey) -> bool {
        match self.line(object) {
            Some(line) => line.held.is_some(),
            None => false,
        }
    }
    #[expect(clippy::manual_find, reason = "the step subset uses explicit bounded traversal without closures")]
    fn line(&self, object: ObjectKey) -> Option<&Answered> {
        for line in &self.answered {
            if line.object == object {
                return Some(line);
            }
        }
        None
    }
    #[must_use]
    pub fn refused(&self, object: ObjectKey) -> Option<&Outcome> {
        for (item, outcome) in &self.refused {
            if *item == object {
                return Some(outcome);
            }
        }
        None
    }
    #[must_use]
    pub fn answered(&self, now: Wall, window: Duration, project: Option<u32>) -> Box<[&Answered]> {
        let mut lines = List::with_capacity(CAPACITY);
        for line in &self.answered {
            if now.as_nanos().saturating_sub(line.at.as_nanos()) < window.as_nanos()
                && (project.is_none() || project == Some(line.project))
            {
                lines.push(line).expect("session lines fit");
            }
        }
        lines.into_boxed()
    }
    #[must_use]
    pub fn toast(&self) -> Option<&Toasted> {
        match &self.toast {
            Some((toast, _)) => Some(toast),
            None => None,
        }
    }
    pub(super) fn next_deadline(&self) -> Option<Time> {
        let mut next = match &self.toast {
            Some((_, deadline)) => Some(*deadline),
            None => None,
        };
        for line in &self.answered {
            if let Some((_, deadline)) = &line.held {
                next = skein_lib::earliest(next, Some(*deadline));
            }
        }
        next
    }
    pub(super) fn went(&mut self, before: &Address, after: &Address) {
        if before != after {
            self.reply = None;
            self.reply_beyond = 0;
        }
        match after {
            Address::Inbox { .. } => {}
            Address::NewChat { .. }
            | Address::Goals { .. }
            | Address::Changes { .. }
            | Address::Chats { .. }
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::Chat { .. }
            | Address::Task { .. }
            | Address::Home { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Repositories { .. }
            | Address::Settings { .. }
            | Address::Configure { .. }
            | Address::Adopt { .. } => self.at = None,
        }
    }
    fn cleared(&mut self, object: ObjectKey) {
        clear_refused(self, object);
        if self.at == Some(object) {
            self.at = None;
        }
        if self.reply == Some(object) {
            self.reply = None;
            self.reply_beyond = 0;
        }
    }
    /// Keep the original settings card through a visit to Needs you (client/README.md, 5.13).
    pub(super) fn remember_settings(&mut self, told: &Told) {
        let Told::Page {
            page: jig_client::Readout::Jig(contract::ReadResult::Value(ReadValue::Proposal(proposal))),
            ..
        } = told
        else {
            return;
        };
        let contract::ProposalAction::Settings(_) = &proposal.action else { return };
        for saved in &self.settings {
            if saved.key == proposal.key {
                return;
            }
        }
        if self.settings.room() == 0 {
            for index in 1..self.settings.len() {
                let previous = index.checked_sub(1).expect("index starts at one");
                let next = self.settings.get(index).expect("saved settings").clone();
                *self.settings.get_mut(previous).expect("previous settings") = next;
            }
            self.settings.truncate(self.settings.len().checked_sub(1).expect("saved settings"));
        }
        self.settings.push((**proposal).clone()).expect("room for settings card");
    }
    pub(super) fn restore_settings(&self, chat: &mut super::chat::Chat, out: &mut Queue<Request>) {
        for proposal in &self.settings {
            if proposal.key.proposer != contract::Proposer::Task(chat.task)
                || chat.cards.room() == 0
                || chat.settings_proposals.room() == 0
            {
                continue;
            }
            let mut snapshot = proposal.clone();
            snapshot.acts.clear();
            chat.settings_proposals.push(snapshot).expect("room for original settings");
            let handle = Handle { asker: Owner::Pages.asker(), number: chat.next };
            chat.next = chat.next.checked_add(1).expect("chat handles fit");
            chat.cards
                .push(Card {
                    object: ObjectKey::Proposal(proposal.key),
                    group: contract::InboxGroup::Proposals,
                    project: Some(proposal.project),
                    position: None,
                    unread: None,
                    handle,
                    page: None,
                    words: None,
                    key: None,
                    outcome: None,
                })
                .expect("room for settings card");
            out.push(Request::Client(jig_client::Event::Read {
                handle,
                read: jig_client::Readable::Jig(contract::Read::Proposal { proposal: proposal.key }),
            }));
        }
    }
    pub(super) fn told(&mut self, told: &Told) -> bool {
        match told {
            Told::Answered { key, outcome } => {
                for index in 0..self.answered.len() {
                    if self.answered.get(index).expect("bounded line").key == Some(*key) {
                        if done(outcome) {
                            self.answered.get_mut(index).expect("matched line").key = None;
                        } else {
                            let object = remove_answered(&mut self.answered, index).object;
                            clear_refused(self, object);
                            if self.refused.room() == 0 {
                                remove_refused(&mut self.refused, 0);
                            }
                            self.refused.push((object, outcome.clone())).expect("room for refusal");
                        }
                        return true;
                    }
                }
                false
            }
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => false,
        }
    }
}
fn done(outcome: &Outcome) -> bool {
    match outcome {
        Outcome::Answered(answer) => match answer {
            jig_client::Answered::Jig(answer) => match answer {
                contract::Answer::Done(_) => true,
                contract::Answer::Refused { .. } => false,
            },
            jig_client::Answered::Family(_) => false,
        },
        Outcome::NotKept
        | Outcome::Unknown
        | Outcome::Limit
        | Outcome::Unconfirmed
        | Outcome::Held
        | Outcome::Unsent
        | Outcome::Full { .. }
        | Outcome::Lost => false,
    }
}
/// Stable removal keeps answered lines in insertion order.
fn remove_answered(list: &mut List<Answered>, index: u32) -> Answered {
    let item = list.get(index).expect("existing item").clone();
    let end = list.len().checked_sub(1).expect("nonempty list");
    for slot in index..end {
        let next = list.get(slot.checked_add(1).expect("bounded slot")).expect("next item").clone();
        *list.get_mut(slot).expect("existing slot") = next;
    }
    list.truncate(end);
    item
}
fn remove_later(list: &mut List<ObjectKey>, index: u32) -> ObjectKey {
    let item = *list.get(index).expect("existing item");
    let end = list.len().checked_sub(1).expect("nonempty list");
    for slot in index..end {
        let next = *list.get(slot.checked_add(1).expect("bounded slot")).expect("next item");
        *list.get_mut(slot).expect("existing slot") = next;
    }
    list.truncate(end);
    item
}
fn remove_refused(list: &mut List<(ObjectKey, Outcome)>, index: u32) -> (ObjectKey, Outcome) {
    let item = list.get(index).expect("existing item").clone();
    let end = list.len().checked_sub(1).expect("nonempty list");
    for slot in index..end {
        let next = list.get(slot.checked_add(1).expect("bounded slot")).expect("next item").clone();
        *list.get_mut(slot).expect("existing slot") = next;
    }
    list.truncate(end);
    item
}
fn clear_refused(session: &mut Session, object: ObjectKey) {
    for index in 0..session.refused.len() {
        if session.refused.get(index).expect("bounded refusal").0 == object {
            remove_refused(&mut session.refused, index);
            return;
        }
    }
}
fn clear_later(session: &mut Session, object: ObjectKey) {
    for index in 0..session.later.len() {
        if session.later.get(index) == Some(&object) {
            remove_later(&mut session.later, index);
            return;
        }
    }
}
#[expect(clippy::manual_find, reason = "the step subset uses explicit bounded traversal without closures")]
fn card_mut(pages: &mut Pages, object: ObjectKey) -> Option<&mut Card> {
    for card in pages.inbox.as_mut()?.cards.iter_mut() {
        if card.object == object {
            return Some(card);
        }
    }
    None
}
pub(crate) fn write_reply(pages: &mut Pages, object: ObjectKey, words: Box<[u8]>, hold: u32) {
    let (words, beyond) = super::composer::held(words, hold);
    let Ok(words) = Text::new(words, hold) else { return };
    if let Some(card) = card_mut(pages, object) {
        card.words = Some(words);
        pages.needs.reply_beyond = beyond;
    }
}
fn question(page: &ReadValue) -> Option<&contract::Question> {
    match page {
        ReadValue::Question(question) => Some(question),
        ReadValue::Task(_)
        | ReadValue::Proposal(_)
        | ReadValue::Escalation(_)
        | ReadValue::PersonTask(_)
        | ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Ended(_)
        | ReadValue::Me(_)
        | ReadValue::Result(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => None,
    }
}
fn question_on(object: ObjectKey, words: Text) -> Option<contract::AnswerOn> {
    match object {
        ObjectKey::Question { task, message } => Some(contract::AnswerOn::Question { task, message, words }),
        ObjectKey::Task(_)
        | ObjectKey::Proposal(_)
        | ObjectKey::Escalation(_)
        | ObjectKey::PersonTask(_)
        | ObjectKey::Result(_)
        | ObjectKey::Note { .. } => None,
    }
}
fn answerable(pages: &Pages, object: ObjectKey) -> Option<&Card> {
    for card in &pages.inbox.as_ref()?.cards {
        if card.object == object && card.key.is_none() && card.project.is_some() && pages.needs.line(object).is_none() {
            let page = card.page.as_ref()?;
            question(page)?;
            match object {
                ObjectKey::Question { .. } => {}
                ObjectKey::Task(_)
                | ObjectKey::Proposal(_)
                | ObjectKey::Escalation(_)
                | ObjectKey::PersonTask(_)
                | ObjectKey::Result(_)
                | ObjectKey::Note { .. } => return None,
            }
            if acts::allowed(page, ActKind::Answer) {
                return Some(card);
            }
        }
    }
    None
}
fn reply_label(words: &[u8]) -> Box<[u8]> {
    let mut characters = 0_u32;
    let mut end = 0;
    for (index, byte) in words.iter().enumerate() {
        if *byte & 0xc0 != 0x80 {
            if characters == 57 {
                end = index;
            }
            characters = characters.checked_add(1).expect("cue characters fit");
        }
    }
    if characters <= 60 {
        return Box::from(words);
    }
    let mut cut = List::with_capacity(u32::try_from(end).expect("cue fits").checked_add(3).expect("ellipsis fits"));
    for byte in words.get(..end).expect("character boundary").iter().chain("…".as_bytes()) {
        cut.push(*byte).expect("cut fits");
    }
    cut.into_boxed()
}
fn make_room(session: &mut Session) -> bool {
    if session.answered.room() > 0 {
        return true;
    }
    for index in 0..session.answered.len() {
        if session.answered.get(index).expect("bounded line").held.is_none() {
            remove_answered(&mut session.answered, index);
            return true;
        }
    }
    false
}
fn answer(pages: &mut Pages, env: &Env<Limits>, client: &Client, object: ObjectKey, words: Box<[u8]>, reply: bool) {
    let Some(card) = answerable(pages, object) else { return };
    let bound = if reply {
        let Some(me) = pages.me(client) else { return };
        let Some(typed) = &card.words else { return };
        if super::composer::over(typed.bytes(), pages.needs.reply_beyond, me.words_bytes) > 0 {
            return;
        }
        env.limits.client.domain.long_text
    } else {
        temper_client_protocol_wire::CUE_BYTES
    };
    let Some(page) = &card.page else { return };
    let Some(question) = question(page) else { return };
    let project = card.project.expect("answerable project");
    let title = question.task.title.bytes().into();
    let Ok(words) = Text::new(words, bound) else {
        pages.needs.toast = Some((Toasted::NotSent, env.now.saturating_add(TOAST)));
        return;
    };
    if !make_room(&mut pages.needs) {
        pages.needs.toast = Some((Toasted::NotSent, env.now.saturating_add(TOAST)));
        return;
    }
    let label = if reply { Label::Reply(reply_label(words.bytes())) } else { Label::Choice(Box::from(words.bytes())) };
    pages
        .needs
        .answered
        .push(Answered {
            object,
            project,
            title,
            label: label.clone(),
            at: env.wall,
            held: Some((words, env.now.saturating_add(HOLD))),
            key: None,
        })
        .expect("room for answer");
    pages.needs.toast = Some((Toasted::Answered { object, label }, env.now.saturating_add(TOAST)));
    pages.needs.cleared(object);
    clear_later(&mut pages.needs, object);
    if reply && let Some(card) = card_mut(pages, object) {
        card.words = None;
    }
}
fn undoable(session: &Session, object: ObjectKey, now: Time) -> Option<u32> {
    for index in 0..session.answered.len() {
        let line = session.answered.get(index).expect("bounded line");
        if line.object == object
            && let Some((_, deadline)) = &line.held
            && now < *deadline
        {
            return Some(index);
        }
    }
    None
}
#[expect(
    clippy::too_many_lines,
    reason = "the Needs acts share one exhaustive dispatch, including session toast variants"
)]
pub(super) fn act(pages: &mut Pages, env: &Env<Limits>, client: &Client, act: Act, out: &mut Queue<Request>) {
    match act {
        Act::Choose { object, words } => answer(pages, env, client, object, words, false),
        Act::OpenReply { object } => {
            if answerable(pages, object).is_some() && pages.needs.reply != Some(object) {
                pages.needs.reply = Some(object);
                pages.needs.reply_beyond = 0;
            }
        }
        Act::Dismiss => pages.needs.toast = None,
        Act::CancelReply => {
            if let Some(object) = pages.needs.reply.take()
                && let Some(card) = card_mut(pages, object)
            {
                card.words = None;
            }
            pages.needs.reply_beyond = 0;
        }
        Act::SendReply { object } => {
            let Some(card) = answerable(pages, object) else { return };
            let Some(words) = &card.words else { return };
            let words = words.bytes().trim_ascii();
            if !words.is_empty() {
                answer(pages, env, client, object, Box::from(words), true);
            }
        }
        Act::Later { object } => {
            let Some(inbox) = &pages.inbox else { return };
            let mut open = false;
            for card in &pages.needs.open(inbox) {
                if card.object == object {
                    open = true;
                }
            }
            if !open {
                return;
            }
            if !pages.needs.later(object) {
                if pages.needs.later.room() == 0 {
                    remove_later(&mut pages.needs.later, 0);
                }
                pages.needs.later.push(object).expect("room for Later");
            }
            pages.needs.cleared(object);
            pages.needs.toast = Some((Toasted::Later, env.now.saturating_add(TOAST)));
        }
        Act::Undo { object } => {
            let Some(index) = undoable(&pages.needs, object, env.now) else { return };
            let line = remove_answered(&mut pages.needs.answered, index);
            match line.label {
                Label::Reply(_) => {
                    if let Some(card) = card_mut(pages, object) {
                        card.words = match line.held {
                            Some((words, _)) => Some(words),
                            None => None,
                        };
                    }
                }
                Label::Choice(_) => {}
            }
            if let Some((toast, _)) = &pages.needs.toast {
                match toast {
                    Toasted::Answered { object: toasted, .. } => {
                        if *toasted == object {
                            pages.needs.toast = None;
                        }
                    }
                    Toasted::AwayOn { .. }
                    | Toasted::AwayOff
                    | Toasted::AwayRefused
                    | Toasted::Overturned
                    | Toasted::OverturnEnded
                    | Toasted::OverturnAlready
                    | Toasted::Later
                    | Toasted::NotSent
                    | Toasted::Paused
                    | Toasted::Resumed
                    | Toasted::Proposed { .. }
                    | Toasted::Refused { .. } => {}
                }
            }
        }
        Act::Open { object } => {
            pages.needs.at = Some(object);
            let project = match pages.address {
                Address::Home { project } => Some(project),
                Address::Inbox { .. }
                | Address::NewChat { .. }
                | Address::Goals { .. }
                | Address::Chats { .. }
                | Address::Changes { .. }
                | Address::Accounts
                | Address::Forges
                | Address::ConnectForge { .. }
                | Address::ConnectModel { .. }
                | Address::Welcome { .. }
                | Address::Chat { .. }
                | Address::Task { .. }
                | Address::Models { .. }
                | Address::Builders { .. }
                | Address::Repositories { .. }
                | Address::Settings { .. }
                | Address::Configure { .. }
                | Address::Adopt { .. } => None,
            };
            out.push(Request::Show { address: Address::Inbox { filter: Filter { project, ..Filter::ALL } } });
        }
    }
}
pub(super) fn fire(pages: &mut Pages, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
    let mut changed = false;
    if let Some((_, deadline)) = &pages.needs.toast
        && env.now >= *deadline
    {
        pages.needs.toast = None;
        changed = true;
    }
    for line in pages.needs.answered.iter_mut() {
        if let Some((_, deadline)) = &line.held
            && env.now >= *deadline
        {
            changed = true;
            let (words, _) = line.held.take().expect("due answer");
            let on = question_on(line.object, words).expect("only questions are held");
            pages.serial = pages.serial.checked_add(1).expect("pages keys fit");
            let key = Key { asker: Owner::Pages.asker(), number: pages.serial };
            line.key = Some(key);
            out.push(Request::Client(jig_client::Event::Send {
                key,
                name: None,
                request: jig_client::Ask::Jig(Box::new(contract::Ask::Answer { project: line.project, on })),
            }));
        }
    }
    changed
}
