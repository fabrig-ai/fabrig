//! Optional native provider wire under the same composed agent loop.
use skein_fake_llm_domain::api::{Query, Script};
use skein_lib::{Time, Token, Wall};
use smith_agent_world::{
    Settings,
    backend::{Backend, Progress, Typed},
};
use smith_domain::Request;

/// Provider faces for composed runs; the wire is opt-in.
#[expect(
    clippy::large_enum_variant,
    reason = "keep the existing typed face inline; only the opt-in wire allocates a backend"
)]
#[derive(Debug)]
pub enum Provider {
    /// Existing typed provider face.
    Typed(Typed),
    /// Production protocol and independent fake provider.
    Wire(Box<smith_protocol_llm_world::Protocol>),
}

impl Provider {
    pub(super) fn continue_provider(&mut self, previous: &mut Self) {
        if let (Self::Wire(current), Self::Wire(previous)) = (self, previous) {
            current.continue_provider(previous);
        }
    }
    pub(super) fn new(settings: &Settings, script: Script, wire: bool, grants: &[super::worker::agent::Grant]) -> Self {
        if !wire {
            return Self::Typed(Typed::scripted(settings, Box::new([script])));
        }
        let [grant] = grants else { panic!("one source pays for the provider story") };
        let configuration = smith_protocol_llm_world::fixture::Dialect::OpenAi.configuration();
        let mut provider = smith_protocol_llm_world::Protocol::with_grant(
            configuration,
            smith_protocol_llm_world::fixture::limits(),
            Box::new([script]),
            settings.seed,
            &settings.limits,
            smith_domain::GrantName { account: grant.account, generation: 1 },
        );
        if grant.generation != 1 {
            provider.grant(
                smith_domain::GrantName { account: grant.account, generation: grant.generation },
                skein_llm::Credential::bearer(b"replacement-token".as_slice().into()),
            );
        }
        Self::Wire(Box::new(provider))
    }
}

impl Backend for Provider {
    fn contract(
        &mut self,
        outcome: smith_domain::run::outcome::OutcomeSpec,
        deliver: Option<smith_domain::run::outcome::ChangeSpec>,
    ) {
        match self {
            Self::Typed(p) => p.contract(outcome, deliver),
            Self::Wire(p) => p.contract(outcome, deliver),
        }
    }
    fn start(&mut self, request: Request, now: Time, wall: Wall) -> Vec<Query> {
        match self {
            Self::Typed(p) => p.start(request, now, wall),
            Self::Wire(p) => p.start(request, now, wall),
        }
    }
    fn advance(&mut self, now: Time, wall: Wall, finished: bool) -> Progress {
        match self {
            Self::Typed(p) => p.advance(now, wall, finished),
            Self::Wire(p) => p.advance(now, wall, finished),
        }
    }
    fn cancel(&mut self, owner: Token, now: Time, wall: Wall) -> bool {
        match self {
            Self::Typed(p) => p.cancel(owner, now, wall),
            Self::Wire(p) => p.cancel(owner, now, wall),
        }
    }
    fn has_events(&self) -> bool {
        match self {
            Self::Typed(p) => p.has_events(),
            Self::Wire(p) => p.has_events(),
        }
    }
    fn is_settled(&self) -> bool {
        match self {
            Self::Typed(p) => p.is_settled(),
            Self::Wire(p) => p.is_settled(),
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        match self {
            Self::Typed(p) => p.next_deadline(),
            Self::Wire(p) => p.next_deadline(),
        }
    }
    fn assert_settled(&self) {
        match self {
            Self::Typed(p) => p.assert_settled(),
            Self::Wire(p) => p.assert_settled(),
        }
    }
}
