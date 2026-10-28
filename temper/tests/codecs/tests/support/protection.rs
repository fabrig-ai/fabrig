use skein_forge::api::{self as api, Bypass};
use temper_engine_domain_forge as forge;

pub fn rule(flags: u8, bypass: Bypass) -> api::Protection {
    api::Protection {
        branch: b"main".as_slice().into(),
        contexts: Box::new([b"build".as_slice().into()]),
        approvals: 2,
        dismiss_stale: true,
        queue: flags & 1 != 0,
        signatures: flags & 2 != 0,
        linear: flags & 4 != 0,
        code_owners: flags & 8 != 0,
        bypass,
    }
}

pub fn rows(rule: api::Protection) -> [forge::Stored; 3] {
    let repository = forge::Repository {
        project: 5,
        home: true,
        provider: api::Repository { forge: 1, repository: 2 },
        host: b"forge.example".as_slice().into(),
        owner: b"org".as_slice().into(),
        name: b"repo".as_slice().into(),
        prefix: b"temper/".as_slice().into(),
        role: forge::Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: forge::Kinds {
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
        protection: forge::Protection::Rule(rule.clone()),
        settings: api::Settings { default_branch: b"main".as_slice().into(), merge: true, squash: true, rebase: true },
    };
    let adopted = forge::AdoptedRepository {
        repository: repository.provider,
        owner: repository.owner.clone(),
        name: repository.name.clone(),
        role: repository.role,
        home: repository.home,
        kinds: repository.kinds,
        protection: repository.protection.clone(),
    };
    [
        forge::Stored::Call {
            key: forge::NamedCall { task: 1, attempt: 2, completion: 3, position: 4 },
            answer: forge::CallPayload::Read(Box::new(Ok(api::Answer::Protection(Some(rule))))),
        },
        forge::Stored::Repository(repository),
        forge::Stored::Application {
            token: 7,
            until: skein_lib::Wall::from_nanos(8),
            outcome: forge::ApplicationOutcome::Adopted(adopted),
        },
    ]
}
