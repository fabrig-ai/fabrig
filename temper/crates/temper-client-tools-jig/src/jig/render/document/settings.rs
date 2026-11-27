//! Client settings views adapt to the neutral settings renderer.
use super::Text;
use crate::jig::render::Parts;
use crate::jig::{Limits, settings_adapter as a};
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
pub(crate) fn text(
    value: &c::ReadValue,
    call: &crate::jig::call::Call,
    limits: &Limits,
    vocabulary: &dyn Parts,
    out: &mut Text,
) {
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
        if call.tool != crate::jig::rows::SETTING_AT || crate::jig::settings_file::revision(call).is_none() {
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
            out.put(b" \"");
            out.words(name.bytes.bytes());
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
