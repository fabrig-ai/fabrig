//! temper's pages own their handles and navigation (client/README.md, 3.1).
use crate::ChatsFilter;
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{
    Client, Handle, Latest, Link, Told, Watchable,
    contract::{Delivery, Me, Subject},
};
use skein_lib::{Env, Queue, Time};
pub mod accounts;
mod activity;
pub mod away;
pub mod builders;
pub mod calls;
pub mod changes;
pub mod chat;
/// Whether the project's settings offer writing drafts (client/README.md, 5.13).
#[must_use]
pub fn drafts_offered(root: &crate::Domain, project: u32) -> bool {
    match root.forge().ways(project) {
        Some(ways) => ways.drafts,
        None => false,
    }
}
/// Whether a chat offers "+ Draft": a configuring chat whatever its project's settings
/// say, another while they offer drafts (client/README.md, 3.5).
#[must_use]
pub fn chat_offers_draft(root: &crate::Domain, task: &jig_client::contract::Task) -> bool {
    let configuring = match root.pages.me(&root.client) {
        Some(me) => chats::configuring(me, task.project, &task.chip.executor),
        None => false,
    };
    configuring || drafts_offered(root, task.project)
}
pub mod chats;
pub mod choice;
pub mod composer;
pub mod connect_model;
pub mod delegation;
pub mod goal;
pub mod goals;
pub mod home;
pub mod inbox;
pub mod models;
pub mod needs;
pub mod report;
pub mod start_goal;
pub mod switcher;
pub mod task;
pub mod welcome;
/// The chosen playground waits for its project membership (temper/docs/design/client/README.md, 4.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaygroundChoice {
    Idle,
    Waiting,
}
#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent session and page-open flags")]
#[expect(clippy::partial_pub_fields, reason = "Home is public page state; navigation bookkeeping stays private")]
pub struct Pages {
    needs: needs::Session,
    away: Option<away::Pending>,
    overturns: away::Overturns,
    away_focus: u64,
    address: Address,
    project: Option<u32>,
    watching: bool,
    shown: u64,
    pub(crate) inbox: Option<inbox::Inbox>,
    pub home: Option<home::Home>,
    changes: Option<changes::Changes>,
    chats: Option<chats::Chats>,
    goals: Option<goals::Goals>,
    has_goals: goals::HasGoals,
    start: Option<start_goal::StartGoal>,
    chat: Option<chat::Chat>,
    task: Option<task::Task>,
    accounts: accounts::Accounts,
    builders: builders::Builders,
    models: models::Models,
    switcher: switcher::Switcher,
    pub(crate) connect_model: Option<Box<connect_model::ConnectModel>>,
    pub(crate) playground_provider: Option<u32>,
    pub(crate) playground_choice: PlaygroundChoice,
    pub(crate) playground_model: bool,
    pub(crate) choice: choice::Draft,
    forges_opened: Option<u16>,
    accounts_open: bool,
    builders_open: bool,
    welcome: Option<welcome::Welcome>,
    default_project: welcome::DefaultProject,
    adopting: Option<welcome::Adopting>,
    pub(crate) welcome_first: Option<welcome::First>,
    pub(crate) welcome_copies: Option<crate::forge::making::Copies>,
    pub(crate) welcome_decision: Option<welcome::FirstRun>,
    party: Option<u64>,
    next: u64,
    pub(crate) serial: u64,
    seen: u64,
    seeing: Option<(u64, jig_client::Key)>,
    pub(crate) confirm: Option<crate::acts::Confirm>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    OpenStart {
        draft: Option<Box<[u8]>>,
        chat: Option<u64>,
    },
    StartDraft {
        words: Box<[u8]>,
    },
    StartName {
        words: Box<[u8]>,
    },
    StartWay {
        template: u32,
    },
    StartAsks {
        always: bool,
    },
    StartCancel,
    StartGoal,
    /// Ready's Create, or its name field's Enter.
    MakeDefault,
    /// Home adopts the first repository again.
    AdoptFirst,
    /// Ask a failed copy again into its kept project.
    CopyAgain {
        org: Box<[u8]>,
        name: Box<[u8]>,
    },
    /// Ask the welcome intent’s copies left unasked after a restart.
    CopyOffered,
    AddOpen {
        start: crate::forge::forges::Start,
    },
    AddChoose {
        start: crate::forge::forges::Start,
    },
    AddWrite {
        field: home::AddField,
        words: Box<[u8]>,
    },
    AddClose,
    AddSubmit,
    /// Skip this project's To begin (client/README.md, 3.5).
    SkipToBegin,
    /// Ready's name field.
    DefaultName {
        words: Box<[u8]>,
    },
    OpenProject {
        project: u32,
    },
    Project {
        project: u32,
    },
    Choice(choice::Act),
    Needs(needs::Act),
    Away(away::Act),
    Models(models::Act),
    Switcher(switcher::Act),
    SignOut,
    Playground,
    RetryPlayground,
    Accounts(accounts::Act),
    ConnectModel(connect_model::Act),
    ChatWords {
        words: Box<[u8]>,
    },
    StartChat,
    /// A starter of Configure (temper/docs/design/client/README.md, 3.5).
    Starter {
        words: Box<[u8]>,
    },
    /// "See how it works": the stock reference beside Configure (client/README.md, 3.5).
    ShowReference,
    Suggest {
        cue: u8,
    },
    ChangeProposal {
        proposal: jig_client::contract::ProposalKey,
    },
    MessageWords {
        words: Box<[u8]>,
    },
    Mode {
        mode: chat::document::Mode,
    },
    ShowSection {
        line: u32,
    },
    AddDraft,
    Send,
    /// Open the task page's amendment editor (client/README.md, 5.11).
    Amend,
    /// Edit the amendment's specification (client/README.md, 5.11).
    AmendWords {
        words: Box<[u8]>,
    },
    /// Toggle the goal's tracking (client/README.md, 5.11).
    AmendTracked,
    /// Edit the goal's priority (client/README.md, 5.11).
    AmendPriority {
        priority: u32,
    },
    /// Drop the open amendment (client/README.md, 5.11).
    DiscardAmendment,
    /// Send the drafted amendment (client/README.md, 5.11).
    SendAmendment,
    /// Reread the task after a moved refusal (client/README.md, 5.11).
    AmendAgain,
    Pause,
    Resume,
    Stop,
    /// Close the page's task: a chat's close, or a task page's cancel (client/README.md, 5.11).
    Close,
    /// Close this configuring chat and return to a fresh Configure (client/README.md, 3.5).
    StartOver,
    DelegateFate {
        task: u64,
        fate: jig_client::shapes::Fate,
    },
    Older,
    Newer,
    Latest,
    ClosedChats,
    MoreGoals,
    GoalsFilter {
        filter: crate::GoalsFilter,
    },
    ChatFilter {
        filter: ChatsFilter,
    },
    Filter {
        filter: crate::address::Filter,
    },
    OpenResult {
        task: u64,
        through: u64,
    },
    Decide {
        object: jig_client::contract::ObjectKey,
        revision: Option<jig_client::contract::Revision>,
        act: jig_client::contract::ActKind,
    },
    CardWords {
        object: jig_client::contract::ObjectKey,
        words: Box<[u8]>,
    },
    Again {
        again: bool,
    },
    Confirm,
    Cancel,
    Go {
        address: Address,
    },
}
#[derive(Debug)]
pub enum Event {
    Started,
    Left,
    AccountKey { value: skein_lib::Secret },
    SignInAddress { value: skein_lib::Secret },
    SignInKept { source: u32, address: Box<[u8]>, refused: bool },
    Went { address: Address },
    Act { act: Act },
    Told { told: Told },
    Link { link: Link },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    OpenTab { provider: u32 },
    Client(jig_client::Event),
    Show { address: Address },
    Replace { address: Address },
}
impl Default for Pages {
    fn default() -> Self {
        Self::new()
    }
}
impl Pages {
    #[must_use]
    pub fn new() -> Self {
        Self {
            needs: needs::Session::default(),
            away: None,
            overturns: away::Overturns::new(),
            away_focus: 0,
            address: Address::Inbox { filter: crate::address::Filter::ALL },
            project: None,
            watching: false,
            shown: 0,
            inbox: None,
            home: None,
            changes: None,
            chats: None,
            goals: None,
            has_goals: goals::HasGoals::new(),
            start: None,
            chat: None,
            task: None,
            accounts: accounts::Accounts::default(),
            builders: builders::Builders::default(),
            models: models::Models::new(),
            switcher: switcher::Switcher::default(),
            connect_model: None,
            playground_provider: None,
            playground_choice: PlaygroundChoice::Idle,
            playground_model: false,
            choice: choice::Draft::default(),
            forges_opened: None,
            accounts_open: false,
            builders_open: false,
            welcome: None,
            default_project: welcome::DefaultProject::new(),
            adopting: None,
            welcome_first: None,
            welcome_copies: None,
            welcome_decision: None,
            party: None,
            next: 1,
            serial: 0,
            seen: 0,
            seeing: None,
            confirm: None,
        }
    }
    #[must_use]
    pub const fn start(&self) -> Option<&start_goal::StartGoal> {
        self.start.as_ref()
    }
    #[must_use]
    pub const fn switcher(&self) -> &switcher::Switcher {
        &self.switcher
    }
    #[must_use]
    pub fn choice(&self) -> &choice::Draft {
        &self.choice
    }
    #[must_use]
    pub fn builders(&self) -> Option<&builders::Builders> {
        if self.builders_open { Some(&self.builders) } else { None }
    }
    #[must_use]
    pub fn accounts(&self) -> Option<&accounts::Accounts> {
        if self.accounts_open && self.party.is_some() { Some(&self.accounts) } else { None }
    }
    #[must_use]
    pub const fn forges_opened(&self) -> Option<u16> {
        self.forges_opened
    }
    #[must_use]
    pub fn connect_model(&self) -> Option<&connect_model::ConnectModel> {
        if self.party.is_some() { self.connect_model.as_deref() } else { None }
    }
    pub(crate) fn sign_in_key(&self) -> Option<jig_client::Key> {
        let request = if let Some(page) = self.connect_model() { &page.request } else { &self.accounts()?.request };
        Some(request.pending.as_ref()?.key)
    }
    #[must_use]
    pub const fn welcome(&self) -> Option<&welcome::Welcome> {
        self.welcome.as_ref()
    }
    #[must_use]
    pub const fn default_project(&self) -> &welcome::DefaultProject {
        &self.default_project
    }
    #[must_use]
    pub const fn welcome_first(&self) -> Option<&welcome::First> {
        self.welcome_first.as_ref()
    }
    #[must_use]
    pub const fn welcome_copies(&self) -> Option<&crate::forge::making::Copies> {
        self.welcome_copies.as_ref()
    }
    #[must_use]
    pub const fn adopting(&self) -> Option<&welcome::Adopting> {
        self.adopting.as_ref()
    }
    pub(crate) fn decide_welcome(&mut self) {
        if let Some(welcome) = &mut self.welcome {
            welcome.decided = true;
            self.shown = self.shown.checked_add(1).expect("pages shown fits");
        }
    }
    #[must_use]
    pub fn account_name(&self) -> skein_lib::RequestKey {
        let request = match &self.connect_model {
            Some(page) => &page.request,
            None => &self.accounts.request,
        };
        if let Some(pending) = &request.pending
            && let None
            | Some(jig_client::Outcome::Unsent | jig_client::Outcome::Lost | jig_client::Outcome::Full { .. }) =
                &pending.outcome
            && let Some(name) = pending.name
        {
            return name;
        }
        account_request_name(self.serial)
    }

