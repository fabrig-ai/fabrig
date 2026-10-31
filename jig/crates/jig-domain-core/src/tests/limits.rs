//! Composite bounds agree with their children (jig/docs/design/domain/engine.md, sections 13 and 17.8).

fn composite() -> crate::Limits {
    let mut limits = super::limits();
    limits.people.operations = limits.tasks.tasks;
    assert!(crate::valid(&limits));
    limits
}

#[test]
fn equal_child_limits_are_valid() {
    assert!(crate::valid(&composite()));
}

#[test]
fn task_revisions_must_equal_settings_revisions() {
    for revisions in [3, 5] {
        let mut limits = composite();
        limits.tasks.revisions = revisions;
        assert!(!crate::valid(&limits), "tasks.revisions = {revisions}");
    }
}

#[test]
fn task_versions_must_equal_settings_versions() {
    for versions in [7, 9] {
        let mut limits = composite();
        limits.tasks.versions = versions;
        assert!(!crate::valid(&limits), "tasks.versions = {versions}");
    }
}

#[test]
fn fleet_versions_must_equal_settings_versions() {
    for versions in [7, 9] {
        let mut limits = composite();
        limits.fleet.versions = versions;
        assert!(!crate::valid(&limits), "fleet.versions = {versions}");
    }
}

#[test]
fn document_roles_cover_authority_roles() {
    let mut limits = composite();
    limits.authority.roles = 2;
    for (roles, valid) in [(1, false), (2, true), (3, true)] {
        limits.documents.roles = roles;
        assert_eq!(crate::valid(&limits), valid, "documents.roles = {roles}");
    }
}
