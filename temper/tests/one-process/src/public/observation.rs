//! A held disk completion makes concurrent admission observable (temper/docs/design/testing/README.md, 2.5).
use crate::world;
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Time, Wall};
use temper_playground::service::Observe;

pub(super) struct Observer {
    pub(super) inner: world::ServiceObserver,
    pub hold: bool,
    pub held: Vec<Complete>,
    pub public_heads: u32,
    pub keyed_heads: u32,
}
impl Observe for Observer {
    fn started(root: Fd, inherited: &skein_world::Inherited) -> Self {
        Self {
            inner: world::ServiceObserver::started(root, inherited),
            hold: false,
            held: Vec::new(),
            public_heads: 0,
            keyed_heads: 0,
        }
    }
    fn complete(&mut self, complete: Complete, service: &mut temper_service::Service) -> Option<Complete> {
        if let (Op::Recv { buf, .. }, Ok(Done::Count(count))) = (&complete.kind, &complete.result) {
            let bytes = &buf[..usize::try_from(*count).expect("received length")];
            if bytes.starts_with(b"GET /jig/v1/public/") {
                self.public_heads += 1;
            }
            if bytes.starts_with(b"POST /jig/v1/request ") {
                self.keyed_heads += 1;
            }
        }
        if self.hold && matches!(complete.kind, Op::Sync { .. }) {
            self.held.push(complete);
            None
        } else {
            self.inner.complete(complete, service)
        }
    }
    fn before_iterate(&mut self, service: &mut temper_service::Service, now: Time, wall: Wall) -> bool {
        if !self.hold {
            for complete in self.held.drain(..) {
                if let Some(complete) = self.inner.complete(complete, service) {
                    service.completions().push(complete);
                }
            }
        }
        self.inner.before_iterate(service, now, wall)
    }
    fn submitted(&mut self, submit: &Submit) {
        self.inner.submitted(submit);
    }
    fn after_iterate(&mut self, service: &mut temper_service::Service, submitted: u32) {
        self.inner.after_iterate(service, submitted);
    }
    fn handover_read(
        &mut self,
        read: &temper_service::HandoverRead,
    ) -> Option<Result<Option<skein_lib::Secret>, temper_service::HandoverUnread>> {
        self.inner.handover_read(read)
    }
    fn drain(&mut self, service: &mut temper_service::Service, now: Time) {
        self.inner.drain(service, now);
    }
    fn work_pending(&self, now: Time) -> Option<bool> {
        if !self.hold && !self.held.is_empty() { Some(true) } else { self.inner.work_pending(now) }
    }
}
