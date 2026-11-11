//! Both sides of a copy meet through HTTP (skein/docs/design/forge/setup/forgejo.md, 1.10).
use core::num::{NonZeroU32, NonZeroU64};
use skein_fake_forge_domain::api::Fill;
use skein_fake_forge_forgejo::{Config, Repository, Route};
use skein_forge::forgejo::request::Grant;
use skein_forge_setup::{
    host::{File, Need, Plan},
    item::Item,
    reconcile::{Failure, Outcome, Stage, Why},
};
use skein_forge_setup_world::{
    Heard, HostWorld,
    fixtures::{self, copy_stories as fixture},
};
use skein_lib::{Duration, Time, Token};

const FORGE: u16 = 1;
const ORG: &[u8] = b"first";
const NAME: &[u8] = b"starter";
const FULL: &[u8] = b"first/starter";
// The catalogue source is not an HTTP repository; the first destination gets id one.
const REPOSITORY: u32 = 1;
const DEADLINE: Duration = Duration::from_secs(1800);
// Source finishes one second after the run's own deadline.
const SLOW: Duration = Duration::from_secs(1801);

fn world(config: Config) -> HostWorld {
    let taken = config.repositories.iter().map(|row| (row.id, row.full.clone())).collect::<Vec<_>>();
    let mut world = HostWorld::with_settings(7, &[], config, fixture::copy_deadline(DEADLINE));
    fixtures::seed_bot_tokens(world.store_mut());
    world.put(Item::ApiToken, fixtures::secret(fixtures::API));
    world.put(Item::GitToken, fixtures::secret(fixtures::GIT));
    world.store_mut().org(ORG, b"kit-admin");
    let (mut domain, config) = fixture::repository_domain();
    for (id, full) in taken {
        skein_fake_forge_domain::create_repository(&mut domain, &config, id, &full, NAME, 1).expect("taken repository");
    }
    world.with_domain(domain, config);
    world
}
fn plan() -> Plan {
    Plan::Copy {
        forge: FORGE,
        org: ORG.into(),
        repository: NAME.into(),
        from: fixture::ADDRESS.into(),
        files: Box::new([File { path: fixture::PATH.into(), content: fixture::CONTENT.into() }]),
    }
}
fn copy(world: &mut HostWorld, run: u64) {
    world.run_plan(run, plan());
    world.finish();
}
fn copied(run: u64, id: u64, at: Duration) -> Heard {
    Heard::Done {
        run: Token::new(run),
        outcome: Outcome::Copied { id: NonZeroU64::new(id).expect("repository"), branch: fixture::BRANCH.into() },
        at,
    }
}
fn stopped(run: u64, why: Why, at: Duration) -> Heard {
    Heard::Stop {
        run: Token::new(run),
        failure: Failure { forge: FORGE, step: Stage::Repository, subject: FULL.into(), why },
        at,
    }
}
fn routes(world: &HostWorld, route: Route) -> u64 {
    world.tally().routes.get(&route).copied().unwrap_or(0)
}
fn marked(world: &HostWorld) -> Option<&[u8]> {
    world.copying(FORGE, ORG, NAME).map(skein_lib::Secret::expose)
}
fn writes(world: &HostWorld) -> Vec<(Route, u64)> {
    // Every HTTP mutation a copy can issue, including credential and access repair.
    [
        Route::CreateOrg,
        Route::CreateRepository,
        Route::Migrate,
        Route::DeleteRepository,
        Route::CreateFile,
        Route::AddCollaborator,
        Route::CreateRepoHook,
        Route::EditRepoHook,
        Route::DeleteRepoHook,
        Route::CreateUser,
        Route::CreateToken,
        Route::DeleteToken,
        Route::CreateTeam,
        Route::EditTeam,
        Route::AddMember,
    ]
    .into_iter()
    .map(|route| (route, routes(world, route)))
    .collect()
}

