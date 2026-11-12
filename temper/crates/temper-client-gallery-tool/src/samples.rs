//! The promised names (client/web/gallery.md, 2), sorted for the gallery index.
use crate::driver::step;
use jig_client::contract as c;
use skein_lib::{Queue, Token};
use temper_client_domain::{Action, Address, Domain, Event, Request, address::Filter, pages::Act};
use temper_client_domain::{ChatsFilter, GoalsFilter};

use Then::{
    AccountRefused, Closing, Declined, Held, OutOfDate, Rejecting, Releasing, Renewed, Shown, SignInReady,
    SignInRefused, SignedOut, SigningIn,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum World {
    Reference,
    Solo,
    WholeStory,
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
    Models { one_account: bool, state: ModelState },
    Hosts { role: bool, connected: bool },
    Working,
    DelegatedGoal,
    HeldChange,
    HeldTask,
    Paused,
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
    SignInReady,
    Declined,
    Renewed,
    SignInRefused,
}
/// The Models tab's switch, provenance and membership states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelState {
    On,
    Off,
    Added,
    Member,
    Read,
    ReadFailed,
    New,
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
    /// Opens a page-owned disclosure in the static print (client/web/gallery.md).
    #[must_use]
    pub fn open_disclosure(&self) -> Option<&[u8]> {
        match self.name {
            "sheet-few" => Some(b"More"),
            "rail-all-projects" => Some(b"vsr"),
            "rail-across" => Some(b"All projects"),
            "chat-model-open"
            | "chat-model-both"
            | "chat-model-all"
            | "chat-model-new"
            | "chat-model-two-accounts"
            | "chat-model-sign-in-again"
            | "chat-started-model-open"
            | "chat-started-model-changing"
            | "chat-started-model-refused"
            | "ask-model-open" => Some(b"Model"),
            "chat-work-open" => Some("Checked settings 3×, read 7 docs".as_bytes()),
            _ => None,
        }
    }
    /// Opens the rail menu through the page part in the static print (client/web/gallery.md).
    #[must_use]
    pub fn open_menu(&self) -> Option<&[u8]> {
        match self.name {
            "rail-new-menu" => Some(b"New draft or change"),
            "rail-person-menu" => Some(b"Robin"),
            _ => None,
        }
    }
}
pub const N: usize = 415;
const CHATS: Address = Address::Chats { filter: ChatsFilter::All };
use World::{Bare, FirstRun, Reference, Solo};
#[must_use]
fn task(task: u64) -> Address {
    Address::Task { task }
}

