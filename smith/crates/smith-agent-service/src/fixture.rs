//! Small admitted service capacities and one model declaration for composition tests.

use alloc::boxed::Box;

/// One native model and its environment declaration for profile scenarios.
#[must_use]
pub fn declarations(environment_bytes: u32) -> crate::profile::Configuration {
    use crate::profile::{Configuration, Endpoint, Model, OversizedReasoning};
    Configuration {
        role_bound: None,
        environment_bytes,
        endpoints: Box::new([Endpoint {
            number: 0,
            provider: skein_llm::Provider::OpenAiCodex,
            option_bytes: 0,
            connect: skein_lib::Duration::from_secs(1),
            handshake: skein_lib::Duration::from_secs(1),
            models: Box::new([Model {
                efforts: Box::new([]),
                name: Box::from(b"fake".as_slice()),
                window: 2_097_152,
                output: 65_536,
                reasoning_item: 2048,
                oversized_reasoning: OversizedReasoning::Fail,
                head: skein_lib::Duration::from_secs(1),
                idle: skein_lib::Duration::from_secs(1),
            }]),
        }]),
    }
}

/// Derive a small service with the same receiving relationships as deployment.
#[must_use]
pub fn small() -> crate::Limits {
    let mut profile = crate::profile::standard();
    profile.declared.memory = u64::MAX;
    profile.declared.conversations = 2;
    profile.declared.calls_per_response = 4;
    profile.declared.jobs = 4;
    profile.declared.tool_payload = 65_536;
    profile.declared.file_bytes = 65_536;
    profile.declared.read_window = 4096;
    profile.declared.shell_output = 512;
    profile.declared.shell_head = 256;
    profile.declared.shell_tail = 256;
    profile.declared.search_hits = 16;
    profile.declared.search_bytes = 1024;
    profile.declared.list_entries = 64;
    profile.policy.parallel_tools = 4;
    profile.policy.check_timeout = skein_lib::Duration::from_secs(30);
    let mut configuration = declarations(0);
    configuration
        .endpoints
        .get_mut(0)
        .expect("one fixture endpoint")
        .models
        .get_mut(0)
        .expect("one fixture model")
        .name = b"fixture-model-capacity".as_slice().into();
    crate::profile::derive(&profile, &configuration).expect("small source declarations satisfy receiving relationships")
}
