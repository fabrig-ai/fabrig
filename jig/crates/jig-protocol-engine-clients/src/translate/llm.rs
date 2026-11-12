//! A task's LLM choice and `me`'s models: the core names a model and its effort by index in
//! configuration's order, the client by name (client/domain/contract/requests.md, 2, and client/domain/contract/documents.md, 1; domain/tasks/README.md, 3).
use super::w;
use crate::Model;
use alloc::boxed::Box;
use jig_domain_authority::Models;
use jig_domain_people as p;
use jig_domain_tasks as t;
use skein_lib::{List, Reader, bytes};

/// The current catalogue in numbered positions, including empty rows for holes
/// (jig/docs/design/domain/engine/settings.md, 11).
#[must_use]
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub fn catalogue(models: &[jig_domain_core::settings::CatalogueModel]) -> Box<[Model]> {
    let mut highest = 0;
    for row in models {
        highest = highest.max(row.number);
    }
    assert!(highest <= t::MODELS, "catalogue numbers bounded by MODELS");
    let mut rows = List::with_capacity(highest);
    for _ in 0..highest {
        rows.push(Model {
            new_until: None,
            number: 0,
            source: jig_domain_core::settings::ModelSource::Stock,
            estimated: false,
            name: Box::default(),
            provider: None,
            window: 0,
            output: 0,
            efforts: Box::default(),
            effort: None,
            prices: crate::Prices { input: 0, cached: 0, cache_write: None, output: 0, unit: 0 },
            description: None,
            shown: None,
            picker: false,
        })
        .expect("one row per catalogue position");
    }
    for row in models {
        let at = row.number.checked_sub(1).expect("catalogue numbers start at one");
        let slot = rows.get_mut(at).expect("catalogue number at most highest");
        let model = &row.model;
        *slot = Model {
            new_until: match row.first_read {
                Some(at) => Some(skein_lib::Wall::from_nanos(
                    at.as_nanos().saturating_add(jig_domain_core::settings::catalogue::NEW_FOR.as_nanos()),
                )),
                None => None,
            },
            number: row.number,
            source: row.source,
            estimated: row.estimated,
            name: model.name.clone(),
            provider: if model.provider == 0 { None } else { Some(model.provider) },
            window: model.window,
            output: model.output,
            efforts: model.efforts.clone(),
            effort: match model.default_effort() {
                Some(effort) => Some(bytes::copy_of(effort)),
                None => None,
            },
            prices: crate::Prices {
                input: model.input_price,
                cached: model.cached_price,
                cache_write: model.cache_write_price,
                output: model.output_price,
                unit: model.price_unit,
            },
            description: row.description.clone(),
            shown: row.shown.clone(),
            picker: row.picker,
        };
    }
    rows.into_boxed()
}

/// A complete permission selector by names, preserving literal ALL even with 64 models.
pub fn authority(value: Models, models: &[Model], limits: &w::Limits) -> Result<Option<w::ModelSet>, w::Problem> {
    if value.is_all() {
        return Ok(Some(w::ModelSet::All));
    }
    let mut names = List::with_capacity(p::MODELS);
    for (at, model) in models.iter().enumerate() {
        if !model.name.is_empty()
            && value.bits() & (1_u64.checked_shl(u32::try_from(at).expect("bounded model index")).unwrap_or(0)) != 0
        {
            names.push(bytes::copy_of(&model.name)).expect("configured models");
        }
    }
    let mut names = names.into_boxed();
    names.sort_unstable();
    let mut rows = List::with_capacity(u32::try_from(names.len()).expect("model bound"));
    for name in names {
        rows.push(name).expect("same model count");
    }
    Ok(Some(w::ModelSet::Names(w::ModelSetNames::new(limits, w::ModelSetNamesParts { names: rows })?)))
}

