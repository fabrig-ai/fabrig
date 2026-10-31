//! Private scratch home and session identity shared by the live probes.
use std::os::unix::fs::PermissionsExt;

/// One live probe's private home and unique program session, removed on drop.
pub(super) struct Probe {
    pub(super) home: skein_scratch::Scratch,
    pub(super) session: String,
}

impl Probe {
    pub(super) fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let number = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        assert!(number <= 0xffff_ffff_ffff, "probe sequence fits the session UUID");
        let process = std::process::id();
        let home = skein_scratch::Scratch::new("smith-live");
        std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700))
            .expect("private test home permissions");
        Self { home, session: format!("{process:08x}-d94e-4ada-98a0-{number:012x}") }
    }
}
