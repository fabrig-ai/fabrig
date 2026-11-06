//! The promised names (client/browser.md, 7.2), sorted for the gallery index.
use crate::driver::step;
use jig_client::contract as c;
use skein_lib::{Queue, Token};
use temper_client_domain::{Action, Address, Domain, Event, Request, address::Filter, pages::Act};
use temper_client_domain::{ChatsFilter, GoalsFilter};

use Then::{
    AccountRefused, Closing, Declined, Filled, Held, KeyReady, OutOfDate, Refused, Rejecting, Releasing, Renewed,
    Shown, SignInReady, SignInRefused, SignedOut, SigningIn,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum World {
    Reference,
    Solo,
    FirstRun { step: u8 },
    Bare,
}
/// Confirmation states named by client/README.md, 3.6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confirmation {
    CloseWork,
    CloseFailed,
    CloseBelow,
    CloseProposal,
    Closed,
    StartOver,
    CloseTask,
    Accept,
    Reject,
    ReleaseUncertain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Then {
    Shown,
    DraftEmpty,
    Mode(temper_client_domain::pages::chat::document::Mode),
    Hosts { role: bool, connected: bool },
    Working,
    DelegatedGoal,
    HeldChange,
    HeldTask,
    Paused,
    Filled,
    Refused,
    SignedOut,
    GoogleSignIn,
    OutOfDate,
    Closing,
    Sheet(Confirmation),
    Rejecting,
    Held,
    TaskPaused,
    Releasing,
    SigningIn,
    Paste(PasteFrame),
    AccountRefused,
    KeyReady,
    SignInReady,
    Declined,
    Renewed,
    SignInRefused,
}
/// The paste's visible frames, reached through the same public events as a browser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasteFrame {
    Waiting,
    Written,
    TooLong,
    Sending,
    Finishing,
    Address,
    State,
    TooMany,
    Ended,
    Expired,
    Lost,
    Unsent,
}
#[derive(Clone, Debug)]
pub struct Sample {
    pub name: &'static str,
    pub world: World,
    pub address: Address,
    pub then: Then,
}
impl Sample {
    /// Opens a page-owned disclosure in the static print (client/browser.md, 7).
    #[must_use]
    pub fn open_disclosure(&self) -> Option<&[u8]> {
        match self.name {
            "chat-byline-open" => Some(b"Change model"),
            "chat-work-open" => Some("Checked settings 3×, read 7 docs".as_bytes()),
            _ => None,
        }
    }
    /// Opens the rail menu through the page part in the static print (client/browser.md, 7).
    #[must_use]
    pub fn open_menu(&self) -> Option<&[u8]> {
        match self.name {
            "rail-new-menu" => Some(b"More ways to start work"),
            "rail-person-menu" => Some(b"Robin"),
            _ => None,
        }
    }
}
pub const N: usize = 276;
const CHATS: Address = Address::Chats { filter: ChatsFilter::All };
use World::{Bare, FirstRun, Reference, Solo};
#[must_use]
fn task(task: u64) -> Address {
    Address::Task { task }
}

