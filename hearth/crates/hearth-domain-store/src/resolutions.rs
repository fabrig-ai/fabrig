//! Canonical resolution and closure hashes (hearth/docs/design/domain/store/records.md, 4).
use crate::{
    Flight, Standing,
    vocabulary::{Closed, Inputs},
};
use hearth_lib::{encode::Digest, hash::H};
#[must_use]
pub fn key(inputs: &Inputs) -> Digest {
    let mut hash = H::new(b"resolution", 4);
    for value in [&inputs.sources, &inputs.request, &inputs.variant, &inputs.apt] {
        for item in value {
            hash.values().item(item);
        }
    }
    hash.finish()
}
#[must_use]
pub fn digest(closure: &[Closed]) -> Digest {
    let mut hash = H::new(b"closure", 1);
    hash.values().list(u64::try_from(closure.len()).expect("bounded closure"));
    for package in closure {
        hash.values().list(4);
        hash.values().bytes(&package.name);
        hash.values().bytes(&package.architecture);
        hash.values().bytes(&package.version);
        hash.values().digest(&package.sha256);
    }
    hash.finish()
}

/// A find leaves unreadable records absent; startup alone validates their hashes.
pub(super) fn find(
    store: &mut crate::Store,
    env: &skein_lib::Env<crate::limits::Limits>,
    event: crate::vocabulary::Event,
    out: &mut skein_lib::Queue<crate::vocabulary::Output>,
) {
    use crate::vocabulary::{Answer, Event, Output, Request};
    let answer = match event {
        Event::Find { key } => match store.standing {
            Standing::Declining(why) => Answer::Declined { why },
            Standing::Usable if store.scan.resolutions.contains_key(&key) => {
                store.flight = Flight::Finding;
                out.push(Output::Request(Request::Load { key }));
                return;
            }
            Standing::Usable => Answer::Absent,
        },
        Event::Resolution { record } => Answer::Found { resolution: record },
        Event::Unreadable | Event::Failed => Answer::Absent,
        Event::Want { key } => return crate::layers::want(store, key, out),
        Event::Layer { .. } => return crate::scan::step(store, env, event, out),
        Event::Record { .. }
        | Event::Room
        | Event::Unwritable { .. }
        | Event::Open
        | Event::Locked
        | Event::Busy
        | Event::Loaded { .. }
        | Event::Page { .. }
        | Event::Done
        | Event::Arrived { .. }
        | Event::Created { .. }
        | Event::Helped { .. }
        | Event::Stage { .. }
        | Event::Seal { .. }
        | Event::Discard { .. } => unreachable!("find terminal"),
    };
    match store.flight {
        Flight::Finding => store.flight = Flight::Idle,
        Flight::Idle | Flight::Writing(_) | Flight::Entering(_) | Flight::Chain(_) => {}
    }
    out.push(Output::Answer(answer));
}

/// Admission is checked before starting a run and again before writing it.
const RESOLUTION_INDICES: u32 = 8 * 16;

pub(super) fn record(
    store: &mut crate::Store,
    env: &skein_lib::Env<crate::limits::Limits>,
    event: crate::vocabulary::Event,
    out: &mut skein_lib::Queue<crate::vocabulary::Output>,
) {
    use crate::vocabulary::{Answer, Decline, Event, Limit, Output, Request, Step};
    let answer = match event {
        Event::Room | Event::Record { .. } => {
            let decline = match store.standing {
                Standing::Declining(why) => Some(why),
                Standing::Usable if store.scan.resolutions.len() >= env.limits.resolutions => {
                    Some(Decline::Full { limit: Limit::Resolutions, most: env.limits.resolutions })
                }
                Standing::Usable
                    if event == Event::Room
                        && store.scan.index.len().saturating_add(RESOLUTION_INDICES) > env.limits.cache_files =>
                {
                    // hearth/docs/design/domain/store/records.md, 1: eight source lines, sixteen indices each.
                    Some(Decline::Full { limit: Limit::CacheFiles, most: env.limits.cache_files })
                }
                Standing::Usable => None,
            };
            if let Some(why) = decline {
                Answer::Declined { why }
            } else if let Event::Record { resolution } = event {
                store.flight = Flight::Writing(resolution.key);
                out.push(Output::Request(Request::Write { resolution }));
                return;
            } else {
                Answer::HasRoom
            }
        }
        Event::Done => {
            let key = match core::mem::replace(&mut store.flight, Flight::Idle) {
                Flight::Writing(key) => key,
                Flight::Idle | Flight::Finding | Flight::Entering(_) | Flight::Chain(_) => unreachable!("record write"),
            };
            store.scan.resolutions.insert(key, ()).expect("admitted resolution");
            Answer::Recorded
        }
        Event::Unwritable { max } => {
            store.flight = Flight::Idle;
            Answer::Declined { why: Decline::Bound { max } }
        }
        Event::Failed => {
            store.flight = Flight::Idle;
            let why = Decline::Unusable { step: Step::Write };
            store.standing = Standing::Declining(why);
            Answer::Declined { why }
        }
        Event::Want { key } => return crate::layers::want(store, key, out),
        Event::Layer { .. } => return crate::scan::step(store, env, event, out),
        Event::Find { .. }
        | Event::Resolution { .. }
        | Event::Unreadable
        | Event::Open
        | Event::Locked
        | Event::Busy
        | Event::Loaded { .. }
        | Event::Page { .. }
        | Event::Arrived { .. }
        | Event::Created { .. }
        | Event::Helped { .. }
        | Event::Stage { .. }
        | Event::Seal { .. }
        | Event::Discard { .. } => unreachable!("record terminal"),
    };
    out.push(Output::Answer(answer));
}
