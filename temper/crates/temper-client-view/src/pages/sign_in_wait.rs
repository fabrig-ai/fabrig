//! The page while a plan's sign-in waits (temper/docs/design/client/README.md, 3.5).
use super::{accounts as account_view, welcome_model};
use crate::{
    accounts::{
        Bindings,
        words::{self, Word as W},
    },
    look,
    words::{Shell, Word, Words},
};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{Domain, pages::accounts};

#[cfg(test)]
#[path = "../tests/sign_in_wait.rs"]
mod tests;

/// Whether the browser refused the press's tab: no fact yet, the one function
/// temper/docs/design/draft/sign-in-new-tab.md, 2, feeds.
pub(super) const fn tab_refused() -> bool {
    false
}

pub(super) fn waiting(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    form: &accounts::request::Request,
    refused: bool,
    text_per_node: u32,
) {
    b.open(Role::Slot);
    b.style(look::WELCOME_AWAY);
    if w.shell == Shell::Browser {
        b.open(Role::Slot);
        b.style(look::WELCOME_SPIN);
        b.close();
    }
    b.open(Role::Slot);
    let title = welcome_model::said(
        w,
        root,
        if refused || w.shell != Shell::Browser { Word::WelcomeAwayOwn } else { Word::WelcomeAwayTitle },
    );
    b.open(Role::Status);
    b.name(&title);
    b.open(Role::Text);
    b.run(Run::Strong, &title);
    b.close();
    b.close();
    b.open(Role::Paragraph);
    b.style(look::FLOW_MUTED);
    let body = w.say(Word::WelcomeAwayBody, &[]);
    b.name(&body);
    b.text(Run::Plain, &body);
    b.close();
    if let Some(address) = form.sign_in_address(root.client()) {
        if skein_ui::markdown::safe_url(address) {
            let label =
                welcome_model::said(w, root, if refused { Word::WelcomePasteOpen } else { Word::WelcomeAwayAgain });
            let bytes = skein_lib::len_u64(label.len()).saturating_add(skein_lib::len_u64(address.len()));
            if bytes > u64::from(text_per_node) {
                crate::cards::text(
                    b,
                    Role::Alert,
                    &words::say(w, W::AddressLong, &[Hole::Number(skein_lib::len_u64(address.len()))]),
                );
            } else if w.shell == Shell::Browser {
                b.open(Role::Link);
                b.style(if refused { look::FLOW_PRIMARY } else { look::WELCOME_AGAIN });
                b.name(if refused { &label } else { label.strip_suffix(" ›".as_bytes()).unwrap_or(&label) });
                b.text(Run::Plain, &label);
                b.external(address);
                b.close();
            }
        } else {
            crate::cards::text(b, Role::Alert, &words::say(w, W::UnsafeAddress, &[]));
        }
    }
    b.close();
    b.close();
    account_view::paste(b, root, w, bindings, form, true);
}