/// Collect only names the ask uses; admission of unknown names follows people's replay lookup.
pub(super) fn requested(
    value: Option<w::ModelSet>,
    names: &mut List<Box<[u8]>>,
) -> Result<Option<p::Models>, super::Untranslatable> {
    let bad = super::Untranslatable { path: w::Path::ModelSetNamesNames };
    Ok(match value {
        None => None,
        Some(w::ModelSet::All) => Some(p::Models::All),
        Some(w::ModelSet::Names(value)) => {
            let mut mask = 0_u64;
            for name in value.into_parts().names.into_boxed() {
                if name.is_empty() {
                    return Err(bad);
                }
                let mut bit = None;
                for (at, previous) in names.iter().enumerate() {
                    if *previous == name {
                        bit = Some(at);
                        break;
                    }
                }
                let at = match bit {
                    Some(at) => at,
                    None => {
                        let at = skein_lib::length::usize_of(names.len());
                        if names.push(name).is_err() {
                            return Err(bad);
                        }
                        at
                    }
                };
                let bit = 1_u64.checked_shl(u32::try_from(at).expect("bounded model index")).unwrap_or(0);
                if mask & bit != 0 {
                    return Err(bad);
                }
                mask |= bit;
            }
            Some(p::Models::Named(mask))
        }
    })
}

/// Whether the configured models fit the core's indices and the wire's bounds.
pub(crate) fn fits(models: &[Model], limits: &w::Limits) -> bool {
    if models.len() > skein_lib::length::usize_of(limits.model_set_names_names)
        || models.len() > skein_lib::length::usize_of(t::MODELS)
        || models.len() > skein_lib::length::usize_of(limits.me_project_models)
    {
        return false;
    }
    for (at, model) in models.iter().enumerate() {
        if model.name.is_empty() {
            continue;
        }
        let name = model.name.len();
        if !skein_codec::text_is_valid(&model.name)
            || name > skein_lib::length::usize_of(jig_domain_core::MODEL_NAME_BYTES)
            || name > skein_lib::length::usize_of(limits.model_set_names_names_item)
            || name > skein_lib::length::usize_of(limits.me_model_name)
            || name > skein_lib::length::usize_of(limits.model_effort_model)
            || name > skein_lib::length::usize_of(limits.refusal_llm_model)
            || model.efforts.len() > skein_lib::length::usize_of(jig_domain_core::EFFORTS)
            || model.efforts.len() > skein_lib::length::usize_of(limits.me_model_efforts)
            || model.prices.unit == 0
            || model.prices.input == 0
            || model.prices.cached == 0
            || model.prices.cache_write == Some(0)
            || model.prices.output == 0
        {
            return false;
        }
        if let Some(description) = &model.description
            && (description.is_empty()
                || !skein_codec::text_is_valid(description)
                || description.len() > skein_lib::length::usize_of(jig_domain_core::MODEL_DESCRIPTION_BYTES)
                || description.len() > skein_lib::length::usize_of(limits.me_model_description_some))
        {
            return false;
        }
        if let Some(shown) = &model.shown
            && (shown.is_empty()
                || !skein_codec::text_is_valid(shown)
                || shown.len() > skein_lib::length::usize_of(limits.me_model_shown_some))
        {
            return false;
        }
        match &model.effort {
            Some(effort) => {
                if !model.efforts.contains(effort)
                    || effort.len() > skein_lib::length::usize_of(limits.me_model_effort_some)
                {
                    return false;
                }
            }
            None => {
                if !model.efforts.is_empty() {
                    return false;
                }
            }
        }
        for earlier in models.iter().take(at) {
            if earlier.name == model.name {
                return false;
            }
        }
        for effort in &model.efforts {
            if effort.is_empty()
                || !skein_codec::text_is_valid(effort)
                || effort.len() > skein_lib::length::usize_of(jig_domain_core::EFFORT_BYTES)
                || effort.len() > skein_lib::length::usize_of(limits.me_model_efforts_item)
                || effort.len() > skein_lib::length::usize_of(limits.model_effort_effort_some)
            {
                return false;
            }
        }
    }
    true
}

