//! Run models, policies and charters (jig/docs/design/domain/engine/executors.md, 1).

use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;

/// One configured model and its deployment-unit prices (domain/agent/charter.md, 5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Model {
    pub provider: u32,
    pub dialect: u32,
    pub endpoint: u32,
    /// The endpoint's backend, when it takes program sources (configuration.md, 3).
    pub program: Option<Box<[u8]>>,
    pub name: Box<[u8]>,
    /// The input tokens a conversation's window may hold on this model.
    pub window: u32,
    /// Prompt cache life in milliseconds; zero for none (domain/tasks/standing.md, 5.3).
    pub cache_life_ms: u32,
    /// The most output tokens one completion may have, thinking included.
    pub output: u32,
    pub input_price: u64,
    pub cached_price: u64,
    /// Cache writes, when priced apart from fresh input.
    pub cache_write_price: Option<u64>,
    pub output_price: u64,
    pub price_unit: u32,
    /// The efforts the model declares, by name in its order; a task's LLM choice names one by
    /// index, and none for a model that declares none (domain/tasks/README.md, 3).
    pub efforts: Box<[Box<[u8]>]>,
    /// On a configured model and a charter's, the default a task given no LLM choice takes,
    /// one of `efforts` by name, or `None` for its first (configuration.md, 3).
    /// On a run's charter, its task's choice's (domain/engine/executors.md, 1), or `None` for the LLM
    /// provider's default when the model declares no efforts.
    pub effort: Option<Box<[u8]>>,
}

impl Model {
    /// An unoccupied catalogue position (domain/engine/settings.md, 11).
    #[must_use]
    pub fn hole() -> Self {
        Self {
            provider: 0,
            dialect: 0,
            endpoint: 0,
            program: None,
            name: Box::default(),
            window: 0,
            cache_life_ms: 0,
            output: 0,
            input_price: 0,
            cached_price: 0,
            cache_write_price: None,
            output_price: 0,
            price_unit: 0,
            efforts: Box::default(),
            effort: None,
        }
    }
    #[must_use]
    pub fn is_hole(&self) -> bool {
        self.name.is_empty()
    }

    /// The default effort a task given no LLM choice takes (domain/tasks/README.md, 3).
    #[must_use]
    pub fn default_effort(&self) -> Option<&[u8]> {
        match &self.effort {
            Some(effort) => Some(effort.as_ref()),
            None => match self.efforts.first() {
                Some(effort) => Some(effort.as_ref()),
                None => None,
            },
        }
    }
}

/// The most efforts a configured model declares (configuration.md, 3).
pub const EFFORTS: u32 = 16;
/// The most bytes in an effort's name: the bound smith's charter holds an effort to
/// (smith's protocol/charter.md, sections 2 and 6).
pub const EFFORT_BYTES: u32 = 32;
/// Byte bounds on a configured model's name and description (protocol/limits.md).
pub use jig_domain_people::{MODEL_DESCRIPTION_BYTES, MODEL_NAME_BYTES};

/// Root-owned charter policy, independent of Smith's run vocabulary.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent charter grants and run behavior")]
pub struct RunPolicy {
    pub instructions: Box<[u8]>,
    pub waiting: skein_lib::Duration,
    pub resume: bool,
    pub turns: u32,
    pub time: skein_lib::Duration,
    /// The turns and time of each run kept for finishing once the rest of
    /// its budget is spent.
    pub reserve_turns: u32,
    pub reserve_time: skein_lib::Duration,
    pub model: Model,
    pub alternatives: Box<[Model]>,
    pub inspect: bool,
    pub modify: bool,
    pub shell: bool,
    pub agents: bool,
    pub call_timeout: skein_lib::Duration,
    /// The host families its runs are offered, a mask of the configured `ToolFamilies` as a
    /// task's authority's tools are; a family it or the task's authority leaves out is not
    /// offered (domain/engine/executors.md, 1).
    pub tools: jig_domain_tasks::Tools,
}

impl RunPolicy {
    /// Every family: the identity under a task's authority, and what a decoder whose
    /// record carries no mask fills in.
    pub const EVERY_FAMILY: jig_domain_tasks::Tools = jig_domain_tasks::Tools(u64::MAX);
}

/// The root's complete task-specific charter, kept behind one bounded
/// assignment cell so delivery queues carry a small fixed-size value. Its settings
/// revision is fixed when preparation begins (domain/engine/settings.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RunCharter {
    /// The settings revision recorded as this run began preparation.
    pub revision: u32,
    pub ended_by: tasks::EndedBy,
    /// Selected grant for main then each retained alternative; none sources have no grant.
    pub sources: Box<[Option<u32>]>,
    pub accounts: Box<[authority::AccountBinding]>,
    pub placement: jig_domain_fleet::PlacementConstraint,
    pub policy: RunPolicy,
    pub contract: tasks::Contract,
    pub authority: tasks::Authority,
    pub budget: u64,
}

/// A completion's maximum price, rounded up once (domain/authority.md, 7).
#[must_use]
pub fn most_costly(model: &Model) -> Option<u64> {
    let rate = model.input_price.max(model.cached_price).max(model.cache_write_price.unwrap_or(model.input_price));
    let input = u128::from(model.window).checked_mul(u128::from(rate))?;
    let output = u128::from(model.output).checked_mul(u128::from(model.output_price))?;
    let sum = input.checked_add(output)?;
    let unit = u128::from(model.price_unit);
    let rounded = sum.checked_div(unit)?.checked_add(u128::from(sum.checked_rem(unit)? != 0))?;
    u64::try_from(rounded).ok()
}

/// Reserve turns and one completion more (domain/authority.md, 7).
#[must_use]
pub fn reserve_floor(policy: &RunPolicy) -> Option<u64> {
    most_costly(&policy.model)?.checked_mul(u64::from(policy.reserve_turns).checked_add(1)?)
}
