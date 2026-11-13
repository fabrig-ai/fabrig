//! One chat borrows its task and retains jig's bounded conversation projection
//! (temper/docs/design/client/pages/README.md and temper/docs/design/client/pages/chat/conversation.md, 2).
pub mod document;
mod request;
pub mod sheet;
mod talking;
mod views;
use super::{Act, Request, activity::Asking, delegation};
use crate::{Limits, owner::Owner};
use alloc::boxed::Box;
use document::{Document, Mode, NoteWrite};
use jig_client::{
    Client, Handle, Key, Latest, Readable, Readout, Told, Watchable, contract as c,
    thread::{
        Thread,
        window::{Edge, End},
    },
};
use skein_lib::{Env, List, Map, Queue, Time, Wall};
pub use talking::Talking;
pub use views::Shown;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sending {
    Message,
    Stop,
    Resume,
    Seen,
    Close,
}
/// Each card's read follows the conversation's waits (client/pages/README.md, client/pages/settings.md, 2).
#[derive(Debug)]
struct CardRead {
    handle: Handle,
    asking: Asking,
    departed: bool,
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "departures are private to the chat refresh and exposed as a slice")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "document reading flags and a confirmed restart have independent lifecycles"
)]
pub struct Chat {
    pub mode: Mode,
    talking: Option<Talking>,
    opening_mode: Option<Mode>,
    pub restarting: bool,
    pub decided: Option<(u64, Wall)>,
    pub departed_proposal: Option<u64>,
    decisions: List<(Handle, c::ProposalKey)>,
    pub draft: Option<Document>,
    pub writing: bool,
    write_up: Option<Key>,
    pub wanted: bool,
    pub shown: Option<u32>,
    pub shown_count: u64,
    draft_read: Handle,
    goal_read: Handle,
    request: request::Reader,
    goal_watched: Option<u64>,
    pub proposed: Option<c::Recipient>,
    draft_asking: Asking,
    draft_project: Option<u32>,
    draft_out: bool,
    notes_seen: (u64, u32, u64),
    pub task: u64,
    pub document: Handle,
    pub activity: Handle,
    /// The latest activity read refusal, retained only for diagnostics.
    pub last_read_none: Option<c::ReadNone>,
    pub run: Option<Handle>,
    asking: Asking,
    current_run: Option<Box<c::RunState>>,
    pub next: u64,
    pub thread: Thread,
    to_end: u64,
    initial_pages: u32,
    pub words: Box<[u8]>,
    pub beyond: u64,
    pub composer_focus: u64,
    /// The proposal "Change something" was pressed on, until something is sent or it is decided (client/pages/settings.md, 2).
    pub change_asked: Option<c::ProposalKey>,
    pub pending: List<(Key, Sending)>,
    pub outcome: Option<jig_client::Outcome>,
    pub outcome_kind: Option<Sending>,
    /// Newest reply number marked seen from the retained window.
    pub seen: u64,
    seen_before: u64,
    pub cards: List<super::inbox::Card>,
    card_reads: List<CardRead>,
    views: views::Views,
    pub delegations: List<delegation::Delegation>,
    departures: List<c::ObjectKey>,
    pub closing: Option<c::Ask>,
    pub final_task: Option<Box<c::Task>>,
}
impl Chat {
    /// The first message's bytes and whether they are whole (client/view.md, 4).
    #[must_use]
    pub fn request<'a>(&'a self, client: &'a Client) -> Option<(&'a [u8], bool)> {
        if self.talking.is_some() {
            return None;
        }
        let words = &self.task(client)?.spec.specification.words;
        Some(self.request.words(words))
    }

    /// A standing role's talk (temper/docs/design/client/pages/talk.md, 2).
    #[must_use]
    pub fn talk(&self) -> Option<&Talking> {
        self.talking.as_ref()
    }

    /// The log's end request (temper/docs/design/client/pages/chat/conversation.md, 2).
    #[must_use]
    pub fn to_end(&self) -> u64 {
        self.to_end
    }

    /// The supplied call under way (temper/docs/design/client/pages/README.md).
    #[must_use]
    pub fn current_call(&self) -> Option<&c::CurrentCall> {
        self.current_run.as_ref()?.call.as_ref()
    }

    pub(crate) fn new(task: u64, mode: Mode, first: u64, limits: &Limits, out: &mut Queue<Request>) -> Self {
        let document = handle(first);
        let activity = handle(first.checked_add(1).expect("chat handles fit"));
        let mut thread = Thread::new(task, &thread_limits(limits));
        thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
        out.push(Request::Client(jig_client::Event::Watch {
            handle: document,
            subject: Watchable::Jig(c::Subject::Task { task }),
        }));
        let mut asking = Asking::new();
        asking.ask(activity, c::Read::Activity { task, before: None }, out);
        Self {
            mode,
            talking: None,
            opening_mode: Some(mode),
            restarting: false,
            decided: None,
            departed_proposal: None,
            decisions: List::with_capacity(limits.client.domain.reads),
            draft: None,
            writing: mode != Mode::Chat,
            write_up: None,
            wanted: false,
            shown: None,
            shown_count: 0,
            draft_read: handle(first.checked_add(2).expect("chat handles fit")),
            goal_read: handle(first.checked_add(3).expect("chat handles fit")),
            request: request::Reader::new(limits.client.domain.long_text),
            goal_watched: None,
            proposed: None,
            draft_asking: Asking::new(),
            draft_project: None,
            draft_out: false,
            notes_seen: (0, 0, 0),
            task,
            document,
            activity,
            last_read_none: None,
            asking,
            run: None,
            current_run: None,
            next: first.checked_add(4).expect("chat handles fit"),
            initial_pages: thread.limits.lines,
            thread,
            to_end: 0,
            words: Box::default(),
            beyond: 0,
            composer_focus: 0,
            change_asked: None,
            pending: List::with_capacity(5),
            outcome: None,
            outcome_kind: None,
            seen: 0,
            seen_before: 0,
            cards: List::with_capacity(limits.client.domain.reads),
            card_reads: List::with_capacity(limits.client.domain.reads),
            views: views::Views::new(limits.forge),
            delegations: List::with_capacity(delegation::bound(limits)),
            departures: List::with_capacity(limits.client.domain.reads),
            closing: None,
            final_task: None,
        }
    }
    pub(crate) fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        self.asking.fire(now, self.activity, out);
        self.draft_asking.fire(now, self.draft_read, out);
        self.request.fire(now, out);
        self.views.fire(now, out);
        for read in self.card_reads.iter_mut() {
            read.asking.fire(now, read.handle, out);
        }
    }
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let mut at = super::activity::earliest(self.asking.next_deadline(), self.draft_asking.next_deadline());
        at = super::activity::earliest(at, self.request.next_deadline());
        at = super::activity::earliest(at, self.views.next_deadline());
        for read in &self.card_reads {
            at = super::activity::earliest(at, read.asking.next_deadline());
        }
        at
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [Some(self.document), Some(self.activity), Some(self.draft_read), self.run].into_iter().flatten()
        {
            unwatch(handle, out);
        }
        if self.goal_watched.is_some() {
            unwatch(self.goal_read, out);
        }
        if let Some(talk) = &self.talking {
            talk.leave(out);
        }
        self.request.leave(out);
        self.views.leave(out);
        for (handle, _) in &self.decisions {
            unwatch(*handle, out);
        }
        for card in &self.cards {
            unwatch(card.handle, out);
        }
        for delegation in &self.delegations {
            unwatch(delegation.handle, out);
        }
        self.next
    }
    /// The newest of this person's starts, with the goal number breaking a clock tie.
    #[must_use]
    pub fn goal_from(&self, root: &crate::Domain) -> Option<(u64, u64)> {
        self.goal_fact(root.client(), root.forge())
    }
    fn goal_fact(&self, client: &Client, forge: &crate::forge::Side) -> Option<(u64, u64)> {
        let project = self.task(client)?.project;
        let mut newest = None;
        for (goal, at) in forge.goals_from(project, self.task) {
            let newer = match newest {
                Some((prior_goal, prior_at)) => (at, goal) > (prior_at, prior_goal),
                None => true,
            };
            if newer {
                newest = Some((goal, at));
            }
        }
        newest
    }
    pub(crate) fn goal_watch(&mut self, client: &Client, forge: &crate::forge::Side, out: &mut Queue<Request>) {
        // The page refresh runs after every event, including a later arrival of me.
        self.opening_mode(client);
        let next = match self.goal_fact(client, forge) {
            Some((goal, _)) => Some(goal),
            None => None,
        };
        if self.goal_watched == next {
            return;
        }
        if self.goal_watched.is_some() {
            unwatch(self.goal_read, out);
        }
        self.goal_watched = next;
        if let Some(goal) = next {
            out.push(Request::Client(jig_client::Event::Watch {
                handle: self.goal_read,
                subject: Watchable::Jig(c::Subject::Chips { tasks: List::from_boxed(Box::new([goal])) }),
            }));
        }
    }
    fn opening_mode(&mut self, client: &Client) {
        let Some(opening) = self.opening_mode else { return };
        if self.mode != opening {
            self.opening_mode = None;
            return;
        }
        let Some(task) = self.task(client) else { return };
        let Some(Latest::Jig(c::Delivery::Me(me))) = latest(client, super::me_handle()) else { return };
        let drafting = super::chats::drafting(me, task.project, &task.chip.executor);
        self.opening_mode = None;
        if drafting && opening == Mode::Chat {
            self.mode = Mode::Split;
        }
    }
    #[must_use]
    pub fn goal_chip<'a>(&self, root: &'a crate::Domain) -> Option<&'a c::Chip> {
        let (goal, _) = self.goal_from(root)?;
        let document = root.client().document(self.goal_read)?;
        if document.state == jig_client::State::Ended {
            return None;
        }
        if let Some(Latest::Jig(c::Delivery::Chips(chips))) = document.latest {
            for chip in chips {
                if chip.task == goal {
                    return Some(chip);
                }
            }
        }
        None
    }
    #[must_use]
    pub fn task<'a>(&'a self, client: &'a Client) -> Option<&'a c::Task> {
        if let Some(task) = &self.final_task {
            return Some(task);
        }
        match client.document(self.document)?.latest? {
            Latest::Jig(c::Delivery::Task(task)) => Some(task),
            Latest::Jig(_) | Latest::Family(_) => None,
        }
    }
    #[expect(clippy::too_many_lines, reason = "one dispatch routes the chat's independent read handles")]
    pub(crate) fn told(&mut self, now: Time, client: &Client, told: Told, out: &mut Queue<Request>) -> bool {
        if let Some(talk) = &mut self.talking
            && talk.told(&told, out)
        {
            self.refresh_talk(client, out);
            return true;
        }
        if let Told::Page { handle, page } = &told
            && self.views.page(now, client, *handle, page)
        {
            return true;
        }
        match told {
            Told::Changed { handle } if handle == crate::forge::ways::handle() => {}
            Told::Changed { handle } if handle == self.document => {
                if let Some(Latest::Jig(c::Delivery::Task(task))) = latest(client, handle) {
                    ask_activity(&mut self.asking, self.activity, self.thread.document(task), out);
                    if self.talking.is_none() {
                        self.request.start(task, &mut self.next, out);
                    }
                    self.watch_run(task, out);
                    self.proposals(task, out);
                    self.open_draft(task.project, out);
                }
            }
            Told::Ended { handle, why: jig_client::Why::SubjectEnded | jig_client::Why::Unknown }
                if handle == self.document =>
            {
                read(handle, Some(c::Read::Task { task: self.task }), out);
            }
            Told::Page { handle, page } if handle == self.document => {
                if let Some(task) = page_task(page, self.task) {
                    ask_activity(&mut self.asking, self.activity, self.thread.document(&task), out);
                    if self.talking.is_none() {
                        self.request.start(&task, &mut self.next, out);
                    }
                    self.watch_run(&task, out);
                    self.proposals(&task, out);
                    self.open_draft(task.project, out);
                    self.final_task = Some(task);
                }
            }
            Told::Changed { handle } | Told::Text { handle } if Some(handle) == self.run => {
                self.run_changed(client, handle, out);
            }
            Told::Page { handle, page } if Some(handle) == self.request.handle => {
                self.request.page(now, client, self.task, page, out);
            }
            Told::Page { handle, page } if handle == self.draft_read => {
                self.draft_page(now, client, page, out);
            }
            Told::Page { handle, page } if handle == self.activity => {
                self.activity_page(now, client, page, out);
            }
            Told::Answered { key, outcome } if self.pending_key(key) => self.answered(key, outcome),
            Told::Pending { key } if self.pending_key(key) => {}
            Told::Page { handle, page } if self.decision_handle(handle).is_some() => {
                self.decision_page(handle, page);
            }
            Told::Page { handle, page } if self.card_handle(handle) => {
                let transient = super::activity::read_end(client, &page).transient();
                for read in self.card_reads.iter_mut() {
                    if read.handle == handle {
                        if transient {
                            read.asking.ended(now);
                        } else {
                            read.asking.answered();
                        }
                        break;
                    }
                }
                for index in 0..self.cards.len() {
                    let card = self.cards.get_mut(index).expect("bounded card");
                    if card.handle == handle && !transient {
                        if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(ended))) = &page
                            && ended.matches(card.object)
                            && let c::EndedValue::Proposal(history) = &ended.value
                        {
                            self.departed_proposal = Some(self.departed_proposal.unwrap_or(0).max(history.number));
                            if let c::ProposalHistoryState::Accepted { .. } = history.state
                                && let Some(at) = ended.at
                                && newer_decision(self.decided, history.number)
                            {
                                self.decided = Some((history.number, at));
                            }
                        }
                        card.page = match page {
                            Readout::Jig(c::ReadResult::Value(value)) => Some(value),
                            Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => {
                                None
                            }
                        };
                        break;
                    }
                }
            }
            Told::Changed { handle } | Told::Ended { handle, .. } if handle == self.goal_read => {}
            Told::Changed { handle } if self.delegation_handle(handle).is_some() => {}
            Told::Ended { handle, why: jig_client::Why::SubjectEnded | jig_client::Why::Unknown }
                if self.delegation_handle(handle).is_some() =>
            {
                let task = self.delegation_handle(handle).expect("delegation handle");
                read(handle, Some(c::Read::Task { task }), out);
            }
            Told::Page { handle, page } if self.delegation_handle(handle).is_some() => {
                for index in 0..self.delegations.len() {
                    let delegation = self.delegations.get_mut(index).expect("bounded delegation");
                    if delegation.handle == handle {
                        if let Some(task) = page_task(page, delegation.task) {
                            delegation.final_task = Some(task);
                        }
                        break;
                    }
                }
            }
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => return false,
        }
        if self.run.is_none() && self.thread.loaded >= self.thread.committed {
            self.thread.end_run();
        }
        self.note_calls(out);
        self.made_cards(client, out);
        self.delegations(client, out);
        self.sync_views(client, out);
        self.refresh_talk(client, out);
        true
    }
    fn open_draft(&mut self, project: u32, out: &mut Queue<Request>) {
        if self.talking.is_some() {
            return;
        }
        if self.draft_project.is_none() {
            self.draft_project = Some(project);
            self.ask_draft(out);
        }
    }
    fn ask_draft(&mut self, out: &mut Queue<Request>) {
        let Some(project) = self.draft_project else { return };
        if self.draft_out {
            self.wanted = true;
            return;
        }
        self.draft_out = true;
        self.draft_asking.ask(
            self.draft_read,
            c::Read::Notes { scope: c::NoteScope::Task { project, task: self.task }, description: None, after: None },
            out,
        );
    }
    fn draft_page(&mut self, now: Time, client: &Client, page: Readout, out: &mut Queue<Request>) {
        if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page))) = page {
            let scope = c::NoteScope::Task { project: self.draft_project.expect("task arrived"), task: self.task };
            let mut lowest: Option<&c::Note> = None;
            let mut working = false;
            for note in &page.rows {
                if note.scope == scope && note.description.bytes().starts_with(b"work:") {
                    working = true;
                    continue;
                }
                if note.scope == scope
                    && match lowest {
                        Some(old) => note.number < old.number,
                        None => true,
                    }
                {
                    lowest = Some(note);
                }
            }
            if let Some(note) = lowest
                && let Ok(revision) = u32::try_from(note.revision.value)
            {
                self.draft = Some(Document {
                    note: note.number,
                    revision,
                    title: Box::from(note.description.bytes()),
                    body: Box::from(note.body.bytes()),
                    changed: note.changed,
                });
            } else if working {
                self.draft = None;
            }
            if lowest.is_none()
                && let Some(after) = &page.after
            {
                self.draft_asking.answered();
                self.draft_asking.ask(
                    self.draft_read,
                    c::Read::Notes { scope, description: None, after: Some(after.clone()) },
                    out,
                );
                return;
            }
        } else if super::activity::read_end(client, &page).transient() {
            self.draft_asking.ended(now);
            return;
        }
        self.draft_asking.answered();
        self.draft_out = false;
        if self.wanted {
            self.wanted = false;
            self.ask_draft(out);
        }
    }
    fn note_calls(&mut self, out: &mut Queue<Request>) {
        let mut newest = self.notes_seen;
        for line in &self.thread.lines() {
            if let jig_client::thread::Line::Turn { name, turn, .. } = line {
                for message in &turn.messages {
                    for block in &message.blocks {
                        if let c::Block::Call(call) = block
                            && document::writes(call.tool.bytes())
                        {
                            newest = newest.max((name.attempt, name.number, call.number));
                        }
                    }
                }
            }
        }
        if newest > self.notes_seen && self.draft_project.is_some() {
            self.notes_seen = newest;
            self.ask_draft(out);
        }
    }
    /// The newest successful note write in the retained conversation.
    #[must_use]
    pub fn newest_write(&self) -> Option<NoteWrite> {
        for line in self.thread.lines().iter().rev() {
            if let jig_client::thread::Line::Turn { turn, .. } = line {
                for message in turn.messages.iter().rev() {
                    for block in message.blocks.iter().rev() {
                        if let c::Block::Call(call) = block
                            && document::writes(call.tool.bytes())
                            && !call.failed
                            && let Some(outcome) = &call.outcome
                            && let Some(write) = document::tool_write(call.tool.bytes(), outcome.bytes())
                        {
                            return Some(write);
                        }
                    }
                }
            }
        }
        None
    }
    fn activity_page(&mut self, now: Time, client: &Client, page: Readout, out: &mut Queue<Request>) {
        if let Readout::Jig(c::ReadResult::None(why)) = &page {
            self.last_read_none = Some(*why);
        }
        let why = super::activity::read_end(client, &page);
        if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page))) = page {
            self.asking.answered();
            let read = self.accept_activity(*page);
            ask_activity(&mut self.asking, self.activity, read, out);
        } else if let Readout::Jig(c::ReadResult::None(c::ReadNone::SignedOut)) = page
            && client.link().state == jig_client::Link::Starting
            && super::awaits_first_read(&self.thread)
        {
            ask_activity(
                &mut self.asking,
                self.activity,
                Some(c::Read::Activity { task: self.task, before: None }),
                out,
            );
        } else if why.transient() {
            self.asking.ended(now);
        } else {
            self.asking.answered();
            self.initial_pages = 0;
            self.thread.failed();
        }
    }
    fn accept_activity(&mut self, page: c::ActivityPage) -> Option<c::Read> {
        let previous = self.thread.lines.edges.after.clone();
        let empty = page.rows.is_empty();
        let filling = self.initial_pages > 0;
        if self.initial_pages > 0 {
            if let Some(jig_client::thread::Reading::Older) = self.thread.reading {
                let mut weight = 0_u64;
                for row in self.thread.lines.rows.iter().chain(page.rows.iter()) {
                    weight = weight.saturating_add(row.weight());
                }
                if weight > self.thread.limits.bytes
                    || self.thread.lines.rows.len().saturating_add(page.rows.len()) > self.thread.limits.lines
                {
                    self.initial_pages = 0;
                    self.thread.reading = None;
                    return self.thread.moved(self.thread.committed);
                }
            }
            self.initial_pages = self.initial_pages.saturating_sub(1);
        }
        if let Some(read) = self.thread.page(page) {
            // An empty initial page advances by cursor rather than rereading the tail.
            if filling && empty && self.thread.loaded == 0 && !self.thread.lines.edges.reaches(End::After) {
                self.thread.reading = None;
            } else {
                return Some(read);
            }
        }
        let mut weight = 0_u64;
        for row in &self.thread.lines.rows {
            weight = weight.saturating_add(row.weight());
        }
        let more = match &self.thread.lines.edges.after {
            Edge::More(_) => true,
            Edge::End | Edge::Dropped => false,
        };
        if more
            && previous != self.thread.lines.edges.after
            && ((self.initial_pages > 0 && weight < OPENING_WEIGHT) || (!filling && empty))
            && self.thread.lines.edges.reaches(End::Before)
            && self.thread.lines.rows.len() < self.thread.limits.lines
            && weight < self.thread.limits.bytes
        {
            return self.thread.older();
        }
        // An empty chat keeps its remaining initial budget for its first committed history.
        if !empty || self.thread.loaded > 0 {
            self.initial_pages = 0;
        }
        None
    }
    fn run_changed(&mut self, client: &Client, handle: Handle, out: &mut Queue<Request>) {
        match latest(client, handle) {
            Some(Latest::Jig(c::Delivery::RunState(run))) => {
                if match &self.current_run {
                    Some(run) => run_notes(run),
                    None => false,
                } && !run_notes(run)
                {
                    self.ask_draft(out);
                }
                self.thread.run(run, client.text(handle).or(run.writing.as_ref()));
                self.current_run = Some(Box::new(c::RunState {
                    task: run.task,
                    attempt: run.attempt,
                    committed_turn: run.committed_turn,
                    writing: None,
                    call: run.call.clone(),
                }));
            }

            Some(Latest::Jig(c::Delivery::Run(text))) => {
                let fallback = c::RunState {
                    task: self.task,
                    attempt: Some(text.attempt),
                    committed_turn: 0,
                    writing: None,
                    call: None,
                };
                self.thread.run(self.current_run.as_deref().unwrap_or(&fallback), client.text(handle));
            }
            _ => {}
        }
    }
    fn answered(&mut self, key: Key, outcome: jig_client::Outcome) {
        if self.pending.as_slice().contains(&(key, Sending::Seen)) {
            let mut kept = List::with_capacity(self.pending.capacity());
            for row in &self.pending {
                if row.0 != key {
                    kept.push(*row).expect("remaining pending fits");
                }
            }
            self.pending = kept;
            if !super::seen_answered(&outcome) {
                self.seen = self.seen_before;
            }
            return;
        }
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)) = &outcome {
            self.thread.answered(key, answer);
        } else {
            self.thread.words.refused(key);
        }
        self.outcome_kind = None;
        let mut kept = List::with_capacity(self.pending.capacity());
        for row in &self.pending {
            if row.0 == key {
                self.outcome_kind = Some(row.1);
            } else {
                kept.push(*row).expect("remaining pending fits");
            }
        }
        self.pending = kept;
        if self.write_up == Some(key) {
            self.write_up = None;
            if !super::seen_answered(&outcome) {
                self.writing = false;
            }
        }
        self.outcome = Some(outcome);
    }
    fn request_end(&mut self, out: &mut Queue<Request>) {
        self.to_end = self.to_end.checked_add(1).expect("end request count fits");
        if !self.thread.lines.edges.reaches(End::Before) {
            ask_activity(&mut self.asking, self.activity, self.thread.latest(), out);
        }
    }
    #[must_use]
    pub fn delegation<'a>(&'a self, client: &'a Client, task: u64) -> Option<&'a c::Task> {
        for delegation in &self.delegations {
            if delegation.task == task {
                if let Some(task) = &delegation.final_task {
                    return Some(task);
                }
                return match latest(client, delegation.handle)? {
                    Latest::Jig(c::Delivery::Task(task)) => Some(task),
                    Latest::Jig(_) | Latest::Family(_) => None,
                };
            }
        }
        None
    }
    /// A delegation's task document, once it says this chat requested it: a result's words name
    /// a task by a number that may be a person's.
    #[must_use]
    pub fn requested<'a>(&'a self, client: &'a Client, task: u64) -> Option<&'a c::Task> {
        let document = self.delegation(client, task)?;
        if document.requester == c::Requester::Task(self.task) { Some(document) } else { None }
    }
    /// Reconcile independently delivered chat and card documents (client/pages/README.md).
    #[must_use]
    pub fn waiting_delegates(&self, client: &Client) -> u32 {
        let Some(task) = self.task(client) else { return 0 };
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_ref();
        let mut count = details.live_delegates;
        for delegation in &self.delegations {
            let Some(document) = self.requested(client, delegation.task) else { continue };
            let mut live = false;
            for delegate in &task.delegates {
                if delegate.chip.task == delegation.task {
                    live = true;
                    break;
                }
            }
            let running = match delegation::facts(document).state {
                delegation::State::Running | delegation::State::Held => true,
                delegation::State::Landed
                | delegation::State::Cleared
                | delegation::State::Uncleared
                | delegation::State::Done
                | delegation::State::Failed
                | delegation::State::Cancelled
                | delegation::State::Ended => false,
            };
            match (live, running) {
                (true, false) => count = count.saturating_sub(1),
                (false, true) => count = count.saturating_add(1),
                (true, true) | (false, false) => {}
            }
        }
        count
    }
    fn heard_result(&self, task: u64) -> bool {
        for line in &self.thread.lines.rows {
            if delegation::heard(&line.source.line) == Some(task) {
                return true;
            }
        }
        false
    }
    fn delegation_handle(&self, handle: Handle) -> Option<u64> {
        for delegation in &self.delegations {
            if delegation.handle == handle {
                return Some(delegation.task);
            }
        }
        None
    }
    fn delegations(&mut self, client: &Client, out: &mut Queue<Request>) {
        let bound = self.delegations.capacity();
        let mut live = List::with_capacity(bound);
        if let Some(task) = self.task(client) {
            for delegate in &task.delegates {
                if live.push(delegate.chip.task).is_err() {
                    break;
                }
            }
        }
        let mut tasks = List::with_capacity(bound);
        let mut called = List::with_capacity(bound);
        // Thread's borrowed lines are chronological, so visit the anchors backwards.
        'turns: for line in self.thread.lines().iter().rev() {
            let turn = match line {
                jig_client::thread::Line::Turn { turn, .. } => turn,
                // A heard result names its delegate as a result block does, before
                // the chat's resumed turn carries one (client/view.md, 5).
                jig_client::thread::Line::Activity { line } => {
                    if let Some(task) = delegation::heard(&line.source.line)
                        && !tasks.as_slice().contains(&task)
                        && tasks.push(task).is_err()
                    {
                        break 'turns;
                    }
                    continue;
                }
                jig_client::thread::Line::Words { .. }
                | jig_client::thread::Line::Undecoded { .. }
                | jig_client::thread::Line::Provisional { .. } => continue,
            };
            for message in turn.messages.iter().rev() {
                for block in message.blocks.iter().rev() {
                    called.clear();
                    match block {
                        c::Block::Call(call) => delegation::delegated(call, &mut called),
                        c::Block::Result { summary, .. } => delegation::results(summary.bytes(), &mut called),
                        c::Block::Text(_) | c::Block::Opaque { .. } => continue,
                    }
                    for task in &called {
                        if !tasks.as_slice().contains(task) && tasks.push(*task).is_err() {
                            break 'turns;
                        }
                    }
                }
            }
        }
        // The live delegates anchor follows the window's anchors (client/view.md, 5).
        for task in &live {
            if !tasks.as_slice().contains(task) && tasks.push(*task).is_err() {
                break;
            }
        }
        for delegation in &self.delegations {
            if !tasks.as_slice().contains(&delegation.task) {
                unwatch(delegation.handle, out);
            }
        }
        let mut ordered = List::with_capacity(bound);
        for task in &tasks {
            let mut kept = None;
            for index in 0..self.delegations.len() {
                let delegation = self.delegations.get_mut(index).expect("bounded delegation");
                if delegation.task == *task {
                    kept = Some(delegation::Delegation {
                        task: *task,
                        handle: delegation.handle,
                        final_task: delegation.final_task.take(),
                        result_read: delegation.result_read,
                    });
                    break;
                }
            }
            let mut delegation = if let Some(kept) = kept {
                kept
            } else {
                let handle = handle(self.next);
                self.next = self.next.checked_add(1).expect("chat handles fit");
                out.push(Request::Client(jig_client::Event::Watch {
                    handle,
                    subject: Watchable::Jig(c::Subject::Task { task: *task }),
                }));
                delegation::Delegation { task: *task, handle, final_task: None, result_read: false }
            };
            // Hearing the result proves the delegate ended; its watch may still
            // hold an earlier document. Read its durable result once (5.7).
            if !delegation.result_read
                && delegation.final_task.is_none()
                && self.delegation(client, *task).is_some()
                && self.heard_result(*task)
            {
                read(delegation.handle, Some(c::Read::Task { task: *task }), out);
                delegation.result_read = true;
            }
            ordered.push(delegation).expect("bounded delegations");
        }
        self.delegations = ordered;
    }
    fn decision_handle(&self, handle: Handle) -> Option<u32> {
        for index in 0..self.decisions.len() {
            if self.decisions.get(index)?.0 == handle {
                return Some(index);
            }
        }
        None
    }
    fn decision_page(&mut self, handle: Handle, page: Readout) {
        let index = self.decision_handle(handle).expect("departed proposal read");
        let (_, key) = self.decisions.swap_remove(index).expect("departed proposal");
        if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(ended))) = page
            && ended.matches(c::ObjectKey::Proposal(key))
            && let c::EndedValue::Proposal(history) = ended.value
            && let c::ProposalHistoryState::Accepted { .. } = history.state
            && let Some(at) = ended.at
            && newer_decision(self.decided, key.number)
        {
            self.decided = Some((key.number, at));
        }
    }
    fn card_handle(&self, handle: Handle) -> bool {
        self.card_for_handle(handle).is_some()
    }
    fn card_for_handle(&self, handle: Handle) -> Option<&super::inbox::Card> {
        for index in 0..self.cards.len() {
            let card = self.cards.get(index)?;
            if card.handle == handle {
                return Some(card);
            }
        }
        None
    }
    fn card(&self, object: c::ObjectKey) -> Option<&super::inbox::Card> {
        for index in 0..self.cards.len() {
            let card = self.cards.get(index)?;
            if card.object == object {
                return Some(card);
            }
        }
        None
    }
    /// Proposals the last task document removed from this chat.
    #[must_use]
    pub fn departures(&self) -> &[c::ObjectKey] {
        self.departures.as_slice()
    }
    fn sending(&self, kind: Sending) -> bool {
        for (_, pending) in &self.pending {
            if *pending == kind {
                return true;
            }
        }
        false
    }
    fn proposals(&mut self, task: &c::Task, out: &mut Queue<Request>) {
        if let Some(proposal) = self.change_asked
            && !task.proposals.as_slice().contains(&proposal)
        {
            self.change_asked = None;
        }
        self.departures.clear();
        for card in &self.cards {
            let c::ObjectKey::Proposal(key) = card.object else { continue };
            if !task.proposals.as_slice().contains(&key) {
                if self.keeps_card(card) {
                    if let Some(c::ReadValue::Proposal(_)) = &card.page {
                        for read in self.card_reads.iter_mut() {
                            if read.handle == card.handle && !read.departed {
                                read.departed = true;
                                read.asking.ask(card.handle, c::Read::Proposal { proposal: key }, out);
                                break;
                            }
                        }
                    }
                    continue;
                }
                unwatch(card.handle, out);
                for index in 0..self.card_reads.len() {
                    if self.card_reads.get(index).expect("bounded card read").handle == card.handle {
                        self.card_reads.swap_remove(index).expect("bounded card read");
                        break;
                    }
                }
                self.departures.push(card.object).expect("removed cards fit their window");
                self.departed_proposal = Some(self.departed_proposal.unwrap_or(0).max(key.number));
                let handle = handle(self.next);
                self.next = self.next.checked_add(1).expect("chat handles fit");
                if self.decisions.push((handle, key)).is_ok() {
                    read(handle, Some(c::Read::Proposal { proposal: key }), out);
                }
            }
        }
        let mut ordered = Map::with_capacity(self.cards.capacity());
        for key in &task.proposals {
            let object = c::ObjectKey::Proposal(*key);
            if ordered.len() == ordered.capacity() {
                break;
            }
            if let Some(card) = self.card(object) {
                ordered.insert((key.number, card.handle.number), card.clone()).expect("retained cards fit");
                continue;
            }
            let handle = handle(self.next);
            self.next = self.next.checked_add(1).expect("chat handles fit");
            ordered
                .insert(
                    (key.number, handle.number),
                    super::inbox::Card {
                        whole: None,
                        object,
                        group: c::InboxGroup::Proposals,
                        project: Some(task.project),
                        position: None,
                        unread: None,
                        handle,
                        page: None,
                        words: None,
                        key: None,
                        outcome: None,
                    },
                )
                .expect("bounded cards");
        }
        for card in &self.cards {
            let c::ObjectKey::Proposal(key) = card.object else { continue };
            if !task.proposals.as_slice().contains(&key) && self.keeps_card(card) && ordered.len() < ordered.capacity()
            {
                ordered.insert((key.number, card.handle.number), card.clone()).expect("retained cards fit");
            }
        }
        self.cards.clear();
        for (_, card) in &ordered {
            self.cards.push(card.clone()).expect("ordered cards fit");
        }
        self.sync_card_reads(out);
    }
    // Retained answers survive task documents; an unanswered making line keeps its read too (5.13).
    fn keeps_card(&self, card: &super::inbox::Card) -> bool {
        if let Some(c::ReadValue::Ended(_)) = &card.page {
            return true;
        }
        if let Some(c::ReadValue::Proposal(proposal)) = &card.page
            && let c::ProposalAction::Settings(_) = &proposal.action
        {
            return true;
        }
        card.page.is_none() && self.making_line(card.object)
    }
    fn making_line(&self, object: c::ObjectKey) -> bool {
        for line in self.thread.lines().into_boxed() {
            if let jig_client::thread::Line::Activity { line } = line
                && let c::Activity::Decided { proposal: Some(c::DecidedProposal { number, made: true }), .. } =
                    line.source.line
                && object == c::ObjectKey::Proposal(c::ProposalKey { proposer: c::Proposer::Task(self.task), number })
            {
                return true;
            }
        }
        false
    }
    fn evictable(&self, card: &super::inbox::Card, client: &Client) -> bool {
        if self.making_line(card.object) {
            return false;
        }
        if let Some(c::ReadValue::Ended(_)) = &card.page {
            return true;
        }
        if card.page.is_some() {
            return false;
        }
        let c::ObjectKey::Proposal(key) = card.object else { return false };
        if let Some(task) = self.task(client)
            && task.proposals.as_slice().contains(&key)
        {
            return false;
        }
        true
    }
    /// Making lines bring back retained proposals independently of the live list (client/pages/settings.md, 2).
    fn made_cards(&mut self, client: &Client, out: &mut Queue<Request>) {
        let mut keys = List::with_capacity(self.thread.limits.lines);
        for line in self.thread.lines().into_boxed() {
            if let jig_client::thread::Line::Activity { line } = line
                && let c::Activity::Decided { proposal: Some(c::DecidedProposal { number, made: true }), .. } =
                    line.source.line
            {
                keys.push(c::ProposalKey { proposer: c::Proposer::Task(self.task), number })
                    .expect("making lines fit the window");
            }
        }
        let mut added = false;
        for key in &keys {
            let object = c::ObjectKey::Proposal(*key);
            if self.card(object).is_some() {
                continue;
            }
            if self.cards.room() == 0 {
                for index in 0..self.cards.len() {
                    let card = self.cards.get(index).expect("bounded card");
                    if self.evictable(card, client) {
                        self.cards.swap_remove(index).expect("bounded card");
                        break;
                    }
                }
            }
            if self.cards.room() == 0 {
                continue;
            }
            let handle = handle(self.next);
            self.next = self.next.checked_add(1).expect("chat handles fit");
            let project = match self.task(client) {
                Some(task) => Some(task.project),
                None => None,
            };
            self.cards
                .push(super::inbox::Card {
                    whole: None,
                    object,
                    group: c::InboxGroup::Proposals,
                    project,
                    position: None,
                    unread: None,
                    handle,
                    page: None,
                    words: None,
                    key: None,
                    outcome: None,
                })
                .expect("room for the made card");
            added = true;
        }
        if !added {
            return;
        }
        let mut ordered = Map::with_capacity(self.cards.capacity());
        for card in &self.cards {
            let c::ObjectKey::Proposal(key) = card.object else { continue };
            ordered.insert((key.number, card.handle.number), card.clone()).expect("ordered cards fit");
        }
        self.cards.clear();
        for (_, card) in &ordered {
            self.cards.push(card.clone()).expect("ordered cards fit");
        }
        self.sync_card_reads(out);
    }
    fn sync_card_reads(&mut self, out: &mut Queue<Request>) {
        for index in (0..self.card_reads.len()).rev() {
            let handle = self.card_reads.get(index).expect("bounded card read").handle;
            if !self.card_handle(handle) {
                unwatch(handle, out);
                self.card_reads.swap_remove(index).expect("bounded card read");
            }
        }
        for card in &self.cards {
            let mut known = false;
            for read in &self.card_reads {
                if read.handle == card.handle {
                    known = true;
                    break;
                }
            }
            if !known {
                let c::ObjectKey::Proposal(proposal) = card.object else { continue };
                let mut asking = Asking::new();
                asking.ask(card.handle, c::Read::Proposal { proposal }, out);
                self.card_reads
                    .push(CardRead { handle: card.handle, asking, departed: false })
                    .expect("one read per card");
            }
        }
    }

    pub(crate) fn prepare_close(&mut self, client: &Client) {
        self.closing = None;
        let Some(task) = self.task(client) else { return };
        if !allowed(task, c::ActKind::Cancel) || self.sending(Sending::Close) {
            return;
        }
        let Some(Latest::Jig(c::Delivery::Me(me))) = latest(client, super::me_handle()) else { return };
        let mut delegates = List::with_capacity(task.delegates.len());
        for delegate in &task.delegates {
            delegates
                .push(jig_client::shapes::Delegate { task: delegate.chip.task, fate: jig_client::shapes::Fate::Cancel })
                .expect("delegates fit");
        }
        let reason = c::Text::new(Box::from(*b"Chat closed"), u32::MAX).expect("bounded close reason");
        self.closing = jig_client::shapes::close_chat(me, task.project, self.task, delegates.as_slice(), reason).ok();
    }
    pub(crate) fn delegate_fate(&mut self, task: u64, fate: jig_client::shapes::Fate) {
        if let Some(c::Ask::Compound { project, operations }) = &mut self.closing {
            for index in 0..operations.len() {
                let operation = operations.get_mut(index).expect("bounded operation");
                match operation {
                    c::Operation::Cancel { task: number, reason, .. }
                    | c::Operation::Move { task: number, reason, .. }
                        if *number == task && task != self.task =>
                    {
                        *operation = match fate {
                            jig_client::shapes::Fate::Cancel => {
                                c::Operation::Cancel { project: *project, task, reason: reason.clone(), grace: None }
                            }
                            jig_client::shapes::Fate::Move => {
                                c::Operation::Move { project: *project, task, reason: reason.clone() }
                            }
                        };
                        break;
                    }
                    c::Operation::Message { .. }
                    | c::Operation::Amend { .. }
                    | c::Operation::Cancel { .. }
                    | c::Operation::Release { .. }
                    | c::Operation::Stop { .. }
                    | c::Operation::Move { .. } => {}
                }
            }
        }
    }
    pub(crate) fn close(&mut self, serial: &mut u64, out: &mut Queue<Request>) -> Option<Key> {
        let ask = self.closing.clone()?;
        if self.sending(Sending::Close) {
            return None;
        }
        *serial = serial.checked_add(1).expect("chat keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending.push((key, Sending::Close)).expect("one close fits");
        self.outcome = None;
        self.outcome_kind = None;
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
        Some(key)
    }
    fn watch_run(&mut self, task: &c::Task, out: &mut Queue<Request>) {
        if let c::ChipPhaseFacts {
            phase:
                c::ChipPhase::Active(c::RunStage::Claimed { .. } | c::RunStage::Running { .. })
                | c::ChipPhase::Closing(c::ClosingStage::Run { .. }),
            ..
        } = task.chip.details.as_ref()
        {
            if self.run.is_none() {
                self.current_run = None;
                let handle = handle(self.next);
                self.next = self.next.checked_add(1).expect("chat handles fit");
                self.run = Some(handle);
                out.push(Request::Client(jig_client::Event::Watch {
                    handle,
                    subject: Watchable::Jig(c::Subject::Run { task: self.task, turn: None, offset: 0 }),
                }));
            }
        } else if let Some(handle) = self.run.take() {
            self.current_run = None;
            unwatch(handle, out);
        }
    }
    fn pending_key(&self, key: Key) -> bool {
        for (pending, _) in &self.pending {
            if *pending == key {
                return true;
            }
        }
        false
    }
    #[expect(clippy::too_many_lines, reason = "the exhaustive conversation act routing stays in one match")]
    pub(crate) fn act(
        &mut self,
        env: &Env<Limits>,
        client: &Client,
        act: Act,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        match act {
            Act::ChangeProposal { proposal } => {
                self.change_asked = Some(proposal);
                self.composer_focus = self.composer_focus.checked_add(1).expect("focus count fits");
            }
            Act::MessageWords { words } => {
                (self.words, self.beyond) = super::composer::held(words, env.limits.client.domain.long_text);
            }
            Act::Mode { mode } => {
                self.opening_mode = None;
                self.mode = mode;
            }
            Act::ShowSection { line } => {
                if self.mode == Mode::Chat {
                    self.mode = Mode::Split;
                }
                self.shown = Some(line);
                self.shown_count = self.shown_count.checked_add(1).expect("shown count fits");
            }
            Act::Newer => ask_activity(&mut self.asking, self.activity, self.thread.newer(), out),
            Act::Older => ask_activity(&mut self.asking, self.activity, self.thread.older(), out),
            Act::Latest => self.request_end(out),
            Act::AddDraft => {
                let Some(task) = self.task(client) else { return };
                if self.draft.is_some() || self.writing || task.result.is_some() || super::chats::ended(&task.chip) {
                    return;
                }
                if self.sending(Sending::Message) {
                    return;
                }
                if self.mode == Mode::Chat {
                    self.mode = Mode::Split;
                }
                self.send(env, client, Sending::Message, Some(document::WRITE_THIS_UP), serial, out);
                for (key, kind) in &self.pending {
                    if *kind == Sending::Message {
                        self.write_up = Some(*key);
                        self.writing = true;
                    }
                }
            }
            Act::Send => {
                self.send(env, client, Sending::Message, None, serial, out);
                self.composer_focus = self.composer_focus.checked_add(1).expect("focus count fits");
            }
            Act::Stop => self.send(env, client, Sending::Stop, None, serial, out),
            Act::Resume => self.send(env, client, Sending::Resume, None, serial, out),
            Act::OpenStart { .. }
            | Act::OpenLanded { .. }
            | Act::StartDraft { .. }
            | Act::StartName { .. }
            | Act::StartWay { .. }
            | Act::StartAsks { .. }
            | Act::StartCancel
            | Act::StartGoal
            | Act::Switcher(_)
            | Act::Away(_)
            | Act::Terminal(_)
            | Act::Told(_)
            | Act::Remembered(_)
            | Act::Models(_)
            | Act::HandArea(_)
            | Act::StopAsking(_)
            | Act::Needs(_)
            | Act::Finding(_)
            | Act::Tell { .. }
            | Act::Choice(_)
            | Act::Accounts(_)
            | Act::ConnectModel(_)
            | Act::MakeDefault
            | Act::AdoptFirst
            | Act::CopyAgain { .. }
            | Act::AddOpen { .. }
            | Act::AddChoose { .. }
            | Act::AddWrite { .. }
            | Act::AddClose
            | Act::AddSubmit
            | Act::CopyOffered
            | Act::DefaultName { .. }
            | Act::OpenProject { .. }
            | Act::Project { .. }
            | Act::SignOut
            | Act::AskLine { .. }
            | Act::AskWith { .. }
            | Act::ChatWords { .. }
            | Act::AskHow
            | Act::StartChat
            | Act::WayTab { .. }
            | Act::WayStop { .. }
            | Act::UseWay { .. }
            | Act::KeepWay { .. }
            | Act::SkipToBegin
            | Act::RetryPlayground
            | Act::Playground
            | Act::Suggest { .. }
            | Act::MoreGoals
            | Act::GoalsFilter { .. }
            | Act::ChatFilter { .. }
            | Act::ClosedChats
            | Act::Filter { .. }
            | Act::Decide { .. }
            | Act::CardWords { .. }
            | Act::Again { .. }
            | Act::Confirm
            | Act::Cancel
            | Act::Close
            | Act::DeleteProject { .. }
            | Act::StartOver
            | Act::DelegateFate { .. }
            | Act::LeaveTalk
            | Act::Pause
            | Act::Amend
            | Act::AmendWords { .. }
            | Act::AmendTracked
            | Act::AmendPriority { .. }
            | Act::DiscardAmendment
            | Act::SendAmendment
            | Act::AmendAgain
            | Act::OpenResult { .. }
            | Act::Go { .. } => unreachable!("only conversation acts routed here"),
        }
    }
    pub(crate) fn see(&mut self, serial: &mut u64, out: &mut Queue<Request>) {
        let mut newest = 0;
        for row in &self.thread.lines.rows {
            if let c::Activity::Turn { reply, .. } = row.source.line {
                newest = newest.max(reply);
            }
        }
        if newest <= self.seen || self.sending(Sending::Seen) {
            return;
        }
        *serial = serial.checked_add(1).expect("chat keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.seen_before = self.seen;
        self.seen = newest;
        self.pending.push((key, Sending::Seen)).expect("one seen fits");
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(c::Ask::Seen {
                through: c::SeenThrough::Conversation { task: self.task, through: self.seen },
            })),
        }));
    }
    fn send(
        &mut self,
        env: &Env<Limits>,
        client: &Client,
        kind: Sending,
        supplied: Option<&[u8]>,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        let message = supplied.unwrap_or(&self.words);
        let Some(task) = self.task(client) else { return };
        let me = match latest(client, super::me_handle()) {
            Some(Latest::Jig(c::Delivery::Me(me))) => Some(me.as_ref()),
            Some(Latest::Jig(_) | Latest::Family(_)) | None => None,
        };
        let message_act = if self.talking.is_some() { c::ActKind::Talk } else { c::ActKind::Message };
        let resumes = self.talking.is_none()
            && match me {
                Some(me) => paused(&task.chip, me) && allowed(task, c::ActKind::Release),
                None => false,
            };
        let words = match kind {
            Sending::Message if !message.is_empty() && allowed(task, message_act) => {
                let Some(me) = me else { return };
                if supplied.is_none() && super::composer::over(&self.words, self.beyond, me.words_bytes) > 0 {
                    return;
                }
                let Ok(words) = c::Text::new(message.into(), env.limits.client.domain.long_text) else { return };
                Some(words)
            }
            Sending::Message => return,
            Sending::Stop | Sending::Resume | Sending::Seen | Sending::Close => None,
        };
        let ask = match kind {
            Sending::Message => {
                let words = words.as_ref().expect("message words checked").clone();
                if let Some(talk) = &self.talking {
                    c::Ask::Talk { project: talk.project, task: talk.task, words }
                } else if resumes {
                    c::Ask::Compound {
                        project: task.project,
                        operations: List::from_boxed(Box::new([
                            c::Operation::Message { project: task.project, task: self.task, words },
                            c::Operation::Release {
                                project: task.project,
                                task: self.task,
                                revision: c::Revision {
                                    value: super::task::chip_hold(&task.chip).expect("paused is held"),
                                },
                                again: None,
                            },
                        ])),
                    }
                } else {
                    c::Ask::Message { project: task.project, task: self.task, words }
                }
            }
            Sending::Resume if resumes => c::Ask::Release {
                project: task.project,
                task: self.task,
                revision: c::Revision { value: super::task::chip_hold(&task.chip).expect("paused is held") },
                again: None,
            },
            Sending::Stop if allowed(task, c::ActKind::Stop) => c::Ask::Stop { project: task.project, task: self.task },
            Sending::Stop | Sending::Resume | Sending::Seen | Sending::Close => return,
        };
        for (_, pending) in &self.pending {
            if *pending == kind {
                return;
            }
        }
        *serial = serial.checked_add(1).expect("chat keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        if let Some(words) = words {
            let Some(me) = me else { return };
            if !self.thread.sending(key, me.party.number, words, 0) {
                self.outcome = Some(jig_client::Outcome::Limit);
                self.outcome_kind = Some(kind);
                return;
            }
            if supplied.is_none() {
                self.words = Box::default();
                self.beyond = 0;
            }
            self.change_asked = None;
        }
        self.pending.push((key, kind)).expect("one request per chat act fits");
        self.outcome = None;
        self.outcome_kind = None;
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
        if kind == Sending::Message && supplied.is_none() {
            self.request_end(out);
        }
    }
}
/// A screen and a half as a chat's opening reads hold it: the held lines'
/// weight, a screen and a half of an 80 by 24 terminal
/// (temper/docs/design/client/pages/chat/conversation.md, 2).
pub(crate) const OPENING_WEIGHT: u64 = 2_880;
#[must_use]
pub fn thread_limits(limits: &Limits) -> jig_client::thread::Limits {
    let bytes = match limits.window_bytes {
        Some(window) => window,
        None => limits.client.domain.document_bytes,
    };
    jig_client::thread::Limits {
        lines: u32::try_from(bytes.checked_div(u64::from(c::LINE_ROOM)).expect("line room is nonzero"))
            .unwrap_or(u32::MAX),
        bytes,
        text: limits.client.domain.long_text,
        live: limits.client.domain.long_text,
    }
}
pub(crate) fn page_task(page: Readout, task: u64) -> Option<Box<c::Task>> {
    match page {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(task))) => Some(task),
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(value))) if value.matches(c::ObjectKey::Task(task)) => {
            if let c::EndedValue::Task { mut document, result } = value.value {
                document.result = Some(c::TaskResult::Document(result));
                Some(document)
            } else {
                None
            }
        }
        Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => None,
    }
}
fn handle(number: u64) -> Handle {
    Handle { asker: Owner::Pages.asker(), number }
}
fn unwatch(handle: Handle, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Unwatch { handle }));
}
fn read(handle: Handle, read: Option<c::Read>, out: &mut Queue<Request>) {
    if let Some(read) = read {
        out.push(Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(read) }));
    }
}
#[must_use]
pub fn allowed(task: &c::Task, act: c::ActKind) -> bool {
    match jig_client::facts::offer(task.acts.as_slice(), act) {
        Some(c::Admission::Allowed | c::Admission::Bounded { .. }) => true,
        Some(c::Admission::Proposed { .. } | c::Admission::Refused(_)) | None => false,
    }
}
fn latest(client: &Client, handle: Handle) -> Option<&Latest> {
    client.document(handle)?.latest
}