/// The core's offer, retaining configuration order and each model's candidate accounts.
pub fn models(
    offer: &[jig_domain_core::Offered],
    models: &[Model],
    limits: &w::Limits,
) -> Result<List<w::MeModel>, w::Problem> {
    let mut rows = List::with_capacity(u32::try_from(offer.len()).expect("checked models"));
    for offered in offer {
        let model = models
            .get(usize::from(offered.model))
            .ok_or(w::Problem { path: w::Path::MeModelName, reason: skein_codec::Reason::Bound })?;
        if model.name.is_empty() {
            continue;
        }
        let mut efforts = List::with_capacity(u32::try_from(model.efforts.len()).expect("checked efforts"));
        for effort in &model.efforts {
            efforts.push(bytes::copy_of(effort)).expect("same capacity");
        }
        rows.push(w::MeModel::new(
            limits,
            w::MeModelParts {
                new_until: match model.new_until {
                    Some(at) => Some(at.as_nanos()),
                    None => None,
                },
                number: model.number,
                source: match model.source {
                    jig_domain_core::settings::ModelSource::Stock => w::ModelSource::Stock,
                    jig_domain_core::settings::ModelSource::List => w::ModelSource::List,
                    jig_domain_core::settings::ModelSource::Person => w::ModelSource::Person,
                },
                estimated: model.estimated,
                name: bytes::copy_of(&model.name),
                provider: model.provider,
                window: model.window,
                output: model.output,
                efforts,
                effort: model.effort.clone(),
                shown: model.shown.clone(),
                accounts: offered.accounts.clone(),
                picker: model.picker,
                prices: w::MePrices::new(
                    limits,
                    w::MePricesParts {
                        input: model.prices.input,
                        cached: model.prices.cached,
                        cache_write: model.prices.cache_write,
                        output: model.prices.output,
                        unit: model.prices.unit,
                    },
                )?,
                description: model.description.clone(),
            },
        )?)
        .expect("same capacity");
    }
    Ok(rows)
}

/// A kept choice by its names; a model or effort past configuration is a problem, as the core
/// keeps only configured ones.
pub fn choice(
    value: Option<&t::LlmChoice>,
    models: &[Model],
    limits: &w::Limits,
) -> Result<Option<w::LlmChoice>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut subagents = List::with_capacity(u32::try_from(value.subagents.len()).expect("bounded sub-agents"));
    for item in &value.subagents {
        subagents.push(model_effort(*item, models, limits)?).expect("same capacity");
    }
    Ok(Some(w::LlmChoice::new(
        limits,
        w::LlmChoiceParts {
            account: value.account,
            main: model_effort(value.main, models, limits)?,
            subagents,
            later: match &value.later {
                Some(later) => Some(later_choice(later, models, limits)?),
                None => None,
            },
        },
    )?))
}
fn later_choice(value: &t::LlmLater, models: &[Model], limits: &w::Limits) -> Result<w::LlmLater, w::Problem> {
    let mut subagents = List::with_capacity(u32::try_from(value.subagents.len()).expect("bounded sub-agents"));
    for item in &value.subagents {
        subagents.push(model_effort(*item, models, limits)?).expect("same capacity");
    }
    w::LlmLater::new(limits, w::LlmLaterParts { main: model_effort(value.main, models, limits)?, subagents })
}
fn model_effort(value: t::ModelEffort, models: &[Model], limits: &w::Limits) -> Result<w::ModelEffort, w::Problem> {
    let unknown = w::Problem { path: w::Path::ModelEffortModel, reason: skein_codec::Reason::Bound };
    let Some(model) = models.get(usize::from(value.model)) else {
        return Err(unknown);
    };
    if model.name.is_empty() {
        return Err(unknown);
    }
    let effort = match value.effort {
        Some(effort) => match model.efforts.get(usize::from(effort)) {
            Some(name) => Some(bytes::copy_of(name)),
            None => return Err(unknown),
        },
        None => None,
    };
    w::ModelEffort::new(limits, w::ModelEffortParts { model: bytes::copy_of(&model.name), effort })
}

