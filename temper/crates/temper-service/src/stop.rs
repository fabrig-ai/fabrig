//! Drain before closing children, close the store last (protocol/README.md, 6).
use crate::{End, Stop, engine::Engine};
use jig_protocol_config::{Path, Problems, What};
use temper_engine_domain as root;

pub(crate) struct Drain {
    pub(crate) closing: bool,
    pub(crate) requested: bool,
    stopped: Option<Stop>,
    pub(crate) unsettled: skein_lib::List<root::Unsettled>,
}
/// A second call aborts, ending at once with the reason a failure's drain
/// carries (protocol/README.md, section 4).
pub(crate) fn shutdown(engine: &mut Engine) {
    if let Some(drain) = &mut engine.drain {
        engine.end = Some(match drain.stopped.take() {
            Some(why) => End::Stopped(why),
            None => End::Aborted,
        });
        return;
    }
    engine.drain = Some(Drain {
        closing: false,
        requested: false,
        stopped: None,
        unsettled: skein_lib::List::with_capacity(engine.env.limits.forge.entries),
    });
    engine.workers.down(engine.env.now, jig_protocol_engine_workers::Event::Stop, &mut engine.events);
    engine.clients.down(jig_protocol_engine_clients::Event::Stop);
    advance(engine);
}
pub(crate) fn drained(engine: &mut Engine) {
    let Some(drain) = &mut engine.drain else {
        return;
    };
    if drain.closing {
        return;
    }
    drain.closing = true;
    engine.setup.close();
    engine.forge.down(skein_forge_connection::Request::Close);
    engine.clients.down(jig_protocol_engine_clients::Event::Close);
    engine.workers.close(engine.env.now, &mut engine.events);
    if engine.accounts.is_some() {
        engine.step(root::Event::Account(jig_domain_accounts::Event::Close));
    }
}
pub(crate) fn advance(engine: &mut Engine) {
    if engine.end.is_some() {
        return;
    }
    let Some(drain) = &mut engine.drain else {
        return;
    };
    if !drain.requested {
        // Finish public restore and the mark before closing the core's children.
        // In particular, Switch must still see the accounts in Wanted.
        if engine.domain.ready() && engine.events.room() > 0 {
            drain.requested = true;
            engine.events.push(root::Event::Stop);
        }
        return;
    }
    if !drain.closing {
        return;
    }
    engine.setup.close();
    let closed = match &engine.accounts {
        Some(accounts) => accounts.is_empty(),
        None => true,
    };
    if closed
        && engine.setup.closed
        && engine.setup.io.is_empty()
        && engine.forge.closed
        && engine.forge.io.is_empty()
        && engine.workers.closed()
        && engine.clients.closed
        && engine.clients.io.is_empty()
        && engine.store.idle()
        && engine.events.is_empty()
        && engine.requests.is_empty()
    {
        engine.store.close();
        if engine.store.closed() {
            engine.end = Some(match drain.stopped.take() {
                Some(why) => End::Stopped(why),
                None => End::Drained,
            });
        }
    }
}
pub(crate) fn frame_too_large(engine: &mut Engine, number: u64, task: u64, bytes: u32, limit: u32) {
    engine.step(root::Event::Store(root::StoreInput::Uncommitted { number }));
    engine.end = Some(End::Stopped(Stop::FrameTooLarge { number, task, bytes, limit }));
}
pub(crate) fn failed(engine: &mut Engine, number: u64) {
    engine.step(root::Event::Store(root::StoreInput::Uncommitted { number }));
    engine.end = Some(End::Stopped(Stop::CommitFailed { number }));
}
pub(crate) fn open(engine: &mut Engine, why: &jig_protocol_store::OpenRefusal) {
    engine.end = Some(End::Stopped(Stop::StoreOpen { why: why.clone() }));
}
pub(crate) fn unavailable(engine: &mut Engine, member: &[u8]) {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(member), What::Refers);
    engine.end = Some(End::Stopped(Stop::Start(problems)));
}

/// Losing the webhook listener drains the engine (protocol/forge.md, 2.3).
pub(crate) fn hooks(engine: &mut Engine) {
    if engine.drain.is_none() {
        shutdown(engine);
    }
    if let Some(drain) = &mut engine.drain
        && drain.stopped.is_none()
    {
        drain.stopped = Some(Stop::Hooks);
    }
}

/// Keep the application's page cause beside the root's named start stop.
pub(crate) fn root(engine: &mut Engine) {
    let stopped = match engine.domain.start_stop() {
        Some(root::StartStop::Core { phase, failure }) => Stop::StartRefused { phase, failure },
        Some(root::StartStop::Load(failure)) => {
            Stop::RestorePage { range: failure.range, failure: engine.store.routes.page_failure().cloned() }
        }
        Some(root::StartStop::ForgeFit { phase, failure }) => Stop::ForgeFit { phase, failure },
        Some(root::StartStop::Binding) | None => Stop::Binding,
    };
    engine.end = Some(End::Stopped(stopped));
}

pub(crate) fn requested(engine: &Engine) -> bool {
    match &engine.drain {
        Some(drain) => drain.requested,
        None => false,
    }
}