#[must_use]
#[expect(clippy::too_many_lines, reason = "one table keeps every named gallery state in its promised order")]
pub fn samples() -> [Sample; N] {
    let mut samples = [
        Sample { name: "agent-work", world: Bare, address: Address::Task { task: 1 }, then: Shown },
        Sample { name: "chat-work-done", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-work-open", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-work-running", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-work-writing", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-work-failed", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-work-one", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-work-unknown", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-work-separated", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-work-empty", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-work-reading", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample {
            name: "builders",
            world: Solo,
            address: Address::Builders { project: 1 },
            then: Then::Hosts { role: true, connected: true },
        },
        Sample {
            name: "builders-none",
            world: Solo,
            address: Address::Builders { project: 1 },
            then: Then::Hosts { role: true, connected: false },
        },
        Sample {
            name: "builders-not-owner",
            world: Solo,
            address: Address::Builders { project: 1 },
            then: Then::Hosts { role: false, connected: false },
        },
        Sample { name: "changes", world: Reference, address: Address::Changes { project: 1 }, then: Shown },
        Sample { name: "changes-empty", world: Reference, address: Address::Changes { project: 1 }, then: Shown },
        Sample { name: "changes-one", world: Reference, address: Address::Changes { project: 1 }, then: Shown },
        Sample { name: "changes-landed-only", world: Reference, address: Address::Changes { project: 1 }, then: Shown },
        Sample {
            name: "rail-away-off",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "rail-away-on",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample { name: "home-away", world: FirstRun { step: 7 }, address: Address::Home { project: 1 }, then: Shown },
        Sample {
            name: "home-away-overturned",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-away-one",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-morning",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-last-landed",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-computed",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-last-looked-none",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-landing-none",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-landing-red",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-chats-sentence",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-three-left",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-two-left",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-accounts-claude",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-accounts-both",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-accounts-key",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-one-left",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample { name: "home-with-goals", world: Solo, address: Address::Home { project: 1 }, then: Shown },
        Sample {
            name: "home-at-work",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-needs-several",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-nothing-needs",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample { name: "home-ready", world: FirstRun { step: 7 }, address: Address::Home { project: 1 }, then: Shown },
        Sample {
            name: "home-adopting",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-copying",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-copy-failed",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample { name: "home-copied", world: FirstRun { step: 7 }, address: Address::Home { project: 1 }, then: Shown },
        Sample {
            name: "home-copy-one",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-copy-offer",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-add-repository",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-add-repository-sheet",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-add-repository-copies",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-refused",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-no-repository",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-after-skip",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-skip-refused",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-needs-under-begin",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-begin-settings-left",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-begin-one-chats",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "goals-solo",
            world: Solo,
            address: Address::Goals { project: 1, filter: GoalsFilter::All },
            then: Shown,
        },
        Sample {
            name: "goals-reference",
            world: Reference,
            address: Address::Goals { project: 1, filter: GoalsFilter::All },
            then: Shown,
        },
        Sample {
            name: "goals-none",
            world: Solo,
            address: Address::Goals { project: 1, filter: GoalsFilter::All },
            then: Shown,
        },
        Sample {
            name: "goals-needs",
            world: Solo,
            address: Address::Goals { project: 1, filter: GoalsFilter::Needs },
            then: Shown,
        },
        Sample {
            name: "goals-under-way",
            world: Solo,
            address: Address::Goals { project: 1, filter: GoalsFilter::Live },
            then: Shown,
        },
        Sample {
            name: "goals-ended",
            world: Solo,
            address: Address::Goals { project: 1, filter: GoalsFilter::Ended },
            then: Shown,
        },
        Sample {
            name: "goals-filter-empty",
            world: Solo,
            address: Address::Goals { project: 1, filter: GoalsFilter::Needs },
            then: Shown,
        },
        Sample {
            name: "goals-more",
            world: Solo,
            address: Address::Goals { project: 1, filter: GoalsFilter::All },
            then: Shown,
        },
        Sample { name: "home-github", world: FirstRun { step: 7 }, address: Address::Home { project: 1 }, then: Shown },
        Sample {
            name: "home-morning-card",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-answered",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-one-need",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "configure",
            world: FirstRun { step: 7 },
            address: Address::Configure { project: 1 },
            then: Shown,
        },
        Sample {
            name: "configure-reference",
            world: FirstRun { step: 7 },
            address: Address::Configure { project: 1 },
            then: Shown,
        },
        Sample {
            name: "configure-reference-draft",
            world: FirstRun { step: 7 },
            address: Address::Configure { project: 1 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Draft),
        },
        Sample {
            name: "configure-typing",
            world: FirstRun { step: 7 },
            address: Address::Configure { project: 1 },
            then: Shown,
        },
        Sample {
            name: "configure-sending",
            world: FirstRun { step: 7 },
            address: Address::Configure { project: 1 },
            then: Shown,
        },
        Sample {
            name: "configure-refused",
            world: FirstRun { step: 7 },
            address: Address::Configure { project: 1 },
            then: Shown,
        },
        Sample {
            name: "configure-chat-document",
            world: Solo,
            address: Address::Chat { task: 10 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Chat),
        },
        Sample {
            name: "configure-chat-document-split",
            world: Solo,
            address: Address::Chat { task: 10 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Split),
        },
        Sample {
            name: "chat-draft-chat",
            world: Solo,
            address: Address::Chat { task: 1 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Chat),
        },
        Sample {
            name: "chat-draft-split",
            world: Solo,
            address: Address::Chat { task: 1 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Split),
        },
        Sample {
            name: "chat-draft-draft",
            world: Solo,
            address: Address::Chat { task: 1 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Draft),
        },
        Sample { name: "configure-chat", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample {
            name: "configure-chat-proposal-deployment",
            world: Solo,
            address: Address::Chat { task: 10 },
            then: Shown,
        },
        Sample {
            name: "configure-chat-proposal-two-projects",
            world: Solo,
            address: Address::Chat { task: 10 },
            then: Shown,
        },
        Sample { name: "configure-chat-proposal", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "configure-chat-accepted", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "configure-chat-replaced", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "configure-chats-tag", world: Solo, address: CHATS, then: Shown },
        Sample { name: "chat-draft-empty", world: Solo, address: Address::Chat { task: 1 }, then: Then::DraftEmpty },
        Sample { name: "chat-start-goal", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-start-goal-refused", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-goal-linked", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-new-draft", world: Solo, address: Address::NewChat { draft: true }, then: Shown },
        Sample {
            name: "chat-new",
            world: FirstRun { step: 7 },
            address: Address::NewChat { draft: false },
            then: Shown,
        },
        Sample {
            name: "chat-new-no-repository",
            world: FirstRun { step: 7 },
            address: Address::NewChat { draft: false },
            then: Shown,
        },
        Sample { name: "chats-no-repository", world: FirstRun { step: 7 }, address: CHATS, then: Shown },
        Sample {
            name: "chats-drafts",
            world: Reference,
            address: Address::Chats { filter: ChatsFilter::Drafts },
            then: Shown,
        },
        Sample { name: "chats-live", world: Solo, address: Address::Chats { filter: ChatsFilter::Live }, then: Shown },
        Sample {
            name: "chats-closed",
            world: Solo,
            address: Address::Chats { filter: ChatsFilter::Closed },
            then: Shown,
        },
        Sample { name: "chats-solo", world: Solo, address: CHATS, then: Shown },
        Sample { name: "chat-solo-T10", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "inbox-solo", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample {
            name: "rail-new-menu",
            world: Solo,
            address: Address::Home { project: 1 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Chat),
        },
        Sample { name: "rail-person-menu", world: Solo, address: Address::Home { project: 1 }, then: Shown },
        Sample { name: "repositories", world: Solo, address: Address::Repositories { project: 1 }, then: Shown },
        Sample { name: "repositories-none", world: Bare, address: Address::Repositories { project: 1 }, then: Shown },
        Sample { name: "forges-empty", world: Bare, address: Address::Forges, then: Shown },
        Sample { name: "forges-setting-up", world: FirstRun { step: 6 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-fetching", world: FirstRun { step: 24 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-fetching-bytes", world: FirstRun { step: 25 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-failed", world: FirstRun { step: 9 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-replacing", world: Solo, address: Address::Forges, then: Shown },
        Sample { name: "forges-removing", world: FirstRun { step: 7 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-in-settings", world: Solo, address: Address::Forges, then: Shown },
        Sample { name: "accounts-in-settings", world: Solo, address: Address::Accounts, then: Shown },
        Sample {
            name: "needs-one",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample {
            name: "needs-saw-open",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample {
            name: "needs-reply-open",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample {
            name: "needs-second-answered",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample {
            name: "needs-later",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample {
            name: "needs-last",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample {
            name: "needs-done",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample { name: "needs-proposal", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "needs-held", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "needs-funds", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample {
            name: "needs-plain-words",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample { name: "goal-amending", world: Reference, address: task(108), then: Shown },
        Sample { name: "goal-closing", world: Reference, address: task(108), then: Shown },
        Sample { name: "goal-toast", world: Reference, address: task(108), then: Shown },
        Sample { name: "goal-sending", world: Reference, address: task(108), then: Shown },
        Sample { name: "goal-running", world: Reference, address: task(108), then: Shown },
        Sample { name: "goal-needs-funds", world: Reference, address: task(108), then: Shown },
        Sample { name: "goal-needs-question", world: Reference, address: task(118), then: Shown },
        Sample { name: "goal-planning", world: Reference, address: task(122), then: Shown },
        Sample { name: "goal-paused", world: Reference, address: task(108), then: Shown },
        Sample { name: "goal-landed", world: Reference, address: task(123), then: Shown },
        Sample { name: "goal-landed-open", world: Reference, address: task(123), then: Shown },
        Sample { name: "goal-closed", world: Reference, address: task(118), then: Shown },
        Sample { name: "goal-failed", world: Reference, address: task(122), then: Shown },
        Sample { name: "goal-no-report", world: Reference, address: task(108), then: Shown },
        Sample { name: "start-goal", world: Reference, address: CHATS, then: Shown },
        Sample { name: "start-goal-draft", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "start-goal-refused", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "settings-away", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-drafts-off", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-goals-off", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-changes-off", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-away-off", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-all-hidden", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-goals", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-chat-open", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-refused", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "task-solo-T3", world: Solo, address: task(3), then: Shown },
        Sample { name: "accounts-solo", world: Solo, address: Address::Accounts, then: Shown },
        welcome_sample("welcome-first-run-0", temper_client_domain::Step::Welcome),
        Sample {
            name: "welcome-go-on",
            world: FirstRun { step: 6 },
            address: Address::Welcome { step: temper_client_domain::Step::WelcomeAsked },
            then: Shown,
        },
        Sample {
            name: "chat-new-first-run-1",
            world: FirstRun { step: 1 },
            address: Address::NewChat { draft: false },
            then: Shown,
        },
        Sample { name: "chats-first-run-1", world: FirstRun { step: 1 }, address: CHATS, then: Shown },
        owned_sample("connect-forge-first-run-6", 6, false),
        Sample { name: "forges-first-run-7", world: FirstRun { step: 7 }, address: Address::Forges, then: Shown },
        Sample {
            name: "connect-forge-choose",
            world: FirstRun { step: 0 },
            address: Address::ConnectForge { forge: None },
            then: Shown,
        },
        welcome_sample("welcome-forge-choose", temper_client_domain::Step::Forge { forge: None }),
        welcome_sample(
            "welcome-forge-details",
            temper_client_domain::Step::Forge { forge: Some(temper_client_domain::Choice::Owned) },
        ),
        welcome_sample(
            "welcome-sheet-copies",
            temper_client_domain::Step::Forge { forge: Some(temper_client_domain::Choice::Owned) },
        ),
        welcome_sample(
            "welcome-sheet-copies-refused",
            temper_client_domain::Step::Forge { forge: Some(temper_client_domain::Choice::Owned) },
        ),
        welcome_sample(
            "welcome-sheet-nothing",
            temper_client_domain::Step::Forge { forge: Some(temper_client_domain::Choice::Owned) },
        ),
        forge_status_sample("welcome-forge-under-way", 6, Some(temper_client_domain::Choice::Forge(3))),
        forge_status_sample("welcome-forge-connected", 7, None),
        forge_status_sample("welcome-forge-github", 21, None),
        welcome_sample("welcome-model-choose", temper_client_domain::Step::Model { way: None }),
        model_forge_sample("welcome-model-forge-under-way", 6),
        model_forge_sample("welcome-model-forge-fetching", 24),
        model_forge_sample("welcome-model-forge-fetching-bytes", 25),
        model_forge_sample("welcome-model-forge-done", 7),
        model_forge_sample("welcome-model-forge-failed", 9),
        welcome_model_sample("welcome-model-claude", temper_client_domain::WayKind::Program, Shown),
        welcome_model_sample("welcome-ready-checking", temper_client_domain::WayKind::Program, Shown),
        welcome_model_sample("welcome-ready-not-yet", temper_client_domain::WayKind::Program, Shown),
        welcome_model_sample("welcome-model-key-details", temper_client_domain::WayKind::Key, Shown),
        model_forge_sample("welcome-model-connected", 7),
        welcome_model_sample("welcome-model-away", temper_client_domain::WayKind::SignIn, SigningIn),
        welcome_model_sample("welcome-model-signing-in", temper_client_domain::WayKind::SignIn, SigningIn),
        welcome_model_sample("welcome-model-signin-ready", temper_client_domain::WayKind::SignIn, SignInReady),
        welcome_model_sample("welcome-model-declined", temper_client_domain::WayKind::SignIn, Declined),
        welcome_model_sample("welcome-model-renewed", temper_client_domain::WayKind::SignIn, Renewed),
        welcome_model_sample("welcome-model-refused", temper_client_domain::WayKind::SignIn, SignInRefused),
        paste_sample("welcome-paste-waiting", PasteFrame::Waiting),
        paste_sample("welcome-paste-written", PasteFrame::Written),
        paste_sample("welcome-paste-too-long", PasteFrame::TooLong),
        paste_sample("welcome-paste-sending", PasteFrame::Sending),
        paste_sample("welcome-paste-finishing", PasteFrame::Finishing),
        paste_sample("welcome-paste-address", PasteFrame::Address),
        paste_sample("welcome-paste-state", PasteFrame::State),
        paste_sample("welcome-paste-too-many", PasteFrame::TooMany),
        paste_sample("welcome-paste-ended", PasteFrame::Ended),
        paste_sample("welcome-paste-expired", PasteFrame::Expired),
        paste_sample("welcome-paste-lost", PasteFrame::Lost),
        paste_sample("welcome-paste-unsent", PasteFrame::Unsent),
        github_sample("github-app-details", 0),
        github_sample("github-install", 20),
        github_sample("github-picker", 21),
        github_sample("github-adopted", 22),
        owned_sample("forge-details", 0, false),
        owned_sample("forge-setting-up-middle", 6, false),
        owned_sample("forge-setup-failed", 9, false),
        owned_sample("welcome-setup-failed", 9, true),
        owned_sample("welcome-ready-copies", 7, true),
        welcome_sample("welcome-ready-none", temper_client_domain::Step::Ready),
        welcome_sample("welcome-ready-signing", temper_client_domain::Step::Ready),
        owned_sample("welcome-ready-under-way", 6, true),
        owned_sample("welcome-ready-adopted", 8, true),
        Sample { name: "adoption", world: Reference, address: Address::Adopt { project: 1 }, then: Shown },
        Sample { name: "adoption-filled", world: Reference, address: Address::Adopt { project: 1 }, then: Filled },
        Sample { name: "adoption-refused", world: Reference, address: Address::Adopt { project: 1 }, then: Refused },
        Sample { name: "chats-reference", world: Reference, address: CHATS, then: Shown },
        Sample { name: "chat-reference-T1", world: Reference, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-solo-T1", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample {
            name: "chat-byline-open",
            world: FirstRun { step: 7 },
            address: Address::NewChat { draft: false },
            then: Shown,
        },
        Sample { name: "chat-paused", world: Solo, address: Address::Chat { task: 6 }, then: Then::Paused },
        Sample { name: "chat-working", world: Solo, address: Address::Chat { task: 6 }, then: Then::Working },
        Sample { name: "chat-solo-T6", world: Solo, address: Address::Chat { task: 6 }, then: Shown },
        Sample { name: "chat-held-task", world: Solo, address: Address::Chat { task: 10 }, then: Then::HeldTask },
        first_chat("chat-change-running", 2, 1),
        first_chat("chat-change-landed", 3, 1),
        Sample {
            name: "chat-change-held",
            world: FirstRun { step: 2 },
            address: Address::Chat { task: 1 },
            then: Then::HeldChange,
        },
        Sample {
            name: "chat-goal",
            world: FirstRun { step: 2 },
            address: Address::Chat { task: 1 },
            then: Then::DelegatedGoal,
        },
        first_chat("chat-first-run-2-T1", 2, 1),
        first_chat("chat-first-run-3-T1", 3, 1),
        first_chat("chat-first-run-4-T4", 4, 4),
        Sample { name: "playground-resetting", world: FirstRun { step: 10 }, address: CHATS, then: Shown },
        Sample { name: "chats-first-run-5", world: FirstRun { step: 5 }, address: CHATS, then: Shown },
        Sample {
            name: "inbox-reference",
            world: Reference,
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
        Sample { name: "accounts-states", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-key", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-removing", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-signing-in", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-program-needs-sign-in", world: Solo, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-empty", world: Bare, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-reference", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-first-run-1", world: FirstRun { step: 1 }, address: Address::Accounts, then: Shown },
        Sample { name: "google-sign-in", world: Bare, address: CHATS, then: Then::GoogleSignIn },
        Sample { name: "signed-out", world: Bare, address: CHATS, then: SignedOut },
        Sample { name: "out-of-date", world: Bare, address: CHATS, then: OutOfDate },
        Sample { name: "chats-empty", world: Bare, address: CHATS, then: Shown },
        Sample { name: "inbox-empty", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample {
            name: "chat-closing-below",
            world: Solo,
            address: Address::Chat { task: 6 },
            then: Then::Sheet(Confirmation::CloseBelow),
        },
        Sample {
            name: "chat-closing-proposal",
            world: Solo,
            address: Address::Chat { task: 6 },
            then: Then::Sheet(Confirmation::CloseProposal),
        },
        Sample {
            name: "task-releasing-uncertain",
            world: Solo,
            address: task(3),
            then: Then::Sheet(Confirmation::ReleaseUncertain),
        },
        Sample {
            name: "chat-closing-work",
            world: Solo,
            address: Address::Chat { task: 6 },
            then: Then::Sheet(Confirmation::CloseWork),
        },
        Sample {
            name: "chat-closing-failed",
            world: Solo,
            address: Address::Chat { task: 6 },
            then: Then::Sheet(Confirmation::CloseFailed),
        },
        Sample {
            name: "chat-closed",
            world: Solo,
            address: Address::Chat { task: 6 },
            then: Then::Sheet(Confirmation::Closed),
        },
        Sample {
            name: "chat-closing-start-over",
            world: Solo,
            address: Address::Chat { task: 10 },
            then: Then::Sheet(Confirmation::StartOver),
        },
        Sample { name: "task-closing", world: Solo, address: task(3), then: Then::Sheet(Confirmation::CloseTask) },
        Sample { name: "task-paused", world: Solo, address: task(3), then: Then::TaskPaused },
        Sample {
            name: "proposal-accepting",
            world: Solo,
            address: Address::Chat { task: 10 },
            then: Then::Sheet(Confirmation::Accept),
        },
        Sample {
            name: "proposal-rejecting",
            world: Solo,
            address: Address::Chat { task: 10 },
            then: Then::Sheet(Confirmation::Reject),
        },
        Sample { name: "chat-closing", world: Solo, address: Address::Chat { task: 6 }, then: Closing },
        Sample { name: "chat-rejecting", world: Solo, address: Address::Chat { task: 10 }, then: Rejecting },
        Sample { name: "task-held", world: Solo, address: task(3), then: Held },
        Sample { name: "task-releasing", world: Solo, address: task(3), then: Releasing },
        Sample {
            name: "connect-model-choose",
            world: Reference,
            address: Address::ConnectModel { way: None },
            then: Shown,
        },
        model_sample("connect-model-key-details", temper_client_domain::WayKind::Key, Shown),
        model_sample("connect-model-program-details", temper_client_domain::WayKind::Program, Shown),
        model_sample("connect-model-signing-in", temper_client_domain::WayKind::SignIn, SigningIn),
        model_sample("connect-model-key-ready", temper_client_domain::WayKind::Key, KeyReady),
        model_sample("connect-model-signin-ready", temper_client_domain::WayKind::SignIn, SignInReady),
        model_sample("connect-model-declined", temper_client_domain::WayKind::SignIn, Declined),
        model_sample("connect-model-renewed", temper_client_domain::WayKind::SignIn, Renewed),
        model_sample("connect-model-refused", temper_client_domain::WayKind::SignIn, SignInRefused),
        model_sample("connect-model-key-lost", temper_client_domain::WayKind::Key, AccountRefused),
        Sample { name: "task-reference-T28", world: Reference, address: task(28), then: Shown },
        Sample { name: "task-reference-T24", world: Reference, address: task(24), then: Shown },
        Sample { name: "task-reference-T12", world: Reference, address: task(12), then: Shown },
        Sample { name: "task-solo-T9", world: Solo, address: task(9), then: Shown },
        Sample { name: "forges-github-install", world: FirstRun { step: 20 }, address: Address::Forges, then: Shown },
        owned_sample("forge-admin-details", 23, false),
    ];
    samples.sort_by_key(|sample| sample.name);
    samples
}
fn first_chat(name: &'static str, step: u8, task: u64) -> Sample {
    Sample { name, world: World::FirstRun { step }, address: Address::Chat { task }, then: Then::Shown }
}
fn github_sample(name: &'static str, step: u8) -> Sample {
    Sample {
        name,
        world: World::FirstRun { step },
        address: Address::ConnectForge {
            forge: Some(if step == 0 {
                temper_client_domain::Choice::GitHub
            } else {
                temper_client_domain::Choice::Forge(2)
            }),
        },
        then: Then::Shown,
    }
}
fn welcome_sample(name: &'static str, step: temper_client_domain::Step) -> Sample {
    Sample { name, world: World::FirstRun { step: 0 }, address: Address::Welcome { step }, then: Then::Shown }
}
fn forge_status_sample(name: &'static str, step: u8, forge: Option<temper_client_domain::Choice>) -> Sample {
    Sample {
        name,
        world: World::FirstRun { step },
        address: Address::Welcome { step: temper_client_domain::Step::Forge { forge } },
        then: Then::Shown,
    }
}
fn model_forge_sample(name: &'static str, step: u8) -> Sample {
    Sample {
        name,
        world: World::FirstRun { step },
        address: Address::Welcome { step: temper_client_domain::Step::Model { way: None } },
        then: Then::Shown,
    }
}
fn owned_sample(name: &'static str, step: u8, ready: bool) -> Sample {
    Sample {
        name,
        world: World::FirstRun { step },
        address: if ready {
            Address::Welcome { step: temper_client_domain::Step::Ready }
        } else {
            Address::ConnectForge {
                forge: Some(if step == 0 {
                    temper_client_domain::Choice::Owned
                } else {
                    temper_client_domain::Choice::Forge(3)
                }),
            }
        },
        then: Then::Shown,
    }
}
fn paste_sample(name: &'static str, frame: PasteFrame) -> Sample {
    welcome_model_sample(name, temper_client_domain::WayKind::SignIn, Then::Paste(frame))
}
fn welcome_model_sample(name: &'static str, kind: temper_client_domain::WayKind, then: Then) -> Sample {
    let mut sample = model_sample(name, kind, then);
    if let Address::ConnectModel { way } = sample.address {
        sample.address = Address::Welcome { step: temper_client_domain::Step::Model { way } };
    }
    sample
}
fn model_sample(name: &'static str, kind: temper_client_domain::WayKind, then: Then) -> Sample {
    let way = temper_client_domain::ModelWay {
        source: None,
        provider: if kind == temper_client_domain::WayKind::Program { 17 } else { 18 },
        kind,
    };
    Sample { name, world: World::Reference, address: Address::ConnectModel { way: Some(way) }, then }
}
fn special_mockup(name: &str) -> Option<String> {
    if name.starts_with("needs-") || name == "inbox-solo" || name == "inbox-reference" || name == "inbox-empty" {
        return Some("../greenfield/needs.html".to_owned());
    }
    match name {
        "settings-away" => return Some("../greenfield/settings.html#away".to_owned()),
        "home-away" | "home-away-one" | "home-away-overturned" => return Some("../greenfield/home.html".to_owned()),
        "changes" | "changes-one" | "changes-empty" => return Some("changes.html".to_owned()),
        "changes-landed-only" => return Some("changes.html?project=hello".to_owned()),
        "forges-github-install" => return Some("forges.html?world=first-run-github&step=3&bare".to_owned()),
        "forge-admin-details" => {
            return Some("connect-forge.html?world=first-run-elsewhere&step=2&forge=3&bare".to_owned());
        }
        "forge-setup-failed" | "welcome-setup-failed" => {
            return Some("connect-forge.html?world=first-run&step=6&forge=3&bare#other-states".to_owned());
        }
        "welcome-model-claude" => {
            return Some("welcome.html?at=model&way=claude".to_owned());
        }
        "welcome-model-key-details" => {
            return Some("welcome.html?world=first-run&step=0&at=model&way=key&bare".to_owned());
        }
        "welcome-model-away"
        | "welcome-model-signing-in"
        | "welcome-model-signin-ready"
        | "welcome-model-connected" => {
            return Some("welcome.html?at=model".to_owned());
        }
        "github-app-details" => {
            return Some("connect-forge.html?world=first-run-github&step=1&forge=2&bare".to_owned());
        }
        "github-install" => return Some("connect-forge.html?world=first-run-github&step=3&forge=2&bare".to_owned()),
        "github-picker" => return Some("connect-forge.html?world=first-run-github&step=4&forge=2&bare".to_owned()),
        "github-adopted" => return Some("connect-forge.html?world=first-run-github&step=5&forge=2&bare".to_owned()),
        "welcome-forge-under-way" | "welcome-forge-connected" | "welcome-forge-github" => {
            return Some("welcome.html?at=forge".to_owned());
        }
        "welcome-forge-choose" => return Some("welcome.html?world=first-run&step=0&at=forge&bare".to_owned()),
        "welcome-sheet-copies" | "welcome-sheet-copies-refused" | "welcome-sheet-nothing" | "welcome-forge-details" => {
            return Some("welcome.html?world=first-run&step=0&at=forge&forge=3&bare".to_owned());
        }
        "welcome-model-choose" => return Some("welcome.html?world=first-run&step=0&at=model&bare".to_owned()),
        "welcome-ready-checking"
        | "welcome-ready-not-yet"
        | "welcome-ready-copies"
        | "welcome-ready-none"
        | "welcome-ready-signing" => {
            return Some("welcome.html?at=ready".to_owned());
        }
        "welcome-ready-under-way" => return Some("welcome.html?world=first-run-real&step=3&at=ready&bare".to_owned()),
        "welcome-ready-adopted" => return Some("welcome.html?world=first-run-real&step=4&at=ready&bare".to_owned()),
        _ => {}
    }
    None
}
#[must_use]
#[expect(clippy::too_many_lines, reason = "one address table maps the gallery pages to their mockups")]
pub fn mockup(sample: &Sample) -> Option<String> {
    if sample.name.starts_with("goal-") {
        let goal = match sample.name {
            "goal-needs-question" | "goal-closed" => "welcome-stall",
            "goal-planning" | "goal-failed" => "forge-cards",
            "goal-landed" | "goal-landed-open" => "pane-chat-line",
            _ => "first-run-projects",
        };
        return Some(format!("../greenfield/goal.html?g={goal}"));
    }
    if matches!(sample.name, "task-solo-T3" | "task-reference-T12") {
        return Some("../greenfield/goal.html".to_owned());
    }
    if sample.name == "chat-new-first-run-1" {
        return None;
    }
    if sample.name.starts_with("goals-") {
        return Some(if sample.world == Reference {
            "goals.html?project=fabrig".to_owned()
        } else {
            "goals.html?project=hello".to_owned()
        });
    }
    if let Then::Paste(frame) = sample.then {
        return Some(format!(
            "connect-model.html?world=first-run&step=0&way=chatgpt&paste={}&elsewhere=1&bare",
            frame.mockup()
        ));
    }
    if let Some(reference) = special_mockup(sample.name) {
        return Some(reference);
    }
    if sample.then == Then::GoogleSignIn {
        return Some("sign-in.html?world=google-sign-in&bare".to_owned());
    }
    if sample.then != Then::Shown {
        if !matches!(sample.address, Address::ConnectModel { .. }) {
            return None;
        }
        // The mockup draws these endings in the first run after ChatGPT is connected.
        // It draws no sign-in under way, key Ready, or lost key answer.
        let anchor = match sample.then {
            Then::SignInReady => "",
            Then::Declined => "#other-states-declined",
            Then::Renewed => "#other-states-renewed",
            Then::SignInRefused => "#other-states-refused",
            Then::Hosts { .. }
            | Then::Shown
            | Then::DraftEmpty
            | Then::Mode(_)
            | Then::Working
            | Then::DelegatedGoal
            | Then::HeldChange
            | Then::Paused
            | Then::HeldTask
            | Then::Filled
            | Then::Refused
            | Then::SignedOut
            | Then::GoogleSignIn
            | Then::OutOfDate
            | Then::Closing
            | Then::Sheet(_)
            | Then::Rejecting
            | Then::Held
            | Then::TaskPaused
            | Then::Releasing
            | Then::SigningIn
            | Then::Paste(_)
            | Then::AccountRefused
            | Then::KeyReady => return None,
        };
        return Some(format!("connect-model.html?world=first-run&step=5&way=chatgpt&bare{anchor}"));
    }
    let world = match sample.world {
        World::Reference => "reference".to_owned(),
        World::Solo => "solo".to_owned(),
        World::FirstRun { step } => format!("first-run&step={step}"),
        World::Bare => return None,
    };
    let (page, task) = match sample.address {
        Address::NewChat { draft } => {
            return Some(if draft { "chat.html?id=new&draft=1" } else { "chat.html?id=new" }.to_owned());
        }
        Address::Chats { .. } => ("chats", None),
        Address::Chat { task } => ("chat", Some(task)),
        Address::Inbox { .. } => ("inbox", None),
        Address::Accounts => ("accounts", None),
        Address::ConnectModel { .. } => ("connect-model", None),
        Address::Welcome { .. } => ("welcome", None),
        Address::Task { task, .. } => ("task", Some(task)),
        Address::ConnectForge { .. } => ("connect-forge", None),
        Address::Forges => ("forges", None),
        Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Home { .. }
        | Address::Settings { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Configure { .. }
        | Address::Adopt { .. } => {
            return None;
        }
    };
    let task = match task {
        Some(task) => format!("&task=T{task}"),
        None => String::new(),
    };
    let forge = match sample.address {
        Address::ConnectForge { forge: Some(temper_client_domain::Choice::Forge(number)) } => {
            format!("&forge={number}")
        }
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Forges
        | Address::ConnectForge {
            forge:
                None
                | Some(
                    temper_client_domain::Choice::GitHub
                    | temper_client_domain::Choice::Owned
                    | temper_client_domain::Choice::Managed
                    | temper_client_domain::Choice::Elsewhere,
                ),
        }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Home { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Goals { .. }
        | Address::Configure { .. }
        | Address::Adopt { .. } => String::new(),
    };
    let way = if let Address::ConnectModel { way: Some(way) } = sample.address {
        match way.kind {
            temper_client_domain::WayKind::SignIn => "&way=chatgpt",
            temper_client_domain::WayKind::Key => "&way=key",
            temper_client_domain::WayKind::Program => "&way=program",
        }
    } else {
        ""
    };
    Some(format!("{page}.html?world={world}{task}{forge}{way}&bare"))
}
fn goal_domain(sample: &Sample) -> Domain {
    let base = match sample.name {
        "goal-amending" | "goal-closing" | "goal-toast" | "goal-sending" => "goal-running",
        name => name,
    };
    let mut root = crate::worlds::goals_world(sample.world, sample.address.clone(), base);
    let act = match sample.name {
        "goal-amending" => Act::Amend,
        "goal-closing" => Act::Close,
        "goal-toast" => Act::Pause,
        "goal-sending" => {
            step(
                &mut root,
                Event::Act {
                    action: Action::Pages(Act::MessageWords {
                        words: Box::from(b"Keep the Ready page in this goal.".as_slice()),
                    }),
                },
                &mut Queue::uncapped(),
            );
            Act::Send
        }
        _ => return root,
    };
    let mut out = Queue::uncapped();
    step(&mut root, Event::Act { action: Action::Pages(act) }, &mut out);
    if sample.name == "goal-toast" {
        while let Some(request) = out.pop() {
            if let Some(jig_client::Down::Send { attempt, .. }) = crate::driver::down(request) {
                step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Answer {
                                attempt,
                                envelope: skein_lib::RequestEnvelope::Final,
                                answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                                    c::DocumentOutcome::Stopped { task: 108 },
                                )))),
                                why: None,
                            },
                        },
                    },
                    &mut Queue::uncapped(),
                );
            }
        }
    }
    root
}
#[must_use]
fn start_domain(sample: &Sample) -> Domain {
    let then = if sample.name == "start-goal" {
        Then::Shown
    } else {
        Then::Mode(temper_client_domain::pages::chat::document::Mode::Chat)
    };
    let (mut root, _) = crate::worlds::world(sample.world, sample.address.clone(), then);
    if sample.name != "start-goal" {
        let mut out = Queue::uncapped();
        step(
            &mut root,
            Event::Act {
                action: Action::Pages(Act::OpenStart {
                    draft: Some(b"docs/design/draft/away-mode.md".as_slice().into()),
                    chat: Some(1),
                }),
            },
            &mut out,
        );
        if sample.name.ends_with("-refused") {
            step(&mut root, Event::Act { action: Action::Pages(Act::StartAsks { always: true }) }, &mut out);
            step(&mut root, Event::Act { action: Action::Pages(Act::StartGoal) }, &mut out);
            step(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
            while let Some(request) = out.pop() {
                if let Some(jig_client::Down::Send { attempt, .. }) = crate::driver::down(request) {
                    step(
                        &mut root,
                        Event::Client {
                            event: jig_client::Event::Up {
                                up: jig_client::Up::Answer {
                                    attempt,
                                    envelope: skein_lib::RequestEnvelope::Final,
                                    why: None,
                                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                                        refusal: c::Refusal::NotOffered,
                                        at: None,
                                    })),
                                },
                            },
                        },
                        &mut Queue::uncapped(),
                    );
                }
            }
        }
    }
    root
}
#[must_use]
#[expect(clippy::too_many_lines, reason = "one dispatch keeps each gallery state with its domain setup")]
pub fn domain(sample: &Sample) -> Domain {
    if sample.name.starts_with("chat-work-") || sample.name == "agent-work" {
        return crate::worlds::chat_work_world(sample.name);
    }
    if sample.name == "chats-drafts" {
        return crate::worlds::drafts_world();
    }
    if sample.name == "settings" || sample.name.starts_with("settings-") {
        return crate::worlds::settings_world(sample.name);
    }
    if sample.name == "welcome-ready-copies" {
        return crate::worlds::copies_world(sample.address.clone(), sample.then);
    }
    if sample.name == "chats-no-repository" {
        let mut root = home_domain("home-no-repository");
        step(
            &mut root,
            Event::Went { place: temper_client_domain::Place::Address(sample.address.clone()) },
            &mut Queue::uncapped(),
        );
        return root;
    }
    if sample.name == "chat-new" || sample.name == "chat-new-no-repository" {
        let mut root = home_domain(if sample.name == "chat-new" { "home-ready" } else { "home-no-repository" });
        step(
            &mut root,
            Event::Went { place: temper_client_domain::Place::Address(Address::NewChat { draft: false }) },
            &mut Queue::uncapped(),
        );
        return root;
    }
    if sample.name.starts_with("start-goal") || sample.name.starts_with("chat-start-goal") {
        return start_domain(sample);
    }
    if sample.name == "chat-goal-linked" {
        return crate::worlds::linked_chat_world(sample.world, sample.address.clone());
    }
    if sample.name.starts_with("needs-") {
        return crate::needs::domain(sample.name);
    }
    if sample.name.starts_with("changes") {
        return crate::worlds::changes_world(sample.name);
    }
    if sample.name.starts_with("configure") {
        return configure_domain(sample);
    }
    if sample.name.starts_with("home-") || matches!(sample.name, "rail-away-on" | "rail-away-off") {
        return home_domain(sample.name);
    }
    if sample.name.starts_with("goals-") || sample.name.starts_with("goal-") {
        return goal_domain(sample);
    }
    let (mut root, accounts_watch) = if sample.then == Then::Sheet(Confirmation::StartOver) {
        crate::worlds::configuring_world(sample.world, sample.address.clone(), sample.then)
    } else {
        crate::worlds::world(sample.world, sample.address.clone(), sample.then)
    };
    let mut out = Queue::uncapped();
    if sample.name.starts_with("welcome-sheet-") {
        use temper_client_domain::forge::{
            Act as F,
            forges::{Act as A, Start},
        };
        let start = if sample.name == "welcome-sheet-nothing" { Start::Nothing } else { Start::Copies };
        step(&mut root, Event::Act { action: Action::Forge(F::Forges(A::Start(start))) }, &mut out);
        if sample.name != "welcome-sheet-nothing" {
            let text = if sample.name == "welcome-sheet-copies-refused" {
                b"not an address".as_slice()
            } else {
                b"github.com/acme/widget\nsource.invalid/team/tool".as_slice()
            };
            step(&mut root, Event::Act { action: Action::Forge(F::CopiesWrite { text: text.into() }) }, &mut out);
        }
        if sample.name == "welcome-sheet-copies-refused" {
            step(&mut root, Event::Act { action: Action::Forge(F::Forges(A::RunOwned)) }, &mut out);
        }
    }
    if sample.name == "welcome-ready-none" || sample.name == "welcome-ready-signing" {
        let mut doc = temper_client_domain::pages::accounts::document(root.client()).expect("Ready accounts").clone();
        doc.usable = false;
        doc.accounts.clear();
        doc.signing.clear();
        if sample.name == "welcome-ready-signing" {
            doc.signing = skein_lib::List::with_capacity(1);
            doc.signing
                .push(c::AccountSignIn { source: 43, provider: 18, holder: c::AccountHolder::Party(73) })
                .expect("one sign-in");
        }
        crate::driver::deliver(
            &mut root,
            accounts_watch.expect("Ready accounts watch"),
            c::Delivery::Accounts(Box::new(doc)),
            &mut out,
        );
    }
    if sample.name.starts_with("welcome-model-")
        || sample.name == "welcome-ready-checking"
        || sample.name == "welcome-ready-not-yet"
        || sample.name.starts_with("welcome-paste-")
    {
        let mut doc = temper_client_domain::pages::accounts::document(root.client()).expect("Model accounts").clone();
        doc.usable =
            matches!(sample.name, "welcome-model-connected" | "welcome-model-signin-ready" | "welcome-model-renewed");
        if doc.usable {
            let mut rows = skein_lib::List::with_capacity(1);
            for row in &doc.accounts {
                if row.provider == Some(18) {
                    rows.push(row.clone()).expect("ChatGPT account");
                    break;
                }
            }
            doc.accounts = rows;
        } else {
            doc.accounts.clear();
        }
        crate::driver::deliver(
            &mut root,
            accounts_watch.expect("Model accounts watch"),
            c::Delivery::Accounts(Box::new(doc)),
            &mut out,
        );
    }
    if sample.name == "welcome-model-claude"
        || sample.name == "welcome-ready-checking"
        || sample.name == "welcome-ready-not-yet"
    {
        claude(&mut root, sample.name, accounts_watch.expect("Claude accounts watch"), &mut out);
        return root;
    }
    if sample.name == "chat-draft-empty" {
        step(&mut root, Event::Act { action: Action::Pages(Act::AddDraft) }, &mut out);
    }
    match sample.then {
        Then::Mode(mode) => {
            step(&mut root, Event::Act { action: Action::Pages(Act::Mode { mode }) }, &mut out);
            if sample.name == "chat-draft-split" {
                step(&mut root, Event::Act { action: Action::Pages(Act::ShowSection { line: 17 }) }, &mut out);
            }
        }
        Then::Hosts { .. }
        | Then::DraftEmpty
        | Then::Shown
        | Then::Working
        | Then::Held
        | Then::TaskPaused
        | Then::DelegatedGoal
        | Then::HeldChange
        | Then::Paused
        | Then::HeldTask => {}
        Then::Filled | Then::Refused => adoption(&mut root, sample.then == Then::Refused, &mut out),
        Then::SignedOut | Then::GoogleSignIn => step(&mut root, Event::SignedOut, &mut out),
        Then::OutOfDate => step(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } },
            },
            &mut out,
        ),
        Then::Sheet(kind) => confirmation(&mut root, kind, &mut out),
        Then::Closing => step(&mut root, Event::Act { action: Action::Pages(Act::Close) }, &mut out),
        Then::Rejecting => step(
            &mut root,
            Event::Act {
                action: Action::Pages(Act::Decide {
                    object: c::ObjectKey::Proposal(crate::worlds::PROPOSAL),
                    revision: None,
                    act: c::ActKind::Reject,
                }),
            },
            &mut out,
        ),
        Then::Releasing => step(
            &mut root,
            Event::Act {
                action: Action::Pages(Act::Decide {
                    object: c::ObjectKey::Task(3),
                    revision: Some(c::Revision { value: 7 }),
                    act: c::ActKind::Release,
                }),
            },
            &mut out,
        ),
        Then::SigningIn
        | Then::Paste(_)
        | Then::AccountRefused
        | Then::KeyReady
        | Then::SignInReady
        | Then::Declined
        | Then::Renewed
        | Then::SignInRefused => account(&mut root, sample.then, &mut out),
    }
    if matches!(sample.name, "forges-replacing" | "forges-removing") {
        use temper_client_domain::forge::{self, forges};
        let act = if sample.name == "forges-replacing" {
            forges::Act::OpenReplace { forge: 2 }
        } else {
            forges::Act::Remove { forge: 3 }
        };
        step(&mut root, Event::Act { action: Action::Forge(forge::Act::Forges(act)) }, &mut out);
    }
    account_states(&mut root, sample, accounts_watch, &mut out);
    account_editor(&mut root, sample, &mut out);
    if sample.then == Then::Paste(PasteFrame::Expired) {
        let mut doc = temper_client_domain::pages::accounts::document(root.client()).expect("sample accounts").clone();
        doc.signing = skein_lib::List::with_capacity(1);
        doc.signing
            .push(c::AccountSignIn { source: 43, provider: 18, holder: c::AccountHolder::Party(73) })
            .expect("one signing");
        crate::driver::deliver(
            &mut root,
            accounts_watch.expect("paste sample accounts watch"),
            c::Delivery::Accounts(Box::new(doc.clone())),
            &mut out,
        );
        doc.signing.clear();
        doc.cut = false;
        crate::driver::deliver(
            &mut root,
            accounts_watch.expect("paste sample accounts watch"),
            c::Delivery::Accounts(Box::new(doc)),
            &mut out,
        );
    }
    root
}
fn account_states(root: &mut Domain, sample: &Sample, accounts_watch: Option<Token>, out: &mut Queue<Request>) {
    if sample.name == "accounts-states" || sample.name == "accounts-program-needs-sign-in" {
        let mut doc = temper_client_domain::pages::accounts::document(root.client()).expect("sample accounts").clone();
        if sample.name == "accounts-states" {
            let template = doc.accounts.get(1).expect("plan account").clone();
            let states = [
                c::AccountState::Granted,
                c::AccountState::Asking,
                c::AccountState::Later { until: None },
                c::AccountState::Later { until: Some(skein_lib::Wall::EPOCH) },
                c::AccountState::SignInAgain,
                c::AccountState::Misconfigured,
                c::AccountState::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Unreadable },
                c::AccountState::Cancelled,
                c::AccountState::Closing,
                c::AccountState::Moving,
                c::AccountState::Granted,
            ];
            doc.accounts = skein_lib::List::with_capacity(u32::try_from(states.len()).expect("sample states"));
            for (index, state) in states.into_iter().enumerate() {
                let mut row = template.clone();
                row.number = u32::try_from(index).expect("sample number").checked_add(1).expect("sample number");
                row.place = row.number;
                row.display = c::DocumentText::complete(
                    c::Text::new(
                        Box::from(format!("state {}", row.number).as_bytes()),
                        crate::driver::limits().client.domain.text,
                    )
                    .expect("sample display"),
                );
                let source = row.sources.get_mut(0).expect("plan source");
                source.number = row.number;
                source.state = state;
                source.usable = index == 0;
                doc.accounts.push(row).expect("sample state");
            }
        } else {
            let row = doc.accounts.get_mut(0).expect("program account");
            let source = row.sources.get_mut(0).expect("program source");
            source.state = c::AccountState::SignInAgain;
            source.usable = false;
            for host in doc.program_hosts.iter_mut() {
                for program in host.programs.iter_mut() {
                    program.state = Some(c::ProgramState::NotSignedIn);
                }
            }
        }
        crate::driver::deliver(
            root,
            accounts_watch.expect("accounts watch"),
            c::Delivery::Accounts(Box::new(doc)),
            out,
        );
    }
}
fn account_editor(root: &mut Domain, sample: &Sample, out: &mut Queue<Request>) {
    if matches!(sample.name, "accounts-key" | "accounts-removing" | "accounts-signing-in") {
        use temper_client_domain::pages::accounts::{Act as A, Operation, Row};
        let row = Row { holder: c::AskAccountHolder::Own, account: 2 };
        let act = if sample.name == "accounts-key" {
            A::KeyFor { row, source: None }
        } else {
            A::Submit {
                name: root.pages().account_name(),
                row,
                operation: if sample.name == "accounts-removing" {
                    Operation::RemoveAccount
                } else {
                    Operation::SignIn { provider: 18 }
                },
            }
        };
        step(root, Event::Act { action: Action::Pages(Act::Accounts(act)) }, out);
        if sample.name == "accounts-signing-in" {
            step(root, Event::Client { event: jig_client::Event::Stored }, out);
            let attempt = sent_attempt(out);
            step(
                root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Answer {
                            attempt,
                            envelope: skein_lib::RequestEnvelope::Final,
                            why: None,
                            answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                                c::DocumentOutcome::AccountSignInStarted {
                                    source: 43,
                                    address: Box::from(b"https://issuer.example/visit".as_slice()),
                                },
                            )))),
                        },
                    },
                },
                out,
            );
        }
    }
}
fn claude(root: &mut Domain, name: &str, watch: Token, out: &mut Queue<Request>) {
    use temper_client_domain::pages::{accounts::Way, connect_model::Act as A};
    if name == "welcome-model-claude" {
        return;
    }
    let mut doc = temper_client_domain::pages::accounts::document(root.client()).expect("Claude accounts").clone();
    doc.settled = false;
    crate::driver::deliver(root, watch, c::Delivery::Accounts(Box::new(doc.clone())), out);
    step(
        root,
        Event::Act {
            action: Action::Pages(Act::ConnectModel(A::SignedIn {
                name: root.pages().account_name(),
                way: Way::Program { provider: 17 },
            })),
        },
        out,
    );
    step(
        root,
        Event::Went {
            place: temper_client_domain::Place::Address(Address::Welcome { step: temper_client_domain::Step::Ready }),
        },
        out,
    );
    step(root, Event::Client { event: jig_client::Event::Stored }, out);
    let attempt = sent_attempt(out);
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                        c::DocumentOutcome::AccountProgramDeclared {
                            source: 43,
                            name: b"claude-pat".as_slice().into(),
                        },
                    )))),
                },
            },
        },
        out,
    );
    if name == "welcome-ready-not-yet" {
        step(root, Event::Client { event: jig_client::Event::Stored }, out);
        let attempt = sent_attempt(out);
        step(
            root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        why: None,
                        answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                            c::DocumentOutcome::AccountChanged,
                        )))),
                    },
                },
            },
            out,
        );
        doc.settled = true;
        crate::driver::deliver(root, watch, c::Delivery::Accounts(Box::new(doc)), out);
    }
}