/// A choice naming a model or an effort configuration lacks is refused before the core sees the
/// ask, as a project mismatch is: by the model's name, and for a compound request the place of its
/// operation (client/domain/contract/requests.md, 2, and client/domain/contract/README.md, 3).
pub(crate) fn refused(
    record: &w::Ask,
    models: &[Model],
    limits: &w::Limits,
) -> Result<Option<w::KeyedAnswer>, w::Problem> {
    let (at, found) = match record {
        w::Ask::Create(value) => (None, batch(value.tasks(), models, limits)?),
        w::Ask::Propose(value) => match value.action() {
            w::ProposeAction::Settings(value) => (None, settings_items(value.items(), models, limits)?),
            w::ProposeAction::Batch(value) => (None, batch(value.tasks(), models, limits)?),
            w::ProposeAction::Amendment(value) => (None, lacking(value.change().llm().as_ref(), models)),
        },
        w::Ask::Amend(value) => (None, lacking(value.change().llm().as_ref(), models)),
        w::Ask::Compound(value) => operations(value.operations(), models, limits)?,
        w::Ask::ChangeSettings(value) => (None, settings_items(value.items(), models, limits)?),
        w::Ask::ReadLists(_)
        | w::Ask::ChangeModel(_)
        | w::Ask::RemoveProject(_)
        | w::Ask::SetVisibility(_)
        | w::Ask::Rename(_)
        | w::Ask::MoveToNewest(_)
        | w::Ask::Restore(_)
        | w::Ask::MakeProject(_)
        | w::Ask::Message(_)
        | w::Ask::Answer(_)
        | w::Ask::Take(_)
        | w::Ask::HandBack(_)
        | w::Ask::Decide(_)
        | w::Ask::Release(_)
        | w::Ask::Lift(_)
        | w::Ask::Stop(_)
        | w::Ask::Cancel(_)
        | w::Ask::MoveTask(_)
        | w::Ask::Seen(_)
        | w::Ask::EditNote(_)
        | w::Ask::DeleteNote(_)
        | w::Ask::ChangePolicy(_)
        | w::Ask::SetRoles(_)
        | w::Ask::MakeService(_)
        | w::Ask::SignIn(_)
        | w::Ask::AccountChange(_)
        | w::Ask::SignOut(_)
        | w::Ask::SetPool(_)
        | w::Ask::Withdraw(_) => return Ok(None),
    };
    let Some(Lacking { model, why }) = found else {
        return Ok(None);
    };
    let refusal = w::Refusal::Llm(w::RefusalLlm::new(limits, w::RefusalLlmParts { model, why })?);
    let answer = w::Final::Refused(w::Refused::new(limits, w::RefusedParts { refusal, at })?);
    let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(answer.measure()));
    answer.encode(&mut out).expect("measured answer");
    match super::answers::owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits) {
        Ok(value) => Ok(Some(value)),
        Err(problem) => Err(problem),
    }
}
/// A model a choice names that configuration lacks, or whose effort its model lacks.
struct Lacking {
    model: Box<[u8]>,
    why: w::LlmWhy,
}
fn batch(bytes: &[u8], models: &[Model], limits: &w::Limits) -> Result<Option<Lacking>, w::Problem> {
    let mut reader = Reader::new(bytes);
    while let Some(piece) = piece(&mut reader)? {
        let task = w::NewTask::decode(limits, &mut Reader::new(&piece))?;
        if let Some(found) = lacking(task.llm().as_ref(), models) {
            return Ok(Some(found));
        }
    }
    Ok(None)
}
fn operations(
    bytes: &[u8],
    models: &[Model],
    limits: &w::Limits,
) -> Result<(Option<u32>, Option<Lacking>), w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut at = 0_u32;
    while let Some(piece) = piece(&mut reader)? {
        if let w::Operation::Amend(value) = w::Operation::decode(limits, &mut Reader::new(&piece))?
            && let Some(found) = lacking(value.change().llm().as_ref(), models)
        {
            return Ok((Some(at), Some(found)));
        }
        at = at.checked_add(1).ok_or(super::problem(skein_codec::Reason::Bound))?;
    }
    Ok((None, None))
}
fn piece(reader: &mut Reader<'_>) -> Result<Option<Box<[u8]>>, w::Problem> {
    match skein_codec::piece::split(reader) {
        Ok(value) => Ok(value),
        Err(_) => Err(super::problem(skein_codec::Reason::Malformed)),
    }
}
/// The first model of a choice that configuration lacks, or whose effort its model lacks.
fn lacking(choice: Option<&w::LlmChoice>, models: &[Model]) -> Option<Lacking> {
    let choice = choice?;
    if let Err(why) = indices(choice.main(), models) {
        return Some(Lacking { model: bytes::copy_of(choice.main().model()), why });
    }
    for item in choice.subagents() {
        if let Err(why) = indices(item, models) {
            return Some(Lacking { model: bytes::copy_of(item.model()), why });
        }
    }
    if let Some(later) = choice.later() {
        if let Err(why) = indices(later.main(), models) {
            return Some(Lacking { model: bytes::copy_of(later.main().model()), why });
        }
        for item in later.subagents() {
            if let Err(why) = indices(item, models) {
                return Some(Lacking { model: bytes::copy_of(item.model()), why });
            }
        }
    }
    None
}

