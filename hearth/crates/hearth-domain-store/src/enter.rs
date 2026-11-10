//! Cache admission preserves the first record and never evicts (hearth/docs/design/domain/store/records.md, 8, hearth/docs/design/domain/store/README.md, 6.1).
use crate::{
    Flight, Standing, Store,
    limits::Limits,
    vocabulary::{Answer, CacheFile, Event, Limit, Output, Request, Step},
};
use hearth_lib::encode::Digest;
use skein_lib::{Env, Queue, Token};
#[derive(Debug, Hash)]
pub(super) struct Entering {
    arrival: Token,
    sha256: Digest,
    kept: bool,
    refused: bool,
}
pub(super) fn step(store: &mut Store, env: &Env<Limits>, event: Event, out: &mut Queue<Output>) {
    match event {
        Event::Arrived { arrival, sha256, size, origin, took } => {
            match store.flight {
                Flight::Idle => {}
                Flight::Finding | Flight::Writing(_) | Flight::Entering(_) | Flight::Chain(_) => {
                    unreachable!("one arrival")
                }
            }
            let present = store.scan.index.get(&sha256) == Some(&true);
            let refused =
                store.standing != Standing::Usable || (!present && store.scan.index.len() >= env.limits.cache_files);
            let kept = !present && !refused;
            store.flight = Flight::Entering(Entering { arrival, sha256, kept, refused });
            let request = if kept {
                store.scan.index.insert(sha256, true).expect("admitted cache file");
                Request::Keep { arrival, record: CacheFile { at: env.wall, origin, sha256, size, took } }
            } else {
                Request::Discard { arrival }
            };
            out.push(Output::Request(request));
        }
        Event::Done | Event::Failed => {
            let entering = match core::mem::replace(&mut store.flight, Flight::Idle) {
                Flight::Entering(entering) => entering,
                Flight::Idle | Flight::Finding | Flight::Writing(_) | Flight::Chain(_) => {
                    unreachable!("arrival terminal")
                }
            };
            let failed = event == Event::Failed;
            if failed {
                if entering.kept {
                    store.scan.index.remove(&entering.sha256);
                }
                if store.standing == Standing::Usable {
                    store.standing = Standing::Declining(crate::vocabulary::Decline::Unusable { step: Step::List });
                }
            }
            if failed {
                out.push(Output::Answer(Answer::Unusable {
                    step: if entering.kept { Step::Keep } else { Step::Remove },
                }));
            } else if entering.refused && store.standing == Standing::Usable {
                out.push(Output::Answer(Answer::Full { limit: Limit::CacheFiles, most: env.limits.cache_files }));
            }
            let answer = if entering.refused || failed {
                Answer::Refused { arrival: entering.arrival }
            } else {
                Answer::Cached { arrival: entering.arrival }
            };
            out.push(Output::Answer(answer));
        }
        Event::Want { key } => crate::layers::want(store, key, out),
        Event::Layer { .. } => crate::scan::step(store, env, event, out),
        Event::Record { .. }
        | Event::Room
        | Event::Unwritable { .. }
        | Event::Find { .. }
        | Event::Resolution { .. }
        | Event::Unreadable
        | Event::Open
        | Event::Locked
        | Event::Busy
        | Event::Loaded { .. }
        | Event::Page { .. }
        | Event::Created { .. }
        | Event::Helped { .. }
        | Event::Stage { .. }
        | Event::Seal { .. }
        | Event::Discard { .. } => {
            unreachable!("arrival terminal")
        }
    }
}
