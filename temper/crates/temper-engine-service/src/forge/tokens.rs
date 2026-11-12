//! Runtime bot identities and GitHub git tokens (temper/docs/design/protocol/forge/engine.md, 1).
use crate::{engine::Engine, settings};
use alloc::boxed::Box;
use skein_forge_connection::Account;
use skein_lib::{Duration, List, Map, Secret, Wall};
use temper_engine_domain as root;
use temper_protocol_worker_wire::v1;

/// One installation's git token; never stored, never in a root.
#[derive(PartialEq, Eq, Debug)]
struct Entry {
    account: u64,
    login: Box<[u8]>,
    token: Secret,
    expires: Wall,
}
#[derive(Debug)]
struct Forge {
    number: u32,
    login: Box<[u8]>,
    author: settings::Author,
    entries: Map<u64, Entry>,
    announced: bool,
    made: List<(Box<[u8]>, u64)>,
}
/// Engine-only token state, bounded by the deployment forge limit.
#[derive(Debug)]
pub struct Tokens {
    forges: Map<u16, Forge>,
}
impl Tokens {
    #[must_use]
    pub fn empty() -> Self {
        Self { forges: Map::with_capacity(0) }
    }
    pub(crate) fn new(settings: &settings::Engine) -> Self {
        Self { forges: Map::with_capacity(settings.limits.forge.forges) }
    }
    pub(crate) fn same_identity(&self, identity: &settings::Identity) -> bool {
        match self.forges.get(&identity.forge) {
            Some(held) => {
                held.number == identity.number && held.login == identity.login && held.author == identity.author
            }
            None => false,
        }
    }
    pub(crate) fn forget(&mut self, forge: u16) -> Option<u32> {
        match self.forges.remove(&forge) {
            Some(state) => Some(state.number),
            None => None,
        }
    }
    pub(crate) fn keep(&mut self, identity: &settings::Identity, installations: u32) {
        self.forges
            .insert(
                identity.forge,
                Forge {
                    number: identity.number,
                    login: identity.login.clone(),
                    author: identity.author.clone(),
                    entries: Map::with_capacity(installations),
                    announced: false,
                    made: List::with_capacity(installations),
                },
            )
            .expect("kept forge capacity");
    }
}
pub(crate) fn identity(engine: &mut Engine, forge: u16, token: Secret) {
    let Some(state) = engine.values.tokens.forges.get(&forge) else { return };
    let identity = settings::Identity {
        number: state.number,
        forge,
        token,
        login: state.login.clone(),
        author: state.author.clone(),
    };
    let value = crate::translate::identity(&identity, &engine.workers.encoding.limits).expect("kept identity bounds");
    let generation = engine.values.identities.put(identity.number, value);
    engine.events.push(root::Event::Credential {
        number: identity.number,
        generation,
        valid: Duration::from_nanos(u64::MAX),
    });
}
pub(crate) fn token(
    engine: &mut Engine,
    forge: u16,
    installation: u64,
    account: Account,
    token: Secret,
    expires: Wall,
) {
    crate::setup::lifecycle::installation_account(engine, forge, installation, &account);
    let Some(state) = engine.values.tokens.forges.get_mut(&forge) else { return };
    let entry = Entry { account: account.id, login: account.login, token, expires };
    if state.entries.get(&installation) == Some(&entry) {
        return;
    }
    state.entries.insert(installation, entry).expect("component bounds installations");
    if state.announced {
        announce(engine, forge);
    }
}
pub(crate) fn removed(engine: &mut Engine, forge: u16, installation: u64) {
    let Some(state) = engine.values.tokens.forges.get_mut(&forge) else { return };
    if state.entries.remove(&installation).is_some() && state.announced {
        announce(engine, forge);
    }
}
pub(crate) fn accepted(engine: &mut Engine, forge: u16, accepted: &[u64]) {
    let Some(state) = engine.values.tokens.forges.get_mut(&forge) else { return };
    let mut removed = List::with_capacity(state.entries.len());
    for (installation, _) in &state.entries {
        if accepted.binary_search(installation).is_err() {
            removed.push(*installation).expect("bounded entries");
        }
    }
    for installation in &removed {
        state.entries.remove(installation);
    }
    if !removed.is_empty() && state.announced {
        announce(engine, forge);
    }
}
pub(crate) fn connected(engine: &mut Engine, forge: u16) {
    let Some(state) = engine.values.tokens.forges.get_mut(&forge) else { return };
    if !state.announced {
        state.announced = true;
        announce(engine, forge);
    }
}
/// Match the owner's ASCII login against the last value made.
pub(crate) fn account(tokens: &Tokens, forge: u16, owner: &[u8]) -> Option<u64> {
    let state = tokens.forges.get(&forge)?;
    for (login, account) in &state.made {
        if login.eq_ignore_ascii_case(owner) {
            return Some(*account);
        }
    }
    None
}
fn announce(engine: &mut Engine, forge: u16) {
    let state = engine.values.tokens.forges.get_mut(&forge).expect("kept GitHub forge");
    let limits = &engine.workers.encoding.limits;
    let mut entries = List::with_capacity(state.entries.len());
    let mut expires = None;
    state.made.truncate(0);
    for (_, entry) in &state.entries {
        entries
            .push(
                v1::Entry::new(
                    limits,
                    v1::EntryParts {
                        account: entry.account,
                        account_login: entry.login.clone(),
                        token: entry.token.clone(),
                    },
                )
                .expect("component entry fits wire limits"),
            )
            .expect("bounded entries");
        state.made.push((entry.login.clone(), entry.account)).expect("bounded entries");
        expires = Some(match expires {
            Some(previous) => entry.expires.min(previous),
            None => entry.expires,
        });
    }
    let rows = engine.forge_records.rows.forge(forge).expect("connected App records");
    let slug = rows.get(temper_protocol_records::Item::AppSlug).expect("connected slug");
    let id = skein_forge_setup::item::user(rows.get(temper_protocol_records::Item::User).expect("connected User"))
        .expect("valid User")
        .get();
    let value = crate::translate::github_identity(entries, &state.login, &state.author, slug, id, limits)
        .expect("startup checks identity bounds");
    let valid = match expires {
        Some(expires) => Duration::from_nanos(expires.as_nanos().saturating_sub(engine.env.wall.as_nanos())),
        None => Duration::from_nanos(u64::MAX),
    };
    let number = state.number;
    let generation = engine.values.identities.put(number, value);
    engine.events.push(root::Event::Credential { number, generation, valid });
}
