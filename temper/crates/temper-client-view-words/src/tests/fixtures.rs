//! The vocabulary tests keep their own fixtures, without the kit (view-crates.md, 4.2).
use skein_lib::{Duration, RequestLimits};
use temper_client_domain::{Limits, forge};

const FORGE: u16 = temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge);

#[expect(clippy::arithmetic_side_effects, reason = "the request count is the nonzero constant declared here")]
pub(super) fn limits() -> Limits {
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
                // The page handles, project-needs, session Away and Home decisions watches, trials and a running trial task.
                handles: 21, // Four findings subjects and one live-goals menu.
                watches: 23, // Four findings subjects and one live-goals menu.
                turn_bytes: 65536,
                backoff: jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                authority: 12,
                heartbeat: Duration::from_millis(100),
                reads: 8,
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

pub(super) fn words() -> crate::words::Words {
    static WORDS: std::sync::OnceLock<crate::words::Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| english(crate::words::Shell::Terminal)).clone()
}

pub(super) fn browser_words() -> crate::words::Words {
    static WORDS: std::sync::OnceLock<crate::words::Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| english(crate::words::Shell::Browser)).clone()
}

pub(super) fn english(shell: crate::words::Shell) -> crate::words::Words {
    crate::words::Words::english(FORGE, 64, shell).expect("forge English fits")
}

pub(super) fn domain_limits() -> Limits {
    limits()
}
