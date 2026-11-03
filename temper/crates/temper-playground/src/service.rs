//! The service's host and observation hooks (temper/docs/design/playground.md, 3–4).
use crate::{
    programs::{Harness, Parameters},
    story::Links,
};
use skein_io::kernel::{Complete, Exit, Fd, Submit};
use skein_lib::{Queue, Secret, Time, Token, Wall};
use skein_shell::Host;
use skein_sim_world::{Inherited, PipeHost, PipeWriter, Printer};
use std::{
    collections::BTreeSet,
    io::Write,
    ops::{Deref, DerefMut},
};

/// A harness owns every capture, wire, commit, cut and reserved listener here.
pub trait Observe: Sized {
    fn started(root: Fd, inherited: &Inherited) -> Self;
    /// Return true when a completion belongs to the harness's reserved listener.
    fn complete(&mut self, _complete: &Complete, _service: &mut temper_service::Service) -> bool {
        false
    }
    /// Observe completed store operations, then optionally run a reserved listener.
    /// Return true to replace the service's iteration on this turn.
    fn before_iterate(&mut self, _service: &mut temper_service::Service, _now: Time, _wall: Wall) -> bool {
        false
    }
    fn submitted(&mut self, _submit: &Submit) {}
    /// Submission counts and cut checks belong to the observer.
    fn after_iterate(&mut self, _service: &mut temper_service::Service, _submitted: u32) {}
    fn drain(&mut self, _service: &mut temper_service::Service, _now: Time) {}
    /// Override a scenario's handover answer; None reads the fake disk.
    fn handover_read(
        &mut self,
        _read: &temper_service::HandoverRead,
    ) -> Option<Result<Option<Secret>, temper_service::HandoverUnread>> {
        None
    }
    fn handover_removed(&mut self, _remove: &temper_service::HandoverRemoval) -> Option<Result<(), i32>> {
        None
    }
    /// A reserved listener can replace the service's readiness.
    fn work_pending(&self, _now: Time) -> Option<bool> {
        None
    }
    fn state_digest(&self, _service: &temper_service::Service) -> Option<[u8; 32]> {
        None
    }
}
impl Observe for () {
    fn started(_root: Fd, _inherited: &Inherited) -> Self {}
}

