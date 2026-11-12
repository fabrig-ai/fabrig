//! Domain records translated to generated store families (protocol/store/records.md, 1).
pub mod accounts;
pub mod component;
pub mod core;
pub mod notes;
pub mod people;
mod pieces;
pub mod settings;
pub mod tasks;
use alloc::boxed::Box;
use jig_domain_core as domain;

/// Generated family bounds derived from the component's limits (protocol/store/README.md, 8).
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub settings: crate::settings_v3::Limits,
    pub core: crate::core_v6::Limits,
    pub tasks: crate::tasks_v10::Limits,
    pub people: crate::people_v15::Limits,
    pub notes: crate::notes_v1::Limits,
}

#[must_use]
pub fn limits(domain: &domain::Limits) -> Option<View> {
    let tasks = tasks::limits(&domain.tasks)?;
    let notes = notes::limits(&domain.notes)?;
    let core = core::limits(domain, &tasks, &notes)?;
    Some(View { core, tasks, notes, settings: settings::limits(domain)?, people: people::limits(domain)? })
}

/// The catalogue's names in number order: settings and the people family keep
/// indexed models by name (protocol/store/records.md, 1).
#[must_use]
pub fn names(models: &[domain::Model]) -> Box<[domain::settings::ModelName]> {
    domain::settings::model_names(models)
}
/// Encode a record through its owning family; `names` are [`names`] of today's catalogue.
#[must_use]
pub fn encode(row: &domain::Record, limits: &View, names: &[domain::settings::ModelName]) -> Option<Box<[u8]>> {
    match row {
        domain::Record::Core(domain::CoreRecord::Settings(row)) => settings::encode(row, &limits.settings, names).ok(),
        domain::Record::Core(row) => core::encode(row, limits).ok(),
        domain::Record::Tasks(row) => tasks::encode(row, &limits.tasks, names).ok(),
        domain::Record::People(row) => people::encode(row, &limits.people, names).ok(),
        domain::Record::Notes(row) => notes::encode(row, &limits.notes).ok(),
        domain::Record::Accounts(row) => accounts::encode(row).ok(),
    }
}

/// The key's child selects the one decoder for its value; `names` are [`names`] of today's
/// catalogue.
#[must_use]
pub fn decode(
    key: &domain::Key,
    bytes: &[u8],
    limits: &View,
    names: &[domain::settings::ModelName],
) -> Option<domain::Record> {
    match key {
        domain::Key::Core(domain::CoreKey::Settings(key)) => Some(domain::Record::Core(domain::CoreRecord::Settings(
            settings::decode(key, bytes, &limits.settings, names).ok()?,
        ))),
        domain::Key::Core(_) => Some(domain::Record::Core(core::decode(bytes, limits).ok()?)),
        domain::Key::Tasks(_) => Some(domain::Record::Tasks(tasks::decode(bytes, &limits.tasks, names).ok()?)),
        domain::Key::People(_) => Some(domain::Record::People(people::decode(bytes, &limits.people, names).ok()?)),
        domain::Key::Notes(_) => Some(domain::Record::Notes(notes::decode(bytes, &limits.notes).ok()?)),
        domain::Key::Accounts(_) => Some(domain::Record::Accounts(accounts::decode(bytes).ok()?)),
    }
}

/// Model values carry no indices, so their changes precede every restore (protocol/store/records.md).
pub(crate) fn catalogue(store: &mut crate::Store) -> Result<(), crate::restore::Failure> {
    use crate::restore::{Cause, DecodeFailure, Failure};
    use domain::settings::{Key, ModelName, Record, Setting, Value};
    use skein_lib::List;

    let mut names = List::with_capacity(jig_domain_tasks::MODELS);
    for name in &store.names {
        names.push(name.clone()).expect("given catalogue fits the model bound");
    }
    let mut range = crate::keys::model_changes();
    for _ in 0..=jig_domain_tasks::MODELS {
        let page = crate::pages::reader::load_map(store, crate::KeptMap::Store, range.clone())?;
        let mut rows = page.rows.into_iter();
        let Some(row) = rows.next() else {
            store.catalogue(names.into_boxed());
            return Ok(());
        };
        let site = crate::restore::site(store, &row.key);
        let number =
            crate::keys::changed_model(&row.key).ok_or(Failure::at(Cause::Decode(DecodeFailure::Key), site.clone()))?;
        let key = Key::Changed(Setting::Model(number));
        let Ok(record) = settings::decode(&key, &row.value, &store.codec_limits.settings, &[]) else {
            return Err(Failure::at(Cause::Decode(DecodeFailure::Value), site));
        };
        let model = match record {
            Record::Changed { changed, .. } => match changed.value {
                Some(Value::Model(model)) => model,
                Some(
                    Value::Overlay(_)
                    | Value::Charter(_)
                    | Value::Template(_)
                    | Value::Policy(_)
                    | Value::Rules(_)
                    | Value::Application(_),
                )
                | None => return Err(Failure::at(Cause::Decode(DecodeFailure::KeyValueDisagree), site)),
            },
            Record::Default { .. }
            | Record::Revision { .. }
            | Record::Charter { .. }
            | Record::Proposal { .. }
            | Record::Application { .. }
            | Record::Extent { .. } => {
                return Err(Failure::at(Cause::Decode(DecodeFailure::KeyValueDisagree), site));
            }
        };
        if model.number != number || model.model.name.is_empty() {
            return Err(Failure::at(Cause::Decode(DecodeFailure::KeyValueDisagree), site));
        }
        for _ in names.len()..number {
            names.push(ModelName { name: Box::new([]), efforts: Box::new([]) }).expect("model number bounded");
        }
        let index = number.checked_sub(1).expect("nonzero model number");
        *names.get_mut(index).expect("catalogue extended to model number") =
            ModelName { name: model.model.name, efforts: model.model.efforts };
        range.start = crate::pages::reader::past(&row.key)?;
    }
    Err(Failure::rowless(Cause::Decode(DecodeFailure::Key)))
}
