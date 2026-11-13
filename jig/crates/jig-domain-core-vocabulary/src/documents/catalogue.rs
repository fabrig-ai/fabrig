//! Public offers and payer readiness (client/domain/contract/documents.md, 1, and client/domain/contract/README.md, 5.1).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

use super::{DocumentLimits, Text};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use skein_lib::List;

/// Public configuration, independent of private account identities.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent configured source offers and cut facts")]
pub struct ProviderOffer {
    /// Stable configured LLM provider number.
    pub provider: u32,
    /// Actual startup key ceiling for this provider's key offer.
    pub key_bytes: u32,
    /// Configured display name, carrying its own text loss.
    pub name: Text,
    /// Whether a sign-in source is offered.
    pub sign_in: bool,
    /// Whether a key source is offered.
    pub key: bool,
    /// Whole configured program names within the list bound.
    pub programs: List<Box<[u8]>>,
    /// Whether additional program names were omitted.
    pub programs_cut: bool,
    /// Whether configuration offers a source needing no credential.
    pub none: bool,
    /// Whether configured sign-ins use plan terms.
    pub signed_in_plan: bool,
    /// Whole optional public HTTPS manage target, at most 2,048 bytes.
    pub manage_usage: Option<Box<[u8]>>,
}
/// One model of the party's offer (jig/docs/design/domain/engine/views.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Offered {
    pub model: u8,
    pub accounts: List<u32>,
}
/// Availability is an observation, never a run reservation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Readiness {
    Usable,
    Unavailable,
    Unknown,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PayerReadiness {
    /// Stable configured LLM provider number.
    pub provider: u32,
    /// The authenticated party or a project it may read.
    pub payer: accounts::Use,
    /// Current observation from the engine selection and host checks.
    pub readiness: Readiness,
    /// None until program check facts can establish settlement.
    pub settled: Option<bool>,
}

pub fn offers_bound(limits: &DocumentLimits) -> Option<u64> {
    let heap = List::<Box<[u8]>>::price(limits.lines)?
        .checked_add(u64::from(limits.lines).checked_mul(32)?)?
        .checked_add(u64::from(limits.text_bytes))?
        .checked_add(u64::from(accounts::MANAGE_USAGE))?;
    List::<ProviderOffer>::price(limits.lines)?.checked_add(u64::from(limits.lines).checked_mul(heap)?)
}
pub fn rows_bound(limits: &DocumentLimits) -> Option<u64> {
    List::<PayerReadiness>::price(limits.lines)
}
