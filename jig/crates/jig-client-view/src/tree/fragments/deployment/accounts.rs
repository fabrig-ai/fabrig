//! Deployment account identity, ordered sources and payers (pages.md, 7).
use super::{A, P, Words, account_state, field_number, text, wall};
use jig_client::contract::{Account, AccountHolder, AccountKind, AccountUse};
use skein_ui::{Builder, Role};

pub(super) fn details(b: &mut Builder, words: &Words, account: &Account) {
    if let Some(provider) = account.provider {
        field_number(b, words, P::AccountDetail(A::Provider), u64::from(provider));
    }
    if let Some(name) = &account.provider_name {
        crate::words::literal_segment::render(b, name.bytes());
        if name.cut {
            text(b, words, P::Cut, &[]);
        }
    }
    if let Some(address) = &account.manage_usage {
        b.open(Role::Link);
        b.external(address);
        text(b, words, P::AccountDetail(A::ManageUsage), &[]);
        b.close();
    }
    crate::words::literal_segment::render(b, account.display.bytes());
    if account.display.cut {
        text(b, words, P::Cut, &[]);
    }
    match account.holder {
        AccountHolder::Deployment => text(b, words, P::AccountDetail(A::HeldByDeployment), &[]),
        AccountHolder::Party(party) => field_number(b, words, P::AccountDetail(A::HeldByParty), party),
    }
    if account.plan {
        text(b, words, P::AccountDetail(A::Plan), &[]);
    }
    super::list(b, words, P::AccountDetail(A::Sources));
    for source in &account.sources {
        b.open(Role::Item);
        field_number(b, words, P::AccountDetail(A::Source), u64::from(source.number));
        if let Some(name) = &source.name {
            crate::words::literal_segment::render(b, name);
        }
        let kind = match source.kind {
            AccountKind::SignIn => A::SignIn,
            AccountKind::Key => A::Key,
            AccountKind::Program => A::Program,
            AccountKind::None => A::None,
        };
        text(b, words, P::AccountDetail(kind), &[]);
        account_state(b, words, source.state);
        text(b, words, if source.usable { P::Usable } else { P::Unusable }, &[]);
        text(b, words, if source.expiring { P::Expiring } else { P::NotExpiring }, &[]);
        text(b, words, if source.attention { P::Attention } else { P::NoAttention }, &[]);
        wall(b, words, P::SpentUntil, P::UnknownClock, source.spent_until);
        b.close();
    }
    b.close();
    match &account.uses {
        None => text(b, words, P::AccountDetail(A::DefaultUses), &[]),
        Some(uses) => {
            for payer in uses {
                match payer {
                    AccountUse::Party(party) => field_number(b, words, P::AccountDetail(A::PaysParty), *party),
                    AccountUse::Project(project) => {
                        field_number(b, words, P::AccountDetail(A::PaysProject), u64::from(*project));
                    }
                }
            }
        }
    }
}
