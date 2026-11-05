use super::{Event, Pid, Sim};

impl Sim {
    /// Records `digest` under a process's pid or `Pid::MACHINE` at the present time.
    pub fn trace_digest(&mut self, pid: Pid, digest: [u8; 32]) {
        assert!(
            pid == Pid::MACHINE || skein_lib::usize_of(pid.raw()) < self.processes.len(),
            "a process of this simulator or the machine"
        );
        self.record(pid, Event::Digest(digest));
    }
}
