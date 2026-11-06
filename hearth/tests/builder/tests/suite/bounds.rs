//! Peaks shared by the service and protocol layer's checked worlds.
use hearth_builder_world::fixtures::{self, File, Scenario};
// A watermark this small, with more targets than it holds, forces a later
// pass while printer output is still moving through the layers.
pub const WATERMARK: u32 = 2;
pub const TARGETS: u32 = 4;
pub const SEED: u64 = fixtures::SIMULATION_SEED;
#[must_use]
pub fn limits() -> hearth::Limits {
    let mut limits = fixtures::small_limits();
    limits.watermark = WATERMARK;
    limits.domain.render.targets = TARGETS;
    limits
}
#[must_use]
pub fn queued() -> Scenario<'static> {
    targets(TARGETS)
}

/// Many more records than the held client's output and answer queues can take.
pub const STUCK_TARGETS: u32 = 16;
// Shared entries make a queued spec large enough for the ceiling to detect
// render running past the held output, with only sixteen targets.
const STUCK_FILES: u32 = 32;
// A 96-byte directory stays within the listing's 256-byte path limit.
const DIRECTORY_REPEATS: usize = 16;
#[must_use]
pub fn stuck_limits() -> hearth::Limits {
    let mut limits = limits();
    limits.io.sends = 1;
    limits.domain.render.targets = STUCK_TARGETS;
    // Six thin files, two host documents per target, and the shared static files.
    limits.protocol.git.entries = 6 + 2 * STUCK_TARGETS + STUCK_FILES;
    limits.protocol.git.paths_total = limits.protocol.git.entries * limits.protocol.git.path;
    limits
}
#[must_use]
pub fn stuck() -> Scenario<'static> {
    // Shared static files enlarge each queued spec without repeating their
    // contents in the revision or changing the small queued scenario.
    let mut scenario = targets(STUCK_TARGETS);
    let mut files = scenario.files.to_vec();
    for file in 0..STUCK_FILES {
        files.push(File {
            path: format!("tasks/motd/tree/etc/{}/banner{file:02}", "shared".repeat(DIRECTORY_REPEATS)).leak(),
            body: b"banner\n",
            mode: 0o644,
        });
    }
    scenario.files = files.leak();
    scenario
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
