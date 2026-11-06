//! Peaks shared by the service and protocol layer's checked worlds.
use hearth_builder_world::fixtures::{self, File, Scenario};
// The smallest capped service queues hold every MAX_OUT. One more target
// forces a later pass while printer output is still moving through them.
pub const QUEUE: u32 = 3;
pub const TARGETS: u32 = QUEUE + 1;
pub const SEED: u64 = fixtures::SIMULATION_SEED;
#[must_use]
pub fn limits() -> hearth::Limits {
    let mut limits = fixtures::small_limits();
    limits.queue = QUEUE;
    limits.domain.render.targets = TARGETS;
    limits
}
#[must_use]
pub fn queued() -> Scenario<'static> {
    targets(TARGETS)
}

/// Many more records than the held client's output and answer queues can take.
pub const STUCK_TARGETS: u32 = 64;
#[must_use]
pub fn stuck_limits() -> hearth::Limits {
    let mut limits = limits();
    limits.io.sends = 1;
    limits.domain.render.targets = STUCK_TARGETS;
    // Six shared files and two host documents per target.
    limits.protocol.git.entries = 6 + 2 * STUCK_TARGETS;
    limits.protocol.git.paths_total = limits.protocol.git.entries * limits.protocol.git.path;
    limits
}
#[must_use]
pub fn stuck() -> Scenario<'static> {
    targets(STUCK_TARGETS)
}

fn targets(count: u32) -> Scenario<'static> {
    let mut files = fixtures::THIN.to_vec();
    for target in 0..count {
        files.push(File {
            path: format!("hosts/b{target}/b{target}.json").leak(),
            body: b"{\"motd\":{}}",
            mode: 0o644,
        });
        files.push(File {
            path: format!("hosts/b{target}/b{target}-prod.json").leak(),
            body: fixtures::reserved::METAL_EFI,
            mode: 0o644,
        });
    }
    files.retain(|file| !file.path.starts_with("hosts/x1/"));
    Scenario { files: files.leak(), ..fixtures::thin() }
}
