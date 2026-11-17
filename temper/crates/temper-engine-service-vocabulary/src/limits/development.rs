//! Fabrig's own development, the operator's engine document of 2026-11-16.
//! The stock covers it (temper/docs/design/protocol/configuration/limits.md).
use super::*;

// Document, dotted override name, value. The operator's names stay whole.
const FABRIG_DEVELOPMENT_2026_11_16: &[(&str, &str, u64)] = &[
    ("engine", "engine.limits.overrides.clients.exchanges", 36),
    ("engine", "engine.limits.overrides.clients.streams", 32),
    ("engine", "engine.limits.overrides.clients_http.calls", 36),
    ("engine", "engine.limits.overrides.clients_http.streams", 32),
    ("engine", "engine.limits.overrides.core.resume_bytes", 18_431_846),
    ("engine", "engine.limits.overrides.fleet.configured_workers", 8),
    ("engine", "engine.limits.overrides.fleet.slots", 36),
    ("engine", "engine.limits.overrides.fleet.workers", 3),
    ("engine", "engine.limits.overrides.hub.slots", 36),
    ("engine", "engine.limits.overrides.hub.transcript_bytes", 73_727_378),
    ("engine", "engine.limits.overrides.hub.turn_bytes", 18_431_842),
    ("engine", "engine.limits.overrides.hub.turn_queue_bytes", 147_454_736),
    ("engine", "engine.limits.overrides.tasks.inbox_bytes", 5_226_516),
    ("engine", "engine.limits.overrides.tasks.message_bytes", 63_738),
    ("engine", "engine.limits.overrides.tasks.result_bytes", 63_738),
    ("engine", "engine.limits.overrides.tasks.tree_tasks", 1024),
    ("engine", "engine.limits.overrides.io.output", 18_432_866),
    ("engine", "engine.limits.overrides.tasks.party_placed_holds", 64),
    ("engine", "engine.limits.overrides.tasks.project_placed_holds", 128),
    ("engine", "engine.limits.overrides.tasks.placed_holds", 256),
    ("engine", "engine.limits.overrides.clients.queued_bytes", 4_194_304),
    ("engine", "engine.limits.overrides.people.sign_in_lifetime_ms", 604_800_000),
    ("engine", "engine.limits.overrides.authority.executors", 32),
    ("engine", "engine.limits.overrides.tasks.executor_kinds", 32),
    ("engine", "engine.limits.overrides.core.policy_bytes", 8192),
    ("engine", "engine.limits.overrides.tasks.charters", 24),
    ("engine", "engine.limits.overrides.tasks.tasks", 384),
    ("engine", "engine.limits.overrides.tasks.project_tasks", 384),
    ("engine", "engine.limits.overrides.people.inbox_tasks", 768),
    ("engine", "application.limits.overrides.checkout.message_bytes", 63_740),
    ("engine", "application.limits.overrides.forge.client.op_bytes", 63_738),
    ("engine", "application.limits.overrides.forge.client.repositories", 3),
    ("engine", "application.limits.overrides.forge.judge_projects", 8),
    ("engine", "application.limits.overrides.forge.output", 82),
    ("engine", "application.limits.overrides.forge.repositories", 3),
    ("engine", "application.limits.overrides.forge_component.body_bytes", 127_497),
    ("engine", "application.limits.overrides.root.journal.transcript_bytes", 18_431_846),
    ("engine", "application.limits.overrides.root.loads.bytes", 18_432_090),
    ("engine", "application.limits.overrides.root.loads.read_bytes", 36_863_684),
    ("engine", "application.limits.overrides.root.loads.reply_bytes", 18_432_090),
    ("engine", "application.limits.overrides.root.loads.transcript_bytes", 18_431_842),
    ("engine", "application.limits.overrides.root.journal.run_bytes", 65_536),
    ("engine", "application.limits.overrides.forge.change_policy.questions", 4),
    ("engine", "application.limits.overrides.forge.client.answer_bytes", 1_048_576),
    ("engine", "application.limits.overrides.forge.client.resources", 1024),
    ("engine", "application.limits.overrides.forge.changes", 379),
    ("engine", "application.limits.overrides.forge.subscriptions", 64),
    ("engine", "application.limits.overrides.forge.client.lifetime_ms", 300_000),
    ("engine", "application.limits.overrides.forge_component.deadline_ms", 270_000),
    ("engine", "application.limits.overrides.root.brief.gather_ms", 300_000),
    ("engine", "application.limits.overrides.forge.landing_members", 4),
    ("engine", "application.limits.overrides.forge.resources_per_task", 5),
];

#[test]
fn the_stock_covers_fabrigs_development_of_2026_11_16() {
    use jig_protocol_config as config;
    use temper_service_core::derived::{self, STOCK};
    let document =
        config::document(br#"{"overrides":{}}"#, &config::DocumentLimits { bytes: 1024, tokens: 128, depth: 8 })
            .unwrap();
    let mut base = config::stock();
    derived::engine(&mut base, STOCK).unwrap();
    let parsed = config::overrides::parse_over(&document, 0, &config::Path::root().member(b"limits"), &base);
    assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
    let jig = parsed.limits;
    let mut own = stock();
    derive(&mut own, STOCK).unwrap();
    for (document, path, value) in FABRIG_DEVELOPMENT_2026_11_16 {
        let current = if let Some(name) = path.strip_prefix("engine.limits.overrides.") {
            let mut found = None;
            for row in &config::names::NAMES {
                if row.name == name {
                    found = Some(row);
                    break;
                }
            }
            let row = found.expect("recorded jig name");
            let current = (row.value)(&jig);
            if name == "people.sign_in_lifetime_ms" {
                // Dev's automations keep seven days; the stock keeps a day (limits.md, Sized from what).
                assert_eq!(current, 86_400_000);
                assert_eq!(*value, 604_800_000);
                continue;
            }
            current
        } else {
            let name = path.strip_prefix("application.limits.overrides.").expect("recorded application's name");
            (find(name.as_bytes()).expect("recorded temper name").value)(&own)
        };
        assert!(current >= *value, "{document} {path}: stock {current}, development {value}");
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the source check reads the limits modules whose comments it guards")]
fn stock_constants_cite_bounds_not_test_worlds() {
    for source in [
        include_str!("../../../temper-service-core/src/limits.rs"),
        include_str!("../limits.rs"),
        include_str!("../../../temper-forge-setup/src/limits.rs"),
        include_str!("../../../temper-box-machine/src/limits.rs"),
    ] {
        let production = source;
        for line in production.lines() {
            if !line.trim_start().starts_with("//") {
                continue;
            }
            for citation in [" world", " fixture", " story", "tests/", "from a test", "from the test"] {
                assert!(!line.contains(citation), "test-world source: {line}");
            }
        }
    }
}
