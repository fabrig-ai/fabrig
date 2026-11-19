//! Schema texts for the fixed-rule values used by the store codec tests.

pub(super) const NOTES: &str = jig_protocol_store::STORE_FAMILIES[6].current;
pub(super) const TASKS: &str = jig_protocol_store::STORE_FAMILIES[4].current;
pub(super) const CORE: &str = jig_protocol_store::STORE_FAMILIES[3].current;
pub(super) const PEOPLE: &str = jig_protocol_store::STORE_FAMILIES[5].current;
pub(super) const COMPONENT: &str = jig_protocol_store::STORE_FAMILIES[0].current;
pub(super) const ACCOUNTS: &str = jig_protocol_store::STORE_FAMILIES[7].current;
pub(super) const SETTINGS: &str = jig_protocol_store::STORE_FAMILIES[8].current;
pub(super) const TEST_CONNECTOR: &str = jig_test_store::SCHEMA;