    #[must_use]
    pub fn accounts_document<'a>(&self, client: &'a Client) -> Option<&'a jig_client::contract::Accounts> {
        if !self.watching || self.party != Some(accounts::current_me(client)?.party.number) {
            return None;
        }
        accounts::document(client)
    }
    #[must_use]
    pub fn availability(&self, client: &Client) -> Option<bool> {
        Some(self.accounts_document(client)?.usable)
    }
    #[must_use]
    pub const fn confirmation(&self) -> Option<&crate::acts::Confirm> {
        self.confirm.as_ref()
    }
    #[must_use]
    pub const fn needs(&self) -> &needs::Session {
        &self.needs
    }
    #[must_use]
    pub const fn inbox(&self) -> Option<&inbox::Inbox> {
        self.inbox.as_ref()
    }
    #[must_use]
    pub const fn home(&self) -> Option<&home::Home> {
        self.home.as_ref()
    }
    #[must_use]
    pub const fn changes(&self) -> Option<&changes::Changes> {
        self.changes.as_ref()
    }
    #[must_use]
    pub const fn chats(&self) -> Option<&chats::Chats> {
        self.chats.as_ref()
    }
    #[must_use]
    pub const fn goals(&self) -> Option<&goals::Goals> {
        self.goals.as_ref()
    }
    pub(crate) fn has_goals(&self, client: &Client, project: u32) -> bool {
        self.project == Some(project) && self.has_goals.has(client, project)
    }
    #[must_use]
    pub const fn chat(&self) -> Option<&chat::Chat> {
        self.chat.as_ref()
    }
    #[must_use]
    pub const fn task(&self) -> Option<&task::Task> {
        self.task.as_ref()
    }
    pub(crate) fn task_mut(&mut self) -> Option<&mut task::Task> {
        self.task.as_mut()
    }
    #[must_use]
    pub fn cards(&self) -> Option<&skein_lib::List<inbox::Card>> {
        if let Some(task) = &self.task {
            if task.shape == task::Shape::Tracked {
                return Some(&self.inbox.as_ref()?.cards);
            }
            return Some(&task.cards);
        }
        if let Some(chat) = &self.chat { Some(&chat.cards) } else { Some(&self.inbox.as_ref()?.cards) }
    }
    pub(crate) fn cards_mut(&mut self) -> Option<&mut skein_lib::List<inbox::Card>> {
        if let Some(task) = &mut self.task {
            if task.shape == task::Shape::Tracked {
                return Some(&mut self.inbox.as_mut()?.cards);
            }
            return Some(&mut task.cards);
        }
        if let Some(chat) = &mut self.chat { Some(&mut chat.cards) } else { Some(&mut self.inbox.as_mut()?.cards) }
    }
    pub(crate) fn suggest(&mut self, env: &Env<Limits>, client: &Client, words: &[u8], out: &mut Queue<Request>) {
        let me = self.me(client);
        if let Some(chats) = &mut self.chats {
            chats.suggest(env, me, self.project, words, &self.choice, &mut self.serial, out);
            self.shown = self.shown.checked_add(1).expect("pages shown fits");
        }
    }
    pub(crate) fn seeded(seed: u64) -> Self {
        Self { serial: skein_lib::Rng::new(seed).next_u64(), ..Self::new() }
    }
    #[must_use]
    pub const fn project(&self) -> Option<u32> {
        self.project
    }
    #[must_use]
    pub const fn address(&self) -> &Address {
        &self.address
    }
    #[must_use]
    pub fn me<'a>(&self, client: &'a Client) -> Option<&'a Me> {
        match client.document(me_handle())?.latest? {
            Latest::Jig(Delivery::Me(me)) => Some(me),
            Latest::Jig(_) | Latest::Family(_) => None,
        }
    }
}
#[must_use]
pub fn me_handle() -> Handle {
    Handle { asker: Owner::Pages.asker(), number: 0 }
}
/// Any act on the accounts page dismisses its transient confirmation (client/README.md, 3.5).
pub(crate) fn acted(pages: &mut Pages) {
    if pages.accounts.added.take().is_some() {
        pages.accounts.added_from_connect = false;
        pages.shown = pages.shown.checked_add(1).expect("pages shown fits");
    }
}
#[expect(clippy::too_many_lines, reason = "one ordered dispatch routes page events and refreshes shared facts")]
pub fn step(pages: &mut Pages, env: &Env<Limits>, client: &Client, event: Event, out: &mut Queue<Request>) {
    if let Event::AccountKey { .. } | Event::SignInAddress { .. } = &event {
        pages.accounts.added = None;
        pages.accounts.added_from_connect = false;
    }
    match event {
        Event::Act { act: Act::OpenStart { draft, chat } } => start_goal::open(pages, client, draft, chat),
        Event::Act { act: Act::StartDraft { words } } => {
            if let Some(start) = &mut pages.start {
                start.draft_field(words);
            }
        }
        Event::Act { act: Act::Away(_) } => {}
        Event::Act { act: Act::StartName { words } } => {
            if let Some(start) = &mut pages.start {
                start.name(words);
            }
        }
        Event::Act { act: Act::StartWay { template } } => {
            let me = pages.me(client);
            if let Some(start) = &mut pages.start {
                start.way(me, template);
            }
        }
        Event::Act { act: Act::StartAsks { always } } => {
            if let Some(start) = &mut pages.start {
                start.asks(always);
            }
        }
        Event::Act { act: Act::StartCancel } => pages.start = None,
        Event::Act { act: Act::StartGoal } => {
            let me = pages.me(client);
            if let Some(start) = &mut pages.start {
                start.send(env, me, &mut pages.serial, out);
            }
        }
        Event::Started => started(pages, out),
        Event::Went { address } => {
            went(pages, env, address, out);
            if let Some(project) = address_project(&pages.address) {
                choose_project(pages, client, project);
            }
        }
        Event::SignInKept { source, address, refused } => {
            if let Some(page) = &mut pages.connect_model {
                page.request.keep_sign_in(source, address, refused);
            }
        }
        Event::SignInAddress { value } => {
            if let Some(me) = accounts::current_me(client) {
                if let Some(page) = &mut pages.connect_model {
                    page.request.paste_value(value, me.redirect_bytes);
                } else if pages.accounts_open {
                    pages.accounts.request.paste_value(value, me.redirect_bytes);
                }
            }
        }
        Event::AccountKey { value } => {
            if let Some(page) = &mut pages.connect_model {
                page.request.key(value);
            } else if pages.accounts_open
                && accounts::current_me(client).is_some()
                && pages.accounts.key_for.is_some()
                && !pages.accounts.request.busy(client)
            {
                pages.accounts.request.key(value);
            }
        }
        Event::Act { act: Act::ConnectModel(act) } => {
            let chosen = connect_model::way(pages, client);
            if pages.address == (Address::Welcome { step: crate::Step::Ready })
                && let connect_model::Act::SignIn { .. } = &act
            {
                if let Some(mut page) = pages.connect_model.take() {
                    page.leave();
                }
                pages.connect_model = Some(Box::new(connect_model::ConnectModel::default()));
            }
            if let Some(page) = &mut pages.connect_model {
                page.playground |= pages.playground_model;
                page.act(client, &pages.address, chosen, pages.playground_provider, act, &mut pages.serial, out);
            }
        }
        Event::Act { act: Act::Models(act) } => {
            if pages.models.act(client, &pages.address, act, pages.playground_provider, &mut pages.serial, out) {
                pages.shown = pages.shown.checked_add(1).expect("pages shown fits");
            }
        }
        Event::Act { act: Act::Accounts(act) } => {
            if pages.accounts_open {
                pages.accounts.act(client, act, &mut pages.serial, out);
            }
        }
        Event::Left => {
            pages.start = None;
            pages.switcher.close();
            if let Some(inbox) = pages.inbox.take() {
                pages.next = pages.next.max(inbox.leave(out));
            }
            if let Some(changes) = pages.changes.take() {
                changes.leave(out);
                pages.next = pages.next.max(changes.next());
            }
            if let Some(home) = pages.home.take() {
                home.leave(out);
            }
            if let Some(chats) = pages.chats.take() {
                pages.next = pages.next.max(chats.leave(out));
            }
            if let Some(goals) = pages.goals.take() {
                pages.next = pages.next.max(goals.leave(out));
            }
            leave_conversations(pages, out);
            pages.accounts.leave(out);
            pages.builders.leave(out);
            if let Some(mut page) = pages.connect_model.take() {
                page.leave();
            }
            pages.welcome = None;
            pages.accounts_open = false;
            pages.builders_open = false;
        }
        Event::Act { act: Act::Switcher(act) } => {
            let me = pages.me(client);
            pages.switcher.act(me, pages.project, act, &mut pages.serial, out);
        }
        Event::Act { act: Act::Needs(act) } => needs::act(pages, env, client, act, out),
        Event::Act { act: Act::Choice(act) } => choice::act(pages, client, act, out),
        Event::Act { act: Act::ChatWords { words } } => {
            if let Some(chats) = &mut pages.chats {
                chats.write(words, &env.limits);
            }
        }
        Event::Act { act: Act::StartChat } => start_chat(pages, env, client, out),
        Event::Act { act: Act::Starter { words } } => {
            if let Address::Configure { project } = pages.address {
                let me = pages.me(client);
                if let Some(chats) = &mut pages.chats {
                    chats.configure_chat(env, me, Some(project), words, &pages.choice, &mut pages.serial, out);
                }
            }
        }
        Event::Act { act: Act::ShowReference } => {
            if let Address::Configure { .. } = pages.address
                && let Some(chats) = &mut pages.chats
            {
                chats.reference = Some(chat::document::Mode::Split);
            }
        }
        Event::Act { act: Act::Suggest { .. } | Act::Playground | Act::RetryPlayground } => {
            unreachable!("suggestions are resolved by the root")
        }
        Event::Act { act: Act::MoreGoals } => {
            if let Some(goals) = &mut pages.goals {
                goals.older(out);
            }
        }
        Event::Act { act: Act::GoalsFilter { filter } } => {
            if let Address::Goals { project, .. } = pages.address {
                out.push(Request::Show { address: Address::Goals { project, filter } });
            }
        }
        Event::Act { act: Act::ClosedChats } => closed_chats(pages, out),
        Event::Act { act: Act::ChatFilter { filter } } => chat_filter(pages, filter, out),
        Event::Act { act: Act::Filter { filter } } => out.push(Request::Show { address: Address::Inbox { filter } }),
        Event::Act { act: Act::Mode { mode } } => {
            if let Some(chats) = &mut pages.chats {
                if chats.reference.is_some() {
                    chats.reference = Some(mode);
                }
            } else if let Some(chat) = &mut pages.chat {
                chat.act(env, client, Act::Mode { mode }, &mut pages.serial, out);
            }
        }
        Event::Act { act: act @ (Act::ShowSection { .. } | Act::AddDraft) } => {
            if let Some(chat) = &mut pages.chat {
                chat.act(env, client, act, &mut pages.serial, out);
            }
        }
        Event::Act {
            act:
                act @ (Act::ChangeProposal { .. }
                | Act::MessageWords { .. }
                | Act::Send
                | Act::Older
                | Act::Newer
                | Act::Latest),
        } => {
            if let Some(chat) = &mut pages.chat {
                chat.act(env, client, act, &mut pages.serial, out);
            } else if let Some(task) = &mut pages.task {
                task.act(env, client, act, &mut pages.serial, out);
            }
        }
        Event::Act {
            act:
                act @ (Act::Amend
                | Act::AmendWords { .. }
                | Act::AmendTracked
                | Act::AmendPriority { .. }
                | Act::DiscardAmendment
                | Act::SendAmendment
                | Act::AmendAgain),
        } => {
            if pages.confirm.is_none()
                && let Some(task) = &mut pages.task
            {
                task.amend(env, client, act, &mut pages.serial, out);
            }
        }
        Event::Act { act: Act::Resume } if pages.chat.is_some() => {
            pages.chat.as_mut().expect("open chat").act(env, client, Act::Resume, &mut pages.serial, out);
        }
        Event::Act { act: act @ (Act::Pause | Act::Resume) } => {
            if act == Act::Resume
                && let Some(inbox) = &pages.inbox
            {
                for card in &pages.needs.open(inbox) {
                    if card.group != jig_client::contract::InboxGroup::Questions || card.page.is_some() {
                        return;
                    }
                }
            }
            if pages.confirm.is_none()
                && let Some(page) = &mut pages.task
                && (page.shape == task::Shape::Tracked || page.shape == task::Shape::Agent)
            {
                let kind = if act == Act::Pause { task::Sending::Stop } else { task::Sending::Resume };
                page.send(env, client, kind, &mut pages.serial, out);
            }
        }
        Event::Act { act: Act::OpenResult { task, through } } => {
            open_result(pages, task, through, out);
        }
        Event::Act {
            act:
                act @ (Act::Close
                | Act::StartOver
                | Act::Stop
                | Act::DelegateFate { .. }
                | Act::Decide { .. }
                | Act::CardWords { .. }
                | Act::Again { .. }
                | Act::Confirm
                | Act::Cancel),
        } => confirmation(pages, env, client, act, out),
        Event::Act { act: Act::MakeDefault } => make_default(pages, out),
        Event::Act { act: Act::SkipToBegin } => {
            if let Address::Home { project } = pages.address
                && let Some(home) = &mut pages.home
            {
                home.skip(project, env.limits.forge, &mut pages.serial, out);
            }
        }
        Event::Act { act: Act::AdoptFirst } => {
            if let Address::Home { project } = pages.address {
                adopt(pages, env, project, out);
            }
        }
        Event::Act { act: Act::AddOpen { start } } => {
            if start != crate::forge::forges::Start::Nothing
                && let Address::Home { project } = pages.address
            {
                let rows = copy_rows(env, client);
                if home::can_add(&rows, project)
                    && let Some(home) = &mut pages.home
                    && home.add.is_none()
                    && !home.copying()
                {
                    home.add = Some(home::AddSheet::new(start));
                }
            }
        }
        Event::Act { act: Act::AddChoose { start } } => {
            if start != crate::forge::forges::Start::Nothing
                && let Some(home) = &mut pages.home
                && let Some(sheet) = &mut home.add
                && sheet.out.is_none()
            {
                sheet.start = start;
                sheet.refused = None;
            }
        }
        Event::Act { act: Act::AddWrite { field, words } } => {
            if let Some(home) = &mut pages.home
                && let Some(sheet) = &mut home.add
            {
                sheet.write(field, words, env.limits.forge);
            }
        }
        Event::Act { act: Act::AddClose } => {
            if let Some(home) = &mut pages.home
                && let Some(sheet) = &home.add
                && sheet.out.is_none()
            {
                home.add = None;
            }
        }
        Event::Act { act: Act::AddSubmit } => add_submit(pages, env, client, out),
        Event::Act { act: Act::CopyAgain { org, name } } => copy_again(pages, env, client, &org, &name, out),
        Event::Act { act: Act::CopyOffered } => {
            if let Address::Home { project } = pages.address
                && let Some(home) = &pages.home
                && !home.copying()
            {
                let rows = copy_rows(env, client);
                if let Some(copies) = crate::forge::making::copies(&rows, None) {
                    copy_all(pages, env, client, project, &copies, out);
                }
            }
        }
        Event::Act { act: Act::DefaultName { words } } => {
            let state = &mut pages.default_project;
            if state.making.is_none()
                && let Ok(words) = jig_client::contract::Text::new(words, welcome::NAME_BYTES)
            {
                state.written = Some(words.into_bytes());
                state.unnamed = false;
            }
        }
        Event::Act { act: Act::OpenProject { project } } => {
            choose_project(pages, client, project);
            out.push(Request::Show { address: Address::Chats { filter: ChatsFilter::All } });
        }
        Event::Act { act: Act::Project { project } } => {
            choose_project(pages, client, project);
            match &pages.address {
                Address::Goals { .. } => {
                    out.push(Request::Show { address: Address::Goals { project, filter: crate::GoalsFilter::All } });
                }
                Address::Home { .. } => out.push(Request::Show { address: Address::Home { project } }),
                Address::Changes { .. } => out.push(Request::Show { address: Address::Changes { project } }),
                Address::Settings { .. } => out.push(Request::Show { address: Address::Settings { project } }),
                Address::Models { .. } => out.push(Request::Show { address: Address::Models { project } }),
                Address::Builders { .. } => out.push(Request::Show { address: Address::Builders { project } }),
                Address::Repositories { .. } => out.push(Request::Show { address: Address::Repositories { project } }),
                Address::Configure { .. } => out.push(Request::Show { address: Address::Configure { project } }),
                Address::Inbox { .. }
                | Address::Chats { .. }
                | Address::NewChat { .. }
                | Address::Accounts
                | Address::Forges
                | Address::ConnectForge { .. }
                | Address::ConnectModel { .. }
                | Address::Welcome { .. }
                | Address::Chat { .. }
                | Address::Task { .. }
                | Address::Adopt { .. } => {}
            }
        }
        Event::Act { act: Act::SignOut } => out.push(Request::Client(jig_client::Event::SignOut)),
        Event::Act { act: Act::Go { address } } => {
            if address == (Address::NewChat { draft: true })
                && let Address::NewChat { .. } = pages.address
            {
                out.push(Request::Replace { address });
            } else {
                out.push(Request::Show { address });
            }
        }
        Event::Told { told: Told::Changed { handle } } if handle == me_handle() => me_changed(pages, client, out),
        Event::Link { link } => {
            if let Some(task) = &mut pages.task {
                // Successful sign-in can tell SignedOut then Starting in one step.
                // The installed credential, rather than that intermediate tell,
                // decides whether a cancelled read may be asked again.
                task.link(client.link().state);
            }
            link_changed(pages, &env.limits, link, out);
        }
        Event::Told { told: event } => {
            if !told(pages, env, client, event, out) {
                return;
            }
        }
    }
    if let Some(page) = &mut pages.connect_model {
        page.read_environment(&pages.address, client.link().state, &mut pages.next, out);
    }
    default_chosen(pages, client);
    switcher_chosen(pages, client, out);
    returned(pages, client, out);
    if let Address::ConnectModel { .. } = pages.address
        && let Some(added) = connect_model::added(pages, client)
    {
        went(pages, env, Address::Accounts, out);
        pages.accounts.added = Some(added);
        pages.accounts.added_from_connect = true;
        out.push(Request::Replace { address: Address::Accounts });
    } else if pages.accounts_open
        && pages.accounts.request_row.is_some()
        && let Some(pending) = &pages.accounts.request.pending
        && pages.accounts.announced != Some(pending.key)
        && let Some(added) = accounts::signed_in(&pages.accounts.request, client, None)
    {
        pages.accounts.announced = Some(pending.key);
        pages.accounts.added = Some(added);
        pages.accounts.added_from_connect = false;
    }
    let owner = pages.accounts_open
        && match accounts::current_me(client) {
            Some(me) => me.deployment_role == Some(jig_client::contract::HeldRole::Owner),
            None => false,
        };
    pages.accounts.configure(owner, &mut pages.next, out);
    let role = pages.builders_open
        && match accounts::current_me(client) {
            Some(me) => me.deployment_role.is_some(),
            None => false,
        };
    pages.builders.configure(role, &mut pages.next, out);
    let showing = pages.watching
        && pages.project.is_some()
        && match pages.address {
            Address::Welcome { .. } => false,
            Address::Home { .. }
            | Address::Goals { .. }
            | Address::Changes { .. }
            | Address::Inbox { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Chat { .. }
            | Address::Task { .. }
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Repositories { .. }
            | Address::Settings { .. }
            | Address::Configure { .. }
            | Address::Adopt { .. } => true,
        };
    pages.needs.configure(showing, &env.limits, out);
    let me = pages.me(client);
    pages.has_goals.configure(me, pages.project, out);
    if let Address::Home { project } = pages.address
        && pages.home.is_some()
        && pages.has_goals(client, project)
        && pages.goals.is_none()
    {
        // Inbox cards allocate handles as their reads arrive; reserve the goal handles
        // past them and move its next allocation past this page's four handles.
        if let Some(inbox) = &mut pages.inbox {
            pages.next = pages.next.max(inbox.next);
            pages.goals = Some(goals::Goals::new(pages.next));
            pages.next = pages.next.checked_add(4).expect("page handles fit");
            inbox.next = pages.next;
        }
    }
    let project = conversation_project(pages);
    if let Some(chats) = &mut pages.chats {
        chats.configure(me, project, &env.limits, out);
        if let Address::Chats { .. } = pages.address
            && let Some(project) = project
        {
            chats.open_documents(project, env.limits.client.domain.rows, out);
        }
    }
    if let Address::Goals { project, .. } | Address::Home { project } = pages.address
        && let Some(goals) = &mut pages.goals
    {
        goals.configure(me, project, client, &env.limits, out);
    }
    choice::refresh(pages, client);
    redirect_chat(pages, client, out);
    pages.shown = pages.shown.checked_add(1).expect("pages shown count fits");
}
fn told(pages: &mut Pages, env: &Env<Limits>, client: &Client, told: Told, out: &mut Queue<Request>) -> bool {
    if let Told::Answered { key, outcome } = &told
        && let Some(open) = &mut pages.confirm
        && open.sent == Some(*key)
    {
        open.sent = None;
        if seen_answered(outcome) {
            pages.confirm = None;
            if let Some(chat) = &mut pages.chat {
                chat.closing = None;
            }
        } else {
            open.failed = Some(outcome.clone());
        }
    }
    if away::told(pages, env, client, &told, out) {
        return true;
    }
    if start_goal::told(pages, env, client, &told, out) {
        return true;
    }
    if pages.has_goals.told(&told) {
        return true;
    }
    if let Some(changes) = &mut pages.changes
        && changes.told(&told, client, env.limits.forge, out)
    {
        return true;
    }
    if let Some(home) = &mut pages.home
        && home.told(&told, client, env.limits.forge, out)
    {
        return true;
    }
    if default_told(pages, env, client, &told, out) || adoption_told(pages, env, client, &told) {
        pages.shown = pages.shown.checked_add(1).expect("pages shown fits");
        return false;
    }
    if let Told::Answered { key, outcome } = &told
        && let Some((through, pending)) = pages.seeing
        && *key == pending
    {
        if seen_answered(outcome) {
            pages.seen = through;
        }
        pages.seeing = None;
        return false;
    }
    if pages.switcher.told(&told) {
        switcher_chosen(pages, client, out);
        pages.shown = pages.shown.checked_add(1).expect("pages shown fits");
        return false;
    }
    if pages.models.told(client, &told)
        || pages.needs.told(&told)
        || pages.choice.told(&told)
        || crate::acts::told(pages, &told)
    {
        pages.shown = pages.shown.checked_add(1).expect("pages shown fits");
        return false;
    }
    page_told(pages, env, client, told, out)
}

fn started(pages: &mut Pages, out: &mut Queue<Request>) {
    pages.start = None;
    pages.accounts.leave(out);
    pages.builders.leave(out);
    if let Some(mut page) = pages.connect_model.take() {
        page.leave();
    }
    pages.accounts_open = false;
    pages.builders_open = false;
    pages.welcome = None;
    pages.address = Address::Inbox { filter: crate::address::Filter::ALL };
    pages.project = None;
    pages.switcher = switcher::Switcher::default();
    if !pages.watching {
        pages.watching = true;
        out.push(Request::Client(jig_client::Event::Watch {
            handle: me_handle(),
            subject: Watchable::Jig(Subject::Me),
        }));
        out.push(Request::Client(jig_client::Event::Watch {
            handle: accounts::handle(),
            subject: Watchable::Jig(Subject::Accounts),
        }));
    }
}
pub fn fire(pages: &mut Pages, env: &Env<Limits>, _client: &Client, out: &mut Queue<Request>) {
    if needs::fire(pages, env, out) {
        pages.shown = pages.shown.checked_add(1).expect("pages shown fits");
    }
    if let Some(chats) = &mut pages.chats {
        chats.fire(env.now, out);
    }
    if let Some(chat) = &mut pages.chat {
        chat.fire(env.now, out);
    }
    if let Some(task) = &mut pages.task {
        task.fire(env.now, out);
    }
}
#[must_use]
pub fn next_deadline(pages: &Pages) -> Option<Time> {
    let task = match &pages.task {
        Some(task) => task.next_deadline(),
        None => None,
    };
    let chat = match &pages.chat {
        Some(chat) => chat.next_deadline(),
        None => None,
    };
    let chats = match &pages.chats {
        Some(chats) => chats.next_deadline(),
        None => None,
    };
    activity::earliest(activity::earliest(task, chat), activity::earliest(chats, pages.needs.next_deadline()))
}
#[must_use]
pub const fn shown(pages: &Pages) -> u64 {
    pages.shown
}

fn closed_chats(pages: &mut Pages, out: &mut Queue<Request>) {
    if let Some(chats) = &mut pages.chats {
        chats.older(out);
    }
}
fn chat_filter(pages: &Pages, filter: ChatsFilter, out: &mut Queue<Request>) {
    if pages.chats.is_some() {
        out.push(Request::Show { address: Address::Chats { filter } });
    }
}

fn conversation_project(pages: &Pages) -> Option<u32> {
    address_project(&pages.address).or(pages.project)
}

fn contains(me: &Me, project: Option<u32>) -> bool {
    for row in &me.projects {
        if Some(row.project) == project {
            return true;
        }
    }
    false
}

#[expect(clippy::too_many_lines, reason = "one navigation transition releases page reads and opens its destination")]
fn went(pages: &mut Pages, env: &Env<Limits>, address: Address, out: &mut Queue<Request>) {
    let mut opening = chat::document::Mode::Chat;
    let mut started = false;
    if let Some(chats) = &pages.chats
        && let Some((task, mode)) = chats.opening
        && address == (Address::Chat { task })
    {
        opening = mode;
        started = true;
    }
    if pages.address != address {
        pages.forges_opened = None;
        pages.start = None;
    }
    pages.needs.went(&pages.address, &address);
    pages.switcher.went(&address);
    if let (
        Address::Chats { .. } | Address::NewChat { .. },
        Address::Chats { .. } | Address::NewChat { .. },
        Some(page),
    ) = (&pages.address, &address, &mut pages.chats)
    {
        if let Address::Chats { filter } = address {
            page.filter = filter;
        }
        if let Address::NewChat { .. } = address {
            page.close_documents(out);
        }
        pages.address = address;
        return;
    }
    if let (Address::Goals { project: current, .. }, Address::Goals { project, filter }, Some(page)) =
        (&pages.address, &address, &mut pages.goals)
        && current == project
    {
        page.filter = *filter;
        pages.address = address;
        return;
    }
    if let (Address::Task { task: current }, Address::Task { task }, Some(_)) = (&pages.address, &address, &pages.task)
        && current == task
    {
        pages.address = address;
        return;
    }
    if (!pages.accounts_open
        && !pages.builders_open
        && pages.changes.is_none()
        && pages.home.is_none()
        && pages.inbox.is_none()
        && pages.chats.is_none()
        && pages.goals.is_none()
        && pages.chat.is_none()
        && pages.task.is_none()
        && pages.welcome.is_none())
        || pages.address != address
    {
        if let Some(page) = &mut pages.connect_model {
            page.changing_way(&pages.address, &address);
        }
        pages.accounts.leave(out);
        pages.builders.leave(out);
        pages.accounts_open = address == Address::Accounts;
        pages.builders_open = match &address {
            Address::Builders { .. } => true,
            Address::Inbox { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::Chat { .. }
            | Address::Task { .. }
            | Address::Goals { .. }
            | Address::Home { .. }
            | Address::Changes { .. }
            | Address::Settings { .. }
            | Address::Models { .. }
            | Address::Repositories { .. }
            | Address::Configure { .. }
            | Address::Adopt { .. } => false,
        };
        let checking_ready = if let Some(page) = &pages.connect_model {
            address == (Address::Welcome { step: crate::Step::Ready })
                && page.check != connect_model::ProgramCheck::Idle
        } else {
            false
        };
        if connect_model::open(&address) || checking_ready {
            if pages.connect_model.is_none() {
                pages.connect_model = Some(Box::new(connect_model::ConnectModel::default()));
            }
            if let Address::Welcome { .. } = address {
                let page = pages.connect_model.as_mut().expect("open flow has state");
                if page.holder != jig_client::contract::AskAccountHolder::Own {
                    page.leave();
                    **page = connect_model::ConnectModel::default();
                }
            }
        } else if let Some(mut page) = pages.connect_model.take() {
            page.leave();
        }
        pages.welcome = None;
        if let Address::Welcome { .. } = address {
            pages.welcome = Some(welcome::Welcome::new());
        }
        if let Some(changes) = pages.changes.take() {
            changes.leave(out);
            pages.next = pages.next.max(changes.next());
        }
        if let Some(home) = pages.home.take() {
            home.leave(out);
        }
        if let Some(inbox) = pages.inbox.take() {
            pages.next = pages.next.max(inbox.leave(out));
        }
        if let Some(chats) = pages.chats.take() {
            pages.next = pages.next.max(chats.leave(out));
        }
        if let Some(goals) = pages.goals.take() {
            pages.next = pages.next.max(goals.leave(out));
        }
        leave_conversations(pages, out);
        if let Address::Goals { filter, .. } = address {
            let mut page = goals::Goals::new(pages.next);
            page.filter = filter;
            pages.goals = Some(page);
        }
        if let Address::Task { task } = address {
            pages.task = Some(task::Task::new(task, pages.next, &env.limits, out));
        }
        if let Address::Chat { task } = address {
            let mut chat = chat::Chat::new(task, opening, pages.next, &env.limits, out);
            if started {
                chat.composer_focus = 1;
            }
            pages.chat = Some(chat);
        }
        if let Address::Changes { project } = address {
            pages.changes = Some(changes::Changes::new(project, &mut pages.next, env.limits.forge, out));
        }
        if let Address::Home { project } = address {
            pages.home = Some(home::Home::new(project, pages.next, env.limits.forge, out));
            pages.next = pages.next.checked_add(2).expect("page handles fit");
            pages.chats = Some(chats::Chats::new(pages.next));
            pages.next = pages.next.checked_add(2).expect("page handles fit");
            pages.inbox = Some(inbox::Inbox::new(
                crate::address::Filter { project: Some(project), kind: None, goal: None },
                pages.next,
                &env.limits,
                out,
            ));
        }
        if let Address::Settings { .. } | Address::Configure { .. } | Address::NewChat { .. } = address {
            pages.chats = Some(chats::Chats::new(pages.next));
        }
        if let Address::Chats { filter, .. } = address {
            let mut page = chats::Chats::new(pages.next);
            page.filter = filter;
            pages.chats = Some(page);
        }
        if let Address::Inbox { filter } = address {
            pages.inbox = Some(inbox::Inbox::new(filter, pages.next, &env.limits, out));
        }
        pages.choice = choice::Draft::default();
        pages.confirm = None;
    }
    if let Some(page) = &mut pages.connect_model {
        let source = match &address {
            Address::ConnectModel { way: Some(way) }
            | Address::Welcome { step: crate::Step::Model { way: Some(way) } }
                if way.kind == crate::WayKind::SignIn =>
            {
                way.source
            }
            Address::Inbox { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::Chat { .. }
            | Address::Task { .. }
            | Address::Goals { .. }
            | Address::Home { .. }
            | Address::Changes { .. }
            | Address::Settings { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Repositories { .. }
            | Address::Configure { .. }
            | Address::Adopt { .. } => None,
        };
        page.request.return_from(source);
    }
    pages.address = address;
}

pub(crate) fn seen_answered(outcome: &jig_client::Outcome) -> bool {
    match outcome {
        jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Done(_))) => true,
        jig_client::Outcome::Answered(
            jig_client::Answered::Jig(jig_client::contract::Answer::Refused { .. }) | jig_client::Answered::Family(_),
        )
        | jig_client::Outcome::NotKept
        | jig_client::Outcome::Unknown
        | jig_client::Outcome::Limit
        | jig_client::Outcome::Unconfirmed
        | jig_client::Outcome::Held
        | jig_client::Outcome::Unsent
        | jig_client::Outcome::Full { .. }
        | jig_client::Outcome::Lost => false,
    }
}

fn open_result(pages: &mut Pages, task: u64, through: u64, out: &mut Queue<Request>) {
    if let Address::Inbox { .. } = pages.address
        && pages.inbox.is_some()
        && through > pages.seen
        && pages.seeing.is_none()
    {
        pages.serial = pages.serial.checked_add(1).expect("pages keys fit");
        let key = jig_client::Key { asker: Owner::Pages.asker(), number: pages.serial };
        pages.seeing = Some((through, key));
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Seen {
                through: jig_client::contract::SeenThrough::Results { through },
            })),
        }));
    }
    out.push(Request::Show { address: Address::Task { task } });
}

