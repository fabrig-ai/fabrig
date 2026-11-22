use crate::Limits;
use alloc::boxed::Box;
use skein_lib::Duration;
use temper_worker_domain_checkout as checkout;

pub const HOME: &[u8] = b"/srv/worker/home";
pub const SERIAL: u64 = 1000;

#[must_use]
pub fn checkout() -> checkout::Limits {
    checkout::Limits {
        workspaces: 2,
        preparations: 2,
        repositories: 2,
        name_bytes: 64,
        message_bytes: 256,
        conflicts: 4,
        path_bytes: 64,
        remote_timeout: Duration::from_secs(30),
        reds_read_ms: 5_000,
        rerun_processes: 4,
        local_timeout: Duration::from_secs(5),
        facts: 8,
    }
}
#[must_use]
pub fn limits() -> Limits {
    Limits { operations: 2, stdout_bytes: 512, stderr_bytes: 512, read_bytes: 64, sweep_entries: 4, sweep_bytes: 255 }
}

#[must_use]
pub fn settings() -> crate::Settings {
    crate::Settings {
        git: Some(Box::from(&b"/bin/git"[..])),
        home: Box::from(HOME),
        root: skein_io::kernel::Fd::new(3),
        prefix: 6_u64 << 60_u32,
        serial: SERIAL,
        largest_read: 64,
        largest_room: 16,
    }
}
