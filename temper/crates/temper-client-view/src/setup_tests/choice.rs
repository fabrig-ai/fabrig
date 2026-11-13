#![expect(
    clippy::disallowed_methods,
    reason = "ordinary tests inspect semantic trees and allocate maximal model-name fixtures"
)]
use super::fixtures;
pub(super) use super::fixtures::choice::account;
use jig_client::contract as c;

#[test]
fn ready_names_the_usable_account_and_not_the_model() {
    let usable = Some(c::PayerReadiness {
        provider: 17,
        payer: c::AccountUse::Party(73),
        readiness: c::Readiness::Usable,
        settled: Some(true),
    });
    // The model and its effort are chosen elsewhere: Ready names the
    // account, by its kind and who it is.
    let mut plan = account(3, c::AccountKind::SignIn, None);
    plan.provider_name = Some(c::DocumentText::complete(c::Text::new(Box::from(b"openai".as_slice()), 32).unwrap()));
    let mut key = plan.clone();
    key.plan = false;
    key.sources.get_mut(1).unwrap().kind = c::AccountKind::Key;
    let mut program = plan.clone();
    program.plan = false;
    program.provider_name =
        Some(c::DocumentText::complete(c::Text::new(Box::from(b"anthropic".as_slice()), 32).unwrap()));
    program.sources.get_mut(0).unwrap().usable = true;
    for (row, expected) in [
        (plan, "ChatGPT plan · Personal"),
        (key, "OpenAI API key · billed to the key’s organization"),
        (program, "Claude subscription · through Claude Code, signed in"),
    ] {
        let root = line_facts(usable, accounts(&[row]), None, None, None, false);
        let short = crate::pages::accounts::first_usable_short(&root, &fixtures::words()).expect("a usable account");
        assert_eq!(core::str::from_utf8(&short).unwrap(), expected);
    }
}

use super::fixtures::choice::{accounts, line_facts};
