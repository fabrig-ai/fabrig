//! Client settings views adapt to the neutral settings renderer.
use super::Text;
use crate::jig::render::Parts;
use crate::jig::{
    Limits,
    call::{Call, Project, Value, value as argument},
    rows, settings_adapter as a, settings_file,
};
use jig_client::contract as c;
use skein_json::writer::Encoder;
pub(crate) fn setting_text(setting: c::Setting, vocabulary: &dyn Parts, out: &mut Text) {
    if !vocabulary.setting_name(setting, out) {
        settings_form::render::setting_text(a::form_setting(setting), out);
    }
}

pub(super) fn encode_setting(setting: c::Setting, out: &mut Encoder) {
    settings_form::render::encode_setting(a::form_setting(setting), out);
}
pub(super) fn encode_value(value: &c::SettingValue, out: &mut Encoder) {
    settings_form::render::encode_value(&a::form_setting_value(value), out);
}
pub(crate) fn proposal_text(proposal: c::ProposalKey, out: &mut Text) {
    settings_form::render::proposal_text(a::form_proposal_key(&proposal), out);
}
pub(crate) fn text(value: &c::ReadValue, call: &Call, limits: &Limits, vocabulary: &dyn Parts, out: &mut Text) {
    if let c::ReadValue::Setting(document) = value
        && let Some(Project::Number(project)) = settings_file::preview_project(call)
        && let c::Setting::Charter(setting) = document.setting
        && let Some(c::SettingValue::Charter(charter)) = &document.value
    {
        let reason = match argument(call, usize::from(rows::CHARTER_PREVIEW_REASON)) {
            Some(Value::Word(index)) => 1_u32.checked_shl(u32::from(*index)),
            Some(_) | None => None,
        };
        preview_text(document, charter, setting.number, *project, reason, vocabulary.project_charter_directory(), out);
        return;
    }
    if let c::ReadValue::Settings(document) = value {
        inventory_text(document, vocabulary, out);
        return;
    }
    if let c::ReadValue::Setting(document) = value
        && let c::Setting::Application { .. } = document.setting
    {
        if let c::Setting::Application { kind: 11, .. } = document.setting
            && document.value.is_none()
        {
            settings_form::render::text(
                &a::form_read_value(value),
                crate::jig::call::number(call, 0),
                &a::limits(limits),
                out,
            );
            return;
        }
        setting_text(document.setting, vocabulary, out);
        out.put(b", revision ");
        out.number(u64::from(document.revision));
        out.put(b"\n");
        match &document.value {
            Some(value) => named_value_inline(document.setting, value, limits, vocabulary, out),
            None => out.put(b"none"),
        }
        out.put(b"\n");
        if call.tool != rows::SETTING_AT || settings_file::revision(call).is_none() {
            settings_form::render::stock_text(&a::form_read_value(value), out);
        }
        if let c::Setting::Application { kind: 9, .. } = document.setting
            && document.value.is_some()
            && let Some(state) = &document.application
            && state.follows == c::ApplicationFollows::Default
        {
            out.put(b"the stock\n");
        }
        if let Some(state) = &document.application {
            application_text(state, out);
        }
        return;
    }
    settings_form::render::text(&a::form_read_value(value), crate::jig::call::number(call, 0), &a::limits(limits), out);
}

// jig/docs/design/domain/engine/project-charters.md, 5 and 7: the returned
// deployment value is effective, with the overlay also provided separately.
fn preview_text(
    document: &c::SettingDocument,
    charter: &c::Charter,
    number: u32,
    project: u32,
    reason: Option<u32>,
    directory: &[u8],
    out: &mut Text,
) {
    let overlay = document.overlay.as_deref();
    out.put(b"charter ");
    if let Some(bound) = &document.project {
        out.put(&bound.name);
    } else {
        charter_identity(number, charter.name.as_deref(), out);
    }
    out.put(b" for project ");
    out.number(u64::from(project));
    out.put(b", as a new task would run by it\n");
    let (parts, source) = if let Some(bound) = &document.project {
        out.put(b"-- the project's ");
        out.put(directory);
        out.put(&bound.name);
        out.put(b".md, revision ");
        out.number(bound.revision);
        out.put(b", ");
        out.put(&bound.source);
        out.put(b", run by charter ");
        out.number(u64::from(number));
        out.put(b"'s settings");
        charter_words(bound.instructions.bytes(), out);
        (bound.parts.as_slice(), b"the project's".as_slice())
    } else {
        let Some((instructions, parts)) = base_charter_words(charter, overlay) else { return };
        out.put(b"-- charter ");
        charter_identity(number, charter.name.as_deref(), out);
        // Revision 1 holds the first start's defaults (jig/docs/design/domain/engine/settings-store.md, 1).
        let source = if overlay.is_some() || document.revision == 1 {
            out.put(b", the stock's");
            b"the stock's".as_slice()
        } else if number > settings_form::view::STOCK_NUMBERS {
            out.put(b", set whole");
            b"the charter set whole".as_slice()
        } else {
            b"the charter's own".as_slice()
        };
        charter_words(instructions, out);
        (parts, source)
    };
    if let Some(overlay) = overlay {
        out.put(b"-- the deployment's lines, charter ");
        out.number(u64::from(number));
        out.put(b"'s overlay");
        charter_words(overlay.instructions.bytes(), out);
    }
    preview_parts(parts, reason, source, out);
    if let Some(overlay) = overlay {
        preview_parts(overlay.parts.as_slice(), reason, b"the deployment's overlay", out);
    }
}

