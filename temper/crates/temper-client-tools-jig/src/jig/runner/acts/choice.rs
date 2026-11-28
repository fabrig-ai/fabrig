//! Model-choice flags are checked against me's offer
//! (jig/docs/design/domain/tasks/README.md, 3).
use super::{Call, Failed, value};
use crate::jig::{
    call::{Value, repeated, split_model_value},
    render,
    rows::{AMEND_CHOICE, ChoiceFlags},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Writer};

/// At most 256 UTF-8 bytes of a supplied name enter an error; controls end it.
pub(crate) const NAME_ECHO_BYTES: usize = 256;

/// Override the main and, when explicit, the sub-agents using offered model names.
pub(crate) fn selection(
    call: &Call,
    flags: ChoiceFlags,
    me: &c::Me,
    project: u32,
    before: Option<&c::LlmChoice>,
) -> Result<Option<c::LlmChoice>, Failed> {
    let mut choice = llm(call, flags, me, project, before)?;
    let explicit_main = value(call, flags.model).is_some();
    if explicit_main && let Some(choice) = choice.as_mut() {
        choice.later = None;
    }
    let subagent = flags.subagent.expect("a start takes --sub-agent");
    if repeated(call, subagent).is_some() && !explicit_main {
        return Err(Failed::Fixed(b"--sub-agent needs --model"));
    }
    let mut subagents = sub_agents(call, subagent, me, project)?;
    subagent_speeds(
        call,
        flags.subagent_speed,
        subagents.as_mut(),
        me,
        project,
        b" is not a sub-agent --sub-agent names",
    )?;
    if let Some(subagents) = subagents {
        choice.as_mut().expect("sub-agents require a main model").subagents = subagents;
    }
    Ok(choice)
}

/// Replace the main choice only when named, and set speeds on the kept sub-agents.
pub fn amendment(call: &Call, task: &c::Task, me: &c::Me) -> Result<Option<c::LlmChoice>, Failed> {
    let flags = AMEND_CHOICE;
    let speeds = match repeated(call, flags.subagent_speed) {
        Some(values) => !values.is_empty(),
        None => false,
    };
    if value(call, flags.model).is_none() && !speeds {
        return Ok(None);
    }
    let mut choice = llm(call, flags, me, task.project, task.llm.as_ref())?;
    let subagents = match choice.as_mut() {
        Some(choice) => Some(&mut choice.subagents),
        None => None,
    };
    subagent_speeds(call, flags.subagent_speed, subagents, me, task.project, b" is not one of the task's sub-agents")?;
    Ok(choice)
}

