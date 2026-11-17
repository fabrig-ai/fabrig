//! Per-answer connector asks and held load slots (jig/docs/design/domain/engine/views.md, 2).
//! A read's Pending owns its page; a watch also waits there, keeping the views child Making.

use super::{Answer, Destination, EndedGroup, Pending, Range, Scope, TaskPage, Why};
use crate::{Core, Limits, MadePart, PartAsk, PartFailure, PartsAsk, Request, RowPart, documents};
use alloc::{boxed::Box, vec::IntoIter};
use skein_lib::{Env, Id, List, Map, Queue, Token};

pub(crate) fn assert_kinds(kinds: &[documents::parts::PartKind], connectors: &[u16]) {
    for (index, kind) in kinds.iter().enumerate() {
        assert!(connectors.contains(&kind.connector), "part kinds name configured connectors");
        for previous in kinds.iter().take(index) {
            assert!(
                (kind.owner, kind.kind) != (previous.owner, previous.kind),
                "part kinds have unique owner/kind keys"
            );
        }
    }
}

#[derive(Hash, Debug)]
struct Slot {
    at: u32,
    kind: PartAsk,
    bytes: u32,
}

#[derive(Hash, Debug)]
pub(super) struct Making {
    waiting: Map<u16, List<Slot>>,
    next: Option<documents::Cursor>,
}

fn prepare(core: &Core, query: &documents::TaskQuery, page: &mut TaskPage) -> Making {
    let mut waiting: Map<u16, List<Slot>> = Map::with_capacity(query.parts.len());
    for (at, ask) in query.parts.iter().enumerate() {
        let mut declared = None;
        for kind in &core.part_kinds {
            if (kind.owner, kind.kind) == (ask.owner, ask.kind) {
                declared = Some(*kind);
                break;
            }
        }
        if let Some(kind) = declared {
            if !waiting.contains_key(&kind.connector) {
                waiting.insert(kind.connector, List::with_capacity(query.parts.len())).expect("one ask per connector");
            }
            waiting
                .get_mut(&kind.connector)
                .expect("declared connector")
                .push(Slot { at: u32::try_from(at).expect("bounded parts"), kind: *ask, bytes: kind.bytes })
                .expect("query kind slots");
        }
    }
    initialize(&mut page.live, query.parts.len());
    if let EndedGroup::Rows { rows, .. } = &mut page.ended {
        initialize(rows, query.parts.len());
    }
    if page.live.is_empty()
        && match &page.ended {
            EndedGroup::Rows { rows, .. } => rows.is_empty(),
            EndedGroup::None => true,
        }
    {
        waiting = Map::with_capacity(query.parts.len());
    }
    Making { waiting, next: None }
}

fn initialize(rows: &mut List<documents::TaskLine>, kinds: u32) {
    for row in rows.iter_mut() {
        row.parts = List::with_capacity(kinds);
        for _ in 0..kinds {
            row.parts.push(RowPart::Unknown).expect("one slot per query part");
        }
    }
}

fn emit(core: &Core, name: Id<Pending>) -> Queue<Request> {
    let pending = core.reads.get(name).expect("making answer");
    let page = pending.tasks.as_ref().expect("making page");
    let making = pending.parts.as_ref().expect("making parts");
    let ended = match &page.ended {
        EndedGroup::Rows { rows, .. } => rows.as_slice(),
        EndedGroup::None => &[],
    };
    let row_count = page
        .live
        .len()
        .checked_add(u32::try_from(ended.len()).expect("bounded ended rows"))
        .expect("two bounded groups");
    let mut out = Queue::uncapped();
    for (connector, slots) in &making.waiting {
        let mut kinds = List::with_capacity(slots.len());
        for slot in slots {
            kinds.push(slot.kind).expect("connector kinds");
        }
        let mut rows = List::with_capacity(row_count);
        for row in page.live.iter().chain(ended.iter()) {
            rows.push(row.clone()).expect("answer rows");
        }
        out.push(Request::Ask {
            connector: *connector,
            ask: crate::Ask::Parts(PartsAsk { token: name.token(), party: pending.party, kinds, rows }),
        });
    }
    out
}

pub(super) fn loaded(core: &mut Core, env: &Env<Limits>, name: Id<Pending>, mut page: Box<TaskPage>) -> Queue<Request> {
    let Scope::Tasks { query } = &core.reads.get(name).expect("task read").range.scope else {
        unreachable!("task query")
    };
    let making = prepare(core, query, &mut page);
    let ready = making.waiting.is_empty();
    let pending = core.reads.get_mut(name).expect("task read");
    pending.tasks = Some(page);
    pending.parts = Some(making);
    if ready { finish(core, env, name) } else { emit(core, name) }
}

#[expect(clippy::too_many_arguments, reason = "the read continuation lends its retired admission slot")]
pub(super) fn read(
    core: &mut Core,
    env: &Env<Limits>,
    destination: Destination,
    sign_in: u64,
    party: u64,
    query: documents::TaskQuery,
    mut page: Box<TaskPage>,
    continuing: bool,
) -> Queue<Request> {
    let making = prepare(core, &query, &mut page);
    if making.waiting.is_empty() {
        return super::answer(destination, Answer::Tasks(page));
    }
    let occupied = core.reads.len().checked_sub(u32::from(continuing)).expect("continuation retains its retired slot");
    if occupied >= env.limits.load_slots {
        return super::answer(destination, Answer::None(Why::Busy));
    }
    let pending = Pending {
        destination,
        sign_in,
        party,
        range: range(env, party, query),
        tasks: Some(page),
        parts: Some(making),
    };
    let name = core.reads.insert(pending).expect("admitted part load slot");
    emit(core, name)
}

