//! Retained outcomes, field for field with the forge's application family.
use super::FITS;
use crate::{Bounds, Undecodable, v25 as w};
use alloc::boxed::Box;
use jig_protocol_store::people_v17::Record as _;
use skein_lib::{Reader, Wall};
use temper_engine_domain_forge_vocabulary as d;
mod refusal;

pub(super) fn encode(token: u64, until: Wall, outcome: &d::ApplicationOutcome) -> w::Application {
    w::Application {
        token,
        until: until.as_nanos(),
        outcome: crate::generated::store::application_outcome_to_record(outcome),
    }
}
pub(super) fn decode(value: w::Application, bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    let w::Application { token, until, outcome } = value;
    let outcome = crate::generated::store::application_outcome_to_domain(outcome, &super::Context::bounded(bounds))
        .or(Err(Undecodable::Malformed))?;
    Ok(d::Stored::Application { token, until: Wall::from_nanos(until), outcome })
}

/// Owner-shaped `CoreRefused` (skein/docs/design/data/codec/maps.md, 3.3).
#[must_use]
pub fn core_refused_to_record(value: &jig_domain_people::Refusal) -> w::CoreRefused {
    w::CoreRefused {
        refusal: refusal::encode_refusal(value, &jig_protocol_store::people_v17::CEILINGS)
            .expect("refusal facts fit")
            .encode(&jig_protocol_store::people_v17::CEILINGS)
            .expect("refusal facts fit"),
    }
}
pub fn core_refused_to_domain(value: w::CoreRefused) -> Result<jig_domain_people::Refusal, w::Problem> {
    let record = jig_protocol_store::people_v17::Refusal::decode(
        &jig_protocol_store::people_v17::CEILINGS,
        &mut Reader::new(&value.refusal),
    )
    .or(Err(w::paths::CORE_REFUSED_REFUSAL.problem(skein_codec::Reason::Malformed)))?;
    Ok(refusal::decode_refusal(record))
}

/// Owner-shaped `SeveralRepositories` (skein/docs/design/data/codec/maps.md, 3.3).
#[expect(clippy::borrowed_box, reason = "maps pairs take the domain type by reference")]
#[must_use]
pub fn several_repositories_to_record(value: &Box<[d::DraftRepository]>) -> w::SeveralRepositories {
    {
        let mut repositories = skein_lib::List::with_capacity(w::CEILINGS.several_repositories_repositories);
        for named in value {
            repositories.push(crate::generated::store::open_repository_to_record(named)).expect(FITS);
        }
        w::SeveralRepositories { repositories }
    }
}
pub fn several_repositories_to_domain(value: w::SeveralRepositories) -> Result<Box<[d::DraftRepository]>, w::Problem> {
    {
        let mut repositories = skein_lib::List::with_capacity(w::CEILINGS.several_repositories_repositories);
        for named in value.repositories.into_boxed() {
            repositories.push(crate::generated::store::open_repository_to_domain(named)).expect(FITS);
        }
        Ok::<_, w::Problem>(repositories.into_boxed())
    }
    .or(Err(w::paths::SEVERAL_REPOSITORIES_REPOSITORIES.problem(skein_codec::Reason::Malformed)))
}
