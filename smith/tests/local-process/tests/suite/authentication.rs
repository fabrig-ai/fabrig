use skein_io::kernel;
use smith_local_process_world::{
    Authentication, Files, Placement, World,
    referee::{Ending, review},
};

#[test]
fn a_sign_in_uses_the_page_and_redirect_before_lending_the_saved_token() {
    for placement in [Placement::Spawned, Placement::InProcess] {
        let files = Files::new();
        let mut world = World::authenticated(20, &files, placement, Authentication::SignIn);
        world.settle();
        assert_eq!(world.exit(), Some(kernel::Exit::Code(0)));
        assert_eq!(world.oauth_posts(), 1);
        assert_eq!(world.page_visits(), 1);
        assert!(world.browser_replied());
        assert!(world.saved_before_query());
        let saved = smith_local_process_world::credential_fixture::Tokens::new(
            &world.scenario.launch.token_directory,
            smith_local_shell::credentials::token_limits(),
        )
        .expect("private synthetic token store")
        .load(0)
        .expect("synthetic token load")
        .expect("saved synthetic grant");
        assert_eq!(saved.access_token.expose(), b"access-new");
        assert_eq!(saved.refresh_token.as_ref().map(skein_lib::Secret::expose), Some(b"refresh-new".as_slice()));
        review(&world.seen(), &world, Ending::Report(b"First answer".to_vec())).assert_passed(20);
    }
}

#[test]
fn an_expired_grant_refreshes_through_the_shared_private_token_store() {
    let seed = 21;
    for placement in [Placement::Spawned, Placement::InProcess] {
        let files = Files::new();
        let mut world = World::authenticated(seed, &files, placement, Authentication::Refresh);
        world.settle();
        assert_eq!(world.exit(), Some(kernel::Exit::Code(0)));
        assert_eq!(world.oauth_posts(), 1);
        assert_eq!(world.page_visits(), 0);
        assert!(world.saved_before_query());
        review(&world.seen(), &world, Ending::Report(b"First answer".to_vec())).assert_passed(seed);
    }
}

#[test]
fn a_refused_refresh_never_lends_a_token_or_starts_the_agent() {
    for placement in [Placement::Spawned, Placement::InProcess] {
        let files = Files::new();
        let mut world = World::authenticated(22, &files, placement, Authentication::Refused);
        world.settle();
        assert_eq!(world.exit(), Some(kernel::Exit::Code(0)));
        assert_eq!(world.oauth_posts(), 1);
        assert_eq!(world.page_visits(), 0);
        assert!(world.queries().is_empty());
        assert!(world.facts().is_empty());
        assert!(!world.saved_before_query());
        review(&world.seen(), &world, Ending::Unavailable).assert_passed(22);
    }
}

#[test]
fn a_lent_grant_refreshes_while_the_model_is_running_before_it_lapses() {
    let files = Files::new();
    let mut world = World::authenticated(23, &files, Placement::InProcess, Authentication::Proactive);
    world.settle();
    assert_eq!(world.exit(), Some(kernel::Exit::Code(0)));
    assert_eq!(world.oauth_posts(), 1);
    assert_eq!(world.page_visits(), 0);
    let post = world.first_token_post().expect("scheduled refresh reached the peer");
    assert!(post.as_nanos() >= skein_tls_kit::pki::VALID.as_nanos() + skein_lib::Duration::from_secs(1).as_nanos());
    assert!(post.as_nanos() < skein_tls_kit::pki::VALID.as_nanos() + skein_lib::Duration::from_secs(61).as_nanos());
    let token = smith_local_process_world::credential_fixture::Tokens::new(
        &files.path().join("tokens"),
        smith_local_shell::credentials::token_limits(),
    )
    .expect("private store")
    .load(0)
    .expect("record")
    .expect("saved rotation");
    assert_eq!(token.generation, 2);
    assert_eq!(token.refresh_token.as_ref().map(skein_lib::Secret::expose), Some(b"refresh-new".as_slice()));
    review(&world.seen(), &world, Ending::Report(b"First answer".to_vec())).assert_passed(23);
}

#[test]
fn a_current_sign_in_record_lends_its_grant_without_a_refresh() {
    let files = Files::new();
    let mut world = World::authenticated(42, &files, Placement::InProcess, Authentication::Current);
    let tokens = smith_local_process_world::credential_fixture::Tokens::new(
        &files.path().join("tokens"),
        smith_local_shell::credentials::token_limits(),
    )
    .expect("private store");
    let before = std::fs::read(files.path().join("tokens/0.json")).expect("synthetic record");
    world.settle();
    assert_eq!(world.oauth_posts(), 0);
    assert_eq!(world.page_visits(), 0);
    assert!(!world.queries().is_empty(), "the current grant reaches the provider");
    assert_eq!(tokens.load(0).expect("record").expect("kept record").generation, 1);
    assert!(
        std::fs::read(files.path().join("tokens/0.json")).expect("settled record") == before,
        "lending leaves the current record unchanged"
    );
    review(&world.seen(), &world, Ending::Report(b"First answer".to_vec())).assert_passed(42);
}

