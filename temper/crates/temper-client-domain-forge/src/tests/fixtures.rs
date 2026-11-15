//! Private forge test builders (client/domain-crates.md, 4.2).
use crate::{Limits, forge};
use skein_lib::{Duration, Env, RequestLimits, Time, Wall};
pub const NOW: Wall = Wall::EPOCH;
pub const FORGE: u16 = temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge);
pub const PROJECT: u32 = 1;
pub const SEED: u64 = 11;
#[expect(clippy::arithmetic_side_effects, reason = "the request count is the nonzero constant declared here")]
pub fn limits() -> Limits {
    {
        let requests = RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        };
        let client = jig_client::Limits {
            domain: jig_client::DomainLimits {
                accounts: jig_client::AccountCounts { accounts: 16, sources: 8, sign_ins: 4 },
                owners: &[jig_client::Spoken {
                    family: temper_client_protocol_wire::FORGE,
                    version: temper_client_protocol_wire::CLIENT_VERSION,
                }],
                requests,
                ask_bytes: requests.bytes / u64::from(requests.requests),
                // Page handles, stock ways, project needs and the goals fact, trials and a running trial task.
                handles: 24, // Four findings subjects and one live-goals menu.
                watches: 25, // Four findings subjects and one live-goals menu.
                turn_bytes: 65536,
                backoff: jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                authority: 12,
                heartbeat: Duration::from_millis(100),
                reads: 9, // The rail's project people read and eight page reads.
                parts: 8,
                delegates: 2,
                document_bytes: 65536,
                records: 2,
                kept: 4,
                projects: 2,
                text: 32,
                long_text: 32,
                turn_text: 32,
                rows: 2,
                fields: 4,
                segments: 9,
            },
            exchanges: 8,
        };

        Limits { forge: forge::Limits { field_bytes: 256, checks: 8 }, client, row_uses: 8, window_bytes: None }
    }
}
pub fn env(limits: &Limits) -> Env<Limits> {
    Env { now: Time::ZERO, wall: NOW, limits: *limits }
}
pub const PARTY: u64 = 73;
pub mod acts;
pub mod chat;
pub mod panel;
