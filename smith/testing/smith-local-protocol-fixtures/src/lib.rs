//! Small protocol receiving profiles (protocol/host.md, section 5).
//! Shared fixtures (smith/docs/design/testing.md, section 7).

#![forbid(unsafe_code)]

use smith_local_protocol::{StoreLimits, TerminalLimits};

/// One short terminal line and bounded notice.
pub const TERMINAL: TerminalLimits = TerminalLimits { line_bytes: 32, show_bytes: 32 };

/// Two durable turns and delivery records under a small scan bound.
pub const STORE: StoreLimits = StoreLimits { file_bytes: 4096, turns: 2, deliveries: 2, scan_bytes: 4096 };
