//! The model setting's value and shape (jig/docs/design/domain/engine/settings.md, 1 and 11–12).
use super::{Current, Edit, Fault, Item, Setting, Value};
use crate::{Model, Settings};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_link as link;
use skein_lib::{Duration, List, Wall};

/// How long a list model is new (jig/docs/design/domain/engine/settings.md, 12).
pub const NEW_FOR: Duration = Duration::from_secs(7 * 86_400);

/// One good read's setting edits and account facts (domain/engine/settings.md, 12).
#[derive(Debug)]
pub struct ListChange {
    pub items: Box<[Item]>,
    pub names: u64,
    pub left_out: u32,
}

#[must_use]
pub fn from_list(
    provider: u32,
    list: &accounts::ProviderList,
    rows: &[link::ListedModel],
    others: u64,
    at: Wall,
    models: &[CatalogueModel],
    endpoints: &[crate::EndpointFacts],
) -> ListChange {
    let mut working = List::with_capacity(jig_domain_tasks::MODELS);
    for model in models {
        working.push(model.clone()).expect("catalogue number bound");
    }
    let mut items = List::with_capacity(jig_domain_tasks::MODELS);
    let mut names = 0;
    let mut left_out = 0_u32;
    for row in rows {
        if !valid_name(&row.name) || !kept(&row.name, list) {
            continue;
        }
        let mut known = false;
        for model in working.iter_mut() {
            if model.model.name != row.name {
                continue;
            }
            known = true;
            if model.model.provider == provider && (!model.withdrawn || model.source == ModelSource::List) {
                names |= name_bit(model.number);
                if model.withdrawn {
                    model.withdrawn = false;
                    items.push(list_item(model.clone())).expect("one edit per catalogue number");
                }
            }
            break;
        }
        if known {
            continue;
        }
        let Some(mut model) = new_model(provider, row.name.clone(), ModelSource::List, working.as_slice(), endpoints)
        else {
            left_out = left_out.checked_add(1).expect("bounded list rows");
            continue;
        };
        model.picker = false;
        model.first_read = Some(at);
        listed_figures(&mut model, row);
        names |= name_bit(model.number);
        items.push(list_item(model.clone())).expect("one edit per catalogue number");
        working.push(model).expect("new model found a free number");
    }
    for model in models {
        if model.model.provider == provider
            && model.source == ModelSource::List
            && !model.withdrawn
            && (names | others) & name_bit(model.number) == 0
        {
            let mut gone = model.clone();
            gone.withdrawn = true;
            items.push(list_item(gone)).expect("one edit per catalogue number");
        }
    }
    ListChange { items: items.into_boxed(), names, left_out }
}

fn name_bit(number: u32) -> u64 {
    1_u64.checked_shl(number.checked_sub(1).expect("model numbers start at one")).expect("model number at most 64")
}

fn list_item(model: CatalogueModel) -> Item {
    Item { setting: Setting::Model(model.number), edit: Edit::Set(Value::Model(Box::new(model))) }
}

fn listed_figures(model: &mut CatalogueModel, row: &link::ListedModel) {
    if let Some(shown) = &row.shown
        && !shown.is_empty()
        && shown.len() <= skein_lib::length::usize_of(crate::MODEL_NAME_BYTES)
    {
        model.shown = Some(shown.clone());
    }
    if let Some(window) = row.window
        && window != 0
    {
        model.model.window = model.model.window.min(window);
    }
    if let Some(output) = row.output
        && output != 0
    {
        model.model.output = model.model.output.min(output);
    }
    let mut efforts = List::with_capacity(crate::EFFORTS);
    for effort in &row.efforts {
        if !effort.is_empty()
            && effort.len() <= skein_lib::length::usize_of(crate::EFFORT_BYTES)
            && !efforts.as_slice().contains(effort)
            && efforts.room() != 0
        {
            efforts.push(effort.clone()).expect("effort room checked");
        }
    }
    model.model.effort = efforts.as_slice().first().cloned();
    for effort in &efforts {
        if effort.as_ref() == b"high" {
            model.model.effort = Some(effort.clone());
            break;
        }
    }
    model.model.efforts = efforts.into_boxed();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ModelSource {
    Stock,
    List,
    Person,
}

/// One whole model setting (jig/docs/design/domain/engine/settings.md, 1).
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
    pub first_read: Option<Wall>,
}

#[must_use]
pub fn valid_name(name: &[u8]) -> bool {
    const NAME_BYTES: usize = 128;
    if name.is_empty() || name.len() > NAME_BYTES {
        return false;
    }
    for &byte in name {
        if !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'.' && byte != b'_' && byte != b':' {
            return false;
        }
    }
    true
}