/// Service state available to the harness through `ServiceHost`'s dereference.
pub struct ServiceState<H: Harness> {
    pub service: temper_service::Service,
    pub observer: H::Service,
    pub parameters: Parameters,
    in_flight: BTreeSet<Token>,
    now: Time,
    output: Option<PipeWriter>,
    output_failed: bool,
}
/// The service, with an optional standard-output pipe bridge.
pub struct ServiceHost<H: Harness> {
    bare: Option<ServiceState<H>>,
    piped: Option<PipeHost<ServiceState<H>>>,
    capture: Option<(Fd, Printer)>,
}
impl<H: Harness> ServiceHost<H> {
    #[must_use]
    pub fn new(service: temper_service::Service, observer: H::Service, parameters: Parameters) -> Self {
        Self {
            bare: Some(ServiceState {
                service,
                observer,
                parameters,
                in_flight: BTreeSet::new(),
                now: Time::ZERO,
                output: None,
                output_failed: false,
            }),
            piped: None,
            capture: None,
        }
    }
    /// Attach the standard output before stepping the world.
    pub fn capture_output(&mut self, fd: Fd, printer: Printer) {
        let mut state = self.bare.take().expect("output attached once before stepping");
        let (writer, capture) =
            PipeWriter::new(fd, u32::try_from(skein_sim_world::PRINTED_BYTES).expect("output bound"));
        state.output = Some(writer);
        self.piped = Some(capture.host(state));
        self.capture = Some((fd, printer));
    }
    pub(crate) fn output_capture(&mut self) -> Option<(Fd, &mut Printer)> {
        self.capture.as_mut().map(|(fd, printer)| (*fd, printer))
    }
    fn host(&self) -> &dyn Host {
        if let Some(pipe) = &self.piped { pipe } else { self.bare.as_ref().expect("service state") }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        if let Some(pipe) = &mut self.piped { pipe } else { self.bare.as_mut().expect("service state") }
    }
}
impl<H: Harness> Deref for ServiceHost<H> {
    type Target = ServiceState<H>;
    fn deref(&self) -> &Self::Target {
        if let Some(pipe) = &self.piped { pipe.inner() } else { self.bare.as_ref().expect("service state") }
    }
}
impl<H: Harness> DerefMut for ServiceHost<H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(pipe) = &mut self.piped { pipe.inner_mut() } else { self.bare.as_mut().expect("service state") }
    }
}
impl<H: Harness> Host for ServiceHost<H> {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.host().state_digest()
    }
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
        if self.piped.as_ref().is_some_and(|pipe| pipe.failure().is_some()) {
            self.output_failed = true;
            self.service.shutdown();
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
impl<H: Harness> Host for ServiceState<H> {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.observer.state_digest(&self.service)
    }
    fn drain(&mut self) {
        while let Some(read) = self.service.take_handover_read() {
            let answer = self
                .observer
                .handover_read(&read)
                .unwrap_or_else(|| read_handover(&mut self.parameters.disk.borrow_mut(), &read));
            self.service.handover_read(read.forge, read.run, answer);
        }
        while let Some(remove) = self.service.take_handover_removal() {
            let answer = self.observer.handover_removed(&remove).unwrap_or(Ok(()));
            self.service.handover_removed(remove.forge, remove.run, answer);
        }
        if self.parameters.links == Links::Print {
            let output = self.output.as_mut().expect("Links::Print has a standard output");
            while let Some(link) = self.service.links().pop() {
                if output.write_all(&link).and_then(|()| output.write_all(b"\n")).is_err() {
                    self.output_failed = true;
                    self.service.shutdown();
                    break;
                }
            }
        }
        self.observer.drain(&mut self.service, self.now);
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.now = now;
        for _ in 0..self.service.completions().len() {
            let complete = self.service.completions().pop().expect("completion count");
            assert!(self.in_flight.remove(&complete.op), "completion has an operation in flight");
            if !self.observer.complete(&complete, &mut self.service) {
                self.service.completions().push(complete);
            }
        }
        if !self.observer.before_iterate(&mut self.service, now, wall) {
            temper_service::iterate(&mut self.service, now, wall);
        }
        for submit in self.service.submissions().iter() {
            assert!(self.in_flight.insert(submit.op), "one operation per token");
            self.observer.submitted(submit);
        }
        let submitted = self.service.submissions().len();
        self.observer.after_iterate(&mut self.service, submitted);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.service.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.service.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.observer.work_pending(now).unwrap_or_else(|| self.service.work_pending(now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.service.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.service.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.service.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.service.is_empty().then(|| match self.service.end() {
            Some(temper_service::End::Drained) if !self.output_failed => Exit::Code(0),
            _ => Exit::Code(1),
        })
    }
    fn operations(&self) -> u32 {
        self.service.operations()
    }
}
/// Read exactly the handover's fake path, bounded before allocating its contents.
///
/// # Errors
/// Returns the same closed private-file refusals as the shipped host.
pub fn read_handover(
    disk: &mut skein_fake_machine::Machine,
    read: &temper_service::HandoverRead,
) -> Result<Option<Secret>, temper_service::HandoverUnread> {
    use skein_fake_machine::{How, Is, Refusal};
    use skein_io::file::Unsafe;
    use temper_service::HandoverUnread;
    let (index, path) = crate::machine::path(&read.path);
    let root = disk.reopen_root(index);
    let opened = disk.open(root, path, How::ReadNoFollow);
    disk.close(root);
    let file = match opened {
        Ok(file) => file,
        Err(Refusal::NotFound) => return Ok(None),
        Err(Refusal::IsADirectory | Refusal::NotAFile) => return Err(HandoverUnread::NotAFile),
        Err(Refusal::Loop) => return Err(HandoverUnread::Unsafe(Unsafe::Link)),
        Err(why) => return Err(HandoverUnread::Kernel(errno(why))),
    };
    let stat = disk.stat(file);
    let result = if stat.is != Is::File {
        Err(HandoverUnread::NotAFile)
    } else if stat.owner != 1000 {
        Err(HandoverUnread::Unsafe(Unsafe::Owner { found: stat.owner, expected: 1000 }))
    } else if stat.mode & 0o077 != 0 {
        Err(HandoverUnread::Unsafe(Unsafe::Mode(stat.mode & 0o077)))
    } else if stat.links != 1 {
        Err(HandoverUnread::Unsafe(Unsafe::Links(stat.links)))
    } else if stat.size > u64::from(read.max) {
        Err(HandoverUnread::TooLarge)
    } else {
        disk.read(file, 0, read.max)
            .map(|bytes| {
                let bytes = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
                Some(Secret::new(bytes.into()))
            })
            .map_err(|why| HandoverUnread::Kernel(errno(why)))
    };
    disk.close(file);
    result
}
fn errno(why: skein_fake_machine::Refusal) -> i32 {
    use skein_fake_machine::Refusal;
    match why {
        Refusal::Held => 11,
        Refusal::NotFound => 2,
        Refusal::Exists => 17,
        Refusal::NotADirectory => 20,
        Refusal::IsADirectory => 21,
        Refusal::NotEmpty => 39,
        Refusal::Permission => 13,
        Refusal::NoSpace => 28,
        Refusal::Loop => 40,
        Refusal::NameTooLong => 36,
        Refusal::Escape => 18,
        Refusal::NotAFile | Refusal::Beneath => 22,
    }
}
