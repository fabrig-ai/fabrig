//! Cold store rehearsals (`temper/docs/design/testing/store-check.md`, 2–4).

use jig_conformance::store_check::{StoreChecks, finding};
use skein_lib::Wall;
use temper_engine_domain::{Config, Limits};

use crate::{direct::Driver, oracle::config, store::Store};

/// Rehearse a start over an independent durable image, stopping before the switch.
pub fn store(store: &Store, config: &Config, limits: &Limits, wall: Wall, commit: u64) -> Result<(), String> {
    let mut driver = Driver::rehearsing(store.fork(), config::copy(config), limits, wall);
    let mut quiet = 0;
    loop {
        if driver.root.rehearsed() {
            return Ok(());
        }
        let phase = driver.root.start_phase().map_or_else(|| "None".to_owned(), |phase| format!("{phase:?}"));
        if let Some(refusal) = driver.root.start_stop() {
            return Err(finding(commit, &phase, &format!("{refusal:?}")));
        }
        if quiet == 2 {
            return Err(format!("store check: commit {commit}: start stalled at {phase}"));
        }
        let collected = driver.collected;
        driver.advance(true);
        if collected == driver.collected && driver.events.is_empty() && driver.store.pending.is_empty() {
            quiet += 1;
        } else {
            quiet = 0;
        }
    }
}

/// Count only a newly durable commit, including one whose acknowledgement is held.
pub fn committed(
    store: &Store,
    config: &Config,
    limits: &Limits,
    wall: Wall,
    previous: u64,
    checks: &mut StoreChecks,
) -> Result<(), String> {
    if store.applied != previous && checks.due(store.applied) {
        self::store(store, config, limits, wall, store.applied)?;
    }
    Ok(())
}
