//! Kept acts (temper/docs/plan/playground-reset-r4/notes/contracts.md, D4;
//! temper/docs/design/protocol/setup.md, 8.4): retry until a saved answer.
use crate::{Domain, Kept, KeptAct, Key, Request, Stored};
use skein_lib::{Env, Queue};

pub(crate) fn keep(d: &mut Domain, acts: &[Kept], out: &mut Queue<Request>) {
    for row in acts {
        if d.kept.contains_key(&row.key) {
            continue;
        }
        d.kept.insert(row.key, row.act).expect("kept selection within limits");
        out.push(Request::Save { read_afresh: None, release: None, record: Stored::Kept(*row) });
        send(*row, out);
    }
}

pub(crate) fn key(kind: &[u8], numbers: &[u64]) -> [u8; 16] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(b"temper kept\0");
    hash.update(kind);
    hash.update(b"\0");
    for number in numbers {
        hash.update(&number.to_be_bytes());
    }
    hash.finish().get(..16).expect("digest prefix").try_into().expect("16-byte key")
}

fn send(row: Kept, out: &mut Queue<Request>) {
    match row.act {
        KeptAct::RemoveForge { .. } => {}
        KeptAct::Cancel { .. }
        | KeptAct::RemoveProject { .. }
        | KeptAct::RemoveAccount { .. }
        | KeptAct::BoxRemove { .. } => out.push(Request::SendKept(row)),
    }
}

pub(crate) fn send_all(d: &Domain, out: &mut Queue<Request>) {
    for (key, act) in &d.kept {
        send(Kept { key: *key, act: *act }, out);
    }
}

pub(crate) fn answered(d: &mut Domain, env: &Env<crate::Limits>, key: [u8; 16], keep: bool, out: &mut Queue<Request>) {
    if !keep {
        erase(d, key, out);
    }
    crate::removal::finish(d, env, out);
}

pub(crate) fn box_answered(
    d: &mut Domain,
    env: &Env<crate::Limits>,
    forge: u16,
    removed: bool,
    out: &mut Queue<Request>,
) {
    if !removed {
        crate::removal::finish(d, env, out);
        return;
    }
    let mut index = 0_usize;
    for _ in 0..d.kept.len() {
        let (key, act) = d.kept.iter().nth(index).expect("kept row");
        if *act == (KeptAct::BoxRemove { forge }) {
            let key = *key;
            erase(d, key, out);
        } else {
            index = index.checked_add(1).expect("kept index space");
        }
    }
    crate::removal::finish(d, env, out);
}

pub(crate) fn erased_for(d: &Domain, forge: u16) -> bool {
    for (_, act) in &d.kept {
        match *act {
            KeptAct::Cancel { task, .. } => {
                if let Some(change) = d.changes.get(&task)
                    && change.repository.forge == forge
                {
                    return false;
                }
            }
            KeptAct::BoxRemove { forge: number } if number == forge => return false,
            KeptAct::BoxRemove { .. }
            | KeptAct::RemoveForge { .. }
            | KeptAct::RemoveProject { .. }
            | KeptAct::RemoveAccount { .. } => {}
        }
    }
    true
}

pub(crate) fn erase(d: &mut Domain, key: [u8; 16], out: &mut Queue<Request>) {
    if d.kept.remove(&key).is_some() {
        out.push(Request::Erase { key: Key::Kept(key) });
    }
}
