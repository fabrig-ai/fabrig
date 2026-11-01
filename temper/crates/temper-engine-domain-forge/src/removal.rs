//! Forge removal (temper/docs/design/draft/playground-reset.md, 4;
//! temper/docs/plan/playground-reset-r4/notes/contracts.md, D3, D4).
use crate::{Domain, Kept, KeptAct, Key, Limits, RemoveRefusal, Request, Stored};
use skein_lib::{Env, List, Queue};

#[derive(Debug)]
pub(crate) struct State;

impl State {
    pub(crate) const fn new(_l: &Limits) -> Self {
        Self
    }
}

pub(crate) fn removing(d: &Domain, forge: u16) -> bool {
    match d.forge_records.get(&forge) {
        Some(row) => row.removing,
        None => false,
    }
}

pub(crate) fn remove(
    d: &mut Domain,
    env: &Env<Limits>,
    forge: u16,
    out: &mut Queue<Request>,
) -> Result<(), RemoveRefusal> {
    let Some(row) = d.forge_records.get_mut(&forge) else { return Err(RemoveRefusal::NoSuchForge) };
    if row.removing {
        return Err(RemoveRefusal::NoSuchForge);
    }
    row.removing = true;
    let box_remove = d.under_box
        && match &row.setup {
            Some(setup) => match setup.mode {
                crate::record::SetupMode::Owned(_) | crate::record::SetupMode::Fake(_) => true,
                crate::record::SetupMode::Managed { .. } => false,
            },
            None => false,
        };
    out.push(Request::Save { record: Stored::Forge(row.clone()), release: None, read_afresh: None });
    out.push(Request::ForgeRemoved { forge });
    let child_env = Env { now: env.now, wall: env.wall, limits: env.limits.client };
    skein_forge_client::remove_forge(&mut d.client, &child_env, forge, &mut d.client_out);
    crate::domain::drain(d, env, out);
    d.writers.remove(&forge);
    erase_outbox(d, forge, out);
    drop_tables(d, forge);
    crate::domain::retire_ended(d, env, out);
    let mut acts = List::with_capacity(env.limits.changes.checked_add(1).expect("kept bound"));
    for (_, change) in &d.changes {
        if change.repository.forge != forge || change.terminal() {
            continue;
        }
        let Some(repository) = d.repositories.get(&change.repository) else { continue };
        acts.push(Kept {
            key: crate::kept::key(b"cancel", &[u64::from(repository.project), change.task]),
            act: KeptAct::Cancel { project: repository.project, task: change.task },
        })
        .expect("one cancel per change");
    }
    if box_remove {
        acts.push(Kept { key: crate::kept::key(b"box", &[u64::from(forge)]), act: KeptAct::BoxRemove { forge } })
            .expect("one box remove");
    }
    crate::kept::keep(d, acts.as_slice(), out);
    out.push(Request::MarkedForges);
    finish(d, env, out);
    Ok(())
}

fn erase_outbox(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    let mut flights = List::with_capacity(d.flights.len());
    for ((task, entry), ()) in &d.flights {
        let belongs = match d.entries.get(entry) {
            Some(row) => row.entry.repository.forge == forge,
            None => match d.changes.get(task) {
                Some(row) => row.repository.forge == forge,
                None => false,
            },
        };
        if belongs {
            flights.push((*task, *entry)).expect("bounded flights");
        }
    }
    for (task, entry) in flights.into_boxed() {
        d.flights.remove(&(task, entry));
        out.push(Request::Erase { key: Key::Flight { task, entry } });
    }
    let mut entries = List::with_capacity(d.entries.len());
    for (number, row) in &d.entries {
        if row.entry.repository.forge == forge {
            entries.push(*number).expect("bounded entries");
        }
    }
    for number in entries.into_boxed() {
        d.entries.remove(&number);
        out.push(Request::Erase { key: Key::Entry(number) });
    }
}

fn drop_tables(d: &mut Domain, forge: u16) {
    d.connections.remove(&forge);
    d.first.remove(&forge);
    d.apps.remove(&forge);
    d.ended.remove(&forge);
    let mut installations = List::with_capacity(d.installations.len());
    for ((number, installation), _) in &d.installations {
        if *number == forge {
            installations.push(*installation).expect("bounded installations");
        }
    }
    for installation in installations.into_boxed() {
        d.installations.remove(&(forge, installation));
    }
}

pub(crate) fn finish(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut finished = List::with_capacity(env.limits.forges);
    for (forge, row) in &d.forge_records {
        if row.removing && crate::kept::erased_for(d, *forge) {
            finished.push(*forge).expect("bounded forges");
        }
    }
    for forge in finished.into_boxed() {
        d.forge_records.remove(&forge);
        d.configured.remove(&forge);
        d.identities.remove(&forge);
        d.writers.remove(&forge);
        drop_tables(d, forge);
        out.push(Request::Erase { key: Key::Forge(forge) });
        let mut answers = List::with_capacity(d.kept.len());
        for (key, act) in &d.kept {
            if *act == (KeptAct::RemoveForge { forge }) {
                answers.push(*key).expect("bounded kept answers");
            }
        }
        for key in answers.into_boxed() {
            crate::kept::erase(d, env, key, out);
        }
    }
}

#[cfg(test)]
#[path = "tests/removal.rs"]
mod tests;
