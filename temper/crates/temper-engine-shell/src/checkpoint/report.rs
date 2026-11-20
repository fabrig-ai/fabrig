//! Lines from store and child terminals (protocol/checkpoints.md, 2.1–2.3).
use jig_protocol_store::{CheckpointRefusal, Generations, OpenRefusal};
use skein_io::kernel::Exit;
use temper_engine_service::checkpoint::Outcome;

#[must_use]
pub fn lines(binary: &str, outcome: &Outcome, deployment: &[u8]) -> (String, u8) {
    let prefix = format!("{binary} checkpoint");
    match outcome {
        Outcome::Taken(checkpoint) => {
            let name = checkpoint.name.bytes();
            (
                format!(
                    "{prefix}: {} at commit {}, secrets at {}: {} files linked, {} copied ({} bytes) in {} ms; {} bytes held; {} older removed, {} skipped\n",
                    String::from_utf8_lossy(&name),
                    checkpoint.name.store,
                    checkpoint.secrets,
                    checkpoint.linked,
                    checkpoint.copied,
                    checkpoint.bytes,
                    checkpoint.took / 1_000_000,
                    checkpoint.held,
                    checkpoint.removed,
                    checkpoint.skipped
                ),
                0,
            )
        }
        Outcome::Removed(name) => (format!("{prefix}: removed {}\n", String::from_utf8_lossy(&name.bytes())), 0),
        Outcome::Refused { name: Some(name), why: CheckpointRefusal::InUse } => {
            (format!("{prefix}: {} is in use\n", String::from_utf8_lossy(&name.bytes())), 1)
        }
        Outcome::Refused { name: Some(name), why: CheckpointRefusal::Missing } => {
            (format!("{prefix}: no checkpoint {}\n", String::from_utf8_lossy(&name.bytes())), 1)
        }
        Outcome::Refused { why, .. } => (format!("{prefix}: refused: {}\n", why.words()), 1),
        Outcome::OpenRefused(OpenRefusal::Generations { why: Generations::Busy }) => (
            format!(
                "{prefix}: {} is in use by another process; while the engine serves, take one with temper ctl DIR deployment checkpoint\n",
                crate::path(deployment).display()
            ),
            1,
        ),
        Outcome::OpenRefused(why) => {
            crate::lines::store_refusal(binary, temper_engine_service::refused_start(why, deployment))
        }
        Outcome::Child(exit) => (
            String::new(),
            match exit {
                Exit::Code(code) => *code,
                Exit::Signal(signal) => u8::try_from(128_u32.saturating_add(*signal)).unwrap_or(255),
            },
        ),
        Outcome::CantRun { command, why } => {
            let why = match why {
                None => "not found",
                Some(skein_io::Error::Busy) => "no process slot",
                Some(
                    skein_io::Error::Refused
                    | skein_io::Error::Unreachable
                    | skein_io::Error::TimedOut
                    | skein_io::Error::Reset
                    | skein_io::Error::Reach(_)
                    | skein_io::Error::Other,
                ) => "io refused the command",
            };
            (format!("{prefix}: can't run {}: {why}\n", String::from_utf8_lossy(command)), 127)
        }
    }
}
