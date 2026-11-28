//! An agent task's LLM choice: its main model and the models its runs' sub-agents may use, each
//! with an effort and a speed (domain/tasks/README.md, section 3). Tasks checks a choice as a batch or an amendment is
//! checked: an agent task's only, each model configured, each effort among those its model
//! declares, each model among its charter's and within its authority's models (domain/tasks/README.md, section 4 and domain/tasks/changes.md
//! ; domain/authority-checks.md, section 1.1). It resolves a charter's default for the
//! root, which checks an omitted choice against its charter and authority (domain/tasks/README.md, section 3).
//! The root may check a choice alone against its charter and authority's models at the newest
//! revision, before asking for its batch (domain/tasks/README.md, section 4).
//!
//! Model index `i` is catalogue setting `i + 1` and bit `i` of `Models`; a hole has no
//! efforts and belongs to no charter. Effort `e` is the `e`th effort that row declares
//! (domain/engine/settings-catalogue.md, 1; `../../../docs/design/protocol/configuration-engine.md`, section 1).

use alloc::boxed::Box;
pub use jig_domain_contract::ops::{LlmChoice, LlmLater, ModelEffort, STANDARD_SPEED};

use skein_lib::{List, Name};

use crate::domain::Domain;
use crate::{Authority, Executor, Models, Refusal};

/// Most catalogue rows, the bits of `Models` (`../../../docs/design/protocol/configuration-engine.md`, section 1). A
/// choice's sub-agents name at most this many.
pub const MODELS: u32 = 64;

/// Why an LLM choice is refused (domain/tasks/README.md, section 4 and domain/tasks/changes.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LlmFault {
    /// The model is not configured.
    Unconfigured,
    /// The effort is not one the model declares, or none is named for a model that declares some.
    Effort,
    /// The model is not one of the task's charter's.
    Charter,
    /// The task's authority does not allow the model (domain/authority.md, section 3).
    Authority,
    /// The task is a procedure's or a person's, whose executor runs no model.
    Procedure,
    /// The sub-agents name the model twice.
    Repeated,
    /// The speed is not one the model declares, nor `standard`.
    Speed,
}

/// One charter version's models, in charter order; a removal offers none
/// (domain/engine/settings.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CharterModels {
    /// The charter's number.
    pub number: u32,
    pub resumes: bool,
    /// The revision that set this charter value.
    pub revision: u32,
    /// Its configured model indices, at most `MODELS`; none marks a removal.
    pub models: Option<Box<[u8]>>,
    /// The declared speed names for those models, without aliases.
    pub speeds: Box<[DeclaredSpeed]>,
}

/// One model's declared speed name in a charter version.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DeclaredSpeed {
    pub model: u8,
    pub name: Name<32>,
}

impl CharterModels {
    pub(crate) fn takes_speed(&self, chosen: ModelEffort) -> bool {
        let Some(name) = chosen.speed else { return true };
        if name.as_bytes() == STANDARD_SPEED {
            return true;
        }
        for speed in &self.speeds {
            if speed.model == chosen.model && speed.name == name {
                return true;
            }
        }
        false
    }
}

/// The latest charter version at or before `revision` (including removals).
pub(crate) fn version(charters: &[CharterModels], charter: u32, revision: u32) -> Option<&CharterModels> {
    let mut found = None;
    let mut latest = 0;
    for version in charters {
        if version.number == charter && version.revision <= revision && version.revision >= latest {
            latest = version.revision;
            found = Some(version);
        }
    }
    found
}

/// Whether `choice` has the shape a task keeps: its sub-agents at most `MODELS`.
#[must_use]
pub(crate) fn valid_shape(choice: &LlmChoice) -> bool {
    let bound = skein_lib::length::usize_of(MODELS);
    choice.subagents.len() <= bound
        && match &choice.later {
            Some(later) => later.subagents.len() <= bound,
            None => true,
        }
}

/// Whether `new`'s choice, if any, has the shape a task keeps; a recurrence checks the rest.
#[must_use]
pub(crate) fn valid_member(new: &crate::New) -> bool {
    match &new.llm {
        Some(choice) => valid_shape(choice),
        None => true,
    }
}

/// Check `choice` for a task run by `executor` within `authority` (domain/tasks/README.md, section 4;
/// domain/authority-checks.md, section 1.1). A refusal names the first model that fails and why. A choice
/// beyond the authority is refused, not proposed: proposing a model's widening comes later.
pub(crate) fn check(
    domain: &Domain,
    executor: Executor,
    revision: u32,
    authority: &Authority,
    choice: &LlmChoice,
) -> Result<(), Refusal> {
    let charter = match executor {
        Executor::Agent { charter } => charter,
        Executor::Procedure { .. } | Executor::Person(_) | Executor::Job { .. } => {
            return Err(Refusal::Llm { model: choice.main.model, why: LlmFault::Procedure });
        }
    };
    check_agent(domain, charter, revision, authority.models, choice)
}

fn check_agent(
    domain: &Domain,
    charter: u32,
    revision: u32,
    allowed: Models,
    choice: &LlmChoice,
) -> Result<(), Refusal> {
    let version = version(&domain.charters, charter, revision);
    check_models(domain, version, allowed, choice.main, &choice.subagents)?;
    if let Some(later) = &choice.later {
        if let Some(version) = version
            && version.resumes
        {
            return Err(Refusal::Llm { model: later.main.model, why: LlmFault::Charter });
        }
        check_models(domain, version, allowed, later.main, &later.subagents)?;
    }
    Ok(())
}