pub(crate) fn account_request_name(serial: u64) -> skein_lib::RequestKey {
    let mut name = [0; 16];
    for (slot, byte) in name.iter_mut().zip(*b"accounts") {
        *slot = byte;
    }
    for (slot, byte) in name
        .get_mut(8..)
        .expect("name number fits")
        .iter_mut()
        .zip(serial.checked_add(1).expect("page keys fit").to_be_bytes())
    {
        *slot = byte;
    }

    name
}

#[expect(clippy::too_many_lines, reason = "one dispatch keeps page replies beside their request owners")]
fn page_told(pages: &mut Pages, env: &Env<Limits>, client: &Client, told: Told, out: &mut Queue<Request>) -> bool {
    if let Some(page) = &mut pages.connect_model {
        if page.environment_told(&told) {
            return true;
        }
        if let Told::Kept { key, request: jig_client::Ask::Jig(ask) } = &told
            && let jig_client::contract::Ask::AccountChange {
                change: jig_client::contract::AccountChange::FinishSignIn { .. },
                ..
            } = ask.as_ref()
            && !pages.accounts.request.paste.has_pending(*key)
        {
            return page.request.told(client, &told);
        }
        let claim = match &told {
            Told::Kept { key, request: jig_client::Ask::Jig(ask) } => {
                let row_request = match &pages.accounts.request.pending {
                    Some(pending) => pending.key == *key,
                    None => false,
                };
                !row_request && connect_model::creation(ask)
            }
            Told::Kept { .. } => false,
            Told::Answered { .. }
            | Told::Pending { .. }
            | Told::Changed { .. }
            | Told::Link { .. }
            | Told::Dropped { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => true,
        };
        if claim && page.told(client, &told, &mut pages.serial, out) {
            // Both requests observe watch changes, but answers belong to their own key.
            if let Told::Changed { .. } = told {
                pages.accounts.request.refresh(client);
            }
            return true;
        }
    }
    if pages.accounts.request.told(client, &told) {
        return true;
    }
    if let Told::Changed { handle } | Told::Ended { handle, .. } = &told
        && (*handle == needs::handle()
            || *handle == accounts::handle()
            || *handle == me_handle()
            || Some(*handle) == pages.accounts.deployment
            || pages.builders.watches(*handle))
    {
        return true;
    }
    if let Some(task) = &mut pages.task {
        let sheet = match &pages.confirm {
            Some(open) => open.object == jig_client::contract::ObjectKey::Task(task.number),
            None => false,
        };
        if !sheet && let Some(toasted) = task.toasted(&told) {
            pages.needs.say(toasted, env.now);
        }
        return task_told(task, &mut pages.inbox, env, client, told, out);
    }
    if let Some(chat) = &mut pages.chat {
        let moved = match &told {
            Told::Answered { outcome, .. } => seen_answered(outcome),
            Told::Changed { .. } | Told::Text { .. } | Told::Page { .. } => true,
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Ended { .. } => false,
        };
        if !chat.told(env.now, client, told, out) {
            return false;
        }
        if chat.restarting
            && chat.outcome_kind == Some(chat::Sending::Close)
            && let Some(outcome) = &chat.outcome
            && seen_answered(outcome)
        {
            if let Some(outcome) = &chat.outcome
                && seen_answered(outcome)
                && let Some(task) = chat.task(client)
            {
                out.push(Request::Show { address: Address::Configure { project: task.project } });
            }
            chat.restarting = false;
        }
        // Pages owns the shared key sequence, including automatic conversation reads.
        if moved {
            chat.see(&mut pages.serial, out);
        }
        return true;
    }
    if let Some(goals) = &mut pages.goals
        && goals.told(told.clone(), out)
    {
        return true;
    }
    if let Some(chats) = &mut pages.chats {
        if !chats.notes_told(env.now, client, &told, out) && !chats.told(told.clone(), &pages.address, out) {
            return match &mut pages.inbox {
                Some(inbox) => inbox.told(client, told, out),
                None => false,
            };
        }
    } else if !match &mut pages.inbox {
        Some(inbox) => inbox.told(client, told, out),
        None => false,
    } {
        return false;
    }
    true
}

fn task_told(
    task: &mut task::Task,
    inbox: &mut Option<inbox::Inbox>,
    env: &Env<Limits>,
    client: &Client,
    told: Told,
    out: &mut Queue<Request>,
) -> bool {
    // Share a monotone sequence: either child advances past the other's allocations before it runs.
    if let Some(inbox) = &*inbox {
        task.next = task.next.max(inbox.next);
    }
    let handled = task.told(env.now, client, told.clone(), out);
    let filter = if task.shape == task::Shape::Tracked {
        match task.document(client) {
            Some(document) => {
                Some(crate::address::Filter { project: Some(document.project), kind: None, goal: Some(task.number) })
            }
            None => None,
        }
    } else {
        None
    };
    if let Some(open) = &*inbox
        && Some(open.filter) != filter
    {
        task.next = task.next.max(inbox.take().expect("open inbox").leave(out));
    }
    if let Some(filter) = filter {
        if inbox.is_none() {
            *inbox = Some(inbox::Inbox::new(filter, task.next, &env.limits, out));
        }
        let inbox = inbox.as_mut().expect("tracked page inbox");
        inbox.next = inbox.next.max(task.next);
        let handled_inbox = if handled { false } else { inbox.told(client, told, out) };
        task.next = task.next.max(inbox.next);
        return handled || handled_inbox;
    }
    handled
}

#[expect(
    clippy::too_many_lines,
    reason = "confirmation dispatch lists every page act, including start goal and Home's repository sheet"
)]
fn confirmation(pages: &mut Pages, env: &Env<Limits>, client: &Client, act: Act, out: &mut Queue<Request>) {
    if let Act::Confirm = act
        && let Some(page) = &mut pages.task
        && let Some(open) = &pages.confirm
        && open.object == jig_client::contract::ObjectKey::Task(page.number)
        && open.act == jig_client::contract::ActKind::Cancel
    {
        if open.sent.is_some() {
            return;
        }
        let open = pages.confirm.as_mut().expect("open sheet");
        open.failed = None;
        open.sent = page.send(env, client, task::Sending::Cancel, &mut pages.serial, out);
        if open.sent.is_none() {
            pages.confirm = None;
        }
        return;
    }
    let closing = match &pages.confirm {
        Some(open) => open.act == jig_client::contract::ActKind::Cancel,
        None => false,
    };
    match act {
        Act::Stop if pages.chat.is_some() => {
            pages.chat.as_mut().expect("open chat").act(env, client, Act::Stop, &mut pages.serial, out);
        }
        Act::Stop => {}
        Act::Close | Act::StartOver => {
            let restarting = act == Act::StartOver;
            if let Some(chat) = &mut pages.chat {
                if restarting {
                    let Some(task) = chat.task(client) else { return };
                    let Some(document) = client.document(me_handle()) else { return };
                    let Some(me) = document.latest else { return };
                    let Latest::Jig(Delivery::Me(me)) = me else { return };
                    if chat.draft.is_none() || !chats::configuring(me, task.project, &task.chip.executor) {
                        return;
                    }
                }
                chat.prepare_close(client);
                if chat.closing.is_some() {
                    chat.restarting = restarting;
                    pages.confirm = Some(crate::acts::Confirm {
                        object: jig_client::contract::ObjectKey::Task(chat.task),
                        revision: None,
                        act: jig_client::contract::ActKind::Cancel,
                        again: None,
                        sent: None,
                        failed: None,
                    });
                }
            } else if !restarting {
                task_confirmation(pages, client, jig_client::contract::ActKind::Cancel);
            }
        }
        Act::DelegateFate { task, fate } => {
            if closing && let Some(chat) = &mut pages.chat {
                chat.delegate_fate(task, fate);
            }
        }
        Act::Confirm if closing => {
            if let Some(open) = &mut pages.confirm {
                if open.sent.is_some() {
                    return;
                }
                open.failed = None;
                open.sent = match &mut pages.chat {
                    Some(chat) => chat.close(&mut pages.serial, out),
                    None => None,
                };
                if open.sent.is_none() {
                    pages.confirm = None;
                }
            }
        }
        Act::Cancel => {
            pages.confirm = None;
            if let Some(chat) = &mut pages.chat {
                chat.closing = None;
                chat.restarting = false;
            }
        }
        act @ (Act::Decide { .. } | Act::CardWords { .. } | Act::Again { .. } | Act::Confirm) => {
            crate::acts::step(pages, env, act, out);
        }
        Act::OpenStart { .. }
        | Act::StartDraft { .. }
        | Act::StartName { .. }
        | Act::StartWay { .. }
        | Act::StartAsks { .. }
        | Act::StartCancel
        | Act::StartGoal
        | Act::Pause
        | Act::Resume
        | Act::Amend
        | Act::AmendWords { .. }
        | Act::AmendTracked
        | Act::AmendPriority { .. }
        | Act::DiscardAmendment
        | Act::SendAmendment
        | Act::AmendAgain
        | Act::Switcher(_)
        | Act::Away(_)
        | Act::Models(_)
        | Act::Needs(_)
        | Act::Choice(_)
        | Act::MakeDefault
        | Act::AdoptFirst
        | Act::CopyAgain { .. }
        | Act::CopyOffered
        | Act::AddOpen { .. }
        | Act::AddChoose { .. }
        | Act::AddWrite { .. }
        | Act::AddClose
        | Act::AddSubmit
        | Act::DefaultName { .. }
        | Act::OpenProject { .. }
        | Act::Project { .. }
        | Act::Accounts(_)
        | Act::ConnectModel(_)
        | Act::SignOut
        | Act::ChatWords { .. }
        | Act::StartChat
        | Act::ShowReference
        | Act::Starter { .. }
        | Act::SkipToBegin
        | Act::Playground
        | Act::RetryPlayground
        | Act::Suggest { .. }
        | Act::Mode { .. }
        | Act::ShowSection { .. }
        | Act::ChangeProposal { .. }
        | Act::MessageWords { .. }
        | Act::AddDraft
        | Act::Send
        | Act::Older
        | Act::Newer
        | Act::Latest
        | Act::GoalsFilter { .. }
        | Act::MoreGoals
        | Act::ChatFilter { .. }
        | Act::ClosedChats
        | Act::Filter { .. }
        | Act::OpenResult { .. }
        | Act::Go { .. } => unreachable!("only deciding acts routed here"),
    }
}

