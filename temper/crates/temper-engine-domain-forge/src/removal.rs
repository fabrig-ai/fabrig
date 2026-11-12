//! Forge removal (temper/docs/design/domain/forge/adoption.md, 1).
use crate::{Answered, Domain, ForgeRefusal, ForgeState, Key, Mode, RemoveAnswer, Request, forges};
use alloc::boxed::Box;
use jig_domain_people::Role;
use skein_lib::{List, Queue, Token};

pub(crate) fn admit(
    d: &mut Domain,
    request: Token,
    token: Token,
    role: Option<Role>,
    forge: u16,
    out: &mut Queue<Request>,
) {
    let answer = if role != Some(Role::Owner) {
        RemoveAnswer::Refused(ForgeRefusal::Role)
    } else if d.record(forge).is_none() || d.forge_ended(forge) {
        RemoveAnswer::Refused(ForgeRefusal::NoSuchForge)
    } else if match d.playground {
        Some(row) => row.forge == forge,
        None => false,
    } {
        RemoveAnswer::Refused(ForgeRefusal::NotTaken)
    } else if begin(d, forge, out) {
        RemoveAnswer::Removed
    } else {
        RemoveAnswer::Refused(ForgeRefusal::NoSuchForge)
    };
    d.connect.answer(request, token, Answered::RemoveForge(answer), out);
}

/// End the forge and request its kept removal steps in this decision.
pub(crate) fn begin(d: &mut Domain, forge: u16, out: &mut Queue<Request>) -> bool {
    let Some(mut record) = d.record(forge).cloned() else { return false };
    match record.state {
        ForgeState::Kept => {}
        ForgeState::Ended { .. } => return false,
    }
    let box_removed = match record.mode {
        Mode::Owned | Mode::Fake => false,
        Mode::Connected | Mode::Managed => true,
    };
    record.state = ForgeState::Ended { secrets_erased: false, box_removed };
    forges::save(d, record, out);
    out.push(Request::ForgeGone { forge });
    if !box_removed {
        out.push(Request::BoxRemove { forge });
    }
    d.connections.remove(&forge);
    d.first.remove(&forge);
    crate::making::remove(d, forge, out);
    d.apps.remove(&forge);
    let mut installations = List::with_capacity(d.installations.capacity());
    for (key, _) in &d.installations {
        if key.0 == forge {
            installations.push(*key).expect("bounded installations");
        }
    }
    for key in installations.into_boxed() {
        d.installations.remove(&key);
    }
    if d.writers.remove(&forge).is_some() {
        out.push(Request::Erase { key: Key::Writer(forge) });
    }
    crate::domain::retire_forge(d, forge);
    let mut shared = List::with_capacity(d.repositories.capacity());
    for (provider, row) in &d.repositories {
        if provider.forge == forge && !row.contexts.is_empty() {
            shared.push(*provider).expect("bounded adopted repositories");
        }
    }
    for provider in &shared {
        let row = d.repositories.get_mut(provider).expect("selected repository");
        row.contexts = Box::default();
        out.push(Request::Save {
            record: crate::Stored::Repository(row.as_ref().clone()),
            read_afresh: None,
            release: None,
        });
    }
    out.push(Request::MarkedForges);
    true
}

pub(crate) fn secrets_erased(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    answered(d, forge, true, out);
}

pub(crate) fn box_removed(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    answered(d, forge, false, out);
}

fn answered(d: &mut Domain, forge: u16, secrets: bool, out: &mut Queue<Request>) {
    let Some(mut record) = d.record(forge).cloned() else { return };
    match record.state {
        ForgeState::Kept => {}
        ForgeState::Ended { secrets_erased, box_removed } => {
            let secrets_erased = secrets_erased || secrets;
            let box_removed = box_removed || !secrets;
            record.state = ForgeState::Ended { secrets_erased, box_removed };
            forges::save(d, record, out);
            settle(d, forge, out);
        }
    }
}

pub(crate) fn restart(d: &Domain, out: &mut Queue<Request>) {
    for (forge, record) in &d.records {
        match record.state {
            ForgeState::Kept => {}
            ForgeState::Ended { secrets_erased, box_removed } => {
                if !secrets_erased {
                    out.push(Request::ForgeGone { forge: *forge });
                }
                if !box_removed {
                    out.push(Request::BoxRemove { forge: *forge });
                }
            }
        }
    }
}

/// Finish removal's rows and record once both kept steps are answered.
pub(crate) fn settle(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    let Some(record) = d.record(forge) else { return };
    let whole = match record.state {
        ForgeState::Kept => return,
        ForgeState::Ended { secrets_erased, box_removed } => secrets_erased && box_removed,
    };
    if crate::domain::erase_removed_repositories(d, forge, out) && whole {
        forges::erase(d, forge, out);
    }
}

/// A task's erased names can release repositories on any ended forge.
pub(crate) fn unnamed(d: &mut Domain, out: &mut Queue<Request>) {
    let mut ended = List::with_capacity(d.records.capacity());
    for (forge, record) in &d.records {
        match record.state {
            ForgeState::Kept => {}
            ForgeState::Ended { .. } => ended.push(*forge).expect("bounded forge records"),
        }
    }
    for forge in ended.into_boxed() {
        settle(d, forge, out);
    }
}
