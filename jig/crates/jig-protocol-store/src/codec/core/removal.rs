//! Kept project deletion rows (temper/docs/design/draft/playground-reset.md, 5).
use super::{Limits, domain, wire};

pub(super) fn encode(row: &domain::ProjectRemoval, limits: &Limits) -> Result<wire::ProjectRemoval, wire::Problem> {
    let stage = match row.stage {
        domain::RemovalStage::Ending => wire::RemovalStage::Ending,
        domain::RemovalStage::Erasing { after } => {
            wire::RemovalStage::Erasing(wire::RemovalErasing::new(&limits.core, wire::RemovalErasingParts { after })?)
        }
        domain::RemovalStage::Notes => wire::RemovalStage::Notes,
        domain::RemovalStage::Rows => wire::RemovalStage::Rows,
    };
    wire::ProjectRemoval::new(&limits.core, wire::ProjectRemovalParts { project: row.project, stage })
}

pub(super) fn decode(row: wire::ProjectRemoval) -> domain::ProjectRemoval {
    let wire::ProjectRemovalParts { project, stage } = row.into_parts();
    let stage = match stage {
        wire::RemovalStage::Ending => domain::RemovalStage::Ending,
        wire::RemovalStage::Erasing(row) => {
            let wire::RemovalErasingParts { after } = row.into_parts();
            domain::RemovalStage::Erasing { after }
        }
        wire::RemovalStage::Notes => domain::RemovalStage::Notes,
        wire::RemovalStage::Rows => domain::RemovalStage::Rows,
    };
    domain::ProjectRemoval { project, stage }
}
