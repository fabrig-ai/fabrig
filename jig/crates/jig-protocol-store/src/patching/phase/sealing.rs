//! The last upgrade commit follows the jobs (protocol/store/opening.md, 2, step 4).
use super::{Patching, refuse, state};
use crate::{Below, Event, Limits, Store, codec, commits, format, keys};
use jig_domain_core_vocabulary::{CoreKey, CoreRecord, Deployment, Key, Record};
use skein_lib::{Env, List, Queue};

#[derive(Debug)]
pub(super) struct Sealing {
    pub(super) header: Deployment,
    pub(super) number: u64,
    pub(super) before: crate::patching::plan::Levels,
    pub(super) counts: List<crate::patching::check::Counted>,
}
pub(super) fn begin(
    store: &mut Store,
    env: &Env<Limits>,
    sealing: Sealing,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    let Sealing { mut header, number, before, counts } = sealing;
    header.commits = number;
    let Some(format_key) = keys::encode_own(&keys::Own::Format, store.limits.store.key) else {
        return refuse(store, above);
    };
    let Some(header_key) = keys::encode(&Key::Core(CoreKey::Deployment), store.limits.store.key) else {
        return refuse(store, above);
    };
    let mut format = format::current(store.application.version, header.id);
    format.levels = crate::patching::plan::binary(&store.patch_sets);
    let Ok(format) = format.encode() else {
        return refuse(store, above);
    };
    let Some(header) = codec::encode(&Record::Core(CoreRecord::Deployment(header)), &store.codec_limits, &store.names)
    else {
        return refuse(store, above);
    };
    let mut size = u64::from(skein_kv::FRAME_BYTES);
    let Some(layouts_key) = keys::encode_own(&keys::Own::Layouts, store.limits.store.key) else {
        return refuse(store, above);
    };
    if store.limits.store.ops < 3 {
        return refuse(store, above);
    }
    let mut ops = List::with_capacity(3);
    for (key, value) in [(format_key, format), (header_key, header), (layouts_key, store.layouts.encode())] {
        let Some(total) = put_size(&key, &value, size) else { return refuse(store, above) };
        size = total;
        if size > u64::from(store.limits.store.commit) {
            return refuse(store, above);
        }
        ops.push(skein_kv::Op::Put { key, value }).expect("three sealing puts");
    }
    state(store, Patching::Sealing { number, before, counts });
    commits::submit(store, env, number, ops.into_boxed(), above, below);
}

fn put_size(key: &[u8], value: &[u8], before: u64) -> Option<u64> {
    before
        .checked_add(u64::from(skein_kv::put_bytes(u32::try_from(key.len()).ok()?, u32::try_from(value.len()).ok()?)?))
}
