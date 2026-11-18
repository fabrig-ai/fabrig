//! The store's terminal on the client family (temper/docs/design/protocol/checkpoints.md, 3).
use alloc::boxed::Box;
use temper_client_protocol_wire::v1 as w;
use temper_engine_domain as root;

pub(super) fn answer(value: Result<root::Checkpointed, root::CheckpointWhy>, limits: &w::Limits) -> Option<w::Outcome> {
    Some(match value {
        Ok(value) => w::Outcome::Checkpointed(
            w::Checkpointed::new(
                limits,
                w::CheckpointedParts {
                    name: value.name,
                    store: value.store,
                    secrets: value.secrets,
                    linked: value.linked,
                    copied: value.copied,
                    bytes: value.bytes,
                    paused: value.paused,
                    took: value.took,
                    held: value.held,
                    removed: value.removed,
                    skipped: value.skipped,
                },
            )
            .ok()?,
        ),
        Err(why) => {
            let why = match why {
                root::CheckpointWhy::NotOwner => w::CheckpointWhy::NotOwner,
                root::CheckpointWhy::InProgress => w::CheckpointWhy::InProgress,
                root::CheckpointWhy::Unavailable => w::CheckpointWhy::Unavailable,
                root::CheckpointWhy::Files => w::CheckpointWhy::Files,
                root::CheckpointWhy::Unrecognized { path } => {
                    w::CheckpointWhy::Unrecognized(path_record(path, limits)?)
                }
                root::CheckpointWhy::Vanished { path } => w::CheckpointWhy::Vanished(path_record(path, limits)?),
                root::CheckpointWhy::Failed { path, error } => w::CheckpointWhy::Failed(
                    w::CheckpointFailed::new(
                        limits,
                        w::CheckpointFailedParts { path: path.into(), error: error.into() },
                    )
                    .ok()?,
                ),
            };
            w::Outcome::CheckpointRefused(w::CheckpointRefused::new(limits, w::CheckpointRefusedParts { why }).ok()?)
        }
    })
}
fn path_record(path: Box<str>, limits: &w::Limits) -> Option<w::CheckpointPath> {
    w::CheckpointPath::new(limits, w::CheckpointPathParts { path: path.into() }).ok()
}
