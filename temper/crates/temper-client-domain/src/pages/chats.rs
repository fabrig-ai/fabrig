//! Chats and their document provenance (temper/docs/design/client/pages/README.md).
//! Live rows are borrowed from the client; closed rows are one bounded read page.
use super::{
    Request,
    chat::document::{Mode, WRITE_THIS_UP},
};
use crate::address::ChatsFilter;
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::contract::{
    self as c, Delivery, EndedGroup, Me, Read, ReadResult, ReadValue, TaskPage, TaskQuery, Text,
};
use jig_client::{Client, Handle, Key, Latest, Readable, Readout, Told, Watchable};
use skein_lib::{Env, List, Map, Queue, Time, Writer};

/// Ask temper's stock template (temper/docs/design/protocol/configuration/stock.md).
pub const ASKING_TEMPLATE: u32 = 5;

/// A design session's stock template (temper/docs/design/protocol/configuration/stock.md).
pub const DRAFTING_TEMPLATE: u32 = 6;

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the configured query is private watch bookkeeping")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "pagination, delivery and independent read answers are separate facts"
)]
pub struct Chats {
    pub filter: ChatsFilter,
    pub live: Handle,
    pub closed: Handle,
    pub documents: Map<u64, c::DocumentText>,
    pub notes: Handle,
    pub goals: Handle,
    notes_project: Option<u32>,
    notes_seen: u32,
    note_numbers: Map<u64, u64>,
    asking: super::activity::Asking,
    goal_tasks: List<u64>,
    pub words: Box<[u8]>,
    pub beyond: u64,
    pub start: Option<Key>,
    pub page: Option<TaskPage>,
    pub refusal: Option<jig_client::shapes::NotOffered>,
    pub outcome: Option<jig_client::Outcome>,
    pub(crate) opening: Option<(u64, Mode)>,
    starting_mode: Mode,
    keep_words: bool,
    query: Option<TaskQuery>,
    first: bool,
    page_delivered: bool,
    page_answered: bool,
    documents_known: bool,
}
impl Chats {
    pub(crate) fn new(first: u64) -> Self {
        Self {
            filter: ChatsFilter::All,
            live: handle(first),
            closed: handle(first.checked_add(1).expect("page handles fit")),
            documents: Map::with_capacity(0),
            notes: handle(first.checked_add(2).expect("page handles fit")),
            goals: handle(first.checked_add(3).expect("page handles fit")),
            notes_project: None,
            notes_seen: 0,
            note_numbers: Map::with_capacity(0),
            asking: super::activity::Asking::new(),
            goal_tasks: List::with_capacity(0),
            words: Box::default(),
            beyond: 0,
            start: None,
            page: None,
            refusal: None,
            outcome: None,
            opening: None,
            starting_mode: Mode::Chat,
            keep_words: false,
            query: None,
            first: true,
            page_delivered: false,
            page_answered: false,
            documents_known: false,
        }
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [self.live, self.closed] {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        if self.notes_project.is_some() {
            for handle in [self.notes, self.goals] {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
        }
        self.goals.number.checked_add(1).expect("page handles fit")
    }
    pub(crate) fn configure(
        &mut self,
        me: Option<&Me>,
        project: Option<u32>,
        limits: &Limits,
        out: &mut Queue<Request>,
    ) {
        let (Some(me), Some(project)) = (me, project) else { return };
        let Some(template) = jig_client::shapes::template(me, project, jig_client::shapes::CHAT) else { return };
        let query = TaskQuery {
            project: Some(project),
            requester: Some(c::Requester::Person(me.party.number)),
            root: None,
            parent: None,
            label: Some(template.label),
            phase: None,
            executor: None,
            tracked: None,
            order: c::QueryOrder::Activity,
        };
        if self.query == Some(query) {
            return;
        }
        if self.query.is_some() {
            self.leave(out);
        }
        self.notes_project = None;
        self.asking.answered();
        self.goal_tasks = List::with_capacity(0);
        self.documents = Map::with_capacity(0);
        self.note_numbers = Map::with_capacity(0);
        self.page = None;
        self.page_delivered = false;
        self.page_answered = false;
        self.documents_known = false;
        self.first = true;
        self.query = Some(query);
        out.push(Request::Client(jig_client::Event::Watch {
            handle: self.live,
            subject: Watchable::Jig(c::Subject::Tasks { query, window: limits.client.domain.rows }),
        }));
        self.read(None, out);
    }
    pub(crate) fn open_documents(&mut self, project: u32, rows: u32, out: &mut Queue<Request>) {
        if self.notes_project == Some(project) {
            return;
        }
        self.documents_known = false;
        self.notes_project = Some(project);
        self.notes_seen = 0;
        self.documents = Map::with_capacity(rows);
        self.note_numbers = Map::with_capacity(rows);
        self.asking.ask(
            self.notes,
            Read::Notes { scope: c::NoteScope::Tasks { project }, description: None, after: None },
            out,
        );
    }
    pub(crate) fn close_documents(&mut self, out: &mut Queue<Request>) {
        self.documents_known = false;
        if self.notes_project.take().is_none() {
            return;
        }
        for handle in [self.notes, self.goals] {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        self.asking.answered();
        self.documents = Map::with_capacity(0);
        self.note_numbers = Map::with_capacity(0);
        self.goal_tasks = List::with_capacity(0);
    }
    pub(crate) fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        self.asking.fire(now, self.notes, out);
    }
    pub(crate) const fn next_deadline(&self) -> Option<Time> {
        self.asking.next_deadline()
    }
    pub(crate) fn notes_told(&mut self, now: Time, client: &Client, told: &Told, out: &mut Queue<Request>) -> bool {
        let Told::Page { handle, page } = told else { return false };
        if *handle != self.notes {
            return false;
        }
        if let Readout::Jig(ReadResult::Value(ReadValue::Notes(page))) = page {
            self.documents_known = true;
            self.asking.answered();
            let Some(project) = self.notes_project else { return true };
            let bound = self.documents.capacity();
            for note in &page.rows {
                if self.notes_seen >= bound {
                    break;
                }
                self.notes_seen = self.notes_seen.checked_add(1).expect("notes bounded by rows");
                if let c::NoteScope::Task { project: held, task } = note.scope
                    && held == project
                    && !note.description.bytes().starts_with(b"work:")
                    && match self.note_numbers.get(&task) {
                        Some(number) => note.number < *number,
                        None => true,
                    }
                {
                    self.note_numbers.insert(task, note.number).expect("at most rows notes");
                    self.documents.insert(task, note.description.clone()).expect("at most rows documents");
                }
            }
            if self.notes_seen < bound
                && let Some(after) = &page.after
            {
                self.asking.ask(
                    self.notes,
                    Read::Notes {
                        scope: c::NoteScope::Tasks { project },
                        description: None,
                        after: Some(after.clone()),
                    },
                    out,
                );
            }
        } else if super::activity::read_end(client, page).transient() {
            self.asking.ended(now);
        } else {
            self.asking.answered();
            self.documents = Map::with_capacity(self.documents.capacity());
            self.note_numbers = Map::with_capacity(self.note_numbers.capacity());
        }
        true
    }
    /// The newest goal this person started from a row, breaking clock ties by goal number.
    #[must_use]
    pub fn goal_from(forge: &crate::forge::Side, project: u32, task: u64) -> Option<u64> {
        let mut newest = None;
        for (goal, at) in forge.goals_from(project, task) {
            if match newest {
                Some(prior) => (at, goal) > prior,
                None => true,
            } {
                newest = Some((at, goal));
            }
        }
        match newest {
            Some((_, goal)) => Some(goal),
            None => None,
        }
    }
    pub(crate) fn goal_watch(
        &mut self,
        client: &Client,
        forge: &crate::forge::Side,
        rows: u32,
        out: &mut Queue<Request>,
    ) {
        let Some(project) = self.notes_project else { return };
        let mut tasks = List::with_capacity(rows);
        let live = match self.live(client) {
            Some(live) => live.rows.as_slice(),
            None => &[],
        };
        let closed = match &self.page {
            Some(page) => match &page.ended {
                EndedGroup::Rows { rows, .. } => rows.as_slice(),
                EndedGroup::None => &[],
            },
            None => &[],
        };
        // The newest rows across both windows share one chips window.
        let mut previous = None;
        for _ in 0..rows {
            let mut next: Option<(skein_lib::Wall, u64)> = None;
            for lines in [live, closed] {
                for line in lines {
                    let key = (line.ended_at.unwrap_or(line.last_activity), line.chip.task);
                    if let Some(prior) = previous
                        && !newer(prior, key)
                    {
                        continue;
                    }
                    if let Some(chosen) = next
                        && !newer(key, chosen)
                    {
                        continue;
                    }
                    next = Some(key);
                }
            }
            let Some(key) = next else { break };
            previous = Some(key);
            if let Some(goal) = Self::goal_from(forge, project, key.1)
                && !tasks.as_slice().contains(&goal)
            {
                tasks.push(goal).expect("one goal per selected row");
            }
        }
        if tasks == self.goal_tasks {
            return;
        }
        if !self.goal_tasks.is_empty() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle: self.goals }));
        }
        if !tasks.is_empty() {
            out.push(Request::Client(jig_client::Event::Watch {
                handle: self.goals,
                subject: Watchable::Jig(c::Subject::Chips { tasks: tasks.clone() }),
            }));
        }
        self.goal_tasks = tasks;
    }
    #[must_use]
    pub fn goal_chip<'a>(&self, client: &'a Client, goal: u64) -> Option<&'a c::Chip> {
        let document = client.document(self.goals)?;
        if document.state == jig_client::State::Ended {
            return None;
        }
        if let Some(Latest::Jig(Delivery::Chips(chips))) = document.latest {
            for chip in chips {
                if chip.task == goal {
                    return Some(chip);
                }
            }
        }
        None
    }
    fn read(&self, ended: Option<c::Cursor>, out: &mut Queue<Request>) {
        if let Some(query) = self.query {
            out.push(Request::Client(jig_client::Event::Read {
                handle: self.closed,
                read: Readable::Jig(Read::Tasks {
                    query: TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query },
                    live: None,
                    ended,
                }),
            }));
        }
    }
    pub(crate) fn older(&mut self, out: &mut Queue<Request>) {
        if let Some(page) = &self.page
            && let EndedGroup::Rows { after: Some(after), .. } = &page.ended
        {
            self.first = false;
            self.read(Some(after.clone()), out);
        }
    }
    pub(crate) fn write(&mut self, words: Box<[u8]>, limits: &Limits) {
        (self.words, self.beyond) = super::composer::held(words, limits.client.domain.long_text);
    }
    #[expect(clippy::too_many_arguments, reason = "the route selects the first message and opening mode")]
    pub(crate) fn start(
        &mut self,
        env: &Env<Limits>,
        me: Option<&Me>,
        project: Option<u32>,
        choice: &super::choice::Draft,
        draft: bool,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        if self.start.is_some() || self.words.is_empty() {
            return;
        }
        let Some(me) = me else { return };
        if super::composer::over(&self.words, self.beyond, me.words_bytes) > 0 {
            return;
        }
        let words = if draft {
            let Some(size) = self.words.len().checked_add(2) else { return };
            let Some(size) = size.checked_add(WRITE_THIS_UP.len()) else { return };
            let mut words = Writer::new(size);
            words.put(&self.words).expect("measured words");
            words.put(b"\n\n").expect("measured separator");
            words.put(WRITE_THIS_UP).expect("measured draft request");
            words.finish()
        } else {
            self.words.clone()
        };
        let template = if draft
            && let Some(project) = project
            && jig_client::shapes::template(me, project, DRAFTING_TEMPLATE).is_some()
        {
            DRAFTING_TEMPLATE
        } else {
            jig_client::shapes::CHAT
        };
        self.send(env, Some(me), project, template, words, Some(choice), serial, out);
        if self.start.is_some() {
            self.starting_mode = if draft { Mode::Split } else { Mode::Chat };
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "suggested words share the composer's identity, choice and send state"
    )]
    pub(crate) fn suggest(
        &mut self,
        env: &Env<Limits>,
        me: Option<&Me>,
        project: Option<u32>,
        words: &[u8],
        choice: &super::choice::Draft,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        if words.len() > usize::try_from(temper_client_protocol_wire::CUE_BYTES).expect("cue bound fits") {
            return;
        }
        self.send(env, me, project, jig_client::shapes::CHAT, words.into(), Some(choice), serial, out);
    }
    #[expect(clippy::too_many_arguments, reason = "Ask starts share the composer's identity, choice and send state")]
    pub(crate) fn ask_chat(
        &mut self,
        env: &Env<Limits>,
        me: Option<&Me>,
        project: Option<u32>,
        how: bool,
        choice: &super::choice::Draft,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        if self.start.is_some() {
            return;
        }
        let Some(me) = me else { return };
        let words = if how { super::chat::document::HOW_TEMPER_WORKS.into() } else { self.words.clone() };
        let beyond = if how { 0 } else { self.beyond };
        if super::composer::over(&words, beyond, me.words_bytes) > 0 {
            return;
        }
        let template = ASKING_TEMPLATE;
        if let Some(project) = project
            && jig_client::shapes::template(me, project, template).is_none()
        {
            return;
        }
        self.send(env, Some(me), project, template, words, Some(choice), serial, out);
        if self.start.is_some() {
            self.keep_words = how;
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "chat starts supply their template and words and borrow the same bounded draft"
    )]
    fn send(
        &mut self,
        _env: &Env<Limits>,
        me: Option<&Me>,
        project: Option<u32>,
        template: u32,
        words: Box<[u8]>,
        choice: Option<&super::choice::Draft>,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        if self.start.is_some() || words.is_empty() {
            return;
        }
        let (Some(me), Some(project)) = (me, project) else { return };
        if let Some(choice) = choice
            && !choice.valid(me)
        {
            return;
        }
        // Callers bound typed words or cues before a draft's instruction is appended.
        let Ok(words) = Text::new(words, u32::MAX) else { return };
        let ask = if template == jig_client::shapes::CHAT {
            jig_client::shapes::chat(me, project, words)
        } else {
            jig_client::shapes::chat_from(me, project, template, words)
        };
        match ask {
            Ok(mut ask) => {
                if let c::Ask::Create { tasks, .. } = &mut ask
                    && let Some(task) = tasks.get_mut(0)
                    && let Some(choice) = choice
                    && choice.context.is_some()
                {
                    task.llm.clone_from(&choice.requested);
                }
                *serial = serial.checked_add(1).expect("page keys fit");
                let key = Key { asker: Owner::Pages.asker(), number: *serial };
                self.start = Some(key);
                self.starting_mode = Mode::Chat;
                self.keep_words = false;
                self.refusal = None;
                self.outcome = None;
                out.push(Request::Client(jig_client::Event::Send {
                    key,
                    name: None,
                    request: jig_client::Ask::Jig(Box::new(ask)),
                }));
            }
            Err(why) => self.refusal = Some(why),
        }
    }
    pub(crate) fn told(&mut self, told: Told, address: &Address, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if handle == self.live => {
                // Live deliveries omit ended tasks. Refresh the first ended page
                // alongside them, including when its previous read is in flight.
                if self.first {
                    self.read(None, out);
                }
                true
            }
            Told::Page { handle, page: Readout::Jig(ReadResult::Value(ReadValue::Tasks(page))) }
                if handle == self.closed =>
            {
                self.page_delivered = true;
                self.page_answered = true;
                self.page = Some(*page);
                true
            }
            Told::Page { handle, .. } if handle == self.closed => {
                self.page_delivered = true;
                true
            }
            Told::Ended { handle, .. } if handle == self.live => true,
            Told::Changed { handle } | Told::Ended { handle, .. } if handle == self.goals => true,
            Told::Pending { key } if Some(key) == self.start => true,
            Told::Answered { key, outcome } if Some(key) == self.start => {
                if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(value))) = &outcome {
                    if let c::Outcome::Document(c::DocumentOutcome::Created { first, .. }) = value {
                        self.opening = Some((*first, self.starting_mode));
                        show_chat(*first, address, out);
                        if !self.keep_words {
                            self.words = Box::default();
                            self.beyond = 0;
                        }
                    } else if let c::Outcome::Made { what: c::Made::Tasks { tasks } } = value
                        && let Some(task) = tasks.get(0)
                    {
                        self.opening = Some((*task, self.starting_mode));
                        show_chat(*task, address, out);
                        if !self.keep_words {
                            self.words = Box::default();
                            self.beyond = 0;
                        }
                    }
                }
                self.outcome = Some(outcome);
                self.start = None;
                true
            }
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => false,
        }
    }
    /// The live watch has delivered and the closed read has answered with its value, for this query.
    #[must_use]
    pub fn known(&self, client: &Client) -> bool {
        self.page_answered && self.live(client).is_some()
    }
    /// The notes read has answered with its value, for the Drafts filter.
    #[must_use]
    pub const fn documents_known(&self) -> bool {
        self.documents_known
    }
    pub(super) fn delivered(&self, client: &Client, project: u32) -> bool {
        let Some(query) = self.query else { return false };
        query.project == Some(project) && self.page_delivered && super::home::delivered(client, self.live)
    }
    pub(super) fn chat_counts(&self, client: &Client, me: Option<&Me>, project: u32) -> bool {
        let Some(query) = self.query else { return false };
        if query.project != Some(project) {
            return false;
        }
        let mut asking = false;
        if let Some(document) = client.document(self.live)
            && document.state != jig_client::State::Ended
            && let Some(Latest::Jig(Delivery::Tasks(live))) = document.latest
        {
            count_rows(&live.rows, me, project, &mut asking);
        }
        if let Some(page) = &self.page {
            count_rows(&page.live, me, project, &mut asking);
            match &page.ended {
                EndedGroup::Rows { rows, .. } => {
                    count_rows(rows, me, project, &mut asking);
                }
                EndedGroup::None => {}
            }
        }
        asking
    }
    /// The person's newest Ask temper chat in the project, not closed
    /// (temper/docs/design/client/pages/README.md, Settings).
    #[must_use]
    pub fn open_asking(&self, client: &Client, me: Option<&Me>, project: u32) -> Option<u64> {
        let query = self.query?;
        if query.project != Some(project) {
            return None;
        }
        let me = me?;
        let mut newest = None;
        if let Some(document) = client.document(self.live)
            && document.state != jig_client::State::Ended
            && let Some(Latest::Jig(Delivery::Tasks(live))) = document.latest
        {
            newest_asking(&live.rows, me, project, &mut newest);
        }
        if let Some(page) = &self.page {
            newest_asking(&page.live, me, project, &mut newest);
        }
        newest
    }
    #[must_use]
    pub fn live<'a>(&self, client: &'a Client) -> Option<&'a c::Tasks> {
        if let Some(Latest::Jig(Delivery::Tasks(tasks))) = client.document(self.live)?.latest {
            Some(tasks)
        } else {
            None
        }
    }
}