fn configure_domain(sample: &Sample) -> Domain {
    if matches!(
        sample.name,
        "configure-chat-proposal"
            | "configure-chat-proposal-deployment"
            | "configure-chat-proposal-two-projects"
            | "configure-chat-accepted"
            | "configure-chat-replaced"
    ) {
        return crate::worlds::settings_chat_world(sample.name);
    }
    if matches!(
        sample.name,
        "configure-chat" | "configure-chats-tag" | "configure-chat-document" | "configure-chat-document-split"
    ) {
        let mut root = crate::worlds::configuring_world(sample.world, sample.address.clone(), sample.then).0;
        if let Then::Mode(mode) = sample.then {
            step(&mut root, Event::Act { action: Action::Pages(Act::Mode { mode }) }, &mut Queue::uncapped());
        }
        return root;
    }
    let (mut root, _) = crate::worlds::world(sample.world, sample.address.clone(), sample.then);
    if sample.name == "configure" {
        return root;
    }
    let mut out = Queue::uncapped();
    if sample.name.starts_with("configure-reference") {
        step(&mut root, Event::Act { action: Action::Pages(Act::ShowReference) }, &mut out);
        if let Then::Mode(mode) = sample.then {
            step(&mut root, Event::Act { action: Action::Pages(Act::Mode { mode }) }, &mut out);
        }
        return root;
    }
    step(
        &mut root,
        Event::Act {
            action: Action::Pages(Act::ChatWords { words: b"I work alone on small changes.".as_slice().into() }),
        },
        &mut out,
    );
    if sample.name == "configure-typing" {
        return root;
    }
    step(&mut root, Event::Act { action: Action::Pages(Act::StartChat) }, &mut out);
    if sample.name == "configure-refused" {
        step(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
        let attempt = sent_attempt(&mut out);
        step(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        why: None,
                        answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                            refusal: c::Refusal::NotOffered,
                            at: None,
                        })),
                    },
                },
            },
            &mut out,
        );
    }
    root
}

