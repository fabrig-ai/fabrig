use super::chats;
use jig_client::contract as c;
use skein_lib::List;

pub const DONE: u32 = 6;

pub const TOTAL: u32 = 9;

pub const TEXT_BYTES: u32 = 64;

pub const CHARTER: u32 = 1;

pub const LABEL: u32 = 1;

pub const DEPTH: u32 = 1;

#[must_use]
pub fn line() -> c::TaskLine {
    let mut line = chats::line();
    line.delegates_done = DONE;
    line.delegates_total = TOTAL;
    line
}

#[must_use]
pub fn template(number: u32, name: &[u8], tracked: bool) -> c::Template {
    c::Template {
        resources: List::uncapped(),
        number,
        name: c::DocumentText::complete(c::Text::new(name.into(), TEXT_BYTES).unwrap()),
        executor: c::TemplateExecutor::Charter(CHARTER),
        contract: c::TaskContractDocument::Report { words: TEXT_BYTES },
        shallower: DEPTH,
        budget: c::TemplateBudget::AsAllowed { cap: None },
        deadline: c::TemplateDeadline::None,
        tracked: if tracked { Some(LABEL) } else { None },
        flags: line().flags,
        label: LABEL,
        grace: None,
        llm: None,
    }
}
