//! Whole-limit admission before party or run work reaches the root
//! (temper/docs/design/protocol/configuration/limit-settings.md, 4).
use alloc::boxed::Box;
use jig_domain_core::settings::{self as s, Override};
use jig_domain_people as people;
use skein_lib::bytes;
use temper_engine_domain as root;
use temper_engine_domain_forge::settings;

use super::{LimitKind, LimitSettings, resolve_limits};

fn kind(setting: s::Setting) -> Option<LimitKind> {
    if setting == s::Setting::Limits {
        Some(LimitKind::Jig)
    } else if let s::Setting::Application { kind: 22, .. } = setting {
        Some(LimitKind::Temper)
    } else {
        None
    }
}

fn values<'a>(item: &'a s::Item, kinds: &'a [root::KindValue]) -> Option<&'a [Override]> {
    match &item.edit {
        s::Edit::Reset => Some(&[]),
        // Shape and removal refusals remain the settings owner's, not a resolution of an empty list.
        s::Edit::Removed => None,
        s::Edit::Set(value) => {
            if item.setting == s::Setting::Limits {
                return value.as_limits();
            }
            for typed in kinds {
                if item.setting
                    == (s::Setting::Application { kind: typed.kind, scope: typed.scope, number: typed.number })
                    && let settings::Value::Limits(values) = &typed.value.value
                {
                    return Some(values);
                }
            }
            None
        }
    }
}

fn names(values: &[Override], expected: LimitKind) -> Option<Box<[u8]>> {
    for value in values {
        let found = if jig_protocol_config::names::find(&value.name).is_some() {
            Some(LimitKind::Jig)
        } else if crate::limits::find(&value.name).is_some() {
            Some(LimitKind::Temper)
        } else {
            None
        };
        match found {
            Some(found) if found != expected => {
                return Some(bytes::concat(&[
                    &value.name,
                    b" is a limit of ",
                    found.name(),
                    b", not ",
                    expected.name(),
                ]));
            }
            None => return Some(bytes::concat(&[&value.name, b" is no limit of this release"])),
            Some(LimitKind::Jig | LimitKind::Temper) => {}
        }
    }
    None
}

fn refused(index: usize, item: &s::Item, why: Box<[u8]>) -> people::Refusal {
    people::Refusal::Judged { at: u32::try_from(index).expect("bounded item position"), setting: item.setting, why }
}

impl LimitSettings {
    pub(crate) fn check(&self, items: &[s::Item], kinds: &[root::KindValue]) -> Option<people::Refusal> {
        let mut jig = self.jig.as_ref();
        let mut temper = self.temper.as_ref();
        let mut first = None;
        for (index, item) in items.iter().enumerate() {
            let Some(kind) = kind(item.setting) else { continue };
            let Some(values) = values(item, kinds) else { continue };
            if let Some(why) = names(values, kind) {
                return Some(refused(index, item, why));
            }
            if first.is_none() {
                first = Some((index, item));
            }
            match kind {
                LimitKind::Jig => jig = values,
                LimitKind::Temper => temper = values,
            }
        }
        let (mut index, mut item) = first?;
        let Err(problems) = resolve_limits(&self.inputs, jig, temper) else { return None };
        // Resolution names the offending source even when a check spans both lists.
        if let Some(problem) = problems.problems.as_slice().first() {
            for segment in &problem.path.segments {
                let jig_protocol_config::Segment::Member(label) = segment else { continue };
                let source = if label.ends_with(b" (setting limits)") {
                    Some(LimitKind::Jig)
                } else if label.ends_with(b" (setting temper-limits)") {
                    Some(LimitKind::Temper)
                } else {
                    None
                };
                if let Some(source) = source {
                    for (at, candidate) in items.iter().enumerate() {
                        if kind(candidate.setting) == Some(source) {
                            index = at;
                            item = candidate;
                            break;
                        }
                    }
                }
            }
        }
        Some(refused(index, item, temper_engine_service_startup::startup::lines(&problems)))
    }
}

pub(crate) fn party(
    state: Option<&LimitSettings>,
    input: root::Event,
) -> Result<root::Event, Box<jig_protocol_engine_clients::Event>> {
    let why = if let Some(state) = state {
        if let root::Event::Party(root::PartyInput::ChangeSettings { change, kinds, .. }) = &input {
            state.check(&change.items, kinds)
        } else if let root::Event::Party(root::PartyInput::ProposeSettings { change, kinds, .. }) = &input {
            state.check(&change.items, kinds)
        } else {
            None
        }
    } else {
        None
    };
    let Some(why) = why else { return Ok(input) };
    let root::Event::Party(
        root::PartyInput::ChangeSettings { reply_to, .. } | root::PartyInput::ProposeSettings { reply_to, .. },
    ) = input
    else {
        unreachable!("only settings requests are checked")
    };
    Err(Box::new(jig_protocol_engine_clients::Event::Reply { to: reply_to, reply: people::Reply::Refused(why) }))
}

pub(crate) fn host(state: Option<&LimitSettings>, input: root::Event) -> root::Event {
    if let Some(state) = state
        && let root::Event::Worker(root::WorkerInput::DecodedHostCall { body, .. }) = &input
        && let root::Tool::ProposeSettings { items, kinds, .. } = &body.tool
        && let Some(people::Refusal::Judged { why, .. }) = state.check(items, kinds)
    {
        let root::Event::Worker(root::WorkerInput::DecodedHostCall { to, .. }) = input else {
            unreachable!("only decoded proposals are checked")
        };
        return root::Event::Worker(root::WorkerInput::RenderedHostCall {
            to,
            answer: jig_domain_core::SettledAnswer::Host { error: true, body: why },
        });
    }
    input
}
