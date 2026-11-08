//! The view's events (temper/docs/design/client/README.md, 6.14).
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
}
