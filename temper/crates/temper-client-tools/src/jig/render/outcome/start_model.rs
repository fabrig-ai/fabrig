//! Start refusals name the call's flag or the kept choice's part (client/ctl/areas.md area 5).
use super::{Operation, Text};
use crate::jig::{
    call::{Call, Value, Values, value},
    rows,
};
use alloc::boxed::Box;
use jig_client::{Ask, contract as c};
use skein_json::writer::Encoder;
use skein_lib::List;

#[derive(Debug)]
pub(crate) struct Start {
    executor: c::Executor,
    choice: Option<c::LlmChoice>,
    flags: Option<ModelFlags>,
}
#[derive(Debug)]
struct ModelFlags {
    main: Option<Box<[u8]>>,
    subagents: List<(c::Text, Box<[u8]>)>,
}
impl Start {
    pub(super) fn of(ask: &Ask, word: &[u8]) -> Option<Self> {
        if word != b"goal" && word != b"chat" {
            return None;
        }
        if let Ask::Jig(ask) = ask
            && let c::Ask::Create { tasks, .. } | c::Ask::Propose { action: c::ProposeAction::Batch { tasks }, .. } =
                ask.as_ref()
            && let Some(task) = tasks.get(0)
        {
            return Some(Self { executor: task.executor, choice: task.llm.clone(), flags: None });
        }
        None
    }
    pub(super) fn flags(&mut self, call: &Call) {
        let (main, subagent) = if call.tool == rows::GOAL_START_AT {
            (8, 10)
        } else if call.tool == rows::CHAT_CREATE_AT {
            (3, 5)
        } else {
            return;
        };
        let main = if let Some(Value::Text(model)) = value(call, main) { Some(model.clone()) } else { None };
        let capacity = match &self.choice {
            Some(choice) => choice.subagents.len(),
            None => 0,
        };
        let mut subagents = List::with_capacity(capacity);
        if let Some(choice) = &self.choice
            && let Some(Some(Values::Many(values))) = call.values.get(subagent)
        {
            for (chosen, given) in choice.subagents.iter().zip(values.iter()) {
                let Value::Text(given) = given else { unreachable!("sub-agent text") };
                subagents.push((chosen.model.clone(), given.clone())).expect("sent sub-agent count");
            }
        }
        self.flags = Some(ModelFlags { main, subagents });
    }
    fn flag(&self, model: &c::Text) -> Option<(&'static [u8], &[u8])> {
        let flags = self.flags.as_ref()?;
        if let Some(main) = &flags.main
            && main.as_ref() == model.bytes()
        {
            return Some((b"--model", main));
        }
        for (chosen, given) in &flags.subagents {
            if chosen == model {
                return Some((b"--sub-agent", given));
            }
        }
        None
    }
    fn main(&self, model: &c::Text) -> bool {
        if let Some(choice) = &self.choice {
            if choice.main.model == *model {
                return true;
            }
            if let Some(later) = &choice.later
                && later.main.model == *model
            {
                return true;
            }
        }
        false
    }
    pub(super) fn json(&self, refusal: &c::Refusal, out: &mut Encoder) {
        if let c::Refusal::Llm { model, .. } = refusal
            && let Some((flag, value)) = self.flag(model)
        {
            out.key(b"flag");
            out.string(flag);
            out.key(b"value");
            out.string(value);
        }
    }
}
pub(super) fn refusal(operation: &Operation, refusal: &c::Refusal, out: &mut Text) -> bool {
    let Some(start) = &operation.start_model else { return false };
    let c::Refusal::Llm { model, why } = refusal else { return false };
    out.put(match operation.tool {
        Some(tool) => tool.name,
        None if operation.word == b"goal" => b"goal start",
        None => b"chat start",
    });
    out.put(b": ");
    if let Some((flag, value)) = start.flag(model) {
        out.put(flag);
        out.put(b" ");
        name(value, out);
    } else {
        out.put(if start.main(model) { b"main model " } else { b"sub-agent model " });
        name(model.bytes(), out);
    }
    out.put(b": ");
    match why {
        c::LlmWhy::Charter => {
            let c::Executor::Agent { charter } = start.executor else { unreachable!("charter refusal of an agent") };
            out.put(b"not one of charter ");
            out.number(u64::from(charter));
            out.put(b"'s models");
        }
        c::LlmWhy::Authority => out.put(b"outside the authority the task is given"),
        c::LlmWhy::Effort => out.put(b"its effort is not one the model declares"),
        c::LlmWhy::Unconfigured => out.put(b"not a configured model"),
        c::LlmWhy::Procedure => out.put(b"the task's executor runs no model"),
        c::LlmWhy::Repeated => out.put(b"named twice among the sub-agents"),
    }
    true
}
fn name(bytes: &[u8], out: &mut Text) {
    let mut end = crate::jig::render::text_end(bytes, 256);
    for (index, byte) in bytes.get(..end).expect("bounded name").iter().enumerate() {
        let c1 = *byte == 0xc2
            && match bytes.get(index.saturating_add(1)) {
                Some(next) => (0x80..=0x9f).contains(next),
                None => false,
            };
        if *byte < 0x20 || *byte == 0x7f || c1 {
            end = index;
            break;
        }
    }
    out.put(bytes.get(..end).expect("name prefix"));
    if end < bytes.len() {
        out.put(b"...");
    }
}
