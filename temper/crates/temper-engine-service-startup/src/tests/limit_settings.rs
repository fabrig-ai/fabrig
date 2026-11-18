//! Limit settings resolve within a stock store (configuration/limit-settings.md, 3).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary focused tests edit configuration JSON and inspect diagnostic strings"
)]
use crate::{settings, startup};
use jig_domain_core::settings::Override;
use jig_protocol_config as config;
use skein_lib::{List, Secret};

fn inputs() -> startup::LimitInputs {
    let mut document: serde_json::Value = serde_json::from_slice(&super::stock::generated()).unwrap();
    document["workers"] = serde_json::json!([]);
    document["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:0"}]);
    let bytes = serde_json::to_vec(&document).unwrap();
    let seeds = settings::Seeds {
        deployment: [1; 16],
        installation: [2; 16],
        seed: 3,
        crypto: Secret::new(Box::new([4; 32])),
        link_secret: None,
        local_secret: None,
    };
    startup::LimitInputs::new(
        &bytes,
        &config::Directories::working(),
        &super::stock::files(),
        None,
        &seeds,
        &crate::stock::Given::Stock,
        false,
        None,
    )
}
fn item(name: &[u8], value: u64) -> Override {
    Override { name: name.into(), value, reason: b"measured room".as_slice().into() }
}

#[test]
fn settings_follow_scaled_derivation_and_keep_the_stock_store() {
    let inputs = inputs();
    let stock = startup::resolve_limits(&inputs, &[], &[]).unwrap();
    let resolved = startup::resolve_limits(
        &inputs,
        &[item(b"tasks.tasks", 1024)],
        &[item(b"forge.client.entries", 16), item(b"forge.entries", 16), item(b"live_goals", 20)],
    )
    .unwrap();
    assert_eq!(stock.temper.live_goals, 40);
    assert_eq!(stock.jig.core.tasks.tasks, 768);
    assert_eq!(resolved.temper.live_goals, 20);
    assert_eq!(resolved.jig.core.tasks.tasks, 1024);
    assert_eq!(resolved.jig.core.tasks.project_tasks, 384);
    assert_eq!(resolved.jig.core.people.inbox_tasks, 2048);
    assert_eq!(resolved.temper.forge.tasks, 64);
    assert_eq!(resolved.temper.forge.holds, 80);
    assert_eq!(resolved.temper.forge.entries, 16);
    assert_eq!(resolved.temper.forge.client.entries, 16);
    assert_eq!(resolved.store.store, stock.store.store);
    assert_eq!(resolved.store.secrets, stock.store.secrets);
    assert_eq!(resolved.store.commits, stock.store.commits);
}
#[test]
fn eighty_goals_name_the_setting_and_fixed_store_bound() {
    let found = startup::resolve_limits(
        &inputs(),
        &[item(b"tasks.tasks", 1024)],
        &[item(b"forge.entries", 16), item(b"live_goals", 80)],
    )
    .unwrap_err();
    assert_eq!(startup::lines(&found).as_ref(), b"live_goals (setting temper-limits): the store's map value, 199008107 bytes, past its bound of 100881496 bytes; put limits-held in the data directory to start at the stock\n");
}
#[test]
fn a_record_past_the_stock_store_names_its_setting_and_the_stock_figure() {
    let found = startup::resolve_limits(&inputs(), &[item(b"tasks.project_goals", 160)], &[]).unwrap_err();
    let text = String::from_utf8(startup::lines(&found).into_vec()).unwrap();
    assert!(text.starts_with("tasks.project_goals (setting limits):"), "{text}");
    assert!(text.contains("past its bound of 100881496 bytes"), "{text}");
}
#[test]
fn removed_names_are_left_out_and_worker_names_are_refused() {
    let mut inputs = inputs();
    let resolved = startup::resolve_limits(&inputs, &[], &[item(b"forge.gone", 16)]).unwrap();
    assert_eq!(
        resolved.left_out,
        List::from_boxed(Box::new([startup::LeftOut {
            kind: startup::LimitKind::Temper,
            name: b"forge.gone".as_slice().into()
        }]))
    );
    inputs.is_box = true;
    let found = startup::resolve_limits(&inputs, &[], &[item(b"checkout.repositories", 8)]).unwrap_err();
    assert_eq!(startup::lines(&found).as_ref(), b"checkout.repositories (setting temper-limits): this is a worker's limit; it is not taken from temper-limits; put limits-held in the data directory to start at the stock\n");
}
#[test]
fn a_carrier_below_its_derivation_names_the_supplied_figure() {
    let Err(found) = startup::resolve_limits(
        &inputs(),
        &[item(b"people.words", 512), item(b"tasks.message_bytes", 1024), item(b"tasks.result_bytes", 1024)],
        &[],
    ) else {
        panic!("a carrier below its derivation started");
    };
    assert_eq!(startup::lines(&found).as_ref(), b"tasks.message_bytes (setting limits): 1024 bytes, short of the source's derivation, 63738 bytes; put limits-held in the data directory to start at the stock\ntasks.result_bytes (setting limits): 1024 bytes, short of the source's derivation, 63738 bytes; put limits-held in the data directory to start at the stock\n");
}
#[test]
fn short_figures_name_the_kind_that_supplied_them() {
    let found = startup::resolve_limits(&inputs(), &[item(b"tasks.tasks", 0)], &[]).unwrap_err();
    let text = String::from_utf8(startup::lines(&found).into_vec()).unwrap();
    assert!(text.starts_with("tasks.tasks (setting limits):"), "{text}");
    assert!(text.ends_with("put limits-held in the data directory to start at the stock\n"), "{text}");
    let found = startup::resolve_limits(&inputs(), &[], &[item(b"forge.entries", 0)]).unwrap_err();
    let text = String::from_utf8(startup::lines(&found).into_vec()).unwrap();
    assert!(text.contains("forge.entries (setting temper-limits):"), "{text}");
}
