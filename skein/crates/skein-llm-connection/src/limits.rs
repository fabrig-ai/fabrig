//! Startup limits and edge validation (llm-connection.md, section 7).

use skein_lib::{Duration, List};

/// Bounded pool and child machine limits, fixed at startup.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub endpoints: u32,
    pub connections: u32,
    pub calls: u32,
    pub per_endpoint: u32,
    /// Bytes calls may reserve at once from the owner's LLM memory pool.
    pub memory: u64,
    pub idle_keep: Duration,
    pub io: skein_io::Limits,
    pub tls: skein_tls::client::Limits,
}

/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
/// Each endpoint's largest call reservation fits the shared memory pool.
#[must_use]
pub fn valid(limits: &Limits, endpoints: &List<crate::Endpoint>) -> bool {
    if limits.endpoints == 0
        || limits.connections == 0
        || limits.calls == 0
        || limits.per_endpoint == 0
        || limits.connections < limits.calls
        || limits.per_endpoint > limits.connections
        || endpoints.len() > limits.endpoints
    {
        return false;
    }
    for endpoint in endpoints {
        let mut client = endpoint.limits;
        client.http.request = match skein_llm::client::request_head(&endpoint.llm, &endpoint.credential, &client) {
            Ok(head) => head,
            Err(_) => return false,
        };
        if !skein_llm::client::valid(&client) {
            return false;
        }
        match skein_llm::client::largest_reservation(&client) {
            Some(reservation) if reservation <= limits.memory => {}
            Some(_) | None => return false,
        }
        match &endpoint.transport {
            crate::Transport::Tls { .. } => {
                if !skein_tls::client::valid(&limits.tls) {
                    return false;
                }
            }
            crate::Transport::Plaintext => {}
        }
    }
    true
}
