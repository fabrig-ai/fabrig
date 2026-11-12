//! The terminal offered for a program's login (temper/docs/design/client/tui/box.md,
//! Lending its terminal; jig/docs/design/domain/terminal-login.md, 4–6).
use crate::{Domain, Limits, Request, owner::Owner, route};
use alloc::boxed::Box;
use jig_client::{Handle, Latest, State, contract as c};
use skein_lib::{Env, Queue};

#[derive(Debug)]
pub(crate) struct Terminal {
    handle: Option<Handle>,
}
impl Terminal {
    pub(crate) const fn new() -> Self {
        Self { handle: None }
    }
    pub(crate) fn login<'a>(&self, client: &'a jig_client::Client) -> Option<&'a c::TerminalLogin> {
        let document = client.document(self.handle?)?;
        if document.state != State::Live {
            return None;
        }
        let Latest::Jig(c::Delivery::Terminal(terminal)) = document.latest? else { return None };
        terminal.login.as_ref()
    }
}
pub(crate) fn offer(root: &mut Domain, env: &Env<Limits>, terminal: u64, out: &mut Queue<Request>) {
    if root.terminal.handle.is_some() {
        return;
    }
    // Accounts occupies MAX; both global watches stand outside increasing page handles.
    let handle = Handle { asker: Owner::Pages.asker(), number: u64::MAX.checked_sub(1).expect("global handle") };
    root.terminal.handle = Some(handle);
    route::client(
        root,
        env,
        jig_client::Event::Watch { handle, subject: jig_client::Watchable::Jig(c::Subject::Terminal { terminal }) },
        out,
    );
}
pub(crate) fn ended(root: &mut Domain, env: &Env<Limits>, number: u32, exit: c::LoginExit, out: &mut Queue<Request>) {
    root.pages.serial = root.pages.serial.checked_add(1).expect("page keys fit");
    let key = jig_client::Key { asker: Owner::Pages.asker(), number: root.pages.serial };
    route::client(
        root,
        env,
        jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
                holder: c::AskAccountHolder::Own,
                change: c::AccountChange::LoginEnded { number, exit },
            })),
        },
        out,
    );
}
