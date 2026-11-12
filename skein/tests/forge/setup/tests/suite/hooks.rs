use skein_fake_forge_forgejo::Route;
use skein_forge::forgejo::{
    delivery::SUBSCRIBED,
    request::{Grant, Page, Request},
};
use skein_forge_setup::{
    item::Item,
    reconcile::{Failure, Stage, Why, hook_url},
};
use skein_forge_setup_world::{Ended, HostWorld, fixtures};
fn listener() -> std::net::SocketAddr {
    "127.1.2.3:8080".parse().expect("loopback")
}
fn url(forge: u16) -> Box<[u8]> {
    hook_url(listener(), forge).expect("loopback")
}
fn world() -> HostWorld {
    let mut world = fixtures::repositories(7, false, &[b"alice/specimen", b"alice/other"]);
    fixtures::bot_tokens(&mut world);
    world.adopted(&[1]);
    world.listening(listener());
    world
}
fn assert_hook(world: &HostWorld, repository: u32, forge: u16) {
    let hooks = world.repository_hooks(repository);
    let own: Vec<_> = hooks.iter().filter(|h| h.url == url(forge)).collect();
    assert_eq!(own.len(), 1);
    let hook = own[0];
    assert!(hook.active);
    assert_eq!(hook.events.iter().map(AsRef::as_ref).collect::<Vec<&[u8]>>(), SUBSCRIBED);
    assert_eq!(&hook.secret, world.file(Item::WebhookSecret).expect("secret item"));
}
fn writes(world: &HostWorld, since: usize) -> Vec<Route> {
    fixtures::write_routes(world, since)
}
fn secret_writes(world: &HostWorld) -> usize {
    world
        .needs()
        .iter()
        .filter(|need| matches!(need, skein_forge_setup::host::Need::Write { item: Item::WebhookSecret, .. }))
        .count()
}
#[test]
fn an_adopted_repository_gets_a_hook_to_the_listener() {
    let mut world = world();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_hook(&world, 1, 1);
    let secret = world.file(Item::WebhookSecret).expect("secret").expose();
    assert_eq!(secret.len(), 64);
    assert!(secret.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)));
    assert!(world.secrets_absent());
}
#[test]
fn a_second_run_only_deletes_and_creates_each_repositorys_hook() {
    let mut world = world();
    world.adopted(&[2, 1]);
    assert_eq!(world.reconcile(), Ended::Done);
    let since = world.requests().len();
    let files = fixtures::file_writes(&world);
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(
        writes(&world, since),
        [Route::DeleteRepoHook, Route::CreateRepoHook, Route::DeleteRepoHook, Route::CreateRepoHook]
    );
    assert_eq!(fixtures::file_writes(&world), files);
}
#[test]
fn a_deleted_hook_is_made_again() {
    let mut world = world();
    assert_eq!(world.reconcile(), Ended::Done);
    let id = world.repository_hooks(1)[0].id;
    world.delete_repository_hook(1, id);
    let since = world.requests().len();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(writes(&world, since), [Route::CreateRepoHook]);
    assert_hook(&world, 1, 1);
    assert_ne!(world.repository_hooks(1)[0].id, id);
}
#[test]
fn a_hook_on_the_second_page_is_neither_repeated_nor_passed_over() {
    let mut world = world();
    for target in [b"http://other/one".as_slice(), b"http://other/two", &url(1)] {
        world.with_hook(1, target, fixtures::secret(b"old-secret"));
    }
    assert_eq!(world.reconcile(), Ended::Done);
    assert_hook(&world, 1, 1);
    assert_eq!(world.repository_hooks(1).len(), 3);
    assert_eq!(writes(&world, 0), [Route::AddCollaborator, Route::DeleteRepoHook, Route::CreateRepoHook]);
    assert_eq!(
        world
            .requests()
            .iter()
            .filter(|r| matches!(r, Request::RepoHooks { page: Page { page: Some(2), limit: Some(2) }, .. }))
            .count(),
        2,
    );
}
#[test]
fn a_hook_deleted_on_page_two_does_not_skip_the_next_row() {
    let mut world = world();
    for target in [b"http://other/one".as_slice(), b"http://other/two"] {
        world.with_hook(1, target, fixtures::secret(b"other-secret"));
    }
    let old: Vec<_> = (0..3).map(|_| world.with_hook(1, &url(1), fixtures::secret(b"old-secret"))).collect();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_hook(&world, 1, 1);
    assert_eq!(world.repository_hooks(1).len(), 3);
    assert!(world.repository_hooks(1).iter().all(|hook| !old.contains(&hook.id)));
    assert_eq!(
        writes(&world, 0),
        [
            Route::AddCollaborator,
            Route::DeleteRepoHook,
            Route::DeleteRepoHook,
            Route::DeleteRepoHook,
            Route::CreateRepoHook,
        ]
    );
}
#[test]
fn two_hooks_with_the_listeners_url_are_both_deleted_then_one_is_created() {
    let mut world = world();
    let old = [
        world.with_hook(1, &url(1), fixtures::secret(b"old-secret")),
        world.with_hook(1, &url(1), fixtures::secret(b"old-secret")),
    ];
    assert_eq!(world.reconcile(), Ended::Done);
    assert_hook(&world, 1, 1);
    assert!(!old.contains(&world.repository_hooks(1)[0].id));
    assert_eq!(
        writes(&world, 0),
        [Route::AddCollaborator, Route::DeleteRepoHook, Route::DeleteRepoHook, Route::CreateRepoHook]
    );
}
#[test]
fn a_hook_of_another_url_is_left_alone() {
    let mut world = world();
    let other = world.with_hook(1, b"http://other", fixtures::secret(b"other-secret"));
    assert_eq!(world.reconcile(), Ended::Done);
    let hooks = world.repository_hooks(1);
    assert_eq!(hooks.len(), 2);
    assert_eq!(hooks[0].id, other);
    assert_eq!(hooks[0].secret, fixtures::secret(b"other-secret"));
    assert!(!hooks[0].active);
    assert!(hooks[0].events.is_empty());
}
#[test]
fn an_off_loopback_listener_makes_no_hook_or_secret() {
    let mut world = world();
    world.listening("192.0.2.1:8080".parse().expect("off loopback"));
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(world.file(Item::WebhookSecret), None);
    assert!(!world.requests().iter().any(|r| matches!(
        r,
        Request::RepoHooks { .. }
            | Request::CreateRepoHook { .. }
            | Request::EditRepoHook { .. }
            | Request::DeleteRepoHook { .. }
    )));
}
#[test]
fn a_webhook_secret_is_drawn_when_missing() {
    let mut world = world();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(secret_writes(&world), 1);
    let secret = world.file(Item::WebhookSecret).expect("secret").expose();
    assert_eq!(secret.len(), 64);
    assert!(secret.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)));
    assert_hook(&world, 1, 1);
}
fn stale_hook() -> (HostWorld, u64) {
    let mut world = world();
    world.store_mut().grant(1, b"kit", Grant::Write).expect("collaborator");
    let old = world.with_hook(1, &url(1), fixtures::secret(b"old-secret"));
    world.put(Item::WebhookSecret, fixtures::secret(b"seeded-webhook-secret"));
    (world, old)
}
#[test]
fn a_hook_with_another_secret_is_remade_with_the_kept_webhook_secret() {
    let (mut world, old) = stale_hook();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(secret_writes(&world), 0);
    assert_hook(&world, 1, 1);
    assert_ne!(world.repository_hooks(1)[0].id, old);
    assert_eq!(world.repository_hooks(1)[0].secret, fixtures::secret(b"seeded-webhook-secret"));
    assert_eq!(writes(&world, 0), [Route::DeleteRepoHook, Route::CreateRepoHook]);
    assert!(world.secrets_absent());
}
#[test]
fn a_refused_hook_deletion_is_finished_by_the_next_run() {
    let (mut world, old) = stale_hook();
    world.refuse(Route::DeleteRepoHook);
    assert_eq!(world.reconcile(), refused_hook());
    let hooks = world.repository_hooks(1);
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].id, old);
    assert_eq!(hooks[0].secret, fixtures::secret(b"old-secret"));
    assert_eq!(writes(&world, 0), [Route::DeleteRepoHook]);
    assert!(world.secrets_absent());
    world.permit();
    let since = world.requests().len();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_hook(&world, 1, 1);
    assert_ne!(world.repository_hooks(1)[0].id, old);
    assert_eq!(writes(&world, since), [Route::DeleteRepoHook, Route::CreateRepoHook]);
}
#[test]
fn a_refused_hook_creation_is_finished_by_the_next_run() {
    let (mut world, old) = stale_hook();
    world.refuse(Route::CreateRepoHook);
    assert_eq!(world.reconcile(), refused_hook());
    assert!(world.repository_hooks(1).is_empty());
    assert_eq!(writes(&world, 0), [Route::DeleteRepoHook, Route::CreateRepoHook]);
    assert!(world.secrets_absent());
    world.permit();
    let since = world.requests().len();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_hook(&world, 1, 1);
    assert_ne!(world.repository_hooks(1)[0].id, old);
    assert_eq!(writes(&world, since), [Route::CreateRepoHook]);
}
fn refused_hook() -> Ended {
    Ended::Stop(Failure {
        forge: 1,
        step: Stage::Repository,
        subject: b"alice/specimen".as_slice().into(),
        why: Why::Refused(403),
    })
}
#[test]
fn the_secret_is_asked_once_for_many_repositories() {
    let mut world = world();
    world.adopted(&[2, 1]);
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(secret_writes(&world), 1);
    assert_eq!(world.reads().iter().filter(|item| **item == Item::WebhookSecret).count(), 1);
    for id in [1, 2] {
        assert_hook(&world, id, 1);
    }
}
#[test]
fn two_services_remake_only_their_own_hook_on_one_repository() {
    let mut world = world();
    world.service_word(b"alpha");
    assert_eq!(world.reconcile(), Ended::Done);
    let alpha_secret = world.file(Item::WebhookSecret).expect("alpha secret").clone();
    let alpha_id = world.repository_hooks(1)[0].id;
    let alpha_files: Vec<_> = [Item::ApiToken, Item::GitToken, Item::User]
        .into_iter()
        .map(|item| (item, world.file(item).expect("alpha item").clone()))
        .collect();
    world.service_word(b"beta");
    world.forge_number(2);
    assert_eq!(world.reconcile(), Ended::Done);
    assert_hook(&world, 1, 2);
    let beta_secret = world.file(Item::WebhookSecret).expect("beta secret").clone();
    assert_ne!(alpha_secret, beta_secret);
    let alpha = world.repository_hooks(1).into_iter().find(|h| h.url == url(1)).expect("alpha");
    assert_eq!(alpha.id, alpha_id);
    assert_eq!(alpha.secret, alpha_secret);
    let beta_id = world.repository_hooks(1).into_iter().find(|h| h.url == url(2)).expect("beta").id;
    let since = world.requests().len();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(writes(&world, since), [Route::DeleteRepoHook, Route::CreateRepoHook]);
    let alpha = world.repository_hooks(1).into_iter().find(|h| h.url == url(1)).expect("alpha");
    assert_eq!(alpha.id, alpha_id);
    assert_eq!(alpha.secret, alpha_secret);
    let beta_id_after = world.repository_hooks(1).into_iter().find(|h| h.url == url(2)).expect("beta").id;
    assert_ne!(beta_id_after, beta_id);
    world.service_word(b"alpha");
    world.forge_number(1);
    for (item, bytes) in alpha_files {
        world.put(item, bytes);
    }
    world.put(Item::WebhookSecret, alpha_secret.clone());
    let since = world.requests().len();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(writes(&world, since), [Route::DeleteRepoHook, Route::CreateRepoHook]);
    let hooks = world.repository_hooks(1);
    assert_eq!(hooks.len(), 2);
    let alpha = hooks.iter().find(|h| h.url == url(1)).expect("alpha");
    let beta = hooks.iter().find(|h| h.url == url(2)).expect("beta");
    assert_ne!(alpha.id, alpha_id);
    assert_eq!(alpha.secret, alpha_secret);
    assert_eq!(beta.id, beta_id_after);
    assert_eq!(beta.secret, beta_secret);
    assert!(world.secrets_absent());
}
#[test]
fn adopt_makes_the_hook_and_the_reconcile_after_it_remakes_it() {
    let mut world = world();
    assert_eq!(world.adopt(1), Ended::Done);
    assert_hook(&world, 1, 1);
    let since = world.requests().len();
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(writes(&world, since), [Route::DeleteRepoHook, Route::CreateRepoHook]);
}
#[test]
fn no_secret_is_in_a_hook_runs_failure_debug_or_trace() {
    let mut world = world();
    world.refuse(Route::CreateRepoHook);
    assert_eq!(
        world.reconcile(),
        Ended::Stop(Failure {
            forge: 1,
            step: Stage::Repository,
            subject: b"alice/specimen".as_slice().into(),
            why: Why::Refused(403)
        })
    );
    assert!(world.file(Item::WebhookSecret).is_some());
    assert!(world.secrets_absent());
}

