//! The clients' side of a service (jig/docs/design/protocol/service.md, 11.6).
use crate as kit;
use alloc::boxed::Box;
use jig_domain_core_vocabulary as domain;
use jig_domain_people as people;
use jig_protocol_store as store;
use skein_lib::{List, Map, Token, Wall};

/// What a request in flight was for: a kept value's owner, or the exchange whose carried value the store numbers.
#[derive(Clone, Copy, Debug)]
enum Asked {
    Owner(kit::SecretOwner),
    Carry(Token),
}
/// The bridge between the component's secrets and the store's (protocol/service.md, 11.6).
#[derive(Debug)]
pub struct Bridge {
    owners: Map<Token, kit::SecretOwner>,
    carries: Map<Token, Token>,
    next: u64,
    namespace: u8,
}
impl Bridge {
    #[must_use]
    pub fn new(namespace: u8) -> Self {
        Self { owners: Map::with_capacity(u32::MAX), carries: Map::with_capacity(u32::MAX), next: 1, namespace }
    }
    /// Names a request and keeps what it was for; the owner calls its store with what this returns.
    pub fn ask(&mut self, request: kit::SecretsRequest) -> (Token, store::Secrets) {
        let (owner, request) = secret_request(request);
        let token = Token::tagged(self.namespace, self.next).expect("bounded secret routing token");
        self.next = self.next.checked_add(1).expect("secret routing sequence");
        match owner {
            Asked::Owner(owner) => {
                self.owners.insert(token, owner).expect("secret request correlation");
            }
            Asked::Carry(exchange) => {
                self.carries.insert(token, exchange).expect("secret carry correlation");
            }
        }
        (token, request)
    }
    /// No request is in flight.
    #[must_use]
    pub fn idle(&self) -> bool {
        self.owners.is_empty() && self.carries.is_empty()
    }
    /// The component's answer to a store's terminal for a token `ask` named.
    pub fn answer(&mut self, answer: store::Event) -> kit::SecretsAnswer {
        use store::Event;
        match answer {
            // The census goes to the root, never a secret asker (protocol/store/README.md, 6.3).
            Event::Census { .. } => unreachable!("census is not a secret terminal"),
            Event::Page { .. } | Event::PageRefused { .. } => unreachable!("a start's pages need a start-mode store"),
            Event::Kept { owner } => kit::SecretsAnswer::Kept { owner: self.owner(owner) },
            Event::Erased { owner } => kit::SecretsAnswer::Erased { owner: self.owner(owner) },
            Event::Carried { owner, number } => kit::SecretsAnswer::Carried {
                exchange: self.carries.remove(&owner).expect("secret carry owner"),
                number: Some(number),
            },
            Event::Stale { owner } => match self.carries.remove(&owner) {
                Some(exchange) => kit::SecretsAnswer::Carried { exchange, number: None },
                None => {
                    self.owners.remove(&owner).expect("secret stale owner");
                    kit::SecretsAnswer::Failed
                }
            },
            Event::Secret { owner, value } => {
                let owner = self.owner(owner);
                let Some(value) = value else {
                    return kit::SecretsAnswer::Got { owner, value: None };
                };
                let Some(value) = kept(value) else { return kit::SecretsAnswer::Failed };
                kit::SecretsAnswer::Got { owner, value: Some(value) }
            }
            Event::SecretsLoaded { .. }
            | Event::Switched { .. }
            | Event::Opened { .. }
            | Event::OpenRefused { .. }
            | Event::Site { .. }
            | Event::Decoded { .. }
            | Event::Undecodable { .. }
            | Event::Rehearsing { .. }
            | Event::Back { .. }
            | Event::BackRefused { .. }
            | Event::Closed { .. }
            | Event::Patched { .. }
            | Event::Committed { .. }
            | Event::Failed { .. }
            | Event::FrameTooLarge { .. }
            | Event::Loaded { .. }
            | Event::Read { .. }
            | Event::Got { .. }
            | Event::Turns { .. }
            | Event::Unread { .. }
            | Event::SecretsNotReady { .. }
            | Event::Corrupt { .. } => kit::SecretsAnswer::Failed,
        }
    }
    fn owner(&mut self, token: Token) -> kit::SecretOwner {
        self.owners.remove(&token).expect("secret request owner")
    }
}
fn secret_request(request: kit::SecretsRequest) -> (Asked, store::Secrets) {
    use store::Secrets;
    match request {
        kit::SecretsRequest::Keep { owner, value } => {
            let (kind, number) = place(owner);
            let value = if owner == kit::SecretOwner::PublicNames {
                store::SecretValue::Key(value.digest)
            } else {
                store::SecretValue::Digest {
                    expiry: value.expires.as_nanos(),
                    digest: value.digest.expose().try_into().expect("credential digest is 32 bytes"),
                }
            };
            (Asked::Owner(owner), Secrets::Keep { owner: kind, number, value })
        }
        kit::SecretsRequest::Erase { owner } => {
            let (kind, number) = place(owner);
            (Asked::Owner(owner), Secrets::Erase { owner: kind, number })
        }
        kit::SecretsRequest::Get { owner } => {
            let (kind, number) = place(owner);
            (Asked::Owner(owner), Secrets::Get { owner: kind, number })
        }
        kit::SecretsRequest::Hold { exchange, value } => (Asked::Carry(exchange), Secrets::Hold { value }),
        kit::SecretsRequest::Carry { exchange, value, kind } => {
            let kind = match kind {
                kit::CarriedKind::Key => store::CarriedKind::Key,
                kit::CarriedKind::Value => store::CarriedKind::Value,
                kit::CarriedKind::Moved => store::CarriedKind::Moved,
            };
            (Asked::Carry(exchange), Secrets::Carry { value, kind })
        }
    }
}
/// Where the store keeps an owner's value; a carried one is the application's, under the store's number (protocol/store/secrets.md).
#[must_use]
fn place(owner: kit::SecretOwner) -> (store::SecretOwner, u64) {
    use store::SecretOwner;
    match owner {
        kit::SecretOwner::SignIn(number) => (SecretOwner::SignIn, number),
        kit::SecretOwner::Service(number) => (SecretOwner::Service, number),
        kit::SecretOwner::Carried(number) => (SecretOwner::Application, number),
        kit::SecretOwner::PublicNames => (SecretOwner::PublicNames, 0),
    }
}
fn kept(value: store::SecretValue) -> Option<kit::Kept> {
    match value {
        store::SecretValue::Digest { expiry, digest } => {
            Some(kit::Kept { digest: skein_lib::Secret::new(Box::new(digest)), expires: Wall::from_nanos(expiry) })
        }
        store::SecretValue::Key(key) if key.len() == 32 => {
            Some(kit::Kept { digest: key, expires: Wall::from_nanos(u64::MAX) })
        }
        store::SecretValue::Record(_)
        | store::SecretValue::Key(_)
        | store::SecretValue::Carried { .. }
        | store::SecretValue::Application(_)
        | store::SecretValue::Held(_) => None,
    }
}
/// Sign-ins' endings, each held until its own commit is durable (domain/root.md, 5).
#[derive(Debug)]
pub struct Endings {
    commits: Map<u64, Ending>,
    durable: u64,
}
#[derive(Debug)]
struct Ending {
    numbers: List<u64>,
    next: u32,
}
impl Endings {
    #[must_use]
    pub fn new(commits: u32) -> Self {
        Self { commits: Map::with_capacity(commits), durable: 0 }
    }
    /// A commit ends this sign-in; a sign-in it already ends is one ending.
    pub fn ended(&mut self, commit: u64, sign_in: u64) {
        if self.commits.get(&commit).is_none() {
            self.commits
                .insert(commit, Ending { numbers: List::with_capacity(u32::MAX), next: 0 })
                .expect("bounded commits in flight");
        }
        let ending = self.commits.get_mut(&commit).expect("ending commit");
        if !ending.numbers.as_slice().contains(&sign_in) {
            ending.numbers.push(sign_in).expect("bounded commit writes");
        }
    }
    pub fn committed(&mut self, number: u64) {
        self.durable = self.durable.max(number);
    }
    /// Room for one more commit's endings.
    #[must_use]
    pub fn room(&self) -> bool {
        self.commits.len() < self.commits.capacity()
    }
    fn ending_commit_durable(&self, number: u64) -> bool {
        number <= self.durable
    }
    /// An ending's commit is durable.
    #[must_use]
    pub fn ready(&self) -> bool {
        match self.commits.first() {
            Some((number, _)) => self.ending_commit_durable(*number),
            None => false,
        }
    }
    /// The endings held.
    #[must_use]
    pub fn waiting(&self) -> u32 {
        let mut count = 0_u32;
        for (_, ending) in &self.commits {
            let left = ending.numbers.len().checked_sub(ending.next).expect("ending index");
            count = count.checked_add(left).expect("bounded sign-in endings");
        }
        count
    }
    /// The next ending whose commit is durable, in commit order.
    #[expect(clippy::should_implement_trait, reason = "a pull advances only behind the durable commit")]
    pub fn next(&mut self) -> Option<u64> {
        let (&number, _) = self.commits.first()?;
        if !self.ending_commit_durable(number) {
            return None;
        }
        let ending = self.commits.get_mut(&number).expect("durable endings");
        let sign_in = *ending.numbers.get(ending.next).expect("one sign-in per erased key");
        ending.next = ending.next.checked_add(1).expect("bounded erase index");
        if ending.next == ending.numbers.len() {
            self.commits.remove(&number);
        }
        Some(sign_in)
    }
}
/// The sign-in a core key's erasure ends.
#[must_use]
pub fn ending(key: &domain::Key) -> Option<u64> {
    match key {
        domain::Key::People(people::Key::SignIn(number)) => Some(*number),
        domain::Key::People(
            people::Key::Person(..)
            | people::Key::ReadResults { .. }
            | people::Key::EndedRead { .. }
            | people::Key::ReadConversation { .. }
            | people::Key::Roles(..)
            | people::Key::Deleted(..)
            | people::Key::Deletion(..)
            | people::Key::Public(..)
            | people::Key::Policy(..)
            | people::Key::Acted(..)
            | people::Key::Answer(..),
        )
        | domain::Key::Core(..)
        | domain::Key::Tasks(..)
        | domain::Key::Notes(..)
        | domain::Key::Accounts(..) => None,
    }
}
/// jig's own client protocol, then the application's family.
#[must_use]
pub fn served(application: kit::Served) -> List<kit::Served> {
    use jig_client_protocol_wire::VERSION;
    List::from_boxed(Box::new([kit::Served { family: None, first: VERSION, last: VERSION }, application]))
}

#[cfg(test)]
mod tests;
