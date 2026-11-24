//! Public configured offers and volatile payer facts (client/domain/contract/documents.md, 1).
use super::{accounts, pieces, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::{List, Writer};

pub(super) fn offers(values: &List<d::ProviderOffer>, bound: u32, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        let row = mapped::provider_offer_to_record(value);
        let mut out =
            Writer::new(usize::try_from(row.measure(limits)?).or(Err(super::problem(skein_codec::Reason::Bound)))?);
        row.write(limits, &mut out).or(Err(super::problem(skein_codec::Reason::Bound)))?;
        rows.push(out.finish()).or(Err(super::problem(skein_codec::Reason::Bound)))?;
    }
    pieces(&rows, bound, w::paths::ME_PROVIDERS)
}
pub(super) fn readiness(
    values: &List<d::PayerReadiness>,
    bound: u32,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        let row = w::PayerReadiness {
            provider: value.provider,
            payer: accounts::payer(value.payer),
            readiness: mapped::readiness_to_record(&value.readiness),
            settled: value.settled,
        };
        let mut out =
            Writer::new(usize::try_from(row.measure(limits)?).or(Err(super::problem(skein_codec::Reason::Bound)))?);
        row.write(limits, &mut out).or(Err(super::problem(skein_codec::Reason::Bound)))?;
        rows.push(out.finish()).or(Err(super::problem(skein_codec::Reason::Bound)))?;
    }
    pieces(&rows, bound, w::paths::ACCOUNTS_READINESS)
}
