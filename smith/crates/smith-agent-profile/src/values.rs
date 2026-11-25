//! Configuration values without run state (smith/docs/design/shell/README.md, sections 2.1 and 2.6).

use crate::ProgramBackend;
use alloc::boxed::Box;
use skein_channel::StreamMode;
use skein_io as io;
use skein_lib::Duration;
use smith_domain as domain;
use smith_protocol_channel as channel;
use smith_protocol_llm as llm;
use smith_protocol_machine as machine;

/// Route names also cover component terminals that do not name native IO operations.
pub const ROUTE_HEADROOM: u32 = 64;

/// Fixed limits for one agent process and every stage it owns.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub domain: domain::Limits,
    pub channel: channel::Limits,
    pub llm: llm::ComponentLimits,
    pub machine: machine::Limits,
    pub io: io::Limits,
    pub file_slots: u32,
    pub file_read: u32,
    pub file_entries: u32,
    pub file_bytes: u32,
    pub file_timeout: Duration,
    /// Longest hold before projecting live bytes (smith/docs/design/protocol/limits.md 2.3).
    pub live_interval: Duration,
    /// Backlog watermark and maximum entries drained per iteration at each stage.
    pub queue: u32,
    /// Concurrent IO owner and kernel submission routes.
    pub routes: u32,
    /// Declared process memory in bytes, carried without a startup fit check.
    pub memory: u64,
    /// Event ownership, absent for a host that has no process stream.
    pub events: Option<smith_protocol_events::EventLimits>,
    /// The link before and after startup knows which event destinations it owns.
    pub link: Link,
}

/// Link policy from the profile, resolved once into the process's full link figures at startup.
#[derive(Clone, Copy, Debug)]
pub enum Link {
    /// Only the profile's cancellation policy, before startup has finalized event ownership.
    Policy { cancel_grace: Duration },
    /// The derived link figures used by the framed agent and its startup report.
    Derived { cancel_grace: Duration, no_progress: Duration, orphan_exit: Duration },
}

impl Link {
    /// The run's grace to answer after the host is gone.
    #[must_use]
    pub const fn cancel_grace(self) -> Duration {
        match self {
            Self::Policy { cancel_grace } | Self::Derived { cancel_grace, .. } => cancel_grace,
        }
    }
}

/// All startup inputs except the inherited descriptors, which the shell
/// adopts after constructing the service (smith/docs/design/protocol/agent.md, section 4).
#[derive(Debug)]
pub struct Config {
    pub limits: Limits,
    pub domain: domain::Config,
    pub channel_endpoints: smith_protocol_translate::Endpoints,
    pub llm_endpoints: llm::Endpoints,
    /// Executable found once on the commands' PATH, used by every listing and search.
    pub rg: Box<[u8]>,
    pub environment: machine::Environment,
    pub stream_mode: StreamMode,
    pub program: Option<ProgramBackend>,
}
