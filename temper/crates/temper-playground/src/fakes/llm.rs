//! Scripted LLM capacities (temper/docs/design/playground.md, 4).
use skein_lib::Duration;
/// The calm scripted LLM domain configuration.
#[must_use]
pub fn llm_config() -> skein_fake_llm_domain::Config {
    skein_fake_llm_domain::Config {
        calls: 4,
        query_bytes: 65_536,
        script_bytes: 32768,
        answer_bytes: 32768,
        latency_min: skein_lib::Duration::ZERO,
        latency_max: skein_lib::Duration::ZERO,
        overloaded: 0,
        rate_limited: 0,
        retry_after: skein_lib::Duration::ZERO,
        unavailable: 0,
        too_long: 0,
        unauthorized: 0,
        refused: 0,
        no_calls: 0,
        answer_tokens: 1,
        calls_per_answer: 1,
        malformed: 0,
        outside_choice: 0,
        tool_rounds: 0,
        cache_lifetime: skein_lib::Duration::from_secs(300),
        cache_entries: 16,
        strict: false,
        unscoped_reads: 0,
    }
}
/// The scripted LLM's HTTP and document limits.
#[must_use]
pub fn llm_limits() -> skein_fake_llm_protocol::provider::Limits {
    let declared = skein_llm::Declared {
        window: 171,
        output: 4096,
        reasoning_item: 2048,
        tool_payload: 342,
        calls_per_response: 7,
        conversations: 1,
        tools: 174,
        history_items: 684,
    };
    let bounds = skein_llm::Limits::derive(&declared).expect("tiny declaration").dialect(skein_llm::Provider::OpenAi);
    let documents = skein_llm::openai::Limits { answer: bounds.answer.max(32768), ..bounds.native() };
    let mut protocol = skein_fake_llm_protocol::provider::Limits {
        calls: 4,
        http: skein_http::server::Limits {
            head: 4096,
            headers: 32,
            body: 65_536,
            read: 256,
            response: 4096,
            send: 128,
        },
        sse: skein_http::sse::writer::Limits { event: 16384, chunk: 128 },
        documents: skein_fake_llm_protocol::documents::Limits {
            anthropic: documents,
            openai: documents,
            model_ceiling: 4096,
        },
    };
    protocol.documents.openai.request = 65_536;
    protocol.documents.openai.strings = 16_384;
    protocol.documents.openai.tokens = 4096;
    protocol.documents.openai.output_items = 256;
    protocol.documents.anthropic.output_items = 256;
    protocol
}
/// The LLM peer's transport and observation limits.
#[must_use]
pub fn llm_peer_limits() -> skein_fake_peers::Limits {
    skein_fake_peers::Limits {
        io: skein_io::Limits {
            sockets: 8,
            intake: 32_768,
            output: 65_536,
            close_timeout: Duration::from_millis(50),
            ..crate::documents::base_agent_limits().io
        },
        connections: 4,
        queue: 256,
        plaintext: 32_768,
        ciphertext: 65_536,
        observations: 256,
        observation_bytes: 4_194_304,
    }
}