/// Whole-name byte glob matching (jig/docs/design/protocol/configuration.md, 3).
#[must_use]
pub fn matches(pattern: &[u8], name: &[u8]) -> bool {
    let mut at = 0_usize;
    let mut next = 0_usize;
    let mut star = None;
    let mut retry = 0_usize;
    // Between retries the pattern advances; each retry consumes one more name byte.
    let steps = pattern.len().saturating_add(1).saturating_mul(name.len().saturating_add(1));
    for _ in 0..steps {
        if pattern.get(at) == Some(&b'*') {
            at = at.checked_add(1).expect("pattern position");
            star = Some(at);
            retry = next;
        } else if let Some(&byte) = name.get(next) {
            if pattern.get(at) == Some(&b'?') || pattern.get(at) == Some(&byte) {
                at = at.checked_add(1).expect("pattern position");
                next = next.checked_add(1).expect("name position");
            } else if let Some(after) = star {
                retry = retry.checked_add(1).expect("retry position");
                next = retry;
                at = after;
            } else {
                return false;
            }
        } else {
            return at == pattern.len();
        }
    }
    false
}

/// A drop wins over any keep (jig/docs/design/domain/accounts.md, 3.7).
#[must_use]
pub fn kept(name: &[u8], list: &jig_domain_accounts::ProviderList) -> bool {
    for pattern in &list.drop {
        if matches(pattern, name) {
            return false;
        }
    }
    for pattern in &list.keep {
        if matches(pattern, name) {
            return true;
        }
    }
    false
}

#[must_use]
pub fn fits(value: &CatalogueModel, number: u32) -> bool {
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

pub fn shape(value: &CatalogueModel, number: u32, settings: &Settings) -> Result<(), Fault> {
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

#[must_use]
pub fn bound() -> Option<u64> {
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
#[must_use]
pub fn table(current: &Current) -> Box<[Model]> {
    rows(&entries(current))
}

#[must_use]
pub fn rows(models: &[CatalogueModel]) -> Box<[Model]> {
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

#[must_use]
pub fn entries(current: &Current) -> Box<[CatalogueModel]> {
    overlay(current, false, &[])
}

/// Select models through the same default/change overlay as the settings check.
#[must_use]
pub fn overlay(current: &Current, defaults: bool, items: &[Item]) -> Box<[CatalogueModel]> {
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
#[must_use]
pub fn defaults(settings: &Settings) -> Box<[CatalogueModel]> {
    let mut models = List::with_capacity(jig_domain_tasks::MODELS);
    for (at, model) in settings.models.iter().enumerate() {
        if model.is_hole() {
            continue;
        }
        models
            .push(CatalogueModel {
                first_read: None,
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

/// An owner's act, independent of any client revision (domain/engine/settings.md, 11).
#[derive(Clone, Hash, Debug)]
pub enum ModelAct {
    Picker { number: u32, on: bool },
    Add { provider: u32, name: Box<[u8]> },
}

/// The highest number not held by any setting, withdrawn rows included.
#[must_use]
pub fn free_number(models: &[CatalogueModel]) -> Option<u32> {
    for number in (1..=jig_domain_tasks::MODELS).rev() {
        let mut held = false;
        for row in models {
            held |= row.number == number;
        }
        if !held {
            return Some(number);
        }
    }
    None
}

/// A new estimated model takes stock figures and its provider's endpoint (domain/engine/settings.md, 11).
#[must_use]
pub fn new_model(
    provider: u32,
    name: Box<[u8]>,
    source: ModelSource,
    models: &[CatalogueModel],
    endpoints: &[crate::EndpointFacts],
) -> Option<CatalogueModel> {
    let mut own: Option<&CatalogueModel> = None;
    let mut any: Option<&CatalogueModel> = None;
    for row in models {
        if row.source != ModelSource::Stock {
            continue;
        }
        if dearer(row, any) {
            any = Some(row);
        }
        if row.model.provider == provider && dearer(row, own) {
            own = Some(row);
        }
    }
    let mut model = own.or(any)?.model.clone();
    if own.is_none() {
        let mut endpoint = None;
        for facts in endpoints {
            if facts.providers.contains(&provider) {
                endpoint = Some(facts);
                break;
            }
        }
        let endpoint = endpoint?;
        model.endpoint = endpoint.number;
        model.dialect = endpoint.dialect;
        model.program.clone_from(&endpoint.program);
    }
    model.provider = provider;
    model.name = name;
    model.efforts = Box::default();
    model.effort = None;
    Some(CatalogueModel {
        first_read: None,
        number: free_number(models)?,
        model,
        shown: None,
        description: None,
        picker: true,
        withdrawn: false,
        source,
        estimated: true,
    })
}

fn dearer(row: &CatalogueModel, previous: Option<&CatalogueModel>) -> bool {
    let Some(previous) = previous else { return true };
    let price = crate::runs::state::most_costly(&row.model).unwrap_or(u64::MAX);
    let before = crate::runs::state::most_costly(&previous.model).unwrap_or(u64::MAX);
    price > before || (price == before && row.number < previous.number)
}
