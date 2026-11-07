//! The model setting's value and shape (jig/docs/design/domain/engine.md, 17.1 and 17.11).
use super::Fault;
use crate::{Model, Settings};
use alloc::boxed::Box;

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
