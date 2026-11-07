//! A chat's terminal: the look a person chose, the same on every chat the browser opens
//! (temper/docs/design/client/README.md, 3.5, A chat's terminal).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    pub terminal: bool,
    pub theme: Theme,
    pub font: Font,
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
}
impl Look {
    pub(crate) fn act(&mut self, act: Act) {
        match act {
            Act::Enter => self.terminal = true,
            Act::Leave => self.terminal = false,
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
    }
}
/// Whether the terminal draws the page at this address: a chat's, `NewChat` or `Configure`.
#[must_use]
pub fn draws(address: &crate::Address) -> bool {
    use crate::Address;
    match address {
        Address::Chat { .. } | Address::NewChat { .. } | Address::Configure { .. } => true,
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Task { .. }
        | Address::Goals { .. }
        | Address::Home { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Adopt { .. } => false,
    }
}

// The look contract requires Default; the subset forbids deriving it.
impl Default for Look {
    fn default() -> Self {
        Self { terminal: false, theme: Theme::Dark, font: Font::DejaVu }
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