#[test]
fn the_host_adopt_waits_for_the_secret_write_then_makes_the_hook_before_done() {
    use core::num::NonZeroU64;
    use skein_forge_setup::{
        host::{Need, Plan},
        reconcile::Outcome,
    };
    use skein_forge_setup_world::{Heard, HostWorld};
    use skein_lib::{Duration, Token};

    let mut settings = fixtures::host_settings();
    match &mut settings.forges.get_mut(0).expect("one forge").kind {
        skein_forge_setup::host::Kind::Forgejo(given) => given.listener = Some(listener()),
        skein_forge_setup::host::Kind::GitHub(_) => panic!("Forgejo fixture"),
    }
    let mut world = HostWorld::with_settings(7, &[], fixtures::repository_config(&[b"alice/specimen"]), settings);
    fixtures::seed_bot_tokens(world.store_mut());
    world.pause_writes();
    let plan = Plan::Adopt {
        forge: 1,
        repository: skein_forge_setup::host::Repository::Id(NonZeroU64::new(1).expect("repository")),
    };
    world.run_plan(1, plan.clone());
    world.settle();
    assert!(world.heard().is_empty());
    assert!(world.file(Item::WebhookSecret).is_none());
    assert_eq!(world.store().repository_hooks(1).count(), 0);
    assert_eq!(world.tally().routes.get(&Route::ApiSettings), None);
    let Some(Need::Write { item: Item::WebhookSecret, bytes }) = world.needs().last() else {
        panic!("webhook secret write")
    };
    assert_eq!(bytes.len(), 64);
    assert!(bytes.expose().iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)));
    world.write_pending();
    world.finish();
    let hook_id = {
        let hooks: Vec<_> = world.store().repository_hooks(1).collect();
        assert_eq!(hooks.len(), 1);
        let hook = hooks[0];
        assert_eq!(hook.url, url(1));
        assert!(hook.active);
        assert_eq!(hook.events.iter().map(AsRef::as_ref).collect::<Vec<&[u8]>>(), SUBSCRIBED);
        assert_eq!(&hook.secret, world.file(Item::WebhookSecret).expect("written secret"));
        hook.id
    };
    assert_eq!(world.heard(), &[Heard::Done { run: Token::new(1), outcome: Outcome::Nothing, at: Duration::ZERO }]);
    assert_eq!(world.tally().routes.get(&Route::CreateRepoHook), Some(&1));

    let since = world.needs().len();
    world.run_plan(2, plan);
    world.finish();
    assert_ne!(world.store().repository_hooks(1).next().expect("remade hook").id, hook_id);
    assert_eq!(world.tally().routes.get(&Route::CreateRepoHook), Some(&2));
    assert_eq!(world.tally().routes.get(&Route::DeleteRepoHook), Some(&1));
    assert_eq!(world.tally().routes.get(&Route::EditRepoHook), None);
    assert_eq!(
        world.needs().get(since..),
        Some([Need::Read(Item::AdminToken), Need::Read(Item::WebhookSecret)].as_slice())
    );
    assert_eq!(
        world.heard().last(),
        Some(&Heard::Done { run: Token::new(2), outcome: Outcome::Nothing, at: Duration::ZERO })
    );
}

