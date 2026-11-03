//! The forge's settings values beside jig's states (temper/docs/design/client/forge.md, 9).
use super::{Out, Runner, reads};
use crate::jig::{
    call::{Call, Project, Value, value},
    rows,
};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
};
use jig_client::{Event, Family, Readable, Readout, contract as c};
use skein_lib::{Queue, Reader};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) const SLOT: u32 = 7;
#[derive(Debug)]
pub(crate) struct Reading {
    pub items: Option<w::ProposalItemsAnswer>,
    pub current: BTreeMap<(u32, u32, u32), Readout>,
    pub query: Option<w::Read>,
}
impl Reading {
    pub(super) fn new() -> Self {
        Self { items: None, current: BTreeMap::new(), query: None }
    }
}
pub(super) fn resolve_project(call: &mut Call, me: Option<&c::Me>) -> bool {
    if call.tool != rows::SETTING_AT || call.form != 2 {
        return true;
    }
    let project = match value(call, 2) {
        Some(Value::Project(Project::Number(number))) => Some(*number),
        Some(Value::Project(Project::Name(name))) => {
            let mut found = None;
            if let Some(me) = me {
                for project in &me.projects {
                    if project.name.bytes() == name.as_ref() {
                        found = Some(project.project);
                        break;
                    }
                }
            }
            found
        }
        Some(_) => unreachable!("project form"),
        None => return true,
    };
    let Some(project) = project else { return false };
    *call.values.get_mut(2).expect("setting project argument") =
        Some(crate::jig::call::Values::One(Value::Project(Project::Number(project))));
    true
}
pub(super) fn setting_query(kind: u32, scope: u32, number: u32) -> w::Read {
    w::Read::Setting(
        w::SettingRead::new(&w::CEILINGS, w::SettingReadParts { kind, scope, number }).expect("fixed read"),
    )
}
pub(super) fn read(runner: &Runner, reading: &mut reads::Reading, query: w::Read, out: &mut Queue<Out>) {
    reading.pending |= 1 << SLOT;
    let record = w::Reading::new(&w::CEILINGS, w::ReadingParts { read: query.clone() }).expect("fixed read");
    reading.kinds.query = Some(query);
    out.push(Out::Client {
        event: Event::Read {
            handle: reads::handle(runner, reading.called.call_number, SLOT),
            read: Readable::Family {
                scope: c::Scope::Deployment,
                window: c::Window { rows: 1, after: None },
                record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::reading(&record) },
            },
        },
    });
}
pub(super) fn next(reading: &reads::Reading) -> Option<w::Read> {
    let (_, review) = super::proposal::review(reading.pages.get(&1)?)?;
    let key = review.key?;
    let Some(items) = &reading.kinds.items else {
        return Some(w::Read::ProposalItems(
            w::ProposalItemsRead::new(&w::CEILINGS, w::ProposalItemsReadParts { key }).expect("fixed read"),
        ));
    };
    for item in items.items() {
        if !reading.kinds.current.contains_key(&(item.kind(), item.scope(), item.number())) {
            return Some(setting_query(item.kind(), item.scope(), item.number()));
        }
    }
    None
}
fn malformed() -> Readout {
    Readout::Jig(c::ReadResult::None(c::ReadNone::Malformed))
}
fn bytes(page: &Readout) -> Option<&[u8]> {
    match page {
        Readout::Family { family: wire::FORGE, record: Some(bytes) } => Some(bytes),
        Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => None,
    }
}
fn valid(kind: u32, bytes: &[u8]) -> bool {
    let mut reader = Reader::new(bytes);
    let decoded = match kind {
        1 | 2 => w::LandingRule::decode(&w::CEILINGS, &mut reader).is_ok(),
        3 => w::ForgeCharter::decode(&w::CEILINGS, &mut reader).is_ok(),
        _ => false,
    };
    decoded && reader.is_empty()
}
fn setting(page: &Readout, kind: u32) -> Result<Option<c::SettingValue>, ()> {
    let record = wire::decode_setting_answer(bytes(page).ok_or(())?, &w::CEILINGS).ok_or(())?;
    match record.into_parts().value {
        Some(bytes) if valid(kind, &bytes) => Ok(Some(c::SettingValue::Application(bytes))),
        Some(_) => Err(()),
        None => Ok(None),
    }
}
fn proposal_items(page: &Readout) -> Option<w::ProposalItemsAnswer> {
    let record = wire::decode_proposal_items_answer(bytes(page)?, &w::CEILINGS)?;
    let mut keys = BTreeSet::new();
    for item in record.items() {
        if !keys.insert((item.kind(), item.scope(), item.number())) {
            return None;
        }
        if let w::SettingEdit::Set(set) = item.edit()
            && !valid(item.kind(), set.value())
        {
            return None;
        }
    }
    Some(record)
}
pub(super) fn received(reading: &mut reads::Reading, page: Readout) -> Result<(), Readout> {
    let query = reading.kinds.query.take().expect("one forge read in flight");
    match query {
        w::Read::Setting(query) => {
            let decoded = setting(&page, query.kind());
            if reading.called.call.tool == rows::SETTING_AT {
                let Ok(value) = decoded else { return Err(refusal(page)) };
                let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(document)))) =
                    reading.pages.get_mut(&1)
                else {
                    unreachable!("inventory gave the setting state")
                };
                document.value = value;
            } else {
                let page = match decoded {
                    Ok(value) => {
                        Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
                            setting: c::Setting::Application {
                                kind: query.kind(),
                                scope: query.scope(),
                                number: query.number(),
                            },
                            revision: 0,
                            value,
                            application: None,
                            proposed: None,
                        }))))
                    }
                    Err(()) => refusal(page),
                };
                reading.kinds.current.insert((query.kind(), query.scope(), query.number()), page);
            }
        }
        w::Read::ProposalItems(_) => {
            let Some(items) = proposal_items(&page) else { return Err(refusal(page)) };
            reading.kinds.items = Some(items);
        }
        w::Read::Playground => unreachable!("settings reads select a setting or proposal"),
    }
    Ok(())
}
fn refusal(page: Readout) -> Readout {
    match page {
        Readout::Family { record: None, .. } => Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
        Readout::Family { .. } | Readout::Jig(c::ReadResult::Value(_)) => malformed(),
        page @ (Readout::Jig(c::ReadResult::None(_)) | Readout::Limit | Readout::NotServed { .. }) => page,
    }
}

pub(super) fn inventory(reading: &mut reads::Reading, settings: &c::SettingsDocument, setting: c::Setting) {
    let mut application = None;
    for line in &settings.settings {
        if line.setting == setting {
            application.clone_from(&line.application);
            break;
        }
    }
    reading.pages.insert(
        1,
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
            setting,
            revision: settings.newest,
            value: None,
            application,
            proposed: None,
        })))),
    );
}
