//! The model setting's value and shape (jig/docs/design/domain/engine.md, 17.1 and 17.11).
use super::{Current, Edit, Fault, Item, Setting};
use crate::{Model, Settings};
use alloc::boxed::Box;
use skein_lib::List;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ModelSource {
    Stock,
    List,
    Person,
}

/// One whole model setting (jig/docs/design/domain/engine.md, 17.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CatalogueModel {
    pub number: u32,
    pub model: Model,
    pub shown: Option<Box<[u8]>>,
    pub description: Option<Box<[u8]>>,
    pub picker: bool,
    pub withdrawn: bool,
    pub source: ModelSource,
    pub estimated: bool,
}

#[must_use]
pub fn valid_name(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > skein_lib::length::usize_of(crate::MODEL_NAME_BYTES) {
        return false;
    }
    for &byte in name {
        if !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'.' && byte != b'_' && byte != b':' {
            return false;
        }
    }
    true
}

pub(crate) fn fits(value: &CatalogueModel, number: u32) -> bool {
    let model = &value.model;
    if number == 0
        || number > jig_domain_tasks::MODELS
        || value.number != number
        || !valid_name(&model.name)
        || model.window == 0
        || model.output == 0
        || model.price_unit == 0
        || model.efforts.len() > skein_lib::length::usize_of(crate::EFFORTS)
    {
        return false;
    }
    if let Some(shown) = &value.shown
        && (shown.is_empty() || shown.len() > skein_lib::length::usize_of(crate::MODEL_NAME_BYTES))
    {
        return false;
    }
    if let Some(description) = &value.description
        && description.len() > skein_lib::length::usize_of(crate::MODEL_DESCRIPTION_BYTES)
    {
        return false;
    }
    if let Some(program) = &model.program
        && (program.is_empty() || program.len() > skein_lib::length::usize_of(jig_domain_accounts::NAME))
    {
        return false;
    }
    for (at, effort) in model.efforts.iter().enumerate() {
        if effort.is_empty() || effort.len() > skein_lib::length::usize_of(crate::EFFORT_BYTES) {
            return false;
        }
        for previous in model.efforts.iter().take(at) {
            if previous == effort {
                return false;
            }
        }
    }
    match &model.effort {
        Some(effort) => model.efforts.contains(effort),
        None => true,
    }
}

pub(crate) fn shape(value: &CatalogueModel, number: u32, settings: &Settings) -> Result<(), Fault> {
    if !fits(value, number) {
        return Err(Fault::Shape);
    }
    if settings.llm_providers.get(value.model.provider).is_none() {
        return Err(Fault::Reference);
    }
    for endpoint in &settings.endpoints {
        if endpoint.number == value.model.endpoint {
            return if endpoint.dialect == value.model.dialect && endpoint.program == value.model.program {
                Ok(())
            } else {
                Err(Fault::Shape)
            };
        }
    }
    Err(Fault::Reference)
}

pub(crate) fn bound() -> Option<u64> {
    u64::try_from(size_of::<CatalogueModel>())
        .ok()?
        .checked_add(u64::from(crate::MODEL_NAME_BYTES).checked_mul(2)?)?
        .checked_add(u64::from(crate::MODEL_DESCRIPTION_BYTES))?
        .checked_add(u64::from(jig_domain_accounts::NAME))?
        .checked_add(
            u64::from(crate::EFFORTS).checked_mul(
                u64::from(crate::EFFORT_BYTES).checked_add(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
            )?,
        )?
        .checked_add(u64::from(crate::EFFORT_BYTES))
}

/// The catalogue the current settings leave, one row per number up to the highest (17.11).
pub(crate) fn table(current: &Current) -> Box<[Model]> {
    rows(&entries(current))
}

pub(crate) fn rows(models: &[CatalogueModel]) -> Box<[Model]> {
    let mut highest = 0;
    for model in models {
        highest = highest.max(model.number);
    }
    let mut rows = List::with_capacity(highest);
    for number in 1..=highest {
        let mut row = Model::hole();
        for model in models {
            if model.number == number {
                row = model.model.clone();
                break;
            }
        }
        rows.push(row).expect("catalogue number bound");
    }
    rows.into_boxed()
}

pub(crate) fn entries(current: &Current) -> Box<[CatalogueModel]> {
    overlay(current, false, &[])
}

/// Select models through the same default/change overlay as the settings check.
pub(crate) fn overlay(current: &Current, defaults: bool, items: &[Item]) -> Box<[CatalogueModel]> {
    let mut models = List::with_capacity(jig_domain_tasks::MODELS);
    for number in 1..=jig_domain_tasks::MODELS {
        let setting = Setting::Model(number);
        let before = if defaults { current.default_value(setting) } else { current.value(setting) };
        let mut withdrawn = false;
        let value = match super::check::edit(items, setting) {
            Some(Edit::Set(value)) => Some(value),
            Some(Edit::Removed) => {
                withdrawn = true;
                before
            }
            Some(Edit::Reset) => current.default_value(setting),
            None => before,
        };
        if let Some(value) = value
            && let Some(value) = value.as_model()
        {
            let mut value = value.clone();
            value.withdrawn |= withdrawn;
            models.push(value).expect("catalogue number bound");
        }
    }
    models.into_boxed()
}

/// Worlds without numbered defaults take their initial rows as stock (17.7).
pub(crate) fn defaults(settings: &Settings) -> Box<[CatalogueModel]> {
    let mut models = List::with_capacity(jig_domain_tasks::MODELS);
    for (at, model) in settings.models.iter().enumerate() {
        if model.is_hole() {
            continue;
        }
        models
            .push(CatalogueModel {
                number: u32::try_from(at).expect("model position").checked_add(1).expect("model number"),
                model: model.clone(),
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: ModelSource::Stock,
                estimated: false,
            })
            .expect("catalogue number bound");
    }
    models.into_boxed()
}
