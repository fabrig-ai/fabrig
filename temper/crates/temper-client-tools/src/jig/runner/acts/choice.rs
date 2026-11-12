//! Model names come from me; explicit sub-agents replace the template choices
//! (client/ctl/areas.md).
use super::{Call, Failed, value};
use crate::jig::{
    call::{Value, Values},
    render,
    runner::Printed,
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Writer};

/// At most 256 UTF-8 bytes of a supplied name enter an error; controls end it.
pub(crate) const NAME_ECHO_BYTES: usize = 256;

/// Override the main and, when explicit, the sub-agents using offered model names.
pub(crate) fn selection(
    call: &Call,
    model: u8,
    effort: u8,
    subagent: u8,
    me: &c::Me,
    project: u32,
    before: Option<&c::LlmChoice>,
) -> Result<Option<c::LlmChoice>, Failed> {
    let mut choice = llm(call, model, effort, me, project, before)?;
    if let Some(Some(_)) = call.values.get(usize::from(subagent))
        && value(call, usize::from(model)).is_none()
    {
        return Err(Failed::Fixed(b"--sub-agent needs --model"));
    }
    if let Some(subagents) = sub_agents(call, subagent, me, project)? {
        choice.as_mut().expect("sub-agents require a main model").subagents = subagents;
    }
    Ok(choice)
}

pub(crate) fn llm(
    call: &Call,
    model_argument: u8,
    effort_argument: u8,
    me: &c::Me,
    project: u32,
    before: Option<&c::LlmChoice>,
) -> Result<Option<c::LlmChoice>, Failed> {
    let Some(Value::Text(name)) = value(call, usize::from(model_argument)) else { return Ok(before.cloned()) };
    let model = offered(me, project, name).ok_or(Failed::NotOffered { name: reported(name) })?;
    let effort = match value(call, usize::from(effort_argument)) {
        Some(value) => {
            let Value::Text(name) = value else { return Err(Failed::Fixed(b"effort not loaded")) };
            Some(offered_effort(model, name).ok_or(Failed::Unknown { what: b"effort", name: reported(name) })?)
        }
        None => model.effort.clone(),
    };
    let main = c::ModelEffort { model: model.name.clone(), effort };
    let mut choice = match before {
        Some(choice) => choice.clone(),
        None => c::LlmChoice { later: None, account: None, main: main.clone(), subagents: List::with_capacity(0) },
    };
    choice.main = main;
    Ok(Some(choice))
}
pub(crate) fn sub_agents(
    call: &Call,
    argument: u8,
    me: &c::Me,
    project: u32,
) -> Result<Option<List<c::ModelEffort>>, Failed> {
    let Some(Some(Values::Many(values))) = call.values.get(usize::from(argument)) else { return Ok(None) };
    let mut subagents: List<c::ModelEffort> =
        List::with_capacity(u32::try_from(values.len()).expect("bounded sub-agents"));
    for value in values {
        let Value::Text(text) = value else { return Err(Failed::Fixed(b"sub-agent not loaded")) };
        let mut colon = None;
        for (index, byte) in text.iter().enumerate() {
            if *byte == b':' {
                colon = Some(index);
            }
        }
        let (name, effort) = match colon {
            Some(index) => {
                let (name, rest) = text.split_at_checked(index).expect("colon in model choice");
                (name, Some(rest.get(1..).expect("colon before effort")))
            }
            None => (text.as_ref(), None),
        };
        let model = offered(me, project, name).ok_or(Failed::NotOffered { name: reported(name) })?;
        for previous in &subagents {
            if previous.model == model.name {
                return Err(Failed::SubAgentNamedTwice { name: reported(name) });
            }
        }
        let effort = match effort {
            Some(name) => {
                Some(offered_effort(model, name).ok_or(Failed::Unknown { what: b"effort", name: reported(name) })?)
            }
            None => model.effort.clone(),
        };
        subagents.push(c::ModelEffort { model: model.name.clone(), effort }).expect("parsed sub-agents fit");
    }
    Ok(Some(subagents))
}
fn offered<'a>(me: &'a c::Me, project: u32, name: &[u8]) -> Option<&'a c::MeModel> {
    for membership in &me.projects {
        if membership.project == project {
            for model in &membership.models {
                if model.name.bytes() == name {
                    return Some(model);
                }
            }
            return None;
        }
    }
    None
}
fn offered_effort(model: &c::MeModel, name: &[u8]) -> Option<c::Text> {
    for effort in &model.efforts {
        if effort.bytes() == name {
            return Some(effort.clone());
        }
    }
    None
}
// Keep up to four more bytes (at least one more code point) so named()
// sees a name over 256 B as cut.
fn reported(name: &[u8]) -> Box<[u8]> {
    let end = render::text_end(name, NAME_ECHO_BYTES.saturating_add(4));
    Box::from(name.get(..end).expect("bounded failure name"))
}
fn name_end(name: &[u8]) -> usize {
    let mut end = render::text_end(name, NAME_ECHO_BYTES);
    // Cut C0, DEL and UTF-8 C1 controls before they can reach a terminal.
    for (index, byte) in name.get(..end).expect("bounded echo").iter().enumerate() {
        let c1 = *byte == 0xc2
            && match name.get(index.saturating_add(1)) {
                Some(next) => (0x80..=0x9f).contains(next),
                None => false,
            };
        if *byte < 0x20 || *byte == 0x7f || c1 {
            end = index;
            break;
        }
    }
    end
}
fn echo(name: &[u8]) -> Box<[u8]> {
    let end = name_end(name);
    let cut = end < name.len();
    let mut out = Writer::new(end.checked_add(if cut { 3 } else { 0 }).expect("bounded name"));
    out.put(name.get(..end).expect("UTF-8 echo range")).expect("measured echo");
    if cut {
        out.put(b"...").expect("measured cut marker");
    }
    out.finish()
}
pub(super) fn named(prefix: &[u8], what: &[u8], name: &[u8], json: bool) -> Printed {
    let end = name_end(name);
    let cut = end < name.len();
    let length = prefix
        .len()
        .checked_add(what.len())
        .expect("error prefix fits")
        .checked_add(2)
        .expect("separator fits")
        .checked_add(end)
        .expect("echo fits")
        .checked_add(if cut { 3 } else { 0 })
        .expect("cut marker fits");
    let mut out = Writer::new(length);
    out.put(prefix).expect("measured error prefix");
    out.put(what).expect("measured name kind");
    out.put(b": ").expect("measured separator");
    out.put(name.get(..end).expect("UTF-8 echo range")).expect("measured echo");
    if cut {
        out.put(b"...").expect("measured cut marker");
    }
    render::failure(&out.finish(), json, 1)
}

