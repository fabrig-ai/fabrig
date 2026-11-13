//! A chat's terminal: the look a person chose, the same on every chat the browser opens
//! (temper/docs/design/client/pages/README.md, A chat's terminal).

use alloc::boxed::Box;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    pub terminal: bool,
    pub theme: Theme,
    pub font: Font,
    pub night: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    DejaVu,
    Plex,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    /// ">_": the chat's pages in the terminal.
    Enter,
    /// "Exit", `/exit`: the browser's look again.
    Leave,
    /// `/theme`: the other theme.
    Theme,
    /// `/font`: the other font.
    Font,
    /// The night switch: the other of day and night, on any page.
    Night,
    /// A command pressed or sent whole.
    Run { command: Command },
    /// Close a list and clear a command prefix.
    Dismiss,
}
impl Look {
    pub(crate) fn act(&mut self, act: Act, out: &mut skein_lib::Queue<super::Request>) {
        match act {
            Act::Run { .. } | Act::Dismiss => return,
            Act::Enter => self.terminal = true,
            Act::Leave => self.terminal = false,
            Act::Night => self.night = !self.night,
            Act::Theme => {
                self.theme = match self.theme {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                }
            }
            Act::Font => {
                self.font = match self.font {
                    Font::DejaVu => Font::Plex,
                    Font::Plex => Font::DejaVu,
                }
            }
        }
        out.push(super::Request::KeepLook { look: *self });
        match act {
            Act::Enter => out.push(super::Request::Screen { on: true }),
            Act::Leave => out.push(super::Request::Screen { on: false }),
            Act::Theme | Act::Font | Act::Night | Act::Run { .. } | Act::Dismiss => {}
        }
    }
}
/// Whether the browser's terminal draws this page: a chat, talk, `NewChat` or `Ask`.
#[must_use]
pub fn draws(address: &crate::Address) -> bool {
    use crate::Address;
    match address {
        Address::Chat { .. } | Address::Talk { .. } | Address::NewChat { .. } | Address::Ask { .. } => true,
        Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Task { .. }
        | Address::Work { .. }
        | Address::Goals { .. }
        | Address::Home { .. }
        | Address::Changes { .. }
        | Address::Team { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Adopt { .. } => false,
    }
}

// The look contract requires Default; the subset forbids deriving it.
impl Default for Look {
    fn default() -> Self {
        Self { terminal: false, theme: Theme::Dark, font: Font::DejaVu, night: false }
    }
}
#[expect(clippy::derivable_impls, reason = "the strict subset forbids deriving Default")]
impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}
#[expect(clippy::derivable_impls, reason = "the strict subset forbids deriving Default")]
impl Default for Font {
    fn default() -> Self {
        Self::DejaVu
    }
}

/// Commands in the design's table's order (client/pages/README.md, Commands).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Model,
    Effort,
    Draft,
    Pause,
    Close,
    StartOver,
    Goal,
    New,
    Chats,
    Theme,
    Font,
    Help,
    Exit,
}
impl Command {
    pub const ALL: [Command; 13] = [
        Self::Model,
        Self::Effort,
        Self::Draft,
        Self::Pause,
        Self::Close,
        Self::StartOver,
        Self::Goal,
        Self::New,
        Self::Chats,
        Self::Theme,
        Self::Font,
        Self::Help,
        Self::Exit,
    ];
    #[must_use]
    pub fn name(self) -> &'static [u8] {
        match self {
            Self::Model => b"/model",
            Self::Effort => b"/effort",
            Self::Draft => b"/draft",
            Self::Pause => b"/pause",
            Self::Close => b"/close",
            Self::StartOver => b"/start-over",
            Self::Goal => b"/goal",
            Self::New => b"/new",
            Self::Chats => b"/chats",
            Self::Theme => b"/theme",
            Self::Font => b"/font",
            Self::Help => b"/help",
            Self::Exit => b"/exit",
        }
    }
    #[must_use]
    #[expect(clippy::manual_find, reason = "the subset uses bounded loops")]
    pub fn named(words: &[u8]) -> Option<Command> {
        for command in Self::ALL {
            if words == command.name() {
                return Some(command);
            }
        }
        None
    }
}
/// A command's open list under the box (client/pages/README.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Listing {
    Model,
    Effort,
    Chats,
}
/// The page's notes after the conversation, never sent (client/pages/README.md).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Note {
    Help,
    Unknown { words: Box<[u8]> },
    Font { font: Font },
    Closed,
}

