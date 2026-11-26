//! The inbox keeps cards through refreshes (temper/docs/design/client/pages/README.md).
mod tasks;
pub mod whole;
use super::{
    Request,
    activity::{self, Asking},
};
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
use skein_lib::{List, Queue, Set, Time};
pub use tasks::TaskRead;
use whole::Whole;

const ROWS: u32 = 64;
#[derive(Clone, Debug)]
pub struct Card {
    pub object: ObjectKey,
    pub group: InboxGroup,
    pub project: Option<u32>,
    pub position: Option<u64>,
    pub unread: Option<contract::Count>,
    pub handle: Handle,
    pub page: Option<ReadValue>,
    /// A refused proposal read still lets its signed-in maker withdraw by key.
    pub withdraw_unread: bool,
    pub whole: Option<Whole>,
    pub task: Option<TaskRead>,
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
    pub fn withdrawn(&self) -> bool {
        if let Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(contract::Answer::Done(
            contract::Outcome::Document(contract::DocumentOutcome::Withdrawn { .. }),
        )))) = &self.outcome
        {
            return true;
        }
        false
    }
    pub(crate) fn receive(&mut self, page: Readout, source: &Client) {
        self.withdraw_unread = false;
        match page {
            Readout::Jig(ReadResult::Value(value)) => self.page = Some(value),
            Readout::Jig(ReadResult::None(contract::ReadNone::Refused | contract::ReadNone::Malformed)) => {
                self.page = None;
                if let ObjectKey::Proposal(contract::ProposalKey { proposer: contract::Proposer::Party(maker), .. }) =
                    self.object
                    && let Some(me) = super::accounts::current_me(source)
                {
                    self.withdraw_unread = me.party.number == maker;
                }
            }
            Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => {}
        }
    }
    /// The task named by the from line; proposals deliberately name none.
    #[must_use]
    pub fn source(&self) -> Option<u64> {
        if let Some(ReadValue::PersonTask(task)) = &self.page
            && crate::question::in_words(task)
            && let contract::Requester::Task(requester) = task.requester
        {
            return Some(requester);
        }
        match self.object {
            ObjectKey::Proposal(_) | ObjectKey::Note { .. } => None,
            ObjectKey::Task(task)
            | ObjectKey::Escalation(task)
            | ObjectKey::PersonTask(task)
            | ObjectKey::Result(task)
            | ObjectKey::Question { task, .. } => Some(task),
        }
    }
    #[must_use]
    pub fn document(&self) -> Option<&contract::Task> {
        let source = self.source()?;
        match &self.page {
            Some(ReadValue::Task(task)) if task.chip.task == source => return Some(task),
            Some(ReadValue::Escalation(held)) if held.task.chip.task == source => return Some(&held.task),
            Some(_) | None => {}
        }
        self.task.as_ref()?.document.as_deref()
    }
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
    /// Needs' rows through the last page, bounded independently of the cards.
    pub rows: List<contract::InboxEntry>,
    needs_page: Option<Box<contract::InboxPage>>,
    needs_asking: Asking,
    /// The needs read's supplied ancestry, retained for milestone bands.
    pub selection: Option<List<contract::InboxSelection>>,
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
    pub goal: Option<TaskRead>,
    pub milestone: Option<Box<super::milestones::MilestoneRead>>,
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
        let mut inbox = Self {
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
            rows: List::with_capacity(ROWS),
            needs_page: None,
            needs_asking: Asking::new(),
            selection: None,
            previous: List::with_capacity(cards),
            arrived: [None, None],
            answered: [false; 2],
            known: false,
            goal: None,
            milestone: None,
            pending: Set::with_capacity(limits.client.domain.reads.checked_add(2).expect("inbox reads fit")),
        };
        if let Some(goal) = filter.goal {
            inbox.goal = Some(TaskRead::new(goal, &mut inbox.next, out));
        }
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
            if let Some(task) = &card.task {
                task.leave(out);
            }
            if let Some(whole) = &card.whole {
                client(out, jig_client::Event::Unwatch { handle: whole.handle });
            }
        }
        if let Some(goal) = &self.goal {
            goal.leave(out);
        }
        if let Some(milestone) = &self.milestone {
            milestone.leave(out);
        }
        self.next
    }
    pub(super) fn configure(&mut self, client: &Client, limits: &Limits, out: &mut Queue<Request>) {
        if let Some(goal) = &self.goal
            && let Some(task) = &goal.document
            && temper_client_domain_forge::milestones::is(&task.chip)
            && self.milestone.is_none()
        {
            self.milestone =
                Some(Box::new(super::milestones::MilestoneRead::new(task.project, task.chip.task, &mut self.next)));
        }
        if let Some(milestone) = &mut self.milestone {
            milestone.configure_goals(client, limits, out);
        }
    }
    #[must_use]
    pub fn takes_goal(&self, goal: Option<u64>) -> bool {
        if self.filter.goal.is_none() || self.filter.goal == goal {
            return true;
        }
        if let Some(milestone) = &self.milestone {
            for line in &milestone.goals {
                if Some(line.chip.task) == goal {
                    return true;
                }
            }
        }
        false
    }
    fn refresh(&mut self, source: &Client, out: &mut Queue<Request>) {
        self.arrived = [None, None];
        self.needs_page = None;
        self.needs_asking = Asking::new();
        if self.filter.kind == Some(Kind::Finding) {
            self.replace(out);
            self.release_previous(out);
            self.rows.clear();
            self.answered = [true, false];
            self.cut = [false; 2];
            return;
        }
        if self.filter.project.is_some() || self.filter.goal.is_some() {
            for group in [InboxReadGroup::Needs, InboxReadGroup::Updates] {
                if self.filter.kind.is_some() && group == InboxReadGroup::Updates {
                    continue;
                }
                self.read_section(group, None, out);
            }
            return;
        }
        let mut read_needs = false;
        let latest = match source.document(self.entries) {
            Some(document) => document.latest,
            None => None,
        };
        if let Some(Latest::Jig(Delivery::Inbox(inbox))) = latest {
            read_needs = inbox.next.is_some() && inbox.updates.is_empty();
            if !read_needs {
                let mut rows = List::with_capacity(ROWS);
                let mut cut = false;
                for row in &inbox.needs {
                    if rows.len() == rows.capacity() {
                        cut = true;
                        break;
                    }
                    rows.push(row.clone()).expect("needs rows bounded");
                }
                self.arrived[0] =
                    Some(Box::new(contract::InboxPage { rows, selection: None, before: None, after: None, cut }));
            }
            if self.filter.kind.is_none() {
                self.arrived[1] = Some(Box::new(contract::InboxPage {
                    rows: inbox.updates.clone(),
                    selection: None,
                    before: None,
                    after: None,
                    cut: inbox.next.is_some(),
                }));
            }
        }
        if read_needs {
            self.read_section(InboxReadGroup::Needs, None, out);
        } else {
            self.apply(out);
        }
    }
    pub(super) fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        if let Some(at) = self.needs_asking.next_deadline()
            && at <= now
        {
            self.pending.insert(self.queries[0]).expect("inbox query reads fit");
            self.needs_asking.fire(now, self.queries[0], out);
        }
    }
    pub(super) const fn next_deadline(&self) -> Option<Time> {
        self.needs_asking.next_deadline()
    }
    fn read_section(&mut self, group: InboxReadGroup, after: Option<contract::Cursor>, out: &mut Queue<Request>) {
        let handle = match group {
            InboxReadGroup::Needs => self.queries[0],
            InboxReadGroup::Updates => self.queries[1],
        };
        self.pending.insert(handle).expect("inbox query reads fit");
        let first_needs = group == InboxReadGroup::Needs && after.is_none();
        let read = Read::Inbox {
            group,
            project: self.filter.project,
            kind: match self.filter.kind {
                Some(value) => kind(value),
                None => None,
            },
            goal: self.filter.goal,
            after,
        };
        if first_needs {
            self.needs_asking.ask(handle, read, out);
        } else {
            client(out, jig_client::Event::Read { handle, read: Readable::Jig(read) });
        }
    }
    /// Stage one needs chain; only its terminal answer replaces the visible rows.
    fn needs_answer(&mut self, source: &Client, now: Time, page: Readout, out: &mut Queue<Request>) -> bool {
        if let Readout::Jig(ReadResult::Value(ReadValue::Inbox(page))) = page {
            if self.needs_page.is_none() {
                self.needs_page = Some(Box::new(contract::InboxPage {
                    rows: List::with_capacity(ROWS),
                    selection: None,
                    before: None,
                    after: None,
                    cut: false,
                }));
            }
            let staged = self.needs_page.as_mut().expect("needs chain started");
            if staged.selection.is_none() {
                staged.selection.clone_from(&page.selection);
            } else if let (Some(held), Some(selection)) = (&mut staged.selection, &page.selection) {
                let mut joined = List::with_capacity(ROWS);
                for row in held.iter().chain(selection.iter()) {
                    if joined.len() < joined.capacity() {
                        joined.push(row.clone()).expect("selection bounded");
                    }
                }
                *held = joined;
            }
            let mut overflow = false;
            for row in &page.rows {
                if staged.rows.len() == staged.rows.capacity() {
                    overflow = true;
                    break;
                }
                staged.rows.push(row.clone()).expect("needs rows bounded");
            }
            staged.cut |= page.cut || overflow || (staged.rows.len() == ROWS && page.after.is_some());
            if page.after.is_some() && staged.rows.len() < staged.rows.capacity() {
                self.read_section(InboxReadGroup::Needs, page.after, out);
                return false;
            }
            self.needs_asking.answered();
        } else {
            self.needs_page = None;
            if activity::read_end(source, &page).transient() {
                self.needs_asking.ended(now);
            } else {
                self.needs_asking.answered();
            }
        }
        self.arrived[0] = self.needs_page.take();
        true
    }
    fn release_previous(&mut self, out: &mut Queue<Request>) {
        for card in &self.previous {
            if let Some(task) = &card.task {
                let mut kept = false;
                for current in &self.cards {
                    if let Some(read) = &current.task {
                        kept |= read.handle == task.handle;
                    }
                }
                if !kept {
                    task.leave(out);
                }
            }
        }
        self.previous.clear();
    }
    fn withdrawn_and_gone(&self, card: &Card) -> bool {
        if !card.withdrawn() {
            return false;
        }
        for row in &self.rows {
            if row.card_key() == card.object {
                return false;
            }
        }
        true
    }
    /// A watch may remove the entry before the withdrawing ask answers.
    pub(super) fn discard_withdrawn(&mut self, out: &mut Queue<Request>) {
        let mut removed = false;
        for card in &self.cards {
            removed |= self.withdrawn_and_gone(card);
        }
        if !removed {
            return;
        }
        let mut previous = List::with_capacity(self.cards.capacity());
        core::mem::swap(&mut self.cards, &mut previous);
        for card in previous.into_boxed() {
            if self.withdrawn_and_gone(&card) {
                client(out, jig_client::Event::Unwatch { handle: card.handle });
                self.pending.remove(&card.handle);
            } else {
                self.cards.push(card).expect("retained cards fit");
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
            if old.key.is_some() && !old.withdrawn() {
                self.cards.push(old.clone()).expect("previous cards fit");
            }
        }
    }
    fn apply(&mut self, out: &mut Queue<Request>) {
        self.replace(out);
        for index in 0..2 {
            if let Some(page) = self.arrived.get_mut(index).expect("inbox section").take() {
                if index == 0 {
                    self.selection.clone_from(&page.selection);
                    self.rows.clone_from(&page.rows);
                }
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
        self.release_previous(out);
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
        let section = InboxReadGroup::of(group);
        let mut count = 0_u32;
        for card in &self.cards {
            if InboxReadGroup::of(card.group) == section {
                count = count.checked_add(1).expect("bounded cards");
            }
        }
        if section == InboxReadGroup::Needs && count >= self.window {
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
                withdraw_unread: false,
                whole: None,
                task: None,
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
                card.withdraw_unread = old.withdraw_unread;
                card.whole = old.whole.take();
                card.task = old.task.take();
                card.words = old.words.take();
                card.key = old.key;
                card.outcome = old.outcome.take();
                break;
            }
        }
        self.pending.insert(handle).expect("inbox card reads fit");
        client(out, jig_client::Event::Read { handle, read: Readable::Jig(read) });
        let index = self.cards.len().checked_sub(1).expect("new card");
        self.read_whole(index, out);
        self.read_task(index, out);
    }
    /// Read a needs row on demand, reusing the last needs card's place.
    pub(super) fn open(&mut self, object: ObjectKey, out: &mut Queue<Request>) {
        for card in &self.cards {
            if card.object == object {
                return;
            }
        }
        let mut requested = None;
        let mut last = None;
        for row in &self.rows {
            if row.card_key() == object {
                requested = Some(row.clone());
            }
            for (index, card) in self.cards.iter().enumerate() {
                if card.object == row.card_key() {
                    last = Some(index);
                }
            }
        }
        let Some(row) = requested else { return };
        if let Some(last) = last {
            let mut previous = List::with_capacity(self.cards.capacity());
            core::mem::swap(&mut self.cards, &mut previous);
            for (index, card) in previous.into_boxed().into_iter().enumerate() {
                if index == last {
                    client(out, jig_client::Event::Unwatch { handle: card.handle });
                    self.pending.remove(&card.handle);
                    if let Some(whole) = &card.whole {
                        client(out, jig_client::Event::Unwatch { handle: whole.handle });
                    }
                    self.previous.push(card).expect("one replaced needs card");
                } else {
                    self.cards.push(card).expect("kept cards fit");
                }
            }
        }
        self.add(&row, out);
        self.release_previous(out);
    }
    fn read_task(&mut self, index: u32, out: &mut Queue<Request>) {
        let card = self.cards.get_mut(index).expect("bounded card index");
        let Some(source) = card.source() else { return };
        if card.document().is_some() || card.task.is_some() {
            return;
        }
        // A person task in words names its requester only after the card has arrived.
        if card.page.is_none() {
            return;
        }
        card.task = Some(TaskRead::new(source, &mut self.next, out));
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
    pub(super) fn told(
        &mut self,
        source: &Client,
        now: Time,
        told: Told,
        limits: &Limits,
        out: &mut Queue<Request>,
    ) -> bool {
        if let Some(milestone) = &mut self.milestone
            && milestone.told(&told, source, limits, out)
        {
            return true;
        }
        if let Some(goal) = &mut self.goal
            && goal.told(&told)
        {
            return true;
        }
        for card in self.cards.iter_mut() {
            if let Some(task) = &mut card.task
                && task.told(&told)
            {
                return true;
            }
        }
        let changed = self.receive(source, now, told, out);
        self.known |= self.answered(InboxReadGroup::Needs) && self.delivered(source);
        changed
    }
    fn receive(&mut self, source: &Client, now: Time, told: Told, out: &mut Queue<Request>) -> bool {
        if let Told::Changed { handle } = &told
            && *handle == self.entries
        {
            self.refresh(source, out);
            return true;
        }
        if let Told::Ended { handle, .. } = &told
            && self.queries.contains(handle)
        {
            self.pending.remove(handle);
            if *handle == self.queries[0] {
                self.needs_page = None;
                self.needs_asking.answered();
            }
            if !self.pending.contains(&self.queries[0]) && !self.pending.contains(&self.queries[1]) {
                self.apply(out);
            }
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
        if self.queries.contains(&handle) {
            if handle == self.queries[0] {
                if !self.needs_answer(source, now, page, out) {
                    return true;
                }
            } else if let Readout::Jig(ReadResult::Value(ReadValue::Inbox(page))) = page {
                self.arrived[1] = Some(page);
            }
            if !self.pending.contains(&self.queries[0]) && !self.pending.contains(&self.queries[1]) {
                self.apply(out);
            }
        } else {
            for index in 0..self.cards.len() {
                let card = self.cards.get_mut(index).expect("bounded card index");
                if card.handle == handle {
                    card.receive(page, source);
                    self.read_whole(index, out);
                    self.read_task(index, out);
                    return true;
                }
            }
            return answered;
        }
        true
    }
    /// The party's row for a need, whether or not its card is read.
    #[must_use]
    #[expect(clippy::manual_find, reason = "the step subset does not use closures")]
    pub fn need_row(&self, object: ObjectKey) -> Option<&contract::InboxEntry> {
        for row in &self.rows {
            if row.card_key() == object {
                return Some(row);
            }
        }
        None
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
