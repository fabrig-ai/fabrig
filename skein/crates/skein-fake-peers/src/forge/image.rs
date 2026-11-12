//! The peer's lasting domain and layer state (testing/fakes/forge.md, section 5).
use super::{Peer, layer::Service};
use skein_fake_forge_domain as domain;
use skein_lib::{Queue, Reader, Time, Writer, len_u64};

/// The part of an image that could not be taken or restored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    Busy,
    Malformed(&'static str),
    Domain(domain::image::Refusal),
    Forgejo(skein_fake_forge_forgejo::image::Refusal),
    Github(skein_fake_forge_github::image::Refusal),
}
impl Peer {
    /// Takes an image after shutdown, refusing a domain call still awaiting its reply.
    pub fn image(&self) -> Result<Box<[u8]>, Refusal> {
        let forge = self.forge.borrow();
        let domain = forge.domain.image().map_err(domain_refusal)?;
        let service = match &self.service {
            Service::Forgejo(service) => service.image_with_hooks(forge.hooks.as_ref()),
            Service::Github(service) => service.image(),
        };
        let size = domain
            .len()
            .checked_add(service.len())
            .and_then(|n| n.checked_add(16))
            .ok_or(Refusal::Malformed("image length"))?;
        u32::try_from(size).map_err(|_| Refusal::Malformed("image length"))?;
        let mut out = Writer::new(size);
        for part in [&domain, &service] {
            out.put(&len_u64(part.len()).to_be_bytes()).expect("measured image");
            out.put(part).expect("measured image");
        }
        Ok(out.finish())
    }
    /// Replaces lasting state on a fresh peer, before its first pass.
    /// A refusal leaves both the shared domain and the service unchanged.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), Refusal> {
        assert!(self.fresh, "an image is restored before the peer's first pass");
        u32::try_from(bytes.len()).map_err(|_| Refusal::Malformed("image length"))?;
        let mut input = Reader::new(bytes);
        let domain_bytes = part(&mut input, "domain length")?;
        let service_bytes = part(&mut input, "service length")?;
        if !input.is_empty() {
            return Err(Refusal::Malformed("trailing bytes"));
        }
        let mut forge = self.forge.borrow_mut();
        let domain = domain::Domain::from_image(&forge.config, domain_bytes).map_err(domain_refusal)?;
        let hooks = match &mut self.service {
            Service::Forgejo(service) => {
                service.restore(service_bytes).map_err(Refusal::Forgejo)?;
                service.set_hooks(None).expect("clearing an endpoint is valid")
            }
            Service::Github(service) => {
                service.restore(service_bytes).map_err(Refusal::Github)?;
                None
            }
        };
        forge.domain = domain;
        forge.now = Time::ZERO;
        forge.pending = Queue::uncapped();
        forge.hooks = hooks;
        Ok(())
    }
}
fn part<'a>(input: &mut Reader<'a>, name: &'static str) -> Result<&'a [u8], Refusal> {
    let length = input.u64().and_then(|n| u32::try_from(n).ok()).ok_or(Refusal::Malformed(name))?;
    input.bytes(length).ok_or(Refusal::Malformed(name))
}
fn domain_refusal(why: domain::image::Refusal) -> Refusal {
    match why {
        domain::image::Refusal::Busy => Refusal::Busy,
        domain::image::Refusal::Malformed(_) | domain::image::Refusal::Limits(_) => Refusal::Domain(why),
    }
}
