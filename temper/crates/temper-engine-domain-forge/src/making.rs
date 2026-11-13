//! Repositories being made (temper/docs/design/domain/forge/adoption.md, 5).
use crate::{Adoption, ApplicationOutcome, Ask, Domain, Key, Limits, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{Duration, Env, List, Queue, Token};
pub use temper_engine_domain_forge_vocabulary::making::*;

impl Domain {
    #[must_use]
    pub fn making(&self, key: &MakingKey) -> Option<&RepositoryMaking> {
        self.repository_makings.get(key)
    }
    #[must_use]
    pub fn next_making(&self, forge: u16) -> Option<&RepositoryMaking> {
        let mut oldest: Option<&RepositoryMaking> = None;
        for (_, row) in &self.repository_makings {
            if row.key.forge != forge {
                continue;
            }
            match &row.state {
                MakingState::Waiting => {}
                MakingState::Making
                | MakingState::Made { .. }
                | MakingState::Adopting { .. }
                | MakingState::Failed(_) => continue,
            }
            if let Some(kept) = oldest {
                if (row.asked, &row.key) < (kept.asked, &kept.key) {
                    oldest = Some(row);
                }
            } else {
                oldest = Some(row);
            }
        }
        oldest
    }
    #[must_use]
    pub fn making_due(&self, forge: u16) -> Option<MakingKey> {
        for (_, row) in &self.repository_makings {
            if row.key.forge != forge || self.making_acts.contains(&row.key) {
                continue;
            }
            match &row.state {
                MakingState::Made { .. } | MakingState::Adopting { .. } => return Some(row.key.clone()),
                MakingState::Waiting | MakingState::Making | MakingState::Failed(_) => {}
            }
        }
        None
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "the boundary returns a borrowed iterator over one forge's rows in key order"
    )]
    pub fn makings(&self, forge: u16) -> impl Iterator<Item = &RepositoryMaking> {
        self.repository_makings.iter().map(|(_, row)| row).filter(move |row| row.key.forge == forge)
    }
}

#[expect(clippy::too_many_arguments, reason = "a keyed admission carries the core's request identity and authority")]
pub(crate) fn admit(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    retention: Duration,
    role: Option<jig_domain_people::Role>,
    ask: Ask,
    out: &mut Queue<Request>,
) {
    if !crate::adoption::room(d, env, request, out) {
        return;
    }
    let (forge, org, name, from, into) = match ask {
        Ask::MakeRepository { forge, org, name, from, into } => (forge, org, name, from, into),
        Ask::DismissMaking { .. }
        | Ask::SetDraft(_)
        | Ask::Adopt(_)
        | Ask::SetUp { .. }
        | Ask::RemoveForge { .. }
        | Ask::StartPlayground
        | Ask::EndPlayground
        | Ask::Connect { .. }
        | Ask::Install { .. }
        | Ask::Accept { .. }
        | Ask::Reachable { .. }
        | Ask::Replace { .. } => unreachable!("making dispatcher"),
    };
    let key = MakingKey { forge, org, name };
    let outcome = match admission(d, env, role, &key, &from, into.as_ref()) {
        Err(why) => ApplicationOutcome::MakeRefused(why),
        Ok(()) => {
            let mut replace = true;
            let mut copying = false;
            if let Some(row) = d.making(&key) {
                match row.state {
                    MakingState::Failed(_) => {
                        copying = row.copying
                            && match &row.from {
                                MakeFrom::Copy { address: kept } => match &from {
                                    MakeFrom::Copy { address } => kept == address,
                                    MakeFrom::Empty => false,
                                },
                                MakeFrom::Empty => false,
                            };
                    }
                    MakingState::Waiting
                    | MakingState::Making
                    | MakingState::Made { .. }
                    | MakingState::Adopting { .. } => replace = false,
                }
            }
            if replace {
                let row = RepositoryMaking { key, from, into, asked: env.wall, copying, state: MakingState::Waiting };
                save(d, row, out);
            }
            ApplicationOutcome::MakeKept
        }
    };
    crate::adoption::finish(
        d,
        env,
        request,
        crate::adoption::Pending { token, retention, observed: None },
        outcome,
        out,
    );
}

