//! Signing state, without request records (testing/fakes/github.md, 1.7).
use super::Store;
use crate::{
    Limits,
    image::{
        Refusal,
        codec::{Encoder, data, optional_data},
    },
};
use skein_lib::{Queue, Reader, Secret, usize_of};
impl Store {
    pub(crate) fn encode_image(&self, out: &mut Encoder) {
        out.bytes(self.key.expose());
        out.bytes(&self.origin);
        out.optional_data(self.third_origin.as_deref());
        out.optional_data(self.redirect.as_deref());
    }
    pub(crate) fn decode_image(input: &mut Reader<'_>, limits: &Limits) -> Result<Self, Refusal> {
        let key = Secret::new(data(input, 32, "job log key")?);
        if key.len() != 32 {
            return Err(Refusal::Malformed("job log key"));
        }
        let bound = usize_of(limits.github.target_bytes);
        let origin = data(input, bound, "blob origin")?;
        let third_origin = optional_data(input, bound, "third origin")?;
        let redirect = optional_data(input, bound, "blob redirect")?;
        if !super::origin_fits(&origin, limits)
            || !super::optional_origin_fits(third_origin.as_deref(), limits)
            || !super::optional_origin_fits(redirect.as_deref(), limits)
        {
            return Err(Refusal::Limits("job log origins"));
        }
        Ok(Self { key, origin, third_origin, redirect, records: Queue::with_capacity(limits.blob_records) })
    }
}