pub(in crate::jig::render) fn valid_preview(document: &c::SettingDocument) -> bool {
    let c::Setting::Charter(_) = document.setting else { return false };
    let Some(c::SettingValue::Charter(charter)) = &document.value else { return false };
    document.project.is_some() || base_charter_words(charter, document.overlay.as_deref()).is_some()
}

fn base_charter_words<'a>(
    charter: &'a c::Charter,
    overlay: Option<&c::Overlay>,
) -> Option<(&'a [u8], &'a [c::CharterPart])> {
    let instructions = charter.instructions.bytes();
    let parts = charter.parts.as_slice();
    match overlay {
        Some(overlay) => {
            settings_file::without_overlay(instructions, parts, overlay.instructions.bytes(), overlay.parts.as_slice())
        }
        None => Some((instructions, parts)),
    }
}

fn charter_identity(number: u32, name: Option<&[u8]>, out: &mut Text) {
    out.number(u64::from(number));
    if let Some(name) = name {
        out.put(b" ");
        out.put(name);
    }
}

fn charter_words(words: &[u8], out: &mut Text) {
    out.put(b": ");
    out.number(skein_lib::len_u64(words.len()));
    out.put(b" bytes\n");
    out.put(words);
    if !words.is_empty() && !words.ends_with(b"\n") {
        out.put(b"\n");
    }
}

fn preview_parts(parts: &[c::CharterPart], reason: Option<u32>, source: &[u8], out: &mut Text) {
    for part in parts {
        if let Some(reason) = reason
            && part.on & reason == 0
        {
            continue;
        }
        out.put(b"-- part on ");
        let mut bit = 1_u32;
        let mut first = true;
        for name in c::REASON_NAMES {
            if part.on & bit != 0 {
                if !first {
                    out.put(b", ");
                }
                out.put(name.as_bytes());
                first = false;
            }
            bit = bit.checked_shl(1).expect("known reasons");
        }
        out.put(b"; ");
        out.put(source);
        charter_words(part.instructions.bytes(), out);
    }
}

pub(super) fn provider_marks(value: &c::ReadValue, reading: &crate::jig::runner::reads::Reading, out: &mut Text) {
    if let c::ReadValue::Setting(document) = value
        && let c::Setting::Provider(number) = document.setting
    {
        if let Some(c::SettingValue::Provider(provider)) = &document.value
            && provider.withdrawn
        {
            out.put(b"withdrawn\n");
        }
        if reading.provider_defaults.contains(&number.number) {
            out.put(b"the stock's default\n");
        }
    }
}
pub(super) fn providers_text(
    reading: &crate::jig::runner::reads::Reading,
    limits: &Limits,
    vocabulary: &dyn Parts,
    out: &mut Text,
) {
    let mut any = false;
    for (_, page) in reading.pages.range(8..) {
        if let jig_client::Readout::Jig(c::ReadResult::Value(value @ c::ReadValue::Setting(_))) = page {
            text(value, &reading.called.call, limits, vocabulary, out);
            provider_marks(value, reading, out);
            any = true;
        }
    }
    if !any {
        out.put(b"no provider settings\n");
    }
}
pub(super) fn providers_encode(reading: &crate::jig::runner::reads::Reading, out: &mut Encoder) {
    out.object_start();
    out.key(b"providers");
    out.array_start();
    for (_, page) in reading.pages.range(8..) {
        if let jig_client::Readout::Jig(c::ReadResult::Value(value @ c::ReadValue::Setting(_))) = page {
            encode(value, out);
        }
    }
    out.array_end();
    out.key(b"stock");
    out.array_start();
    for number in &reading.provider_defaults {
        out.unsigned(u64::from(*number));
    }
    out.array_end();
    out.object_end();
}

pub(super) fn edit_word(edit: &c::RevisionItemEdit) -> &'static [u8] {
    settings_form::render::edit_word(&a::form_revision_item_edit(edit))
}
pub(in crate::jig::render) fn encode(value: &c::ReadValue, out: &mut Encoder) {
    settings_form::render::encode(&a::form_read_value(value), out);
}

pub(crate) fn fault_word(fault: c::SettingFault) -> &'static [u8] {
    settings_form::render::fault_word(a::form_setting_fault(fault))
}