#[test]
fn the_host_adopts_by_path_with_one_hook_at_the_listener_after_two_runs() {
    use skein_forge_setup::host::{Kind, Plan, Repository};
    use skein_forge_setup::reconcile::Outcome;
    use skein_forge_setup_world::{Heard, HostWorld};
    use skein_lib::{Duration, Token};
    let mut settings = fixtures::host_settings();
    match &mut settings.forges.get_mut(0).expect("one forge").kind {
        Kind::Forgejo(given) => given.listener = Some(listener()),
        Kind::GitHub(_) => panic!("Forgejo fixture"),
    }
    let mut world = HostWorld::with_settings(7, &[], fixtures::repository_config(&[b"alice/specimen"]), settings);
    fixtures::seed_bot_tokens(world.store_mut());
    let plan = Plan::Adopt {
        forge: 1,
        repository: Repository::Path { owner: b"alice".as_slice().into(), name: b"specimen".as_slice().into() },
    };
    for run in [1, 2] {
        world.run_plan(run, plan.clone());
        world.finish();
        let hooks: Vec<_> = world.store().repository_hooks(1).collect();
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].url, url(1));
        assert_eq!(
            world.heard().last(),
            Some(&Heard::Done { run: Token::new(run), outcome: Outcome::Nothing, at: Duration::ZERO })
        );
    }
}
