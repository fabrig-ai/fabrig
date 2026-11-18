//! Schema texts for the fixed-rule values used by the store codec tests.
#![expect(clippy::disallowed_macros, reason = "tests embed the families' schema texts to build fixed-rule values")]

pub(super) const NOTES: &str = include_str!("../../../../crates/jig-protocol-store/schema/notes-v1.schema");
pub(super) const TASKS: &str = include_str!("../../../../crates/jig-protocol-store/schema/tasks-v13.schema");
pub(super) const CORE: &str = include_str!("../../../../crates/jig-protocol-store/schema/core-v6.schema");
pub(super) const PEOPLE: &str = include_str!("../../../../crates/jig-protocol-store/schema/people-v17.schema");
pub(super) const COMPONENT: &str = include_str!("../../../../crates/jig-protocol-store/schema/component-v3.schema");
pub(super) const ACCOUNTS: &str = include_str!("../../../../crates/jig-protocol-store/schema/accounts-v2.schema");
pub(super) const SECRETS: &str = include_str!("../../../../crates/jig-protocol-store/schema/secrets-v1.schema");
pub(super) const SETTINGS: &str = include_str!("../../../../crates/jig-protocol-store/schema/settings-v3.schema");
pub(super) const TEST_CONNECTOR: &str = include_str!("../../../../testing/jig-test-store/schema/test-connector-v2.schema");
