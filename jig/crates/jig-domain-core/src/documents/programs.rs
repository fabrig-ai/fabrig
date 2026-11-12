//! Secret-free per-host login observations, joined at read time (accounts.md, 3.4).

use crate::{Core, documents::DocumentLimits};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
pub use jig_domain_core_vocabulary::documents::programs::*;
use jig_domain_link::{LoginState, SignOutResult};
use skein_lib::{List, Token, bytes};
fn detail(value: &[u8], limits: &DocumentLimits, cut: &mut bool) -> Box<[u8]> {
    let bound = usize::try_from(limits.text_bytes.min(4_096)).unwrap_or(0);
    let end = value.len().min(bound);
    *cut |= end < value.len();
    bytes::copy_of(value.get(..end).expect("bounded detail prefix"))
}
fn state(value: &LoginState, limits: &DocumentLimits, cut: &mut bool) -> LoginState {
    match value {
        LoginState::Usable => LoginState::Usable,
        LoginState::NotSignedIn => LoginState::NotSignedIn,
        LoginState::SignedInOtherwise(method) => LoginState::SignedInOtherwise(*method),
        LoginState::Failed { reason, detail: text } => {
            LoginState::Failed { reason: *reason, detail: detail(text, limits, cut) }
        }
    }
}
fn signed_out(value: &SignOutResult, limits: &DocumentLimits, cut: &mut bool) -> SignOutResult {
    match value {
        SignOutResult::SignedOut => SignOutResult::SignedOut,
        SignOutResult::Failed { reason, detail: text } => {
            SignOutResult::Failed { reason: *reason, detail: detail(text, limits, cut) }
        }
    }
}
pub(super) fn logins(
    core: &Core,
    limits: &DocumentLimits,
    channel: Token,
    holder: Option<accounts::Holder>,
) -> (List<ProgramLogin>, bool) {
    let mut rows = List::with_capacity(limits.lines);
    let mut cut = false;
    if channel == Token::new(0) {
        return (rows, cut);
    }
    for name in core.fleet.program_names() {
        if let Some(holder) = holder
            && accounts::programs::owner(&core.accounts, name.as_bytes()) != Some(holder)
        {
            continue;
        }
        if rows.room() == 0 {
            cut = true;
            continue;
        }
        let mut row = ProgramLogin {
            name: bytes::copy_of(name.as_bytes()),
            state: None,
            sign_out: None,
            cleanup: core.fleet.program_cleanup(*name),
            cut: false,
        };
        if !core.fleet.awaiting_logins(channel)
            && let Some(logins) = core.fleet.program_logins(channel)
        {
            for login in logins {
                if login.name == *name {
                    row.state = Some(state(&login.state, limits, &mut row.cut));
                }
            }
        }
        if let Some(results) = core.fleet.program_sign_outs(channel)
            && let Some(result) = results.get(name)
        {
            row.sign_out = Some(signed_out(result, limits, &mut row.cut));
        }
        rows.push(row).expect("bounded program facts");
    }
    (rows, cut)
}
pub(super) fn hosts(core: &Core, limits: &crate::Limits, holder: accounts::Holder) -> (List<ProgramHost>, bool) {
    let mut hosts = List::with_capacity(limits.documents.lines);
    let mut cut = false;
    for host in &core.fleet.host_views() {
        let Some(worker) = host.worker else { continue };
        let Some(name) = core.fleet.worker_name(worker) else { continue };
        let (programs, host_cut) = logins(core, &limits.documents, host.channel, Some(holder));
        if programs.is_empty() && !host_cut {
            continue;
        }
        if hosts.room() == 0 {
            cut = true;
            continue;
        }
        hosts
            .push(ProgramHost {
                name: bytes::copy_of(name),
                channel: host.channel.raw(),
                connected: host.connected,
                awaiting_logins: core.fleet.awaiting_logins(host.channel),
                programs,
                cut: host_cut,
            })
            .expect("bounded host facts");
    }
    (hosts, cut)
}
