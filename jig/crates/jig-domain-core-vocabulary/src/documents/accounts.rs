//! The party's accounts (`client/domain/contract/documents.md`, 1; `domain/accounts.md`, sections 3 and 5).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

use super::{DocumentLimits, Text, deployment::AccountState};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use skein_lib::{List, Wall};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SourceLine {
    pub name: Option<Box<[u8]>>,
    pub place: u32,
    pub number: u32,
    pub kind: accounts::Kind,
    pub state: AccountState,
    pub usable: bool,
    pub expiring: bool,
    pub attention: bool,
    pub spent_until: Option<Wall>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AccountLine {
    pub list: Option<AccountList>,
    pub number: u32,
    pub place: u32,
    pub uses: Option<List<accounts::Use>>,
    pub plan: bool,
    pub provider: Option<u32>,
    pub provider_name: Option<Text>,
    pub manage_usage: Option<Box<[u8]>>,
    pub display: Text,
    pub holder: accounts::Holder,
    pub sources: List<SourceLine>,
    pub facts: List<accounts::AccountFact>,
    pub cut: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SignInLine {
    pub source: u32,
    pub provider: u32,
    pub holder: accounts::Holder,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LastEnd {
    pub source: u32,
    pub end: accounts::End,
}
/// Secret-free facts of the party's accounts, including configured provider terms.
/// Per-host program facts are joined from the current fleet inventory. Uses name explicit payers,
/// or leave the default; `usable` follows those uses for this party;
/// `settled` counts the party's sign-ins and terminal login under way.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent aggregate and list loss facts")]
pub struct Accounts {
    pub program_hosts: List<super::ProgramHost>,
    pub accounts: List<AccountLine>,
    pub signing: List<SignInLine>,
    pub last_end: Option<LastEnd>,
    pub lendable: bool,
    pub lending: Option<accounts::TerminalLogin>,
    pub lent_end: Option<accounts::LentEnd>,
    pub usable: bool,
    pub settled: bool,
    pub cut: bool,
    /// Current observation from the engine selection and host checks.
    pub readiness: List<super::PayerReadiness>,
    /// Whether additional provider/payer rows were omitted.
    pub readiness_cut: bool,
}

/// Whole fact labels and text beside their fixed-size values (domain/accounts.md, 3.7).
pub fn facts_bound() -> Option<u64> {
    List::<accounts::AccountFact>::price(accounts::FACTS)?.checked_add(
        u64::from(accounts::FACTS)
            .checked_mul(u64::from(accounts::FACT_LABEL).checked_add(u64::from(accounts::FACT_TEXT))?)?,
    )
}

pub fn document_bound(limits: &DocumentLimits) -> Option<u64> {
    let account_heap = List::<SourceLine>::price(limits.lines)?
        .checked_add(u64::from(limits.lines).checked_mul(u64::from(accounts::NAME))?)?
        .checked_add(u64::from(limits.text_bytes).checked_mul(2)?)?
        .checked_add(u64::from(accounts::MANAGE_USAGE))?
        .checked_add(List::<accounts::Use>::price(accounts::USES)?)?
        .checked_add(facts_bound()?)?;
    u64::try_from(size_of::<Accounts>())
        .ok()?
        .checked_add(32)?
        .checked_add(super::programs::hosts_bound(limits)?)?
        .checked_add(List::<AccountLine>::price(limits.lines)?)?
        .checked_add(u64::from(limits.lines).checked_mul(account_heap)?)?
        .checked_add(List::<SignInLine>::price(limits.lines)?)?
        .checked_add(super::catalogue::rows_bound(limits)?)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AccountList {
    pub read: Option<Wall>,
    pub tried: Option<Wall>,
    pub failure: Option<accounts::ReadFailure>,
    pub left_out: u32,
}
