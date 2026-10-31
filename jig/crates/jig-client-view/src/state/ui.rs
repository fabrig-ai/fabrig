//! The client spine: frame facts, the shared link, page generation and watches.
//! No platform or application value is known. Each entry handles one owned event;
//! fire expires one timer, and reclaim frees entities whose obligations ended.
//! Nothing is accepted before Start. See client/domain/client.md, 3, 4 and 8.
//!
//! | Client | Event | Next | Emits |
//! |---|---|---|---|
//! | unstarted | start | starting | frame open |
//! | starting/offline | frame delivery | live | confirmed, retention, link |
//! | active | client link | client link | link |
//! | active | unsupported stream hello | out of date | park and close |
//! | active | different party from kept work | someone else's work | confirm nobody |
//! | active | frame hand off | unchanged | frame answer |
use crate::{
    boundary::{Event, Refused, Request},
    sizes::limits::ui::Config,
    state::{
        action::Act,
        address::{Address, Place},
        handles::Holder,
        page::{Arrangement, Page, Section},
        store::Record,
    },
    tree::frame::me::FrameState,
};
use jig_client::Link;
use jig_client::contract::{Delivery, Limit, Me, Subject};
use skein_lib::{Deadlines, Duration, Env, Id, List, Map, Queue, Time, Wall};

use alloc::boxed::Box;
pub(crate) mod borrowed;
pub(crate) mod client;
pub use borrowed::Borrowed;
pub(crate) mod owed;

/// View deadlines: one per notice, linger and saved record, plus a tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Timer {
    /// A visible notice reaches its monotonic end.
    Notice(Id<crate::tree::notices::kept::Notice>),
    /// A leaving card has lingered.
    Linger(Id<crate::state::objects::Object>),
    /// A draft save has coalesced.
    Save(crate::state::store::StoreKey),
    /// A shown wall time needs another minute's wording.
    Minute,
}

