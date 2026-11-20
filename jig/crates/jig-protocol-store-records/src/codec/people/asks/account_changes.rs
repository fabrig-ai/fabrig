//! Holder account changes retain only carried-value digests (domain/accounts.md, section 5).
use super::{domain, wire};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
pub(super) fn encode(
    value: &domain::AccountChange,
    limits: &wire::Limits,
) -> Result<wire::AccountChange, wire::Problem> {
    Ok(match value {
        domain::AccountChange::LoginProgram { provider, name } => {
            wire::AccountChange::LoginProgram(wire::AccountChangeLoginProgram {
                provider: *provider,
                name: name.clone(),
            })
        }
        domain::AccountChange::CancelLogin { number } => {
            wire::AccountChange::CancelLogin(wire::AccountChangeCancelLogin { number: *number })
        }
        domain::AccountChange::LoginEnded { number, exit } => {
            wire::AccountChange::LoginEnded(wire::AccountChangeLoginEnded {
                number: *number,
                exit: encode_exit(*exit, limits),
            })
        }
        domain::AccountChange::ReadLists { provider } => {
            wire::AccountChange::ReadLists(wire::AccountChangeReadLists { provider: *provider })
        }
        domain::AccountChange::MoveOut { account } => {
            wire::AccountChange::MoveOut(wire::AccountChangeMoveOut { account: *account })
        }
        domain::AccountChange::Written { number } => {
            wire::AccountChange::Written(wire::AccountChangeWritten { number: *number })
        }
        domain::AccountChange::FinishSignIn { source, digest } => {
            wire::AccountChange::FinishSignIn(wire::AccountChangeFinishSignIn {
                source: *source,
                digest: bytes::copy_of(digest),
            })
        }
        domain::AccountChange::MoveIn { digest } => {
            wire::AccountChange::MoveIn(wire::AccountChangeMoveIn { digest: bytes::copy_of(digest) })
        }
        domain::AccountChange::DeclareProgram { provider, join, name } => {
            wire::AccountChange::DeclareProgram(wire::AccountChangeDeclareProgram {
                provider: *provider,
                join: *join,
                name: name.clone(),
            })
        }
        domain::AccountChange::CheckProgram { source } => {
            wire::AccountChange::CheckProgram(wire::AccountChangeCheckProgram { source: *source })
        }
        domain::AccountChange::AddKey { provider, join, digest } => {
            wire::AccountChange::AddKey(wire::AccountChangeAddKey {
                provider: *provider,
                join: *join,
                digest: bytes::copy_of(digest),
            })
        }
        domain::AccountChange::ReplaceKey { source, digest } => {
            wire::AccountChange::ReplaceKey(wire::AccountChangeReplaceKey {
                source: *source,
                digest: bytes::copy_of(digest),
            })
        }
        domain::AccountChange::RemoveKey { source } => {
            wire::AccountChange::RemoveKey(wire::AccountChangeRemoveKey { source: *source })
        }
        domain::AccountChange::Resume { source } => {
            wire::AccountChange::Resume(wire::AccountChangeResume { source: *source })
        }
        domain::AccountChange::Remove { account } => {
            wire::AccountChange::Remove(wire::AccountChangeRemove { account: *account })
        }
        domain::AccountChange::MoveAccount { account, place } => {
            wire::AccountChange::MoveAccount(wire::AccountChangeMoveAccount { account: *account, place: *place })
        }
        domain::AccountChange::MoveSource { account, source, place } => {
            wire::AccountChange::MoveSource(wire::AccountChangeMoveSource {
                account: *account,
                source: *source,
                place: *place,
            })
        }
        domain::AccountChange::SetUses { account, uses } => wire::AccountChange::SetUses(wire::AccountChangeSetUses {
            account: *account,
            uses: encode_uses(uses.as_deref(), limits)?,
        }),
    })
}
pub(super) fn decode(value: wire::AccountChange) -> Result<domain::AccountChange, wire::Problem> {
    Ok(match value {
        wire::AccountChange::LoginProgram(value) => {
            domain::AccountChange::LoginProgram { provider: value.provider, name: value.name }
        }
        wire::AccountChange::CancelLogin(value) => domain::AccountChange::CancelLogin { number: value.number },
        wire::AccountChange::LoginEnded(value) => {
            domain::AccountChange::LoginEnded { number: value.number, exit: decode_exit(&value.exit) }
        }
        wire::AccountChange::MoveOut(value) => domain::AccountChange::MoveOut { account: value.account },
        wire::AccountChange::Written(value) => domain::AccountChange::Written { number: value.number },
        wire::AccountChange::FinishSignIn(value) => domain::AccountChange::FinishSignIn {
            source: value.source,
            digest: digest(value.digest, wire::paths::ACCOUNT_CHANGE_FINISH_SIGN_IN_DIGEST)?,
        },
        wire::AccountChange::MoveIn(value) => {
            domain::AccountChange::MoveIn { digest: digest(value.digest, wire::paths::ACCOUNT_CHANGE_MOVE_IN_DIGEST)? }
        }
        wire::AccountChange::DeclareProgram(value) => {
            if jig_domain_link::SourceName::new(&value.name).is_none() {
                return Err(wire::Problem {
                    path: wire::paths::ACCOUNT_CHANGE_DECLARE_PROGRAM_NAME,
                    reason: skein_codec::Reason::Malformed,
                });
            }
            domain::AccountChange::DeclareProgram { provider: value.provider, join: value.join, name: value.name }
        }
        wire::AccountChange::ReadLists(value) => domain::AccountChange::ReadLists { provider: value.provider },
        wire::AccountChange::CheckProgram(value) => domain::AccountChange::CheckProgram { source: value.source },
        wire::AccountChange::AddKey(value) => domain::AccountChange::AddKey {
            provider: value.provider,
            join: value.join,
            digest: digest(value.digest, wire::paths::ACCOUNT_CHANGE_ADD_KEY_DIGEST)?,
        },
        wire::AccountChange::ReplaceKey(value) => domain::AccountChange::ReplaceKey {
            source: value.source,
            digest: digest(value.digest, wire::paths::ACCOUNT_CHANGE_REPLACE_KEY_DIGEST)?,
        },
        wire::AccountChange::RemoveKey(value) => domain::AccountChange::RemoveKey { source: value.source },
        wire::AccountChange::Resume(value) => domain::AccountChange::Resume { source: value.source },
        wire::AccountChange::Remove(value) => domain::AccountChange::Remove { account: value.account },
        wire::AccountChange::MoveAccount(value) => {
            domain::AccountChange::MoveAccount { account: value.account, place: value.place }
        }
        wire::AccountChange::MoveSource(value) => {
            domain::AccountChange::MoveSource { account: value.account, source: value.source, place: value.place }
        }
        wire::AccountChange::SetUses(value) => {
            domain::AccountChange::SetUses { account: value.account, uses: decode_uses(value.uses) }
        }
    })
}
fn encode_uses(
    uses: Option<&[domain::Use]>,
    limits: &wire::Limits,
) -> Result<Option<List<wire::AccountUse>>, wire::Problem> {
    let Some(uses) = uses else { return Ok(None) };
    let mut out = List::with_capacity(limits.account_change_set_uses_uses_some);
    for value in uses {
        let value = match value {
            domain::Use::Party(number) => wire::AccountUse::Party(wire::AccountUseParty { number: *number }),
            domain::Use::Project(number) => wire::AccountUse::Project(wire::AccountUseProject { number: *number }),
        };
        if out.push(value).is_err() {
            return Err(wire::Problem {
                path: wire::paths::ACCOUNT_CHANGE_SET_USES_USES,
                reason: skein_codec::Reason::Bound,
            });
        }
    }
    Ok(Some(out))
}
fn decode_uses(uses: Option<List<wire::AccountUse>>) -> Option<Box<[domain::Use]>> {
    let uses = uses?;
    let mut out = List::with_capacity(uses.len());
    for value in uses.into_boxed() {
        out.push(match value {
            wire::AccountUse::Party(value) => domain::Use::Party(value.number),
            wire::AccountUse::Project(value) => domain::Use::Project(value.number),
        })
        .expect("account uses room");
    }
    Some(out.into_boxed())
}
fn digest(bytes: Box<[u8]>, path: wire::Path) -> Result<[u8; 32], wire::Problem> {
    match bytes.as_ref().try_into() {
        Ok(value) => Ok(value),
        Err(_) => Err(wire::Problem { path, reason: skein_codec::Reason::Bound }),
    }
}

fn encode_exit(value: domain::LoginExit, _limits: &wire::Limits) -> wire::LoginExit {
    match value {
        domain::LoginExit::Exited { code } => wire::LoginExit::Exited(wire::LoginExitExited { code }),
        domain::LoginExit::Killed { signal } => wire::LoginExit::Killed(wire::LoginExitKilled { signal }),
        domain::LoginExit::NotStarted => wire::LoginExit::NotStarted,
    }
}
fn decode_exit(value: &wire::LoginExit) -> domain::LoginExit {
    match value {
        wire::LoginExit::Exited(value) => domain::LoginExit::Exited { code: value.code },
        wire::LoginExit::Killed(value) => domain::LoginExit::Killed { signal: value.signal },
        wire::LoginExit::NotStarted => domain::LoginExit::NotStarted,
    }
}
