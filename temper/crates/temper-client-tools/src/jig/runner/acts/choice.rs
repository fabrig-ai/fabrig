//! Model names come from me; account and sub-agent preferences stay as supplied
//! by the template or task (client/tools.md, 5.5).
use super::{Call, Failed, value};
use crate::jig::{call::Value, render, runner::Printed};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Writer};

/// At most 256 UTF-8 bytes of a supplied name enter an error; controls end it.
pub(crate) const NAME_ECHO_BYTES: usize = 256;

pub(crate) fn llm(
    call: &Call,
    model_argument: u8,
    effort_argument: u8,
    me: &c::Me,
    project: u32,
    before: Option<&c::LlmChoice>,
) -> Result<Option<c::LlmChoice>, Failed> {
    let Some(Value::Text(name)) = value(call, usize::from(model_argument)) else { return Ok(before.cloned()) };
    let model = offered(me, project, name).ok_or(Failed::Unknown { what: b"model", name: reported(name) })?;
    let effort = match value(call, usize::from(effort_argument)) {
        Some(value) => {
            let Value::Text(name) = value else { return Err(Failed::Fixed(b"effort not loaded")) };
            Some(offered_effort(model, name).ok_or(Failed::Unknown { what: b"effort", name: reported(name) })?)
        }
        None => {
            if !model.efforts.is_empty() {
                return Err(Failed::EffortRequired { name: reported(name) });
            }
            None
        }
    };
    let main = c::ModelEffort { model: model.name.clone(), effort };
    let mut choice = match before {
        Some(choice) => choice.clone(),
        None => c::LlmChoice { account: None, main: main.clone(), subagents: List::with_capacity(0) },
    };
    choice.main = main;
    Ok(Some(choice))
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
pub(super) fn named(prefix: &[u8], what: &[u8], name: &[u8], json: bool) -> Printed {
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
