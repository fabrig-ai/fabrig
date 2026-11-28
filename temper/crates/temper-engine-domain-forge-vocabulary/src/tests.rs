use crate::*;
mod effect_words;
mod forge_user;
mod history_read;
mod landing_settings;
mod purpose;
mod rules_by_path;
#[test]
fn stored_repository_bytes_count_its_landing() {
    let mut row = adopted();
    let before = stored_bytes(&Stored::Repository(row.clone())).unwrap();
    row.landing = b"staging".as_slice().into();
    assert_eq!(stored_bytes(&Stored::Repository(row)), before.checked_add(3));
}

#[test]
fn effect_lookups_report_declared_writes_and_exclude_other_kinds() {
    use EffectForm::{Creation, Transition};
    use Recovery::{Conditional, Idempotent};
    for (kind, shape, waits) in [
        (0, None, false),
        (1, None, false), // read, not a write
        (3, Some((Creation, Conditional)), false),
        (4, Some((Transition, Conditional)), true),
        (11, Some((Transition, Idempotent)), false),
        (18, None, false),
    ] {
        assert_eq!((effect_shape(kind), waits_for_dependencies(kind)), (shape, waits), "kind {kind}");
    }
}

const REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 2 };
pub(crate) fn adopted() -> Repository {
    Repository {
        contexts: Box::new([]),
        landing: b"main".as_slice().into(),
        refusals: Box::default(),
        workflows: Box::default(),
        project: 5,
        home: true,
        provider: REPO,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: Box::from(&b"main"[..]),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}