fn sent_attempt(out: &mut Queue<Request>) -> Token {
    for _ in 0..out.len() {
        if let Some(request) = out.pop()
            && let Some(down) = crate::driver::down(request)
        {
            match down {
                jig_client::Down::Send { attempt, .. } => return attempt,
                jig_client::Down::Open { .. }
                | jig_client::Down::Close { .. }
                | jig_client::Down::Continue { .. }
                | jig_client::Down::Read { .. }
                | jig_client::Down::Save { .. }
                | jig_client::Down::Erase { .. } => {}
            }
        }
    }
    unreachable!("the sample's request was stored and sent")
}
fn adoption(root: &mut Domain, refused: bool, out: &mut Queue<Request>) {
    use temper_client_domain::forge::{self, Act as A, Text as T, Typed};
    for (field, value, typed) in [
        (T::Forge, "1", Typed::Forge(Some(1))),
        (T::Owner, "acme", Typed::Bytes),
        (T::Name, "shop", Typed::Bytes),
        (T::Prefix, "temper/", Typed::Bytes),
        (T::Landing, "main", Typed::Bytes),
        (T::Checks, "1, 2", Typed::Checks(Some(Box::new([1, 2])))),
    ] {
        step(
            root,
            Event::Act { action: Action::Forge(A::Write { field, text: Box::from(value.as_bytes()), typed }) },
            out,
        );
    }
    step(root, Event::Act { action: Action::Forge(A::Choose { role: forge::Role::Owned }) }, out);
    if refused {
        step(root, Event::Act { action: Action::Forge(A::Adopt) }, out);
        step(root, Event::Client { event: jig_client::Event::Stored }, out);
        let attempt = sent_attempt(out);
        step(
            root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                            refusal: c::Refusal::NotOffered,
                            at: None,
                        })),
                        why: None,
                    },
                },
            },
            out,
        );
    }
}
fn account(root: &mut Domain, then: Then, out: &mut Queue<Request>) {
    use temper_client_domain::pages::{accounts::Way, connect_model::Act as A};
    let refused = then == Then::AccountRefused;
    let key = refused || then == Then::KeyReady;
    let way = if key { Way::Key { provider: 18 } } else { Way::SignIn { provider: 18, plan: true } };
    step(
        root,
        Event::Went {
            place: temper_client_domain::Place::Address(temper_client_domain::pages::connect_model::address(
                root.pages().address(),
                Some(temper_client_domain::pages::connect_model::model_way(way)),
            )),
        },
        out,
    );
    if !key {
        let act = A::SignIn { name: [97; 16], way };
        step(root, Event::Act { action: Action::Pages(Act::ConnectModel(act)) }, out);
    }
    if key {
        step(
            root,
            Event::AccountKey { value: skein_lib::Secret::new(Box::from(b"gallery-example-key".as_slice())) },
            out,
        );
    }
    if key {
        step(root, Event::Act { action: Action::Pages(Act::ConnectModel(A::Submit { name: [97; 16] })) }, out);
    }
    step(root, Event::Client { event: jig_client::Event::Stored }, out);
    let attempt = sent_attempt(out);
    // The tool branch's carried-key refusal: its final answer was lost, so
    // the account form explains that the key must be supplied again.
    let answer = if refused {
        None
    } else if key {
        Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::AccountKeyKept {
            source: 3,
        }))))
    } else {
        Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::AccountSignInStarted {
                source: 43,
                address: Box::from(b"https://issuer.example/visit/gallery-sign-in".as_slice()),
            },
        ))))
    };
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: if refused {
                        skein_lib::RequestEnvelope::Lost
                    } else {
                        skein_lib::RequestEnvelope::Final
                    },
                    answer,
                    why: None,
                },
            },
        },
        out,
    );
    if let Then::Paste(frame) = then {
        paste(root, frame, out);
    }
}
impl PasteFrame {
    fn mockup(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Written => "written",
            Self::TooLong => "too-long",
            Self::Sending => "sending",
            Self::Finishing => "finishing",
            Self::Address => "address",
            Self::State => "state",
            Self::TooMany => "too-many",
            Self::Ended => "not-waiting",
            Self::Expired => "expired",
            Self::Lost => "lost",
            Self::Unsent => "unsent",
        }
    }
}
fn paste(root: &mut Domain, frame: PasteFrame, out: &mut Queue<Request>) {
    use PasteFrame as F;
    use temper_client_domain::pages::connect_model::Act as A;
    if matches!(frame, F::Waiting | F::Expired) {
        return;
    }
    let value = if frame == F::TooLong {
        vec![
            b'x';
            usize::try_from(
                temper_client_domain::pages::accounts::current_me(root.client()).expect("sample me").redirect_bytes
            )
            .expect("redirect bound fits")
                + 1
        ]
    } else {
        b"gallery-private-address".to_vec()
    };
    step(root, Event::SignInAddress { value: skein_lib::Secret::new(value.into_boxed_slice()) }, out);
    if matches!(frame, F::Written | F::TooLong) {
        return;
    }
    step(root, Event::Act { action: Action::Pages(Act::ConnectModel(A::Finish { name: [98; 16] })) }, out);
    step(root, Event::Client { event: jig_client::Event::Stored }, out);
    let attempt = sent_attempt(out);
    if frame == F::Sending {
        return;
    }
    let envelope = match frame {
        F::Lost => skein_lib::RequestEnvelope::Lost,
        F::Unsent => skein_lib::RequestEnvelope::Again,
        F::Waiting
        | F::Written
        | F::TooLong
        | F::Sending
        | F::Finishing
        | F::Address
        | F::State
        | F::TooMany
        | F::Ended
        | F::Expired => skein_lib::RequestEnvelope::Final,
    };
    let answer = match frame {
        F::Finishing => Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::AccountFinishing { source: 43 },
        )))),
        F::Address | F::State | F::TooMany | F::Ended => {
            let why = match frame {
                F::Address => c::SignInRedirectWhy::Address,
                F::State => c::SignInRedirectWhy::State,
                F::TooMany => c::SignInRedirectWhy::TooMany,
                F::Ended => c::SignInRedirectWhy::NotWaiting,
                F::Waiting | F::Written | F::TooLong | F::Sending | F::Finishing | F::Expired | F::Lost | F::Unsent => {
                    unreachable!("only refusal frames select a redirect refusal")
                }
            };
            Some(jig_client::Answered::Jig(c::Answer::Refused {
                refusal: c::Refusal::SignInRedirect { why },
                at: None,
            }))
        }
        F::Waiting | F::Written | F::TooLong | F::Sending | F::Expired | F::Lost | F::Unsent => None,
    };
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Answer { attempt, envelope, answer, why: None } },
        },
        out,
    );
}

