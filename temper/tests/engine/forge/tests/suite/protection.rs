use skein_fake_forge_domain::api as raw;
use skein_forge::api::{self as api, Bypass};
use skein_forge_client::{Condition, Effect, Entry, Outcome};
use temper_engine_domain_forge as top;
use temper_engine_forge_world::{REPO, World};

fn protection(github: bool, users: &[u64]) -> raw::Protection {
    raw::Protection {
        queue: github,
        linear: github,
        code_owners: github,
        bypassing_users: users.into(),
        branch: b"main".as_slice().into(),
        contexts: Box::new([b"build".as_slice().into()]),
        approvals: 2,
        dismiss_stale: true,
    }
}
fn expected(github: bool, signatures: bool, bypass: Bypass) -> top::Protection {
    top::Protection::Rule(api::Protection {
        queue: github,
        signatures,
        linear: github,
        code_owners: github,
        bypass,
        branch: b"main".as_slice().into(),
        contexts: Box::new([b"build".as_slice().into()]),
        approvals: 2,
        dismiss_stale: true,
    })
}
fn kept(world: &World) -> &top::Protection {
    let Some(top::Stored::Repository(row)) = world.stored().get(&top::Key::Repository(REPO)) else {
        panic!("adopted repository");
    };
    &row.protection
}
fn cases() -> [(&'static str, bool, bool, &'static [u64], Bypass); 4] {
    [
        ("GitHub acting user bypasses", true, true, &[1], Bypass::Always),
        ("GitHub another user bypasses", true, false, &[2], Bypass::Never),
        ("Forgejo signed commits", false, true, &[], Bypass::Never),
        ("Forgejo no extra rules", false, false, &[], Bypass::Never),
    ]
}

#[test]
fn adoption_keeps_every_protection_fact_as_answered() {
    for (name, github, signatures, users, bypass) in cases() {
        let mut world = World::with_protection(11, protection(github, users), signatures);
        world.adopt();
        let expected = expected(github, signatures, bypass);
        assert_eq!(kept(&world), &expected, "{name}");
        let adopted = world
            .seen()
            .iter()
            .find_map(|request| {
                if let top::Request::Adopted { result: Ok(adopted), .. } = request { Some(adopted) } else { None }
            })
            .expect("successful adoption");
        assert_eq!(adopted.repository.protection, expected, "{name}: adoption result");
        world.restart();
        assert_eq!(kept(&world), &expected, "{name}: restart");
        assert_eq!(world.writes(), 0, "adoption never changes the forge");
    }
}

#[test]
fn capability_refresh_keeps_every_protection_fact_as_answered() {
    for (name, github, signatures, users, bypass) in cases() {
        let mut world = World::with_protection(12, protection(github, users), !signatures);
        world.adopt();
        assert_eq!(kept(&world), &expected(github, !signatures, bypass), "{name}: before refresh");
        world.signatures(signatures);
        refresh_after_protected_write(&mut world, name);
        let expected = expected(github, signatures, bypass);
        assert_eq!(kept(&world), &expected, "{name}: refreshed rules");
        world.restart();
        assert_eq!(kept(&world), &expected, "{name}: restart");
    }
}

#[test]
fn a_writer_adopts_with_protection_unknown_never_absent() {
    // Forgejo refuses both protection reads to a Write collaborator, rule or none.
    for (name, rule) in [("a rule on main", Some(protection(false, &[]))), ("no rule", None)] {
        let mut world = match rule {
            Some(rule) => World::with_protection(13, rule, false),
            None => World::new(13),
        };
        world.permission(raw::Permission::Write);
        world.adopt();
        assert_eq!(kept(&world), &top::Protection::Unknown, "{name}");
        let adopted = world
            .seen()
            .iter()
            .find_map(|request| {
                if let top::Request::Adopted { result: Ok(adopted), .. } = request { Some(adopted) } else { None }
            })
            .expect("successful writer adoption");
        assert_eq!(adopted.repository.protection, top::Protection::Unknown, "{name}: adoption result");
        assert_eq!(
            adopted.repository.kinds,
            top::Kinds {
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
            "{name}: writer capabilities"
        );
        world.restart();
        assert_eq!(kept(&world), &top::Protection::Unknown, "{name}: restart");
        assert_eq!(world.writes(), 0, "{name}: adoption never changes the forge");
    }
}

#[test]
fn a_writers_capability_refresh_keeps_protection_unknown() {
    let mut world = World::with_protection(14, protection(false, &[]), false);
    world.permission(raw::Permission::Write);
    world.adopt();
    assert_eq!(kept(&world), &top::Protection::Unknown, "before refresh");
    refresh_after_protected_write(&mut world, "writer");
    assert_eq!(kept(&world), &top::Protection::Unknown, "after refresh");
    world.restart();
    assert_eq!(kept(&world), &top::Protection::Unknown, "restart");
}

fn refresh_after_protected_write(world: &mut World, name: &str) {
    world.event(top::Event::Enqueue {
        entry: Entry {
            number: 1,
            task: 9,
            repository: REPO,
            effect: Effect {
                write: api::Write::DeleteBranch { branch: b"main".as_slice().into() },
                condition: Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    });
    world.event(top::Event::Committed { entry: 1 });
    world.run_for(5);
    assert!(
        world.seen().iter().any(|request| matches!(
            request,
            top::Request::Outcome { outcome: Outcome::Failed(api::Error::Protected), .. }
        )),
        "{name}: refusal triggers refresh"
    );
}