fn handle(number: u64) -> Handle {
    Handle { asker: Owner::Pages.asker(), number }
}

/// A chat of template 5's charter (temper/docs/design/client/pages/README.md, To begin).
#[must_use]
pub fn asking(me: &Me, project: u32, executor: &c::ChipExecutor) -> bool {
    let Some(template) = jig_client::shapes::template(me, project, ASKING_TEMPLATE) else { return false };
    match template.executor {
        c::TemplateExecutor::Charter(charter) => *executor == c::ChipExecutor::Agent { charter },
        c::TemplateExecutor::Chosen => false,
    }
}

/// A design session: a chat of template 6's charter (temper/docs/design/client/pages/README.md).
#[must_use]
pub fn drafting(me: &Me, project: u32, executor: &c::ChipExecutor) -> bool {
    let Some(template) = jig_client::shapes::template(me, project, DRAFTING_TEMPLATE) else { return false };
    match template.executor {
        c::TemplateExecutor::Charter(charter) => *executor == c::ChipExecutor::Agent { charter },
        c::TemplateExecutor::Chosen => false,
    }
}

fn count_rows(rows: &List<c::TaskLine>, me: Option<&Me>, project: u32, found: &mut bool) {
    for row in rows {
        if row.project == project {
            let asking = match me {
                Some(me) => asking(me, project, &row.chip.executor),
                None => false,
            };
            *found |= asking;
        }
    }
}

