//! Secret-free party account facts (client/domain/contract/documents.md, 1; domain/accounts.md, 3–5).
use super::{AccountPlace, AccountState, AccountUnread, DocumentText};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};
/// An account's payers at most: `jig_domain_accounts::USES` (domain/accounts.md, 3).
pub const ACCOUNT_USES: u32 = 16;
/// Whole public manage-usage addresses, matching configuration.md, section 3.
pub const ACCOUNT_MANAGE_USAGE: u32 = 2_048;
/// A whole ASCII HTTPS manage target safe for client rendering (client/domain/contract/documents.md, 1).
/// Configuration additionally checks host labels and ports; it accepts a subset.
#[must_use]
pub fn valid_manage_usage(value: &[u8]) -> bool {
    if value.len() > skein_lib::length::usize_of(ACCOUNT_MANAGE_USAGE) {
        return false;
    }
    let Some(host) = value.strip_prefix(b"https://") else { return false };
    if host.is_empty() || host.first() == Some(&b'/') {
        return false;
    }
    for byte in value {
        if *byte <= b' ' || *byte >= 127 || [b'@', b'?', b'#', b'\\', b'"', b'\'', b'<', b'>'].contains(byte) {
            return false;
        }
    }
    true
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[expect(clippy::struct_excessive_bools, reason = "independent aggregate and list loss facts")]
pub struct Accounts {
    pub program_hosts: List<super::ProgramHost>,
    pub accounts: List<AccountLine>,
    pub signing: List<AccountSignIn>,
    pub last_end: Option<AccountLastEnd>,
    pub lendable: bool,
    pub lending: Option<TerminalLogin>,
    pub lent_end: Option<AccountLentEnd>,
    pub usable: bool,
    pub settled: bool,
    pub cut: bool,
    /// Current observation from the engine selection and host checks.
    pub readiness: List<super::PayerReadiness>,
    /// Whether additional provider/payer rows were omitted.
    pub readiness_cut: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountLine {
    pub list: Option<AccountList>,
    pub number: u32,
    pub place: u32,
    pub uses: Option<List<AccountUse>>,
    pub plan: bool,
    pub provider: Option<u32>,
    pub provider_name: Option<DocumentText>,
    pub manage_usage: Option<Box<[u8]>>,
    pub display: DocumentText,
    pub holder: AccountHolder,
    pub sources: List<AccountSource>,
    pub facts: List<AccountFact>,
    pub cut: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountSource {
    pub name: Option<Box<[u8]>>,
    pub place: u32,
    pub number: u32,
    pub kind: AccountKind,
    pub state: AccountState,
    pub usable: bool,
    pub expiring: bool,
    pub attention: bool,
    pub spent_until: Option<Wall>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountSignIn {
    pub source: u32,
    pub provider: u32,
    pub holder: AccountHolder,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountLastEnd {
    pub source: u32,
    pub end: AccountEnd,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountUse {
    Party(u64),
    Project(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountHolder {
    Deployment,
    Party(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountKind {
    SignIn,
    Key,
    Program,
    None,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountBound {
    Accounts,
    HolderAccounts,
    Sources,
    SignIns,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountKeyFailure {
    TooLong,
    NotStored,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountRefusal {
    NotLimited,
    Uses,
    /// The program source name remains reserved after its account is forgotten.
    ProgramNameReserved,
    NotKept {
        why: AccountKeyFailure,
    },
    Limit {
        bound: AccountBound,
    },
    Held {
        provider: u32,
    },
    Provider {
        provider: u32,
    },
    Subject,
    Unknown,
    Numbers,
    Kind {
        kind: AccountKind,
    },
    Closing,
    NotStarted,
    NotSignedIn,
    NotOurs,
    Cancelled,
    Moving,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountFailure {
    Later { after: Option<Duration> },
    SignInAgain,
    MissingScope { scope: u32 },
    Misconfigured,
    Unloaded { at: AccountPlace, why: AccountUnread },
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountEnd {
    Made { account: u32, declined: bool },
    Renewed { account: u32, declined: bool },
    Refused { refusal: AccountRefusal },
    Failed { failure: AccountFailure },
}

/// A lent terminal's login (domain/terminal-login.md, 5).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Terminal {
    pub login: Option<TerminalLogin>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TerminalLogin {
    pub number: u32,
    pub provider: u32,
    pub name: Box<[u8]>,
    pub cancelled: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AccountLentEnd {
    pub number: u32,
    pub end: LoginEnd,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoginEnd {
    SignedIn { source: u32 },
    NotFinished { exit: Option<LoginExit> },
    Cancelled,
    Refused { refusal: AccountRefusal },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoginExit {
    Exited { code: u8 },
    Killed { signal: u8 },
    NotStarted,
}

/// Registry numbers the client reads (domain/accounts.md, 3.7).
pub const FACT_EMAIL: u16 = 1;
pub const FACT_NAME: u16 = 2;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountFact {
    pub kind: u16,
    pub label: Box<[u8]>,
    pub value: AccountFactValue,
    pub seen: Wall,
    pub source: Option<u32>,
    pub origin: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AccountFactValue {
    Text(Box<[u8]>),
    Count(u64),
    Instant(Wall),
    Span(Duration),
    Share(u16),
    Window(AccountWindow),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AccountWindow {
    pub length: Option<Duration>,
    pub used: Option<u16>,
    pub resets: Option<Wall>,
    pub state: u16,
}
impl AccountLine {
    #[must_use]
    pub fn email(&self) -> Option<&[u8]> {
        email(&self.facts)
    }
}
impl super::Account {
    #[must_use]
    pub fn email(&self) -> Option<&[u8]> {
        email(&self.facts)
    }
}
fn email(facts: &List<AccountFact>) -> Option<&[u8]> {
    for fact in facts {
        if fact.kind == FACT_EMAIL {
            return match &fact.value {
                AccountFactValue::Text(text) => Some(text),
                AccountFactValue::Count(_)
                | AccountFactValue::Instant(_)
                | AccountFactValue::Span(_)
                | AccountFactValue::Share(_)
                | AccountFactValue::Window(_) => None,
            };
        }
    }
    None
}

/// Public list history (jig/docs/design/client/domain/contract/documents.md, 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AccountList {
    pub read: Option<Wall>,
    pub tried: Option<Wall>,
    pub failure: Option<AccountListFailure>,
    pub left_out: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountListFailure {
    Refused { status: u16 },
    Unreachable,
    Unreadable,
    Deadline,
    Agent,
    Room,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountAnswer {
    Reading { count: u32 },
}