#[test]
fn smith_on_a_terminal_signs_in_and_commits_in_place() {
    use smith_local_process_world::referee::CheckoutRead;

    let files = Files::new();
    let mut world = World::signed_in_change(20, &files);
    let before = world.head().expect("initial head");
    world.settle();

    let issuer = smith_agent_process_world::fake::address(444);
    let sign_in = format!("Sign in: http://{issuer}/authorize");
    assert!(world.shown().split(|byte| *byte == b'\n').any(|line| line.starts_with(sign_in.as_bytes())));
    assert_eq!(world.page_visits(), 1);
    assert!(world.browser_replied());
    assert!(world.saved_before_query());
    assert_eq!(saved_grant(&world).generation, 1);
    world.assert_ended(Ending::Change { before });
    let tree = world.commit_files();
    assert_eq!(tree[b"original.txt".as_slice()], b"edited\n");
    assert_eq!(tree[b"result.txt".as_slice()], b"new\n");
    assert_eq!(tree[b"command.txt".as_slice()], b"ran\n");
    assert_eq!(tree[b"checks-ran.txt".as_slice()], b"checked\n");
}

// The raised lead covers the held record's one-hour lifetime, while the
// earliest refresh time can independently forbid rotation during this run.
const HELD_LIFETIME: skein_lib::Duration = skein_lib::Duration::from_secs(3600);
const RAISED_LEAD: skein_lib::Duration = skein_lib::Duration::from_secs(36_000);
const HELD_BEARER: &[u8] = b"access-held";
const ROTATED_BEARER: &[u8] = b"access-new";

fn saved_grant(world: &World) -> skein_oauth::SavedToken {
    smith_local_process_world::credential_fixture::Tokens::new(
        &world.scenario.launch.token_directory,
        smith_local_shell::credentials::token_limits(),
    )
    .expect("private fixture store")
    .load(0)
    .expect("fixture record read")
    .expect("saved synthetic grant")
}

fn exec_on_raised_lead(
    files: &Files,
    earliest_refresh_at: Option<skein_lib::Wall>,
    expected: (&'static [u8], u64),
) -> World {
    let expires_at = skein_lib::Wall::from_nanos(skein_tls_kit::pki::VALID.as_nanos() + HELD_LIFETIME.as_nanos());
    let mut world =
        World::with_refresh_window(43, files, RAISED_LEAD, expires_at, earliest_refresh_at, HELD_BEARER, expected);
    assert_eq!(saved_grant(&world).generation, 1, "the held record begins at generation one");
    world.settle();
    world
}

fn assert_accepted_exec(world: &World) {
    world.assert_ended(Ending::Report(b"First answer".to_vec()));
    assert_eq!(
        world.facts().iter().filter(|fact| matches!(fact, smith_local_domain::Fact::Answered { .. })).count(),
        1,
        "exactly one accepted activation"
    );
    assert!(world.saved_before_query());
    for output in [world.shown(), world.errors()] {
        for value in [HELD_BEARER, ROTATED_BEARER, b"refresh-old", b"refresh-new"] {
            assert!(
                !smith_local_process_world::fixture::contains(output, value),
                "no credential value on either stream"
            );
        }
    }
}

#[test]
fn an_exec_on_a_raised_lead_refreshes_its_record_at_once() {
    let files = Files::new();
    let world = exec_on_raised_lead(&files, None, (ROTATED_BEARER, 2));
    assert_accepted_exec(&world);
    assert_eq!(world.oauth_posts(), 1, "one token request to the issuer");
    assert_eq!(world.oauth_statuses(), [200]);
    assert_eq!(saved_grant(&world).generation, 2);
}

#[test]
fn an_exec_before_the_earliest_refresh_time_keeps_its_record() {
    let files = Files::new();
    let earliest = skein_lib::Wall::from_nanos(skein_tls_kit::pki::VALID.as_nanos() + HELD_LIFETIME.as_nanos());
    let world = exec_on_raised_lead(&files, Some(earliest), (HELD_BEARER, 1));
    assert_accepted_exec(&world);
    assert_eq!(world.oauth_posts(), 0, "no token request before the earliest refresh time");
    assert_eq!(saved_grant(&world).generation, 1);
}