fn task_confirmation(pages: &mut Pages, client: &Client, act: jig_client::contract::ActKind) {
    if let Some(page) = &pages.task
        && let Some(task) = page.document(client)
        && chat::allowed(task, act)
    {
        pages.confirm = Some(crate::acts::Confirm {
            object: jig_client::contract::ObjectKey::Task(page.number),
            revision: None,
            act,
            again: None,
            sent: None,
            failed: None,
        });
    }
}

fn address_project(address: &Address) -> Option<u32> {
    match address {
        Address::Home { project }
        | Address::Goals { project, .. }
        | Address::Changes { project }
        | Address::Settings { project }
        | Address::Models { project }
        | Address::Builders { project }
        | Address::Repositories { project }
        | Address::Configure { project } => Some(*project),
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Adopt { .. } => None,
    }
}

fn choose_project(pages: &mut Pages, client: &Client, project: u32) {
    if let Some(me) = pages.me(client) {
        for row in &me.projects {
            if row.project == project {
                if pages.project != Some(project) {
                    pages.switcher.close();
                }
                pages.project = Some(project);
                break;
            }
        }
    }
}

/// Select the chosen playground once its record and membership arrive (temper/docs/design/client/README.md, 4.8).
pub(crate) fn playground_project(pages: &mut Pages, client: &Client, project: Option<u32>) {
    if pages.playground_choice == PlaygroundChoice::Waiting
        && let Some(project) = project
    {
        choose_project(pages, client, project);
        if pages.project == Some(project) {
            pages.playground_choice = PlaygroundChoice::Idle;
        }
    }
}

