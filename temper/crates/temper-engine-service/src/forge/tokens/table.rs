//! Token state and account lookup (protocol/services.md, 4.2).
use crate::settings;
use alloc::boxed::Box;
use skein_lib::{List, Map, Secret, Wall};

/// One installation's git token, supplied by the component; never stored, never in a root.
#[derive(PartialEq, Eq, Debug)]
pub struct Entry {
    pub account: u64,
    pub login: Box<[u8]>,
    pub token: Secret,
    pub expires: Wall,
}
/// A kept identity and announcement state, updated by the component's token events.
#[derive(Debug)]
pub struct Forge {
    pub number: u32,
    pub login: Box<[u8]>,
    pub author: settings::Author,
    pub entries: Map<u64, Entry>,
    pub announced: bool,
    pub made: List<(Box<[u8]>, u64)>,
}
/// Token state shared with the component, bounded by the deployment forge limit.
#[derive(Debug)]
pub struct Tokens {
    /// Kept identities and token state updated by the component's events.
    pub forges: Map<u16, Forge>,
}
impl Tokens {
    #[must_use]
    pub fn empty() -> Self {
        Self { forges: Map::with_capacity(0) }
    }
    /// Reserve token state from the checked deployment settings.
    #[must_use]
    pub fn new(settings: &settings::Engine) -> Self {
        Self { forges: Map::with_capacity(settings.limits.forge.forges) }
    }
    /// Whether a kept identity matches a record being installed.
    #[must_use]
    pub fn same_identity(&self, identity: &settings::Identity) -> bool {
        match self.forges.get(&identity.forge) {
            Some(held) => {
                held.number == identity.number && held.login == identity.login && held.author == identity.author
            }
            None => false,
        }
    }
    /// Remove the forge and return its identity number to the component.
    pub fn forget(&mut self, forge: u16) -> Option<u32> {
        match self.forges.remove(&forge) {
            Some(state) => Some(state.number),
            None => None,
        }
    }
    /// Install identity state before the component supplies its tokens.
    pub fn keep(&mut self, identity: &settings::Identity, installations: u32) {
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
/// Match the owner's ASCII login against the last value made.
#[must_use]
pub fn account(tokens: &Tokens, forge: u16, owner: &[u8]) -> Option<u64> {
    let state = tokens.forges.get(&forge)?;
    for (login, account) in &state.made {
        if login.eq_ignore_ascii_case(owner) {
            return Some(*account);
        }
    }
    None
}
