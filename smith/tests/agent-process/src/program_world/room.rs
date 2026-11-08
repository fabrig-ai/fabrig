//! Declared room and credential children for burst rows (smith/docs/design/domain/run.md, 5.3).
use super::Case;
use skein_lib::{Duration, List};

// The widest row asks six calls; all enter one batch before children answer.
const BURST_CALLS: u32 = 6;
// Main and the credential-backed child endpoint.
const ENDPOINTS: u32 = 2;

pub(super) fn profile(case: Case) -> smith_agent_service::profile::Profile {
    let mut profile = crate::profile();
    if let Case::Burst { conversations } = case {
        profile.declared.conversations = conversations;
        profile.policy.parallel_tools = BURST_CALLS;
    }
    profile
}

pub(super) fn declarations(case: Case) -> smith_agent_service::profile::Configuration {
    let mut declarations = crate::declarations(4096);
    if matches!(case, Case::Burst { .. }) {
        let mut direct = declarations.endpoints[0].clone();
        direct.number = 1;
        declarations.endpoints = vec![declarations.endpoints[0].clone(), direct].into();
    }
    declarations
}

pub(super) fn configuration(conversations: u32) -> smith_agent_service::Config {
    let case = Case::Burst { conversations };
    let profile = profile(case);
    let declarations = declarations(case);
    let mut limits = smith_agent_service::profile::derive(&profile, &declarations).expect("room profile");
    limits.domain.configured_endpoint_bytes = 4096;
    limits.llm.translation.client.request = 1_048_576;
    let mut config = crate::configuration_with_limits(skein_llm::Provider::OpenAi, &limits);
    config.domain.endpoints =
        Box::new([smith_domain::run::charter::Endpoint(0), smith_domain::run::charter::Endpoint(1)]);
    let mut direct = config.domain.limits[0].clone();
    direct.endpoint = smith_domain::run::charter::Endpoint(1);
    config.domain.limits = vec![config.domain.limits[0].clone(), direct].into();
    let mut endpoints = List::with_capacity(ENDPOINTS);
    for (number, name) in [(0, b"".as_slice()), (1, b"direct".as_slice())] {
        endpoints.push(smith_protocol_translate::Endpoint { name: name.into(), number, dialect: 3 }).expect("endpoint");
    }
    config.channel_endpoints = smith_protocol_translate::Endpoints::new(endpoints);
    config.llm_endpoints = smith_protocol_llm::Endpoints::new(
        [0, 1]
            .map(|number| smith_protocol_llm::ConfiguredEndpoint {
                name: smith_domain::llm::Endpoint(number),
                connect: Duration::from_secs(1),
                handshake: Duration::from_secs(1),
                destination: skein_llm_connection::Endpoint {
                    limits: limits.llm.translation.client,
                    credential: skein_llm::client::CredentialLimits { value: limits.llm.grant_value_bytes },
                    address: skein_io::kernel::Addr::from((std::net::Ipv4Addr::LOCALHOST, 443)),
                    transport: skein_llm_connection::Transport::Plaintext,
                    llm: skein_llm::Endpoint::openai(),
                },
                oversized_reasoning: smith_protocol_llm::OversizedReasoning::Fail,
            })
            .into(),
        ENDPOINTS,
    )
    .expect("main and direct endpoint");
    config
}

/// Grant the child either main's program or a credential on the second endpoint.
#[must_use]
pub fn child_charter(direct: bool) -> Box<[u8]> {
    crate::wire::edit_charter(&super::child_charter(), |parts| {
        // Main, four children and finish fit the run's turn budget.
        crate::wire::budget(parts, 24, 80);
        if direct {
            let mut child = parts.models.as_slice()[0].clone().into_parts();
            child.endpoint = b"direct".as_slice().into();
            child.account = Some(1);
            parts.models = List::with_capacity(1);
            parts
                .models
                .push(smith_charter::Llm::new(&smith_charter::CEILINGS, child).expect("direct child"))
                .expect("child");
        }
    })
}

pub(super) fn grant(frame: skein_channel::Frame, case: Case) -> skein_channel::Frame {
    if !matches!(case, Case::Burst { .. }) {
        return frame;
    }
    crate::wire::edit_start(&frame, |start| {
        let mut grants = List::with_capacity(ENDPOINTS);
        for grant in start.grants.as_slice() {
            grants.push(grant.clone()).expect("main grant");
        }
        grants
            .push(
                smith_channel::Grant::new(
                    &smith_channel::CEILINGS,
                    smith_channel::GrantParts {
                        account: 1,
                        generation: 1,
                        valid: Duration::from_secs(7200),
                        value: crate::bearer(b"token"),
                    },
                )
                .expect("child credential"),
            )
            .expect("child grant");
        start.grants = grants;
    })
}

pub(super) fn peer(story: &smith_fake_claude::Story) -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api;
    let scripts = story
        .cued
        .iter()
        .map(|script| {
            let answer = script
                .turns
                .iter()
                .flat_map(|turn| &turn.steps)
                .find_map(
                    |step| {
                        if let smith_fake_claude::Step::Text(text) = step { Some(text.as_bytes()) } else { None }
                    },
                )
                .expect("child text");
            api::Script {
                cue: script.cue.as_bytes().into(),
                turns: Box::new([smith_agent_world::fixture::says(
                    Box::new([api::Line::Text { text: answer.into() }]),
                    1,
                )]),
            }
        })
        .collect::<Vec<_>>();
    crate::fake::configured_openai(
        scripts.into(),
        crate::fake::credential(false),
        Duration::ZERO,
        skein_fake_peers::Transport::Plaintext,
    )
}