fn check_models(
    domain: &Domain,
    version: Option<&CharterModels>,
    allowed: Models,
    main: ModelEffort,
    subagents: &[ModelEffort],
) -> Result<(), Refusal> {
    fits(domain, version, allowed, main)?;
    for (at, chosen) in subagents.iter().enumerate() {
        for earlier in subagents.iter().take(at) {
            if earlier.model == chosen.model {
                return Err(Refusal::Llm { model: chosen.model, why: LlmFault::Repeated });
            }
        }
        // Each passing entry names another configured model, so the loop ends by `MODELS`.
        fits(domain, version, allowed, *chosen)?;
    }
    Ok(())
}

fn fits(domain: &Domain, version: Option<&CharterModels>, allowed: Models, chosen: ModelEffort) -> Result<(), Refusal> {
    let Some(declared) = domain.efforts.get(usize::from(chosen.model)) else {
        return Err(Refusal::Llm { model: chosen.model, why: LlmFault::Unconfigured });
    };
    let effort = match chosen.effort {
        Some(effort) => effort < *declared,
        None => *declared == 0,
    };
    if !effort {
        return Err(Refusal::Llm { model: chosen.model, why: LlmFault::Effort });
    }
    let Some(version) = version else {
        return Err(Refusal::Llm { model: chosen.model, why: LlmFault::Charter });
    };
    if !version.models.as_deref().unwrap_or(&[]).contains(&chosen.model) {
        return Err(Refusal::Llm { model: chosen.model, why: LlmFault::Charter });
    }
    if !version.takes_speed(chosen) {
        return Err(Refusal::Llm { model: chosen.model, why: LlmFault::Speed });
    }
    if !allows(allowed, chosen.model) {
        return Err(Refusal::Llm { model: chosen.model, why: LlmFault::Authority });
    }
    Ok(())
}

fn allows(models: Models, model: u8) -> bool {
    match 1_u64.checked_shl(u32::from(model)) {
        Some(bit) => models.bits() & bit != 0,
        None => false,
    }
}

/// An unnamed task must have at least one charter model within its authority (domain/tasks/README.md, 3).
pub(crate) fn any_allowed(models: &[u8], allowed: Models) -> bool {
    for model in models {
        if allows(allowed, *model) {
            return true;
        }
    }
    false
}

fn first_effort(domain: &Domain, model: u8) -> Option<u8> {
    match domain.efforts.get(usize::from(model)) {
        Some(0) | None => None,
        Some(_) => Some(0),
    }
}

impl Domain {
    /// Check `choice` for an agent task of `charter` within `models`, at the newest
    /// revision, as a batch checks it (domain/tasks/README.md, section 4): each model configured,
    /// named once among the sub-agents, with an allowed effort and speed; each model one
    /// the charter allows and `models` allows. A refusal names the first model that fails.
    pub fn check_llm(&self, charter: u32, models: Models, choice: &LlmChoice) -> Result<(), Refusal> {
        check_agent(self, charter, self.newest(), models, choice)
    }

    /// The default LLM choice of an agent task of `charter` within `models`
    /// (domain/tasks/README.md, section 3): main is the first of the charter's models,
    /// in its order, within `models`, and the sub-agents the charter's others within `models`;
    /// each with its model's first declared effort, or the provider's default for a model that
    /// declares none. `None` when the charter is not configured or allows none of `models`.
    #[must_use]
    pub fn default_llm(&self, charter: u32, models: Models) -> Option<LlmChoice> {
        self.default_llm_at(charter, self.newest(), models)
    }

    /// The charter-order choice at a task's own settings revision (domain/tasks/README.md, section 3).
    #[must_use]
    pub fn default_llm_at(&self, charter: u32, revision: u32, models: Models) -> Option<LlmChoice> {
        let configured = self.charter_models(charter, revision)?;
        let mut main = None;
        let mut subagents =
            List::with_capacity(u32::try_from(configured.len()).expect("a charter's models fit `MODELS`"));
        for model in configured {
            if !allows(models, *model) {
                continue;
            }
            // The first effort is the fallback; core applies each charter model's named
            // default (protocol/configuration-engine.md, 1) when it resolves a run's choice.
            let chosen = ModelEffort { model: *model, effort: first_effort(self, *model), speed: None };
            match main {
                None => main = Some(chosen),
                Some(_) => subagents.push(chosen).expect("one per charter model"),
            }
        }
        Some(LlmChoice { later: None, account: None, main: main?, subagents: subagents.into_boxed() })
    }
}

/// Clearing the creator preference keeps the pin; an explicit different account supersedes it
/// only in the admitted amendment (domain/accounts-uses.md, 1.3; domain/tasks/changes.md).
pub(crate) fn replaces_account(kept: Option<u32>, choice: &LlmChoice) -> bool {
    match choice.account {
        Some(account) => kept.is_some() && kept != Some(account),
        None => false,
    }
}

/// Why an explicit account cannot fund the prospective task (domain/accounts-uses.md, 1.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AccountWhy {
    Unknown,
    Provider,
    Uses,
    Limited,
}
