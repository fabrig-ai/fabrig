//! The TUI's look names (temper/docs/design/client/tui/theme.md and temper/docs/design/client/tui/box.md).
use skein_lib::Token;
use skein_ui::{Owner, Region, Style, look::Names};

pub const OWNER: Owner = Owner { number: 65_538 };
pub const TOP: Region = Region { owner: OWNER, number: 0 };
pub const BOTTOM: Region = Region { owner: OWNER, number: 1 };
pub const SECTIONS: Region = Region { owner: OWNER, number: 2 };
pub const WORKING: Region = Region { owner: OWNER, number: 3 };
pub const BOX: Region = Region { owner: OWNER, number: 4 };
pub const LIST: Region = Region { owner: OWNER, number: 5 };
pub const PANE: Region = Region { owner: OWNER, number: 6 };
pub const TITLE: Style = Style { owner: OWNER, token: 0 };
pub const HINT: Style = Style { owner: OWNER, token: 1 };
pub const LIVE: Style = Style { owner: OWNER, token: 2 };
pub const WAITING: Style = Style { owner: OWNER, token: 3 };
pub const DOWN: Style = Style { owner: OWNER, token: 4 };
pub const REFUSED: Style = Style { owner: OWNER, token: 5 };
pub const NAME: Style = Style { owner: OWNER, token: 6 };
pub const FAINT: Style = Style { owner: OWNER, token: 7 };
pub const ADDRESS: Style = Style { owner: OWNER, token: 8 };
pub const BYLINE: Style = Style { owner: OWNER, token: 9 };
pub const HEAD: Style = Style { owner: OWNER, token: 10 };
pub const SIGNAL: Style = Style { owner: OWNER, token: 11 };
pub const SAID: Style = Style { owner: OWNER, token: 12 };
pub const AGENT: Style = Style { owner: OWNER, token: 13 };
pub const RUN: Style = Style { owner: OWNER, token: 14 };
pub const RUN_FAILED: Style = Style { owner: OWNER, token: 15 };
pub const DOTS: Style = Style { owner: OWNER, token: 16 };
pub const RUNNING: Style = Style { owner: OWNER, token: 17 };
pub const END: Style = Style { owner: OWNER, token: 18 };
pub const FIELD: Style = Style { owner: OWNER, token: 19 };
pub const LIST_HEAD: Style = Style { owner: OWNER, token: 20 };
pub const OPTION: Style = Style { owner: OWNER, token: 21 };
pub const NOTE: Style = Style { owner: OWNER, token: 22 };
pub const SPACED: Style = Style { owner: OWNER, token: 23 };
/// Command names use plain weight while active, with descriptions kept in their own paint.
pub const COMMAND: Style = Style { owner: OWNER, token: 24 };
pub const BULLETS: Style = Style { owner: OWNER, token: 25 };
pub const LEFT_OFF: Style = Style { owner: OWNER, token: 26 };
pub const PICKED: Style = Style { owner: OWNER, token: 27 };
pub const RULE_TITLE: Style = Style { owner: OWNER, token: 28 };
pub const STEP_DONE: Style = Style { owner: OWNER, token: 29 };
pub const STEP_NOW: Style = Style { owner: OWNER, token: 30 };
pub const STEP_AHEAD: Style = Style { owner: OWNER, token: 31 };
pub const PLAYGROUND: Style = Style { owner: OWNER, token: 32 };
pub const END_FAINT: Style = Style { owner: OWNER, token: 33 };
pub const NAMES: Names<'static> = Names {
    owner: OWNER,
    prefix: b"tui",
    styles: &[
        b"tui-title",
        b"tui-hint",
        b"tui-live",
        b"tui-waiting",
        b"tui-down",
        b"tui-refused",
        b"tui-name",
        b"tui-faint",
        b"tui-address",
        b"tui-byline",
        b"tui-head",
        b"tui-signal",
        b"tui-said",
        b"tui-agent",
        b"tui-run",
        b"tui-run-failed",
        b"tui-dots",
        b"tui-running",
        b"tui-end",
        b"tui-field",
        b"tui-list-head",
        b"tui-option",
        b"tui-note",
        b"tui-spaced",
        b"tui-command",
        b"tui-bullets",
        b"tui-left-off",
        b"tui-picked",
        b"tui-rule-title",
        b"tui-step-done",
        b"tui-step-now",
        b"tui-step-ahead",
        b"tui-playground",
        b"tui-end-faint",
    ],
    regions: &[b"tui-top", b"tui-bottom", b"tui-sections", b"tui-working", b"tui-box", b"tui-list", b"tui-pane"],
};
/// The link field's binding token under OWNER.
pub const LINK: Token = Token::new(1);
