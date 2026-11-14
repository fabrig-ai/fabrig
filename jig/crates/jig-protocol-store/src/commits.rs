//! Root commits, numbered with skein-kv's (protocol/store/README.md, 5 and 6.2).
use crate::{CommitCheck, Event, Store, Write, codec, format, keys};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{CoreRecord, Record};
use skein_lib::{List, Map, Queue, Token};

pub(crate) fn submit(
    store: &mut Store,
    env: &skein_lib::Env<crate::Limits>,
    number: u64,
    ops: Box<[skein_kv::Op]>,
    above: &mut Queue<Event>,
    below: &mut crate::Below,
) {
    let kv_env = crate::directory::env(store, env, false);
    let mut events = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &kv_env,
        skein_kv::Request::Commit { owner: Token::new(number), ops },
        &mut events,
        &mut below.store,
    );
    crate::directory::events_up(store, false, &mut events, above, below);
}
#[derive(Debug)]
pub(crate) enum Refusal {
    Check(CommitCheck),
    FrameTooLarge { task: u64, bytes: u32, limit: u32 },
}

#[expect(clippy::too_many_lines, reason = "one ordered translation of root writes, including atomic opening erases")]
pub(crate) fn operations(
    store: &mut Store,
    number: u64,
    writes: Box<[Write]>,
    format_due: bool,
) -> Result<crate::transcripts::Pending, Refusal> {
    if writes.len() > usize::try_from(store.limits.commits.writes).ok().ok_or(Refusal::Check(CommitCheck::Writes))? {
        return Err(Refusal::Check(CommitCheck::Writes));
    }
    // Wires can erase every opening as well as the root's writes.
    let base = most_ops(store.limits.commits.writes, store.limits.orphan_erases)
        .ok_or(Refusal::Check(CommitCheck::Capacity))?;
    let capacity = base.checked_add(store.limits.store.ops).ok_or(Refusal::Check(CommitCheck::Capacity))?;
    let mut ops = List::with_capacity(capacity);
    let mut frames = Queue::uncapped();
    let mut drops = Queue::uncapped();
    let mut bytes = u64::from(skein_kv::FRAME_BYTES);
    let mut deployment = None;
    let mut wires = false;
    for write in writes {
        match write {
            Write::Wires => {
                if crate::patching::switch::pending(store) {
                    return Err(Refusal::Check(CommitCheck::Switching));
                }
                if let Err(check) = crate::openings::clear(store, &mut ops) {
                    return Err(Refusal::Check(check));
                }
                wires = true;
                let op = match operation(store, Write::Wires) {
                    Ok(op) => op,
                    Err(check) => return Err(Refusal::Check(check)),
                };
                ops.push(op).ok().ok_or(Refusal::Check(CommitCheck::Capacity))?;
            }
            Write::DropTranscript { task } => {
                if crate::patching::switch::pending(store) {
                    return Err(Refusal::Check(CommitCheck::Switching));
                }
                if let Err(check) = crate::transcripts::drop::queue(store, task, number) {
                    return Err(Refusal::Check(check));
                }
                let op = match operation(store, Write::DropTranscript { task }) {
                    Ok(op) => op,
                    Err(check) => return Err(Refusal::Check(check)),
                };
                ops.push(op).ok().ok_or(Refusal::Check(CommitCheck::Capacity))?;
                drops.push(task);
            }
            Write::Save(Record::Core(CoreRecord::Turn(turn))) => {
                if crate::patching::switch::pending(store) {
                    return Err(Refusal::Check(CommitCheck::Switching));
                }
                let body = crate::transcripts::frame::encode(&turn, &store.codec_limits)
                    .ok_or(Refusal::Check(CommitCheck::Encoding))?;
                let bytes = u32::try_from(body.len()).ok().ok_or(Refusal::Check(CommitCheck::Encoding))?;
                let limit = store.limits.store.page.bytes;
                if bytes > limit {
                    return Err(Refusal::FrameTooLarge { task: turn.task, bytes, limit });
                }
                let offset = match crate::transcripts::place(store, number, &turn, u64::from(bytes), &mut ops) {
                    Ok(value) => value,
                    Err(check) => return Err(Refusal::Check(check)),
                };
                frames.push(crate::transcripts::Frame { task: turn.task, offset, bytes: body });
            }
            write @ (Write::Save(
                Record::Core(
                    CoreRecord::Deployment(_)
                    | CoreRecord::Settings(_)
                    | CoreRecord::Projection(_)
                    | CoreRecord::Call(_)
                    | CoreRecord::RunProof(_)
                    | CoreRecord::Terminal(_)
                    | CoreRecord::ProposalDecision(_)
                    | CoreRecord::EscalationDecision(_),
                )
                | Record::Tasks(_)
                | Record::People(_)
                | Record::Notes(_)
                | Record::Accounts(_),
            )
            | Write::Opening(_)
            | Write::Copy { .. }
            | Write::Erase(_)
            | Write::Put { .. }
            | Write::Remove { .. }) => {
                if let Some(id) = header_id(&write) {
                    deployment = Some(id);
                }
                let op = match operation(store, write) {
                    Ok(op) => op,
                    Err(check) => return Err(Refusal::Check(check)),
                };
                ops.push(op).ok().ok_or(Refusal::Check(CommitCheck::Capacity))?;
            }
        }
    }
    if format_due {
        let id = deployment.expect("the first root commit saves its deployment header");
        let mut row = format::current(store.application.version, id);
        row.levels = crate::patching::plan::binary(&store.patch_sets);
        ops.push(skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Format, store.limits.store.key)
                .ok_or(Refusal::Check(CommitCheck::Encoding))?,
            value: row.encode().ok().ok_or(Refusal::Check(CommitCheck::Encoding))?,
        })
        .ok()
        .ok_or(Refusal::Check(CommitCheck::Capacity))?;
        ops.push(skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Layouts, store.limits.store.key)
                .ok_or(Refusal::Check(CommitCheck::Encoding))?,
            value: store.layouts.encode(),
        })
        .ok()
        .ok_or(Refusal::Check(CommitCheck::Capacity))?;
        ops.push(skein_kv::Op::Put {
            key: keys::installation(store.limits.store.key).ok_or(Refusal::Check(CommitCheck::Encoding))?,
            value: codec::accounts::Installation { id: store.installation }.encode(),
        })
        .ok()
        .ok_or(Refusal::Check(CommitCheck::Capacity))?;
    }
    if let Err(check) = crate::transcripts::drop::erases(store, &mut ops) {
        return Err(Refusal::Check(check));
    }
    let mut bounded = List::with_capacity(capacity);
    for op in unique(ops.into_boxed(), capacity).ok_or(Refusal::Check(CommitCheck::Capacity))? {
        if let Err(check) = append(&mut bounded, op, &mut bytes, store.limits.store.commit) {
            return Err(Refusal::Check(check));
        }
    }
    store.mark.accept(wires, number);
    Ok(crate::transcripts::Pending { number, frames, ops: bounded.into_boxed(), drops })
}
fn operation(store: &mut Store, write: Write) -> Result<skein_kv::Op, CommitCheck> {
    let limit = store.limits.store.key;
    match write {
        Write::Wires => Ok(skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Wires, limit).ok_or(CommitCheck::Encoding)?,
            value: codec::component::wires(store.wires),
        }),
        Write::Opening(opening) => {
            let key =
                keys::encode_own(&keys::Own::Opening(opening.worker.clone()), limit).ok_or(CommitCheck::Encoding)?;
            let value = codec::openings::encode(opening).ok_or(CommitCheck::Encoding)?;
            store.opening_keys.insert(key.clone(), ()).or(Err(CommitCheck::Capacity))?;
            Ok(skein_kv::Op::Put { key, value })
        }
        Write::Copy { from, to } => copy(store, from, to),
        Write::Save(record) => Ok(skein_kv::Op::Put {
            key: keys::encode(&record.key(), limit).ok_or(CommitCheck::Encoding)?,
            value: codec::encode(&record, &store.codec_limits, &store.names).ok_or(CommitCheck::Encoding)?,
        }),
        Write::Erase(key) => Ok(skein_kv::Op::Erase { key: keys::encode(&key, limit).ok_or(CommitCheck::Encoding)? }),
        Write::Put { key, value } => {
            Ok(skein_kv::Op::Put { key: keys::application(&key, limit).ok_or(CommitCheck::Encoding)?, value })
        }
        Write::Remove { key } => {
            Ok(skein_kv::Op::Erase { key: keys::application(&key, limit).ok_or(CommitCheck::Encoding)? })
        }
        Write::DropTranscript { task } => Ok(skein_kv::Op::Erase {
            key: keys::encode_own(&keys::Own::Transcript { task }, limit).ok_or(CommitCheck::Encoding)?,
        }),
    }
}