fn switcher_chosen(pages: &mut Pages, client: &Client, out: &mut Queue<Request>) {
    if let Some(project) = pages.switcher.opening {
        choose_project(pages, client, project);
        if pages.project == Some(project) {
            pages.switcher.opening = None;
            out.push(Request::Show { address: Address::Home { project } });
        }
    }
}

fn redirect_chat(pages: &Pages, client: &Client, out: &mut Queue<Request>) {
    if let Some(task) = &pages.task
        && let Some(address) = task.chat(client, pages.me(client))
    {
        out.push(Request::Show { address });
    }
}

fn me_changed(pages: &mut Pages, client: &Client, out: &mut Queue<Request>) {
    if let Some(me) = accounts::current_me(client) {
        let parked = if let Some(pending) = &pages.accounts.request.pending
            && let Some(held) = client.pending(pending.key)
        {
            held.party != me.party.number
        } else {
            false
        };
        if (pages.party.is_some() && pages.party != Some(me.party.number)) || parked {
            pages.accounts.leave(out);
            pages.builders.leave(out);
            if let Some(mut page) = pages.connect_model.take() {
                page.leave();
            }
            pages.accounts = accounts::Accounts::default();
            out.push(Request::Client(jig_client::Event::Unwatch { handle: accounts::handle() }));
            out.push(Request::Client(jig_client::Event::Watch {
                handle: accounts::handle(),
                subject: Watchable::Jig(Subject::Accounts),
            }));
        }
        if pages.party != Some(me.party.number) {
            pages.start = None;
            pages.seen = 0;
            pages.seeing = None;
            pages.choice = choice::Draft::default();
            pages.switcher = switcher::Switcher::default();
            pages.models = models::Models::new();
        }
        pages.party = Some(me.party.number);
        if connect_model::open(&pages.address) {
            if pages.connect_model.is_none() {
                pages.connect_model = Some(Box::new(connect_model::ConnectModel::default()));
            }
            let page = pages.connect_model.as_mut().expect("open flow has state");
            if page.holder == jig_client::contract::AskAccountHolder::Deployment
                && me.deployment_role != Some(jig_client::contract::HeldRole::Owner)
            {
                page.request.clear_key();
                page.holder = jig_client::contract::AskAccountHolder::Own;
            }
        }
    }
    project_changed(pages, client);
    default_chosen(pages, client);
}

