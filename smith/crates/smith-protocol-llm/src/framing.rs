//! Startup measures provider punctuation with empty tool text.
//! Only a byte count leaves this module; no credential or destination is kept.
//! `prompt_allowance` prices the configured tool inventory and fixed routing
//! fields. Contract: protocol/limits.md, section3.2.

use alloc::boxed::Box;

use skein_lib::List;
use skein_llm::client;

use crate::Error;

/// Measure fixed provider framing, including the largest supported optional routing fields.
/// Observed endpoint option text are counted separately in its own part.
pub fn prompt_allowance(provider: skein_llm::Provider, client: &client::Limits, tools: u32) -> Result<u64, Error> {
    let endpoint = match provider {
        skein_llm::Provider::OpenAi => skein_llm::Endpoint::openai(),
        skein_llm::Provider::OpenAiCodex => return Err(Error::Invalid),
        skein_llm::Provider::Anthropic => skein_llm::Endpoint::anthropic(),
    };
    let reasoning_effort: Option<Box<[u8]>> = match provider {
        skein_llm::Provider::OpenAi => Some(Box::new([0; 32])),
        skein_llm::Provider::OpenAiCodex => return Err(Error::Invalid),
        skein_llm::Provider::Anthropic => Some(Box::from(b"medium".as_slice())),
    };
    let mut prompt = skein_llm::Prompt {
        model: Box::from(b"a".as_slice()),
        instructions: Box::new([]),
        tools: offered_tools(tools)?,
        choice: skein_llm::ToolChoice::Auto,
        messages: Box::new([skein_llm::Message {
            role: skein_llm::Role::User,
            content: Box::new([skein_llm::Block::Text { text: Box::new([]), replay: None }]),
        }]),
        reasoning_effort,
        affinity: Some(skein_llm::Affinity { key: [0; 16], thread: u32::MAX }),
        max_output_tokens: None,
    };
    prompt.output_ceiling(provider, client.declared_output_tokens).or(Err(Error::Invalid))?;
    let credential = match provider {
        skein_llm::Provider::OpenAi => skein_llm::Credential::bearer(Box::from(b"a".as_slice())),
        skein_llm::Provider::Anthropic => skein_llm::Credential::key(Box::from(b"a".as_slice())),
        skein_llm::Provider::OpenAiCodex => return Err(Error::Invalid),
    };
    let mut limits = *client;
    // Only these observed fixed records participate in this measurement.
    // Runtime history/tool maxima do not belong to its temporary wrappers.
    limits.tools = tools;
    limits.history_items = 2;
    limits.http.request = skein_tls::client::MAX_PLAINTEXT;
    limits.http.read = 4096;
    limits.http.send = skein_tls::client::MAX_PLAINTEXT;
    limits.sse.chunk = limits.http.read;
    let measured = match client::measure(&prompt, Some(&credential), &endpoint, &limits) {
        Ok(measured) => measured,
        Err(error) => return Err(Error::from_client(error)),
    };
    Ok(u64::from(measured.body))
}

/// The same fixed empty schema and metadata for each offered tool.
pub(super) fn offered_tools(tools: u32) -> Result<Box<[skein_llm::Tool]>, Error> {
    let schema =
        skein_llm::Json::from_bytes(b"{}", &skein_llm::DocumentLimits { bytes: 2, strings: 1, tokens: 2, depth: 2 })
            .or(Err(Error::Invalid))?;
    let mut offered = List::with_capacity(tools);
    for _ in 0..tools {
        offered
            .push(skein_llm::Tool {
                name: Box::from(b"a".as_slice()),
                description: Box::new([]),
                schema: schema.clone(),
            })
            .or(Err(Error::Limit))?;
    }
    Ok(offered.into_boxed())
}