fn inventory_text(document: &c::SettingsDocument, vocabulary: &dyn Parts, out: &mut Text) {
    out.put(b"newest revision ");
    out.number(u64::from(document.newest));
    out.put(b"\nsettings:\n");
    for line in &document.settings {
        out.put(b"  ");
        setting_text(line.setting, vocabulary, out);
        if let Some(name) = &line.name {
            out.put(b" ");
            out.put(name);
        }
        if let Some(shown) = &line.shown {
            out.put(b" \"");
            out.words(shown.bytes.bytes());
            out.put(b"\"");
        }
        out.put(b": ");
        match line.follows {
            c::SettingFollows::Overlay => out.put(b"overlay"),
            c::SettingFollows::Default => {
                out.put(b"default");
                if let Some(last) = line.last
                    && last != 0
                {
                    out.put(b", last changed in revision ");
                    out.number(u64::from(last));
                }
            }
            c::SettingFollows::Change => {
                out.put(b"changed in revision ");
                if let Some(last) = line.last {
                    out.number(u64::from(last));
                }
            }
        }
        if let Some(state) = &line.application
            && let c::ApplicationFollows::Removed { .. } = state.follows
        {
            out.put(b", removed");
        }
        out.put(b"\n");
    }
    out.put(b"held revisions:\n");
    for held in &document.held {
        out.put(b"  ");
        head_text(&held.head, out);
        out.put(b"    live tasks: ");
        out.number(u64::from(held.holders));
        out.put(b"\n");
    }
    out.put(b"pending proposals:\n");
    for p in &document.pending {
        out.put(b"  ");
        proposal_text(c::ProposalKey { proposer: p.proposer, number: u64::from(p.number) }, out);
        out.put(b"\n");
    }
}

fn head_text(head: &c::RevisionHead, out: &mut Text) {
    out.put(b"revision ");
    out.number(u64::from(head.number));
    out.put(b", against revision ");
    out.number(u64::from(head.against));
    out.put(b", at ");
    super::utc(head.at, out);
    out.put(b" UTC");
    out.put(b", by ");
    match &head.by {
        c::RevisionDecider::FirstStart => out.put(b"the first start"),
        c::RevisionDecider::Release => out.put(b"the release"),
        c::RevisionDecider::List { .. } => out.put(b"the provider's list"),
        c::RevisionDecider::Party(p) => {
            out.put(b"party ");
            out.number(p.number);
        }
    }
    if let Some(p) = head.proposal {
        out.put(b", from ");
        proposal_text(p, out);
    }
    out.put(if head.moved { b", running work moves\n" } else { b", running work stays\n" });
}
pub(super) fn encode_named_value(
    setting: c::Setting,
    value: &c::SettingValue,
    vocabulary: &dyn Parts,
    out: &mut Encoder,
) {
    if let c::Setting::Application { kind, .. } = setting
        && let c::SettingValue::Application(bytes) = value
        && vocabulary.setting_value(kind, bytes, out)
    {
        return;
    }
    encode_value(value, out);
}
pub(super) fn named_value_inline(
    setting: c::Setting,
    value: &c::SettingValue,
    limits: &Limits,
    vocabulary: &dyn Parts,
    out: &mut Text,
) {
    if let c::Setting::Application { kind, .. } = setting
        && let c::SettingValue::Application(bytes) = value
        && settings_form::render::application_value_text(kind, bytes, out)
    {
        return;
    }
    let bounds =
        skein_json::writer::Limits { depth: 32, length: u32::try_from(limits.result_bytes).expect("result bound") };
    let mut measure = Encoder::measure(&bounds);
    encode_named_value(setting, value, vocabulary, &mut measure);
    let mut write = Encoder::write(measure.measured().expect("received value fits"), &bounds);
    encode_named_value(setting, value, vocabulary, &mut write);
    out.put(&write.finish());
}
fn application_text(state: &c::ApplicationState, out: &mut Text) {
    match state.follows {
        c::ApplicationFollows::Default => out.put(b"default"),
        c::ApplicationFollows::Changed { revision } => {
            out.put(b"changed at revision ");
            out.number(u64::from(revision));
        }
        c::ApplicationFollows::Removed { revision } => {
            out.put(b"removed at revision ");
            out.number(u64::from(revision));
        }
    }
    out.put(b", ");
    out.number(u64::from(state.size));
    out.put(b" bytes\n");
}

pub(super) fn inventory_limits(reading: &crate::jig::runner::reads::Reading, vocabulary: &dyn Parts, out: &mut Text) {
    use crate::jig::runner::application::{Answer, Query};
    let mut values = skein_lib::List::uncapped();
    if let Some(jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(document)))) = reading.pages.get(&2)
        && let Some(value) = &document.value
    {
        values.push((a::form_setting(document.setting), a::form_setting_value(value))).expect("two limit kinds");
    }
    let setting = c::Setting::Application { kind: 22, scope: 0, number: 1 };
    if let Some(page) = reading.pages.get(&3)
        && let Some(Answer::Setting(Some(value))) = vocabulary.application_answer(Query::Setting(setting), page)
    {
        values.push((a::form_setting(setting), a::form_setting_value(&value))).expect("two limit kinds");
    }
    settings_form::render::limits_text(values.as_slice(), out);
}