pub(crate) fn project_changed(pages: &mut Pages, client: &Client) {
    if let Some(me) = pages.me(client)
        && !contains(me, pages.project)
    {
        pages.switcher.close();
        pages.project = match me.projects.get(0) {
            Some(row) => Some(row.project),
            None => None,
        };
    }
    if let Some(project) = address_project(&pages.address) {
        choose_project(pages, client, project);
    }
}
// jig ends every pending read at a credential boundary and replays none
// (jig's `client/domain/client.md`, 7): a page that has read nothing yet asks
// once more when a successful sign-in has ended its first read.
fn awaits_first_read(thread: &jig_client::thread::Thread) -> bool {
    thread.loaded == 0
        && thread.lines.rows.is_empty()
        && match &thread.reading {
            Some(jig_client::thread::Reading::Latest { known }) => *known == 0,
            Some(jig_client::thread::Reading::Older | jig_client::thread::Reading::Newer) | None => false,
        }
}

fn link_changed(pages: &mut Pages, limits: &Limits, link: Link, out: &mut Queue<Request>) {
    match link {
        Link::SignedOut | Link::OutOfDate => {
            pages.start = None;
            pages.has_goals.leave(out);
            if pages.watching {
                out.push(Request::Client(jig_client::Event::Unwatch { handle: me_handle() }));
                pages.watching = false;
                out.push(Request::Client(jig_client::Event::Unwatch { handle: accounts::handle() }));
            }
            pages.accounts.leave(out);
            pages.builders.leave(out);
            if let Some(mut page) = pages.connect_model.take() {
                page.leave();
            }
            pages.accounts = accounts::Accounts::default();
            pages.needs.configure(false, limits, out);
            pages.needs = needs::Session::default();
            pages.overturns = away::Overturns::new();
            pages.choice = choice::Draft::default();
            pages.switcher = switcher::Switcher::default();
            pages.models = models::Models::new();
            pages.party = None;
            pages.project = None;
            pages.seen = 0;
            pages.seeing = None;
        }
        Link::Starting | Link::Live | Link::Offline => {
            if !pages.watching {
                pages.watching = true;
                out.push(Request::Client(jig_client::Event::Watch {
                    handle: me_handle(),
                    subject: Watchable::Jig(Subject::Me),
                }));
                out.push(Request::Client(jig_client::Event::Watch {
                    handle: accounts::handle(),
                    subject: Watchable::Jig(Subject::Accounts),
                }));
            }
        }
    }
}