#[expect(clippy::disallowed_methods, reason = "Unicode whitespace is read from the composer’s validated UTF-8")]
pub(super) fn command_words(words: &[u8]) -> bool {
    if !words.starts_with(b"/") {
        return false;
    }
    let Ok(text) = core::str::from_utf8(words) else { return false };
    for character in text.chars() {
        if character.is_whitespace() {
            return false;
        }
    }
    true
}
impl super::Pages {
    pub(crate) fn restore_look(&mut self, look: Look) {
        self.look = look;
        self.redraw();
    }
    /// The list a command opened, while it is open.
    #[must_use]
    pub fn listing(&self) -> Option<Listing> {
        self.listing
    }
    /// The terminal's notes, oldest first, at most eight.
    #[must_use]
    pub fn notes(&self) -> &[Note] {
        self.notes.as_slice()
    }
    pub(super) fn terminal_note(&mut self, note: Note) {
        if self.notes.len() == 8 {
            let mut kept = skein_lib::List::with_capacity(8);
            for prior in self.notes.as_slice().get(1..).expect("eight notes") {
                kept.push(prior.clone()).expect("seven notes fit");
            }
            self.notes = kept;
        }
        self.notes.push(note).expect("eight notes fit");
    }
    pub(super) fn terminal_words(&self) -> &[u8] {
        if let Some(chat) = &self.chat {
            &chat.words
        } else if let Some(chats) = &self.chats {
            &chats.words
        } else {
            b""
        }
    }
    fn clear_terminal_words(&mut self) {
        if let Some(chat) = &mut self.chat {
            chat.words = Box::default();
            chat.beyond = 0;
            chat.shown = None;
            chat.composer_focus = chat.composer_focus.checked_add(1).expect("focus count fits");
        } else if let Some(chats) = &mut self.chats {
            chats.words = Box::default();
            chats.beyond = 0;
        }
        self.redraw();
    }
    fn redraw(&mut self) {
        self.shown = self.shown.checked_add(1).expect("pages shown fits");
    }
    pub(super) fn terminal_act(
        &mut self,
        env: &skein_lib::Env<crate::Limits>,
        client: &jig_client::Client,
        act: Act,
        out: &mut skein_lib::Queue<super::Request>,
    ) -> Option<super::Act> {
        use super::{Act as P, chat::document::Mode};
        let command = match act {
            Act::Run { command } => command,
            Act::Dismiss => {
                if self.terminal_words().starts_with(b"/") {
                    self.clear_terminal_words();
                }
                return None;
            }
            Act::Enter | Act::Leave | Act::Theme | Act::Font | Act::Night => {
                let font = act == Act::Font;
                self.look.act(act, out);
                if font {
                    self.terminal_note(Note::Font { font: self.look.font });
                }
                return None;
            }
        };
        self.clear_terminal_words();
        let act = match command {
            Command::Model => {
                self.listing = Some(Listing::Model);
                return None;
            }
            Command::Effort => {
                self.listing = Some(Listing::Effort);
                return None;
            }
            Command::Chats => {
                if self.chats.is_none() {
                    if let Some(chat) = &self.chat {
                        self.next = self.next.max(chat.next);
                    }
                    let page = super::chats::Chats::new(self.next);
                    self.next = page.goals.number.checked_add(1).expect("page handles fit");
                    if let Some(chat) = &mut self.chat {
                        chat.next = self.next;
                    }
                    self.chats = Some(page);
                }
                let me = self.me(client);
                if let Some(page) = &mut self.chats {
                    page.configure(me, self.project, &env.limits, out);
                }
                self.listing = Some(Listing::Chats);
                return None;
            }
            Command::Draft => {
                if let Some(chat) = &self.chat {
                    if chat.draft.is_some() || chat.writing {
                        P::Mode { mode: if chat.mode == Mode::Chat { Mode::Split } else { Mode::Chat } }
                    } else {
                        P::AddDraft
                    }
                } else if self.address == (crate::Address::NewChat { draft: true }) {
                    P::Mode { mode: Mode::Chat }
                } else if self.address == (crate::Address::NewChat { draft: false }) {
                    P::Go { address: crate::Address::NewChat { draft: true } }
                } else {
                    return None;
                }
            }
            Command::Pause => P::Stop,
            Command::Close => P::Close,
            Command::StartOver => P::StartOver,
            Command::Goal => {
                let Some(chat) = &self.chat else { return None };
                let Some(draft) = &chat.draft else { return None };
                P::OpenStart {
                    draft: Some(super::chat::sheet::path(&super::chat::document::slug(&draft.title))),
                    chat: Some(chat.task),
                }
            }
            Command::New => P::Go { address: crate::Address::NewChat { draft: false } },
            Command::Theme => {
                self.look.act(Act::Theme, out);
                return None;
            }
            Command::Font => {
                self.look.act(Act::Font, out);
                self.terminal_note(Note::Font { font: self.look.font });
                return None;
            }
            Command::Help => {
                self.terminal_note(Note::Help);
                return None;
            }
            Command::Exit => {
                self.look.act(Act::Leave, out);
                return None;
            }
        };
        Some(act)
    }
    pub(super) fn terminal_send(
        &mut self,
        env: &skein_lib::Env<crate::Limits>,
        client: &jig_client::Client,
        out: &mut skein_lib::Queue<super::Request>,
    ) -> Sending {
        let reads_commands = if let crate::Address::Home { .. } = self.address { true } else { draws(&self.address) };
        if !self.look.terminal || !reads_commands {
            return Sending::Words;
        }
        let words = self.terminal_words();
        if command_words(words) {
            if let Some(command) = Command::named(words) {
                return Sending::Command(self.terminal_act(env, client, Act::Run { command }, out));
            }
            let words = skein_lib::bytes::copy_of(words);
            self.clear_terminal_words();
            self.terminal_note(Note::Unknown { words });
            return Sending::Command(None);
        }
        if let Some(chat) = &self.chat
            && let Some(task) = chat.task(client)
            && (task.result.is_some() || super::chats::ended(&task.chip))
        {
            self.clear_terminal_words();
            self.terminal_note(Note::Closed);
            return Sending::Command(None);
        }
        Sending::Words
    }
}

