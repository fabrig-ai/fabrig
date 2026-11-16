use alloc::boxed::Box;
use jig_domain_core_vocabulary as domain;

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
