//! Configuration admission without run state (smith/docs/design/shell/README.md, section 2.6).

use crate::Limits;
use skein_io as io;
use smith_domain as domain;
use smith_protocol_channel as channel;
use smith_protocol_llm as llm;
use smith_protocol_machine as machine;

/// Whether the owned lower parts can be built (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn effects_valid(limits: &Limits, endpoints: &llm::Endpoints) -> bool {
    // FileIo requires four positive capacities and a positive timeout.
    limits.file_slots > 0
        && limits.file_read > 0
        && limits.file_entries > 0
        && limits.file_bytes > 0
        && limits.file_timeout.as_nanos() > 0
        && limits.machine.processes.checked_mul(2).is_some()
        && llm::component_valid(&limits.llm, endpoints)
        && machine::valid(&limits.machine)
        && smith_protocol_events::events::valid(&crate::events::sources(limits))
}

/// Whether all service parts and their IO composition fit (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits, endpoints: &llm::Endpoints) -> bool {
    effects_valid(limits, endpoints)
        && domain::valid(&limits.domain)
        && channel::valid(&limits.channel)
        && io_valid(limits)
}

/// Whether native IO tables and the shared socket composition fit (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn io_valid(limits: &Limits) -> bool {
    let Some(other) = limits.io.sockets.checked_sub(limits.llm.connection.connections) else { return false };
    let writers = match limits.events {
        Some(events) => match events.sinks.destinations.checked_add(1) {
            Some(writers) => writers,
            None => return false,
        },
        None => 1,
    };
    let child = limits.llm.connection.io;
    io::valid(&limits.io)
        && other >= writers
        && child.intake == limits.io.intake
        && child.receive == limits.io.receive
        && child.output == limits.io.output
        && child.sends == limits.io.sends
}

#[cfg(test)]
mod native_io {
    use super::*;
    use crate::fixture::small as limits;
    use skein_lib::Duration;

    #[test]
    fn invalid_limits_are_refused() {
        let base = limits();
        assert!(io_valid(&base));
        for native in [
            io::Limits { sockets: u32::MAX, ..base.io },
            io::Limits { sockets: u32::MAX / 8 + 1, ..base.io },
            io::Limits { sockets: 1 << 24, ..base.io },
            io::Limits { sockets: 1, ..base.io },
            io::Limits { sockets: 2, ..base.io },
            io::Limits { intake: base.io.intake + 1, ..base.io },
            io::Limits { receive: base.io.receive + 1, ..base.io },
            io::Limits { output: base.io.output + 1, ..base.io },
            io::Limits { sends: base.io.sends + 1, ..base.io },
            io::Limits { refusals: 0, ..base.io },
            io::Limits { accepts: 0, ..base.io },
            io::Limits { retry: Duration::ZERO, ..base.io },
        ] {
            assert!(!io_valid(&Limits { io: native, ..base }), "{native:?}");
        }
        let mut limits = base;
        limits.events = Some(smith_protocol_events::EventLimits {
            sinks: smith_protocol_events_fixtures::sinks::limits(u32::MAX),
            capture: smith_protocol_events::sinks::Capture::None,
            ..smith_protocol_events_fixtures::event_limits()
        });
        assert!(!io_valid(&limits));
        limits.events.as_mut().unwrap().sinks.destinations = base.io.sockets - base.llm.connection.connections;
        assert!(!io_valid(&limits));
        limits.events.as_mut().unwrap().sinks.destinations -= 1;
        assert!(io_valid(&limits));
        // Only the four shared buffer capacities must agree.
        limits.llm.connection.io.refusals += 1;
        assert!(io_valid(&limits));
    }
}
