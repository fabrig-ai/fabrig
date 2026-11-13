//! Secret-free per-host login observations, joined at read time (accounts.md, 3.4).

use super::DocumentLimits;
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_fleet as fleet;
use jig_domain_link::{LoginState, SignOutResult};
use skein_lib::List;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProgramLogin {
    pub name: Box<[u8]>,
    pub state: Option<LoginState>,
    pub sign_out: Option<SignOutResult>,
    pub cleanup: bool,
    pub cut: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProgramHost {
    pub name: Box<[u8]>,
    pub channel: u64,
    pub connected: bool,
    pub awaiting_logins: bool,
    pub programs: List<ProgramLogin>,
    pub cut: bool,
}
pub fn logins_bound(limits: &DocumentLimits) -> Option<u64> {
    List::<ProgramLogin>::price(limits.lines)?.checked_add(
        u64::from(limits.lines).checked_mul(
            u64::from(accounts::NAME).checked_add(u64::from(limits.text_bytes.min(4_096)).checked_mul(2)?)?,
        )?,
    )
}
pub fn hosts_bound(limits: &DocumentLimits) -> Option<u64> {
    List::<ProgramHost>::price(limits.lines)?.checked_add(
        u64::from(limits.lines).checked_mul(u64::from(fleet::WORKER_NAME).checked_add(logins_bound(limits)?)?)?,
    )
}