// Hand-made Home deliveries: the first-run world has no accepted settings or
// first chat, and keeps Ready's repository under its setup scenario (browser.md, 7.2).
#[expect(
    clippy::too_many_lines,
    reason = "one hand-made Home fixture answers each open and read through the public event path"
)]
fn home_domain(name: &str) -> Domain {
    use crate::driver::{deliver, down};
    use jig_client::{Down, Latest, Up, Watchable};
    use skein_lib::List;
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let away_sample = matches!(name, "rail-away-on" | "rail-away-off");
    let home_away = matches!(name, "home-away" | "home-away-one" | "home-away-overturned");
    let away_name = name;
    let status_name = if away_sample { "home-morning" } else { name };
    let name = match name {
        "rail-away-on" | "rail-away-off" => "home-morning",
        "home-away"
        | "home-away-one"
        | "home-away-overturned"
        | "home-last-landed"
        | "home-computed"
        | "home-last-looked-none"
        | "home-landing-none"
        | "home-landing-red"
        | "home-chats-sentence" => "home-nothing-needs",
        name => name,
    };
    let with_goals = name == "home-with-goals";
    let goals_source = with_goals.then(|| {
        crate::worlds::goals_world(
            Solo,
            Address::Goals { project: 1, filter: temper_client_domain::GoalsFilter::All },
            "goals-solo",
        )
    });
    let name = if with_goals { "home-at-work" } else { name };
    let (source, _) = crate::worlds::world(FirstRun { step: 7 }, Address::Settings { project: 1 }, Shown);
    let mut me = source.pages().me(source.client()).expect("sample me").clone();
    let mut projects = List::with_capacity(1);
    projects.push(me.projects.get(0).expect("Hello project").clone()).expect("one project");
    me.projects = projects;
    if home_away {
        me.projects.get_mut(0).expect("Away project").name = home_text("fabrig");
    }
    if name.starts_with("home-cop") {
        me.projects.get_mut(0).expect("Home copy project").name = home_text("Widget");
    }
    me.needs_total = match name {
        "home-at-work" | "home-needs-under-begin" | "home-one-need" => 1,
        "home-morning" | "home-needs-several" | "home-morning-card" | "home-answered" => 3,
        _ => 0,
    };
    me.needs = c::Needs { questions: me.needs_total, proposals: 0, escalations: 0, person_tasks: 0 };
    let accounts = home_accounts(
        name,
        temper_client_domain::pages::accounts::document(source.client()).expect("sample accounts").clone(),
    );
    let (chats, _) = crate::worlds::world(Solo, CHATS, Shown);
    if with_goals {
        me.projects.get_mut(0).expect("Home project").templates = chats
            .pages()
            .me(chats.client())
            .expect("Solo frame")
            .projects
            .get(0)
            .expect("Solo project")
            .templates
            .clone();
    }
    let Some(Latest::Jig(c::Delivery::Tasks(chat_rows))) =
        chats.client().document(chats.pages().chats().expect("chats").live).expect("chats watch").latest
    else {
        panic!("sample chats")
    };
    let mut root = temper_client_domain::unlinked(&crate::driver::limits(), 11);
    let mut out = Queue::uncapped();
    step(
        &mut root,
        Event::Start { address: Address::Home { project: 1 }, kept: List::with_capacity(0), with: None },
        &mut out,
    );
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: Up::Served { served: true, versions: None } },
    ] {
        step(&mut root, Event::Client { event }, &mut out);
    }
    let mut away_watch = None;
    let mut decided_watch = None;
    let everyday = matches!(
        name,
        "home-morning"
            | "home-at-work"
            | "home-needs-several"
            | "home-nothing-needs"
            | "home-morning-card"
            | "home-answered"
            | "home-one-need"
    );
    let in_project = everyday
        || matches!(
            name,
            "home-two-left"
                | "home-accounts-claude"
                | "home-accounts-both"
                | "home-accounts-key"
                | "home-one-left"
                | "home-ready"
                | "home-after-skip"
                | "home-skip-refused"
                | "home-needs-under-begin"
                | "home-begin-one-chats"
        );
    for _ in 0..256 {
        let Some(request) = out.pop() else { break };
        let Some(request) = down(request) else { continue };
        match request {
            Down::Open { watch, subject: Watchable::Jig(subject) } => {
                let value = match subject {
                    c::Subject::Me => c::Delivery::Me(Box::new(me.clone())),
                    c::Subject::Accounts => c::Delivery::Accounts(Box::new(accounts.clone())),
                    c::Subject::Settings => {
                        let mut lines = List::with_capacity(1);
                        lines
                            .push(c::SettingLine {
                                setting: c::Setting::Rules,
                                name: None,
                                follows: if everyday
                                    || matches!(name, "home-one-left" | "home-ready" | "home-begin-one-chats")
                                {
                                    c::SettingFollows::Change
                                } else {
                                    c::SettingFollows::Default
                                },
                                last: None,
                                application: None,
                            })
                            .expect("settings line");
                        c::Delivery::Settings(Box::new(c::SettingsDocument {
                            newest: 0,
                            settings: lines,
                            held: List::with_capacity(0),
                            pending: List::with_capacity(0),
                        }))
                    }
                    c::Subject::Tasks { query, .. } if query.tracked == Some(true) => {
                        let mut rows = List::with_capacity(0);
                        if let Some(source) = &goals_source {
                            rows = source
                                .pages()
                                .goals()
                                .expect("Goals source")
                                .live(source.client())
                                .expect("live goals")
                                .rows
                                .clone();
                        }
                        c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false }))
                    }
                    c::Subject::Tasks { .. } => {
                        let mut rows = List::with_capacity(1);
                        if matches!(name, "home-ready" | "home-begin-settings-left" | "home-needs-under-begin") {
                            let mut row = chat_rows.rows.get(0).expect("sample chat").clone();
                            row.project = 1;
                            rows.push(row).expect("one chat");
                        }
                        if everyday {
                            rows = home_chat_rows(name, chat_rows.rows.get(0).expect("sample chat"), false);
                        }
                        c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false }))
                    }
                    c::Subject::Inbox { .. } => c::Delivery::Inbox(Box::new(temper_fixtures::client_documents::inbox(
                        home_needs(name),
                        me.needs_total,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    ))),
                    _ => continue,
                };
                deliver(&mut root, watch, value, &mut out);
            }
            Down::Open { watch, subject: Watchable::Family { record, .. } } => {
                let rows = home_forges(name, in_project);
                let bytes = match wire::decode_watching(&record.bytes, &w::CEILINGS).expect("Home watch").watch() {
                    w::Watch::Dismissed => {
                        let mut dismissed = List::with_capacity(1);
                        if name == "home-after-skip" {
                            dismissed
                                .push(w::PersonFact::ToBegin(
                                    w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project: 1 }).expect("Home skip"),
                                ))
                                .expect("one skip");
                        }
                        wire::dismissals(
                            &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed })
                                .expect("Home dismissals"),
                        )
                    }
                    w::Watch::Forges => wire::forges(
                        &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows }).expect("Home forges"),
                    ),
                    w::Watch::Status(_) => match if home_away {
                        Some(crate::worlds::home_away_status())
                    } else if status_name == "home-morning" {
                        Some(morning_status())
                    } else {
                        crate::worlds::home_status(status_name)
                    } {
                        Some(status) => wire::project_status(&status),
                        None => continue,
                    },
                    w::Watch::Away if home_away => crate::worlds::home_away_record(away_name),
                    w::Watch::Decided(_) if home_away => {
                        decided_watch = Some(watch);
                        continue;
                    }
                    w::Watch::Away if away_sample => {
                        away_watch = Some(watch);
                        crate::worlds::away_record(away_name == "rail-away-off")
                    }
                    w::Watch::Away
                    | w::Watch::Decided(_)
                    | w::Watch::Change(_)
                    | w::Watch::Landing(_)
                    | w::Watch::Landings(_)
                    | w::Watch::Changes(_) => {
                        continue;
                    }
                };
                step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes,
                                },
                            },
                        },
                    },
                    &mut out,
                );
            }
            Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { query, .. }) } => {
                if query.tracked == Some(true)
                    && let Some(source) = &goals_source
                {
                    crate::driver::answer(
                        &mut root,
                        read,
                        c::ReadValue::Tasks(Box::new(c::TaskPage {
                            live: List::with_capacity(0),
                            live_after: None,
                            ended: c::EndedGroup::Rows {
                                rows: List::from_boxed(source.pages().goals().expect("Goals source").ended().into()),
                                after: None,
                                cut: false,
                            },
                        })),
                        &mut out,
                    );
                    continue;
                }
                crate::driver::answer(
                    &mut root,
                    read,
                    c::ReadValue::Tasks(Box::new(c::TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: c::EndedGroup::Rows {
                            rows: if everyday && query.tracked != Some(true) {
                                home_chat_rows(name, chat_rows.rows.get(0).expect("sample chat"), true)
                            } else {
                                List::with_capacity(0)
                            },
                            after: None,
                            cut: false,
                        },
                    })),
                    &mut out,
                );
            }
            Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) } => {
                let rows = if group == c::InboxReadGroup::Needs { home_needs(name) } else { List::with_capacity(0) };
                crate::driver::answer(
                    &mut root,
                    read,
                    c::ReadValue::Inbox(Box::new(c::InboxPage {
                        rows,
                        cut: false,
                        before: None,
                        after: None,
                        selection: goals_source.as_ref().map(|source| {
                            let page = source.pages().goals().expect("Goals source");
                            let live = page.live(source.client()).expect("live goals");
                            let mut goals = List::uncapped();
                            for line in live.rows.iter().chain(page.ended()) {
                                if page.needs_person(line.chip.task) {
                                    goals.push(line.chip.task).expect("goal selection");
                                }
                            }
                            List::from_boxed(Box::new([c::InboxSelection {
                                cursor: c::InboxCursor::Number { number: 1 },
                                goals,
                            }]))
                        }),
                    })),
                    &mut out,
                );
            }
            Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { task, message }) } => {
                let mut chip = chat_rows.rows.get(0).expect("sample chat").chip.clone();
                chip.task = task;
                chip.title = home_text(if matches!(name, "home-morning-card" | "home-answered") {
                    match task {
                        3 => "welcome-stall",
                        2 => "forge-cards",
                        _ => "Add a README",
                    }
                } else {
                    home_title(task)
                });
                crate::driver::answer(
                    &mut root,
                    read,
                    c::ReadValue::Question(Box::new(c::Question {
                        task: chip,
                        from: None,
                        message,
                        words: home_text(home_question(name, task)),
                        since: home_since(name, 4),
                        acts: {
                            let mut offers = List::with_capacity(1);
                            offers
                                .push(c::Offered {
                                    act: c::ActKind::Answer,
                                    admission: c::Admission::Allowed,
                                    effect: None,
                                })
                                .expect("answer offer");
                            offers
                        },
                        ending: None,
                    })),
                    &mut out,
                );
            }
            Down::Read {
                read,
                what: jig_client::Readable::Jig(c::Read::Notes { scope: c::NoteScope::Task { project, task }, .. }),
            } if goals_source.is_some() => {
                let mut rows = List::uncapped();
                if let Some(report) =
                    goals_source.as_ref().expect("Goals source").pages().goals().expect("Goals page").report(task)
                {
                    rows.push(c::Note {
                        number: 1,
                        revision: c::Revision { value: 1 },
                        scope: c::NoteScope::Task { project, task },
                        description: home_text(&format!("report: {}", String::from_utf8_lossy(&report.headline))),
                        body: home_text(""),
                        writer: c::NoteAuthor::Task { task, attempt: 1 },
                        tasks: List::with_capacity(0),
                        changed: None,
                        acts: List::with_capacity(0),
                        cut: false,
                    })
                    .expect("goal report");
                }
                crate::driver::answer(
                    &mut root,
                    read,
                    c::ReadValue::Notes(Box::new(c::NotePage { rows, before: None, after: None, cut: false })),
                    &mut out,
                );
            }
            Down::Send { attempt, request: jig_client::Ask::Family { record, .. }, .. }
                if wire::decode_requesting(&record.bytes, &w::CEILINGS)
                    .is_some_and(|request| matches!(request.request(), w::Request::Looked(_))) =>
            {
                step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::Answer {
                                attempt,
                                envelope: skein_lib::RequestEnvelope::Final,
                                why: None,
                                answer: Some(jig_client::Answered::Family(jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::answer(
                                        &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Looked })
                                            .expect("Home look"),
                                    ),
                                })),
                            },
                        },
                    },
                    &mut out,
                );
            }
            Down::Read { read, what: jig_client::Readable::Family { record, .. } } => {
                let value = match wire::decode_reading(&record.bytes, &w::CEILINGS).expect("Home read").read() {
                    w::Read::Setting(setting) if setting.kind() == 7 => {
                        crate::worlds::project_ways(
                            &mut root,
                            read,
                            false,
                            with_goals || home_away,
                            home_away,
                            home_away || away_sample,
                            &mut out,
                        );
                        continue;
                    }
                    w::Read::Deployment => Some(wire::deployment_answer(
                        &w::DeploymentAnswer::new(
                            &w::CEILINGS,
                            w::DeploymentAnswerParts { welcome: true, owned: true, program_login: None },
                        )
                        .expect("deployment"),
                    )),
                    _ => None,
                };
                step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::ReadDone {
                                read,
                                result: jig_client::Readout::Family { family: wire::FORGE, record: value },
                            },
                        },
                    },
                    &mut out,
                );
            }
            _ => {}
        }
    }
    assert!(out.is_empty(), "Home fixture settles");
    if name == "home-answered" {
        step(
            &mut root,
            Event::Act {
                action: Action::Pages(Act::Needs(temper_client_domain::pages::needs::Act::Choose {
                    object: c::ObjectKey::Question { task: 3, message: 1 },
                    words: "In jig’s store".as_bytes().into(),
                })),
            },
            &mut out,
        );
    }
    if matches!(name, "home-adopting" | "home-refused") {
        step(&mut root, Event::Act { action: Action::Pages(Act::AdoptFirst) }, &mut out);
        step(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
        let attempt = sent_attempt(&mut out);
        if name == "home-refused" {
            let outcome = w::Outcome::ForgeRefused(
                w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::HomeHeld })
                    .expect("refusal"),
            );
            let answer = w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).expect("Home answer");
            step(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: Up::Answer {
                            attempt,
                            envelope: skein_lib::RequestEnvelope::Final,
                            answer: Some(jig_client::Answered::Family(jig_client::Family {
                                family: wire::FORGE,
                                version: wire::CLIENT_VERSION,
                                bytes: wire::answer(&answer),
                            })),
                            why: None,
                        },
                    },
                },
                &mut out,
            );
        }
    }
    if name == "home-skip-refused" {
        step(&mut root, Event::Act { action: Action::Pages(Act::SkipToBegin) }, &mut out);
        step(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
        let attempt = sent_attempt(&mut out);
        step(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                            refusal: c::Refusal::NotOffered,
                            at: None,
                        })),
                        why: None,
                    },
                },
            },
            &mut out,
        );
    }
    if matches!(name, "home-add-repository-sheet" | "home-add-repository-copies") {
        crate::worlds::home_add_sheet(&mut root, name == "home-add-repository-copies");
    }
    if home_away {
        crate::worlds::home_away_deliveries(&mut root, decided_watch.expect("Decided watch"), away_name);
    }
    if away_sample {
        crate::worlds::rail_away_toggle(&mut root, away_watch.expect("Away watch"), away_name == "rail-away-on");
    }
    root
}
fn home_text(words: &str) -> c::DocumentText {
    c::DocumentText::complete(
        c::Text::new(words.as_bytes().into(), crate::driver::limits().client.domain.text).expect("Home words"),
    )
}
fn home_since(name: &str, minutes: u64) -> skein_lib::Wall {
    skein_lib::Wall::from_nanos(
        crate::worlds::sample_now(FirstRun { step: 7 }, name)
            .as_nanos()
            .checked_sub(minutes.checked_mul(60_000_000_000).expect("Home minutes fit"))
            .expect("Home time follows epoch"),
    )
}
fn home_title(task: u64) -> &'static str {
    match task {
        3 => "Add a README",
        2 => "Hello in Rust",
        1 => "What is in this repository?",
        _ => unreachable!("Home's three chats"),
    }
}
fn home_chat_rows(name: &str, source: &c::TaskLine, ended: bool) -> skein_lib::List<c::TaskLine> {
    let tasks: &[u64] = match (name, ended) {
        (
            "home-morning" | "home-at-work" | "home-needs-several" | "home-morning-card" | "home-answered"
            | "home-one-need",
            false,
        ) => &[3],
        (
            "home-morning" | "home-at-work" | "home-needs-several" | "home-morning-card" | "home-answered"
            | "home-one-need",
            true,
        ) => &[2, 1],
        ("home-nothing-needs", true) => &[1],
        _ => &[],
    };
    let mut rows = skein_lib::List::with_capacity(3);
    for task in tasks {
        let mut row = source.clone();
        row.project = 1;
        row.chip.task = *task;
        row.chip.title = home_text(home_title(*task));
        row.last_activity = home_since(name, if ended { 60 } else { 4 });
        row.ended_at = if ended { Some(home_since(name, 60)) } else { None };
        row.waits_on = if ended {
            c::WaitsOn::Nothing
        } else {
            c::WaitsOn::Party { whom: c::Whom::Person(73), entries: 1, task: *task, kind: c::WaitedFor::Question }
        };
        rows.push(row).expect("Home chats");
    }
    rows
}

