#[cfg(target_os = "macos")]
#[path = "darwin/files.rs"]
mod files;
#[path = "listener_calls.rs"]
mod listener_calls;
#[cfg(target_os = "linux")]
#[path = "linux/ring.rs"]
mod ring;