fn header_id(write: &Write) -> Option<[u8; 16]> {
    match write {
        Write::Save(Record::Core(CoreRecord::Deployment(header))) => Some(header.id),
        Write::Save(
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
            | Record::Accounts(_),
        )
        | Write::Copy { .. }
        | Write::Wires
        | Write::Opening(_)
        | Write::Erase(_)
        | Write::Put { .. }
        | Write::Remove { .. }
        | Write::DropTranscript { .. } => None,
    }
}

fn append(ops: &mut List<skein_kv::Op>, op: skein_kv::Op, bytes: &mut u64, limit: u32) -> Result<(), CommitCheck> {
    let key = u32::try_from(op.key().len()).ok().ok_or(CommitCheck::Bytes)?;
    let size = match &op {
        skein_kv::Op::Put { key: _, value } => {
            skein_kv::put_bytes(key, u32::try_from(value.len()).ok().ok_or(CommitCheck::Bytes)?)
                .ok_or(CommitCheck::Bytes)?
        }
        skein_kv::Op::Erase { key: _ } => skein_kv::erase_bytes(key).ok_or(CommitCheck::Bytes)?,
    };
    let total = bytes.checked_add(u64::from(size)).ok_or(CommitCheck::Bytes)?;
    if total > u64::from(limit) {
        return Err(CommitCheck::Bytes);
    }
    ops.push(op).or(Err(CommitCheck::Capacity))?;
    *bytes = total;
    Ok(())
}

