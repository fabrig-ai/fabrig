//! Done rows are component records, kept by their own family (protocol/store/layouts.md, 3).
use crate::{Job, OpenRefusal, Owner, Store, keys, open, registry_v1};
use alloc::boxed::Box;
use skein_lib::List;

pub(crate) fn key(store: &Store, owner: Owner, job: &Job) -> Option<Box<[u8]>> {
    keys::encode_own(&keys::Own::Job { owner, name: Box::from(job.name) }, store.limits.store.key)
}
pub(crate) fn done(store: &mut Store, owner: Owner, job: &Job) -> Result<bool, OpenRefusal> {
    let key = key(store, owner, job).ok_or(OpenRefusal::Failed)?;
    let Some(value) = open::get_open(store, store.directory.owner, key)? else { return Ok(false) };
    let Ok(row) = registry_v1::Job::decode(&registry_v1::CEILINGS, &mut skein_lib::Reader::new(&value)) else {
        return Err(OpenRefusal::Failed);
    };
    if row.ended() == 0 || row.ended() > store.stores.store.last() {
        return Err(OpenRefusal::Failed);
    }
    Ok(true)
}
pub(crate) fn pending(store: &mut Store) -> Result<bool, OpenRefusal> {
    let mut pending = false;
    for set in store.patch_sets.sets {
        for job in set.jobs {
            if !done(store, set.owner, job)? {
                pending = true;
            }
        }
    }
    Ok(pending)
}
pub(crate) fn operation(store: &Store, owner: Owner, job: &Job, ended: u64) -> Option<skein_kv::Op> {
    let row = registry_v1::Job::new(&registry_v1::CEILINGS, registry_v1::JobParts { ended }).ok()?;
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure()));
    row.encode(&mut writer).ok()?;
    Some(skein_kv::Op::Put { key: key(store, owner, job)?, value: writer.finish() })
}
pub(crate) fn first(store: &Store, ops: &mut List<skein_kv::Op>, number: u64) -> Option<()> {
    for set in store.patch_sets.sets {
        for job in set.jobs {
            ops.push(operation(store, set.owner, job, number)?).ok()?;
        }
    }
    Some(())
}
