//! Durable removal cuts (temper/docs/design/draft/playground-reset.md, 4 and 8).
use crate::{direct::Driver, forge_stand::ForgeStand, store::Store};
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::Wall;
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;

/// The connector's typed rows, read from the real store's committed mirror.
#[must_use]
pub fn forge_rows(store: &Store) -> Vec<forge::Stored> {
    store
        .rows
        .values()
        .filter_map(|record| match record {
            engine::Record::Forge { row } => Some(*row.clone()),
            engine::Record::Core(_) => None,
        })
        .collect()
}

/// Acts saved by decisions, including those whose answers have already erased them.
#[must_use]
pub fn saved_acts(driver: &Driver) -> Vec<forge::KeptAct> {
    driver
        .transactions
        .iter()
        .flatten()
        .filter_map(|write| {
            if let engine::Write::Save(engine::Record::Forge { row }) = write
                && let forge::Stored::KeptAct(row) = row.as_ref()
            {
                Some(*row)
            } else {
                None
            }
        })
        .collect()
}

/// Acts whose answers have not committed.
#[must_use]
pub fn kept_acts(store: &Store) -> Vec<forge::KeptAct> {
    forge_rows(store)
        .into_iter()
        .filter_map(|row| if let forge::Stored::KeptAct(row) = row { Some(row) } else { None })
        .collect()
}

/// The core's retained keyed answers, independent of the connector's kept rows.
#[must_use]
pub fn acted(store: &Store) -> Vec<([u8; 16], people::Act, people::Outcome)> {
    store
        .rows
        .values()
        .filter_map(|record| {
            if let engine::Record::Core(core::Record::People(people::Stored::Acted { key, act, outcome, .. })) = record
            {
                Some((key.key, act.clone(), outcome.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// Commit the next decision without delivering its terminal to the old process.
pub fn commit(driver: &mut Driver) -> u64 {
    for _ in 0..200 {
        if !driver.store.pending.is_empty() {
            return driver.store.apply();
        }
        driver.advance(false);
    }
    panic!("cut did not reach a decision");
}

/// Crash the typed store and restore a fresh root at the story's unchanged wall.
#[must_use]
pub fn cold(mut store: Store, config: engine::Config, limits: &engine::Limits, seed: u64, wall: Wall) -> Driver {
    store.crash(seed);
    let mut driver = Driver::cold(store, config, limits);
    driver.env.wall = wall;
    driver.env.now = skein_lib::Time::from_nanos(wall.as_nanos());
    driver.period_sent = true;
    driver.forge = Some(ForgeStand::new());
    driver.pump();
    assert!(driver.root.ready() && !driver.stopped, "cold root: {:?}", driver.root.restore_failure());
    driver
}
