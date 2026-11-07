//! The view's events (temper/docs/design/client/README.md, 6.14).
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Typed { value: Box<[u8]> },
    SignIn,
    Quit,
}
