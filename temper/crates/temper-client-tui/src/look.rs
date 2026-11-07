//! The TUI's look names (temper/docs/design/client/README.md, 6.15).
use skein_lib::Token;
use skein_ui::{Owner, Region, Style, look::Names};

pub const OWNER: Owner = Owner { number: 65_538 };
pub const TOP: Region = Region { owner: OWNER, number: 0 };
pub const BOTTOM: Region = Region { owner: OWNER, number: 1 };
pub const TITLE: Style = Style { owner: OWNER, token: 0 };
pub const HINT: Style = Style { owner: OWNER, token: 1 };
pub const LIVE: Style = Style { owner: OWNER, token: 2 };
pub const WAITING: Style = Style { owner: OWNER, token: 3 };
pub const DOWN: Style = Style { owner: OWNER, token: 4 };
pub const REFUSED: Style = Style { owner: OWNER, token: 5 };
pub const NAMES: Names<'static> = Names {
    owner: OWNER,
    prefix: b"tui",
    styles: &[b"tui-title", b"tui-hint", b"tui-live", b"tui-waiting", b"tui-down", b"tui-refused"],
    regions: &[b"tui-top", b"tui-bottom"],
};
/// The link field's binding token under OWNER.
pub const LINK: Token = Token::new(1);
/// The quit button's binding token under OWNER.
pub const QUIT: Token = Token::new(2);
