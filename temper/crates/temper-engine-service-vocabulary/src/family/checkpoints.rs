//! The store's terminal on the client family (temper/docs/design/protocol/checkpoints.md, 3).
use super::maps;
use temper_client_protocol_wire::v1 as w;
use temper_client_protocol_wire::v1::Record;
use temper_engine_domain as root;

pub(super) fn answer(value: Result<root::Checkpointed, root::CheckpointWhy>, limits: &w::Limits) -> Option<w::Outcome> {
    Some(match value {
        Ok(value) => w::Outcome::Checkpointed(maps::checkpointed_to_record(&value)),
        Err(why) => {
            let why = maps::checkpoint_why_to_record(&why);
            why.measure(limits).ok()?;
            w::Outcome::CheckpointRefused(w::CheckpointRefused { why })
        }
    })
}
