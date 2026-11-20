//! The page while a plan's sign-in waits (temper/docs/design/client/pages/README.md).
use super::{accounts as account_view, welcome_model};
use crate::{
    accounts::{
        Bindings,
        words::{self, Word as W},
    },
    look,
    words::{Shell, Word, Words},
};
use skein_ui::{Builder, Role};
use temper_client_domain::{
    Domain,
    pages::{accounts, connect_model::ConnectModel},
};
use temper_client_view_parts::away::{self, Form, Note};

#[cfg(test)]
#[path = "../tests/sign_in_wait.rs"]
mod tests;

/// Whether the browser refused the press's tab (temper/docs/design/client/pages/README.md).
#[must_use]
pub const fn tab_refused(page: &ConnectModel) -> bool {
    page.tab_refused()
}

pub fn waiting(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    form: &accounts::request::Request,
    refused: bool,
    text_per_node: u32,
) {
    let welcome = root.pages().welcome().is_some();
    let title = welcome_model::said(
        w,
        root,
        if refused || w.shell != Shell::Browser { Word::WelcomeAwayOwn } else { Word::WelcomeAwayTitle },
    );
    let link = if w.shell == Shell::Terminal {
        words::say(w, W::OpenSignIn, &[])
    } else {
        welcome_model::said(w, root, if refused { Word::WelcomePasteOpen } else { Word::WelcomeAwayAgain })
    };
    let address = match form.sign_in_address(root.client()) {
        Some(address) => Some(away::Target::External { address: skein_lib::bytes::copy_of(address) }),
        None => None,
    };
    away::waiting(
        b,
        w,
        &Note {
            title,
            body: w.say(Word::WelcomeAwayBody, &[]),
            link,
            address,
            form: if welcome { Form::Welcome } else { Form::Connection },
            refused,
        },
        text_per_node,
    );
    if !welcome {
        b.open(Role::Slot);
        b.style(look::CONNECT_ACTS);
        super::connect_model::other_ways(b, root, w, bindings);
        b.close();
    }
    account_view::paste(b, root, w, bindings, form, true);
}