#[must_use]
#[expect(clippy::too_many_lines, reason = "one table keeps every named gallery state in its promised order")]
#[expect(clippy::large_stack_arrays, reason = "the gallery returns its bounded named-state table as a fixed array")]
pub fn samples() -> [Sample; N] {
    let mut samples = [
        Sample { name: "team-role", world: Bare, address: Address::Team { project: 1, role: Some(52) }, then: Shown },
        Sample {
            name: "night-team-role",
            world: Bare,
            address: Address::Team { project: 1, role: Some(52) },
            then: Shown,
        },
        Sample { name: "team", world: Bare, address: Address::Team { project: 1, role: None }, then: Shown },
        Sample { name: "team-alone", world: Bare, address: Address::Team { project: 1, role: None }, then: Shown },
        Sample { name: "night-team", world: Bare, address: Address::Team { project: 1, role: None }, then: Shown },
        Sample {
            name: "rail-new-change",
            world: Reference,
            address: Address::Work { project: 1, filter: temper_client_domain::WorkFilter::All },
            then: Shown,
        },
        Sample {
            name: "rail-all-projects",
            world: Reference,
            address: Address::Work { project: 1, filter: temper_client_domain::WorkFilter::All },
            then: Shown,
        },
        Sample { name: "rail-across", world: Reference, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample {
            name: "work",
            world: Reference,
            address: Address::Work { project: 1, filter: temper_client_domain::WorkFilter::All },
            then: Shown,
        },
        Sample {
            name: "work-pair",
            world: Reference,
            address: Address::Work { project: 1, filter: temper_client_domain::WorkFilter::All },
            then: Shown,
        },
        Sample {
            name: "work-drafts",
            world: Reference,
            address: Address::Work { project: 1, filter: temper_client_domain::WorkFilter::Drafts },
            then: Shown,
        },
        Sample {
            name: "work-goals",
            world: Reference,
            address: Address::Goals { project: 1, filter: GoalsFilter::All },
            then: Shown,
        },
        Sample { name: "work-changes", world: Reference, address: Address::Changes { project: 1 }, then: Shown },
        Sample {
            name: "work-empty",
            world: Reference,
            address: Address::Work { project: 1, filter: temper_client_domain::WorkFilter::All },
            then: Shown,
        },
        Sample { name: "remembered-none", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "remembered-three", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "remembered-full", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "remembered-changing", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "remembered-refused", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "night-remembered-none", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "night-remembered-three", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "night-remembered-full", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "night-remembered-changing", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "night-remembered-refused", world: Reference, address: Address::Remembered, then: Shown },
        Sample { name: "members", world: Reference, address: Address::Members, then: Shown },
        Sample { name: "members-refused", world: Reference, address: Address::Members, then: Shown },
        Sample { name: "limits", world: Reference, address: Address::Limits, then: Shown },
        Sample { name: "limits-not-owner", world: Reference, address: Address::Limits, then: Shown },
        Sample {
            name: "night-work",
            world: Reference,
            address: Address::Work { project: 1, filter: temper_client_domain::WorkFilter::All },
            then: Shown,
        },
        Sample { name: "night-limits", world: Reference, address: Address::Limits, then: Shown },
        Sample { name: "how", world: FirstRun { step: 7 }, address: Address::Settings { project: 1 }, then: Shown },
        Sample {
            name: "how-approve",
            world: FirstRun { step: 7 },
            address: Address::Settings { project: 1 },
            then: Shown,
        },
        Sample { name: "how-draft", world: Reference, address: Address::Settings { project: 1 }, then: Shown },
        Sample {
            name: "how-yours",
            world: FirstRun { step: 7 },
            address: Address::Settings { project: 1 },
            then: Shown,
        },
        Sample {
            name: "how-card",
            world: FirstRun { step: 7 },
            address: Address::Settings { project: 1 },
            then: Shown,
        },
        Sample {
            name: "night-how",
            world: FirstRun { step: 7 },
            address: Address::Settings { project: 1 },
            then: Shown,
        },
        Sample { name: "how-rules", world: Reference, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "how-rules-card", world: Reference, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "how-team", world: Reference, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "night-how-team", world: Reference, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "how-follows", world: Reference, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "night-how-follows", world: Reference, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "how-reaches", world: Reference, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "night-how-reaches", world: Reference, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "night-home", world: FirstRun { step: 7 }, address: Address::Home { project: 1 }, then: Shown },
        Sample { name: "night-ask", world: FirstRun { step: 7 }, address: Address::Ask { project: 1 }, then: Shown },
        Sample {
            name: "night-chat-split",
            world: Solo,
            address: Address::Chat { task: 1 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Split),
        },
        Sample { name: "approve-typing", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "approve-unread", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "approve-asking", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "approve-approved", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "approve-signed-in", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "approve-refused", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "approve-expired", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "approve-used", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "approve-unknown", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample {
            name: "approve-signed-out",
            world: Bare,
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
        Sample {
            name: "approve-signed-out-google",
            world: Bare,
            address: Address::Inbox { filter: Filter::ALL },
            then: Then::GoogleSignIn,
        },
        Sample { name: "approve-busy", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "ask", world: Solo, address: Address::Ask { project: 1 }, then: Shown },
        Sample { name: "chat-terminal-question", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-terminal-proposal", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "chat-terminal-draft", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-terminal-commands", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-terminal-model", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-terminal-light", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-terminal-closed", world: Solo, address: Address::Chat { task: 6 }, then: Shown },
        Sample { name: "chat-terminal", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-terminal-working", world: Solo, address: Address::Chat { task: 6 }, then: Then::Working },
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
            name: "models",
            world: Reference,
            address: Address::Models { project: 1 },
            then: Then::Models { one_account: false, state: ModelState::On },
        },
        Sample {
            name: "models-read",
            world: Reference,
            address: Address::Models { project: 1 },
            then: Then::Models { one_account: false, state: ModelState::Read },
        },
        Sample {
            name: "models-read-failed",
            world: Reference,
            address: Address::Models { project: 1 },
            then: Then::Models { one_account: false, state: ModelState::ReadFailed },
        },
        Sample {
            name: "models-new",
            world: Reference,
            address: Address::Models { project: 1 },
            then: Then::Models { one_account: false, state: ModelState::New },
        },
        Sample {
            name: "models-add",
            world: Reference,
            address: Address::Models { project: 1 },
            then: Then::Models { one_account: false, state: ModelState::On },
        },
        Sample {
            name: "models-off",
            world: Reference,
            address: Address::Models { project: 1 },
            then: Then::Models { one_account: false, state: ModelState::Off },
        },
        Sample {
            name: "models-added",
            world: Reference,
            address: Address::Models { project: 1 },
            then: Then::Models { one_account: false, state: ModelState::Added },
        },
        Sample {
            name: "models-member",
            world: Reference,
            address: Address::Models { project: 1 },
            then: Then::Models { one_account: false, state: ModelState::Member },
        },
        Sample {
            name: "models-one-account",
            world: Reference,
            address: Address::Models { project: 1 },
            then: Then::Models { one_account: true, state: ModelState::On },
        },
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
        Sample {
            name: "home-decided",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "needs-decided",
            world: FirstRun { step: 7 },
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
        Sample {
            name: "needs-decided-project",
            world: FirstRun { step: 7 },
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample {
            name: "needs-only-you",
            world: Reference,
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
        Sample {
            name: "needs-only-you-done",
            world: Reference,
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
        Sample { name: "home-away", world: FirstRun { step: 7 }, address: Address::Home { project: 1 }, then: Shown },
        Sample {
            name: "home-away-quiet",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
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
        Sample { name: "all-home", world: World::WholeStory, address: Address::AllHome, then: Shown },
        Sample { name: "all-home-nothing-new", world: World::WholeStory, address: Address::AllHome, then: Shown },
        Sample { name: "all-home-one-project", world: World::WholeStory, address: Address::AllHome, then: Shown },
        Sample { name: "all-home-away", world: World::WholeStory, address: Address::AllHome, then: Shown },
        Sample { name: "all-home-away-on", world: World::WholeStory, address: Address::AllHome, then: Shown },
        Sample { name: "night-all-home", world: World::WholeStory, address: Address::AllHome, then: Shown },
        Sample { name: "home-step-pair", world: Solo, address: Address::Home { project: 1 }, then: Shown },
        Sample { name: "home-step-team", world: Solo, address: Address::Home { project: 1 }, then: Shown },
        Sample { name: "home-step-stock", world: Solo, address: Address::Home { project: 1 }, then: Shown },
        Sample { name: "home-step-scale", world: Solo, address: Address::Home { project: 1 }, then: Shown },
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
            name: "home-begin-both-left",
            world: FirstRun { step: 7 },
            address: Address::Home { project: 1 },
            then: Shown,
        },
        Sample {
            name: "home-begin-ask-left",
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
            name: "home-try-stopped",
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
            name: "home-begin-with-needs",
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
            name: "ask-first-run",
            world: FirstRun { step: 7 },
            address: Address::Ask { project: 1 },
            then: Shown,
        },
        Sample { name: "ask-typing", world: FirstRun { step: 7 }, address: Address::Ask { project: 1 }, then: Shown },
        Sample {
            name: "ask-ways-approve",
            world: FirstRun { step: 7 },
            address: Address::Ask { project: 1 },
            then: Shown,
        },
        Sample {
            name: "ask-ways-draft",
            world: FirstRun { step: 7 },
            address: Address::Ask { project: 1 },
            then: Shown,
        },
        Sample {
            name: "ask-ways-card",
            world: FirstRun { step: 7 },
            address: Address::Ask { project: 1 },
            then: Shown,
        },
        Sample {
            name: "ask-ways-card-person",
            world: FirstRun { step: 7 },
            address: Address::Ask { project: 1 },
            then: Shown,
        },
        Sample {
            name: "ask-ways-yours",
            world: FirstRun { step: 7 },
            address: Address::Ask { project: 1 },
            then: Shown,
        },
        Sample {
            name: "ask-ways-refused",
            world: FirstRun { step: 7 },
            address: Address::Ask { project: 1 },
            then: Shown,
        },
        Sample {
            name: "ask-ways-member",
            world: FirstRun { step: 7 },
            address: Address::Ask { project: 1 },
            then: Shown,
        },
        Sample {
            name: "ask-ways-kept",
            world: FirstRun { step: 7 },
            address: Address::Ask { project: 1 },
            then: Shown,
        },
        Sample { name: "ask-sending", world: FirstRun { step: 7 }, address: Address::Ask { project: 1 }, then: Shown },
        Sample { name: "ask-refused", world: FirstRun { step: 7 }, address: Address::Ask { project: 1 }, then: Shown },
        Sample {
            name: "ask-chat-document",
            world: Solo,
            address: Address::Chat { task: 10 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Chat),
        },
        Sample {
            name: "ask-chat-document-split",
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
        Sample { name: "chat-view-way", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "chat-view-proposal", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "chat-view-charter", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "ask-chat", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "ask-chat-proposal-deployment", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample {
            name: "ask-chat-proposal-two-projects",
            world: Solo,
            address: Address::Chat { task: 10 },
            then: Shown,
        },
        Sample { name: "ask-chat-proposal", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "ask-chat-accepted", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "ask-chat-rejected", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "ask-chat-replaced", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "ask-chat-withdrawn", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "ask-chats-tag", world: Solo, address: CHATS, then: Shown },
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
            world: Reference,
            address: Address::Home { project: 1 },
            then: Then::Mode(temper_client_domain::pages::chat::document::Mode::Chat),
        },
        Sample { name: "rail-person-menu", world: Solo, address: Address::Home { project: 1 }, then: Shown },
        Sample { name: "repositories", world: Solo, address: Address::Repositories { project: 1 }, then: Shown },
        Sample {
            name: "repositories-making",
            world: FirstRun { step: 7 },
            address: Address::Repositories { project: 1 },
            then: Shown,
        },
        Sample { name: "repositories-none", world: Bare, address: Address::Repositories { project: 1 }, then: Shown },
        Sample {
            name: "forges-added",
            world: FirstRun { step: 7 },
            address: Address::ConnectForge { forge: Some(temper_client_domain::Choice::Forge(3)) },
            then: Shown,
        },
        Sample { name: "forges-empty", world: Bare, address: Address::Forges, then: Shown },
        Sample { name: "forges-setting-up", world: FirstRun { step: 6 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-fetching", world: FirstRun { step: 24 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-fetching-bytes", world: FirstRun { step: 25 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-failed", world: FirstRun { step: 9 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-replacing", world: Solo, address: Address::Forges, then: Shown },
        Sample { name: "forges-removing", world: FirstRun { step: 7 }, address: Address::Forges, then: Shown },
        Sample { name: "forges-in-settings", world: Solo, address: Address::Forges, then: Shown },
        Sample { name: "accounts-in-settings", world: Solo, address: Address::Accounts, then: Shown },
        Sample { name: "needs-list", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "needs-list-second", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "needs-tell", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample {
            name: "needs-approval-first",
            world: Solo,
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
        Sample { name: "night-needs", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample {
            name: "night-needs-only-you",
            world: Reference,
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
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
        Sample { name: "needs-approval", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample {
            name: "needs-approval-path",
            world: Solo,
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
        Sample {
            name: "needs-approval-no-account",
            world: Solo,
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
        Sample { name: "needs-proposal", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample {
            name: "needs-stop-asking",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample {
            name: "needs-stop-asking-set",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
        Sample {
            name: "needs-why",
            world: Solo,
            address: Address::Inbox { filter: Filter { project: Some(1), ..Filter::ALL } },
            then: Shown,
        },
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
        Sample {
            name: "start-goal",
            world: Reference,
            address: Address::Goals { project: 1, filter: GoalsFilter::All },
            then: Shown,
        },
        Sample { name: "start-goal-draft", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "start-goal-refused", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "settings", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-drafts-off", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-goals-off", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
        Sample { name: "settings-changes-off", world: Solo, address: Address::Settings { project: 1 }, then: Shown },
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
        Sample {
            name: "connect-forge-elsewhere",
            world: FirstRun { step: 0 },
            address: Address::ConnectForge { forge: Some(temper_client_domain::Choice::Elsewhere) },
            then: Shown,
        },
        Sample {
            name: "connect-forge-connected",
            world: FirstRun { step: 7 },
            address: Address::ConnectForge { forge: None },
            then: Shown,
        },
        Sample {
            name: "connect-forge-yours-admin",
            world: FirstRun { step: 0 },
            address: Address::ConnectForge { forge: Some(temper_client_domain::Choice::Managed) },
            then: Shown,
        },
        Sample {
            name: "connect-forge-yours-tokens",
            world: FirstRun { step: 0 },
            address: Address::ConnectForge { forge: Some(temper_client_domain::Choice::Managed) },
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
        Sample { name: "sheet-adopt", world: Reference, address: Address::Adopt { project: 1 }, then: Shown },
        Sample { name: "sheet-searching", world: Reference, address: Address::Adopt { project: 1 }, then: Shown },
        Sample { name: "sheet-none", world: Reference, address: Address::Adopt { project: 1 }, then: Shown },
        Sample { name: "sheet-few", world: Reference, address: Address::Adopt { project: 1 }, then: Shown },
        Sample { name: "sheet-empty", world: Reference, address: Address::Adopt { project: 1 }, then: Shown },
        Sample { name: "sheet-copy", world: FirstRun { step: 7 }, address: Address::Adopt { project: 1 }, then: Shown },
        Sample {
            name: "sheet-copy-lines",
            world: FirstRun { step: 7 },
            address: Address::Adopt { project: 1 },
            then: Shown,
        },
        Sample {
            name: "sheet-empty-source",
            world: FirstRun { step: 7 },
            address: Address::Adopt { project: 1 },
            then: Shown,
        },
        Sample {
            name: "sheet-from-home",
            world: FirstRun { step: 7 },
            address: Address::Adopt { project: 1 },
            then: Shown,
        },
        Sample {
            name: "sheet-from-chats",
            world: FirstRun { step: 7 },
            address: Address::Adopt { project: 1 },
            then: Shown,
        },
        Sample { name: "chats-reference", world: Reference, address: CHATS, then: Shown },
        Sample { name: "chat-reference-T1", world: Reference, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-solo-T1", world: Solo, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-model-open", world: Bare, address: Address::NewChat { draft: false }, then: Shown },
        Sample { name: "chat-model-both", world: Bare, address: Address::NewChat { draft: false }, then: Shown },
        Sample { name: "chat-model-new", world: Bare, address: Address::NewChat { draft: false }, then: Shown },
        Sample { name: "chat-model-all", world: Bare, address: Address::NewChat { draft: false }, then: Shown },
        Sample {
            name: "chat-model-two-accounts",
            world: Bare,
            address: Address::NewChat { draft: false },
            then: Shown,
        },
        Sample {
            name: "chat-model-sign-in-again",
            world: Bare,
            address: Address::NewChat { draft: false },
            then: Shown,
        },
        Sample { name: "chat-model-fixed", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-started-model-open", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-started-model-changing", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-started-model-refused", world: Bare, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "ask-model-open", world: Bare, address: Address::Ask { project: 1 }, then: Shown },
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
        Sample { name: "accounts-expiring", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-key", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-removing", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-signing-in", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-program-needs-sign-in", world: Solo, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-empty", world: Bare, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-reference", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-first-run-1", world: FirstRun { step: 1 }, address: Address::Accounts, then: Shown },
        Sample { name: "google-sign-in", world: Bare, address: CHATS, then: Then::GoogleSignIn },
        Sample { name: "no-project", world: Bare, address: CHATS, then: Shown },
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
        Sample {
            name: "connect-model-connected",
            world: Reference,
            address: Address::ConnectModel { way: None },
            then: Shown,
        },
        model_sample("connect-model-key-details", temper_client_domain::WayKind::Key, Shown),
        model_sample("connect-model-program-details", temper_client_domain::WayKind::Program, Shown),
        model_sample("connect-model-signing-in", temper_client_domain::WayKind::SignIn, SigningIn),
        model_sample("accounts-added", temper_client_domain::WayKind::SignIn, SignInReady),
        model_sample("connect-model-declined", temper_client_domain::WayKind::SignIn, Declined),
        model_sample("accounts-renewed", temper_client_domain::WayKind::SignIn, Renewed),
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
    if matches!(name, "needs-list" | "needs-list-second" | "needs-tell" | "needs-approval-first" | "night-needs") {
        return Some("../whole-story/needs.html".to_owned());
    }
    if name.starts_with("needs-") || name == "inbox-solo" || name == "inbox-reference" || name == "inbox-empty" {
        return Some("../greenfield/needs.html".to_owned());
    }
    match name {
        "team" | "night-team" => return Some("../whole-story/team/team.html".to_owned()),
        "team-role" | "night-team-role" => return Some("../whole-story/team.html".to_owned()),
        "all-home"
        | "all-home-nothing-new"
        | "all-home-away"
        | "all-home-away-on"
        | "all-home-one-project"
        | "night-all-home"
        | "home-step-stock"
        | "home-step-scale" => return Some("../whole-story/home.html".to_owned()),
        "home-step-pair" => return Some("../whole-story/pair/home.html".to_owned()),
        "home-step-team" => return Some("../whole-story/team/home.html".to_owned()),
        "how-team" | "night-how-team" => return Some("../web/whole-story/team/how.html".to_owned()),
        "how-approve" => return Some("../web/whole-story/pair/how.html".to_owned()),
        "how" | "how-draft" | "how-yours" | "how-card" | "night-how" | "how-reaches" | "night-how-reaches"
        | "how-rules" | "how-rules-card" | "how-follows" | "night-how-follows" => {
            return Some("../web/whole-story/how.html".to_owned());
        }
        "accounts-added" | "accounts-renewed" => return Some("accounts.html?world=reference&bare".to_owned()),
        "home-decided" | "home-away" | "home-away-one" | "home-away-overturned" | "home-away-quiet" => {
            return Some("../greenfield/home.html".to_owned());
        }
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
    match sample.name {
        "work" | "work-drafts" | "work-goals" | "work-changes" | "night-work" => {
            return Some("../web/whole-story/work.html".to_owned());
        }
        "work-pair" | "rail-new-change" => return Some("../web/whole-story/pair/work.html".to_owned()),
        _ => {}
    }

    if sample.name.starts_with("approve-") {
        return None;
    }
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
            Then::Models { .. }
            | Then::Hosts { .. }
            | Then::Shown
            | Then::DraftEmpty
            | Then::Mode(_)
            | Then::Working
            | Then::DelegatedGoal
            | Then::HeldChange
            | Then::Paused
            | Then::HeldTask
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
            | Then::AccountRefused => return None,
        };
        return Some(format!("connect-model.html?world=first-run&step=5&way=chatgpt&bare{anchor}"));
    }
    let world = match sample.world {
        World::Reference => "reference".to_owned(),
        World::Solo => "solo".to_owned(),
        World::FirstRun { step } => format!("first-run&step={step}"),
        World::Bare | World::WholeStory => return None,
    };
    let (page, task) = match sample.address {
        Address::NewChat { draft } => {
            return Some(if draft { "chat.html?id=new&draft=1" } else { "chat.html?id=new" }.to_owned());
        }
        Address::Chats { .. } => ("chats", None),
        Address::Chat { task } => ("chat", Some(task)),
        Address::AllHome | Address::Inbox { .. } => ("inbox", None),
        Address::Remembered | Address::Members | Address::Limits | Address::Accounts => ("accounts", None),
        Address::ConnectModel { .. } => ("connect-model", None),
        Address::Welcome { .. } => ("welcome", None),
        Address::Task { task, .. } => ("task", Some(task)),
        Address::ConnectForge { .. } => ("connect-forge", None),
        Address::Forges => ("forges", None),
        Address::Work { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Team { .. }
        | Address::Home { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Ask { .. }
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
        Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Remembered
        | Address::Members
        | Address::Limits
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
        | Address::Team { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Work { .. }
        | Address::Goals { .. }
        | Address::Ask { .. }
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
    if sample.name == "start-goal" {
        step(
            &mut root,
            Event::Act { action: Action::Pages(Act::OpenStart { draft: None, chat: None }) },
            &mut Queue::uncapped(),
        );
    } else {
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
/// Kept sign-offs in the Team and Work samples, as the whole story draws them.
pub(super) fn signoffs(parties: &[u64], at: u64) -> skein_lib::List<temper_client_protocol_wire::v1::ChangesSigned> {
    use temper_client_protocol_wire::v1 as w;
    let mut signed = skein_lib::List::with_capacity(8);
    for party in parties {
        signed
            .push(
                w::ChangesSigned::new(&w::CEILINGS, w::ChangesSignedParts { party: *party, head: [0xab; 32], at })
                    .expect("sample signer"),
            )
            .expect("at most eight sample signers");
    }
    signed
}

#[must_use]
#[expect(clippy::too_many_lines, reason = "one dispatch keeps each gallery state with its domain setup")]
pub fn domain(sample: &Sample) -> Domain {
    use temper_client_domain::pages::terminal::{Act as T, Command};
    if sample.name.starts_with("chat-view-") {
        return crate::worlds::view_world(sample.name);
    }
    let base = match sample.name {
        "night-team" => Some("team"),
        "night-team-role" => Some("team-role"),
        "night-needs" => Some("needs-list"),
        "night-needs-only-you" => Some("needs-only-you"),
        "night-home" => Some("home-begin-ask-left"),
        "night-all-home" => Some("all-home"),
        "night-ask" => Some("ask-first-run"),
        "night-how" => Some("how"),
        "night-how-reaches" => Some("how-reaches"),
        "night-how-team" => Some("how-team"),
        "night-how-follows" => Some("how-follows"),
        "night-chat-split" => Some("chat-draft-split"),
        "night-work" => Some("work"),
        "night-limits" => Some("limits"),
        "night-remembered-none" => Some("remembered-none"),
        "night-remembered-three" => Some("remembered-three"),
        "night-remembered-full" => Some("remembered-full"),
        "night-remembered-changing" => Some("remembered-changing"),
        "night-remembered-refused" => Some("remembered-refused"),
        _ => None,
    };
    if let Some(name) = base {
        let mut root = domain(&Sample { name, ..sample.clone() });
        let wall = crate::worlds::now(if sample.world == World::WholeStory { World::WholeStory } else { World::Solo });
        temper_client_domain::step(
            &mut root,
            &skein_lib::Env { now: skein_lib::Time::ZERO, wall, limits: crate::driver::limits() },
            Event::Act { action: Action::Pages(Act::Terminal(T::Night)) },
            &mut Queue::uncapped(),
        );
        return root;
    }
    if matches!(
        sample.name,
        "rail-new-change"
            | "rail-all-projects"
            | "rail-across"
            | "work"
            | "work-pair"
            | "work-drafts"
            | "work-goals"
            | "work-changes"
            | "work-empty"
            | "members"
            | "members-refused"
            | "limits"
            | "limits-not-owner"
    ) {
        return crate::worlds::rail_work_world(sample.name, sample.address.clone());
    }
    if sample.name.starts_with("remembered-") {
        return crate::worlds::remembered_world(sample.name);
    }
    if sample.name == "no-project" {
        return crate::worlds::goals_world(sample.world, sample.address.clone(), sample.name);
    }
    if sample.name == "how" || sample.name.starts_with("how-") {
        return crate::worlds::ways_world(sample.name);
    }
    if sample.name.starts_with("approve-") {
        return crate::worlds::approve_world(if sample.name == "approve-signed-out-google" {
            "approve-signed-out"
        } else {
            sample.name
        });
    }
    if sample.name.starts_with("chat-terminal") {
        let mut root = if sample.name == "chat-terminal-question" {
            crate::worlds::goals_world(sample.world, sample.address.clone(), sample.name)
        } else if sample.name == "chat-terminal-proposal" {
            crate::worlds::settings_chat_world("ask-chat-proposal")
        } else if sample.name == "chat-terminal-closed" {
            domain(&Sample {
                name: "chat-closed",
                world: sample.world,
                address: sample.address.clone(),
                then: Then::Sheet(Confirmation::Closed),
            })
        } else if sample.name != "chat-terminal-working" {
            let base = Sample {
                name: if sample.name == "chat-terminal" { "chat-draft-chat" } else { "chat-draft-split" },
                world: sample.world,
                address: sample.address.clone(),
                then: Then::Mode(if sample.name == "chat-terminal" {
                    temper_client_domain::pages::chat::document::Mode::Chat
                } else {
                    temper_client_domain::pages::chat::document::Mode::Split
                }),
            };
            domain(&base)
        } else {
            crate::worlds::goals_world(sample.world, sample.address.clone(), sample.name)
        };
        step(
            &mut root,
            Event::Act { action: Action::Pages(Act::Terminal(temper_client_domain::pages::terminal::Act::Enter)) },
            &mut Queue::uncapped(),
        );
        let mut out = Queue::uncapped();
        if sample.name == "chat-terminal-commands" || sample.name == "chat-terminal-model" {
            let words: &[u8] = if sample.name == "chat-terminal-model" { b"/model" } else { b"/" };
            step(&mut root, Event::Act { action: Action::Pages(Act::MessageWords { words: words.into() }) }, &mut out);
            if sample.name == "chat-terminal-model" {
                step(&mut root, Event::Act { action: Action::Pages(Act::Send) }, &mut out);
            }
        }
        if sample.name == "chat-terminal-light" {
            for command in [Command::Theme, Command::Font] {
                step(&mut root, Event::Act { action: Action::Pages(Act::Terminal(T::Run { command })) }, &mut out);
            }
        }
        return root;
    }
    if sample.name.starts_with("chat-model-")
        || sample.name.starts_with("chat-started-model-")
        || sample.name == "ask-model-open"
    {
        return crate::worlds::picker_world(sample.name);
    }
    if sample.name.starts_with("chat-work-") || sample.name == "agent-work" {
        return crate::worlds::chat_work_world(sample.name);
    }
    if sample.name == "chats-drafts" {
        return crate::worlds::drafts_world();
    }
    if sample.name == "settings" || sample.name.starts_with("settings-") {
        return crate::worlds::settings_world(sample.name);
    }
    if sample.name == "repositories-making" {
        return crate::worlds::repositories_making_world();
    }
    if sample.name == "welcome-ready-copies" {
        return crate::worlds::copies_world(sample.address.clone(), sample.then);
    }
    if matches!(sample.name, "sheet-from-home" | "sheet-from-chats") {
        let source = if sample.name == "sheet-from-home" { "home-add-repository" } else { "chats-no-repository" };
        let source = samples().into_iter().find(|sample| sample.name == source).expect("sheet entry sample");
        let mut root = domain(&source);
        step(
            &mut root,
            Event::Went { place: temper_client_domain::Place::Address(sample.address.clone()) },
            &mut Queue::uncapped(),
        );
        return root;
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
    if matches!(sample.name, "needs-decided" | "needs-decided-project") {
        return home_domain(sample.name);
    }
    if sample.name.starts_with("needs-") {
        return crate::needs::domain(sample.name);
    }
    if sample.name.starts_with("changes") {
        return crate::worlds::changes_world(sample.name);
    }
    if sample.name == "team" || sample.name == "team-alone" || sample.name == "team-role" {
        return crate::worlds::team_world(sample.name);
    }
    if sample.name.starts_with("ask-") {
        return ask_domain(sample);
    }
    if sample.world == World::WholeStory || sample.name.starts_with("home-step-") {
        return crate::worlds::whole_story_world(sample.name, sample.address.clone());
    }
    if sample.name.starts_with("home-") || matches!(sample.name, "rail-away-on" | "rail-away-off") {
        return home_domain(sample.name);
    }
    if sample.name.starts_with("goals-") || sample.name.starts_with("goal-") {
        return goal_domain(sample);
    }
    let (mut root, accounts_watch) = if sample.then == Then::Sheet(Confirmation::StartOver) {
        crate::worlds::asking_world(sample.world, sample.address.clone(), sample.then)
    } else {
        crate::worlds::world(sample.world, sample.address.clone(), sample.then)
    };
    let mut out = Queue::uncapped();
    if sample.name.starts_with("sheet-") {
        adoption_sheet(&mut root, sample.name, &mut out);
        return root;
    }
    if sample.name.starts_with("connect-forge-yours-") {
        use temper_client_domain::forge::{
            self,
            admin::{PlaceField, Way},
        };
        step(
            &mut root,
            Event::Act {
                action: Action::Forge(forge::Act::Forges(forge::forges::Act::PlaceWrite {
                    field: PlaceField::Host,
                    text: b"https://forge.example".as_slice().into(),
                })),
            },
            &mut out,
        );
        if sample.name == "connect-forge-yours-tokens" {
            step(&mut root, Event::Act { action: Action::Forge(forge::Act::AdminWay(Way::Tokens)) }, &mut out);
        }
    }
    if sample.name == "connect-model-choose" || sample.name == "connect-model-connected" {
        let mut doc = temper_client_domain::pages::accounts::document(root.client()).expect("Model accounts").clone();
        let mut accounts = skein_lib::List::with_capacity(1);
        if sample.name == "connect-model-connected" {
            for row in &doc.accounts {
                if row.provider == Some(18) {
                    accounts.push(row.clone()).expect("ChatGPT alone");
                }
            }
        }
        doc.accounts = accounts;
        crate::driver::deliver(
            &mut root,
            accounts_watch.expect("Model watch"),
            c::Delivery::Accounts(Box::new(doc)),
            &mut out,
        );
    }
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
    if sample.name == "models-add" {
        step(
            &mut root,
            Event::Act {
                action: Action::Pages(Act::Models(temper_client_domain::pages::models::Act::Name(
                    b"claude-haiku-5-5".as_slice().into(),
                ))),
            },
            &mut out,
        );
    }
    match sample.then {
        Then::Mode(mode) => {
            step(&mut root, Event::Act { action: Action::Pages(Act::Mode { mode }) }, &mut out);
            if sample.name == "chat-draft-split" {
                step(&mut root, Event::Act { action: Action::Pages(Act::ShowSection { line: 17 }) }, &mut out);
            }
        }
        Then::Models { .. }
        | Then::Hosts { .. }
        | Then::DraftEmpty
        | Then::Shown
        | Then::Working
        | Then::Held
        | Then::TaskPaused
        | Then::DelegatedGoal
        | Then::HeldChange
        | Then::Paused
        | Then::HeldTask => {}
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
    account_states(&mut root, sample.name, accounts_watch, &mut out);
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
pub(super) fn account_states(root: &mut Domain, name: &str, accounts_watch: Option<Token>, out: &mut Queue<Request>) {
    if matches!(
        name,
        "accounts-states"
            | "accounts-program-needs-sign-in"
            | "accounts-expiring"
            | "needs-only-you"
            | "needs-only-you-done"
    ) {
        let mut doc = temper_client_domain::pages::accounts::document(root.client()).expect("sample accounts").clone();
        if name == "accounts-states" {
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
        } else if name == "accounts-program-needs-sign-in" {
            let row = doc.accounts.get_mut(0).expect("program account");
            let source = row.sources.get_mut(0).expect("program source");
            source.state = c::AccountState::SignInAgain;
            source.usable = false;
            for host in doc.program_hosts.iter_mut() {
                for program in host.programs.iter_mut() {
                    program.state = Some(c::ProgramState::NotSignedIn);
                }
            }
        } else {
            for row in doc.accounts.iter_mut() {
                let source = row.sources.get_mut(0).expect("deciding source");
                match source.kind {
                    c::AccountKind::SignIn => {
                        source.state = if name == "accounts-expiring" {
                            c::AccountState::Granted
                        } else {
                            c::AccountState::SignInAgain
                        };
                        source.usable = name == "accounts-expiring";
                        source.expiring = name == "accounts-expiring";
                    }
                    c::AccountKind::Program => {
                        source.state = c::AccountState::Granted;
                        source.usable = true;
                        source.expiring = name == "needs-only-you";
                    }
                    c::AccountKind::Key | c::AccountKind::None => {}
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
        doc.accounts = crate::worlds::accounts(World::Reference, Shown).accounts;
        for account in doc.accounts.iter_mut() {
            for source in account.sources.iter_mut() {
                if source.name.as_deref() == Some(b"claude-pat") {
                    source.number = 43;
                }
                source.usable = false;
                source.state = c::AccountState::SignInAgain;
            }
        }
        for host in doc.program_hosts.iter_mut() {
            for program in host.programs.iter_mut() {
                program.state = Some(c::ProgramState::NotSignedIn);
            }
        }
        crate::driver::deliver(root, watch, c::Delivery::Accounts(Box::new(doc)), out);
    }
}

fn ask_domain(sample: &Sample) -> Domain {
    if sample.name.starts_with("ask-ways-") {
        return crate::worlds::ways_world(sample.name);
    }
    if matches!(
        sample.name,
        "ask-chat-proposal"
            | "ask-chat-proposal-deployment"
            | "ask-chat-proposal-two-projects"
            | "ask-chat-accepted"
            | "ask-chat-rejected"
            | "ask-chat-withdrawn"
            | "ask-chat-replaced"
    ) {
        return crate::worlds::settings_chat_world(sample.name);
    }
    if matches!(sample.name, "ask-chat" | "ask-chats-tag" | "ask-chat-document" | "ask-chat-document-split") {
        let mut root = crate::worlds::asking_world(sample.world, sample.address.clone(), sample.then).0;
        if let Then::Mode(mode) = sample.then {
            step(&mut root, Event::Act { action: Action::Pages(Act::Mode { mode }) }, &mut Queue::uncapped());
        }
        return root;
    }
    let mut root = if matches!(sample.name, "ask-first-run" | "ask-typing" | "ask-sending" | "ask-refused") {
        crate::worlds::to_begin_world(sample.world, sample.address.clone(), sample.then)
    } else {
        crate::worlds::world(sample.world, sample.address.clone(), sample.then).0
    };
    if sample.name == "ask-first-run" {
        return root;
    }
    let mut out = Queue::uncapped();
    step(
        &mut root,
        Event::Act {
            action: Action::Pages(Act::ChatWords { words: b"I work alone on small changes.".as_slice().into() }),
        },
        &mut out,
    );
    if sample.name == "ask-typing" {
        return root;
    }
    step(&mut root, Event::Act { action: Action::Pages(Act::StartChat) }, &mut out);
    if sample.name == "ask-refused" {
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
fn account(root: &mut Domain, then: Then, out: &mut Queue<Request>) {
    use temper_client_domain::pages::{accounts::Way, connect_model::Act as A};
    let refused = then == Then::AccountRefused;
    let key = refused;
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
// first chat, and keeps Ready's repository under its setup scenario (client/web/gallery.md, 2).
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
    let home_away = matches!(
        name,
        "home-decided"
            | "needs-decided"
            | "needs-decided-project"
            | "home-away"
            | "home-away-one"
            | "home-away-overturned"
            | "home-away-quiet"
    );
    let needs_decided = matches!(name, "needs-decided" | "needs-decided-project");
    let needs_project = name == "needs-decided-project";
    let away_name = name;
    let status_name = if away_sample { "home-morning" } else { name };
    let name = match name {
        "rail-away-on" | "rail-away-off" => "home-morning",
        "home-decided" | "needs-decided" | "needs-decided-project" | "home-away-quiet" => "home-one-need",
        "home-away"
        | "home-away-one"
        | "home-away-overturned"
        | "home-last-landed"
        | "home-computed"
        | "home-last-looked-none"
        | "home-landing-none"
        | "home-landing-red"
        | "home-chats-sentence"
        | "home-try-stopped" => "home-nothing-needs",
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
    if needs_decided && !needs_project {
        let mut other = me.projects.get(0).expect("first project").clone();
        other.project = 2;
        other.name = home_text("Widget");
        me.projects = List::from_boxed(Box::new([me.projects.get(0).expect("first project").clone(), other]));
    }
    if name.starts_with("home-cop") {
        me.projects.get_mut(0).expect("Home copy project").name = home_text("Widget");
    }
    me.needs_total = match name {
        "home-at-work" | "home-begin-with-needs" | "home-one-need" => 1,
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
        Event::Start {
            address: if needs_decided {
                Address::Inbox { filter: Filter { project: if needs_project { Some(1) } else { None }, ..Filter::ALL } }
            } else {
                Address::Home { project: 1 }
            },
            kept: List::with_capacity(0),
            with: None,
        },
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
    let mut decided_watches = List::with_capacity(2);
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
            "home-begin-ask-left"
                | "home-accounts-claude"
                | "home-accounts-both"
                | "home-accounts-key"
                | "home-ready"
                | "home-after-skip"
                | "home-skip-refused"
                | "home-begin-with-needs"
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
                                setting: c::Setting::Application { kind: 2, scope: 1, number: 1 },
                                name: None,
                                follows: if everyday || matches!(name, "home-ready" | "home-begin-settings-left") {
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
                        if matches!(name, "home-ready" | "home-begin-settings-left" | "home-begin-with-needs") {
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
                        &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, linked: false })
                            .expect("Home forges"),
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
                    w::Watch::Trials if status_name == "home-try-stopped" => {
                        let row = w::TrialRow::new(
                            &w::CEILINGS,
                            w::TrialRowParts {
                                account: accounts.accounts.get(0).expect("Home trial account").number,
                                task: Some(91),
                                took: None,
                                step: w::TrialStep::Answers,
                                state: w::TrialState::Failed(
                                    w::TrialFailed::new(&w::CEILINGS, w::TrialFailedParts { why: w::TrialWhy::Waited })
                                        .expect("Home stopped try"),
                                ),
                            },
                        )
                        .expect("Home trial row");
                        let document =
                            w::Trials::new(&w::CEILINGS, w::TrialsParts { rows: List::from_boxed(Box::new([row])) })
                                .expect("Home trials");
                        let mut writer =
                            skein_lib::Writer::new(usize::try_from(document.measure()).expect("trial bytes fit"));
                        document.encode(&mut writer).expect("measured trial bytes");
                        writer.finish()
                    }
                    w::Watch::Away if home_away => crate::worlds::home_away_record(away_name),
                    w::Watch::Decided(subject) if home_away => {
                        decided_watches.push((watch, subject.project())).expect("two decided projects");
                        continue;
                    }
                    w::Watch::Decided(subject) => wire::decisions(
                        &w::Decisions::new(
                            &w::CEILINGS,
                            w::DecisionsParts {
                                project: subject.project(),
                                rows: List::with_capacity(0),
                                asked: List::with_capacity(0),
                                cut: false,
                            },
                        )
                        .expect("Home decided document"),
                    ),
                    w::Watch::Away if away_sample => {
                        away_watch = Some(watch);
                        crate::worlds::away_record(away_name == "rail-away-off")
                    }
                    w::Watch::Away
                    | w::Watch::Change(_)
                    | w::Watch::Landing(_)
                    | w::Watch::Landings(_)
                    | w::Watch::Drafts(_)
                    | w::Watch::Follows(_)
                    | w::Watch::Ways(_)
                    | w::Watch::Trials
                    | w::Watch::Approvals
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
                            &mut out,
                        );
                        continue;
                    }
                    w::Read::Setting(setting) if setting.kind() == 8 => {
                        crate::worlds::project_classes(&mut root, read, &mut out);
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
        assert!(!decided_watches.is_empty(), "Decided watch");
        for (watch, project) in decided_watches.into_boxed() {
            crate::worlds::home_away_deliveries(&mut root, watch, away_name, project);
        }
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
        "home-at-work" | "home-begin-with-needs" | "home-one-need" => &[3],
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

fn making_sheet(root: &mut Domain, name: &str, out: &mut Queue<Request>) {
    use temper_client_domain::forge::{self, sheet};
    if name == "sheet-empty-source" {
        step(
            root,
            Event::Act { action: Action::Forge(forge::Act::Sheet(sheet::Act::Source(sheet::Source::Empty))) },
            out,
        );
        return;
    }
    if matches!(name, "sheet-copy" | "sheet-copy-lines") {
        step(
            root,
            Event::Act { action: Action::Forge(forge::Act::Sheet(sheet::Act::Source(sheet::Source::Copy))) },
            out,
        );
        let addresses = if name == "sheet-copy" {
            b"github.com/octo/vsr\ncodeberg.org/forgejo/runner".as_slice()
        } else {
            b"github/octo/vsr\ngithub.com/octo/raft".as_slice()
        };
        step(
            root,
            Event::Act { action: Action::Forge(forge::Act::Sheet(sheet::Act::Addresses(addresses.into()))) },
            out,
        );
    }
}

fn adoption_sheet(root: &mut Domain, name: &str, out: &mut Queue<Request>) {
    use temper_client_domain::forge::{self, sheet};
    use temper_client_protocol_wire::v1 as w;
    making_sheet(root, name, out);
    if name == "sheet-empty-source" {
        return;
    }
    let sheet::Reading::Asked { key, .. } = root.forge().sheet().expect("sheet").reading else { return };
    let count = match name {
        "sheet-few" => 4,
        "sheet-empty" | "sheet-copy" => 0,
        "sheet-copy-lines" => 1,
        _ => 16,
    };
    let mut rows = skein_lib::List::with_capacity(16);
    for id in 1..=count {
        let repository = match id {
            1 => {
                if name == "sheet-copy-lines" {
                    b"raft".as_slice().into()
                } else {
                    b"widget".as_slice().into()
                }
            }
            2 => b"notes".as_slice().into(),
            _ => skein_lib::bytes::concat(&[b"repository-", skein_lib::Decimal::of(id).as_bytes()]),
        };
        rows.push(
            w::Reached::new(
                &w::CEILINGS,
                w::ReachedParts {
                    id,
                    owner: w::Account::new(
                        &w::CEILINGS,
                        w::AccountParts {
                            id: 3,
                            login: if name == "sheet-copy-lines" {
                                b"octo".as_slice().into()
                            } else {
                                b"sam".as_slice().into()
                            },
                        },
                    )
                    .expect("owner"),
                    name: repository,
                    private: id == 2,
                },
            )
            .expect("row"),
        )
        .expect("page");
    }
    step(
        root,
        Event::Forge(forge::Event::SheetRead {
            request: key,
            answer: Ok(w::Page::new(
                &w::CEILINGS,
                w::PageParts { rows, more: name == "sheet-adopt" || name == "sheet-searching" },
            )
            .expect("page")),
        }),
        out,
    );
    step(
        root,
        Event::Act {
            action: Action::Forge(forge::Act::Write {
                field: forge::Text::Landing,
                text: b"main".as_slice().into(),
                typed: forge::Typed::Bytes,
            }),
        },
        out,
    );
    if name == "sheet-searching" {
        for id in [1, 2] {
            step(root, Event::Act { action: Action::Forge(forge::Act::Sheet(sheet::Act::Pick { id, on: true })) }, out);
        }
        step(
            root,
            Event::Act { action: Action::Forge(forge::Act::Sheet(sheet::Act::Find(b"widget".as_slice().into()))) },
            out,
        );
    }
    if name == "sheet-none" {
        step(
            root,
            Event::Act { action: Action::Forge(forge::Act::Sheet(sheet::Act::Find(b"sam/missing".as_slice().into()))) },
            out,
        );
    }
}
