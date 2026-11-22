//! Protocol backend constructors for the shared agent world (smith/docs/design/protocol/llm.md, section 10).
use crate::fixture::Configuration;
use skein_fake_checkout::Checkout;
use skein_fake_llm_domain as provider;
use smith_agent_world::Settings;
use smith_domain::{self as agent, run};
use smith_protocol_llm::Limits as TranslationLimits;

/// The shared agent loop with native byte observations.
#[derive(Debug)]
pub struct World(smith_agent_world::World<crate::component_backend::Protocol>);

impl core::ops::Deref for World {
    type Target = smith_agent_world::World<crate::component_backend::Protocol>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl World {
    /// Queue the host's Start and run its completions through one production Component.
    /// The scripted byte peers observe the requests; the pool owns socket reuse and closure.
    #[must_use]
    pub fn with_protocol(
        mut settings: Settings,
        transcript: Option<agent::Transcript>,
        configuration: Configuration,
        limits: TranslationLimits,
        scripts: Box<[provider::api::Script]>,
    ) -> World {
        settings.window = crate::MODEL_WINDOW;
        Self(smith_agent_world::World::with_backend(
            &settings,
            transcript,
            Box::default(),
            crate::component_backend::Protocol::new(configuration, limits, scripts, settings.seed, &settings.limits),
        ))
    }

    /// Replay with calls answered by the host after its last saved turn.
    #[must_use]
    pub fn with_protocol_answers(
        mut settings: Settings,
        transcript: Option<agent::Transcript>,
        answered: Box<[agent::AnsweredCall]>,
        configuration: Configuration,
        limits: TranslationLimits,
        scripts: Box<[provider::api::Script]>,
    ) -> World {
        settings.window = crate::MODEL_WINDOW;
        Self(smith_agent_world::World::with_backend(
            &settings,
            transcript,
            answered,
            crate::component_backend::Protocol::new(configuration, limits, scripts, settings.seed, &settings.limits),
        ))
    }

    /// Caller-selected mounts and observed Client/byte-peer scripts, using the same
    /// receiving and settling paths as `with_protocol`; no unused fixture root is seeded.
    #[must_use]
    pub fn with_workspace_protocol(
        mut settings: Settings,
        transcript: Option<agent::Transcript>,
        workspace: Option<run::Workspace>,
        disk: Checkout,
        configuration: Configuration,
        limits: TranslationLimits,
        scripts: Box<[provider::api::Script]>,
    ) -> World {
        settings.window = crate::MODEL_WINDOW;
        Self(smith_agent_world::World::with_selected_backend(
            &settings,
            transcript,
            Box::default(),
            workspace,
            disk,
            crate::component_backend::Protocol::new(configuration, limits, scripts, settings.seed, &settings.limits),
            None,
            None,
        ))
    }

    /// Run the caller's charter and concrete saved history through the production component.
    #[must_use]
    pub fn with_workspace_protocol_charter(
        mut settings: Settings,
        transcript: Option<agent::Transcript>,
        workspace: Option<run::Workspace>,
        disk: Checkout,
        protocol: (Configuration, TranslationLimits),
        scripts: Box<[provider::api::Script]>,
        charter: run::Charter,
    ) -> World {
        settings.window = crate::MODEL_WINDOW;
        let (configuration, limits) = protocol;
        Self(smith_agent_world::World::with_selected_backend(
            &settings,
            transcript,
            Box::default(),
            workspace,
            disk,
            crate::component_backend::Protocol::new(configuration, limits, scripts, settings.seed, &settings.limits),
            Some(charter),
            None,
        ))
    }
    /// Strike one accepted credential at the world's current clock.
    ///
    /// # Errors
    /// Returns the fake's refusal when its account capacity is exceeded.
    pub fn strike(&mut self, credential: u32, strike: provider::api::Strike) -> Result<(), provider::api::Error> {
        let now = self.0.now();
        self.0.backend_mut().strike(credential, strike, now)
    }

    /// Supply a renewed bearer after the host observes an unauthorized notice.
    /// The fake's sign-out ends only when the host supplies that next generation.
    pub fn renew_sign_in(&mut self, generation: u64) {
        let now = self.now();
        let valid = skein_lib::Duration::from_secs(7200);
        self.0.backend_mut().renew(generation, now.saturating_add(valid));
        self.0
            .grant_at(now, agent::Grant { renewable: true, name: agent::GrantName { account: 0, generation }, valid });
        self.clear(0, &provider::api::Strike::SignedOut);
    }

    /// Clear a strike at the world's current clock, ending its ledger interval.
    pub fn clear(&mut self, credential: u32, strike: &provider::api::Strike) {
        let now = self.0.now();
        self.0.backend_mut().clear(credential, strike, now);
    }

    /// Record an excluded draw at its injection boundary.
    pub fn leave_out_fault(&mut self, scenario: &'static str, kind: skein_world::faults::Kind) {
        let now = self.now();
        self.0.backend_mut().faults.ledger.leave_out(scenario, kind, now);
    }

    /// This seed's ledger, recorded at fake boundaries.
    #[must_use]
    pub fn fault_ledger(&self) -> &skein_world::faults::FaultLedger {
        &self.0.backend().faults.ledger
    }

    /// Judge this seed once, retaining the attributed counters beside its faults.
    /// # Errors
    /// Returns the run-blame table's findings.
    pub fn judge_faults(&mut self) -> Result<(), String> {
        let mut ledger = core::mem::take(&mut self.0.backend_mut().faults.ledger);
        let result = self.0.judge_faults(&mut ledger);
        self.0.backend_mut().faults.ledger = ledger;
        result
    }

    /// Logical completion calls and their independently received HTTP requests.
    #[must_use]
    pub fn protocol_calls(&self) -> &[crate::component_backend::Call] {
        self.0.backend().calls()
    }

    /// Schedule a message bearing the scripted host's label.
    pub fn message_at(&mut self, at: skein_lib::Time, name: skein_lib::Token, text: Box<[u8]>) {
        self.0.message_at(at, name, b"host".as_slice().into(), text);
    }
}