fn home_forges(name: &str, in_project: bool) -> skein_lib::List<temper_client_protocol_wire::v1::ForgeRow> {
    use skein_lib::List;
    use temper_client_protocol_wire::v1 as w;
    let mut rows = List::with_capacity(1);
    if name == "home-no-repository" {
        return rows;
    }
    let github = name == "home-github";
    let mut repositories = List::with_capacity(1);
    if in_project {
        repositories
            .push(
                w::Adoption::new(
                    &w::CEILINGS,
                    w::AdoptionParts { owner: b"work".as_slice().into(), name: b"hello".as_slice().into(), project: 1 },
                )
                .expect("adoption"),
            )
            .expect("repository");
    }
    let mut installations = List::with_capacity(1);
    if github {
        installations
            .push(
                w::Installation::new(
                    &w::CEILINGS,
                    w::InstallationParts {
                        id: 1,
                        account: Some(
                            w::Account::new(&w::CEILINGS, w::AccountParts { id: 73, login: b"sam".as_slice().into() })
                                .expect("account"),
                        ),
                        suspended: false,
                        stray: false,
                    },
                )
                .expect("installation"),
            )
            .expect("one installation");
    }
    let first = if github || name.starts_with("home-add-repository") {
        None
    } else {
        Some(
            w::FirstRepository::new(
                &w::CEILINGS,
                w::FirstRepositoryParts {
                    playground: false,
                    org: b"work".as_slice().into(),
                    repository: b"hello".as_slice().into(),
                    state: w::FirstState::Made(w::Made::new(&w::CEILINGS, w::MadeParts { id: 1 }).expect("made")),
                    label: b"main".as_slice().into(),
                },
            )
            .expect("first"),
        )
    };
    rows.push(
        w::ForgeRow::new(
            &w::CEILINGS,
            w::ForgeRowParts {
                starts_with: if name.starts_with("home-add-repository") {
                    w::StartsWith::Nothing
                } else {
                    w::StartsWith::Repository
                },
                host: b"forge.example:3000".as_slice().into(),
                number: if github { 2 } else { 3 },
                kind: if github { w::Kind::Github } else { w::Kind::Forgejo },
                mode: if github { w::Mode::Managed } else { w::Mode::Owned },
                connection: w::Connection::Connected(
                    w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                        .expect("connected"),
                ),
                offer: w::Offer::Offered,
                first,
                installations,
                ended: None,
                app: None,
                web: None,
                repositories,
                owned: None,
                admin: false,
                read_only: false,
                ended_stop: None,
                makings: List::with_capacity(0),
            },
        )
        .expect("Home row"),
    )
    .expect("one forge");
    if name.starts_with("home-cop") {
        let row = rows.get(0).expect("Home forge").clone().into_parts();
        *rows.get_mut(0).expect("Home forge") =
            w::ForgeRow::new(&w::CEILINGS, crate::worlds::home_copies(row, name)).expect("Home copies");
    }
    rows
}

