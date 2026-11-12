//! Protocol state kept across restarts (testing/fakes/forge.md, section 4).
//! Counts and integers are big-endian; byte strings carry their u64 length.
use crate::{Service, delivery::Endpoint};
use alloc::boxed::Box;
use skein_lib::Secret;
pub(crate) mod codec;
use codec::{Encoder, data, flag, num, reader};

/// An image refused before any state is replaced. The name identifies the field.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    Malformed(&'static str),
    Limits(&'static str),
}
impl Service {
    /// Keeps the setup store and the owner's hook endpoint, not connection state.
    #[must_use]
    pub fn image(&self) -> Box<[u8]> {
        let mut measure = Encoder::measure();
        self.encode_image(&mut measure);
        let mut out = measure.writer();
        self.encode_image(&mut out);
        out.finish()
    }
    fn encode_image(&self, out: &mut Encoder) {
        out.bytes(b"forgejo image");
        self.setup.encode_image(out);
        out.flag(self.config.hooks.is_some());
        if let Some(endpoint) = &self.config.hooks {
            out.num(u64::from(endpoint.forge));
            out.bytes(&endpoint.host);
            out.bytes(endpoint.secret.expose());
        }
    }
    /// Replaces retained state on a service built from the same configuration.
    /// A refusal leaves all of that service's state untouched.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), Refusal> {
        let mut input = reader(bytes)?;
        if data(&mut input, 32, "format").or(Err(Refusal::Malformed("format")))?.as_ref() != b"forgejo image" {
            return Err(Refusal::Malformed("format"));
        }
        let setup = self.setup.decode_image(&mut input)?;
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
        self.config.hooks = hooks;
        self.refused = None;
        Ok(())
    }
}
