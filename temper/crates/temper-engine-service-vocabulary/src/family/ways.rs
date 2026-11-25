//! Paths, watched ways and read views (temper/docs/design/client/forge/records.md;
//! temper/docs/design/client/forge/served.md).
use super::{Context, maps};
use alloc::boxed::Box;
use skein_codec::Reason;
use skein_lib::{List, usize_of};
use temper_client_protocol_wire::v1 as w;
use temper_client_protocol_wire::v1::Record;
use temper_engine_domain::ways;

/// Encode the project's ways, refusing a document beyond any wire bound.
#[must_use]
pub fn ways_panel(document: ways::WaysDocument, limits: &w::Limits) -> Option<Box<[u8]>> {
    let panel = maps::ways_panel_to_record(&document).ok()?;
    drop(document);
    panel.encode(limits).ok()
}

pub(super) fn view_answer(view: &ways::View, limits: &w::Limits) -> Option<Box<[u8]>> {
    let view = maps::view_to_record(view, &Context::Sending(limits)).ok()?;
    let answer = w::ViewAnswer { view };
    answer.encode(limits).ok()
}

// A role is a flattened WayRole and its running flag.
pub(crate) fn way_role_to_record(entry: &ways::WayRoleEntry) -> w::WayRole {
    w::WayRole {
        charter: entry.role.charter,
        words: entry.role.words.clone(),
        budget: entry.role.budget,
        running: entry.running,
    }
}

// The settings codec owns the bytes and the ways domain has no Reset edit.
pub(crate) fn proposal_item_to_record(item: &ways::Item) -> w::ProposalItem {
    let edit = match &item.edit {
        ways::Edit::Set(value) => {
            w::SettingEdit::Set(w::SettingValue { value: temper_engine_smith::kinds::encode(value) })
        }
        ways::Edit::Remove => w::SettingEdit::Removed,
    };
    w::ProposalItem { kind: item.kind, scope: item.scope, number: item.number, edit }
}

// The domain keeps each charter and its card as a pair, not a record.
pub(crate) fn roles_view_to_record(
    values: &[(u32, ways::Card)],
    context: &Context<'_>,
) -> Result<w::RolesView, w::Problem> {
    let capacity = context.limits().roles_view_roles;
    if values.len() > usize_of(capacity) {
        return Err(w::paths::ROLES_VIEW_ROLES.problem(Reason::Bound));
    }
    let mut roles = List::with_capacity(capacity);
    for (charter, value) in values {
        let card = maps::card_to_record(value)?;
        roles.push(w::CharterView { charter: *charter, card }).expect("checked roles count");
    }
    Ok(w::RolesView { roles })
}