/// The first named effort of `items` that its model, as `me` shows it, doesn't offer
/// (client/ctl/tools.md, 3).
pub(crate) fn settings_effort(items: &[c::SettingItem], me: &c::Me) -> Option<Box<[u8]>> {
    for (index, item) in items.iter().enumerate() {
        let set = match &item.edit {
            c::SettingEdit::Set(set) => set,
            c::SettingEdit::Removed | c::SettingEdit::Reset => continue,
        };
        match &set.value {
            c::SettingValue::Template(template) => {
                if let Some(llm) = &template.llm {
                    if let Some(why) = setting_effort(index, &llm.main.model, llm.main.effort.as_ref(), me) {
                        return Some(why);
                    }
                    for model in &llm.subagents {
                        if let Some(why) = setting_effort(index, &model.model, model.effort.as_ref(), me) {
                            return Some(why);
                        }
                    }
                }
            }
            c::SettingValue::Charter(charter) => {
                if let Some(why) = setting_effort(index, &charter.model.name, charter.model.effort.as_ref(), me) {
                    return Some(why);
                }
                for model in &charter.alternatives {
                    if let Some(why) = setting_effort(index, &model.name, model.effort.as_ref(), me) {
                        return Some(why);
                    }
                }
            }
            c::SettingValue::Overlay(_)
            | c::SettingValue::Policy(_)
            | c::SettingValue::Rules(_)
            | c::SettingValue::Model(_)
            | c::SettingValue::Application(_) => {}
        }
    }
    None
}
fn setting_effort(index: usize, name: &c::Text, effort: Option<&c::Text>, me: &c::Me) -> Option<Box<[u8]>> {
    let effort = effort?;
    for project in &me.projects {
        for model in &project.models {
            if model.name == *name {
                if offered_effort(model, effort.bytes()).is_some() {
                    return None;
                }
                return Some(effort_refusal(index, name.bytes(), effort.bytes(), model));
            }
        }
    }
    None
}
fn effort_refusal(index: usize, name: &[u8], effort: &[u8], model: &c::MeModel) -> Box<[u8]> {
    let item = skein_lib::Decimal::of(
        u64::try_from(index).expect("bounded item index").checked_add(1).expect("item number fits"),
    );
    let name = echo(name);
    let effort = echo(effort);
    let mut parts = List::with_capacity(model.efforts.len().saturating_mul(2).saturating_add(9));
    parts.push(Box::from(b"settings file item ".as_slice())).expect("prefix fits");
    parts.push(Box::from(item.as_bytes())).expect("item fits");
    parts.push(Box::from(b": model ".as_slice())).expect("model prefix fits");
    parts.push(name).expect("name fits");
    parts.push(Box::from(b" offers no effort ".as_slice())).expect("effort prefix fits");
    parts.push(effort).expect("effort fits");
    parts.push(Box::from(b"; it offers ".as_slice())).expect("offered prefix fits");
    if model.efforts.is_empty() {
        parts.push(Box::from(b"none".as_slice())).expect("empty efforts fit");
    }
    for (index, effort) in model.efforts.as_slice().iter().enumerate() {
        if index != 0 {
            parts.push(Box::from(b", ".as_slice())).expect("separator fits");
        }
        parts.push(echo(effort.bytes())).expect("offered effort fits");
    }
    let mut length = 0usize;
    for part in &parts {
        length = length.checked_add(part.len()).expect("refusal fits");
    }
    let mut out = Writer::new(length);
    for part in &parts {
        out.put(part).expect("measured refusal");
    }
    out.finish()
}
