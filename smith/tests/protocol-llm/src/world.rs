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