/// Jig's client state, owned once by the application client root.
#[derive(Debug)]
pub struct Ui {
    pub(crate) limits: Config,
    pub(crate) seed: u64,
    pub(crate) started: bool,
    pub(crate) drafts: crate::state::drafts::Drafts,
    pub(crate) confirming: Option<crate::state::confirm::Confirmation>,
    pub(crate) proposing: Option<Id<crate::state::objects::Object>>,
    pub(crate) refused_amendments: Map<jig_client::contract::ObjectKey, Box<jig_client::contract::Ask>>,
    pub(crate) now: Time,
    pub(crate) clock: crate::state::clock::State,
    pub(crate) notices: crate::tree::notices::kept::Notices,
    pub(crate) linger: Duration,
    pub(crate) save_delay: Duration,
    pub(crate) confirmed: Option<u64>,
    pub(crate) text_limit: u32,
    pub(crate) intents: crate::state::intents::Intents,
    pub(crate) fields: crate::state::fields::Fields,
    pub(crate) signin: crate::state::signin::SignIn,
    pub(crate) blocked: bool,
    pub(crate) link: Link,
    pub(crate) page: Page,
    pub(crate) sections: List<Section>,
    pub(crate) section_statuses: List<crate::state::page::SectionStatus>,
    pub(crate) section_context: u32,
    pub(crate) arrangement: Arrangement,
    pub(crate) inbox: crate::state::page::inbox::State,
    pub(crate) project_page: crate::state::page::project::ProjectPage,
    pub(crate) deployment_page: crate::state::page::deployment::DeploymentPage,
    pub(crate) task_page: crate::state::page::task::TaskPage,
    pub(crate) chats: crate::state::page::conversations::State,
    pub(crate) navigation: crate::state::page::navigation::Navigation,
    pub(crate) updates_count: u32,
    pub(crate) address: Address,
    pub(crate) wall: Wall,
    pub(crate) shown: u64,
    pub(crate) kept: List<Record>,
    pub(crate) kept_requests: Map<jig_client::Key, u64>,
    pub(crate) handles: crate::state::handles::Handles,
    pub(crate) frame_watch: Option<jig_client::Handle>,
    pub(crate) timers: Deadlines<Timer>,
    pub(crate) objects: crate::state::objects::Objects,
    pub(crate) object_rows: crate::state::windows::ObjectWindow,
    pub(crate) conversation_rows: crate::state::windows::ConversationWindow,
    pub(crate) conversation: jig_client::thread::Thread,
    pub(crate) history_rows: crate::state::windows::HistoryWindow,
}
impl Ui {
    /// Steps the presentation with the client its root lends for this entry.
    pub fn step(&mut self, now: Time, wall: Wall, client: &jig_client::Client, event: Event, out: &mut Queue<Request>) {
        let env = Env { now, wall, limits: self.limits };
        self.now = now;
        self.wall = wall;
        match event {
            Event::Client { told } if self.started => client::told(self, &env, client, told, self.seed, out),
            Event::Client { .. } => {}
            event @ Event::Start { .. } => {
                step(self, &env, event, out, client);
                client::told(self, &env, client, jig_client::Told::Link { link: client.link().state }, self.seed, out);
            }
            event @ (Event::Went { .. } | Event::Left | Event::Act { .. } | Event::Named { .. }) => {
                step(self, &env, event, out, client);
            }
        }
        settle(self, &env, out, client);
    }
    /// Fires one presentation deadline using the client's current state.
    pub fn fire(&mut self, now: Time, wall: Wall, client: &jig_client::Client, out: &mut Queue<Request>) {
        let env = Env { now, wall, limits: self.limits };
        fire(self, &env, out, client);
        settle(self, &env, out, client);
    }
    /// Allocates bounded state from limits.
    pub fn new(
        limits: &Config,
        client: &jig_client::DomainLimits,
        navigation: crate::state::page::navigation::Navigation,
        home: Address,
        arrangement: Arrangement,
    ) -> Result<Self, Refused> {
        if !valid_navigation(&navigation, limits.navigation_entries) {
            return Err(Refused::Navigation);
        }
        if !arrangement.valid() {
            return Err(Refused::Arrangement);
        }
        assert!(crate::sizes::limits::ui::valid(limits, client), "client limits are valid");
        let mut conversation = jig_client::thread::Thread::new(
            0,
            &jig_client::thread::Limits { lines: client.rows, bytes: client.document_bytes, text: client.text },
        );
        conversation.words = jig_client::thread::KeptWords::new(limits.conversation_words);
        Ok(Self {
            limits: *limits,
            seed: 0,
            started: false,
            drafts: crate::state::drafts::Drafts::new(limits.drafts),
            confirming: None,
            proposing: None,
            refused_amendments: Map::with_capacity(limits.objects),
            now: Time::ZERO,
            clock: crate::state::clock::State { page_times: false, armed: false },
            notices: crate::tree::notices::kept::Notices::new(limits.notices, limits.notice_life),
            linger: limits.linger,
            save_delay: limits.save_delay,
            confirmed: None,
            text_limit: client.text,
            intents: crate::state::intents::Intents::new(limits.intents),
            fields: crate::state::fields::Fields::new(client.fields, client.text),
            signin: crate::state::signin::SignIn::new(),
            blocked: false,
            link: Link::Starting,
            page: Page::Starting,
            sections: List::with_capacity(crate::state::page::MOST_SECTIONS),
            section_statuses: List::with_capacity(crate::state::page::MOST_SECTIONS),
            section_context: u32::MAX,
            arrangement,
            inbox: crate::state::page::inbox::State::new(client.rows),
            project_page: crate::state::page::project::ProjectPage::new(client.rows),
            deployment_page: crate::state::page::deployment::DeploymentPage::new(),
            task_page: crate::state::page::task::TaskPage::new(client.rows, limits.objects),
            chats: crate::state::page::conversations::State::new(),
            navigation,
            updates_count: limits.updates_count,
            address: home,
            wall: Wall::EPOCH,
            shown: 0,
            kept: List::with_capacity(client.records),
            kept_requests: Map::with_capacity(client.kept),
            handles: crate::state::handles::Handles::new(client.watches, client.reads),
            frame_watch: None,
            timers: Deadlines::with_capacity(
                crate::sizes::limits::ui::timers(limits, client).expect("timer capacity is validated"),
            ),
            objects: crate::state::objects::Objects::new(limits.objects, limits.sources),
            object_rows: crate::state::windows::ObjectWindow::new(client.rows),
            conversation_rows: crate::state::windows::ConversationWindow::new(client.rows),
            conversation,
            history_rows: crate::state::windows::HistoryWindow::new(client.rows),
        })
    }
    /// The page's sections in their display order (client/domain/client.md, 5.1).
    #[must_use]
    pub fn sections(&self) -> &[Section] {
        match self.page {
            Page::Work { .. } | Page::Elsewhere => self.sections.as_slice(),
            Page::Starting | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => &[],
        }
    }
    /// Installs ordered navigation and its home before start; capacities are checked.
    pub fn configure_navigation(
        &mut self,
        navigation: crate::state::page::navigation::Navigation,
    ) -> Result<(), crate::state::page::navigation::Navigation> {
        if self.started || !valid_navigation(&navigation, self.limits.navigation_entries) {
            return Err(navigation);
        }
        self.navigation = navigation;
        Ok(())
    }
    /// Installs an application's arrangement before start (client/domain/client.md, 5.1).
    pub fn configure_arrangement(&mut self, arrangement: Arrangement) -> Result<(), Arrangement> {
        if self.started || !arrangement.valid() {
            return Err(arrangement);
        }
        self.arrangement = arrangement;
        Ok(())
    }
    /// Sections added beside each task or project address's own.
    #[must_use]
    pub const fn arrangement(&self) -> &Arrangement {
        &self.arrangement
    }
    /// Inbox context, filters and historical cursors for its shared cards.
    #[must_use]
    pub const fn inbox(&self) -> &crate::state::page::inbox::State {
        &self.inbox
    }
    /// Current project facts, board, notes and the goal form.
    #[must_use]
    pub const fn project_page(&self) -> &crate::state::page::project::ProjectPage {
        &self.project_page
    }
    /// Current admitted deployment facts.
    #[must_use]
    pub const fn deployment_page(&self) -> &crate::state::page::deployment::DeploymentPage {
        &self.deployment_page
    }
    /// The task header and its bounded plan and history choices.
    #[must_use]
    pub const fn task_page(&self) -> &crate::state::page::task::TaskPage {
        &self.task_page
    }
    /// The new-chat form's request and factual refusal.
    #[must_use]
    pub const fn chats(&self) -> &crate::state::page::conversations::State {
        &self.chats
    }
    /// Configured destinations, in the application's order.
    #[must_use]
    pub const fn navigation(&self) -> &crate::state::page::navigation::Navigation {
        &self.navigation
    }
    /// Counts and current identity shown on every page, including application pages.
    #[must_use]
    pub fn frame_summary(&self, lent: &jig_client::Client) -> crate::tree::frame::me::Summary {
        let mut summary =
            self.frame(lent).summary(self.address, self.link, self.page_behind_value(), self.updates_count);
        match self.address {
            Address::Task { task, .. } => {
                if let Some(id) = self.objects.find(jig_client::contract::ObjectKey::Task(task))
                    && let Some(object) = self.borrow(lent).objects().get(id)
                {
                    match &object.body {
                        Some(crate::state::objects::Body::Task(task)) => summary.project = Some(task.project),
                        Some(
                            crate::state::objects::Body::Historical(_)
                            | crate::state::objects::Body::Chip(_)
                            | crate::state::objects::Body::OwnerProposal(_)
                            | crate::state::objects::Body::OwnerEscalation(_)
                            | crate::state::objects::Body::Question(_)
                            | crate::state::objects::Body::OwnerPersonTask(_)
                            | crate::state::objects::Body::Result(_)
                            | crate::state::objects::Body::Note { .. },
                        )
                        | None => {}
                    }
                }
            }
            Address::Inbox { .. } | Address::Conversations { .. } | Address::Project { .. } | Address::Deployment => {}
        }
        summary
    }
    fn page_behind_value(&self) -> bool {
        match self.page {
            Page::Work { behind, .. } => behind,
            Page::Starting
            | Page::Elsewhere
            | Page::SigningIn
            | Page::Missing
            | Page::OutOfDate
            | Page::SomeoneElse => false,
        }
    }
    /// Visible notices and their monotonic ends.
    #[must_use]
    pub const fn notices(&self) -> &crate::tree::notices::kept::Notices {
        &self.notices
    }
    /// Page policy names whether its own documents show wall times.
    pub fn timed_page(&mut self, showing: bool) {
        self.clock.page_times = showing;
        crate::state::clock::sync(self);
    }
    /// An object policy leaves a card visible for the configured linger.
    pub fn leave(&mut self, id: Id<crate::state::objects::Object>, ending: jig_client::contract::Ending) -> bool {
        let Some(object) = self.objects.get_mut(id) else {
            return false;
        };
        let until = self.now.saturating_add(self.linger);
        object.card =
            crate::state::objects::Card::Leaving { how: crate::state::objects::Left::Ended { how: ending }, until };
        self.timers.arm(Timer::Linger(id), until).expect("one linger deadline per object fits");
        self.changed();
        true
    }
    /// Keeps the durable outcome on a card until its configured linger ends.
    pub fn leave_answer(
        &mut self,
        id: Id<crate::state::objects::Object>,
        outcome: jig_client::contract::Outcome,
    ) -> bool {
        let Some(object) = self.objects.get_mut(id) else {
            return false;
        };
        let until = self.now.saturating_add(self.linger);
        object.card =
            crate::state::objects::Card::Leaving { how: crate::state::objects::Left::Answered { outcome }, until };
        self.timers.arm(Timer::Linger(id), until).expect("one linger deadline per object fits");
        self.changed();
        true
    }
    /// The one object confirmation, including its observed revision and choices.
    #[must_use]
    pub const fn confirmation(&self) -> Option<&crate::state::confirm::Confirmation> {
        self.confirming.as_ref()
    }
    /// Pending intents and any terminal awaiting reclaim.
    #[must_use]
    pub const fn intents(&self) -> &crate::state::intents::Intents {
        &self.intents
    }
    /// Current-party drafts, including their saved touch order.
    #[must_use]
    pub const fn drafts(&self) -> &crate::state::drafts::Drafts {
        &self.drafts
    }
    /// Current fields, with counts of domain writes.
    #[must_use]
    pub const fn fields(&self) -> &crate::state::fields::Fields {
        &self.fields
    }
    /// Writes a field from domain policy and advances the view's shown count.
    pub fn write_field(
        &mut self,
        purpose: crate::state::fields::Purpose,
        text: jig_client::contract::Text,
    ) -> Result<(), Limit> {
        self.fields.write(purpose, text)?;
        self.changed();
        Ok(())
    }
    /// This domain's restored and saved records.
    #[must_use]
    pub const fn records(&self) -> &List<Record> {
        &self.kept
    }
    /// What the frame most recently said, without interpreting authority.
    #[must_use]
    pub fn frame<'a>(&self, client: &'a jig_client::Client) -> FrameState<'a> {
        match client.link().state {
            Link::SignedOut | Link::OutOfDate => return FrameState { document: None },
            Link::Starting | Link::Live | Link::Offline => {}
        }
        let document = match self.frame_watch {
            Some(handle) => match client.document(handle) {
                Some(document) => match document.latest {
                    Some(jig_client::Latest::Jig(Delivery::Me(frame)))
                        if crate::tree::frame::me::fits(frame, &self.limits, client.domain_limits()) =>
                    {
                        Some(frame.as_ref())
                    }
                    Some(jig_client::Latest::Jig(_) | jig_client::Latest::Family(_)) | None => None,
                },
                None => None,
            },
            None => None,
        };
        FrameState { document }
    }
    /// The one page the view builds.
    #[must_use]
    pub const fn page(&self) -> &Page {
        &self.page
    }
    /// The link judged from the frame watch alone.
    #[must_use]
    pub const fn link(&self) -> Link {
        self.link
    }
    /// The count of changes affecting the built view.
    #[must_use]
    pub const fn shown(&self) -> u64 {
        self.shown
    }
    /// The display wall clock at the last entry.
    #[must_use]
    pub const fn wall(&self) -> Wall {
        self.wall
    }
    /// The first native handle named by this page, retained until it leaves.
    pub(crate) fn page_handle(&self) -> Option<jig_client::Handle> {
        self.handles.anchor
    }
    /// The page's objects held once by key.
    #[must_use]
    pub fn objects(&self) -> &crate::state::objects::Objects {
        &self.objects
    }
    /// Object names in the page's current bounded window.
    #[must_use]
    pub const fn object_rows(&self) -> &crate::state::windows::ObjectWindow {
        &self.object_rows
    }
    /// The current task's activity, provisional turns and page-owned folds.
    #[must_use]
    pub const fn conversation(&self) -> &jig_client::thread::Thread {
        &self.conversation
    }
    /// Activity including turns in the current window.
    #[must_use]
    pub const fn turn_rows(&self) -> &jig_client::thread::window::TurnWindow {
        &self.conversation.lines
    }
    /// Conversations in the current bounded window.
    #[must_use]
    pub const fn conversation_rows(&self) -> &crate::state::windows::ConversationWindow {
        &self.conversation_rows
    }
    /// History in the current bounded window.
    #[must_use]
    pub const fn history_rows(&self) -> &crate::state::windows::HistoryWindow {
        &self.history_rows
    }
    /// The next timer the application's loop should fire.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.timers.next()
    }
    /// Reclaims entities only after their terminals arrived.
    pub fn reclaim(&mut self) {
        self.objects.reclaim();
        self.intents.reclaim();
        self.notices.reclaim();
        crate::state::clock::sync(self);
    }
    pub(crate) fn changed(&mut self) {
        self.shown = self.shown.saturating_add(1);
    }
    pub(crate) fn set_link(&mut self, link: Link) {
        if self.link != link {
            let previous = self.link;
            self.link = link;
            match previous {
                Link::Starting | Link::SignedOut | Link::OutOfDate => {}
                Link::Live | Link::Offline => {
                    crate::tree::notices::kept::show(self, crate::tree::notices::kept::Kind::Link(link));
                }
            }
            self.changed();
        }
    }
    pub(crate) fn hold_watch(
        &mut self,
        subject: Subject,
        owner: Holder,
        out: &mut Queue<Request>,
    ) -> Option<jig_client::Handle> {
        let handle = self.handles.watch(subject, owner, out);
        if handle.is_none() {
            crate::state::page::routing::refuse(self, Limit::Watches);
        }
        handle
    }
    pub(crate) fn release(&mut self, handle: jig_client::Handle, out: &mut Queue<Request>) {
        if self.handles.watches.remove(&handle).is_none() {
            return;
        }
        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle } });
        self.unname(handle);
    }
    fn unname(&mut self, handle: jig_client::Handle) {
        let count = self.objects.len();
        let excess = self.objects.replace(crate::state::objects::Source::Watch(handle), &[]);
        assert!(excess == 0, "empty source replacement never refuses");
        if self.objects.len() != count {
            self.changed();
        }
        if self.frame_watch == Some(handle) {
            self.frame_watch = None;
        }
    }
    pub(crate) fn leave_handles(&mut self, all: bool, out: &mut Queue<Request>) {
        self.handles.anchor = None;
        for _ in 0..self.handles.watches.len() {
            let mut found = None;
            for (handle, held) in &self.handles.watches {
                if all || held.owner != Holder::Me {
                    found = Some(*handle);
                    break;
                }
            }
            let Some(handle) = found else {
                break;
            };
            self.release(handle, out);
        }
    }
    pub(crate) fn show_work(&mut self, env: &Env<Config>, out: &mut Queue<Request>, lent: &jig_client::Client) {
        crate::state::page::routing::enter(self, env, out, lent);
    }
}

