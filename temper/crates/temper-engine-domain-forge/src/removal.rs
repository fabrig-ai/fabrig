//! Whole-forge removal and its durable acts
//! (temper/docs/design/draft/playground-reset.md, section 4).
use crate::{Domain, ForgeState, KeptAct, KeptActKind, Key, Limits, RemoveAnswer, RemoveRefusal, Request, Stored};
use skein_lib::{Env, List, Queue};

pub(crate) const fn bound(l: &Limits) -> Option<u32> {
    let Some(forges) = l.forges.checked_mul(2) else { return None };
    l.tasks.checked_add(forges)
}

pub(crate) fn kept_key(forge: u16, number: u64, purpose: &[u8]) -> [u8; 16] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(b"temper forge removal\0");
    hash.update(&forge.to_be_bytes());
    hash.update(&number.to_be_bytes());
    hash.update(purpose);
    hash.finish().get(..16).expect("digest prefix").try_into().expect("16-byte key")
}

pub(crate) fn requested(d: &mut Domain, env: &Env<Limits>, forge: u16, out: &mut Queue<Request>) -> RemoveAnswer {
    let Some(record) = d.records.get(&forge) else { return RemoveAnswer::Refused(RemoveRefusal::NoSuchForge) };
    if record.state != ForgeState::Kept {
        return RemoveAnswer::Refused(RemoveRefusal::NoSuchForge);
    }
    if record.playground {
        return RemoveAnswer::Refused(RemoveRefusal::Playground);
    }
    remove(d, env, forge, out);
    RemoveAnswer::Removing
}

/// The playground's reset uses the same decision without the request's refusals.
pub(crate) fn remove(d: &mut Domain, env: &Env<Limits>, forge: u16, out: &mut Queue<Request>) {
    let Some(mut record) = d.records.get(&forge).cloned() else { return };
    if record.state != ForgeState::Kept {
        return;
    }
    let group = d.under_box && record.setup.is_some();
    record.state = ForgeState::Removing;
    record.first = None;
    d.keep_record(record.clone());
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Forge(record) });
    out.push(Request::Unregister { forge });
    crate::domain::retire_forge(d, env, forge, out);
    // Changes can be retained after their task has ended; only live work is cancelled.
    let mut cancels = List::with_capacity(d.changes.len());
    for (task, change) in &d.changes {
        if change.repository.forge == forge
            && !change.terminal()
            && let Some(repository) = d.repositories.get(&change.repository)
        {
            cancels
                .push(KeptAct {
                    key: kept_key(forge, *task, b"cancel"),
                    forge,
                    act: KeptActKind::Cancel { project: repository.project, task: *task },
                })
                .expect("bounded changes");
        }
    }
    for row in cancels.into_boxed() {
        keep(d, row, out);
    }
    if group {
        keep(d, KeptAct { key: kept_key(forge, 0, b"group"), forge, act: KeptActKind::RemoveGroup }, out);
    }
    keep(d, KeptAct { key: kept_key(forge, 0, b"secrets"), forge, act: KeptActKind::EraseSecrets }, out);
    out.push(Request::MarkedForges);
}

fn keep(d: &mut Domain, row: KeptAct, out: &mut Queue<Request>) {
    d.kept.insert(row.key, row).expect("live tasks plus two acts per forge");
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::KeptAct(row) });
    out.push(Request::SendKeptAct(row));
}

pub(crate) fn send_kept(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut removing = List::with_capacity(d.records.len());
    for (forge, record) in &d.records {
        if record.state == ForgeState::Removing {
            removing.push(*forge).expect("bounded forge records");
        }
    }
    for forge in removing.into_boxed() {
        crate::domain::retire_forge(d, env, forge, out);
        finish(d, forge, out);
    }
    for (_, row) in &d.kept {
        out.push(Request::SendKeptAct(*row));
    }
}

pub(crate) fn answered(d: &mut Domain, key: [u8; 16], keep: bool, out: &mut Queue<Request>) {
    if !keep && let Some(row) = d.kept.remove(&key) {
        out.push(Request::Erase { key: Key::KeptAct(key) });
        finish(d, row.forge, out);
    }
}

pub(crate) fn group_removed(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    answer_kind(d, forge, KeptActKind::RemoveGroup, out);
}

pub(crate) fn secrets_erased(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    answer_kind(d, forge, KeptActKind::EraseSecrets, out);
}

fn answer_kind(d: &mut Domain, forge: u16, act: KeptActKind, out: &mut Queue<Request>) {
    let mut keys = List::with_capacity(d.kept.len());
    for (key, row) in &d.kept {
        if row.forge == forge && row.act == act {
            keys.push(*key).expect("bounded kept acts");
        }
    }
    for key in keys.into_boxed() {
        answered(d, key, false, out);
    }
}

fn finish(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    let Some(record) = d.records.get(&forge) else { return };
    if record.state != ForgeState::Removing {
        return;
    }
    for (_, row) in &d.kept {
        if row.forge == forge {
            return;
        }
    }
    d.records.remove(&forge);
    out.push(Request::Erase { key: Key::Forge(forge) });
}

impl Domain {
    /// Whether removal's final answer erased this forge's record.
    #[must_use]
    pub fn removed(&self, forge: u16) -> bool {
        !self.records.contains_key(&forge)
    }
}
