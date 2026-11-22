//! Native descriptors, processes, signals and clocks (runtime/shell.md, 8).

#![cfg(unix)]

#[cfg(target_os = "macos")]
#[path = "platform.rs"]
mod darwin;
mod keeper;
mod kernel;
#[cfg(target_os = "linux")]
#[path = "platform.rs"]
mod linux;
mod posix;
mod readiness;
mod terminal;
mod tests;

use skein_shell::testing::system;
use skein_shell::{Watch, close_startup_fd, effective_user, host_name, make_root, open_root, process_id};
use skein_shell::{resolve_system_links, take_foreground, unreaped_children};

#[cfg(target_os = "linux")]
use skein_shell::{keeper as process, spawn_dying_with_parent};