fn show_chat(task: u64, from: &Address, out: &mut Queue<Request>) {
    let address = Address::Chat { task };
    out.push(if let Address::NewChat { .. } = from { Request::Replace { address } } else { Request::Show { address } });
}

fn newest_asking(rows: &List<c::TaskLine>, me: &Me, project: u32, newest: &mut Option<u64>) {
    for row in rows {
        if row.project == project
            && asking(me, project, &row.chip.executor)
            && row.ended_at.is_none()
            && !ended(&row.chip)
        {
            *newest = Some(match *newest {
                Some(task) => task.max(row.chip.task),
                None => row.chip.task,
            });
        }
    }
}

/// A chat whose phase or phase facts have ended (temper/docs/design/client/pages/README.md).
#[must_use]
pub fn ended(chip: &c::Chip) -> bool {
    use c::{ChipPhase, ChipPhaseFacts, Phase};
    if let Phase::Done | Phase::Ended | Phase::Failed | Phase::Cancelled = chip.phase {
        return true;
    }
    if let ChipPhaseFacts { phase: ChipPhase::Ended, .. } = chip.details.as_ref() {
        return true;
    }
    false
}

fn newer(a: (skein_lib::Wall, u64), b: (skein_lib::Wall, u64)) -> bool {
    a.0 > b.0 || (a.0 == b.0 && a.1 < b.1)
}

pub(crate) fn refresh(root: &mut crate::Domain, out: &mut Queue<Request>) {
    if let Some(chats) = &mut root.pages.chats {
        chats.goal_watch(&root.client, &root.forge, chats.documents.capacity(), out);
    }
}
