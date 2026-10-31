//! Agent trace observation and withheld exits belong to the hosted agent,
//! leaving process dispatch to skein's Delegate (testing.md, section 2.1).
use super::wait_in_trace;
use skein_io::kernel;
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;

/// A hosted agent with its public wait observation and exit policy.
pub struct AgentProcess {
    host: skein_world::PipeHost<smith_agent_shell::Agent>,
    wait_seen: bool,
    lingering: bool,
}

impl AgentProcess {
    pub(super) fn new(host: skein_world::PipeHost<smith_agent_shell::Agent>) -> Self {
        Self { host, wait_seen: false, lingering: false }
    }

    pub(super) fn linger(&mut self) {
        self.lingering = true;
    }

    /// Whether the agent emitted a wait call in its public trace.
    #[must_use]
    pub fn wait_seen(&self) -> bool {
        self.wait_seen
    }
}

impl Host for AgentProcess {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.host.state_digest()
    }
    fn drain(&mut self) {
        self.host.drain();
        self.wait_seen |= wait_in_trace(self.host.submissions());
    }
    fn is_empty(&self) -> bool {
        !self.lingering && self.host.is_empty()
    }
    fn exit(&self) -> Option<kernel::Exit> {
        if self.lingering { None } else { self.host.exit() }
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host.iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        self.host.completions()
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        self.host.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host.next_policy_deadline()
    }
    fn operations(&self) -> u32 {
        self.host.operations()
    }
}
