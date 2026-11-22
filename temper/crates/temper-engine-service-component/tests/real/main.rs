//! Store permissions on the real kernel (jig/docs/design/protocol/store/README.md, 6.3).
#![cfg(test)]
use temper_engine_service_component::{limits, settings, store};
use temper_engine_service_stock::stock;

#[path = "../../src/tests/fixtures/disk.rs"]
mod fixtures;

mod setup {
    mod tests {
        mod admin_tests;
    }
}
