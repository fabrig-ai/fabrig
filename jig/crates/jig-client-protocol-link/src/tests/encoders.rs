#![expect(clippy::trivially_copy_pass_by_ref, reason = "fixture translations borrow source facts uniformly")]
//! Wire fixtures for link translation tests (protocol/client/wire.md, 1).
use crate::Limits;
use crate::translate::settings::*;
use crate::translate::too_large;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Writer};

pub fn encode_revision_item_pieces(
    values: &List<c::RevisionItem>,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    if values.len() > 1024 {
        return Err(too_large());
    }
    let mut rows = List::with_capacity(values.len());
    let mut size = 0_u32;
    for value in values {
        let row = wire_revision_item(value, limits)?;
        size = size
            .checked_add(row.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if size > limits.revision_document_items {
            return Err(too_large());
        }
        rows.push(row).expect("same capacity");
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(size));
    for row in rows.into_boxed() {
        writer
            .put(&row.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture").to_be_bytes())
            .expect("measured");
        row.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured");
    }
    Ok(writer.finish())
}
pub fn restore_read_revision(value: &w::ReadRevision) -> c::ReadRevision {
    c::ReadRevision { number: value.number }
}
pub fn restore_read_revision_answer(
    value: &w::ReadRevisionAnswer,
    limits: &Limits,
) -> Result<c::ReadRevisionAnswer, w::Problem> {
    Ok(c::ReadRevisionAnswer { value: restore_revision_answer(&value.value, limits)? })
}
pub fn restore_read_revisions(value: &w::ReadRevisions) -> c::ReadRevisions {
    c::ReadRevisions {
        before: match &value.before {
            Some(item) => Some(*item),
            None => None,
        },
    }
}
#[must_use]
pub fn restore_read_setting(value: &w::ReadSetting) -> c::ReadSetting {
    c::ReadSetting {
        setting: restore_setting(&value.setting),
        revision: value.revision,
        proposal: match &value.proposal {
            Some(proposal) => Some(crate::translate::answers::proposal_key(proposal)),
            None => None,
        },
    }
}
#[must_use]
pub fn wire_held_revision(value: &c::HeldRevision) -> w::HeldRevision {
    w::HeldRevision { head: wire_revision_head(&value.head), holders: value.holders }
}
#[must_use]
pub fn wire_malformed_settings(value: &c::MalformedSettings) -> w::MalformedSettings {
    w::MalformedSettings {
        setting: wire_setting(&value.setting),
        fault: wire_setting_fault(&value.fault),
        detail: None,
    }
}
pub fn wire_outcome_settings_changed(value: &c::OutcomeSettingsChanged) -> w::OutcomeSettingsChanged {
    w::OutcomeSettingsChanged { revision: value.revision }
}
pub fn wire_read_revision_answer(
    value: &c::ReadRevisionAnswer,
    limits: &w::Limits,
) -> Result<w::ReadRevisionAnswer, w::Problem> {
    Ok(w::ReadRevisionAnswer { value: wire_revision_answer(&value.value, limits)? })
}
pub fn wire_refusal_full(value: &c::RefusalFull) -> w::RefusalFull {
    let mut tasks = List::with_capacity(value.tasks.len());
    for item in &value.tasks {
        tasks.push(*item).expect("same capacity");
    }
    w::RefusalFull { revision: value.revision, tasks, more: value.more }
}
pub fn wire_refusal_stale(value: &c::RefusalStale) -> w::RefusalStale {
    w::RefusalStale { newest: value.newest }
}
pub fn wire_revision_answer(value: &c::RevisionAnswer, limits: &w::Limits) -> Result<w::RevisionAnswer, w::Problem> {
    Ok(match value {
        c::RevisionAnswer::Revision(item) => w::RevisionAnswer::Revision(wire_revision_document(item, limits)?),
        c::RevisionAnswer::Forgotten(item) => w::RevisionAnswer::Forgotten(wire_revision_forgotten(item)),
        c::RevisionAnswer::Unknown => w::RevisionAnswer::Unknown,
    })
}
pub fn wire_revision_decider(value: &c::RevisionDecider) -> w::RevisionDecider {
    match value {
        c::RevisionDecider::FirstStart => w::RevisionDecider::FirstStart,
        c::RevisionDecider::Release => w::RevisionDecider::Release,
        c::RevisionDecider::List { provider } => w::RevisionDecider::List(w::RevisionList { provider: *provider }),
        c::RevisionDecider::Party(item) => w::RevisionDecider::Party(wire_revision_party(item)),
    }
}
pub fn wire_revision_document(
    value: &c::RevisionDocument,
    limits: &w::Limits,
) -> Result<w::RevisionDocument, w::Problem> {
    Ok(w::RevisionDocument {
        head: wire_revision_head(&value.head),
        items: encode_revision_item_pieces(&value.items, limits)?,
    })
}
pub fn wire_revision_edit(value: &c::RevisionEdit, limits: &w::Limits) -> Result<w::RevisionEdit, w::Problem> {
    Ok(match value {
        c::RevisionEdit::Set(item) => w::RevisionEdit::Set(wire_setting_set(item, limits)?),
        c::RevisionEdit::TooLarge(item) => w::RevisionEdit::TooLarge(wire_size(item)),
        c::RevisionEdit::Application(v) => {
            w::RevisionEdit::Application(w::ApplicationEdit { edit: wire_sized_edit(v) })
        }
        c::RevisionEdit::Removed => w::RevisionEdit::Removed,
        c::RevisionEdit::Reset => w::RevisionEdit::Reset,
    })
}
pub fn wire_revision_forgotten(value: &c::RevisionForgotten) -> w::RevisionForgotten {
    w::RevisionForgotten { oldest: value.oldest }
}
#[must_use]
pub fn wire_revision_head(value: &c::RevisionHead) -> w::RevisionHead {
    w::RevisionHead {
        number: value.number,
        against: value.against,
        at: value.at.as_nanos(),
        by: wire_revision_decider(&value.by),
        proposal: match &value.proposal {
            Some(item) => Some(crate::translate::asks::wire_proposal_key(item)),
            None => None,
        },
        moved: value.moved,
    }
}
pub fn wire_revision_item(value: &c::RevisionItem, limits: &w::Limits) -> Result<w::RevisionItem, w::Problem> {
    Ok(w::RevisionItem {
        setting: wire_setting(&value.setting),
        edit: wire_revision_edit(&value.edit, limits)?,
        replaced: wire_revision_replaced(&value.replaced, limits)?,
    })
}
pub fn wire_revision_party(value: &c::RevisionParty) -> w::RevisionParty {
    w::RevisionParty { number: value.number }
}
pub fn wire_revision_replaced(
    value: &c::RevisionReplaced,
    limits: &w::Limits,
) -> Result<w::RevisionReplaced, w::Problem> {
    Ok(match value {
        c::RevisionReplaced::Value(v) => w::RevisionReplaced::Value(wire_setting_set(v, limits)?),
        c::RevisionReplaced::TooLarge(v) => w::RevisionReplaced::TooLarge(wire_size(v)),
        c::RevisionReplaced::Default => w::RevisionReplaced::Default,
        c::RevisionReplaced::Removed => w::RevisionReplaced::Removed,
        c::RevisionReplaced::NotOffered => w::RevisionReplaced::NotOffered,
        c::RevisionReplaced::Application { size } => {
            w::RevisionReplaced::Application(w::ApplicationSize { size: *size })
        }
    })
}
#[must_use]
pub fn wire_revision_row(value: &c::RevisionRow) -> w::RevisionRow {
    let mut items = List::with_capacity(value.items.len());
    for item in &value.items {
        items.push(wire_revision_item_head(item)).expect("same capacity");
    }
    w::RevisionRow { head: wire_revision_head(&value.head), items, more: value.more }
}
#[must_use]
pub fn wire_revisions_page(value: &c::RevisionsPage) -> w::RevisionsPage {
    let mut rows = List::with_capacity(value.rows.len());
    for item in &value.rows {
        rows.push(wire_revision_row(item)).expect("same capacity");
    }
    w::RevisionsPage {
        rows,
        next: match &value.next {
            Some(item) => Some(*item),
            None => None,
        },
        forgotten: value.forgotten,
    }
}
pub fn wire_setting_document(value: &c::SettingDocument, limits: &w::Limits) -> Result<w::SettingDocument, w::Problem> {
    Ok(w::SettingDocument {
        overlay: match &value.overlay {
            Some(item) => Some(wire_overlay(item)),
            None => None,
        },
        setting: wire_setting(&value.setting),
        revision: value.revision,
        value: match &value.value {
            Some(v) => Some(wire_setting_value(v, limits)?),
            None => None,
        },
        application: match &value.application {
            Some(v) => Some(wire_application_state(v)),
            None => None,
        },
        proposed: match &value.proposed {
            Some(v) => Some(wire_sized_edit(v)),
            None => None,
        },
    })
}
pub fn wire_setting_fault(value: &c::SettingFault) -> w::SettingFault {
    match value {
        c::SettingFault::Shape => w::SettingFault::Shape,
        c::SettingFault::Count => w::SettingFault::Count,
        c::SettingFault::Reference => w::SettingFault::Reference,
        c::SettingFault::Rules => w::SettingFault::Rules,
        c::SettingFault::Budget => w::SettingFault::Budget,
        c::SettingFault::Grace => w::SettingFault::Grace,
        c::SettingFault::Owner => w::SettingFault::Owner,
        c::SettingFault::Judged => w::SettingFault::Judged,
        c::SettingFault::Linked => w::SettingFault::Linked,
        c::SettingFault::Accounts => w::SettingFault::Accounts,
        c::SettingFault::Range => w::SettingFault::Range,
        c::SettingFault::Name => w::SettingFault::Name,
    }
}
pub fn wire_setting_follows(value: &c::SettingFollows) -> w::SettingFollows {
    match value {
        c::SettingFollows::Default => w::SettingFollows::Default,
        c::SettingFollows::Overlay => w::SettingFollows::Overlay,
        c::SettingFollows::Change => w::SettingFollows::Change,
    }
}
#[must_use]
pub fn wire_setting_line(value: &c::SettingLine) -> w::SettingLine {
    w::SettingLine {
        name: None,
        setting: wire_setting(&value.setting),
        shown: match &value.shown {
            Some(item) => Some(wire_document_text(item)),
            None => None,
        },
        follows: wire_setting_follows(&value.follows),
        last: value.last,
        application: match &value.application {
            Some(v) => Some(wire_application_state(v)),
            None => None,
        },
    }
}
#[must_use]
pub fn wire_settings_document(value: &c::SettingsDocument) -> w::SettingsDocument {
    let mut settings = List::with_capacity(value.settings.len());
    for item in &value.settings {
        settings.push(wire_setting_line(item)).expect("same capacity");
    }
    let mut held = List::with_capacity(value.held.len());
    for item in &value.held {
        held.push(wire_held_revision(item)).expect("same capacity");
    }
    let mut pending = List::with_capacity(value.pending.len());
    for item in &value.pending {
        pending.push(wire_settings_proposal(item)).expect("same capacity");
    }
    w::SettingsDocument { newest: value.newest, settings, held, pending }
}
pub fn wire_settings_proposal(value: &c::SettingsProposal) -> w::SettingsProposal {
    w::SettingsProposal {
        proposer: crate::generated::client::proposer_to_record(&value.proposer),
        number: value.number,
    }
}
#[must_use]
pub fn wire_revision_item_head(value: &c::RevisionItemHead) -> w::RevisionItemHead {
    w::RevisionItemHead { setting: wire_setting(&value.setting), edit: wire_revision_item_edit(&value.edit) }
}
pub fn wire_revision_item_edit(value: &c::RevisionItemEdit) -> w::RevisionItemEdit {
    match value {
        c::RevisionItemEdit::Set => w::RevisionItemEdit::Set,
        c::RevisionItemEdit::Removed => w::RevisionItemEdit::Removed,
        c::RevisionItemEdit::Reset => w::RevisionItemEdit::Reset,
    }
}
pub fn wire_application_state(v: &c::ApplicationState) -> w::ApplicationState {
    let follows = match v.follows {
        c::ApplicationFollows::Default => w::ApplicationFollows::Default,
        c::ApplicationFollows::Changed { revision } => {
            w::ApplicationFollows::Changed(w::ApplicationRevision { revision })
        }
        c::ApplicationFollows::Removed { revision } => {
            w::ApplicationFollows::Removed(w::ApplicationRevision { revision })
        }
    };
    w::ApplicationState { follows, size: v.size, charters: v.charters.clone() }
}
pub fn wire_sized_edit(v: &c::SizedEdit) -> w::SizedEdit {
    match v {
        c::SizedEdit::Set { size } => w::SizedEdit::Set(w::ApplicationSize { size: *size }),
        c::SizedEdit::Remove => w::SizedEdit::Remove,
        c::SizedEdit::Reset => w::SizedEdit::Reset,
    }
}
pub fn encode_heads(values: &List<c::RevisionItemHead>, bound: u32) -> Result<List<w::RevisionItemHead>, w::Problem> {
    if values.len() > bound {
        return Err(too_large());
    }
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(wire_revision_item_head(value)).expect("same capacity");
    }
    Ok(rows)
}
pub fn encode_review_pieces(values: &List<c::ReviewItem>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    if values.len() > 128 {
        return Err(too_large());
    }
    let mut rows = List::with_capacity(values.len());
    let mut size = 0_u32;
    for value in values {
        let row = wire_review_item(value, limits)?;
        size = size
            .checked_add(row.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if size > limits.settings_review_items {
            return Err(too_large());
        }
        rows.push(row).expect("same capacity");
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(size));
    for row in rows.into_boxed() {
        writer
            .put(&row.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture").to_be_bytes())
            .expect("measured");
        row.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured");
    }
    Ok(writer.finish())
}
pub fn wire_origin_revision(value: &c::OriginRevision) -> w::OriginRevision {
    w::OriginRevision { revision: value.revision }
}
pub fn wire_owner_proposal_action_settings(
    value: &c::ProposalActionSettings,
    limits: &w::Limits,
) -> Result<w::ProposalActionSettings, w::Problem> {
    Ok(w::ProposalActionSettings {
        against: value.against,
        items: encode_heads(&value.items, limits.proposal_action_settings_items)?,
        review: match &value.review {
            Some(item) => Some(wire_settings_review(item, limits)?),
            None => None,
        },
        made: value.made,
    })
}
pub fn wire_owner_retained_action_settings(
    value: &c::RetainedActionSettings,
    limits: &w::Limits,
) -> Result<w::RetainedActionSettings, w::Problem> {
    Ok(w::RetainedActionSettings {
        projects: value.projects.clone(),
        against: value.against,
        items: encode_heads(&value.items, limits.retained_action_settings_items)?,
        made: value.made,
    })
}
pub fn wire_review_item(value: &c::ReviewItem, limits: &w::Limits) -> Result<w::ReviewItem, w::Problem> {
    Ok(w::ReviewItem {
        setting: wire_setting(&value.setting),
        edit: wire_revision_item_edit(&value.edit),
        now: wire_review_value(&value.now, limits)?,
        origin: wire_setting_origin(&value.origin),
        would_be: wire_review_value(&value.would_be, limits)?,
        application: match &value.application {
            Some(v) => Some(w::ApplicationReview {
                edit: wire_sized_edit(&v.edit),
                now: match &v.now {
                    Some(v) => Some(wire_application_state(v)),
                    None => None,
                },
            }),
            None => None,
        },
    })
}
pub fn wire_review_value(value: &c::ReviewValue, limits: &w::Limits) -> Result<w::ReviewValue, w::Problem> {
    Ok(match value {
        c::ReviewValue::Value(item) => w::ReviewValue::Value(wire_setting_set(item, limits)?),
        c::ReviewValue::TooLarge(item) => w::ReviewValue::TooLarge(wire_size(item)),
        c::ReviewValue::None => w::ReviewValue::None,
    })
}
pub fn wire_setting_origin(value: &c::SettingOrigin) -> w::SettingOrigin {
    match value {
        c::SettingOrigin::Default(item) => w::SettingOrigin::Default(wire_origin_revision(item)),
        c::SettingOrigin::Change(item) => w::SettingOrigin::Change(wire_origin_revision(item)),
        c::SettingOrigin::NotOffered => w::SettingOrigin::NotOffered,
    }
}
pub fn wire_settings_review(value: &c::SettingsReview, limits: &w::Limits) -> Result<w::SettingsReview, w::Problem> {
    Ok(w::SettingsReview {
        key: value.key,
        newest: value.newest,
        items: encode_review_pieces(&value.items, limits)?,
        cut: value.cut,
    })
}
#[must_use]
pub fn wire_outcome_restored(proposal: u64, to: u32, left_out: &List<c::Setting>) -> w::OutcomeRestored {
    let mut settings = List::with_capacity(left_out.len());
    for setting in left_out {
        settings.push(wire_setting(setting)).expect("same capacity");
    }
    w::OutcomeRestored { proposal, to, left_out: settings }
}
pub fn wire_refusal_forgotten(oldest: u32) -> w::RefusalForgotten {
    w::RefusalForgotten { oldest }
}
pub fn wire_size(value: &c::Size) -> w::Size {
    w::Size { bytes: value.bytes }
}
