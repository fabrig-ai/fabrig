#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test assertions inspect semantic trees and diagnostic redaction"
)]
use super::fixtures;
pub(super) use super::fixtures::accounts::{SOURCE, card_document, document, email_facts};
use jig_client::contract as c;
use skein_ui::Role;

#[test]
fn move_refusals_keep_their_words() {
    use c::{AccountEnd as E, AccountRefusal as R};
    for (refusal, expected) in [
        (R::NotSignedIn, b"Not signed in".as_slice()),
        (R::NotOurs, b"Not this application's record"),
        (R::Cancelled, b"The move was cancelled"),
        (R::Moving, b"Moving to another deployment"),
    ] {
        assert_eq!(sign_in_end_words(document(0), E::Refused { refusal }), expected, "{refusal:?}");
    }
}

#[test]
fn sign_in_ends_preserve_their_typed_details() {
    use c::{AccountEnd as E, AccountFailure as F, AccountRefusal as R};
    for (end, expected) in [
        (E::Made { account: 29, declined: true }, b"You signed in without plan usage.".as_slice()),
        (E::Renewed { account: 29, declined: false }, b"same account again; account 29"),
        (E::Refused { refusal: R::NotKept { why: c::AccountKeyFailure::TooLong } }, b"The key is too long."),
        (E::Refused { refusal: R::Limit { bound: c::AccountBound::Sources } }, b"Source limit reached."),
        (E::Refused { refusal: R::Kind { kind: c::AccountKind::Program } }, b"Source kind refused: Program login"),
        (E::Failed { failure: F::MissingScope { scope: 19 } }, b"Missing required scope 19"),
        (E::Failed { failure: F::Later { after: Some(skein_lib::Duration::from_secs(5)) } }, b"Try again after 5"),
    ] {
        let mut doc = document(0);
        doc.last_end = Some(c::AccountLastEnd { source: SOURCE, end });
        assert!(skein_lib::bytes::find(&sign_in_end_words(doc, end), expected).is_some());
    }
}
fn sign_in_end_words(mut doc: c::Accounts, end: c::AccountEnd) -> Vec<u8> {
    doc.last_end = Some(c::AccountLastEnd { source: SOURCE, end });
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    b.open(Role::Main);
    crate::pages::accounts::end(&mut b, &fixtures::words(), &end, &doc.accounts);
    b.close();
    let tree = b.finish();
    for (index, status) in tree.nodes().iter().enumerate() {
        if status.role != Role::Status {
            continue;
        }
        let node = tree.nodes().get(index.checked_add(1).unwrap()).unwrap();
        let mut words = Vec::new();
        for run in &node.text {
            words.extend_from_slice(&run.text);
        }
        if words.starts_with(b"Signed in") || words.starts_with("That’s".as_bytes()) {
            return words;
        }
    }
    fixtures::text(&tree).into_vec()
}
#[test]
fn a_completed_sign_in_names_the_plan_and_display() {
    for (email, display) in [
        (None, b"alice@example.com".as_slice()),
        (Some(b"alice@example.com".as_slice()), b""),
        (Some(b"alice@example.com".as_slice()), b"Different display"),
    ] {
        let mut doc = card_document(b"openai", display, c::AccountKind::SignIn);
        doc.accounts.get_mut(0).unwrap().facts = email_facts(email);
        assert_eq!(
            sign_in_end_words(doc, c::AccountEnd::Made { account: 1, declined: false }),
            b"Signed in to ChatGPT as alice@example.com"
        );
    }
}
#[test]
fn a_renewed_sign_in_explains_how_to_add_another_account() {
    for (email, display) in [
        (None, b"alice@example.com".as_slice()),
        (Some(b"alice@example.com".as_slice()), b""),
        (Some(b"alice@example.com".as_slice()), b"Different display"),
    ] {
        let mut doc = card_document(b"openai", display, c::AccountKind::SignIn);
        doc.accounts.get_mut(0).unwrap().facts = email_facts(email);
        assert_eq!(sign_in_end_words(doc, c::AccountEnd::Renewed { account: 1, declined: false }), "That’s alice@example.com again. To add another account, sign out of ChatGPT in your browser first, then Continue with ChatGPT.".as_bytes());
    }
}
#[test]
fn a_sign_in_end_keeps_the_number_when_its_account_has_not_arrived() {
    for (end, expected) in [
        (c::AccountEnd::Made { account: 29, declined: false }, b"Signed in; account 29".as_slice()),
        (c::AccountEnd::Renewed { account: 29, declined: false }, b"Signed in to the same account again; account 29"),
    ] {
        assert_eq!(sign_in_end_words(document(0), end), expected);
    }
}
#[test]
fn a_sign_in_end_without_a_display_names_the_plan_unless_renewed() {
    for (end, expected) in [
        (c::AccountEnd::Made { account: 1, declined: false }, b"Signed in to ChatGPT".as_slice()),
        (c::AccountEnd::Renewed { account: 1, declined: false }, b"Signed in to the same account again; account 1"),
    ] {
        assert_eq!(sign_in_end_words(card_document(b"openai", b"", c::AccountKind::SignIn), end), expected);
    }
}
#[test]
fn a_named_sign_in_end_still_reports_declined_plan_usage() {
    assert!(
        sign_in_end_words(
            card_document(b"openai", b"alice@example.com", c::AccountKind::SignIn),
            c::AccountEnd::Made { account: 1, declined: true }
        )
        .ends_with(b"You signed in without plan usage.")
    );
}
