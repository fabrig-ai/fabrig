//! Routing the engine's drain without deciding connector work (domain/engine/restart.md, 5).
use crate::{Domain, Event, Request, Work};
use alloc::boxed::Box;
use skein_lib::{Queue, ReplyTo};

pub(super) fn defer(domain: &mut Domain, event: Event) -> Result<Event, ()> {
    if !domain.stopping() {
        return Ok(event);
    }
    match event {
        Event::ConnectorPage { .. } => Err(()),
        event @ (Event::Connector { .. }
        | Event::ConnectorTimer { .. }
        | Event::WriterRead { .. }
        | Event::ProjectionEffect { .. }
        | Event::TopicCall { .. }
        | Event::EffectCall { .. }) => {
            domain.busy.push(Box::new(event));
            Err(())
        }
        event @ (Event::Link { .. }
        | Event::ResultRead { .. }
        | Event::Stop
        | Event::WatchApplication { .. }
        | Event::ReadApplication { .. }
        | Event::Application { .. }
        | Event::StartPageRefused { .. }
        | Event::StartBegin { .. }
        | Event::Timer(_)
        | Event::Store(_)
        | Event::Engine(_)
        | Event::Core(_)
        | Event::Credential { .. }
        | Event::CallAnswered { .. }
        | Event::Text { .. }
        | Event::Turn { .. }
        | Event::Answer { .. }
        | Event::Committed { .. }
        | Event::Failed { .. }) => Ok(event),
    }
}
pub(super) fn release(domain: &mut Domain, out: &mut Queue<Request>) {
    for _ in 0..domain.busy.len() {
        out.push(Request::Busy { event: domain.busy.pop().expect("returned connector input") });
    }
}
pub(super) fn call(domain: &mut Domain, to: ReplyTo, work: &mut Queue<Work>) {
    crate::link::route(domain, skein_lib::Time::ZERO, jig_domain_hub_link::Event::Busy { to }, work);
}

impl Domain {
    #[must_use]
    pub fn stopping(&self) -> bool {
        self.engine.core().stopping()
    }
}