fn ask_activity(asking: &mut Asking, handle: Handle, read: Option<c::Read>, out: &mut Queue<Request>) {
    if let Some(read) = read {
        asking.ask(handle, read, out);
    }
}

fn run_notes(run: &c::RunState) -> bool {
    match &run.call {
        Some(call) => document::writes(call.tool.bytes()),
        None => false,
    }
}

/// The person's own pause (client/pages/README.md): held by a placed hold of theirs, mode Stop, over the task's own subtree.
#[must_use]
pub fn paused(chip: &c::Chip, me: &c::Me) -> bool {
    let c::ChipPhaseFacts { phase, details } = chip.details.as_ref();
    match phase {
        c::ChipPhase::Held { .. } => {
            let Some(hold) = &details.hold else { return false };
            let Some(placed) = &hold.placed else { return false };
            placed.mode == c::HoldMode::Stop
                && placed.scope == (c::HoldScope::Subtree { task: chip.task })
                && !placed.lifted
                && placed.owner == (c::HoldOwner::Party { person: me.party.number })
        }
        c::ChipPhase::Waiting { .. }
        | c::ChipPhase::Active(_)
        | c::ChipPhase::Person { .. }
        | c::ChipPhase::Closing(_)
        | c::ChipPhase::Ended => false,
    }
}

fn newer_decision(decided: Option<(u64, Wall)>, number: u64) -> bool {
    match decided {
        Some((previous, _)) => number >= previous,
        None => true,
    }
}
