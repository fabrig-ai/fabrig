//! Application settings reads through the host's family hooks (client/tools.md, 4).
use super::{Out, Runner, reads};
use crate::jig::rows;
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Event, Readout, contract as c};
use skein_lib::{List, Queue};

pub(super) const SLOT: u32 = 7;
/// A host builds and decodes its own family records for these reads.
#[derive(Clone, Copy, Debug)]
pub enum Query {
    Setting(c::Setting),
    ProposalItems { key: u64 },
}
#[derive(Debug)]
pub enum Answer {
    Setting(Option<c::SettingValue>),
    ProposalItems(List<c::SettingItem>),
}
#[derive(Debug)]
pub(crate) struct Reading {
    pub items: Option<List<c::SettingItem>>,
    pub current: BTreeMap<(u32, u32, u32), Readout>,
    pub query: Option<Query>,
}
impl Reading {
    pub(super) fn new() -> Self {
        Self { items: None, current: BTreeMap::new(), query: None }
    }
}
pub(super) fn read(runner: &Runner, reading: &mut reads::Reading, query: Query, out: &mut Queue<Out>) {
    reading.pending |= 1 << SLOT;
    reading.application.query = Some(query);
    out.push(Out::Client {
        event: Event::Read {
            handle: reads::handle(runner, reading.called.call_number, SLOT),
            read: reading.parts.application_read(query),
        },
    });
}
pub(super) fn next(reading: &reads::Reading) -> Option<Query> {
    let (_, review) = super::proposal::review(reading.pages.get(&1)?)?;
    let proposal = review.key?;
    let Some(items) = &reading.application.items else {
        return Some(Query::ProposalItems { key: proposal });
    };
    for item in items {
        if !reading.application.current.contains_key(&key(item.setting)) {
            return Some(Query::Setting(item.setting));
        }
    }
    None
}
fn malformed() -> Readout {
    Readout::Jig(c::ReadResult::None(c::ReadNone::Malformed))
}
pub(super) fn received(reading: &mut reads::Reading, page: Readout) -> Result<(), Readout> {
    let query = reading.application.query.take().expect("one application read in flight");
    match query {
        Query::Setting(setting) => {
            let decoded = match reading.parts.application_answer(query, &page) {
                Some(Answer::Setting(value)) => Ok(value),
                Some(Answer::ProposalItems(_)) | None => Err(()),
            };
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
                            setting,
                            revision: 0,
                            value,
                            application: None,
                            proposed: None,
                        }))))
                    }
                    Err(()) => refusal(page),
                };
                reading.application.current.insert(key(setting), page);
            }
        }
        Query::ProposalItems { .. } => {
            let Some(Answer::ProposalItems(items)) = reading.parts.application_answer(query, &page) else {
                return Err(refusal(page));
            };
            reading.application.items = Some(items);
        }
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

pub(crate) fn key(setting: c::Setting) -> (u32, u32, u32) {
    let c::Setting::Application { kind, scope, number } = setting else { unreachable!("application setting") };
    (kind, scope, number)
}
