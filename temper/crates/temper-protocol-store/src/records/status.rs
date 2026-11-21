//! Status families (temper/docs/design/protocol/store.md, 2).
use super::w;

use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_facts(row: &d::ProjectFacts) -> w::ProjectFacts {
    crate::generated::store::project_facts_to_record(row, &super::Context::UNBOUNDED).expect(super::FITS)
}
pub(super) fn decode_facts(row: w::ProjectFacts) -> d::ProjectFacts {
    crate::generated::store::project_facts_to_domain(row, &super::Context::UNBOUNDED).expect(super::FITS)
}

pub(super) fn encode_person_brief(row: &d::PersonBrief) -> w::PersonBrief {
    crate::generated::store::person_brief_to_record(row, &super::Context::UNBOUNDED).expect(super::FITS)
}

pub(super) fn decode_person_brief(row: w::PersonBrief) -> d::PersonBrief {
    crate::generated::store::person_brief_to_domain(row, &super::Context::UNBOUNDED).expect(super::FITS)
}

pub(super) fn encode_looked(row: &d::Looked) -> w::Looked {
    crate::generated::store::looked_to_record(row, &super::Context::UNBOUNDED).expect(super::FITS)
}

pub(super) fn decode_looked(row: w::Looked) -> d::Looked {
    crate::generated::store::looked_to_domain(row, &super::Context::UNBOUNDED).expect(super::FITS)
}