fn range(env: &Env<Limits>, party: u64, query: documents::TaskQuery) -> Range {
    Range {
        party,
        scope: Scope::Tasks { query },
        after: None,
        backwards: false,
        rows: env.limits.documents.rows,
        bytes: env.limits.load_bytes,
    }
}

pub(crate) fn watch(core: &mut Core, env: &Env<Limits>, watcher: Token, lines: documents::Lines) -> Queue<Request> {
    let documents::Lines { query, lines, next, ended_mark } = lines;
    let binding = core.watch_binding(watcher).expect("making watch");
    let (party, sign_in) = (binding.party, binding.sign_in);
    let mut page = Box::new(TaskPage { live: lines, live_next: None, ended: EndedGroup::None, ended_mark });
    let mut making = prepare(core, &query, &mut page);
    making.next = next;
    if making.waiting.is_empty() || core.reads.len() >= env.limits.load_slots {
        for row in page.live.iter_mut() {
            for (_, slots) in &making.waiting {
                for slot in slots {
                    *row.parts.get_mut(slot.at).expect("query slot") = RowPart::Failed(PartFailure::Busy);
                }
            }
        }
        return crate::routing::view::routes::query_made(
            core,
            env,
            watcher,
            Ok(documents::Document::Lines(documents::Lines { query, lines: page.live, next, ended_mark })),
        );
    }
    let name = core
        .reads
        .insert(Pending {
            destination: Destination::Watch(watcher),
            sign_in,
            party,
            range: range(env, party, query),
            tasks: Some(page),
            parts: Some(making),
        })
        .expect("admitted watch part load slot");
    emit(core, name)
}

pub(crate) fn answered(
    core: &mut Core,
    env: &Env<Limits>,
    connector: u16,
    token: Token,
    parts: List<MadePart>,
) -> Queue<Request> {
    let name = Id::<Pending>::from_token(token);
    let Some(pending) = core.reads.get_mut(name) else { return Queue::uncapped() };
    let Some(making) = &mut pending.parts else { return Queue::uncapped() };
    let Some(slots) = making.waiting.remove(&connector) else { return Queue::uncapped() };
    let page = pending.tasks.as_mut().expect("making page");
    let rows = page
        .live
        .len()
        .checked_add(match &page.ended {
            EndedGroup::Rows { rows, .. } => rows.len(),
            EndedGroup::None => 0,
        })
        .expect("bounded groups");
    let valid = rows.checked_mul(slots.len()) == Some(parts.len());
    let mut parts = parts.into_boxed().into_iter();
    fill(&mut page.live, &slots, &mut parts, valid);
    if let EndedGroup::Rows { rows, .. } = &mut page.ended {
        fill(rows, &slots, &mut parts, valid);
    }
    if making.waiting.is_empty() { finish(core, env, name) } else { Queue::uncapped() }
}

fn fill(rows: &mut List<documents::TaskLine>, slots: &List<Slot>, parts: &mut IntoIter<MadePart>, valid: bool) {
    for row in rows.iter_mut() {
        for slot in slots {
            let value = if valid {
                match parts.next().expect("validated part count") {
                    MadePart::Made(bytes) => {
                        if bytes.len() <= skein_lib::length::usize_of(slot.bytes) {
                            RowPart::Made(bytes)
                        } else {
                            RowPart::Failed(PartFailure::TooLarge)
                        }
                    }
                    MadePart::Failed(why) => RowPart::Failed(why),
                }
            } else {
                RowPart::Failed(PartFailure::Unreadable)
            };
            *row.parts.get_mut(slot.at).expect("query slot") = value;
        }
    }
}

fn finish(core: &mut Core, env: &Env<Limits>, name: Id<Pending>) -> Queue<Request> {
    let pending = core.reads.get_mut(name).expect("completed parts");
    let destination = core::mem::replace(&mut pending.destination, Destination::Done);
    let page = pending.tasks.take().expect("completed page");
    let making = pending.parts.take().expect("completed parts");
    let sign_in = pending.sign_in;
    let party = pending.party;
    let Scope::Tasks { query } = &pending.range.scope else { unreachable!("parts of task queries") };
    let query = query.clone();
    core.reads.retire(name);
    let allowed = documents::task_query::admission(core, party, &query).is_ok();
    if let Destination::Watch(watcher) = destination {
        let document = if core.people.person(sign_in, env.now, env.wall) != Some(party) {
            Err(jig_domain_views::End::SignedOut)
        } else if !allowed {
            Err(jig_domain_views::End::NotAllowed)
        } else {
            Ok(documents::Document::Lines(documents::Lines {
                query,
                lines: page.live,
                next: making.next,
                ended_mark: page.ended_mark,
            }))
        };
        return crate::routing::view::routes::query_made(core, env, watcher, document);
    }
    if let Destination::Person { key, .. } = &destination
        && !core.current_proof(key.task, key.attempt)
    {
        return Queue::uncapped();
    }
    let invalid_person = if let Destination::Person { key, project, .. } = &destination {
        !super::person::valid(core, *key, *project, party)
    } else {
        false
    };
    let answer = if sign_in != 0 && core.people.person(sign_in, env.now, env.wall) != Some(party) {
        Answer::None(Why::SignedOut)
    } else if !allowed || invalid_person {
        Answer::None(Why::Unknown)
    } else {
        Answer::Tasks(page)
    };
    super::answer(destination, answer)
}