#[expect(clippy::too_many_arguments, reason = "a keyed dismissal carries the core's request identity and authority")]
pub(crate) fn dismiss(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    retention: Duration,
    role: Option<jig_domain_people::Role>,
    key: MakingKey,
    into: Option<u32>,
    out: &mut Queue<Request>,
) {
    if !crate::adoption::room(d, env, request, out) {
        return;
    }
    let outcome = if role == Some(jig_domain_people::Role::Owner) {
        match d.making(&key) {
            None => ApplicationOutcome::MakeKept,
            Some(row)
                if match &row.into {
                    Some(held) => into != Some(held.project),
                    None => into.is_some(),
                } =>
            {
                ApplicationOutcome::MakeRefused(MakeRefusal::NotOwner)
            }
            Some(row) => match &row.state {
                MakingState::Failed(_) => {
                    erase(d, key, out);
                    ApplicationOutcome::MakeKept
                }
                MakingState::Waiting
                | MakingState::Making
                | MakingState::Made { .. }
                | MakingState::Adopting { .. } => ApplicationOutcome::MakeRefused(MakeRefusal::NotFailed),
            },
        }
    } else {
        ApplicationOutcome::MakeRefused(MakeRefusal::NotOwner)
    };
    crate::adoption::finish(
        d,
        env,
        request,
        crate::adoption::Pending { token, retention, observed: None },
        outcome,
        out,
    );
}

fn admission(
    d: &Domain,
    env: &Env<Limits>,
    role: Option<jig_domain_people::Role>,
    key: &MakingKey,
    from: &MakeFrom,
    into: Option<&MakeInto>,
) -> Result<(), MakeRefusal> {
    if role != Some(jig_domain_people::Role::Owner) {
        return Err(MakeRefusal::NotOwner);
    }
    let Some(record) = d.record(key.forge) else { return Err(MakeRefusal::NoSuchForge) };
    if d.forge_ended(key.forge) {
        return Err(MakeRefusal::NoSuchForge);
    }
    if record.kind != crate::Kind::Forgejo || (record.mode != crate::Mode::Owned && record.mode != crate::Mode::Managed)
    {
        return Err(MakeRefusal::NotOffered);
    }
    let bound = skein_lib::usize_of(env.limits.name_bytes);
    if key.org.len() > bound
        || key.name.len() > bound
        || !temper_client_protocol_wire::source::org(&key.org)
        || !temper_client_protocol_wire::source::repository(&key.name)
    {
        return Err(MakeRefusal::Invalid);
    }
    match from {
        MakeFrom::Empty => {}
        MakeFrom::Copy { address } => {
            if address.len() > temper_client_protocol_wire::source::ADDRESS_BYTES
                || temper_client_protocol_wire::source::parse(address).is_err()
            {
                return Err(MakeRefusal::Invalid);
            }
        }
    }
    if let Some(into) = into {
        if into.host.is_empty() || into.host.len() > bound {
            return Err(MakeRefusal::Invalid);
        }
        if into.home && d.home(into.project).is_some() {
            return Err(MakeRefusal::HomeHeld);
        }
    }
    if d.making(key).is_none() && d.repository_makings.len() >= env.limits.makings {
        return Err(MakeRefusal::Full);
    }
    Ok(())
}