/// A named choice by the indices configuration's order gives its models and efforts; a name
/// configuration lacks is untranslatable, as the refusal above answers it first.
pub(crate) fn people(
    value: Option<w::LlmChoice>,
    models: &[Model],
    path: w::Path,
) -> Result<Option<p::LlmChoice>, super::Untranslatable> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Ok(main) = indices(value.main(), models) else {
        return Err(super::Untranslatable { path });
    };
    let mut subagents = List::with_capacity(value.subagents().len());
    for item in value.subagents() {
        let Ok(item) = indices(item, models) else {
            return Err(super::Untranslatable { path });
        };
        subagents.push(item).expect("same capacity");
    }
    Ok(Some(p::LlmChoice {
        later: match value.later() {
            Some(later) => Some(people_later(later, models, path)?),
            None => None,
        },
        account: *value.account(),
        main,
        subagents: subagents.into_boxed(),
    }))
}
fn people_later(value: &w::LlmLater, models: &[Model], path: w::Path) -> Result<p::LlmLater, super::Untranslatable> {
    let Ok(main) = indices(value.main(), models) else {
        return Err(super::Untranslatable { path });
    };
    let mut subagents = List::with_capacity(value.subagents().len());
    for item in value.subagents() {
        let Ok(item) = indices(item, models) else {
            return Err(super::Untranslatable { path });
        };
        subagents.push(item).expect("same capacity");
    }
    Ok(p::LlmLater { main, subagents: subagents.into_boxed() })
}
fn indices(value: &w::ModelEffort, models: &[Model]) -> Result<p::ModelEffort, w::LlmWhy> {
    let mut found = None;
    for (at, model) in models.iter().enumerate() {
        if !model.name.is_empty() && *model.name == *value.model() {
            found = Some((at, model));
            break;
        }
    }
    let Some((at, model)) = found else {
        return Err(w::LlmWhy::Unconfigured);
    };
    let effort = match value.effort() {
        Some(name) => {
            let mut effort = None;
            for (index, item) in model.efforts.iter().enumerate() {
                if **item == **name {
                    effort = Some(u8::try_from(index).expect("checked efforts"));
                    break;
                }
            }
            if effort.is_none() {
                return Err(w::LlmWhy::Effort);
            }
            effort
        }
        None => None,
    };
    Ok(p::ModelEffort { model: u8::try_from(at).expect("checked models"), effort })
}

