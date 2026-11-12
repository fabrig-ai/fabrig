//! Ctl argv and client-value adapter for the neutral settings file form.
use super::{
    Limits,
    call::{Call, Value, Values, number, value},
    settings_adapter,
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;
pub(crate) fn selector(call: &Call) -> c::Setting {
    if call.form == 1 {
        return c::Setting::Rules;
    }
    let number =
        c::SettingNumber { number: u32::try_from(number(call, 1).expect("setting number")).expect("parsed u32") };
    match value(call, 0) {
        Some(Value::Word(0)) => c::Setting::Charter(number),
        Some(Value::Word(1)) => c::Setting::Template(number),
        Some(Value::Word(2)) => c::Setting::Policy(number),
        Some(Value::Word(3)) => c::Setting::Model(number),
        Some(_) | None => unreachable!("parsed setting kind"),
    }
}
pub(crate) fn request(call: &Call, limits: &Limits) -> Result<c::ChangeSettings, Box<[u8]>> {
    let slot = call.values.first().and_then(Option::as_ref);
    let files = match slot {
        Some(Values::One(v)) => core::slice::from_ref(v),
        Some(Values::Many(v)) => v,
        None => return Err(Box::from(b"settings file: FILE: input not loaded".as_slice())),
    };
    let form_limits = settings_adapter::limits(limits);
    let mut items: List<settings_form::view::SettingItem> = List::uncapped();
    let mut origins: List<(Box<[u8]>, u32)> = List::uncapped();
    for file in files {
        let (name, decoded) = match file {
            Value::Text(bytes) => (b"-".as_slice(), settings_form::file::decode(bytes, &form_limits)),
            Value::SettingsFile(input) => {
                let decoded = settings_form::file::decode(&input.bytes, &form_limits).and_then(|mut items| {
                    settings_form::overlay::assemble(&mut items, input.directories.as_slice(), &form_limits)?;
                    Ok(items)
                });
                (input.name.as_ref(), decoded)
            }
            Value::File(_) | Value::Stdin | Value::Number(_) | Value::Word(_) | Value::Duration(_)
            | Value::Proposal(_) | Value::Digest(_) | Value::Condition(_) | Value::Switch | Value::Project(_)
            | Value::Turn(_) | Value::Cursor(_) | Value::Group(_) | Value::Order(_) | Value::Tool(_) => {
                return Err(Box::from(b"settings file: FILE: input not loaded".as_slice()));
            }
        };
        let decoded = decoded.map_err(|why| if files.len() > 1 { named_refusal(name, &why) } else { why })?;
        if !settings_form::overlay::directories(&decoded).is_empty() {
            return Err(Box::from(b"settings file: overlay.charters: directories not loaded".as_slice()));
        }
        for (index, next) in decoded.iter().enumerate() {
            let item = u32::try_from(index).expect("bounded item").checked_add(1).expect("item number");
            let previous = items.iter().position(|old| old.setting == next.setting);
            if let Some(index) = previous {
                let (first_file, first_item) = origins.get(u32::try_from(index).expect("item index")).expect("origin");
                let old = items.get_mut(u32::try_from(index).expect("item index")).expect("previous item");
                if let (Some(first), Some(second)) = (overlay_mut(&mut old.edit), overlay(&next.edit)) {
                    merge(first, second, &form_limits)?;
                } else {
                    return Err(conflict(next.setting, &old.edit, &next.edit, first_file, *first_item, name, item));
                }
            } else {
                items.push(next.clone()).expect("bounded files");
                origins.push((name.into(), item)).expect("same count");
            }
        }
    }
    let change = settings_form::view::ChangeSettings {
        against: u32::try_from(number(call, if call.tool == crate::jig::rows::PROPOSE_SETTINGS_AT { 2 } else { 1 }).expect("against")).expect("parsed u32"),
        items,
        running: settings_form::view::Running::Stays,
    };
    settings_form::wire::wire_change_settings(&change, &form_limits.settings)
        .or(Err(Box::<[u8]>::from(b"settings file: encoded items bytes: bound".as_slice())))?;
    let mut items = List::with_capacity(change.items.len());
    for item in &change.items { items.push(settings_adapter::client_setting_item(item)).expect("same count"); }
    Ok(c::ChangeSettings {
        against: change.against, items,
        running: if call.tool == crate::jig::rows::CHANGE_SETTINGS_AT && value(call, 2) == Some(&Value::Word(1)) { c::Running::Moves } else { c::Running::Stays },
    })
}
fn overlay(edit: &settings_form::view::SettingEdit) -> Option<&settings_form::view::Overlay> {
    if let settings_form::view::SettingEdit::Set(set) = edit
        && let settings_form::view::SettingValue::Overlay(v) = &set.value { Some(v) } else { None }
}
fn overlay_mut(edit: &mut settings_form::view::SettingEdit) -> Option<&mut settings_form::view::Overlay> {
    if let settings_form::view::SettingEdit::Set(set) = edit
        && let settings_form::view::SettingValue::Overlay(v) = &mut set.value { Some(v) } else { None }
}
#[expect(clippy::disallowed_macros, reason = "settings input boundary names the conflicting files and items")]
fn conflict(setting: settings_form::view::Setting, first: &settings_form::view::SettingEdit, second: &settings_form::view::SettingEdit, first_file: &[u8], first_item: u32, second_file: &[u8], second_item: u32) -> Box<[u8]> {
    let mut measured = settings_form::Text::measure(); settings_form::render::setting_text(setting, &mut measured);
    let mut text = settings_form::Text::writing_exact(measured); settings_form::render::setting_text(setting, &mut text);
    alloc::format!("settings apply: {}: {} in {} item {} beside {} in {} item {}", core::str::from_utf8(&text.finish()).expect("setting name"), edit_name(second), core::str::from_utf8(second_file).unwrap_or("<file>"), second_item,
        edit_name(first), core::str::from_utf8(first_file).unwrap_or("<file>"), first_item).into_bytes().into_boxed_slice()
}
fn edit_name(edit: &settings_form::view::SettingEdit) -> &'static str {
    use settings_form::view::SettingEdit;
    match edit { SettingEdit::Set(_) => if overlay(edit).is_some() { "an overlay" } else { "set" }, SettingEdit::Removed => "removal", SettingEdit::Reset => "reset" }
}
#[must_use]
pub fn named_refusal(name: &[u8], why: &[u8]) -> Box<[u8]> {
    let rest = why.strip_prefix(b"settings file").unwrap_or(why);
    let mut out = skein_lib::Writer::new(name.len().checked_add(rest.len()).expect("bounded diagnostic").checked_add(14).expect("prefix"));
    for bytes in [b"settings file ".as_slice(), name, rest] { out.put(bytes).expect("measured diagnostic"); }
    out.finish()
}
fn merge(first: &mut settings_form::view::Overlay, second: &settings_form::view::Overlay, limits: &settings_form::Limits) -> Result<(), Box<[u8]>> {
    first.instructions = settings_form::overlay::join(&first.instructions, second.instructions.bytes(), limits.settings.charter_instructions)?;
    let mut parts = List::uncapped();
    for part in first.parts.iter().chain(second.parts.iter()) { parts.push(part.clone()).expect("bounded parts"); }
    first.parts = parts;
    if second.rest.is_some() { first.rest = second.rest.clone(); }
    if second.model.is_some() { first.model = second.model.clone(); }
    if second.alternatives.is_some() { first.alternatives = second.alternatives.clone(); }
    if second.inspect.is_some() { first.inspect = second.inspect.clone(); }
    if second.modify.is_some() { first.modify = second.modify.clone(); }
    if second.shell.is_some() { first.shell = second.shell.clone(); }
    if second.agents.is_some() { first.agents = second.agents.clone(); }
    if second.tools.is_some() { first.tools = second.tools.clone(); }
    if second.hosts.is_some() { first.hosts = second.hosts.clone(); }
    if second.waiting.is_some() { first.waiting = second.waiting.clone(); }
    if second.resume.is_some() { first.resume = second.resume.clone(); }
    if second.turns.is_some() { first.turns = second.turns.clone(); }
    if second.time.is_some() { first.time = second.time.clone(); }
    if second.reserve_turns.is_some() { first.reserve_turns = second.reserve_turns.clone(); }
    if second.reserve_time.is_some() { first.reserve_time = second.reserve_time.clone(); }
    if second.call_timeout.is_some() { first.call_timeout = second.call_timeout.clone(); }
    if second.spend.is_some() { first.spend = second.spend.clone(); }
    if second.documents.is_some() { first.documents = second.documents.clone(); }
    Ok(())
}

/// Decode the settings file into client items using the shared neutral form.
pub fn decode(bytes: &[u8], limits: &Limits) -> Result<List<c::SettingItem>, Box<[u8]>> {
    let items = settings_form::file::decode(bytes, &settings_adapter::limits(limits))?;
    let mut out = List::with_capacity(items.len());
    for item in &items {
        out.push(settings_adapter::client_setting_item(item)).expect("one client item per form item");
    }
    Ok(out)
}
