//! Composite bounds agree with their children (jig/docs/design/domain/engine/README.md, section 7, and jig/docs/design/domain/engine/settings.md, section 8).

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
fn each_wait_figure_must_be_positive() {
    use crate::Figures;
    use skein_lib::Duration;

    let limits = composite();
    let stock = limits.waits;
    let zero = Duration::ZERO;
    for (name, waits) in [
        ("question", Figures { question: zero, ..stock }),
        ("party", Figures { party: zero, ..stock }),
        ("news", Figures { news: zero, ..stock }),
        ("host", Figures { host: zero, ..stock }),
        ("account", Figures { account: zero, ..stock }),
        ("step", Figures { step: zero, ..stock }),
        ("room", Figures { room: zero, ..stock }),
        ("sign_off", Figures { sign_off: zero, ..stock }),
    ] {
        assert!(!crate::valid(&crate::Limits { waits, ..limits }), "{name}");
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

#[test]
fn asking_has_one_distinct_family_bit() {
    use crate::{ToolFamilies, authority};

    assert_eq!(ToolFamilies::JIG.ask, authority::Tools(2048));
    assert!(ToolFamilies::JIG.valid());
    for (ask, valid) in [(2048, true), (1024, false), (0, false), (3072, false)] {
        let tools = ToolFamilies { ask: authority::Tools(ask), ..ToolFamilies::JIG };
        assert_eq!(tools.valid(), valid, "ask = {ask}");
    }
}
