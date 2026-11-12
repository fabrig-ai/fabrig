//! Fields replaced and text appended over a default (jig/docs/design/domain/engine/overlays.md, 1).
use alloc::boxed::Box;
use crate::{Charter, CharterPart};
use skein_lib::{List, Writer};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Overlay {
    pub rest: Option<bool>,
    pub model: Option<crate::Model>,
    pub alternatives: Option<Box<[crate::Model]>>,
    pub inspect: Option<bool>,
    pub modify: Option<bool>,
    pub shell: Option<bool>,
    pub agents: Option<bool>,
    pub tools: Option<jig_domain_tasks::Tools>,
    pub hosts: Option<jig_domain_fleet::Kinds>,
    pub waiting: Option<skein_lib::Duration>,
    pub resume: Option<bool>,
    pub turns: Option<u32>,
    pub time: Option<skein_lib::Duration>,
    pub reserve_turns: Option<u32>,
    pub reserve_time: Option<skein_lib::Duration>,
    pub call_timeout: Option<skein_lib::Duration>,
    pub spend: Option<u64>,
    pub documents: Option<Box<[crate::Document]>>,
    pub instructions: Box<[u8]>,
    pub parts: Box<[CharterPart]>,
}
impl Overlay {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            rest: None,
            model: None,
            alternatives: None,
            inspect: None,
            modify: None,
            shell: None,
            agents: None,
            tools: None,
            hosts: None,
            waiting: None,
            resume: None,
            turns: None,
            time: None,
            reserve_turns: None,
            reserve_time: None,
            call_timeout: None,
            spend: None,
            documents: None,
            instructions: Box::new([]), parts: Box::new([]),
        }
    }
}
/// The default with every named field replaced and the overlay's text and parts appended.
#[must_use]
pub fn overlaid(default: &Charter, overlay: &Overlay) -> Charter {
    let mut value = default.clone();
    if let Some(field) = &overlay.rest { value.rest = *field; }
    if let Some(field) = &overlay.model { value.policy.model.clone_from(field); }
    if let Some(field) = &overlay.alternatives { value.policy.alternatives.clone_from(field); }
    if let Some(field) = &overlay.inspect { value.policy.inspect = *field; }
    if let Some(field) = &overlay.modify { value.policy.modify = *field; }
    if let Some(field) = &overlay.shell { value.policy.shell = *field; }
    if let Some(field) = &overlay.agents { value.policy.agents = *field; }
    if let Some(field) = &overlay.tools { value.policy.tools = *field; }
    if let Some(field) = &overlay.hosts { value.hosts = *field; }
    if let Some(field) = &overlay.waiting { value.policy.waiting = *field; }
    if let Some(field) = &overlay.resume { value.policy.resume = *field; }
    if let Some(field) = &overlay.turns { value.policy.turns = *field; }
    if let Some(field) = &overlay.time { value.policy.time = *field; }
    if let Some(field) = &overlay.reserve_turns { value.policy.reserve_turns = *field; }
    if let Some(field) = &overlay.reserve_time { value.policy.reserve_time = *field; }
    if let Some(field) = &overlay.call_timeout { value.policy.call_timeout = *field; }
    if let Some(field) = &overlay.spend { value.spend = *field; }
    if let Some(field) = &overlay.documents { value.documents.clone_from(field); }
    if !overlay.instructions.is_empty() {
        let length = default.policy.instructions.len().checked_add(overlay.instructions.len())
            .expect("admitted instruction lengths")
            .checked_add(if default.policy.instructions.is_empty() { 0 } else { 2 }).expect("joining blank line");
        let mut instructions = Writer::new(length);
        instructions.put(&default.policy.instructions).expect("default counted");
        if !default.policy.instructions.is_empty() { instructions.put(b"\n\n").expect("blank line counted"); }
        instructions.put(&overlay.instructions).expect("overlay counted");
        value.policy.instructions = instructions.finish();
    }
    let count = default.parts.len().checked_add(overlay.parts.len()).expect("admitted part counts");
    let mut parts = List::with_capacity(u32::try_from(count).expect("admitted parts"));
    for part in default.parts.iter().chain(overlay.parts.iter()) { parts.push(part.clone()).expect("parts counted"); }
    value.parts = parts.into_boxed();
    value
}