fn start_chat(pages: &mut Pages, env: &Env<Limits>, client: &Client, out: &mut Queue<Request>) {
    let me = pages.me(client);
    if let Some(chats) = &mut pages.chats {
        if let Address::Configure { project } = pages.address {
            chats.configure_chat(env, me, Some(project), chats.words.clone(), &pages.choice, &mut pages.serial, out);
        } else {
            let draft = pages.address == (Address::NewChat { draft: true });
            chats.start(env, me, pages.project, &pages.choice, draft, &mut pages.serial, out);
        }
    }
}

fn returned(pages: &mut Pages, client: &Client, out: &mut Queue<Request>) {
    let Some(page) = &mut pages.connect_model else { return };
    if !page.request.returned()
        || accounts::document(client).is_none()
        || page.request.sign_in_state(client) != accounts::SignInState::Unknown
    {
        return;
    }
    match &mut pages.address {
        Address::ConnectModel { way: Some(way) } | Address::Welcome { step: crate::Step::Model { way: Some(way) } } => {
            way.source = None;
        }
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Goals { .. }
        | Address::Home { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Configure { .. }
        | Address::Adopt { .. } => return,
    }
    page.request.return_from(None);
    pages.shown = pages.shown.checked_add(1).expect("pages shown fits");
    out.push(Request::Replace { address: pages.address.clone() });
}

fn leave_conversations(pages: &mut Pages, out: &mut Queue<Request>) {
    if let Some(chat) = pages.chat.take() {
        pages.next = pages.next.max(chat.leave(out));
    }
    if let Some(task) = pages.task.take() {
        pages.next = pages.next.max(task.leave(out));
    }
}

fn make_default(pages: &mut Pages, out: &mut Queue<Request>) {
    if pages.address != (Address::Welcome { step: crate::Step::Ready }) || pages.default_project.making.is_some() {
        return;
    }
    match pages.welcome_decision {
        Some(welcome::FirstRun::Whole | welcome::FirstRun::Unfinished { step: crate::Step::Ready }) => {}
        Some(welcome::FirstRun::Empty | welcome::FirstRun::Unfinished { .. }) | None => return,
    }
    let offered = welcome::offered(pages.welcome_first(), pages.welcome_copies());
    let Ok(asked) = jig_client::contract::Text::new(pages.default_project.asked(&offered).into(), welcome::NAME_BYTES)
    else {
        return;
    };
    if asked.bytes().is_empty() {
        pages.default_project.unnamed = true;
        return;
    }
    pages.serial = pages.serial.checked_add(1).expect("pages keys fit");
    let key = jig_client::Key { asker: Owner::Pages.asker(), number: pages.serial };
    let state = &mut pages.default_project;
    state.unnamed = false;
    state.attempt = state.attempt.checked_add(1).expect("welcome attempts fit");
    state.making = Some(key);
    state.ended = None;
    state.project = None;
    let number = skein_lib::Decimal::of(state.attempt);
    let mut name = skein_lib::Writer::new(16_usize.checked_add(number.as_bytes().len()).expect("welcome name fits"));
    name.put(b"welcome-default-").expect("measured welcome name");
    name.put(number.as_bytes()).expect("measured welcome number");
    let name = name.finish();
    out.push(Request::Client(jig_client::Event::Send {
        key,
        name: Some(crate::key::named(crate::key::TOOLS, &crate::key::Name::new(&name).expect("welcome name fits"))),
        request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::MakeProject { name: asked, policy: 1 })),
    }));
}
fn default_told(pages: &mut Pages, env: &Env<Limits>, client: &Client, told: &Told, out: &mut Queue<Request>) -> bool {
    let Some(key) = pages.default_project.making else { return false };
    let outcome = match told {
        Told::Answered { key: answered, outcome } if *answered == key => outcome,
        Told::Pending { key: pending } if *pending == key => {
            match client.pending(key) {
                Some(pending) if pending.progress == jig_client::Progress::Unknown => {}
                Some(_) | None => return true,
            }
            &jig_client::Outcome::Unknown
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
    };
    let offered = welcome::offered(pages.welcome_first(), pages.welcome_copies());
    pages.default_project.making = None;
    match outcome {
        jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Done(
            jig_client::contract::Outcome::Document(jig_client::contract::DocumentOutcome::ProjectMade { project }),
        ))) => pages.default_project.project = Some(*project),
        jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Refused {
            refusal,
            ..
        })) => {
            if *refusal == jig_client::contract::Refusal::NameTaken
                && let Some(me) = pages.me(client)
            {
                for row in &me.projects {
                    if row.name.bytes().eq_ignore_ascii_case(pages.default_project.asked(&offered)) {
                        pages.default_project.project = Some(row.project);
                        break;
                    }
                }
            }
            if pages.default_project.project.is_none() {
                pages.default_project.ended = Some(welcome::Ended::Refused(refusal.clone()));
            }
        }
        jig_client::Outcome::Answered(
            jig_client::Answered::Family(_) | jig_client::Answered::Jig(jig_client::contract::Answer::Done(_)),
        )
        | jig_client::Outcome::NotKept
        | jig_client::Outcome::Unknown
        | jig_client::Outcome::Limit
        | jig_client::Outcome::Unconfirmed
        | jig_client::Outcome::Held
        | jig_client::Outcome::Unsent
        | jig_client::Outcome::Full { .. }
        | jig_client::Outcome::Lost => pages.default_project.ended = Some(welcome::Ended::Unknown),
    }
    if let Some(project) = pages.default_project.project {
        default_chosen(pages, client);
        out.push(Request::Show { address: Address::Home { project } });
        adopt(pages, env, project, out);
        if let Some(copies) = pages.welcome_copies.clone() {
            copy_all(pages, env, client, project, &copies, out);
        }
    }
    true
}
fn default_chosen(pages: &mut Pages, client: &Client) {
    if let Some(project) = pages.default_project.project {
        choose_project(pages, client, project);
        if pages.project == Some(project) {
            pages.default_project.project = None;
        }
    }
}