#[test]
fn a_copy_lands_private_with_the_source_the_bots_access_and_the_starter() {
    let mut world = world(fixture::catalogue(Fill::Whole));
    copy(&mut world, 1);
    assert_eq!(world.heard(), &[copied(1, u64::from(REPOSITORY), Duration::ZERO)]);
    assert_eq!(marked(&world), Some([1].as_slice()));
    assert!(world.file(Item::Copying).is_none(), "copy marks are not credentials");
    assert!(world.copying(FORGE, ORG, b"another-copy").is_none());
    assert!(world.store().is_private(u64::from(REPOSITORY)));
    assert_eq!(world.store().collaborator(REPOSITORY, b"kit"), Some(Grant::Write));
    assert_eq!(world.domain().branch(fixture::SOURCE, fixture::BRANCH), Some(fixture::SOURCE_COMMIT));
    assert_eq!(world.domain().branch(FULL, fixture::BRANCH), Some(fixture::STARTER_COMMIT));
    let object = world.domain().object(fixture::STARTER_COMMIT).expect("starter commit");
    assert_eq!(object.parent, Some(fixture::SOURCE_COMMIT));
    assert_eq!(object.tree.get(b"README.md".as_slice()).expect("source readme").as_ref(), fixture::README);
    assert_eq!(object.tree.get(fixture::PATH).expect("starter file").as_ref(), fixture::CONTENT);
    let author = object.author.as_ref().expect("bot author");
    assert_eq!(author.name.as_ref(), b"kit");
    assert_eq!(author.email.as_ref(), b"kit@example.invalid");
    assert_eq!(object.author, object.committer);
}

#[test]
fn a_copy_run_again_writes_nothing() {
    let mut world = world(fixture::catalogue(Fill::Whole));
    copy(&mut world, 1);
    let before = writes(&world);
    let needs = world.needs().len();
    copy(&mut world, 2);
    assert_eq!(world.heard().last(), Some(&copied(2, u64::from(REPOSITORY), Duration::ZERO)));
    assert_eq!(writes(&world), before);
    assert!(!world.needs()[needs..].iter().any(|need| matches!(need, Need::Write { .. })));
    assert_eq!(world.domain().branch(FULL, fixture::BRANCH), Some(fixture::STARTER_COMMIT));
}

#[test]
fn a_copy_cut_short_run_again_finishes() {
    let mut world = world(fixture::catalogue(Fill::Cut));
    copy(&mut world, 1);
    assert_eq!(world.heard(), &[stopped(1, Why::Refused(500), Duration::ZERO)]);
    assert_eq!(marked(&world), Some([1].as_slice()));
    assert_eq!(world.domain().branch(FULL, fixture::BRANCH), None);
    let mut config = fixture::catalogue(Fill::Whole);
    config.repositories = skein_lib::List::with_capacity(1);
    config
        .repositories
        .push(Repository { id: REPOSITORY, full: FULL.into(), protected: None })
        .expect("cut repository");
    world.restart_forge(config);
    copy(&mut world, 2);
    // The fake keeps its high-water id when the old destination is deleted.
    assert_eq!(world.heard().last(), Some(&copied(2, 2, Duration::ZERO)));
    assert_eq!(routes(&world, Route::DeleteRepository), 1);
    assert_eq!(routes(&world, Route::Migrate), 1);
    assert_eq!(marked(&world), Some([1].as_slice()));
    assert_eq!(world.domain().branch(FULL, fixture::BRANCH), Some(fixture::STARTER_COMMIT));
}

#[test]
fn a_name_taken_stops_and_writes_nothing() {
    let mut config = fixture::catalogue(Fill::Whole);
    config.repositories = skein_lib::List::with_capacity(1);
    config.repositories.push(Repository { id: REPOSITORY, full: FULL.into(), protected: None }).expect("taken name");
    let mut world = world(config);
    let before = writes(&world);
    copy(&mut world, 1);
    assert_eq!(world.heard(), &[stopped(1, Why::Taken, Duration::ZERO)]);
    assert_eq!(marked(&world), None);
    assert_eq!(writes(&world), before);
    assert!(!world.needs().iter().any(|need| matches!(need, Need::Write { .. })));
}

#[test]
fn a_refused_source_says_the_forges_words() {
    let mut world = world(fixtures::config());
    copy(&mut world, 1);
    assert_eq!(world.heard(), &[stopped(1, Why::Refused(422), Duration::ZERO)]);
    // copy-facts' setup-migrate-missing-source records these Forgejo words.
    let words = b"Authentication failed: Clone: exit status 128 - fatal: could not read Username for 'http://127.0.0.1:3001': terminal prompts disabled\n.";
    assert_eq!(world.said(), &[(Token::new(1), words.as_slice().into(), 0)]);
    assert_eq!(marked(&world), Some([1].as_slice()));
    assert_eq!(world.domain().metadata(FULL).expect("refused copy stays").default_branch.as_ref(), b"main");
    assert!(world.domain().branches(FULL).is_empty());
    assert!(world.store().is_private(u64::from(REPOSITORY)));
}

