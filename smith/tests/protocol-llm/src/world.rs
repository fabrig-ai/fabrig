//! Protocol backend constructors for the shared agent world (protocol/llm.md, section 10).
use crate::wire;
use skein_fake_checkout::Checkout;
use skein_fake_llm_domain as provider;
use smith_agent_world::Settings;
use smith_domain::{self as agent, run};
use smith_protocol_llm::Limits as WireLimits;

/// The shared agent loop with native byte observations.
#[derive(Debug)]
pub struct World(smith_agent_world::World<wire::Composition>);

impl core::ops::Deref for World {
    type Target = smith_agent_world::World<wire::Composition>;
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
    /// Opt into observed adapter/Client/byte-peer routing for the queued original
    /// Start and discovery. Caller data supplies native endpoint, credentials,
    /// limits and scripts; typed worlds keep their existing backend.
    /// The selected wire backend constructs no typed provider or provider queues;
    /// observed Clients are prepared only when the root issues Complete. Receiving
    /// metadata comes only from each observed Complete. The fixture asserts a
    /// 256-call ceiling and compatible admission before any wire effect.
    /// The host supplies optional concrete history and committed results; the
    /// charter's resume flag selects it, and root/session admission precedes
    /// any provider effect. Success queues the original Start; its one Answer
    /// and every observed lower Closed remain owed while the caller drives it.
    #[must_use]
    pub fn with_wire(
        mut settings: Settings,
        transcript: Option<agent::Transcript>,
        configuration: wire::Configuration,
        limits: WireLimits,
        scripts: Box<[provider::api::Script]>,
    ) -> World {
        settings.window = crate::MODEL_WINDOW;
        Self(smith_agent_world::World::with_backend(
            &settings,
            transcript,
            Box::default(),
            wire::Composition::new(configuration, limits, scripts),
        ))
    }

    /// Native replay with calls answered by the host after its last saved turn.
    #[must_use]
    pub fn with_wire_answers(
        mut settings: Settings,
        transcript: Option<agent::Transcript>,
        answered: Box<[agent::AnsweredCall]>,
        configuration: wire::Configuration,
        limits: WireLimits,
        scripts: Box<[provider::api::Script]>,
    ) -> World {
        settings.window = crate::MODEL_WINDOW;
        Self(smith_agent_world::World::with_backend(
            &settings,
            transcript,
            answered,
            wire::Composition::new(configuration, limits, scripts),
        ))
    }

    /// Caller-selected mounts and observed Client/byte-peer scripts, using the same
    /// receiving and settling paths as `with_wire`; no unused fixture root is seeded.
    #[must_use]
    pub fn with_workspace_wire(
        mut settings: Settings,
        transcript: Option<agent::Transcript>,
        workspace: Option<run::Workspace>,
        disk: Checkout,
        configuration: wire::Configuration,
        limits: WireLimits,
        scripts: Box<[provider::api::Script]>,
    ) -> World {
        settings.window = crate::MODEL_WINDOW;
        Self(smith_agent_world::World::with_selected_backend(
            &settings,
            transcript,
            Box::default(),
            workspace,
            disk,
            wire::Composition::new(configuration, limits, scripts),
            None,
            None,
        ))
    }

    /// Caller-selected complete Charter, concrete saved history and native
    /// configuration/limits use the original root Start, observed Client and byte
    /// peer. Only the selected native backend exists; receiving bounds and
    /// all pending lower terminals apply through the one host Answer.
    #[must_use]
    pub fn with_workspace_wire_charter(
        mut settings: Settings,
        transcript: Option<agent::Transcript>,
        workspace: Option<run::Workspace>,
        disk: Checkout,
        wire: (wire::Configuration, WireLimits),
        scripts: Box<[provider::api::Script]>,
        charter: run::Charter,
    ) -> World {
        settings.window = crate::MODEL_WINDOW;
        let (configuration, limits) = wire;
        Self(smith_agent_world::World::with_selected_backend(
            &settings,
            transcript,
            Box::default(),
            workspace,
            disk,
            wire::Composition::new(configuration, limits, scripts),
            Some(charter),
            None,
        ))
    }
    /// Observed native bindings, retained after retirement.
    #[must_use]
    pub fn wire_bindings(&self) -> &[wire::Binding] {
        self.0.backend().bindings()
    }

    /// Schedule a message bearing the scripted host's label.
    pub fn message_at(&mut self, at: skein_lib::Time, name: skein_lib::Token, text: Box<[u8]>) {
        self.0.message_at(at, name, b"host".as_slice().into(), text);
    }
}
