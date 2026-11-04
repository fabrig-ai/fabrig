//! The protocol world's direct-client setup; production owns only translation.

use skein_lib::Token;
use skein_llm::{Credential, Endpoint, client};
use smith_domain::{llm, run};

pub use smith_protocol_llm::{
    Below, BelowEvent, Component, ComponentError, ComponentLimits, ConfiguredEndpoint, EndpointError, EndpointOptions,
    Endpoints, Error, FromDomain, GrantError, Grants, Limits, Receiving, Stated, ToDomain, ToolKind, ToolSchema,
    completion_bound, component_valid, decode, decode_deliver, decode_finish, deliver_schema, finish_schema,
    render_bound, render_outcome, schemas_for_contract, valid,
};

pub use smith_protocol_llm::fixture::{Context, cancelled, completion, failed, prompt, refusal, schemas};

/// All owned input to one preparation. Caller credentials are consumed as a grant;
/// acquisition and refresh remain outside Smith.
#[expect(missing_debug_implementations, reason = "input owns caller credential bytes")]
pub struct Input {
    /// Root completion callback owner, echoed exactly by its observed terminal.
    pub owner: Token,

    /// Observed root prompt, consumed without retained borrowed prompt data.
    pub prompt: llm::Prompt,

    /// Caller-configured neutral endpoint identity; must equal the prompt's identity.
    pub endpoint_name: llm::Endpoint,

    /// Shared Client endpoint configuration, passed unchanged without provider branches.
    pub endpoint: Endpoint,

    /// Observed caller-provided credential; Smith owns no exchange or refresh client.
    pub credential: Credential,

    /// Receiving metadata of the root's observed Complete request.
    pub receiving: Receiving,
}

/// One prepared skein Client and its retained translation context.
/// The caller drives the Client itself; no second client or protocol state machine is created.
#[expect(missing_debug_implementations, reason = "the observed Client holds credential-bearing HTTP state")]
pub struct Prepared {
    /// Observed skein Client, retained through real Reusable or Close/Closed settlement.
    pub client: client::Client,

    /// Owned declarations and receiving contract, consumed by exactly one observed terminal translation.
    pub context: Context,
}

/// Prepare the skein client with the production translator.
/// # Errors
/// Refuses incompatible receiving limits or invalid prompt declarations.
pub fn prepare(input: Input, limits: &Limits) -> Result<Prepared, Error> {
    let Input { owner, prompt, endpoint_name, endpoint, credential, receiving } = input;
    if prompt.endpoint != endpoint_name {
        return Err(Error::Invalid);
    }
    let ceiling = prompt.max_tokens;
    let (mut translated, context) = smith_protocol_llm::fixture::prepare_prompt(owner, prompt, receiving, limits)?;
    translated.output_ceiling(endpoint.provider, ceiling).map_err(Error::from_client)?;
    let client = client::Client::prepare(
        skein_llm::Call { owner, endpoint, credential: Some(credential), prompt: translated },
        &limits.client,
    )
    .map_err(Error::from_client)?;
    Ok(Prepared { client, context })
}

/// Prepare a client with the production schemas for the admitted contract.
/// # Errors
/// Refuses invalid declarations or receiving limits.
pub fn prepare_for_contract(
    input: Input,
    outcome: &run::outcome::OutcomeSpec,
    deliver: Option<&run::outcome::ChangeSpec>,
    limits: &Limits,
) -> Result<Prepared, Error> {
    let Input { owner, prompt, endpoint_name, endpoint, credential, receiving } = input;
    if prompt.endpoint != endpoint_name {
        return Err(Error::Invalid);
    }
    let ceiling = prompt.max_tokens;
    let (mut translated, context) =
        smith_protocol_llm::fixture::prepare_for_contract(owner, prompt, outcome, deliver, receiving, limits)?;
    translated.output_ceiling(endpoint.provider, ceiling).map_err(Error::from_client)?;
    let client = client::Client::prepare(
        skein_llm::Call { owner, endpoint, credential: Some(credential), prompt: translated },
        &limits.client,
    )
    .map_err(Error::from_client)?;
    Ok(Prepared { client, context })
}

/// Effective fixture quantities shared by direct-client tests.
#[must_use]
pub fn stated() -> Stated {
    Stated::from_tools(&crate::LIMITS.session.tools, skein_llm_world::limits().input)
        .expect("fixture numeric bounds fit")
}
