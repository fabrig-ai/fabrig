//! Placement protects kept personal logins (accounts.md, 3.4; domain/engine/fleet.md).

use jig_domain_accounts::Holder;

/// One bounded fact per host, independent of any run or payer.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HostRule {
    Open,
    Person(u64),
    Several,
    Unknown,
}
impl HostRule {
    #[must_use]
    pub fn allows(self, holder: Holder) -> bool {
        match self {
            Self::Open => true,
            Self::Person(person) => holder == Holder::Party(person),
            Self::Several | Self::Unknown => false,
        }
    }
}
