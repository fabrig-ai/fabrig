//! Codec worlds (temper/docs/design/testing/README.md, 2).

#[path = "../support/store.rs"]
mod fixture;
#[path = "../support/protection.rs"]
mod protection;

mod client_wire;
mod drift;
mod records;
mod store_protection;
mod store_queue;
mod worker_wire;

mod fuzzy {
    mod client_wire;
    mod public_wire;
    mod records;
    mod store;
    mod worker_wire;
}
