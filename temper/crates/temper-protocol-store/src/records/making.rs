//! Making rows (temper/docs/design/protocol/store.md, 2, and temper/docs/design/protocol/store-restart.md, 1).
use crate::v25 as w;

use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode(row: &d::RepositoryMaking) -> w::RepositoryMaking {
    crate::generated::store::repository_making_to_record(row, &super::Context::UNBOUNDED).expect(super::FITS)
}

pub(super) fn decode(row: w::RepositoryMaking) -> d::RepositoryMaking {
    crate::generated::store::repository_making_to_domain(row, &super::Context::UNBOUNDED).expect(super::FITS)
}
