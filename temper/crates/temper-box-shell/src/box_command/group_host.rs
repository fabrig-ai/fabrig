//! Held listeners opened only when their group asks (protocol/setup/place.md, 5).
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use temper_box_machine::host::Service;
pub(super) struct GroupHost(pub Service);
impl Host for GroupHost {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.0.iterate(now, wall);
        while let Some((forge, address)) = self.0.take_group_listener() {
            let descriptor = skein_shell::open_listener(address, 128).ok().map(|(fd, _)| fd);
            self.0.group_listener_opened(forge, descriptor);
        }
    }
    fn drain(&mut self) {
        self.0.drain();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.0.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.0.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.0.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.0.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.0.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.0.exit()
    }
    fn operations(&self) -> u32 {
        self.0.operations()
    }
}
