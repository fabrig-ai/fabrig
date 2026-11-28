//! Connector effect identities and origins (jig/docs/design/domain/engine/vocabulary.md, 1.3).

pub use jig_domain_contract::effects::EffectOrigin;

pub use jig_domain_contract::effects::{EffectKey, EffectPurpose};
pub mod connector;
pub mod projections;
