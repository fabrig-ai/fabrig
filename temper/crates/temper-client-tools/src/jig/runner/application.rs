//! Application settings reads through the host's family hooks (client/ctl/README.md, 4).
use super::{Out, Runner, reads};
use crate::jig::rows;
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Event, Readout, contract as c};
use skein_lib::{List, Map, Queue};

pub(super) const SLOT: u32 = 7;
/// A host builds and decodes its own family records for these reads.
#[derive(Clone, Copy, Debug)]
pub enum Query {
    Setting(c::Setting),
    ProposalItems {
        key: u64,
    },
    /// Read the explicit value set in this revision, rather than the current value.
    RevisionSetting {
        setting: c::Setting,
        revision: u32,
    },
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
    /// The preceding explicit set revision, or unavailable history, per setting.
    pub previous: Map<(u32, u32, u32), Option<u32>>,
    /// The setting and earlier revision being inspected.
    pub seeking: Option<(c::Setting, u32)>,
}
impl Reading {
    pub(super) fn new() -> Self {
        Self { items: None, current: BTreeMap::new(), query: None, previous: Map::uncapped(), seeking: None }
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
    if reading.called.call.tool == rows::REVISION_AT {
        return next_revision(reading);
    }
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
fn next_revision(reading: &reads::Reading) -> Option<Query> {
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Revision(answer))) = reading.pages.get(&1)? else {
        return None;
    };
    let c::RevisionAnswer::Revision(document) = answer.as_ref() else { return None };
    for item in &document.items {
        if let c::Setting::Application { .. } = item.setting
            && let c::RevisionEdit::Application(c::SizedEdit::Set { .. }) = item.edit
        {
            return Some(Query::RevisionSetting { setting: item.setting, revision: document.head.number });
        }
        if let c::RevisionReplaced::Application { .. } = item.replaced
            && let Some(Some(revision)) = reading.application.previous.get(&key(item.setting))
        {
            return Some(Query::RevisionSetting { setting: item.setting, revision: *revision });
        }
    }
    None
}

/// Find the preceding explicit set through retained records, without reading today's value.
pub(super) fn next_history(reading: &mut reads::Reading) -> Option<c::Read> {
    if reading.called.call.tool != rows::REVISION_AT {
        return None;
    }
    if reading.application.seeking.is_none() {
        let Readout::Jig(c::ReadResult::Value(c::ReadValue::Revision(answer))) = reading.pages.get(&1)? else {
            return None;
        };
        let c::RevisionAnswer::Revision(document) = answer.as_ref() else { return None };
        for item in &document.items {
            if let c::RevisionReplaced::Application { .. } = item.replaced
                && !reading.application.previous.contains_key(&key(item.setting))
            {
                if let Some(number) = document.head.number.checked_sub(1) {
                    reading.application.seeking = Some((item.setting, number));
                    break;
                }
                reading
                    .application
                    .previous
                    .insert(key(item.setting), None)
                    .expect("one correlation per revision item");
            }
        }
    }
    let (_, number) = reading.application.seeking?;
    Some(c::Read::Revision(c::ReadRevision { number }))
}

