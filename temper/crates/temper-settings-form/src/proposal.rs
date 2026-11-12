//! Compact proposal reads beside revisions (temper/docs/design/domain/agent/ask-temper.md, 1).
use crate::{Limits, Text, render, view, wire};
use alloc::boxed::Box;
use jig_client_protocol_wire::v19 as w;
use skein_lib::{List, Reader};

#[cfg(test)]
mod tests;

/// Render a waiting settings proposal whole; `see` cuts its parts at the host boundary.
pub fn proposal_text(value: &w::Proposal, limits: &Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut measured = Text::measure();
    waiting(value, limits, &mut measured)?;
    let mut out = Text::writing_exact(measured);
    waiting(value, limits, &mut out)?;
    Ok(out.finish())
}

/// Render the retained outcome and item heads of a decided settings proposal.
pub fn ended_proposal_text(value: &w::Ended, limits: &Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut measured = Text::measure();
    decided(value, limits, &mut measured)?;
    let mut out = Text::writing_exact(measured);
    decided(value, limits, &mut out)?;
    Ok(out.finish())
}

fn waiting(value: &w::Proposal, limits: &Limits, out: &mut Text) -> Result<(), w::Problem> {
    let settings = match value.action() {
        w::ProposalAction::Settings(settings) => settings,
        w::ProposalAction::Batch(_)
        | w::ProposalAction::Effect(_)
        | w::ProposalAction::ProcedureEffect(_)
        | w::ProposalAction::Amend(_)
        | w::ProposalAction::Widen(_)
        | w::ProposalAction::Release(_) => return Err(wire::root_too_large()),
    };
    out.put(b"proposal ");
    render::proposal_text(wire::answers_proposal_key(value.key(), limits), out);
    out.put(b" of settings, waiting, against revision ");
    out.number(u64::from(settings.against()));
    out.put(b"\n");
    reason(value.reason(), out);
    items(settings.items(), limits, out);
    if let Some(review) = settings.review() {
        out.put(b"review at revision ");
        out.number(u64::from(review.newest()));
        out.put(b":\n");
        let count = wire::settings_item_count(review.items(), limits)?;
        let mut reader = Reader::new(review.items());
        for _ in 0..count {
            let piece = match skein_codec::piece::split(&mut reader) {
                Ok(Some(piece)) => piece,
                Ok(None) => unreachable!("counted review item"),
                Err(why) => return Err(wire::root_piece_problem(why)),
            };
            let item = w::ReviewItem::decode(&limits.settings, &mut Reader::new(&piece))?;
            out.put(b"  ");
            let setting = wire::settings_restore_setting(item.setting(), limits);
            render::setting_text(setting, out);
            out.put(b"\n    now:\n");
            match item.application() {
                Some(application) => application_review(application, out),
                None => {
                    review_value(setting, item.now(), limits, out)?;
                    out.put(b"    would be:\n");
                    review_value(setting, item.would_be(), limits, out)?;
                }
            }
        }
        if review.cut() {
            out.put(b"cut: more not shown\n");
        }
    }
    Ok(())
}

fn decided(value: &w::Ended, limits: &Limits, out: &mut Text) -> Result<(), w::Problem> {
    let history = match value.value() {
        w::EndedValue::Proposal(value) => value.value(),
        w::EndedValue::Task(_)
        | w::EndedValue::PersonProposal(_)
        | w::EndedValue::Escalation(_)
        | w::EndedValue::Question(_)
        | w::EndedValue::PersonTask(_) => return Err(wire::root_too_large()),
    };
    let settings = match history.action() {
        w::RetainedAction::Settings(settings) => settings,
        w::RetainedAction::Batch(_)
        | w::RetainedAction::Effect(_)
        | w::RetainedAction::ProcedureEffect(_)
        | w::RetainedAction::Amend(_)
        | w::RetainedAction::Widen(_)
        | w::RetainedAction::Release(_) => return Err(wire::root_too_large()),
    };
    let key = match value.key() {
        w::ObjectKey::Proposal(value) => value.value(),
        w::ObjectKey::Task(_)
        | w::ObjectKey::Escalation(_)
        | w::ObjectKey::Question(_)
        | w::ObjectKey::PersonTask(_)
        | w::ObjectKey::Result(_)
        | w::ObjectKey::Note(_) => return Err(wire::root_too_large()),
    };
    out.put(b"proposal ");
    render::proposal_text(wire::answers_proposal_key(key, limits), out);
    out.put(b" of settings, ");
    match history.state() {
        w::ProposalHistoryState::Accepted(_) => {
            out.put(b"accepted as revision ");
            out.number(u64::from(settings.made().ok_or(wire::root_too_large())?));
        }
        w::ProposalHistoryState::Rejected(value) => {
            out.put(b"rejected: ");
            out.put(value.reason().bytes());
        }
        w::ProposalHistoryState::Withdrawn => out.put(b"withdrawn"),
        w::ProposalHistoryState::Pending(_) => return Err(wire::root_too_large()),
    }
    out.put(b"\n");
    reason(history.reason(), out);
    items(settings.items(), limits, out);
    Ok(())
}