fn save(d: &mut Domain, row: RepositoryMaking, out: &mut Queue<Request>) {
    d.repository_makings.insert(row.key.clone(), row.clone()).expect("making admitted within limit");
    out.push(Request::Save { record: Stored::Making(row), release: None, read_afresh: None });
    out.push(Request::MarkedForges);
}
fn erase(d: &mut Domain, key: MakingKey, out: &mut Queue<Request>) {
    d.making_acts.remove(&key);
    if d.repository_makings.remove(&key).is_some() {
        out.push(Request::Erase { key: Key::Making(key) });
        out.push(Request::MarkedForges);
    }
}
pub(crate) fn started(d: &mut Domain, key: MakingKey, out: &mut Queue<Request>) {
    let Some(mut row) = d.making(&key).cloned() else { return };
    match row.state {
        MakingState::Waiting => {
            row.state = MakingState::Making;
            save(d, row, out);
        }
        MakingState::Making | MakingState::Made { .. } | MakingState::Adopting { .. } | MakingState::Failed(_) => {}
    }
}
pub(crate) fn copying(d: &mut Domain, key: MakingKey, out: &mut Queue<Request>) {
    let Some(mut row) = d.making(&key).cloned() else { return };
    row.copying = true;
    save(d, row, out);
    out.push(Request::MakingCopied { key });
}
pub(crate) fn made(d: &mut Domain, key: MakingKey, id: u64, branch: Box<[u8]>, out: &mut Queue<Request>) {
    let Some(mut row) = d.making(&key).cloned() else { return };
    if row.into.is_none() {
        erase(d, key, out);
        return;
    }
    row.state = MakingState::Adopting { id, branch };
    save(d, row, out);
}
pub(crate) fn stopped(d: &mut Domain, key: MakingKey, mut failed: MakingFailed, out: &mut Queue<Request>) {
    let Some(mut row) = d.making(&key).cloned() else { return };
    if failed.reason == MakeFailure::Taken {
        row.copying = false;
    }
    failed.words = Box::from(failed.words.get(..failed.words.len().min(256)).expect("bounded failure words"));
    row.state = MakingState::Failed(failed);
    d.making_acts.remove(&key);
    save(d, row, out);
}
pub(crate) fn adopt(d: &mut Domain, key: MakingKey, out: &mut Queue<Request>) {
    if d.making_acts.contains(&key) {
        return;
    }
    let Some(row) = d.making(&key) else { return };
    let (id, branch) = match &row.state {
        MakingState::Made { id, branch } | MakingState::Adopting { id, branch } => (*id, branch.clone()),
        MakingState::Waiting | MakingState::Making | MakingState::Failed(_) => return,
    };
    let Some(into) = &row.into else {
        erase(d, key, out);
        return;
    };
    let Ok(repository) = u32::try_from(id) else {
        stopped(
            d,
            key,
            MakingFailed {
                step: MakeStep::Adoption,
                reason: MakeFailure::Other,
                words: Box::from(b"repository id exceeds adoption range".as_slice()),
            },
            out,
        );
        return;
    };
    let adoption = Adoption {
        forge: key.forge,
        project: into.project,
        home: into.home,
        host: into.host.clone(),
        owner: key.org.clone(),
        name: key.name.clone(),
        role: crate::Role::Owned,
        ci: true,
        checks: Box::new([]),
        prefix: Box::from(b"temper/".as_slice()),
        landing: branch,
    };
    let act = act_key(row);
    d.making_acts.insert(key.clone()).expect("one act per making");
    out.push(Request::MakingAct {
        repository: skein_forge::api::Repository { forge: key.forge, repository },
        key,
        act,
        adoption,
    });
}
fn act_key(row: &RepositoryMaking) -> [u8; 16] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(b"temper making");
    hash.update(&row.key.forge.to_be_bytes());
    for part in [&row.key.org, &row.key.name] {
        hash.update(&skein_lib::len_u64(part.len()).to_be_bytes());
        hash.update(part);
    }
    hash.update(&row.asked.as_nanos().to_be_bytes());
    let mut key = [0; 16];
    for (slot, byte) in key.iter_mut().zip(hash.finish()) {
        *slot = byte;
    }
    key
}
pub(crate) fn acted(d: &mut Domain, key: MakingKey, outcome: Option<MakingFailed>, out: &mut Queue<Request>) {
    match outcome {
        None => erase(d, key, out),
        Some(mut failed) => {
            failed.step = MakeStep::Adoption;
            stopped(d, key, failed, out);
        }
    }
}
pub(crate) fn remove(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    let mut keys = List::with_capacity(d.repository_makings.capacity());
    for row in d.makings(forge) {
        keys.push(row.key.clone()).expect("bounded making rows");
    }
    for key in keys.into_boxed() {
        erase(d, key, out);
    }
}
