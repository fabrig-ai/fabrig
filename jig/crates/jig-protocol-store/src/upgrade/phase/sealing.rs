//! The last upgrade commit follows the jobs (protocol/store/opening.md, 2, step 4).
use super::{Patching, refuse, state};
use crate::{Below, Event, Limits, Store, codec, commits, format, keys, open};
use jig_domain_core_vocabulary::{CoreKey, CoreRecord, Deployment, Key, Record};
use skein_lib::{Env, List, Queue};

#[derive(Debug)]
pub(super) struct Sealing {
    pub(super) header: Deployment,
    pub(super) number: u64,
    pub(super) before: crate::upgrade::admission::Levels,
    pub(super) counts: List<crate::upgrade::check::Counted>,
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
    format.levels = crate::upgrade::admission::binary(&store.owner_sets);
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
    let capacity = 3;
    if store.limits.store.ops < capacity {
        return refuse(store, above);
    }
    let mut ops = List::with_capacity(capacity);
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

pub(super) fn opened(
    store: &mut Store,
    before: crate::upgrade::admission::Levels,
    counts: List<crate::upgrade::check::Counted>,
) -> Option<Sealing> {
    let key = Key::Core(CoreKey::Deployment);
    let encoded = keys::encode(&key, store.limits.store.key)?;
    let bytes = open::get_open(store, store.directory.owner, encoded).ok()??;
    let record = codec::decode(&key, &bytes, &store.codec_limits, &store.names)?;
    let header = match record {
        Record::Core(CoreRecord::Deployment(header)) => header,
        Record::Core(
            CoreRecord::Settings(_)
            | CoreRecord::Projection(_)
            | CoreRecord::Call(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::ProposalDecision(_)
            | CoreRecord::EscalationDecision(_),
        )
        | Record::Tasks(_)
        | Record::People(_)
        | Record::Notes(_)
        | Record::Accounts(_) => return None,
    };
    let last = store.stores.store.last();
    if header.commits != last {
        return None;
    }
    Some(Sealing { header, number: last.checked_add(1)?, before, counts })
}
