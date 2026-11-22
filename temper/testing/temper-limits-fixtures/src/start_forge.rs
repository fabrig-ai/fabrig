//! Forge values for the root's held-start fixtures.
use alloc::boxed::Box;
use temper_engine_domain_forge as forge;

#[must_use]
pub fn configured_forges() -> Box<[forge::Configured]> {
    Box::new([forge::Configured {
        number: 1,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Connected,
        admin_connect: false,
        bot: Box::new([]),
        owned: None,
        git: Box::new([]),
        web: None,
        read_only: false,
        asked: false,
    }])
}

#[must_use]
pub fn repository() -> temper_engine_domain_forge::Repository {
    use temper_engine_domain_forge as forge;
    forge::Repository {
        contexts: Box::new([]),
        landing: b"main".as_slice().into(),
        workflows: Box::default(),
        project: 1,
        home: false,
        provider: skein_forge::api::Repository { forge: 1, repository: 1 },
        host: b"forge.invalid".as_slice().into(),
        owner: b"owner".as_slice().into(),
        name: b"repo".as_slice().into(),
        prefix: Box::new([]),
        role: forge::Role::Context,
        ci: false,
        checks: Box::new([]),
        kinds: forge::Kinds {
            read: false,
            push: false,
            open: false,
            land: false,
            review: false,
            status: false,
            comment: false,
            issue: false,
            branch: false,
        },
        protection: forge::Protection::Unknown,
        settings: skein_forge::api::Settings {
            default_branch: b"main".as_slice().into(),
            merge: true,
            squash: false,
            rebase: false,
        },
    }
}

#[must_use]
pub fn uncertain(repository: skein_forge::api::Repository) -> skein_forge_client::Entry {
    use skein_forge_client::{Attempt, Condition, Effect, Entry, Position};
    Entry {
        number: 1,
        task: 1,
        repository,
        effect: Effect {
            write: skein_forge::api::Write::OpenPull {
                title: b"start stop".as_slice().into(),
                body: Box::new([]),
                head: b"topic".as_slice().into(),
                base: b"main".as_slice().into(),
            },
            condition: Condition::None,
        },
        start: Some(Position { at: skein_lib::Time::ZERO, comment: 0, review: 0, review_page: 1 }),
        attempt: Some(Attempt {
            sent: skein_lib::Time::ZERO,
            deadline: skein_lib::Time::from_nanos(100),
            wall: skein_lib::Wall::EPOCH,
            expires: skein_lib::Wall::from_nanos(100),
        }),
        failures: 0,
        queued: false,
    }
}

#[must_use]
pub fn status(repository: skein_forge::api::Repository) -> skein_forge_client::Entry {
    let mut entry = uncertain(repository);
    entry.number = 2;
    entry.start = None;
    entry.attempt = None;
    entry.effect.write = skein_forge::api::Write::Status {
        commit: [3; 32],
        context: b"stop".as_slice().into(),
        check: skein_forge::api::Check::Passed,
        description: Box::new([]),
        link: Box::new([]),
    };
    entry
}
