//! Private scratch home and session identity shared by the live probes.
use std::os::unix::fs::PermissionsExt;

/// One live probe's private home and unique program session, removed on drop.
pub struct Probe<Home = skein_scratch::Scratch> {
    pub home: Home,
    pub session: String,
}

#[expect(clippy::new_without_default, reason = "A probe explicitly allocates a private home and unique session.")]
impl Probe {
    /// Allocate private scratch state and a unique program session for one probe.
    #[must_use]
    pub fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let number = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self::with_home(std::process::id(), number, |label, mode| {
            let home = skein_scratch::Scratch::new(label);
            std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(mode))
                .expect("private test home permissions");
            home
        })
    }
}

impl<Home> Probe<Home> {
    fn with_home(process: u32, number: u64, make_directory: impl FnOnce(&str, u32) -> Home) -> Self {
        assert!(number <= 0xffff_ffff_ffff, "probe sequence fits the session UUID");
        Self {
            home: make_directory("smith-live", 0o700),
            session: format!("{process:08x}-d94e-4ada-98a0-{number:012x}"),
        }
    }
}

#[cfg(test)]
mod tests;
