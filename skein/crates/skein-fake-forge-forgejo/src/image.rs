//! Protocol state kept across restarts (testing/fakes/forge.md, section 4).
//! Counts and integers are big-endian; byte strings carry their u64 length.
use crate::{Repository, Service, delivery::Endpoint};
use alloc::boxed::Box;
use skein_lib::{List, Reader, Secret};
pub(crate) mod codec;
use codec::{Encoder, data, flag, num, reader};

/// An image refused before any state is replaced. The name identifies the field.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    Malformed(&'static str),
    Limits(&'static str),
}
impl Service {
    /// Keeps the setup store, repository table and owner's hook endpoint, not connection state.
    #[must_use]
    pub fn image(&self) -> Box<[u8]> {
        self.image_with_hooks(self.config.hooks.as_ref())
    }
    /// Keeps an owner's endpoint held outside the service between hosted passes.
    #[must_use]
    pub fn image_with_hooks(&self, hooks: Option<&Endpoint>) -> Box<[u8]> {
        let mut measure = Encoder::measure();
        self.encode_image(&mut measure, hooks);
        let mut out = measure.writer();
        self.encode_image(&mut out, hooks);
        out.finish()
    }
    fn encode_image(&self, out: &mut Encoder, hooks: Option<&Endpoint>) {
        out.bytes(b"forgejo image 2");
        self.setup.encode_image(out);
        out.num(u64::from(self.config.repositories.len()));
        for repository in &self.config.repositories {
            out.num(u64::from(repository.id));
            out.bytes(&repository.full);
            out.flag(repository.protected.is_some());
            if let Some(branch) = &repository.protected {
                out.bytes(branch);
            }
        }
        out.flag(hooks.is_some());
        if let Some(endpoint) = hooks {
            out.num(u64::from(endpoint.forge));
            out.bytes(&endpoint.host);
            out.bytes(endpoint.secret.expose());
        }
    }
    /// Replaces retained state on a service built from the same configuration.
    /// A refusal leaves all of that service's state untouched.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), Refusal> {
        let mut input = reader(bytes)?;
        if data(&mut input, 32, "format").or(Err(Refusal::Malformed("format")))?.as_ref() != b"forgejo image 2" {
            return Err(Refusal::Malformed("format"));
        }
        let setup = self.setup.decode_image(&mut input)?;
        let repositories = self.repositories_from_image(&mut input)?;
        let hooks = if flag(&mut input, "endpoint")? {
            let forge =
                u16::try_from(num(&mut input, "endpoint forge")?).or(Err(Refusal::Malformed("endpoint forge")))?;
            // The owner's endpoint has no length bound at set_hooks; only emptiness is refused.
            let host = data(&mut input, bytes.len(), "endpoint host")?;
            let secret = Secret::new(data(&mut input, bytes.len(), "endpoint secret")?);
            if host.is_empty() || secret.is_empty() {
                return Err(Refusal::Malformed("endpoint"));
            }
            Some(Endpoint { forge, host, secret })
        } else {
            None
        };
        if !input.is_empty() {
            return Err(Refusal::Malformed("trailing bytes"));
        }
        self.setup = setup;
        self.config.repositories = repositories;
        self.config.hooks = hooks;
        self.refused = None;
        Ok(())
    }

    fn repositories_from_image(&self, input: &mut Reader<'_>) -> Result<List<Repository>, Refusal> {
        let capacity = self.config.repositories.capacity();
        let length = codec::count(input, capacity, "repositories")?;
        let mut repositories: List<Repository> = List::with_capacity(capacity);
        for _ in 0..length {
            let id = codec::small(input, "repository id")?;
            if id == 0 {
                return Err(Refusal::Malformed("repository id"));
            }
            let full = data(input, self.repository_bytes, "repository name")?;
            let protected = if flag(input, "repository protected")? {
                Some(data(input, self.branch_bytes, "repository branch")?)
            } else {
                None
            };
            for prior in &repositories {
                if prior.id == id || prior.full == full {
                    return Err(Refusal::Malformed("duplicate repository"));
                }
            }
            repositories.push(Repository { id, full, protected }).expect("admitted repository count");
        }
        Ok(repositories)
    }
}
