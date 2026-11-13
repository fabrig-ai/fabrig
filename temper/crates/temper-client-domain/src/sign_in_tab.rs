//! A sign-in's tab: the press opens it, the answer points or closes it, never the state
//! (temper/docs/design/client/pages/README.md, `ConnectModel`, The sign-in's tab).
use crate::{Address, Domain, Request, Step, Tab};
use jig_client::{Answered, Outcome, Told, contract as c};
use skein_lib::Queue;

pub(crate) fn told(root: &mut Domain, told: &Told, out: &mut Queue<Request>) {
    let Told::Answered { key, outcome } = told else { return };
    let pointing = root.tab == Some(*key);
    let waiting = awaiting(root, *key);
    if !pointing && !waiting {
        return;
    }
    if pointing {
        root.tab = None;
    }
    if waiting
        && let Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::AccountSignInStarted { source, address },
        )))) = outcome
    {
        if pointing {
            out.push(Request::Tab { tab: Tab::Point { to: address.clone() } });
        }
        out.push(Request::KeepSignIn { source: *source, address: address.clone(), refused: !pointing });
        let mut address = root.pages.address().clone();
        match &mut address {
            Address::ConnectModel { way: Some(way) } | Address::Welcome { step: Step::Model { way: Some(way) } } => {
                way.source = Some(*source);
                out.push(Request::Show { address, replace: true });
            }
            Address::AllHome
            | Address::Inbox { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Told
            | Address::Remembered
            | Address::Members
            | Address::Limits
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::Chat { .. }
            | Address::Talk { .. }
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
            | Address::Tell { .. } | Address::Adopt { .. } => {}
        }
    } else if pointing {
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

fn awaiting(root: &Domain, key: jig_client::Key) -> bool {
    let request = if let Some(page) = root.pages.connect_model() {
        &page.request
    } else {
        let Some(page) = root.pages.accounts() else { return false };
        &page.request
    };
    let Some(pending) = &request.pending else { return false };
    pending.key == key && pending.outcome.is_none()
}