/// Last writes replace their first slot; encoded keys are the map's identity.
fn unique(ops: Box<[skein_kv::Op]>, capacity: u32) -> Option<Box<[skein_kv::Op]>> {
    let mut positions = Map::with_capacity(capacity);
    let mut kept = List::with_capacity(capacity);
    for op in ops {
        if let Some(position) = positions.get(op.key()).copied() {
            *kept.get_mut(position).expect("each indexed operation has its first slot") = op;
        } else {
            positions.insert(skein_lib::bytes::copy_of(op.key()), kept.len()).ok()?;
            kept.push(op).ok()?;
        }
    }
    Some(kept.into_boxed())
}

/// A commit's most operations: two turn places per write, the first commit's
/// own rows (Format, Layouts and the installation's identifier), and the
/// orphans' erases it carries.
pub(crate) fn most_ops(writes: u32, orphan_erases: u32) -> Option<u32> {
    writes.checked_mul(2)?.checked_add(3)?.checked_add(orphan_erases)
}

fn copy(
    store: &mut Store,
    from: jig_domain_core_vocabulary::Key,
    to: jig_domain_core_vocabulary::Key,
) -> Result<skein_kv::Op, CommitCheck> {
    let from = keys::encode(&from, store.limits.store.key).ok_or(CommitCheck::Encoding)?;
    let value = crate::open::get(store, Token::new(0), from).or(Err(CommitCheck::Copy))?.ok_or(CommitCheck::Copy)?;
    Ok(skein_kv::Op::Put { key: keys::encode(&to, store.limits.store.key).ok_or(CommitCheck::Encoding)?, value })
}
