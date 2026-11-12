//! The core's vocabulary: what crosses its edge (jig/docs/design/domain/engine/README.md, section 3).

pub mod acts;
pub mod documents;
pub mod inbox;
pub mod limits;
pub mod numbers;
pub mod public;
pub mod reads;
pub mod settings;
pub mod start;
pub mod stored;
pub mod translate;
pub mod vocabulary;

use crate::Model;
use alloc::boxed::Box;
use jig_domain_accounts::{self as accounts, Kind, Ways};
use jig_domain_authority as authority;
use jig_domain_people as people;

/// Policy copied into a later project (configuration.md, 3; domain/model.md, 6.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConfiguredPolicy {
    pub policy: authority::Policy,
    pub permissions: Box<[people::PermissionRole]>,
    pub templates: Box<[people::TemplateOverride]>,
}

/// One first-start project seed (configuration.md, 3; domain/model.md, 6.1).
#[derive(Hash, Debug)]
pub struct SeedProject {
    pub number: u32,
    pub name: Box<str>,
    pub policy: ConfiguredPolicy,
}

/// Application-supplied initial core state, in jig's vocabulary
/// (`domain/engine/restart.md`; `configuration.md`, 3).
#[derive(Debug)]
pub struct Config {
    /// Configured worker names in authenticated protocol position order.
    pub worker_names: Box<[Box<[u8]>]>,
    /// Identity committed with the first decision.
    pub deployment: [u8; 16],
    /// Deterministic task scheduling seed.
    pub seed: u64,
    /// Workers share this engine's process and cannot outlive it.
    pub workers_in_process: bool,
    /// Normal retained turns per attempt in the deployment's hub.
    pub hub_turns: u32,
    /// Initial authenticated owners.
    pub owners: Box<[people::InitialOwner]>,
    /// Configured deployment acts for each connector (domain/people/acts.md, 3).
    pub grants: Box<[people::ActGrant]>,
    /// Configured authority and policy.
    pub defaults: settings::Defaults,
    /// An application's kinds of settings (domain/engine/settings.md, 5).
    pub kinds: Box<[settings::Kind]>,
    /// Deployment and run settings retained by the core.
    pub settings: Settings,
    /// Projects used only by the first start (configuration.md, 3).
    pub seed_projects: Box<[SeedProject]>,
    /// Numbered connectors the application routes for task lifecycle handoffs.
    pub connectors: Box<[u16]>,
    /// Application job kinds and their bounds (jig/docs/design/domain/engine/executors.md, 5).
    pub jobs: Box<[jig_domain_link::JobBound]>,
    /// Connectors' procedures that start kept (jig/docs/design/domain/connectors.md, 6.1).
    pub kept_starts: Box<[jig_domain_tasks::KeptStart]>,
}

/// Deployment and current catalogue settings used by core decisions.
#[derive(Hash, Debug)]
pub struct Settings {
    pub endpoints: Box<[EndpointFacts]>,
    /// Provider number for the application's services.
    pub deployment_provider: u16,
    /// The current catalogue by number: row `i` is setting `i + 1`, including holes
    /// (domain/engine/settings.md, 11; authority.md, 3; domain/tasks/README.md, 3).
    pub models: Box<[Model]>,
    /// Maximum committed conversation bytes resumed whole.
    pub resume_bytes: u32,
    /// Maximum transcript text retained for a fresh run's brief.
    pub tail_bytes: u32,
    /// Jig's one-bit authority family for each engine tool.
    pub tools: ToolFamilies,
    /// Tool names whose calls belong to the task's requester alone (domain/engine/briefs.md, 2.1).
    pub private_tools: Box<[Box<[u8]>]>,
    /// Configured LLM providers, for the accounts child (accounts.md, 3.1).
    pub llm_providers: accounts::Providers,
    /// Actual client key ceiling, supplied by startup after protocol limits are checked.
    pub key_bytes: u32,
    /// The driver's redirect bound: the most bytes a pasted sign-in address may have.
    pub redirect_bytes: u32,
}

impl Settings {
    /// The provider's credential endpoint, where configured (protocol/configuration.md, 3).
    #[must_use]
    #[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
    pub fn direct_endpoint(&self, provider: u32) -> Option<&EndpointFacts> {
        for endpoint in &self.endpoints {
            if endpoint.direct.contains(&provider) {
                return Some(endpoint);
            }
        }
        None
    }
}

/// Jig's authority bits for the engine's tool families.
#[derive(Clone, Copy, Hash, Debug)]
pub struct ToolFamilies {
    pub delegate: authority::Tools,
    pub message: authority::Tools,
    pub control: authority::Tools,
    pub decide: authority::Tools,
    pub propose: authority::Tools,
    pub subscribe: authority::Tools,
    pub effect: authority::Tools,
    pub note: authority::Tools,
    pub recall: authority::Tools,
    pub read: authority::Tools,
    pub settings: u64,
    /// An application's tools for a run that explains to its person (authority.md, 3).
    pub ask: authority::Tools,
}

/// Connector number reserved for the core's recurring procedure (domain/tasks/README.md, 5).
pub const RECURRING: u16 = u16::MAX;

impl ToolFamilies {
    /// Every offered family has one distinct authority bit.
    #[must_use]
    pub fn valid(&self) -> bool {
        let mut seen = 0_u64;
        for family in [
            self.delegate,
            self.message,
            self.control,
            self.decide,
            self.propose,
            self.subscribe,
            self.effect,
            self.note,
            self.recall,
            self.read,
            authority::Tools(self.settings),
            self.ask,
        ] {
            if family.0.count_ones() != 1 || seen & family.0 != 0 {
                return false;
            }
            seen |= family.0;
        }
        true
    }

    /// The twelve engine tool families, numbered alike in every deployment.
    pub const JIG: Self = Self {
        delegate: authority::Tools(1),
        message: authority::Tools(2),
        control: authority::Tools(4),
        decide: authority::Tools(8),
        propose: authority::Tools(16),
        subscribe: authority::Tools(32),
        effect: authority::Tools(64),
        note: authority::Tools(128),
        recall: authority::Tools(256),
        read: authority::Tools(512),
        settings: 1024,
        ask: authority::Tools(2048),
    };
}

/// Endpoint identity used to check model settings (domain/engine/settings.md, 11).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EndpointFacts {
    /// Providers that name this endpoint for added models (domain/engine/settings.md, 11).
    pub providers: Box<[u32]>,
    /// Providers that name this endpoint as their direct way (protocol/configuration.md, 3).
    pub direct: Box<[u32]>,
    pub number: u32,
    pub dialect: u32,
    pub program: Option<Box<[u8]>>,
}

impl Settings {
    #[must_use]
    pub fn model_ways(&self, model: &Model) -> Ways {
        Ways {
            program: model.program.is_some(),
            direct: model.program.is_none() || self.direct_endpoint(model.provider).is_some(),
        }
    }

    #[must_use]
    pub fn model_endpoint(&self, model: &Model, kind: Kind) -> Option<u32> {
        if !self.model_ways(model).allows(kind) {
            return None;
        }
        if kind == Kind::Program || model.program.is_none() {
            Some(model.endpoint)
        } else {
            Some(self.direct_endpoint(model.provider)?.number)
        }
    }
}