fn reason(value: &w::DocumentText, out: &mut Text) {
    out.put(b"reason:\n");
    let mut from = 0;
    for (at, byte) in value.bytes().iter().chain(core::iter::once(&b'\n')).enumerate() {
        if *byte != b'\n' {
            continue;
        }
        out.put(b"  ");
        out.put(value.bytes().get(from..at).expect("reason line within document"));
        out.put(b"\n");
        from = at.checked_add(1).expect("document bound");
    }
}

fn items(values: &List<w::RevisionItemHead>, limits: &Limits, out: &mut Text) {
    out.put(b"items:\n");
    for item in values {
        let item = wire::settings_restore_revision_item_head(item, limits);
        out.put(b"  ");
        render::setting_text(item.setting, out);
        out.put(b": ");
        out.put(render::edit_word(&item.edit));
        out.put(b"\n");
    }
}

fn review_value(
    setting: view::Setting,
    value: &w::ReviewValue,
    limits: &Limits,
    out: &mut Text,
) -> Result<(), w::Problem> {
    out.put(b"      ");
    match value {
        w::ReviewValue::Value(value) => {
            let value = wire::settings_restore_setting_value(value.value(), limits)?;
            match setting {
                view::Setting::Application { kind, .. } => application_value(kind, &value, limits, out),
                view::Setting::Charter(_)
                | view::Setting::Template(_)
                | view::Setting::Model(_)
                | view::Setting::Policy(_)
                | view::Setting::ProjectPolicy { .. }
                | view::Setting::Rules => render::value_inline(&value, limits, out),
            }
        }
        w::ReviewValue::TooLarge(size) => {
            out.put(b"too large, ");
            out.number(size.bytes());
            out.put(b" bytes");
        }
        w::ReviewValue::None => out.put(b"none"),
    }
    out.put(b"\n");
    Ok(())
}

fn application_review(value: &w::ApplicationReview, out: &mut Text) {
    out.put(b"      ");
    match value.now() {
        Some(now) => {
            out.number(u64::from(now.size()));
            out.put(b" bytes");
        }
        None => out.put(b"none"),
    }
    out.put(b"\n    would be:\n      ");
    match value.edit() {
        w::SizedEdit::Set(size) => {
            out.number(u64::from(size.size()));
            out.put(b" bytes");
        }
        w::SizedEdit::Remove => out.put(b"none"),
        w::SizedEdit::Reset => out.put(b"default"),
    }
    out.put(b"\n");
}

fn application_value(kind: u32, value: &view::SettingValue, limits: &Limits, out: &mut Text) {
    let bounds =
        skein_json::writer::Limits { depth: 32, length: u32::try_from(limits.result_bytes).expect("result bound") };
    let mut measure = skein_json::writer::Encoder::measure(&bounds);
    encode_application_value(kind, value, &mut measure);
    let mut write = skein_json::writer::Encoder::write(measure.measured().expect("received value fits"), &bounds);
    encode_application_value(kind, value, &mut write);
    out.put(&write.finish());
}

fn encode_application_value(kind: u32, value: &view::SettingValue, out: &mut skein_json::writer::Encoder) {
    match value {
        view::SettingValue::Application(bytes) => {
            if crate::kinds::render(kind, bytes, out) {
                return;
            }
        }
        view::SettingValue::Overlay
        | view::SettingValue::Model(_)
        | view::SettingValue::Charter(_)
        | view::SettingValue::Template(_)
        | view::SettingValue::Policy(_)
        | view::SettingValue::Rules(_) => {}
    }
    render::encode_value(value, out);
}
