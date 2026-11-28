//! The open-time name table (jig/docs/design/protocol/store/records.md, 1).
use alloc::boxed::Box;
use jig_domain_core_vocabulary::settings::{Key, ModelName, Record, Setting, Value};
use skein_lib::List;

/// Model values carry no indices, so their changes precede every restore.
pub(crate) fn catalogue(store: &mut crate::Store) -> Result<(), crate::restore::Failure> {
    use crate::restore::{Cause, DecodeFailure, Failure};

    let mut names = List::with_capacity(jig_domain_tasks::MODELS);
    for name in &store.names {
        names.push(name.clone()).expect("given catalogue fits the model bound");
    }
    let given = names.len();
    let mut overlaid = 0_u64;
    let mut shadowed = 0_u64;
    let mut range = crate::keys::model_changes();
    for _ in 0..=jig_domain_tasks::MODELS {
        let page = crate::pages::reader::load_map(store, crate::KeptMap::Store, range.clone())?;
        let mut rows = page.rows.into_iter();
        let Some(row) = rows.next() else {
            // A stored change wins at its own number; only given entries become holes.
            let omitted = shadowed & !overlaid;
            for at in 0..given {
                if omitted & bit(at) != 0 {
                    *names.get_mut(at).expect("given catalogue position") = hole();
                }
            }
            store.catalogue(names.into_boxed());
            return Ok(());
        };
        let site = crate::restore::site(store, &row.key);
        let number =
            crate::keys::changed_model(&row.key).ok_or(Failure::at(Cause::Decode(DecodeFailure::Key), site.clone()))?;
        let key = Key::Changed(Setting::Model(number));
        let Ok(record) = crate::codec::settings::decode(&key, &row.value, &store.codec_limits.settings, &[]) else {
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
                    | Value::Provider(_)
                    | Value::Limits(_)
                    | Value::Worker(_)
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
        for (at, given) in store.names.iter().enumerate() {
            if given.name == model.model.name {
                shadowed |= bit(u32::try_from(at).expect("given model index"));
            }
        }
        for _ in names.len()..number {
            names.push(hole()).expect("model number bounded");
        }
        let index = number.checked_sub(1).expect("nonzero model number");
        overlaid |= bit(index);
        *names.get_mut(index).expect("catalogue extended to model number") =
            ModelName { name: model.model.name, efforts: model.model.efforts };
        range.start = crate::pages::reader::past(&row.key)?;
    }
    Err(Failure::rowless(Cause::Decode(DecodeFailure::Key)))
}

fn bit(index: u32) -> u64 {
    1_u64.checked_shl(index).expect("model index fits the catalogue")
}

fn hole() -> ModelName {
    ModelName { name: Box::new([]), efforts: Box::new([]) }
}