/// A model the core refused, by its configured name; an index past configuration is a problem,
/// as the core names only configured ones.
pub(crate) fn refusal(
    model: u8,
    why: p::LlmWhy,
    models: &[Model],
    limits: &w::Limits,
) -> Result<w::Refusal, w::Problem> {
    let Some(model) = models.get(usize::from(model)) else {
        return Err(w::Problem { path: w::Path::RefusalLlmModel, reason: skein_codec::Reason::Bound });
    };
    if model.name.is_empty() {
        return Err(w::Problem { path: w::Path::RefusalLlmModel, reason: skein_codec::Reason::Bound });
    }
    let why = match why {
        p::LlmWhy::Unconfigured => w::LlmWhy::Unconfigured,
        p::LlmWhy::Effort => w::LlmWhy::Effort,
        p::LlmWhy::Charter => w::LlmWhy::Charter,
        p::LlmWhy::Authority => w::LlmWhy::Authority,
        p::LlmWhy::Procedure => w::LlmWhy::Procedure,
        p::LlmWhy::Repeated => w::LlmWhy::Repeated,
    };
    Ok(w::Refusal::Llm(w::RefusalLlm::new(limits, w::RefusalLlmParts { model: bytes::copy_of(&model.name), why })?))
}

fn settings_items(bytes: &[u8], models: &[Model], limits: &w::Limits) -> Result<Option<Lacking>, w::Problem> {
    let mut reader = Reader::new(bytes);
    for _ in 0..t::SETTINGS_ITEMS {
        let Some(bytes) = piece(&mut reader)? else {
            return Ok(None);
        };
        let mut row = Reader::new(&bytes);
        let item = w::SettingItem::decode(limits, &mut row)?;
        if !row.is_empty() {
            return Err(super::problem(skein_codec::Reason::Malformed));
        }
        let found = match item.edit() {
            w::SettingEdit::Removed | w::SettingEdit::Reset => None,
            w::SettingEdit::Set(value) => match value.value() {
                w::SettingValue::Template(value) => lacking(value.llm().as_ref(), models),
                w::SettingValue::Charter(value) => {
                    let mut found = settings_model(value.model(), models);
                    for alternative in value.alternatives() {
                        if found.is_none() {
                            found = settings_model(alternative, models);
                        }
                    }
                    found
                }
                w::SettingValue::Overlay(value) => {
                    let mut found = None;
                    if let Some(model) = value.model() {
                        found = settings_model(model, models);
                    }
                    if let Some(alternatives) = value.alternatives() {
                        for model in alternatives {
                            if found.is_none() {
                                found = settings_model(model, models);
                            }
                        }
                    }
                    found
                }
                w::SettingValue::Policy(value) => {
                    let mut found = settings_selector(value.ceiling().models().as_ref(), models);
                    for role in value.roles() {
                        if found.is_none() {
                            found = settings_selector(role.authority().models().as_ref(), models);
                        }
                    }
                    found
                }
                w::SettingValue::Model(_) | w::SettingValue::Application(_) => None,
                w::SettingValue::Rules(value) => settings_selector(value.ceiling().models().as_ref(), models),
            },
        };
        if found.is_some() {
            return Ok(found);
        }
    }
    if !reader.is_empty() {
        return Err(super::problem(skein_codec::Reason::Bound));
    }
    Ok(None)
}
fn settings_selector(selector: Option<&w::ModelSet>, models: &[Model]) -> Option<Lacking> {
    match selector {
        Some(w::ModelSet::Names(names)) => {
            for name in names.names() {
                let mut configured = false;
                for model in models {
                    configured |= model.name.as_ref() == name.as_ref();
                }
                if !configured {
                    return Some(Lacking { model: name.clone(), why: w::LlmWhy::Unconfigured });
                }
            }
        }
        Some(w::ModelSet::All) | None => {}
    }
    None
}
fn settings_model(model: &w::SettingModel, models: &[Model]) -> Option<Lacking> {
    for configured in models {
        if configured.name.as_ref() == model.name() {
            if let Some(effort) = model.effort() {
                let mut offered = false;
                for name in &configured.efforts {
                    offered |= name.as_ref() == effort.as_ref();
                }
                if !offered {
                    return Some(Lacking { model: bytes::copy_of(model.name()), why: w::LlmWhy::Effort });
                }
            }
            return None;
        }
    }
    Some(Lacking { model: bytes::copy_of(model.name()), why: w::LlmWhy::Unconfigured })
}
