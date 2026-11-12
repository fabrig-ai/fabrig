//! GitHub's retained layer state (testing/fakes/github.md, 1.7).
//! Counts and integers are big-endian; byte strings carry their u64 length.
use crate::{
    Service,
    app::App,
    logs, pages,
    service::{Account, Row, Shown},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, Rng, Secret, Wall, usize_of};
pub(crate) mod codec;
use codec::{Encoder, count, data, flag, num, optional_num, reader, small, strings};

/// An image refused without replacing any state. The name identifies the field.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    Malformed(&'static str),
    Limits(&'static str),
}
impl Service {
    /// Keeps durable layer state, excluding launch and connection records.
    #[must_use]
    pub fn image(&self) -> Box<[u8]> {
        let mut measure = Encoder::measure();
        self.encode_image(&mut measure);
        let mut out = measure.writer();
        self.encode_image(&mut out);
        out.finish()
    }
    #[expect(clippy::manual_map, reason = "step code spells optional wall times without closures")]
    fn encode_image(&self, out: &mut Encoder) {
        out.bytes(b"github image");
        out.num(self.rng.state());
        out.num(self.app.id);
        out.bytes(&self.app.slug);
        out.num(self.app.owner);
        out.bytes(self.app.key);
        out.bytes(self.app.webhook.expose());
        pages::image::permissions(out, &self.app.permissions);
        out.strings(&self.app.events);
        out.flag(self.key_revoked);
        out.bytes(&self.hook);
        out.num(match self.shown {
            Shown::Ruleset => 0,
            Shown::Classic => 1,
            Shown::Both => 2,
        });
        out.num(self.ruleset_id);
        out.flag(self.pull_requests_only);
        out.flag(self.required_signatures);
        out.num(u64::from(self.accounts.len()));
        for account in &self.accounts {
            out.bytes(&account.login);
            out.num(account.id);
            out.flag(account.organization);
        }
        out.bytes(&self.person);
        out.num(u64::from(self.tokens.len()));
        for (secret, user, expires) in &self.tokens {
            out.bytes(secret.expose());
            out.num(*user);
            out.optional_num(match expires {
                Some(time) => Some(time.as_nanos()),
                None => None,
            });
        }
        out.num(u64::from(self.repositories.len()));
        for (_, row) in &self.repositories {
            out.num(u64::from(row.id));
            out.bytes(&row.owner);
            out.num(row.owner_id);
            out.flag(row.organization);
            out.bytes(&row.name);
            out.flag(row.private);
        }
        self.installations.encode_image(out);
        self.pages.encode_image(out);
        self.deliveries.encode_image(out);
        self.logs.encode_image(out);
    }
    /// Replaces durable state on a service built from the same configuration.
    /// Malformed and over-limit images leave the service untouched.
    #[expect(clippy::manual_map, reason = "step code spells optional wall times without closures")]
    #[expect(clippy::too_many_lines, reason = "the flat image schema stages every field before one atomic replacement")]
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), Refusal> {
        let mut input = reader(bytes)?;
        if data(&mut input, 32, "format").or(Err(Refusal::Malformed("format")))?.as_ref() != b"github image" {
            return Err(Refusal::Malformed("format"));
        }
        let limits = self.limits;
        let string = usize_of(limits.github.string_bytes);
        let cap = limits.github.inner;
        let rng = Rng::new(num(&mut input, "service rng")?);
        let id = num(&mut input, "app id")?;
        let slug = data(&mut input, string, "app slug")?;
        let owner = num(&mut input, "app owner")?;
        let key_bytes = data(&mut input, self.app.key.len().max(2048), "app key")?;
        let mut key = None;
        if key_bytes.as_ref() == self.app.key {
            key = Some(self.app.key);
        }
        for candidate in crate::keys::KEYS {
            if key_bytes.as_ref() == candidate {
                key = Some(candidate);
            }
        }
        let key = key.ok_or(Refusal::Malformed("app key"))?;
        // Seeded secrets have no configured length bound; conversion makes 32 bytes.
        let webhook = Secret::new(data(&mut input, self.app.webhook.len().max(32), "app webhook secret")?);
        let permissions = pages::image::read_permissions(&mut input, &limits)?.into_boxed();
        let events = strings(&mut input, cap, string, "app events")?;
        if id == 0
            || id == self.bot
            || slug.is_empty()
            || slug.len().checked_add(b"[bot]".len()).ok_or(Refusal::Limits("app slug"))? > string
        {
            return Err(Refusal::Malformed("app identity"));
        }
        let app = App { id, slug, owner, permissions, events, key, webhook };
        let key_revoked = flag(&mut input, "app key revoked")?;
        let hook = data(&mut input, string, "app hook")?;
        let shown = match num(&mut input, "protection display")? {
            0 => Shown::Ruleset,
            1 => Shown::Classic,
            2 => Shown::Both,
            _ => return Err(Refusal::Malformed("protection display")),
        };
        let ruleset_id = num(&mut input, "ruleset id")?;
        if ruleset_id == 0 {
            return Err(Refusal::Malformed("ruleset id"));
        }
        let pull_requests_only = flag(&mut input, "pull requests only")?;
        let required_signatures = flag(&mut input, "required signatures")?;
        let length = count(&mut input, cap, "accounts")?;
        let mut accounts: List<Account> = List::with_capacity(cap);
        for _ in 0..length {
            let login = data(&mut input, string, "account login")?;
            let id = num(&mut input, "account id")?;
            let organization = flag(&mut input, "account organization")?;
            if id == 0 || !pages::unreserved(&login) {
                return Err(Refusal::Malformed("account identity"));
            }
            for prior in &accounts {
                if prior.id == id || prior.login == login {
                    return Err(Refusal::Malformed("duplicate account"));
                }
            }
            accounts.push(Account { login, id, organization }).expect("admitted accounts");
        }
        let person = data(&mut input, string, "person")?;
        let mut found = false;
        for account in &accounts {
            if account.login == person {
                found = true;
            }
        }
        if !found {
            return Err(Refusal::Malformed("person account"));
        }
        let length = count(&mut input, cap, "person tokens")?;
        let mut tokens: List<(Secret, u64, Option<Wall>)> = List::with_capacity(cap);
        for _ in 0..length {
            let secret = Secret::new(data(&mut input, string, "person token")?);
            let user = num(&mut input, "person token user")?;
            let expires = match optional_num(&mut input, "person token expiry")? {
                Some(value) => Some(Wall::from_nanos(value)),
                None => None,
            };
            let mut person_found = false;
            for account in &accounts {
                if account.id == user && !account.organization {
                    person_found = true;
                }
            }
            if secret.is_empty() || !person_found {
                return Err(Refusal::Malformed("person token"));
            }
            for (prior, _, _) in &tokens {
                if prior.expose() == secret.expose() {
                    return Err(Refusal::Malformed("duplicate person token"));
                }
            }
            tokens.push((secret, user, expires)).expect("admitted person tokens");
        }
        let length = count(&mut input, cap, "repositories")?;
        let mut repositories: Map<u32, Row> = Map::with_capacity(cap);
        for _ in 0..length {
            let id = small(&mut input, "repository id")?;
            let owner = data(&mut input, string, "repository owner")?;
            let owner_id = num(&mut input, "repository owner id")?;
            let organization = flag(&mut input, "repository organization")?;
            let name = data(&mut input, string, "repository name")?;
            let private = flag(&mut input, "repository private")?;
            if id == 0
                || owner_id == 0
                || owner.is_empty()
                || name.is_empty()
                || owner.contains(&b'/')
                || name.contains(&b'/')
                || repositories.contains_key(&id)
            {
                return Err(Refusal::Malformed("repository identity"));
            }
            let full_length = b"https://github.com/"
                .len()
                .checked_add(owner.len())
                .ok_or(Refusal::Limits("repository full name"))?
                .checked_add(1)
                .ok_or(Refusal::Limits("repository full name"))?
                .checked_add(name.len());
            if full_length.ok_or(Refusal::Limits("repository full name"))? > string {
                return Err(Refusal::Limits("repository full name"));
            }
            for (_, prior) in &repositories {
                if prior.owner == owner && prior.name == name {
                    return Err(Refusal::Malformed("duplicate repository"));
                }
            }
            drop(
                repositories
                    .insert(id, Row { id, owner, owner_id, organization, name, private })
                    .expect("admitted repositories"),
            );
        }
        let installations = self.installations.decode_image(&mut input, &limits.installations)?;
        let pages = pages::Store::decode_image(&mut input, &limits)?;
        let deliveries = self.deliveries.decode_image(&mut input, &limits)?;
        let logs = logs::Store::decode_image(&mut input, &limits)?;
        if !input.is_empty() {
            return Err(Refusal::Malformed("trailing bytes"));
        }
        self.rng = rng;
        self.app = app;
        self.key_revoked = key_revoked;
        self.hook = hook;
        self.shown = shown;
        self.ruleset_id = ruleset_id;
        self.pull_requests_only = pull_requests_only;
        self.required_signatures = required_signatures;
        self.accounts = accounts;
        self.person = person;
        self.tokens = tokens;
        self.repositories = repositories;
        self.installations = installations;
        self.pages = pages;
        self.deliveries.restore_image(deliveries);
        self.logs = logs;
        self.posts.clear();
        self.git_calls.clear();
        self.rate_heads = Map::with_capacity(limits.github.inner);
        self.verified.forget();
        Ok(())
    }
}
