//! Settings proposal and review facts (client/domain/contract/requests.md, 2, and client/domain/contract/documents.md, 1).
use super::settings::{
    restore_revision_item_edit, restore_revision_item_head, restore_setting, restore_setting_item, restore_setting_set,
    restore_size, wire_setting_item,
};
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v22 as w;
use skein_lib::{List, Reader, Writer};
pub fn restore_origin_revision(value: &w::OriginRevision) -> Result<c::OriginRevision, w::Problem> {
    Ok(c::OriginRevision { revision: value.revision() })
}
pub fn wire_propose_settings(value: &c::ProposeSettings, limits: &w::Limits) -> Result<w::ProposeSettings, w::Problem> {
    w::ProposeSettings::new(
        limits,
        w::ProposeSettingsParts { against: value.against, items: encode_setting_pieces(&value.items, limits)? },
    )
}
pub fn restore_propose_settings(value: &w::ProposeSettings, limits: &Limits) -> Result<c::ProposeSettings, w::Problem> {
    Ok(c::ProposeSettings { against: value.against(), items: decode_setting_pieces(value.items(), limits)? })
}
pub fn restore_review_item(value: &w::ReviewItem, limits: &Limits) -> Result<c::ReviewItem, w::Problem> {
    Ok(c::ReviewItem {
        setting: restore_setting(value.setting())?,
        edit: restore_revision_item_edit(value.edit())?,
        now: restore_review_value(value.now(), limits)?,
        origin: restore_setting_origin(value.origin())?,
        would_be: restore_review_value(value.would_be(), limits)?,
        application: match value.application() {
            Some(v) => Some(c::ApplicationReview {
                edit: super::settings::restore_sized_edit(v.edit()),
                now: match v.now() {
                    Some(v) => Some(super::settings::restore_application_state(v)),
                    None => None,
                },
            }),
            None => None,
        },
    })
}
pub fn restore_settings_review(value: &w::SettingsReview, limits: &Limits) -> Result<c::SettingsReview, w::Problem> {
    Ok(c::SettingsReview {
        key: *value.key(),
        newest: value.newest(),
        items: decode_review_pieces(value.items(), limits)?,
        cut: value.cut(),
    })
}
pub fn restore_owner_proposal_action_settings(
    value: &w::ProposalActionSettings,
    limits: &Limits,
) -> Result<c::ProposalActionSettings, w::Problem> {
    Ok(c::ProposalActionSettings {
        against: value.against(),
        items: decode_heads(value.items(), limits.receiving().proposal_action_settings_items)?,
        review: match value.review() {
            Some(item) => Some(restore_settings_review(item, limits)?),
            None => None,
        },
        made: *value.made(),
    })
}
pub fn restore_owner_retained_action_settings(
    value: &w::RetainedActionSettings,
    limits: &Limits,
) -> Result<c::RetainedActionSettings, w::Problem> {
    Ok(c::RetainedActionSettings {
        projects: value.projects().clone(),
        against: value.against(),
        items: decode_heads(value.items(), limits.receiving().retained_action_settings_items)?,
        made: *value.made(),
    })
}
pub fn restore_setting_origin(value: &w::SettingOrigin) -> Result<c::SettingOrigin, w::Problem> {
    Ok(match value {
        w::SettingOrigin::Default(item) => c::SettingOrigin::Default(restore_origin_revision(item)?),
        w::SettingOrigin::Change(item) => c::SettingOrigin::Change(restore_origin_revision(item)?),
        w::SettingOrigin::NotOffered => c::SettingOrigin::NotOffered,
    })
}
pub fn restore_review_value(value: &w::ReviewValue, limits: &Limits) -> Result<c::ReviewValue, w::Problem> {
    Ok(match value {
        w::ReviewValue::Value(item) => c::ReviewValue::Value(restore_setting_set(item, limits)?),
        w::ReviewValue::TooLarge(item) => c::ReviewValue::TooLarge(restore_size(item)?),
        w::ReviewValue::None => c::ReviewValue::None,
    })
}
fn decode_heads(values: &List<w::RevisionItemHead>, bound: u32) -> Result<List<c::RevisionItemHead>, w::Problem> {
    if values.len() > bound {
        return Err(super::too_large());
    }
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(restore_revision_item_head(value)?).expect("same capacity");
    }
    Ok(rows)
}
fn encode_setting_pieces(values: &List<c::SettingItem>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    if values.len() > 128 {
        return Err(super::too_large());
    }
    let mut rows = List::with_capacity(values.len());
    let mut size = 0_u32;
    for value in values {
        let row = wire_setting_item(value, limits)?;
        size = size.checked_add(row.measure()).ok_or(super::too_large())?.checked_add(4).ok_or(super::too_large())?;
        if size > limits.propose_settings_items {
            return Err(super::too_large());
        }
        rows.push(row).expect("same capacity");
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(size));
    for row in rows.into_boxed() {
        writer.put(&row.measure().to_be_bytes()).expect("measured");
        row.encode(&mut writer).expect("measured");
    }
    Ok(writer.finish())
}
fn decode_setting_pieces(values: &[u8], limits: &Limits) -> Result<List<c::SettingItem>, w::Problem> {
    if values.len() > skein_lib::length::usize_of(limits.receiving().propose_settings_items) {
        return Err(super::too_large());
    }
    let count = piece_count(values)?;
    let mut reader = Reader::new(values);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let piece = super::piece(&mut reader)?.ok_or(super::too_large())?;
        let mut input = Reader::new(&piece);
        let row = w::SettingItem::decode(limits.receiving(), &mut input)?;
        if !input.is_empty() {
            return Err(super::too_large());
        }
        rows.push(restore_setting_item(&row, limits)?).expect("counted pieces");
    }
    Ok(rows)
}
fn decode_review_pieces(values: &[u8], limits: &Limits) -> Result<List<c::ReviewItem>, w::Problem> {
    if values.len() > skein_lib::length::usize_of(limits.receiving().settings_review_items) {
        return Err(super::too_large());
    }
    let count = piece_count(values)?;
    let mut reader = Reader::new(values);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let piece = super::piece(&mut reader)?.ok_or(super::too_large())?;
        let mut input = Reader::new(&piece);
        let row = w::ReviewItem::decode(limits.receiving(), &mut input)?;
        if !input.is_empty() {
            return Err(super::too_large());
        }
        rows.push(restore_review_item(&row, limits)?).expect("counted pieces");
    }
    Ok(rows)
}
fn piece_count(values: &[u8]) -> Result<u32, w::Problem> {
    let mut reader = Reader::new(values);
    for count in 0_u32..=128 {
        if super::piece(&mut reader)?.is_none() {
            return Ok(count);
        }
    }
    Err(super::too_large())
}
