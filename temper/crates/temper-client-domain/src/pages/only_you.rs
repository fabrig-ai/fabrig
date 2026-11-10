//! Only you (temper/docs/design/client/pages/README.md, Only you).
use jig_client::contract::{AccountHolder, AccountKind, AccountLine, AccountState, Accounts};
use skein_lib::List;

/// Why a source needs its person.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// `AccountState::SignInAgain`.
    SignedOut,
    /// `AccountSource::expiring`, the source not signed out.
    Expiring,
}

/// An account's deciding source that the person alone can sign in again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    /// `AccountLine::number`.
    pub account: u32,
    /// `AccountSource::number`.
    pub source: u32,
    /// `AccountKind::SignIn` or `AccountKind::Program`, never another.
    pub kind: AccountKind,
    pub why: Why,
}

/// Only you's items, in the document's order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnlyYou {
    pub items: List<Item>,
    /// The `accounts` document was `cut`: the head says "N+".
    pub cut: bool,
}

/// The rule over one document, pure: `party` the person's (`me`'s
/// `party.number`), `playground` the playground's provider, whose account
/// is the deployment's and makes no item.
#[must_use]
pub fn items(document: &Accounts, party: u64, playground: Option<u32>) -> OnlyYou {
    let mut count = 0_u32;
    for account in &document.accounts {
        if item(account, party, playground).is_some() {
            count = count.checked_add(1).expect("items fit the document's account count");
        }
    }
    let mut items = List::with_capacity(count);
    for account in &document.accounts {
        if let Some(item) = item(account, party, playground) {
            items.push(item).expect("capacity counted from the same document");
        }
    }
    OnlyYou { items, cut: document.cut }
}

fn item(account: &AccountLine, party: u64, playground: Option<u32>) -> Option<Item> {
    if account.holder != AccountHolder::Party(party) || (playground.is_some() && account.provider == playground) {
        return None;
    }
    let source = account.sources.get(0)?;
    match source.kind {
        AccountKind::SignIn | AccountKind::Program => {}
        AccountKind::Key | AccountKind::None => return None,
    }
    let why = if source.state == AccountState::SignInAgain {
        Why::SignedOut
    } else if source.expiring {
        Why::Expiring
    } else {
        return None;
    };
    Some(Item { account: account.number, source: source.number, kind: source.kind, why })
}

/// `items` over the root's accounts, person and playground provider.
/// With no document or no `me`, no items and not cut.
#[must_use]
pub fn read(root: &crate::Domain) -> OnlyYou {
    let Some(document) = root.pages().accounts_document(root.client()) else {
        return OnlyYou { items: List::with_capacity(0), cut: false };
    };
    let Some(me) = root.pages().me(root.client()) else {
        return OnlyYou { items: List::with_capacity(0), cut: false };
    };
    items(document, me.party.number, root.forge().playground_provider())
}
