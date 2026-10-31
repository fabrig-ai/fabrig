//! Small translation and component presets (smith/docs/design/protocol/llm.md, sections 8 and 10).
//! The caller supplies its dependency bounds and stated quantities; this crate
//! owns the remaining admission capacities and their receiving derivation.
use crate::{ComponentLimits, Limits, Receiving, Stated};
use skein_lib::Duration;
use skein_llm::client;

/// Small translation capacities with caller-owned client and tool declarations.
#[must_use]
pub fn small(client: client::Limits, stated: Stated) -> Limits {
    Limits {
        client,
        tool_bytes: 32_768,
        history_bytes: u64::from(client.request),
        rendered_result: client.strings,
        stated,
    }
}

/// Receiving capacities derived from the client's complete output ownership.
#[must_use]
pub fn receiving(client: &client::Limits, decoded_call_bytes: u64) -> Option<Receiving> {
    Some(Receiving {
        max_completion_bytes: crate::completion_bound(client, decoded_call_bytes)?,
        max_completion_blocks: client.output_items,
        decoded_call_bytes,
        max_failure_bytes: client.detail_bytes,
    })
}

/// Two concurrent calls through one endpoint, with the caller's idle policy.
#[must_use]
pub fn component(adapter: Limits, idle_keep: Duration) -> Option<ComponentLimits> {
    Some(ComponentLimits {
        adapter,
        connection: skein_llm_connection::Limits {
            memory: 1 << 40,
            endpoints: 1,
            connections: 2,
            calls: 2,
            per_endpoint: 2,
            idle_keep,
            io: skein_io::Limits {
                sockets: 3,
                names: 0,
                refusals: 1,
                intake: 19_000,
                receive: 1024,
                output: 19_000,
                sends: 2,
                accepts: 1,
                backlog: 2,
                close_timeout: Duration::from_secs(1),
                retry: Duration::from_millis(10),
            },
            tls: skein_tls::client::Limits {
                read: client::largest_read(&adapter.client),
                send: 4096,
                records: skein_tls::client::MAX_RECORD,
            },
        },
        receiving: receiving(&adapter.client, 4096)?,
        contract_bytes: 4096,
        llms: 1,
        accounts: 1,
        grant_value_bytes: 128,
    })
}
