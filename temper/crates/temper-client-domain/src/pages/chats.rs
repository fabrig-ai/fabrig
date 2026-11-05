//! Live chats are borrowed from the client; closed rows are one bounded read page.
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
use skein_lib::{Env, Queue, Writer};

/// The configuring agent's stock template (temper/docs/design/protocol/configuration.md, 13.4).
const CONFIGURING_TEMPLATE: u32 = 3;

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the configured query is private watch bookkeeping")]
pub struct Chats {
    pub filter: ChatsFilter,
    pub live: Handle,
    pub closed: Handle,
    pub words: Box<[u8]>,
    pub start: Option<Key>,
    pub page: Option<TaskPage>,
    pub refusal: Option<jig_client::shapes::NotOffered>,
    pub outcome: Option<jig_client::Outcome>,
    pub(crate) opening: Option<(u64, Mode)>,
    starting_mode: Mode,
    query: Option<TaskQuery>,
    first: bool,
    page_delivered: bool,
}
impl Chats {
    pub(crate) fn new(first: u64) -> Self {
        Self {
            filter: ChatsFilter::All,
            live: handle(first),
            closed: handle(first.checked_add(1).expect("page handles fit")),
            words: Box::default(),
            start: None,
            page: None,
            refusal: None,
            outcome: None,
            opening: None,
            starting_mode: Mode::Chat,
            query: None,
            first: true,
            page_delivered: false,
        }
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [self.live, self.closed] {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        self.closed.number.checked_add(1).expect("page handles fit")
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
        self.page = None;
        self.page_delivered = false;
        self.first = true;
        self.query = Some(query);
        out.push(Request::Client(jig_client::Event::Watch {
            handle: self.live,
            subject: Watchable::Jig(c::Subject::Tasks { query, window: limits.client.domain.rows }),
        }));
        self.read(None, out);
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
    pub(crate) fn write(&mut self, words: Box<[u8]>, _limits: &Limits) {
        if words.len() <= usize::try_from(temper_client_protocol_wire::CUE_BYTES).expect("text bound fits") {
            self.words = words;
        }
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
        let words = if draft {
            let Some(size) = self.words.len().checked_add(2) else { return };
            let Some(size) = size.checked_add(WRITE_THIS_UP.len()) else { return };
            if size > usize::try_from(temper_client_protocol_wire::CUE_BYTES).expect("cue bound fits") {
                return;
            }
            let mut words = Writer::new(size);
            words.put(&self.words).expect("measured cue");
            words.put(b"\n\n").expect("measured separator");
            words.put(WRITE_THIS_UP).expect("measured draft request");
            words.finish()
        } else {
            self.words.clone()
        };
        self.send(env, me, project, jig_client::shapes::CHAT, words, Some(choice), serial, out);
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
        self.send(env, me, project, jig_client::shapes::CHAT, words.into(), Some(choice), serial, out);
    }
    pub(crate) fn configure_chat(
        &mut self,
        env: &Env<Limits>,
        me: Option<&Me>,
        project: Option<u32>,
        words: Box<[u8]>,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        if self.start.is_some()
            || words.len() > usize::try_from(temper_client_protocol_wire::CUE_BYTES).expect("text bound fits")
        {
            return;
        }
        self.words = words;
        let template = configuring_template(me, project);
        self.send(env, me, project, template, self.words.clone(), None, serial, out);
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
        let Ok(words) = Text::new(words, temper_client_protocol_wire::CUE_BYTES) else { return };
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
                self.page = Some(*page);
                true
            }
            Told::Page { handle, .. } if handle == self.closed => {
                self.page_delivered = true;
                self.page = None;
                true
            }
            Told::Ended { handle, .. } if handle == self.live => true,
            Told::Pending { key } if Some(key) == self.start => true,
            Told::Answered { key, outcome } if Some(key) == self.start => {
                if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(value))) = &outcome {
                    if let c::Outcome::Document(c::DocumentOutcome::Created { first, .. }) = value {
                        self.opening = Some((*first, self.starting_mode));
                        show_chat(*first, address, out);
                    } else if let c::Outcome::Made { what: c::Made::Tasks { tasks } } = value
                        && let Some(task) = tasks.get(0)
                    {
                        self.opening = Some((*task, self.starting_mode));
                        show_chat(*task, address, out);
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
    pub(super) fn delivered(&self, client: &Client, project: u32) -> bool {
        let Some(query) = self.query else { return false };
        query.project == Some(project) && self.page_delivered && super::home::delivered(client, self.live)
    }
    pub(super) fn chat_counts(&self, client: &Client, me: Option<&Me>, project: u32) -> (bool, bool) {
        let Some(query) = self.query else { return (false, false) };
        if query.project != Some(project) {
            return (false, false);
        }
        let mut counts = (false, false);
        if let Some(document) = client.document(self.live)
            && document.state != jig_client::State::Ended
            && let Some(Latest::Jig(Delivery::Tasks(live))) = document.latest
        {
            count_rows(&live.rows, me, project, &mut counts);
            counts.1 |= live.more;
        }
        if let Some(page) = &self.page {
            count_rows(&page.live, me, project, &mut counts);
            counts.1 |= page.live_after.is_some();
            match &page.ended {
                EndedGroup::Rows { rows, after, .. } => {
                    count_rows(rows, me, project, &mut counts);
                    counts.1 |= after.is_some();
                }
                EndedGroup::None => {}
            }
        }
        counts
    }
    /// The person's newest configuring chat in the project, not closed
    /// (temper/docs/design/client/README.md, 3.5, Settings).
    #[must_use]
    pub fn open_configuring(&self, client: &Client, me: Option<&Me>, project: u32) -> Option<u64> {
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
            newest_configuring(&live.rows, me, project, &mut newest);
        }
        if let Some(page) = &self.page {
            newest_configuring(&page.live, me, project, &mut newest);
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

/// A chat of template 3's charter (temper/docs/design/client/README.md, 3.5, To begin).
#[must_use]
pub fn configuring(me: &Me, project: u32, executor: &c::ChipExecutor) -> bool {
    let Some(template) = jig_client::shapes::template(me, project, CONFIGURING_TEMPLATE) else { return false };
    match template.executor {
        c::TemplateExecutor::Charter(charter) => *executor == c::ChipExecutor::Agent { charter },
        c::TemplateExecutor::Chosen => false,
    }
}

fn count_rows(rows: &skein_lib::List<c::TaskLine>, me: Option<&Me>, project: u32, counts: &mut (bool, bool)) {
    for row in rows {
        if row.project == project {
            let configuring = match me {
                Some(me) => configuring(me, project, &row.chip.executor),
                None => false,
            };
            counts.0 |= configuring;
            counts.1 |= !configuring;
        }
    }
}

fn configuring_template(me: Option<&Me>, project: Option<u32>) -> u32 {
    if let Some(me) = me
        && let Some(project) = project
        && jig_client::shapes::template(me, project, CONFIGURING_TEMPLATE).is_some()
    {
        CONFIGURING_TEMPLATE
    } else {
        jig_client::shapes::CHAT
    }
}

fn show_chat(task: u64, from: &Address, out: &mut Queue<Request>) {
    let address = Address::Chat { task };
    out.push(if let Address::NewChat { .. } = from { Request::Replace { address } } else { Request::Show { address } });
}

fn newest_configuring(rows: &skein_lib::List<c::TaskLine>, me: &Me, project: u32, newest: &mut Option<u64>) {
    for row in rows {
        if row.project == project
            && configuring(me, project, &row.chip.executor)
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

/// A chat whose phase or phase facts have ended (temper/docs/design/client/README.md, 3.5).
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
