//! Startup limits and edge validation (llm/connection.md, section 7).

use skein_lib::{Duration, List};
use skein_llm as llm;
use skein_tls::client as tls;

use crate::endpoint::{EndpointError, Transport};

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
    check(limits, endpoints).is_ok()
}

pub(crate) fn check(limits: &Limits, endpoints: &List<crate::Endpoint>) -> Result<(), EndpointError> {
    if endpoints.len() > limits.endpoints {
        return Err(EndpointError::TooMany);
    }
    if limits.endpoints == 0
        || limits.connections == 0
        || limits.calls == 0
        || limits.per_endpoint == 0
        || limits.per_endpoint > limits.connections
    {
        return Err(EndpointError::Limits);
    }
    if limits.connections < limits.calls {
        return Err(EndpointError::ConnectionsCalls { connections: limits.connections, calls: limits.calls });
    }
    for endpoint in endpoints {
        let mut client = endpoint.limits;
        client.http.request = match llm::client::request_head(&endpoint.llm, &endpoint.credential, &client) {
            Ok(head) => head,
            Err(error) => return Err(EndpointError::Client(error)),
        };
        let largest = llm::client::largest_reservation(&client).ok_or(EndpointError::Limits)?;
        if largest > limits.memory {
            return Err(EndpointError::MemoryLargestCall { memory: limits.memory, call: largest });
        }
        let read = llm::client::largest_read(&client);
        let send = llm::client::largest_room(&client);
        let sse = client.sse.chunk;
        if sse > client.http.read {
            return Err(EndpointError::SseChunkHttpRead { demand: sse, cap: client.http.read });
        }
        let tokenizer = llm::client::largest_event_demand(&client);
        if tokenizer > sse {
            return Err(EndpointError::TokenizerDemandSseChunk { demand: tokenizer, cap: sse });
        }
        match &endpoint.transport {
            Transport::Tls { .. } => {
                if read > limits.tls.read {
                    return Err(EndpointError::HttpReadTlsRead { demand: read, cap: limits.tls.read });
                }
                if send > limits.tls.send {
                    return Err(EndpointError::HttpSendTlsSend { demand: send, cap: limits.tls.send });
                }
                if tls::LARGEST_READ > limits.io.largest_read() {
                    return Err(EndpointError::TlsReadIoIntake {
                        demand: tls::LARGEST_READ,
                        cap: limits.io.largest_read(),
                    });
                }
                if tls::largest_room(&limits.tls) > limits.io.largest_room() {
                    return Err(EndpointError::TlsSendIoOutput {
                        demand: tls::largest_room(&limits.tls),
                        cap: limits.io.largest_room(),
                    });
                }
            }
            Transport::Plaintext => {
                if !endpoint.address.ip().is_loopback() {
                    return Err(EndpointError::PlaintextAddress);
                }
                if read > limits.io.largest_read() {
                    return Err(EndpointError::HttpReadIoIntake { demand: read, cap: limits.io.largest_read() });
                }
                if send > limits.io.largest_room() {
                    return Err(EndpointError::HttpSendIoOutput { demand: send, cap: limits.io.largest_room() });
                }
            }
        }
    }
    for endpoint in endpoints {
        let mut client = endpoint.limits;
        client.http.request = match llm::client::request_head(&endpoint.llm, &endpoint.credential, &client) {
            Ok(head) => head,
            Err(error) => return Err(EndpointError::Client(error)),
        };
        if !llm::client::valid(&client) {
            return Err(EndpointError::Limits);
        }
        match &endpoint.transport {
            Transport::Tls { .. } => {
                if !tls::valid(&limits.tls) {
                    return Err(EndpointError::Limits);
                }
            }
            Transport::Plaintext => {}
        }
    }
    Ok(())
}
