#![expect(
    clippy::disallowed_macros,
    reason = "ordinary tests edit startup documents and compare their diagnostic lines"
)]
use super::stock;
use crate::startup;
use jig_protocol_config as config;
use serde_json::{Value, json};

fn engine(value: &Value, directories: &config::Directories) -> Result<config::Engine, config::Problems> {
    let bytes = serde_json::to_vec(value).unwrap();
    let document =
        config::document(&bytes, &config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 }).unwrap();
    startup::engine(&document, 0, directories)
}
#[test]
fn moved_engine_members_print_their_destinations() {
    let document: Value = serde_json::from_slice(&stock::generated()).unwrap();
    for (member, words) in [
        ("deployment", "moved: temper keeps the store in the data directory's state"),
        ("providers", "moved: temper fixes its identity providers: 1 link, 2 deployment, 3 Google's"),
        ("connectors", "moved: temper fixes its connector, the forge's, 1"),
        ("acts", "moved: temper fixes the forge connector's acts"),
        ("projects", "moved: projects are made at runtime; make one from the client or temper ctl"),
        ("clients.heartbeat_ms", "moved: temper fixes the clients' heartbeat, printed link and approval"),
        ("clients.link", "moved: temper fixes the clients' heartbeat, printed link and approval"),
        ("clients.approval", "moved: temper fixes the clients' heartbeat, printed link and approval"),
    ] {
        let mut value = document["engine"].clone();
        if let Some(name) = member.strip_prefix("clients.") {
            value["clients"][name] = json!(true);
        } else {
            value[member] = json!(null);
        }
        let found = engine(&value, &config::Directories::working()).unwrap_err();
        let found = startup::Problems::jig(found, &config::Path::root().member(b"engine"));
        assert_eq!(startup::lines(&found).as_ref(), format!("engine.{member}: {words}\n").as_bytes());
    }
}
#[test]
fn temper_constants_pass_the_whole_check_with_stock_limits_in_both_shapes() {
    let document: Value = serde_json::from_slice(&stock::generated()).unwrap();
    let directories = config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    for listener in [json!([{"in_process":"workers"}]), json!([{"socket":"127.0.0.1:0"}])] {
        let mut value = document["engine"].clone();
        value["hosts"]["listener"] = listener;
        value.as_object_mut().unwrap().remove("clients");
        let engine = engine(&value, &directories).unwrap();
        assert_eq!(engine.deployment.directory.as_ref(), b"/data/state");
        assert_eq!(engine.deployment.seed, Some(7));
        assert_eq!(
            engine.providers.as_ref(),
            [
                config::IdentityProvider { number: 1, kind: config::ProviderKind::Link },
                config::IdentityProvider { number: 2, kind: config::ProviderKind::Deployment },
                config::IdentityProvider { number: 3, kind: config::ProviderKind::Application },
            ]
        );
        assert_eq!(engine.connectors.as_ref(), [1]);
        assert_eq!(
            engine.acts.as_ref(),
            [jig_domain_people::ActGrant {
                connector: 1,
                may: Box::new([
                    jig_domain_people::ActKind::RemoveProject,
                    jig_domain_people::ActKind::RemoveAccount,
                    jig_domain_people::ActKind::Adopt
                ]),
            }]
        );
        assert_eq!(engine.limits.core.people.act_grants, 1);
        assert!(engine.projects.is_empty());
        assert_eq!(engine.clients.heartbeat, skein_lib::Duration::from_millis(10_000));
        assert!(engine.clients.link && engine.clients.approval);
        assert!(engine.clients.provider.is_none());
    }
}

#[test]
fn fleet_grace_rounds_the_stop_bound_and_margin_up_to_a_minute() {
    use skein_lib::Duration;
    for (stop, grace) in [(326, 360), (330, 360), (331, 420)] {
        assert_eq!(startup::fleet_grace(Duration::from_secs(stop)), Duration::from_secs(grace));
    }
}
#[test]
fn the_fleet_uses_the_link_stop_bound_unless_an_override_is_larger() {
    let document: Value = serde_json::from_slice(super::fixtures::ENGINE).unwrap();
    for (ping, silence, offered, derived) in [
        (5000_u64, 15000_u64, None, 360_000),
        (5000, 15000, Some(420_000), 360_000),
        (5000, 15000, Some(359_999), 360_000),
        (5000, 15000, Some(0), 360_000),
        (5000, 15000, Some(360_000), 360_000),
        (7000, 21000, None, 420_000),
    ] {
        let mut value = document.clone();
        value["limits"]["overrides"]["link.ping_ms"] = json!(ping);
        value["limits"]["overrides"]["link.silence_ms"] = json!(silence);
        value["limits"]["overrides"].as_object_mut().unwrap().remove("fleet.grace_ms");
        if let Some(offered) = offered {
            value["limits"]["overrides"]["fleet.grace_ms"] = json!(offered);
        }
        let engine = engine(&value, &config::Directories::working()).unwrap();
        let mut problems = startup::Problems::empty();
        startup::check_fleet_grace(&engine, &mut problems);
        if let Some(offered) = offered
            && offered < derived
        {
            assert_eq!(startup::lines(&problems).as_ref(), format!("engine.limits.overrides.fleet.grace_ms: {offered} ms; requires at least the derived fleet grace of {derived} ms\n").as_bytes());
        } else {
            assert!(problems.is_empty());
            assert_eq!(engine.limits.core.fleet.grace.as_millis(), offered.unwrap_or(derived));
        }
    }
}
