//! The connector's act, secret-free, kept with its outcome (people.md, 5.3).
use super::{asks, compound, domain, wire};

pub(super) fn encode(act: &domain::Act, limits: &wire::Limits) -> Result<wire::Act, wire::Problem> {
    Ok(match act {
        domain::Act::RemoveProject { project } => wire::Act::RemoveProject(wire::ActRemoveProject::new(
            limits,
            wire::ActRemoveProjectParts { project: *project },
        )?),
        domain::Act::Cancel { project, task, reason } => wire::Act::Cancel(wire::ActCancel::new(
            limits,
            wire::ActCancelParts { project: *project, task: *task, reason: reason.clone() },
        )?),
        domain::Act::MakeProject { maker, name, policy } => wire::Act::MakeProject(wire::ActMakeProject::new(
            limits,
            wire::ActMakeProjectParts { maker: *maker, name: name.as_bytes().into(), policy: *policy },
        )?),
        domain::Act::Close { project, task, grace, reason } => wire::Act::Close(wire::ActClose::new(
            limits,
            wire::ActCloseParts {
                project: *project,
                task: *task,
                grace: compound::encode_grace(*grace),
                reason: reason.clone(),
            },
        )?),
        domain::Act::AddKey { provider, join, digest, uses } => wire::Act::AddKey(wire::ActAddKey::new(
            limits,
            wire::ActAddKeyParts {
                provider: *provider,
                join: *join,
                digest: *digest,
                uses: asks::account_changes::encode_uses(
                    uses.as_deref(),
                    limits,
                    limits.act_add_key_uses_some,
                    wire::Path::ActAddKeyUses,
                )?,
            },
        )?),
        domain::Act::RemoveAccount { account } => wire::Act::RemoveAccount(wire::ActRemoveAccount::new(
            limits,
            wire::ActRemoveAccountParts { account: *account },
        )?),
        domain::Act::Adopt { project, adoption } => wire::Act::Adopt(wire::ActAdopt::new(
            limits,
            wire::ActAdoptParts { project: *project, adoption: asks::encode_adoption(adoption, limits)? },
        )?),
    })
}
pub(super) fn decode(act: wire::Act, limits: &wire::Limits) -> Result<domain::Act, wire::Problem> {
    Ok(match act {
        wire::Act::RemoveProject(value) => domain::Act::RemoveProject { project: value.project() },
        wire::Act::Cancel(value) => {
            let wire::ActCancelParts { project, task, reason } = value.into_parts();
            domain::Act::Cancel { project, task, reason }
        }
        wire::Act::MakeProject(value) => {
            let wire::ActMakeProjectParts { maker, name, policy } = value.into_parts();
            domain::Act::MakeProject { maker, name: asks::project_name(&name, wire::Path::ActMakeProjectName)?, policy }
        }
        wire::Act::Close(value) => {
            let wire::ActCloseParts { project, task, grace, reason } = value.into_parts();
            domain::Act::Close { project, task, grace: compound::decode_grace(grace), reason }
        }
        wire::Act::AddKey(value) => {
            let wire::ActAddKeyParts { provider, join, digest, uses } = value.into_parts();
            domain::Act::AddKey { provider, join, digest, uses: asks::account_changes::decode_uses(uses) }
        }
        wire::Act::RemoveAccount(value) => {
            let wire::ActRemoveAccountParts { account } = value.into_parts();
            domain::Act::RemoveAccount { account }
        }
        wire::Act::Adopt(value) => {
            let wire::ActAdoptParts { project, adoption } = value.into_parts();
            domain::Act::Adopt { project, adoption: asks::decode_adoption(adoption, limits)? }
        }
    })
}