/// Whether a send remains words or was handled as a terminal command.
pub(super) enum Sending {
    Words,
    Command(Option<super::Act>),
}
/// Resolve terminal actions before the page's ordinary, nonrecursive dispatch.
pub(super) fn prepare(
    pages: &mut super::Pages,
    env: &skein_lib::Env<crate::Limits>,
    client: &jig_client::Client,
    event: super::Event,
    out: &mut skein_lib::Queue<super::Request>,
) -> Option<super::Event> {
    use super::{Act as P, Event};
    let event = match event {
        Event::Act { act } => {
            if act != P::Confirm && act != P::Cancel && confirmation_list(pages, client) {
                super::confirmation(pages, env, client, P::Cancel, out);
            }
            Event::Act { act }
        }
        event @ (Event::Started
        | Event::Left
        | Event::AccountKey { .. }
        | Event::SignInAddress { .. }
        | Event::SignInKept { .. }
        | Event::Went { .. }
        | Event::Told { .. }
        | Event::Push(_)
        | Event::Link { .. }) => event,
    };
    match event {
        Event::Act { act: P::Terminal(act) } => {
            pages.listing = None;
            pages.redraw();
            Some(Event::Act { act: pages.terminal_act(env, client, act, out)? })
        }
        Event::Act { act: act @ (P::Send | P::StartChat) } => {
            if pages.listing.take().is_some() {
                pages.redraw();
            }
            match pages.terminal_send(env, client, out) {
                Sending::Words => Some(Event::Act { act }),
                Sending::Command(act) => Some(Event::Act { act: act? }),
            }
        }
        Event::Act { act } => {
            if pages.listing.take().is_some() {
                pages.redraw();
            }
            Some(Event::Act { act })
        }
        event @ (Event::Started | Event::Left) => {
            if pages.listing.take().is_some() {
                pages.redraw();
            }
            pages.notes.clear();
            Some(event)
        }
        event @ (Event::AccountKey { .. }
        | Event::SignInAddress { .. }
        | Event::SignInKept { .. }
        | Event::Went { .. }
        | Event::Told { .. }
        | Event::Push(_)
        | Event::Link { .. }) => Some(event),
    }
}

fn confirmation_list(pages: &super::Pages, client: &jig_client::Client) -> bool {
    let Some(open) = &pages.confirm else { return false };
    let Some(chat) = &pages.chat else { return false };
    let Some(task) = chat.task(client) else { return false };
    if pages.look.terminal && open.act == jig_client::contract::ActKind::Accept && open.sent.is_none() {
        return true;
    }
    pages.look.terminal
        && draws(&pages.address)
        && open.act == jig_client::contract::ActKind::Cancel
        && open.object == jig_client::contract::ObjectKey::Task(chat.task)
        && open.sent.is_none()
        && chat.closing.is_some()
        && task.delegates.is_empty()
}
