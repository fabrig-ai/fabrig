//! Milestone task queries by the offered template label (client/ctl/milestones.md, 4).
use crate::jig::call::value;
use jig_client::contract as c;
pub(super) fn list(call: &crate::jig::call::Call, me: &c::Me) -> Option<c::Read> {
    let project = super::reads::project(call, me)?;
    let template = jig_client::shapes::template(me, project, 8)?;
    let ended = value(call, 1).is_some();
    let after = crate::jig::call::cursor(call, 2);
    Some(c::Read::Tasks {
        query: c::TaskQuery {
            project: Some(project),
            requester: None,
            root: None,
            parent: None,
            label: Some(template.label),
            phase: if ended { Some(c::PhaseClass::Ended) } else { None },
            executor: None,
            tracked: Some(true),
            order: c::QueryOrder::Number,
        },
        live: if ended { None } else { after.clone() },
        ended: if ended { after } else { None },
    })
}
