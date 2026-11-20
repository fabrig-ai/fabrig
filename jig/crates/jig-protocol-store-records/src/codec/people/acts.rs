//! The connector's act, secret-free, kept with its outcome (domain/people/acts.md, 3).
use super::{asks, compound, domain, wire};

pub(super) fn encode(act: &domain::Act, limits: &wire::Limits) -> Result<wire::Act, wire::Problem> {
    Ok(match act {
        domain::Act::RemoveProject { project, grace } => wire::Act::RemoveProject(wire::ActRemoveProject {
            project: *project,
            grace: compound::encode_grace(*grace),
        }),
        domain::Act::MakeProject { maker, name, policy } => wire::Act::MakeProject(wire::ActMakeProject {
            maker: *maker,
            name: name.as_bytes().into(),
            policy: *policy,
        }),
        domain::Act::Close { project, task, grace, reason } => wire::Act::Close(wire::ActClose {
            project: *project,
            task: *task,
            grace: compound::encode_grace(*grace),
            reason: reason.clone(),
        }),
        domain::Act::AddKey { provider, join, digest } => {
            wire::Act::AddKey(wire::ActAddKey { provider: *provider, join: *join, digest: *digest })
        }
        domain::Act::RemoveAccount { account } => {
            wire::Act::RemoveAccount(wire::ActRemoveAccount { account: *account })
        }
        domain::Act::Adopt { project, adoption } => {
            wire::Act::Adopt(wire::ActAdopt { project: *project, adoption: asks::encode_adoption(adoption, limits)? })
        }
    })
}
pub(super) fn decode(act: wire::Act, limits: &wire::Limits) -> Result<domain::Act, wire::Problem> {
    Ok(match act {
        wire::Act::RemoveProject(value) => {
            domain::Act::RemoveProject { project: value.project, grace: compound::decode_grace(value.grace) }
        }
        wire::Act::MakeProject(value) => {
            let wire::ActMakeProject { maker, name, policy } = value;
            domain::Act::MakeProject {
                maker,
                name: asks::project_name(&name, wire::paths::ACT_MAKE_PROJECT_NAME)?,
                policy,
            }
        }
        wire::Act::Close(value) => {
            let wire::ActClose { project, task, grace, reason } = value;
            domain::Act::Close { project, task, grace: compound::decode_grace(grace), reason }
        }
        wire::Act::AddKey(value) => {
            let wire::ActAddKey { provider, join, digest } = value;
            domain::Act::AddKey { provider, join, digest }
        }
        wire::Act::RemoveAccount(value) => {
            let wire::ActRemoveAccount { account } = value;
            domain::Act::RemoveAccount { account }
        }
        wire::Act::Adopt(value) => {
            let wire::ActAdopt { project, adoption } = value;
            domain::Act::Adopt { project, adoption: asks::decode_adoption(adoption, limits)? }
        }
    })
}
