//! Local scenario configuration for Skein's shared loopback peer.
//! No transport or provider machinery is owned here (testing.md, section 4).
pub use skein_fake_peers::llm::Peer;
pub use smith_agent_process_world::fake::queries;

/// Script the public Responses route on the simulated world's plaintext peer.
#[must_use]
pub fn peer(
    scripts: Box<[skein_fake_llm_domain::api::Script]>,
    credential: skein_llm::Credential,
    latency: skein_lib::Duration,
) -> Peer {
    smith_agent_process_world::fake::configured_openai(
        scripts,
        credential,
        latency,
        skein_fake_peers::Transport::Plaintext,
    )
}