/// Routes one event, accepting no input until Start.
pub fn step(domain: &mut Ui, env: &Env<Config>, event: Event, out: &mut Queue<Request>, lent: &jig_client::Client) {
    if !domain.started {
        match event {
            Event::Start { records, place, seed } => {
                domain.seed = seed;
                domain.intents.seed = seed;
                start(domain, env, place, records, out, lent);
            }
            Event::Client { .. } | Event::Left | Event::Went { .. } | Event::Act { .. } | Event::Named { .. } => {}
        }
        return;
    }
    domain.now = env.now;
    domain.wall = env.wall;
    match event {
        Event::Client { .. } => unreachable!("client tells enter through Ui::step"),
        Event::Went { place } => {
            crate::state::confirm::close(domain);
            crate::state::page::routing::navigate(domain, env, place, true, out, lent);
        }
        Event::Act { action } => act_event(domain, env, action, out, lent),
        Event::Start { .. } => {}
        Event::Named { objects, sections } => {
            crate::state::page::shown::show(domain, env, sections, out, lent);
            crate::state::application::show(domain, env, objects, out, lent);
        }
        Event::Left => crate::state::page::routing::navigate(domain, env, Place::Elsewhere, false, out, lent),
    }
}
fn settle(domain: &mut Ui, env: &Env<Config>, out: &mut Queue<Request>, lent: &jig_client::Client) {
    domain.section_context = u32::MAX;
    crate::state::drafts::sync(domain, out);
    crate::state::confirm::sync(domain, lent);
    crate::state::activity::sending::restore(domain, lent);
    crate::state::activity::chips::sync(domain, env, out, lent);
    crate::state::clock::sync(domain);
}
fn act_event(domain: &mut Ui, env: &Env<Config>, act: Act, out: &mut Queue<Request>, lent: &jig_client::Client) {
    match act {
        Act::ProjectSettings { ask } => crate::state::page::project::owner_form::review(domain, ask),
        Act::ProjectServiceRole { role } => crate::state::page::project::owner_form::service_role(domain, role),
        Act::Go { address } => {
            crate::state::confirm::close(domain);
            crate::state::page::routing::navigate(domain, env, Place::Jig(address), false, out, lent);
        }
        Act::Edit { field: crate::state::fields::Purpose::Composer { task }, text } => {
            if let Err(which) = crate::state::drafts::edit(domain, task, text, out) {
                crate::state::page::routing::refuse(domain, which);
            }
        }
        Act::Edit { field, text } => match domain.fields.edit(field, text) {
            Ok(()) => {
                crate::state::confirm::edited(domain, field);
                domain.changed();
            }
            Err(which) => {
                crate::state::page::routing::refuse(domain, which);
            }
        },
        Act::Dismiss => dismiss_act(domain),
        Act::SignOut => crate::state::signin::sign_out(domain, out),
        Act::BeginConversation { project } => crate::state::page::conversations::start(domain, env, project, out, lent),
        Act::OpenChat { task } => crate::state::page::conversations::open_chat(domain, env, task, out, lent),
        Act::OpenUpdates => crate::state::page::conversations::open_updates(domain, out, lent),
        Act::FilterInbox { filter } => crate::state::page::conversations::filter_inbox(domain, filter, out),
        Act::Filter { group, project } => crate::state::page::conversations::filter_inbox(
            domain,
            crate::state::page::inbox::Filter { project, kind: Some(group), goal: None },
            out,
        ),
        Act::FoldCall { name, open } => crate::state::activity::routing::fold(domain, name, open, lent),
        Act::FoldCurrentCall { identity, open } => {
            crate::state::activity::routing::fold_current(domain, identity, open, lent);
        }
        Act::More { window, end } => {
            crate::state::activity::routing::more(domain, window, end, out);
            crate::state::page::conversations::more(domain, window, end, out, lent);
            crate::state::page::task::more(domain, window, end, out);
            crate::state::page::project::more(domain, window, end, out, lent);
        }
        Act::GoalForm { form } => crate::state::page::project::goals::configure(domain, form),
        Act::FilterTasks { filter } => crate::state::page::project::tasks(domain, env, filter, out, lent),
        Act::FilterNotes { filter } => crate::state::page::project::notes::filter(domain, filter, out),
        Act::Submit { field: crate::state::fields::Purpose::NewGoal } => {
            crate::state::page::project::goals::open(domain, lent);
        }
        Act::FilterPlan { filter } => crate::state::page::task::filter(domain, filter),
        Act::HistoryScope { subtree } => crate::state::page::task::history(domain, subtree, out),
        Act::OpenEnded { task } => crate::state::page::task::open_ended(domain, task, out, lent),
        Act::Submit { field: crate::state::fields::Purpose::NewChat } => {
            let project = domain.shown_conversations().or(domain.shown_inbox()).unwrap_or(domain.shown_project());
            if let Some(project) = project {
                crate::state::page::conversations::start(domain, env, project, out, lent);
            }
        }
        Act::Submit { field: crate::state::fields::Purpose::Composer { task } } => {
            if let Some(id) = domain.objects.find(jig_client::contract::ObjectKey::Task(task)) {
                crate::state::confirm::open(domain, env, id, jig_client::contract::ActKind::Message, lent);
            }
        }
        Act::WriteRelease { release } => crate::state::confirm::write_release(domain, release),
        Act::MoveGoal { task, up } => crate::state::page::project::moving::open(domain, env, task, up, lent),
        Act::ProposeInstead { object } => {
            crate::state::confirm::proposal::open(domain, env, Id::from_token(object), lent);
        }
        Act::Priority { priority } => crate::state::confirm::priority(domain, priority),
        Act::ChatAbout { object } => {
            if let Some(object) = domain.borrow(lent).objects().get(Id::from_token(object)) {
                let key = object.key;
                crate::state::confirm::close(domain);
                crate::state::page::conversations::about(domain, env, key, out, lent);
            }
        }
        Act::AmendRelease { object } => crate::state::confirm::amend_release(domain, env, Id::from_token(object), lent),
        Act::CloseChat { object } => crate::state::confirm::close_chat(domain, env, Id::from_token(object), lent),
        Act::Intend { object, act } => crate::state::confirm::open(domain, env, Id::from_token(object), act, lent),
        Act::Choose { option } => crate::state::confirm::choose(domain, option),
        Act::CannotDo => crate::state::confirm::cannot(domain),
        Act::Again { again } => crate::state::confirm::again(domain, again),
        Act::Dispose { task, take_over } => crate::state::confirm::dispose(domain, task, take_over),
        Act::Change { change } => crate::state::confirm::change(domain, *change),
        Act::Confirm => {
            if domain.project_page.settings.review.is_some() {
                crate::state::page::project::owner_form::send(domain, out);
            } else {
                confirm_act(domain, out, lent);
            }
        }
        Act::LookAgain { object } => {
            crate::state::objects::answer::look_again(domain, Id::from_token(object), out, lent);
        }
        Act::Fold { object, open } => crate::state::page::task::fold(domain, object, open, lent),
        Act::Submit { .. } => {}
    }
}
fn start(
    domain: &mut Ui,
    env: &Env<Config>,
    place: Place,
    records: List<Record>,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    domain.started = true;
    domain.now = env.now;
    domain.wall = env.wall;
    match place {
        Place::Jig(address) => domain.address = address,
        Place::Elsewhere => domain.signin.application = true,
        Place::Missing => domain.page = Page::Missing,
    }
    crate::state::store::restore(domain, lent.domain_limits().text, records, out);
    domain.frame_watch = domain.hold_watch(Subject::Me, Holder::Me, out);
    domain.changed();
}
// Restored table scopes are facts; the client owns the warning and its end.
pub(crate) fn foreign_kept_requests(domain: &Ui, party: u64) -> bool {
    for (_, scope) in &domain.kept_requests {
        if *scope != party {
            return true;
        }
    }
    false
}
pub(crate) fn refresh_kept_page(
    domain: &mut Ui,
    env: &Env<Config>,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let Some(party) = domain.frame(lent).party() else {
        return;
    };
    if domain.blocked || domain.confirmed.is_none() {
        return;
    }
    if foreign_kept_requests(domain, party) {
        if domain.page != Page::SomeoneElse {
            domain.leave_handles(false, out);
            domain.page = Page::SomeoneElse;
            domain.changed();
        }
    } else if domain.page == Page::SomeoneElse {
        domain.show_work(env, out, lent);
    }
}
pub(crate) fn changed_handle(
    domain: &mut Ui,
    env: &Env<Config>,
    client: &jig_client::Client,
    handle: jig_client::Handle,
    out: &mut Queue<Request>,
) {
    let Some(held) = domain.handles.watches.get(&handle) else {
        return;
    };
    if domain.blocked {
        return;
    }
    let owner = held.owner;
    let subject = held.subject.clone();
    let Some(document) = client.document(handle) else {
        return;
    };
    if document.state == jig_client::State::Live {
        let Some(jig_client::Latest::Jig(delivery)) = document.latest else {
            return;
        };
        match owner {
            Holder::Me => {
                if let Delivery::Me(frame) = delivery {
                    frame_delivery(domain, env, *frame.clone(), out, client);
                }
            }
            Holder::Application => crate::state::application::delivery(domain, env, delivery.clone(), out, client),
            Holder::Page | Holder::Sections => {
                crate::state::page::shown::context_subject(domain, &subject);
                let contexts = crate::state::objects::contexts(
                    delivery,
                    client.domain_limits().rows.checked_mul(2).expect("two inbox groups fit"),
                );
                crate::state::objects::dropping(domain, handle, delivery);
                crate::state::page::routing::delivery(domain, env, handle, delivery.clone(), out, client);
                crate::state::objects::apply_contexts(domain, contexts);
                crate::state::objects::read_dropped(domain, out);
            }
        }
    }
    let mut behind = false;
    for (handle, held) in &domain.handles.watches {
        if held.owner == Holder::Page
            && let Some(doc) = client.document(*handle)
        {
            behind |= doc.state == jig_client::State::Behind;
        }
    }
    if let Page::Work { behind: value, .. } = &mut domain.page {
        *value = behind;
    }
    domain.changed();
}
pub(crate) fn frame_delivery(
    domain: &mut Ui,
    env: &Env<Config>,
    frame: Me,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    if !crate::tree::frame::me::fits(&frame, &env.limits, lent.domain_limits()) {
        crate::state::page::routing::refuse(domain, Limit::Objects);
        return;
    }
    domain.signin.service = match frame.party.kind {
        jig_client::contract::PartyKind::Service => true,
        jig_client::contract::PartyKind::Person => false,
    };
    let party = frame.party.number;
    let mut mismatch = false;
    for record in &domain.kept {
        let owner = match record {
            Record::Place { party, .. } | Record::Draft { party, .. } => *party,
        };
        if owner != party {
            mismatch = true;
        }
    }
    domain.confirmed = if mismatch { None } else { Some(party) };
    crate::state::page::project::goals::sync(domain, lent);
    domain.changed();
    if mismatch || foreign_kept_requests(domain, party) {
        domain.leave_handles(false, out);
        domain.page = Page::SomeoneElse;
        domain.changed();
    } else {
        match domain.page {
            Page::Starting | Page::SomeoneElse | Page::SigningIn => {
                if domain.signin.application {
                    domain.page = Page::Elsewhere;
                    domain.changed();
                } else {
                    domain.show_work(env, out, lent);
                }
            }
            Page::Work { .. } | Page::Elsewhere | Page::Missing | Page::OutOfDate => {}
        }
    }
}
pub(crate) fn ended_handle(
    domain: &mut Ui,
    handle: jig_client::Handle,
    why: jig_client::Why,
    out: &mut Queue<Request>,
) {
    let Some(held) = domain.handles.watches.remove(&handle) else {
        return;
    };
    crate::state::page::shown::context_subject(domain, &held.subject);
    if why == jig_client::Why::Limit {
        crate::state::page::routing::refuse(domain, Limit::Watches);
    }
    let final_task = why == jig_client::Why::SubjectEnded || why == jig_client::Why::Unknown;
    if final_task
        && (held.owner == Holder::Page || held.owner == Holder::Sections)
        && let Subject::Task { task } = held.subject
    {
        let key = jig_client::contract::ObjectKey::Task(task);
        let source = crate::state::objects::Source::Window { number: FINAL_TASK_READ };
        if let Some(id) = domain.objects.find(key) {
            domain
                .objects
                .get_mut(id)
                .expect("named task exists")
                .sources
                .remove(&crate::state::objects::Source::Watch(handle));
        }
        if let Some(id) = domain.objects.name(key, source) {
            domain.task_page.header = Some(id);
            let what = jig_client::contract::Read::Task { task };
            if !domain.handles.pending(&what) && domain.handles.read(source, what, out).is_none() {
                crate::state::page::routing::refuse(domain, Limit::Reads);
            }
        } else {
            crate::state::page::routing::refuse(domain, Limit::Objects);
        }
    }
    if final_task && held.owner != Holder::Me {
        crate::state::objects::ending_source(domain, handle);
    }
    if why == jig_client::Why::NotAllowed && held.owner == Holder::Sections {
        crate::state::page::shown::status(domain, None, None, true);
    }
    domain.unname(handle);
    if final_task {
        crate::state::objects::read_dropped(domain, out);
    }
    if why == jig_client::Why::NotAllowed
        && held.owner == Holder::Page
        && let Subject::Task { .. } = held.subject
    {
        domain.page = Page::Missing;
    }
    domain.changed();
}
/// Read window for the task's final document after its watch ends.
pub(crate) const FINAL_TASK_READ: u16 = 24;
pub(crate) fn signed_out(domain: &mut Ui, out: &mut Queue<Request>, lent: &jig_client::Client) {
    crate::state::signin::signed_out(domain, out, lent);
    if !domain.blocked {
        domain.project_page.clear();
    }
}
/// Fires at most one expired deadline, respecting watch terminal obligations.
pub fn fire(domain: &mut Ui, env: &Env<Config>, out: &mut Queue<Request>, lent: &jig_client::Client) {
    domain.now = env.now;
    domain.wall = env.wall;
    let Some(timer) = domain.timers.expire(env.now) else {
        return;
    };
    match timer {
        Timer::Minute => crate::state::clock::minute(domain),
        Timer::Notice(id) => crate::tree::notices::kept::dismiss(domain, id.token()),
        Timer::Save(key) => crate::state::store::fire_save(domain, key, out),
        Timer::Linger(id) => {
            if let Some(object) = domain.borrow(lent).objects().get(id) {
                match object.card {
                    crate::state::objects::Card::Leaving { until, .. } => {
                        if until <= env.now {
                            let key = object.key;
                            if domain.task_page.header == Some(id) {
                                domain.objects.get_mut(id).expect("lingering header exists").card =
                                    crate::state::objects::Card::Open;
                            } else {
                                domain.objects.remove(key);
                            }
                            domain.changed();
                        }
                    }
                    crate::state::objects::Card::Open
                    | crate::state::objects::Card::Deciding { .. }
                    | crate::state::objects::Card::NotSent { .. }
                    | crate::state::objects::Card::Refused { .. }
                    | crate::state::objects::Card::Unknown { .. } => {}
                }
            }
        }
    }
    crate::state::clock::sync(domain);
}
/// The next deadline of the client, for roots using free entry points.
#[must_use]
pub fn next_deadline(domain: &Ui) -> Option<Time> {
    domain.next_deadline()
}
/// Frees retired slots at the loop's reclaim point.
pub fn reclaim(domain: &mut Ui) {
    domain.reclaim();
}

fn confirm_act(domain: &mut Ui, out: &mut Queue<Request>, lent: &jig_client::Client) {
    if domain.project_page.form.confirmation.is_some() {
        crate::state::page::project::goals::send(domain, out, lent);
    } else {
        crate::state::confirm::send(domain, out, lent);
    }
}
fn dismiss_act(domain: &mut Ui) {
    if domain.project_page.settings.review.take().is_some() {
        domain.project_page.settings.before = None;
        domain.project_page.settings.pool_period = None;
        domain.changed();
        return;
    }
    if domain.project_page.form.confirmation.take().is_some() {
        domain.changed();
    } else if domain.confirming.is_some() {
        crate::state::confirm::close(domain);
    } else {
        crate::tree::notices::kept::dismiss_first(domain);
    }
}

fn valid_navigation(navigation: &crate::state::page::navigation::Navigation, capacity: u32) -> bool {
    if navigation.entries.capacity() > capacity {
        return false;
    }
    for (index, entry) in navigation.entries.as_slice().iter().enumerate() {
        if let Some(prefix) = navigation.entries.as_slice().get(..index)
            && prefix.contains(entry)
        {
            return false;
        }
    }
    true
}
