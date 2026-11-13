//! Shared view fixtures staged for the kit (temper/docs/design/client/view-crates.md, 4.2–4.3).

#![expect(
    clippy::unwrap_used,
    clippy::manual_map,
    clippy::needless_pass_by_value,
    clippy::single_match_else,
    reason = "the shared test fixtures retain their setup and semantic inspection unchanged"
)]
#![cfg_attr(
    not(test),
    expect(clippy::wildcard_imports, reason = "the shared test fixtures retain their imports unchanged")
)]
use skein_lib::{Duration, List, RequestLimits, Wall};
use temper_client_domain::{Limits, forge};
use temper_client_view_words::{look, paths, words};
pub use temper_fixtures::first_run;

pub mod accounts;
pub mod cards;
pub mod chat;
pub mod chat_helpers;
pub mod chats;
pub mod choice;
pub mod confirm;
pub mod connect_forge;
pub mod connect_model;
pub mod delegation;
pub mod document;
pub mod goal;
pub mod goals;
pub mod home;
pub mod inbox;
pub mod limits_helpers;
pub mod needs;
pub mod panel;
pub mod public_records;
pub mod rail;
pub mod readers;
pub mod roots;
pub mod task;
pub mod task_helpers;
pub mod team;
pub mod team_sessions;
pub mod terminal;
pub mod welcome;
pub use readers::*;
pub use roots::*;

pub const NOW: Wall = Wall::EPOCH;
pub const FORGE: u16 = temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge);
pub const PROJECT: u32 = 1;
pub const SECOND_PROJECT: u32 = 2;
pub const SEED: u64 = 11;
#[expect(clippy::arithmetic_side_effects, reason = "the request count is the nonzero constant declared here")]
#[must_use]
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
pub fn words() -> crate::words::Words {
    static WORDS: std::sync::OnceLock<crate::words::Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| english(crate::words::Shell::Terminal)).clone()
}
pub fn browser_words() -> crate::words::Words {
    static WORDS: std::sync::OnceLock<crate::words::Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| english(crate::words::Shell::Browser)).clone()
}
#[must_use]
pub fn english(shell: crate::words::Shell) -> crate::words::Words {
    crate::words::Words::english(FORGE, 64, shell).expect("forge English fits")
}

#[must_use]
pub fn tree_limits() -> skein_ui::Limits {
    skein_ui::Limits {
        // The largest goal plan and its frame, including Changes' link and text,
        // and five nodes for the rail menu wrapper, menu and three items.
        // The default draft entry adds a goal link and its text.
        // Seven decided rows each add their Allowed sentence; Work and Deployment add their frame nodes.
        nodes: 224,
        depth: 32,
        text_per_node: 512,
        text_bytes: 98304,
        drawing_bytes: crate::paths::DATA,
        scene_marks: crate::paths::MARKS,
        scene_bytes: crate::paths::SCENE_BYTES,
        styles_per_node: 5,
        patches: 960,
        bindings: 128,
        slots: 32,
        markdown_depth: 3,
    }
}
#[must_use]
pub fn domain_limits() -> Limits {
    limits()
}
#[must_use]
pub fn path(segments: &[&[u8]]) -> List<Box<[u8]>> {
    let mut path = List::with_capacity(u32::try_from(segments.len()).expect("fixture segments fit"));
    for segment in segments {
        path.push(skein_lib::bytes::copy_of(segment)).expect("fixture path fits");
    }
    path
}
