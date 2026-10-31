#![expect(clippy::manual_map, reason = "step code carries optional owner facts without closures")]
//! Original history revisions, snapshots and lifecycle facts (protocol/client.md, 3).
use super::{objects, owner, w};
use jig_domain_core::documents as d;
use jig_domain_tasks as t;
pub fn line(models: &[crate::Model], value: &d::HistoryLine, limits: &w::Limits) -> Result<w::HistoryLine, w::Problem> {
    let change = match value.change {
        t::Change::Cancelled => w::HistoryChange::Cancelled,
        t::Change::Released => w::HistoryChange::Released,
        t::Change::Amended => w::HistoryChange::Amended,
        t::Change::Moved => w::HistoryChange::Moved,
        t::Change::Proposed => w::HistoryChange::Proposed,
        t::Change::ProposalPassed => w::HistoryChange::ProposalPassed,
        t::Change::ProposalAccepted => w::HistoryChange::ProposalAccepted,
        t::Change::ProposalRejected => w::HistoryChange::ProposalRejected,
        t::Change::ProposalWithdrawn => w::HistoryChange::ProposalWithdrawn,
    };
    w::HistoryLine::new(
        limits,
        w::HistoryLineParts {
            task: value.task,
            revision: value.revision,
            by: owner::requester(value.by, limits)?,
            change,
            reason: owner::text(&value.reason, limits)?,
            proposal: match &value.proposal {
                Some(item) => Some(objects::ended::proposal_history(models, item, limits)?),
                None => None,
            },
            amendment: match &value.amendment {
                Some((before, after)) => Some(w::HistoryAmendment::new(
                    limits,
                    w::HistoryAmendmentParts {
                        before: objects::proposals::amendment(models, before, limits)?,
                        after: objects::proposals::amendment(models, after, limits)?,
                    },
                )?),
                None => None,
            },
            at: match value.at {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        },
    )
}
