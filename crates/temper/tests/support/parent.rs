//! Cleanup and bounded observations for parent-death process tests.
use std::{
    process::{Child, Command},
    time::Duration,
};

#[derive(Debug)]
pub struct Guard {
    pub child: Option<Child>,
    pub pids: Vec<u32>,
}
impl Guard {
    #[must_use]
    pub fn new(child: Child) -> Self {
        Self { child: Some(child), pids: Vec::new() }
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            let _killed = child.kill();
            let _reaped = child.wait();
        }
        for pid in &self.pids {
            signal(*pid, "KILL");
        }
    }
}
pub fn signal(pid: u32, signal: &str) {
    let _sent = Command::new("/usr/bin/kill").args([&format!("-{signal}"), &pid.to_string()]).output();
}
#[must_use]
pub fn live(pid: u32) -> bool {
    std::fs::read_to_string(format!("/proc/{pid}/stat"))
        .is_ok_and(|stat| stat.rsplit_once(") ").is_some_and(|(_, rest)| !rest.starts_with('Z')))
}
pub fn wait(mut ready: impl FnMut() -> bool) {
    let clock = skein_shell::Clock::new();
    let end = clock.now().now.saturating_add(skein_lib::Duration::from_secs(5));
    loop {
        if ready() {
            return;
        }
        assert!(clock.now().now < end, "condition settled within five seconds");
        std::thread::sleep(Duration::from_millis(10));
    }
}
pub fn descendants(pid: u32, out: &mut Vec<u32>) {
    if let Ok(children) = std::fs::read_to_string(format!("/proc/{pid}/task/{pid}/children")) {
        for child in children.split_whitespace().map(|child| child.parse().expect("child pid")) {
            out.push(child);
            descendants(child, out);
        }
    }
}