#[test]
fn a_refused_copy_run_again_deletes_the_empty_repository_and_finishes() {
    let mut world = world(fixtures::config());
    copy(&mut world, 1);
    assert_eq!(world.heard(), &[stopped(1, Why::Refused(422), Duration::ZERO)]);
    assert_eq!(marked(&world), Some([1].as_slice()));
    assert!(world.domain().branches(FULL).is_empty());
    assert_eq!(world.domain().metadata(FULL).expect("refused copy").default_branch.as_ref(), b"main");
    let mut config = fixture::catalogue(Fill::Whole);
    config.repositories = skein_lib::List::with_capacity(1);
    config.repositories.push(Repository { id: REPOSITORY, full: FULL.into(), protected: None }).expect("refused copy");
    world.restart_forge(config);
    copy(&mut world, 2);
    // The world's admin token has write:organization, required by the fake's deletion route.
    assert_eq!(world.heard().last(), Some(&copied(2, 2, Duration::ZERO)));
    assert_eq!(routes(&world, Route::DeleteRepository), 1);
    assert_eq!(routes(&world, Route::Migrate), 1);
    assert_eq!(marked(&world), Some([1].as_slice()));
    assert!(world.store().is_private(2));
    assert!(!world.store().is_private(u64::from(REPOSITORY)));
    assert_eq!(world.domain().metadata(FULL).expect("whole copy").default_branch.as_ref(), fixture::BRANCH);
    assert_eq!(world.domain().branches(FULL).len(), world.domain().branches(fixture::SOURCE).len());
    assert_eq!(world.domain().branch(FULL, fixture::BRANCH), Some(fixture::STARTER_COMMIT));
    let object = world.domain().object(fixture::STARTER_COMMIT).expect("starter commit");
    assert_eq!(object.parent, Some(fixture::SOURCE_COMMIT));
    assert_eq!(object.tree.get(b"README.md".as_slice()).expect("source readme").as_ref(), fixture::README);
    assert_eq!(object.tree.get(fixture::PATH).expect("starter file").as_ref(), fixture::CONTENT);
}

#[test]
fn a_slow_copy_past_its_deadline_stops_timed_out_and_the_next_run_finds_it() {
    let mut world = world(fixture::catalogue(Fill::Slow(SLOW)));
    world.run_plan(1, plan());
    world.settle();
    world.advance_to(Time::from_nanos(DEADLINE.as_nanos() - 1));
    assert!(world.heard().is_empty());
    world.advance_to(Time::ZERO.saturating_add(DEADLINE));
    assert_eq!(world.heard(), &[stopped(1, Why::TimedOut, DEADLINE)]);
    assert_eq!(world.aborts(), 1);
    assert_eq!(marked(&world), Some([1].as_slice()));
    world.advance_to(Time::ZERO.saturating_add(SLOW));
    let before = writes(&world);
    copy(&mut world, 2);
    assert_eq!(world.heard().last(), Some(&copied(2, u64::from(REPOSITORY), SLOW)));
    assert_eq!(routes(&world, Route::Migrate), 1);
    assert_eq!(routes(&world, Route::DeleteRepository), 0);
    // Only the missing starter is written when the completed copy is found.
    let mut expected = before;
    for (route, count) in &mut expected {
        if *route == Route::CreateFile || *route == Route::AddCollaborator {
            *count = 1;
        }
    }
    assert_eq!(writes(&world), expected);
}

#[test]
fn a_second_writing_run_beside_a_copy_is_busy() {
    let mut world = world(fixture::catalogue(Fill::Slow(SLOW)));
    world.run_plan(1, plan());
    world.settle();
    world.run(2);
    world.run_plan(3, Plan::Reachable { forge: FORGE, installation: None, page: NonZeroU32::new(1).expect("page") });
    world.settle();
    assert!(
        matches!(world.heard(), [Heard::Stop { run, failure, .. }, Heard::Done { run: reader, outcome: Outcome::Reachable { .. }, .. }]
        if *run == Token::new(2) && failure.why == Why::Busy && *reader == Token::new(3)),
        "{:?}",
        world.heard()
    );
    assert_eq!(routes(&world, Route::Migrate), 1);
    world.finish();
    assert_eq!(
        world.heard().iter().filter(|event| matches!(event, Heard::Stop { run, .. } if *run == Token::new(1))).count(),
        1
    );
}
