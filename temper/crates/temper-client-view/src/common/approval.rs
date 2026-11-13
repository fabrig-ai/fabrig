//! Shared approval items (temper/docs/design/client/view-crates.md, 3.3).
use crate::{
    forge_sign_in::Forges,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::bytes::concat;
use skein_ui::phrases::Hole;
use temper_client_domain::forge::{approvals::Row, forges::Kind};

pub(crate) fn address(row: &Row, sign_in: &Forges) -> Option<Box<[u8]>> {
    let address = if row.kind == Kind::Forgejo
        && let Some(forge) = sign_in.number(row.forge)
    {
        let repository = crate::forge::repository::web_address(b"", &row.owner, &row.name)?;
        let number = skein_lib::Decimal::of(row.pull);
        let path = concat(&[&repository, b"/pulls/", number.as_bytes(), b"/files"]);
        forge.open_path(&path)?
    } else {
        row.address.clone()
    };
    if skein_ui::markdown::safe_url(&address) { Some(address) } else { None }
}

pub(crate) fn forge(row: &Row, w: &Words) -> Box<[u8]> {
    w.say(
        match row.kind {
            Kind::Forgejo => Word::ApprovalForgejo,
            Kind::GitHub => Word::ApprovalGitHub,
        },
        &[],
    )
}

pub(crate) fn question(row: &Row, w: &Words) -> Box<[u8]> {
    w.say(Word::ApprovalQuestion, &[Hole::Name(row.title.clone()), Hole::Name(forge(row, w))])
}

pub(crate) fn review_words(row: &Row, w: &Words) -> Box<[u8]> {
    w.say(if row.sets.is_empty() { Word::ApprovalReview } else { Word::ApprovalSignOff }, &[Hole::Name(forge(row, w))])
}