/// Advance a history search only through matching retained revision records.
pub(super) fn history_received(reading: &mut reads::Reading, page: &Readout) {
    let (setting, number) = reading.application.seeking.take().expect("history read in flight");
    reading.application.previous.insert(key(setting), None).expect("one correlation per revision item");
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Revision(answer))) = page else { return };
    let c::RevisionAnswer::Revision(document) = answer.as_ref() else { return };
    if document.head.number != number {
        return;
    }
    for item in &document.items {
        if item.setting == setting {
            if let c::RevisionEdit::Application(c::SizedEdit::Set { .. }) = item.edit {
                reading
                    .application
                    .previous
                    .insert(key(setting), Some(number))
                    .expect("one correlation per revision item");
            }
            return;
        }
    }
    if let Some(before) = number.checked_sub(1) {
        reading.application.seeking = Some((setting, before));
    }
}
fn revision_received(
    reading: &mut reads::Reading,
    query: Query,
    page: Readout,
    limits: &crate::jig::Limits,
) -> Result<(), Readout> {
    let Query::RevisionSetting { setting, revision } = query else { unreachable!("revision query") };
    let value = match reading.parts.application_answer(query, &page) {
        Some(Answer::Setting(Some(value))) => value,
        Some(Answer::Setting(None)) => return Err(Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown))),
        Some(Answer::ProposalItems(_)) | None => return Err(refusal(page)),
    };
    if reading.called.call.tool == rows::SETTING_AT {
        let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(document)))) = reading.pages.get_mut(&1)
        else {
            unreachable!("historical setting header")
        };
        document.value = Some(value);
        return Ok(());
    }
    join_revision(reading, setting, revision, value, limits);
    Ok(())
}
fn join_revision(
    reading: &mut reads::Reading,
    setting: c::Setting,
    revision: u32,
    value: c::SettingValue,
    limits: &crate::jig::Limits,
) {
    let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Revision(answer)))) = reading.pages.get_mut(&1) else {
        unreachable!("revision metadata precedes its values")
    };
    let c::RevisionAnswer::Revision(document) = answer.as_mut() else { unreachable!("kept revision") };
    let maximum = limits
        .result_bytes
        .checked_div(4)
        .expect("nonzero divisor")
        .checked_div(u64::from(document.items.len().max(1)))
        .expect("at least one revision item");
    let edit = revision_edit(setting, value, maximum, reading.parts);
    for index in 0..document.items.len() {
        let item = document.items.get_mut(index).expect("counted revision item");
        if item.setting == setting {
            if revision == document.head.number {
                item.edit = edit;
            } else {
                item.replaced = match edit {
                    c::RevisionEdit::Set(value) => c::RevisionReplaced::Value(value),
                    c::RevisionEdit::TooLarge(size) => c::RevisionReplaced::TooLarge(size),
                    c::RevisionEdit::Application(_) | c::RevisionEdit::Removed | c::RevisionEdit::Reset => {
                        unreachable!("measured set")
                    }
                };
            }
            break;
        }
    }
}
fn revision_edit(
    setting: c::Setting,
    value: c::SettingValue,
    maximum: u64,
    parts: &dyn crate::jig::render::Parts,
) -> c::RevisionEdit {
    let bounds = skein_json::writer::Limits { depth: 32, length: u32::try_from(maximum).unwrap_or(u32::MAX) };
    let mut measure = skein_json::writer::Encoder::measure(&bounds);
    let (c::Setting::Application { kind, .. }, c::SettingValue::Application(bytes)) = (setting, &value) else {
        unreachable!("decoded application value")
    };
    let fits = parts.setting_value(kind, bytes, &mut measure) && measure.measured().is_ok();
    let size = u64::try_from(bytes.len()).expect("bounded value");
    if fits {
        c::RevisionEdit::Set(c::SettingSet { value })
    } else {
        c::RevisionEdit::TooLarge(c::Size { bytes: size })
    }
}

fn malformed() -> Readout {
    Readout::Jig(c::ReadResult::None(c::ReadNone::Malformed))
}
pub(super) fn received(
    reading: &mut reads::Reading,
    page: Readout,
    limits: &crate::jig::Limits,
) -> Result<(), Readout> {
    let query = reading.application.query.take().expect("one application read in flight");
    if let Query::RevisionSetting { setting, revision } = query {
        let result = revision_received(reading, query, page, limits);
        if result.is_err() && reading.application.previous.get(&key(setting)) == Some(&Some(revision)) {
            reading.application.previous.insert(key(setting), None).expect("one correlation per revision item");
            return Ok(());
        }
        return result;
    }
    match query {
        Query::RevisionSetting { .. } => unreachable!("revision delivered separately"),
        Query::Setting(setting) => return setting_received(reading, query, setting, page),
        Query::ProposalItems { .. } => {
            let Some(Answer::ProposalItems(items)) = reading.parts.application_answer(query, &page) else {
                return Err(refusal(page));
            };
            reading.application.items = Some(items);
        }
    }
    Ok(())
}
fn setting_received(
    reading: &mut reads::Reading,
    query: Query,
    setting: c::Setting,
    page: Readout,
) -> Result<(), Readout> {
    let decoded = match reading.parts.application_answer(query, &page) {
        Some(Answer::Setting(value)) => Ok(value),
        Some(Answer::ProposalItems(_)) | None => Err(()),
    };
    if reading.called.call.tool == rows::SETTING_AT {
        let Ok(value) = decoded else { return Err(refusal(page)) };
        let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(document)))) = reading.pages.get_mut(&1)
        else {
            unreachable!("inventory gave the setting state")
        };
        document.value = value;
    } else {
        let page = match decoded {
            Ok(value) => Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
                overlay: None,
                setting,
                revision: 0,
                value,
                application: None,
                proposed: None,
            })))),
            Err(()) => refusal(page),
        };
        reading.application.current.insert(key(setting), page);
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
            overlay: None,
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

pub(super) fn historical(
    runner: &Runner,
    reading: &mut reads::Reading,
    setting: c::Setting,
    revision: u32,
    out: &mut Queue<Out>,
) {
    reading.pages.insert(
        1,
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
            overlay: None,
            setting,
            revision,
            value: None,
            application: None,
            proposed: None,
        })))),
    );
    read(runner, reading, Query::RevisionSetting { setting, revision }, out);
}
