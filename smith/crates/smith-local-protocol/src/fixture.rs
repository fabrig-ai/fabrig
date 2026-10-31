//! Small protocol receiving profiles (protocol/hosts.md, section 5).
use crate::{GitLimits, PlainLimits, StoreLimits, TerminalLimits};

/// One short terminal line and bounded notice.
pub const TERMINAL: TerminalLimits = TerminalLimits { line_bytes: 32, show_bytes: 32 };

/// Two durable turns and delivery records under a small scan bound.
pub const STORE: StoreLimits = StoreLimits { file_bytes: 4096, turns: 2, deliveries: 2, scan_bytes: 4096 };

/// Small child output, conflicts and diagnostics.
pub const GIT: GitLimits =
    GitLimits { argument_bytes: 4096, output_bytes: 64, conflicts: 2, path_bytes: 64, detail_bytes: 32 };

/// A small plain directory snapshot.
pub const PLAIN: PlainLimits = PlainLimits { entries: 2, path_bytes: 64, file_bytes: 128 };
