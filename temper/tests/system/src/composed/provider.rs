//! Optional native provider wire under the same composed agent loop.
use skein_fake_llm_domain::api::{Query, Script};
use skein_lib::{Time, Token, Wall};
use smith_agent_world::{
    Settings,
    backend::{Backend, Progress, Typed},
};
use smith_domain::Request;

use smith_protocol_llm_world::{Peer, Protocol, fixture};

/// One checked document and its wire configuration, shared by every activation in a walk.
#[expect(missing_debug_implementations, reason = "the wire configuration owns fixture credential bytes")]
pub struct Playback {
    configuration: fixture::Configuration,
    peer: Peer,
    pub(super) limits: smith_protocol_llm::Limits,
}

impl Playback {
    /// Play the document with all of its cues, turn rules, takes and domain limits intact.
    #[must_use]
    pub fn new(document: &temper_fake::llm::Document, configuration: fixture::Configuration, seed: u64) -> Self {
        // Carry stock tool descriptions and the saved conversation in each native request.
        let mut limits = fixture::limits();
        limits.client.request = 131_072;
        limits.client.http.request = limits.client.request;
        limits.client.http.send = 8192;
        limits.client.input = 8192;
        limits.stated.call_input_bytes = limits.client.input;
        limits.history_bytes = u64::from(limits.client.request);
        limits.rendered_result = 8192;
        Self { configuration, peer: Peer::new(document.domain(seed), document.limits()), limits }
    }
}

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
    pub(super) fn strike(&mut self, credential: u32, strike: skein_fake_llm_domain::api::Strike, now: Time) {
        match self {
            Self::Typed(provider) => provider.provider_mut().strike(credential, strike, now),
            Self::Wire(provider) => provider.strike(credential, strike, now),
        }
        .expect("account fault fits");
    }

    pub(super) fn clear(&mut self, credential: u32, strike: &skein_fake_llm_domain::api::Strike, now: Time) {
        match self {
            Self::Typed(provider) => provider.provider_mut().clear(credential, strike),
            Self::Wire(provider) => provider.clear(credential, strike, now),
        }
    }

    pub(super) fn continue_provider(&mut self, previous: &mut Self) {
        if let (Self::Wire(current), Self::Wire(previous)) = (self, previous) {
            current.continue_provider(previous);
        }
    }
    pub(super) fn new(
        settings: &Settings,
        script: &Script,
        wire: bool,
        playback: Option<&Playback>,
        grants: &[super::worker::agent::Grant],
    ) -> Self {
        if !wire {
            assert!(playback.is_none(), "documents play through the wire");
            return Self::Typed(Typed::scripted(settings, Box::new([script.clone()])));
        }
        let [grant] = grants else { panic!("one source pays for the provider story") };
        let initial = smith_domain::GrantName { account: grant.account, generation: 1 };
        let mut provider = if let Some(playback) = playback {
            Protocol::with_peer(
                &playback.configuration,
                playback.limits,
                playback.peer.clone(),
                settings.seed,
                &settings.limits,
                initial,
            )
        } else {
            Protocol::with_grant(
                fixture::Dialect::OpenAi.configuration(),
                fixture::limits(),
                Box::new([script.clone()]),
                settings.seed,
                &settings.limits,
                initial,
            )
        };
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
