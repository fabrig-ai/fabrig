//! Route the transcript census while the settling job waits (protocol/store/jobs.md, 5.4).
use super::{Running, Waiting, refuse, state, take};
use crate::patching::phase::Patching;
use crate::transcripts::census::{Census, Next};
use crate::{Arrived, Below, Event, JobCount, Limits, Store};
use alloc::boxed::Box;
use skein_lib::{Env, Queue};

pub(super) fn resume(
    store: &mut Store,
    env: &Env<Limits>,
    mut running: Box<Running>,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    if running.census.is_none() {
        running.census = Some(Census::new());
    }
    let census = running.census.as_mut().expect("jig's census initialized before its first page");
    let Ok(next) = census.next(store) else { return refuse(store, above) };
    match next {
        Next::Done(counts) => {
            for name in ["put", "retired"] {
                running.counts.push(JobCount { name: Box::from(name), count: 0 }).expect("two store counts");
            }
            for count in counts {
                running.counts.push(count).expect("four turns counts");
            }
            running.census = None;
            running.first = false;
            state(store, Patching::Jobs(running));
            super::resume(store, env, above, below);
        }
        Next::File(request) => {
            running.waiting = Waiting::Census;
            state(store, Patching::Jobs(running));
            below.transcripts.push(request);
        }
        Next::Log(request) => {
            running.waiting = Waiting::Census;
            state(store, Patching::Jobs(running));
            let kv_env = crate::directory::env(store, env, false);
            let mut events = Queue::uncapped();
            skein_kv::down(&mut store.stores.store, &kv_env, request, &mut events, &mut below.store);
            crate::directory::events_up(store, false, &mut events, above, below);
            super::resume(store, env, above, below);
        }
    }
}
pub(in crate::patching::phase) fn file(
    store: &mut Store,
    env: &Env<Limits>,
    arrived: Arrived,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    let Some(mut running) = take_running(store) else { return refuse(store, above) };
    let Some(census) = &mut running.census else { return refuse(store, above) };
    if census.file(arrived.event).is_err() {
        return refuse(store, above);
    }
    running.waiting = Waiting::Ready;
    state(store, Patching::Jobs(running));
    super::resume(store, env, above, below);
}
fn take_running(store: &mut Store) -> Option<Box<Running>> {
    match *take(store) {
        Patching::Jobs(running) => Some(running),
        Patching::Closing { .. }
        | Patching::Dropping(_)
        | Patching::Copying(_)
        | Patching::Mapping { .. }
        | Patching::Recovering { .. }
        | Patching::Walking(_)
        | Patching::Sealing { .. }
        | Patching::Checking(_) => None,
    }
}
pub(in crate::patching::phase) fn records(
    store: &mut Store,
    records: Option<Box<[skein_kv::LogRecord]>>,
    above: &mut Queue<Event>,
) {
    let Some(mut running) = take_running(store) else { return refuse(store, above) };
    let Some(census) = &mut running.census else { return refuse(store, above) };
    if census.records(records, &store.codec_limits).is_err() {
        return refuse(store, above);
    }
    running.waiting = Waiting::Ready;
    state(store, Patching::Jobs(running));
}