fn llm(
    call: &Call,
    flags: ChoiceFlags,
    me: &c::Me,
    project: u32,
    before: Option<&c::LlmChoice>,
) -> Result<Option<c::LlmChoice>, Failed> {
    let Some(Value::Text(name)) = value(call, flags.model) else { return Ok(before.cloned()) };
    let Some(model) = offered(me, project, name) else { return Err(not_offered(b"--model", name, None)) };
    let effort = match value(call, flags.effort) {
        Some(value) => {
            let Value::Text(name) = value else { return Err(Failed::Fixed(b"effort not loaded")) };
            let Some(effort) = offered_effort(model, name) else {
                return Err(unknown_effort(b"--effort", name, model, name, false));
            };
            Some(effort)
        }
        None => model.effort.clone(),
    };
    let speed = match value(call, flags.speed) {
        Some(value) => {
            let Value::Text(name) = value else { return Err(Failed::Fixed(b"speed not loaded")) };
            let Some(speed) = offered_speed(model, name) else {
                return Err(unknown_speed(b"--speed", name, model, None));
            };
            Some(speed)
        }
        None => None,
    };
    let main = c::ModelEffort { model: model.name.clone(), effort, speed };
    let mut choice = match before {
        Some(choice) => choice.clone(),
        None => c::LlmChoice { later: None, account: None, main: main.clone(), subagents: List::with_capacity(0) },
    };
    choice.main = main;
    Ok(Some(choice))
}
fn sub_agents(call: &Call, argument: usize, me: &c::Me, project: u32) -> Result<Option<List<c::ModelEffort>>, Failed> {
    let Some(values) = repeated(call, argument) else { return Ok(None) };
    let mut subagents: List<c::ModelEffort> =
        List::with_capacity(u32::try_from(values.len()).expect("bounded sub-agents"));
    for value in values {
        let Value::Text(text) = value else { return Err(Failed::Fixed(b"sub-agent not loaded")) };
        let (name, effort) = match split_model_value(text) {
            Some((name, effort)) => (name, Some(effort)),
            None => (text.as_ref(), None),
        };
        let Some(model) = offered(me, project, name) else {
            return Err(not_offered(b"--sub-agent", text, Some(name)));
        };
        for previous in &subagents {
            if previous.model == model.name {
                return Err(flag_refusal(b"--sub-agent", text, &[&echo(name), b" is already a sub-agent"]));
            }
        }
        let effort = match effort {
            Some(name) => {
                let Some(effort) = offered_effort(model, name) else {
                    return Err(unknown_effort(b"--sub-agent", text, model, name, true));
                };
                Some(effort)
            }
            None => model.effort.clone(),
        };
        subagents
            .push(c::ModelEffort { model: model.name.clone(), effort, speed: None })
            .expect("parsed sub-agents fit");
    }
    Ok(Some(subagents))
}
fn subagent_speeds(
    call: &Call,
    argument: usize,
    mut subagents: Option<&mut List<c::ModelEffort>>,
    me: &c::Me,
    project: u32,
    not_named: &[u8],
) -> Result<(), Failed> {
    let Some(values) = repeated(call, argument) else { return Ok(()) };
    let flag = b"--sub-agent-speed";
    let mut named = List::with_capacity(u32::try_from(values.len()).expect("bounded sub-agent speeds"));
    for value in values {
        let Value::Text(text) = value else { return Err(Failed::Fixed(b"sub-agent speed not loaded")) };
        let (name, speed) = split_model_value(text).expect("checked model:speed");
        let selection = match subagents.as_deref_mut() {
            Some(subagents) => subagent_named(subagents, name),
            None => None,
        };
        let Some(selection) = selection else { return Err(flag_refusal(flag, text, &[&echo(name), not_named])) };
        for previous in &named {
            if *previous == name {
                return Err(flag_refusal(flag, text, &[&echo(name), b" already has a speed"]));
            }
        }
        let Some(model) = offered(me, project, name) else { return Err(not_offered(flag, text, Some(name))) };
        let Some(speed) = offered_speed(model, speed) else {
            return Err(unknown_speed(flag, text, model, Some(speed)));
        };
        selection.speed = Some(speed);
        named.push(name).expect("one name per sub-agent speed");
    }
    Ok(())
}
fn subagent_named<'a>(subagents: &'a mut List<c::ModelEffort>, name: &[u8]) -> Option<&'a mut c::ModelEffort> {
    for index in 0..subagents.len() {
        let entry = subagents.get(index).expect("sub-agent index");
        if entry.model.bytes() == name {
            return subagents.get_mut(index);
        }
    }
    None
}
fn offered_speed(model: &c::MeModel, name: &[u8]) -> Option<c::Text> {
    if name == b"standard" {
        return Some(c::Text::new(Box::from(name), 32).expect("standard fits a speed name"));
    }
    for speed in &model.speeds {
        if speed.name.bytes() == name {
            return Some(speed.name.clone());
        }
    }
    None
}
fn unknown_speed(flag: &[u8], value: &[u8], model: &c::MeModel, speed: Option<&[u8]>) -> Failed {
    let mut parts: List<Box<[u8]>> = List::with_capacity(model.speeds.len().saturating_mul(2).saturating_add(6));
    if let Some(speed) = speed {
        parts.push(echo(speed)).expect("speed fits");
        parts.push(Box::from(b" is ".as_slice())).expect("speed separator fits");
    }
    parts.push(Box::from(b"not a speed ".as_slice())).expect("reason fits");
    parts.push(echo(model.name.bytes())).expect("model fits");
    parts.push(Box::from(b" declares; it declares ".as_slice())).expect("declaration fits");
    if model.speeds.is_empty() {
        parts.push(Box::from(b"none".as_slice())).expect("no speeds fit");
    }
    for (index, speed) in model.speeds.iter().enumerate() {
        if index != 0 {
            parts.push(Box::from(b", ".as_slice())).expect("separator fits");
        }
        parts.push(echo(speed.name.bytes())).expect("declared speed fits");
    }
    flag_refusal(flag, value, &[&joined(&parts)])
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
fn not_offered(flag: &[u8], value: &[u8], model: Option<&[u8]>) -> Failed {
    match model {
        Some(model) => {
            flag_refusal(flag, value, &[&echo(model), b" is not among the models you may choose in this project"])
        }
        None => flag_refusal(flag, value, &[b"not among the models you may choose in this project"]),
    }
}
fn unknown_effort(flag: &[u8], value: &[u8], model: &c::MeModel, effort: &[u8], subagent: bool) -> Failed {
    let mut parts: List<Box<[u8]>> = List::with_capacity(model.efforts.len().saturating_mul(2).saturating_add(6));
    if subagent {
        parts.push(echo(effort)).expect("effort fits");
        parts.push(Box::from(b" is ".as_slice())).expect("effort separator fits");
    }
    parts.push(Box::from(b"not an effort ".as_slice())).expect("reason fits");
    parts.push(echo(model.name.bytes())).expect("model fits");
    parts.push(Box::from(b" declares; it declares ".as_slice())).expect("declaration fits");
    if model.efforts.is_empty() {
        parts.push(Box::from(b"none".as_slice())).expect("no efforts fit");
    }
    for (index, effort) in model.efforts.as_slice().iter().enumerate() {
        if index != 0 {
            parts.push(Box::from(b", ".as_slice())).expect("separator fits");
        }
        parts.push(echo(effort.bytes())).expect("declared effort fits");
    }
    flag_refusal(flag, value, &[&joined(&parts)])
}
fn flag_refusal(flag: &[u8], value: &[u8], why: &[&[u8]]) -> Failed {
    let mut parts = List::with_capacity(u32::try_from(why.len()).expect("bounded reason").saturating_add(4));
    parts.push(Box::from(flag)).expect("flag fits");
    parts.push(Box::from(b" ".as_slice())).expect("space fits");
    parts.push(echo(value)).expect("value fits");
    parts.push(Box::from(b": ".as_slice())).expect("separator fits");
    for part in why {
        parts.push(Box::from(*part)).expect("reason fits");
    }
    Failed::Detail(joined(&parts))
}
fn joined(parts: &List<Box<[u8]>>) -> Box<[u8]> {
    let mut length = 0usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("refusal fits");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("measured refusal");
    }
    out.finish()
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
/// The first named effort or speed its model, as `me` shows it, does not offer
/// (client/ctl/tools-acts.md, 1).
pub(crate) fn settings_choice(items: &[c::SettingItem], me: &c::Me) -> Option<Box<[u8]>> {
    for (index, item) in items.iter().enumerate() {
        let set = match &item.edit {
            c::SettingEdit::Set(set) => set,
            c::SettingEdit::Removed | c::SettingEdit::Reset => continue,
        };
        match &set.value {
            c::SettingValue::Template(template) => {
                if let Some(llm) = &template.llm {
                    if let Some(why) = setting_llm(index, &llm.main, &llm.subagents, me) {
                        return Some(why);
                    }
                    if let Some(later) = &llm.later
                        && let Some(why) = setting_llm(index, &later.main, &later.subagents, me)
                    {
                        return Some(why);
                    }
                }
            }
            c::SettingValue::Overlay(overlay) => {
                if let Some(model) = &overlay.model
                    && let Some(why) =
                        setting_model(index, &model.name, model.effort.as_ref(), model.speed.as_ref(), me)
                {
                    return Some(why);
                }
                if let Some(models) = &overlay.alternatives {
                    for model in models {
                        if let Some(why) =
                            setting_model(index, &model.name, model.effort.as_ref(), model.speed.as_ref(), me)
                        {
                            return Some(why);
                        }
                    }
                }
            }
            c::SettingValue::Charter(charter) => {
                let main = &charter.model;
                if let Some(why) = setting_model(index, &main.name, main.effort.as_ref(), main.speed.as_ref(), me) {
                    return Some(why);
                }
                for model in &charter.alternatives {
                    if let Some(why) =
                        setting_model(index, &model.name, model.effort.as_ref(), model.speed.as_ref(), me)
                    {
                        return Some(why);
                    }
                }
            }
            c::SettingValue::Policy(_)
            | c::SettingValue::Rules(_)
            | c::SettingValue::Limits(_)
            | c::SettingValue::Provider(_)
            | c::SettingValue::Worker(_)
            | c::SettingValue::Model(_)
            | c::SettingValue::Application(_) => {}
        }
    }
    None
}
fn setting_llm(index: usize, main: &c::ModelEffort, subagents: &List<c::ModelEffort>, me: &c::Me) -> Option<Box<[u8]>> {
    if let Some(why) = setting_model(index, &main.model, main.effort.as_ref(), main.speed.as_ref(), me) {
        return Some(why);
    }
    for model in subagents {
        if let Some(why) = setting_model(index, &model.model, model.effort.as_ref(), model.speed.as_ref(), me) {
            return Some(why);
        }
    }
    None
}
fn setting_model(
    index: usize,
    name: &c::Text,
    effort: Option<&c::Text>,
    speed: Option<&c::Text>,
    me: &c::Me,
) -> Option<Box<[u8]>> {
    if effort.is_none() && speed.is_none() {
        return None;
    }
    for project in &me.projects {
        for model in &project.models {
            if model.name != *name {
                continue;
            }
            if let Some(effort) = effort
                && offered_effort(model, effort.bytes()).is_none()
            {
                return Some(setting_refusal(index, effort.bytes(), model, SettingMember::Effort));
            }
            if let Some(speed) = speed
                && speed.bytes() != b"standard"
            {
                for declared in &model.speeds {
                    if declared.name == *speed {
                        return None;
                    }
                }
                return Some(setting_refusal(index, speed.bytes(), model, SettingMember::Speed));
            }
            return None;
        }
    }
    None
}
enum SettingMember {
    Effort,
    Speed,
}
fn setting_refusal(index: usize, supplied: &[u8], model: &c::MeModel, member: SettingMember) -> Box<[u8]> {
    let (prefix, listed, count) = match member {
        SettingMember::Effort => (b" offers no effort ".as_slice(), b"; it offers ".as_slice(), model.efforts.len()),
        SettingMember::Speed => (b" declares no speed ".as_slice(), b"; it declares ".as_slice(), model.speeds.len()),
    };
    let item = skein_lib::Decimal::of(
        u64::try_from(index).expect("bounded item index").checked_add(1).expect("item number fits"),
    );
    let mut parts = List::with_capacity(count.saturating_mul(2).saturating_add(9));
    parts.push(Box::from(b"settings file item ".as_slice())).expect("prefix fits");
    parts.push(Box::from(item.as_bytes())).expect("item fits");
    parts.push(Box::from(b": model ".as_slice())).expect("model prefix fits");
    parts.push(echo(model.name.bytes())).expect("name fits");
    parts.push(Box::from(prefix)).expect("member prefix fits");
    parts.push(echo(supplied)).expect("supplied name fits");
    parts.push(Box::from(listed)).expect("list prefix fits");
    if count == 0 {
        parts.push(Box::from(b"none".as_slice())).expect("empty list fits");
    }
    for at in 0..count {
        if at != 0 {
            parts.push(Box::from(b", ".as_slice())).expect("separator fits");
        }
        let name = match member {
            SettingMember::Effort => model.efforts.get(at).expect("effort within count"),
            SettingMember::Speed => &model.speeds.get(at).expect("speed within count").name,
        };
        parts.push(echo(name.bytes())).expect("offered name fits");
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
