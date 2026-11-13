//! Lent account credentials stay in the protocol layer (protocol/credentials.md, 3).
use alloc::boxed::Box;
use jig_domain_link::{Form, Grant};
use jig_protocol_accounts::table::{self, Table};
use jig_protocol_engine_workers::{GrantValue, Scheme};
use skein_lib::{List, Map, Secret, Time};

pub use temper_service_core::values::{grant_bytes, token_credential_bytes};

/// A configured key can exceed the OAuth token bound. The worker has no
/// provider declarations and reserves the key ceiling unconditionally.
#[must_use]
pub fn credential_bytes(engine: &jig_protocol_config::Engine) -> u32 {
    let token = engine.limits.token;
    for provider in &engine.llm.providers {
        if provider.keys {
            return token_credential_bytes(token);
        }
    }
    token
}
/// The engine's git identities' values (protocol/credentials.md, 4): two newest
/// generations per number, each encoded once by temper-protocol-worker-wire.
#[derive(Debug)]
pub struct Identities {
    entries: Map<u32, Generations>,
}
#[derive(Debug)]
struct Generations {
    generation: u64,
    newest: Secret,
    previous: Option<Secret>,
}
impl Identities {
    /// Room for each kept identity's number.
    #[must_use]
    pub fn with_capacity(identities: u32) -> Self {
        Self { entries: Map::with_capacity(identities) }
    }
    /// Keep the next generation, pushing out the older of the two retained values.
    pub fn put(&mut self, number: u32, value: Secret) -> u64 {
        if let Some(entry) = self.entries.get_mut(&number) {
            entry.previous = Some(core::mem::replace(&mut entry.newest, value));
            entry.generation = entry.generation.checked_add(1).expect("identity generations");
            return entry.generation;
        }
        self.entries
            .insert(number, Generations { generation: 1, newest: value, previous: None })
            .expect("kept forge identity capacity");
        1
    }
    /// Forget the credential generations when the component removes an identity.
    pub fn forget(&mut self, number: u32) {
        self.entries.remove(&number);
    }
    /// Either of the two newest generations, or none for an older one.
    #[must_use]
    pub fn get(&self, number: u32, generation: u64) -> Option<&Secret> {
        let entry = self.entries.get(&number)?;
        if generation == entry.generation {
            Some(&entry.newest)
        } else if generation != 0 && generation.checked_add(1) == Some(entry.generation) {
            entry.previous.as_ref()
        } else {
            None
        }
    }
    /// The newest generation and its value.
    #[must_use]
    pub fn newest(&self, number: u32) -> Option<(u64, &Secret)> {
        let entry = self.entries.get(&number)?;
        Some((entry.generation, &entry.newest))
    }
    /// Identity numbers the component supplies to its root at start.
    #[must_use]
    pub fn numbers(&self) -> List<u32> {
        let mut numbers = List::with_capacity(self.entries.len());
        for (number, _) in &self.entries {
            numbers.push(*number).expect("identity count");
        }
        numbers
    }
}
#[derive(Debug)]
pub struct Values {
    pub accounts: Table,
    /// The two newest encoded values of each kept git identity.
    pub identities: Identities,
    pub tokens: crate::forge::tokens::Tokens,
}
impl Values {
    /// Named source slots, bounded by the accounts driver.
    #[must_use]
    pub fn new(accounts: u32, token_bytes: u32) -> Self {
        Self {
            accounts: Table::new(accounts, token_bytes),
            identities: Identities::with_capacity(0),
            tokens: crate::forge::tokens::Tokens::empty(),
        }
    }
}

impl Values {
    /// One optional value per grant, preserving the root's order.
    #[must_use]
    pub fn grants(&self, grants: &[Grant], now: Time) -> Box<[Option<GrantValue>]> {
        let mut values = List::with_capacity(u32::try_from(grants.len()).expect("bounded grants"));
        for grant in grants {
            let value = match grant.form {
                Form::Credential(_) => {
                    account_grant(grant.form, self.accounts.value(grant.account, grant.generation, now))
                }
                Form::Application => identity_grant(self.identities.get(grant.account, grant.generation)),
                Form::Program(name) => Some(GrantValue::Program { name }),
            };
            values.push(value).expect("one slot per grant");
        }
        values.into_boxed()
    }
}

fn account_grant(form: Form, value: Option<&table::Value>) -> Option<GrantValue> {
    let value = value?;
    let actual = match &value.scheme {
        table::Scheme::Bearer => Form::Credential(jig_domain_link::Scheme::Bearer),
        table::Scheme::Key => Form::Credential(jig_domain_link::Scheme::Key),
    };
    if form != actual {
        return None;
    }
    Some(match &value.scheme {
        table::Scheme::Bearer => GrantValue::Credential { scheme: Scheme::Bearer, token: value.token.clone() },
        table::Scheme::Key => GrantValue::Credential { scheme: Scheme::Key, token: value.token.clone() },
    })
}
fn identity_grant(value: Option<&Secret>) -> Option<GrantValue> {
    let value = value?;
    Some(GrantValue::Application { secret: value.clone() })
}
impl Values {
    /// Reserve identity and token tables from the component's checked settings.
    pub fn fill(&mut self, settings: &crate::settings::Engine) {
        self.tokens = crate::forge::tokens::Tokens::new(settings);
        self.identities = Identities::with_capacity(settings.limits.forge.forges);
    }
}
