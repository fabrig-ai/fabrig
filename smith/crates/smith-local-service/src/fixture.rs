//! Small local composition edges (protocol/limits.md, section 3.8).
//! The caller supplies the native layers' receiving profiles.
use crate::{Limits, ProcessLimits};

/// Small service handoffs and envelope bound over the supplied child profiles.
#[must_use]
pub fn small(local: smith_local_domain::Limits, process: ProcessLimits) -> Limits {
    Limits { local, process, envelope_bytes: 1 << 20, queue: 64, watermark: 64 }
}

/// Small host IO edges over the supplied IO and channel profiles.
#[must_use]
pub const fn process(io: skein_io::Limits, channel: smith_host_protocol::Limits) -> ProcessLimits {
    ProcessLimits { io, channel, detail_bytes: 128, queue: 64 }
}