// Home account variants keep the first-run world's documents and vary its held sources.
fn home_accounts(name: &str, mut accounts: c::Accounts) -> c::Accounts {
    if !matches!(name, "home-accounts-claude" | "home-accounts-both" | "home-accounts-key") {
        return accounts;
    }
    let mut row = accounts.accounts.get(0).expect("first-run account").clone();
    if name == "home-accounts-key" {
        row.plan = false;
        row.sources.get_mut(0).expect("first-run source").kind = c::AccountKind::Key;
    } else {
        row.provider = Some(2);
        row.provider_name = Some(home_text("anthropic"));
        row.sources.get_mut(0).expect("first-run source").kind = c::AccountKind::Program;
    }
    let mut rows = skein_lib::List::with_capacity(2);
    if name == "home-accounts-both" {
        rows.push(accounts.accounts.get(0).expect("ChatGPT account").clone()).expect("ChatGPT row");
        row.number = row.number.checked_add(1).expect("second account");
        row.sources.get_mut(0).expect("Claude source").number = 2;
    }
    rows.push(row).expect("account variant");
    accounts.accounts = rows;
    accounts
}

fn home_needs(name: &str) -> skein_lib::List<c::InboxEntry> {
    let mut rows = skein_lib::List::with_capacity(3);
    let tasks: &[u64] = match name {
        "home-at-work" | "home-needs-under-begin" | "home-one-need" => &[3],
        "home-morning" | "home-needs-several" | "home-morning-card" | "home-answered" => &[3, 2, 1],
        _ => &[],
    };
    for task in tasks {
        rows.push(temper_fixtures::client_documents::entry(
            c::ObjectKey::Question { task: *task, message: 1 },
            1,
            1,
            home_since(name, 4),
            None,
        ))
        .expect("Home needs");
    }
    rows
}

