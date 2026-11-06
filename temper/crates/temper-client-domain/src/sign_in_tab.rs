//! A sign-in's tab: the press opens it, the answer points or closes it, never the state
//! (temper/docs/design/client/README.md, 3.5, `ConnectModel`, The sign-in's tab).
use crate::{Address, Domain, Request, Step, Tab};
use jig_client::{Answered, Outcome, Told, contract as c};
use skein_lib::Queue;

pub(crate) fn told(root: &mut Domain, told: &Told, out: &mut Queue<Request>) {
    let Told::Answered { key, outcome } = told else { return };
    if root.tab != Some(*key) {
        return;
    }
    root.tab = None;
    if let Some(page) = root.pages.connect_model()
        && let Some(pending) = &page.request.pending
        && pending.key == *key
        && let Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::AccountSignInStarted { source, address },
        )))) = outcome
    {
        out.push(Request::Tab { tab: Tab::Point { to: address.clone() } });
        let mut address = root.pages.address().clone();
        match &mut address {
            Address::ConnectModel { way: Some(way) } | Address::Welcome { step: Step::Model { way: Some(way) } } => {
                way.source = Some(*source);
                out.push(Request::Show { address, replace: true });
            }
            Address::Inbox { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
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
            | Address::Adopt { .. } => {}
        }
    } else {
        out.push(Request::Tab { tab: Tab::Close });
    }
}

pub(crate) fn refused(root: &mut Domain) {
    if let Some(key) = root.tab.take()
        && let Some(page) = &mut root.pages.connect_model
        && let Some(pending) = &page.request.pending
        && pending.key == key
    {
        page.tab_refused = true;
        root.tab_revision = root.tab_revision.checked_add(1).expect("tab revisions fit");
    }
}
