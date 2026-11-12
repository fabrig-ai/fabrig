//! The inbox keeps cards through refreshes (temper/docs/design/client/pages/README.md).
pub mod whole;
use super::Request;
use crate::{
    Limits,
    address::{Filter, Kind},
    owner::Owner,
};
use alloc::boxed::Box;
use jig_client::{
    Client, Handle, Latest, Readable, Readout, Told, Watchable,
    contract::{self, Delivery, InboxGroup, InboxReadGroup, ObjectKey, Read, ReadResult, ReadValue},
};
use skein_lib::{List, Queue, Set};
use whole::Whole;
#[derive(Clone, Debug)]
pub struct Card {
    pub object: ObjectKey,
    pub group: InboxGroup,
    pub project: Option<u32>,
    pub position: Option<u64>,
    pub unread: Option<contract::Count>,
    pub handle: Handle,
    pub page: Option<ReadValue>,
    pub whole: Option<Whole>,
    pub words: Option<contract::Text>,
    pub key: Option<jig_client::Key>,
    pub outcome: Option<jig_client::Outcome>,
}
/// What a card's question reads (temper/docs/design/client/pages/README.md, Whole, Cut).
#[derive(Debug)]
pub enum QuestionWords<'a> {
    Whole(&'a [u8]),
    Reading(&'a [u8]),
    Cut(&'a [u8]),
}
impl Card {
    #[must_use]
    pub fn question_words(&self) -> Option<QuestionWords<'_>> {
        if let Some(ReadValue::Question(question)) = &self.page {
            return Some(if question.words.cut {
                QuestionWords::Cut(question.words.bytes())
            } else {
                QuestionWords::Whole(question.words.bytes())
            });
        }
        if let Some(ReadValue::PersonTask(task)) = &self.page
            && crate::question::in_words(task)
        {
            let words = &task.spec.specification.words;
            if !words.cut {
                return Some(QuestionWords::Whole(words.bytes()));
            }
            if let Some(whole) = &self.whole {
                if let Some(words) = whole.words() {
                    return Some(QuestionWords::Whole(words));
                }
                if whole.reading {
                    return Some(QuestionWords::Reading(words.bytes()));
                }
            }
            return Some(QuestionWords::Cut(words.bytes()));
        }
        None
    }
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "refresh bookkeeping is private to the page")]
pub struct Inbox {
    pub filter: Filter,
    pub entries: Handle,
    pub cards: List<Card>,
    /// Incomplete source documents, for needs and updates respectively.
    pub cut: [bool; 2],
    answered: [bool; 2],
    known: bool,
    previous: List<Card>,
    pending: Set<Handle>,
    /// The filtered reads' answers, applied together when both have answered.
    arrived: [Option<Box<contract::InboxPage>>; 2],
    pub queries: [Handle; 2],
    pub next: u64,
    pub window: u32,
    long_text: u32,
}
fn client(out: &mut Queue<Request>, event: jig_client::Event) {
    out.push(Request::Client(event));
}
/// Inbox's sixteen cards share the read pool with Home's other facts (client/pages/README.md).
#[must_use]
pub fn card_capacity(reads: u32) -> u32 {
    reads.min(16)
}
impl Inbox {
    pub(super) fn new(filter: Filter, first: u64, limits: &Limits, out: &mut Queue<Request>) -> Self {
        let cards = card_capacity(limits.client.domain.reads);
        let inbox = Self {
            filter,
            long_text: limits.client.domain.long_text,
            cut: [false; 2],
            entries: handle(first),
            queries: [
                handle(first.checked_add(1).expect("inbox handles fit")),
                handle(first.checked_add(2).expect("inbox handles fit")),
            ],
            next: first.checked_add(3).expect("inbox handles fit"),
            window: limits.client.domain.rows.min((cards / 2).max(1)),
            cards: List::with_capacity(cards),
            previous: List::with_capacity(cards),
            arrived: [None, None],
            answered: [false; 2],
            known: false,
            pending: Set::with_capacity(limits.client.domain.reads.checked_add(2).expect("inbox reads fit")),
        };
        client(
            out,
            jig_client::Event::Watch {
                handle: inbox.entries,
                subject: Watchable::Jig(contract::Subject::Inbox { window: inbox.window }),
            },
        );
        inbox
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [self.entries, self.queries[0], self.queries[1]] {
            client(out, jig_client::Event::Unwatch { handle });
        }
        for card in &self.cards {
            client(out, jig_client::Event::Unwatch { handle: card.handle });
            if let Some(whole) = &card.whole {
                client(out, jig_client::Event::Unwatch { handle: whole.handle });
            }
        }
        self.next
    }
    fn refresh(&mut self, source: &Client, out: &mut Queue<Request>) {
        self.arrived = [None, None];
        if self.filter.kind == Some(Kind::Finding) {
            self.replace(out);
            self.answered = [true, false];
            self.cut = [false; 2];
            return;
        }
        if self.filter.project.is_some() || self.filter.goal.is_some() {
            for (handle, group) in self.queries.into_iter().zip([InboxReadGroup::Needs, InboxReadGroup::Updates]) {
                if self.filter.kind.is_some() && group == InboxReadGroup::Updates {
                    continue;
                }
                self.pending.insert(handle).expect("inbox query reads fit");
                client(
                    out,
                    jig_client::Event::Read {
                        handle,
                        read: Readable::Jig(Read::Inbox {
                            group,
                            project: self.filter.project,
                            kind: match self.filter.kind {
                                Some(value) => kind(value),
                                None => None,
                            },
                            goal: self.filter.goal,
                            after: None,
                        }),
                    },
                );
            }
            return;
        }
        self.replace(out);
        self.cut = [false; 2];
        let latest = match source.document(self.entries) {
            Some(document) => document.latest,
            None => None,
        };
        if let Some(Latest::Jig(Delivery::Inbox(inbox))) = latest {
            self.answered = [true, self.filter.kind.is_none()];
            self.cut = [inbox.next.is_some(); 2];
            for rows in [&inbox.needs, &inbox.updates] {
                for row in rows {
                    self.add(row, out);
                }
            }
        }
    }
    fn replace(&mut self, out: &mut Queue<Request>) {
        for card in self.cards.iter_mut() {
            client(out, jig_client::Event::Unwatch { handle: card.handle });
            if let Some(whole) = &mut card.whole {
                whole.reading = false;
                client(out, jig_client::Event::Unwatch { handle: whole.handle });
            }
        }
        self.pending = Set::with_capacity(self.pending.capacity());
        self.previous.clear();
        core::mem::swap(&mut self.cards, &mut self.previous);
        // A send still awaiting confirmation keeps its Pending card.
        for old in &self.previous {
            if old.key.is_some() {
                self.cards.push(old.clone()).expect("previous cards fit");
            }
        }
    }
    fn apply(&mut self, out: &mut Queue<Request>) {
        self.replace(out);
        for index in 0..2 {
            if let Some(page) = self.arrived.get_mut(index).expect("inbox section").take() {
                *self.answered.get_mut(index).expect("inbox section") = true;
                *self.cut.get_mut(index).expect("inbox section") = page.cut;
                for row in &page.rows {
                    self.add(row, out);
                }
            } else {
                for old_index in 0..self.previous.len() {
                    let old = self.previous.get(old_index).expect("bounded card");
                    let updates = old.group == InboxGroup::Replies || old.group == InboxGroup::Results;
                    if usize::from(updates) == index {
                        self.add_card(old.object, old.group, old.project, old.position, old.unread, out);
                    }
                }
            }
        }
    }
    fn add(&mut self, row: &contract::InboxEntry, out: &mut Queue<Request>) {
        if let Some(filter) = self.filter.project
            && !row.shows_in(filter)
        {
            return;
        }
        self.add_card(
            row.card_key(),
            row.group(),
            row.project,
            position(row.object),
            unread(row.reply.as_deref()),
            out,
        );
    }
    fn add_card(
        &mut self,
        object: ObjectKey,
        group: InboxGroup,
        project: Option<u32>,
        position: Option<u64>,
        unread: Option<contract::Count>,
        out: &mut Queue<Request>,
    ) {
        if self.cards.len() == self.cards.capacity() {
            return;
        }
        if let Some(filter) = self.filter.kind
            && kind(filter) != Some(group)
        {
            return;
        }
        for card in &self.cards {
            if card.object == object {
                return;
            }
        }
        let read = match object {
            ObjectKey::Task(task) => Read::Task { task },
            ObjectKey::Proposal(proposal) => Read::Proposal { proposal },
            ObjectKey::Escalation(task) => Read::Escalation { task },
            ObjectKey::Question { task, message } => Read::Question { task, message },
            ObjectKey::PersonTask(task) => Read::PersonTask { task },
            ObjectKey::Result(task) => Read::Result { task },
            ObjectKey::Note { .. } => return,
        };
        let handle = Handle { asker: Owner::Pages.asker(), number: self.next };
        self.next = self.next.checked_add(1).expect("inbox handles fit");
        self.cards
            .push(Card {
                object,
                group,
                project,
                position,
                unread,
                handle,
                page: None,
                whole: None,
                words: None,
                key: None,
                outcome: None,
            })
            .expect("inbox cards bounded");
        for index in 0..self.previous.len() {
            let old = self.previous.get_mut(index).expect("bounded card");
            if old.object == object {
                let index = self.cards.len().checked_sub(1).expect("new card");
                let card = self.cards.get_mut(index).expect("new card");
                card.page = old.page.take();
                card.whole = old.whole.take();
                card.words = old.words.take();
                card.key = old.key;
                card.outcome = old.outcome.take();
                break;
            }
        }
        self.pending.insert(handle).expect("inbox card reads fit");
        client(out, jig_client::Event::Read { handle, read: Readable::Jig(read) });
        self.read_whole(self.cards.len().checked_sub(1).expect("new card"), out);
    }
    fn read_whole(&mut self, index: u32, out: &mut Queue<Request>) {
        let card = self.cards.get_mut(index).expect("bounded card index");
        let Some(ReadValue::PersonTask(task)) = &card.page else { return };
        if !crate::question::in_words(task) || !task.spec.specification.words.cut {
            return;
        }
        if let Some(whole) = &card.whole
            && (whole.reading || whole.words().is_some())
        {
            return;
        }
        let handle = handle(self.next);
        self.next = self.next.checked_add(1).expect("inbox handles fit");
        card.whole = Some(Whole::start(task.task.task, handle, out));
    }
    /// Whether this section's read has answered with its value since the page opened
    /// (temper/docs/design/client/pages/README.md).
    #[must_use]
    pub const fn answered(&self, group: InboxReadGroup) -> bool {
        match group {
            InboxReadGroup::Needs => *self.answered.first().expect("needs section"),
            InboxReadGroup::Updates => *self.answered.last().expect("updates section"),
        }
    }
    /// The needs' read has answered and every card read it sent has: what the cards say
    /// stands, through each later refresh, until the next answer changes it.
    #[must_use]
    pub const fn known(&self) -> bool {
        self.known
    }
    pub(super) fn told(&mut self, source: &Client, told: Told, out: &mut Queue<Request>) -> bool {
        let changed = self.receive(source, told, out);
        self.known |= self.answered(InboxReadGroup::Needs) && self.delivered(source);
        changed
    }
    fn receive(&mut self, source: &Client, told: Told, out: &mut Queue<Request>) -> bool {
        if let Told::Changed { handle } = &told
            && *handle == self.entries
        {
            self.refresh(source, out);
            return true;
        }
        let Told::Page { handle, page } = told else { return false };
        for card in self.cards.iter_mut() {
            if let Some(whole) = &mut card.whole
                && whole.handle == handle
                && let ObjectKey::PersonTask(task) = card.object
            {
                whole.told(task, self.long_text, page, out);
                return true;
            }
        }
        let answered = self.pending.remove(&handle);
        if self.queries.contains(&handle) && (self.filter.project.is_some() || self.filter.goal.is_some()) {
            let index = usize::from(handle == self.queries[1]);
            if let Readout::Jig(ReadResult::Value(ReadValue::Inbox(page))) = page {
                *self.arrived.get_mut(index).expect("query names one section") = Some(page);
            }
            if !self.pending.contains(&self.queries[0]) && !self.pending.contains(&self.queries[1]) {
                self.apply(out);
            }
        } else {
            let Readout::Jig(ReadResult::Value(value)) = page else { return answered };
            for index in 0..self.cards.len() {
                let card = self.cards.get_mut(index).expect("bounded card index");
                if card.handle == handle {
                    card.page = Some(value);
                    self.read_whole(index, out);
                    return true;
                }
            }
            return false;
        }
        true
    }
    /// The needs' groups in the inbox's order (temper/docs/design/client/pages/README.md, Home).
    #[expect(clippy::disallowed_methods, reason = "Home's contract exposes a filtered iterator over the inbox's cards")]
    pub fn needs(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter().filter(|card| match card.group {
            InboxGroup::Proposals | InboxGroup::Questions | InboxGroup::Escalations | InboxGroup::PersonTasks => true,
            InboxGroup::Results | InboxGroup::Replies => false,
        })
    }
    /// The entries have arrived and every read sent by this refresh has answered.
    #[must_use]
    pub fn delivered(&self, client: &Client) -> bool {
        let Some(document) = client.document(self.entries) else { return false };
        document.latest.is_some() && self.pending.is_empty()
    }
    #[must_use]
    pub fn results_through(&self) -> Option<u64> {
        let mut through = None;
        for card in &self.cards {
            if card.group == InboxGroup::Results {
                through = through.max(card.position);
            }
        }
        through
    }
}
#[must_use]
pub const fn kind(kind: Kind) -> Option<InboxGroup> {
    match kind {
        Kind::Proposal => Some(InboxGroup::Proposals),
        Kind::Question => Some(InboxGroup::Questions),
        Kind::Held => Some(InboxGroup::Escalations),
        Kind::PersonTask => Some(InboxGroup::PersonTasks),
        Kind::Finding => None,
    }
}
fn handle(number: u64) -> Handle {
    Handle { asker: Owner::Pages.asker(), number }
}
fn position(object: contract::InboxObject) -> Option<u64> {
    if let contract::InboxObject::Result { position } = object { Some(position) } else { None }
}

fn unread(reply: Option<&contract::TaskLine>) -> Option<contract::Count> {
    match reply {
        Some(line) => match &line.conversation {
            Some(conversation) => Some(conversation.unread),
            None => None,
        },
        None => None,
    }
}