fn home_question(name: &str, task: u64) -> &'static str {
    if matches!(name, "home-morning-card" | "home-answered") {
        match task {
            3 => {
                "Fix the dropped watch in jig’s store, or keep it alive in temper serve?\n\nMakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local but leaves jig’s behaviour as it is.\n\nSaw: The box test reproduces it every time: the store’s write lands, then the watch closes before the answer comes. The test and its log are on the goal.\nChoice: In jig’s store (recommended)\nChoice: In temper serve"
            }
            2 => {
                "The “connect your deployment” help page isn’t written. Link the server help on GitHub for now, or hide the link until the page exists?\n\nThe welcome’s design names the link a placeholder.\nChoice: Link the GitHub page (recommended)\nChoice: Hide the link"
            }
            _ => {
                "The repository has no license. Name one in the README?\nChoice: MIT (recommended)\nChoice: Apache-2.0\nChoice: No license line"
            }
        }
    } else {
        "The repository has no license. Name one in the README?\n\nThe README is written apart from this line.\nChoice: MIT (recommended)\nChoice: Apache-2.0\nChoice: No license line"
    }
}

fn morning_status() -> temper_client_protocol_wire::v1::ProjectStatus {
    use temper_client_protocol_wire::v1 as w;
    let now = crate::worlds::sample_now(FirstRun { step: 7 }, "home-morning").as_nanos();
    let minute = 60_000_000_000;
    w::ProjectStatus::new(&w::CEILINGS, w::ProjectStatusParts {
        project: 1,
        landed: 41,
        last_landed: Some(now - 6 * minute),
        brief: Some(w::StatusBrief::new(&w::CEILINGS, w::StatusBriefParts {
            sentence: b"41 changes landed overnight.".as_slice().into(),
            lede: b"first-run-projects is six of nine tasks in. Its store task hit the wasm size limit twice and was split in two.".as_slice().into(),
            at: now - 22 * minute,
            landed: 41,
        }).expect("morning brief fits")),
        previous: Some(now - 9 * 60 * minute),
        seen: 0,
        queued: 3,
        red: false,
        branch: b"main".as_slice().into(),
    }).expect("morning status fits")
}

fn confirmation(root: &mut Domain, kind: Confirmation, out: &mut Queue<Request>) {
    let act = match kind {
        Confirmation::CloseWork
        | Confirmation::CloseFailed
        | Confirmation::CloseBelow
        | Confirmation::CloseProposal
        | Confirmation::CloseTask => Act::Close,
        Confirmation::StartOver => Act::StartOver,
        Confirmation::Accept | Confirmation::Reject => Act::Decide {
            object: c::ObjectKey::Proposal(crate::worlds::PROPOSAL),
            revision: None,
            act: if kind == Confirmation::Accept { c::ActKind::Accept } else { c::ActKind::Reject },
        },
        Confirmation::ReleaseUncertain => Act::Decide {
            object: c::ObjectKey::Task(3),
            revision: Some(c::Revision { value: 7 }),
            act: c::ActKind::Release,
        },
        Confirmation::Closed => return,
    };
    if kind == Confirmation::Reject {
        step(
            root,
            Event::Act {
                action: Action::Pages(Act::CardWords {
                    object: c::ObjectKey::Proposal(crate::worlds::PROPOSAL),
                    words: Box::from(b"Keep the existing output.".as_slice()),
                }),
            },
            out,
        );
    }
    step(root, Event::Act { action: Action::Pages(act) }, out);
    if kind == Confirmation::CloseWork || kind == Confirmation::CloseFailed {
        step(
            root,
            Event::Act { action: Action::Pages(Act::DelegateFate { task: 5, fate: jig_client::shapes::Fate::Move }) },
            out,
        );
    }
    if kind == Confirmation::CloseFailed {
        step(root, Event::Act { action: Action::Pages(Act::Confirm) }, out);
        while let Some(request) = out.pop() {
            if let Some(jig_client::Down::Send { attempt, .. }) = crate::driver::down(request) {
                step(
                    root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Answer {
                                attempt,
                                envelope: skein_lib::RequestEnvelope::Final,
                                answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                                    refusal: c::Refusal::NotOffered,
                                    at: None,
                                })),
                                why: None,
                            },
                        },
                    },
                    &mut Queue::uncapped(),
                );
            }
        }
    }
}
