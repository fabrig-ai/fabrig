//! The view's events (temper/docs/design/client/README.md, 6.14 and 6.17).
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Typed { value: Box<[u8]> },
    SignIn,
    Send,
    Command { command: temper_client_domain::pages::terminal::Command },
    Act { act: temper_client_domain::pages::Act },
    Aside { object: jig_client::contract::ObjectKey },
    Dismiss,
    Stop,
    Stay,
    Log,
    LeftOff { task: Option<u64> },
}

/// The TUI's own command (temper/docs/design/client/README.md, 6.17).
pub const LOG_COMMAND: &[u8] = b"/log";
