//! Public configured offers and volatile payer facts (client/domain/contract/documents.md, 1).
use jig_client_protocol_wire::Record as _;
use super::{accounts, owner, pieces, w};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::{List, Writer};

pub(super) fn offers(values: &List<d::ProviderOffer>, bound: u32, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        let row = w::ProviderOffer {
                sign_in: value.sign_in,
                key: value.key,
                programs_cut: value.programs_cut,
                none: value.none,
                signed_in_plan: value.signed_in_plan,
                provider: value.provider,
                key_bytes: value.key_bytes,
                name: owner::text(&value.name, limits)?,
                programs: value.programs.clone(),
                manage_usage: value.manage_usage.clone(),
            };
        let mut out = Writer::new(usize::try_from(row.measure(limits)?).or(Err(super::problem(skein_codec::Reason::Bound)))?);
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
                payer: accounts::payer(value.payer, limits)?,
                readiness: match value.readiness {
                    d::Readiness::Usable => w::Readiness::Usable,
                    d::Readiness::Unavailable => w::Readiness::Unavailable,
                    d::Readiness::Unknown => w::Readiness::Unknown,
                },
                settled: value.settled,
            };
        let mut out = Writer::new(usize::try_from(row.measure(limits)?).or(Err(super::problem(skein_codec::Reason::Bound)))?);
        row.write(limits, &mut out).or(Err(super::problem(skein_codec::Reason::Bound)))?;
        rows.push(out.finish()).or(Err(super::problem(skein_codec::Reason::Bound)))?;
    }
    pieces(&rows, bound, w::paths::ACCOUNTS_READINESS)
}