/// The shared forges watch supplies Ready and Home (client/README.md, 3.5).
fn copy_rows(env: &Env<Limits>, client: &Client) -> crate::forge::forges::State {
    use crate::forge::forges;
    match client.document(forges::handle()) {
        Some(document) if document.state != jig_client::State::Ended => match document.latest {
            Some(Latest::Family(record)) => match forges::rows(record, &env.limits.forge) {
                Some(rows) => forges::State::Rows(rows),
                None => forges::State::Ended,
            },
            Some(Latest::Jig(_)) | None => forges::State::Unknown,
        },
        Some(_) | None => forges::State::Unknown,
    }
}
/// Home makes repositories into the project under its own keys (client/README.md, 3.5).
fn add_submit(pages: &mut Pages, env: &Env<Limits>, client: &Client, out: &mut Queue<Request>) {
    use crate::forge;
    let Address::Home { project } = pages.address else { return };
    let Some(forge) = home::add_forge(&copy_rows(env, client), None) else { return };
    let Some(home) = &mut pages.home else { return };
    let Some(sheet) = &mut home.add else { return };
    if sheet.out.is_some() {
        return;
    }
    let requests = match sheet.requests(forge, project) {
        Ok(requests) => requests,
        Err(refused) => {
            sheet.refused = Some(refused);
            return;
        }
    };
    sheet.refused = None;
    for ask in &requests {
        let Some(request) = forge::make_repository(ask, &env.limits.forge) else { return };
        pages.serial = pages.serial.checked_add(1).expect("pages keys fit");
        let key = jig_client::Key { asker: Owner::Pages.asker(), number: pages.serial };
        let number = skein_lib::Decimal::of(pages.serial);
        let label = skein_lib::bytes::concat(&[b"home-add-", number.as_bytes()]);
        let name = crate::key::named(crate::key::TOOLS, &crate::key::Name::new(&label).expect("home add key fits"));
        home.add.as_mut().expect("sheet is open").out = Some(key);
        home.keep_copy(key);
        out.push(Request::Client(jig_client::Event::Send { key, name: Some(name), request }));
    }
}

/// Ready and Home ask copies under the same keys (client/README.md, 3.5).
fn copy_all(
    pages: &mut Pages,
    env: &Env<Limits>,
    client: &Client,
    project: u32,
    copies: &crate::forge::making::Copies,
    out: &mut Queue<Request>,
) {
    use crate::forge::{self, listed, making};
    use temper_client_protocol_wire::v1 as w;
    let rows = copy_rows(env, client);
    let home = match listed::held(&rows, None, project) {
        listed::Held::None => true,
        listed::Held::Unknown | listed::Held::One { .. } | listed::Held::Several { .. } => false,
    };
    let mut number = 0_u64;
    for address in &copies.addresses {
        number = number.checked_add(1).expect("copy places fit");
        let mut asked = false;
        for row in &listed::rows(&rows, None) {
            if row.number == copies.forge {
                asked = making::asked(row, address);
            }
        }
        if asked {
            continue;
        }
        let Some((org, name)) = making::target(address) else { continue };
        let Ok(into) = w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project, home: home && number == 1 }) else {
            continue;
        };
        let Ok(ask) = making::ask(copies.forge, &org, &name, Some(address), Some(into)) else { continue };
        let Some(request) = forge::make_repository(&ask, &env.limits.forge) else { continue };
        pages.serial = pages.serial.checked_add(1).expect("pages keys fit");
        let key = jig_client::Key { asker: Owner::Pages.asker(), number: pages.serial };
        if let Some(home) = &mut pages.home {
            home.keep_copy(key);
        }
        out.push(Request::Client(jig_client::Event::Send { key, name: Some(forge::copy_key(number)), request }));
    }
}

/// A failed row keeps the source and adoption the person asked (client/README.md, 3.5, Home).
fn copy_again(
    pages: &mut Pages,
    env: &Env<Limits>,
    client: &Client,
    org: &[u8],
    name: &[u8],
    out: &mut Queue<Request>,
) {
    use crate::forge::{self, making};
    use temper_client_protocol_wire::v1 as w;
    let Address::Home { project } = pages.address else { return };
    let Some(home) = &pages.home else { return };
    if home.copying() {
        return;
    }
    for row in &forge::listed::rows(&copy_rows(env, client), None) {
        for making in &row.makings {
            if making.org.as_ref() != org || making.name.as_ref() != name {
                continue;
            }
            let Some(into) = &making.into else { continue };
            if into.project() != project {
                continue;
            }
            match &making.state {
                w::MakingState::Failed(_) => {}
                w::MakingState::Waiting
                | w::MakingState::Making
                | w::MakingState::Made(_)
                | w::MakingState::Adopting(_) => continue,
            }
            let from = match &making.from {
                w::MakeFrom::Copy(copy) => Some(copy.address()),
                w::MakeFrom::Empty => None,
            };
            let Ok(ask) = making::ask(row.number, org, name, from, Some(into.clone())) else { return };
            let Some(request) = forge::make_repository(&ask, &env.limits.forge) else { return };
            pages.serial = pages.serial.checked_add(1).expect("pages keys fit");
            let key = jig_client::Key { asker: Owner::Pages.asker(), number: pages.serial };
            let number = skein_lib::Decimal::of(pages.serial);
            let label = skein_lib::bytes::concat(&[b"home-copy-again-", number.as_bytes()]);
            let name = crate::key::named(crate::key::TOOLS, &crate::key::Name::new(&label).expect("copy key fits"));
            pages.home.as_mut().expect("Home is open").keep_copy(key);
            out.push(Request::Client(jig_client::Event::Send { key, name: Some(name), request }));
            return;
        }
    }
}

/// Ready and Home share the ctl adoption (temper/docs/design/client/README.md, 3.5).
fn adopt(pages: &mut Pages, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    use crate::forge;
    let Some(first) = pages.welcome_first.clone() else { return };
    let attempt = if let Some(previous) = &pages.adopting {
        if previous.out.is_some() {
            return;
        }
        match &previous.ended {
            Some(welcome::AdoptionEnded::Refused(_)) => previous.attempt.checked_add(1).expect("adoption attempts fit"),
            Some(welcome::AdoptionEnded::Unknown) | None => previous.attempt,
        }
    } else {
        0
    };
    let adoption = forge::Adoption {
        project,
        forge: first.forge,
        owner: first.org.clone(),
        name: first.name.clone(),
        prefix: b"temper/".as_slice().into(),
        role: forge::Role::Owned,
        home: true,
        landing: b"main".as_slice().into(),
        ci: true,
        checks: Box::default(),
    };
    let Some(request) = forge::ask(&adoption, &env.limits.forge) else { return };
    pages.serial = pages.serial.checked_add(1).expect("pages keys fit");
    let key = jig_client::Key { asker: Owner::Pages.asker(), number: pages.serial };
    let name = if attempt == 0 {
        forge::first_repository_key(first.forge, false)
    } else {
        forge::first_repository_again_key(first.forge, attempt)
    };
    out.push(Request::Client(jig_client::Event::Send { key, name: Some(name), request }));
    pages.adopting = Some(welcome::Adopting { project, first, out: Some(key), attempt, ended: None });
}
fn adoption_told(pages: &mut Pages, env: &Env<Limits>, client: &Client, told: &Told) -> bool {
    use crate::forge;
    let Some(adopting) = &mut pages.adopting else { return false };
    let Some(key) = adopting.out else { return false };
    let outcome = match told {
        Told::Answered { key: answered, outcome } if *answered == key => outcome,
        Told::Pending { key: pending } if *pending == key => {
            match client.pending(key) {
                Some(pending) if pending.progress == jig_client::Progress::Unknown => {}
                Some(_) | None => return true,
            }
            &jig_client::Outcome::Unknown
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
    };
    let answer = match outcome {
        jig_client::Outcome::Answered(answered) => {
            forge::answer(&env.limits, answered).unwrap_or(forge::Answer::Unknown)
        }
        jig_client::Outcome::Unknown
        | jig_client::Outcome::Limit
        | jig_client::Outcome::Held
        | jig_client::Outcome::NotKept
        | jig_client::Outcome::Unsent
        | jig_client::Outcome::Full { .. }
        | jig_client::Outcome::Lost
        | jig_client::Outcome::Unconfirmed => forge::Answer::Unknown,
    };
    adopting.out = None;
    adopting.ended = match answer {
        forge::Answer::Adopted(_) => None,
        forge::Answer::Unknown => Some(welcome::AdoptionEnded::Unknown),
        refused @ (forge::Answer::CoreRefused(_)
        | forge::Answer::ForgeRefused(_)
        | forge::Answer::EntranceRefused(_)) => Some(welcome::AdoptionEnded::Refused(refused)),
    };
    true
}

/// Settings replaces its connection page with the live row; welcome owns its steps.
pub(crate) fn forge_progress(root: &mut crate::Domain, env: &Env<Limits>, out: &mut Queue<crate::Request>) {
    let Address::ConnectForge { forge } = root.pages.address() else { return };
    let Some(number) = crate::forge::connect::following(*forge, root.forge()) else { return };
    crate::route::pages(root, env, Event::Went { address: Address::Forges }, out);
    root.pages.forges_opened = Some(number);
    out.push(crate::Request::Show { address: Address::Forges, replace: true });
}
