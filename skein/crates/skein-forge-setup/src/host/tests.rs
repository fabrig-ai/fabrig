use super::*;
use crate::tests::fixtures;
use skein_lib::Wall;

#[test]
fn abort_upgrades_a_graceful_close_already_sent_to_io() {
    use crate::host::connection::Connection;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: crate::tests::host::limits() };
    let forge = Forge {
        kind: Kind::Forgejo(fixtures::given()),
        address: skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()),
        transport: Transport::Plaintext { name: None },
    };
    let mut c = Connection::new(
        Token::new(1),
        &forge,
        b"kit",
        &skein_forge::forgejo::request::Request::User.into(),
        &fixtures::secret(fixtures::ADMIN),
        &env.limits,
    )
    .expect("connection");
    c.socket = Some(Token::new(9));
    c.fail();
    let mut io = Queue::with_capacity(4);
    assert!(c.pump(&env, &mut io));
    assert_eq!(io.pop(), Some(skein_io::Request::Close { entity: Token::new(9) }));
    c.abort();
    assert!(c.pump(&env, &mut io));
    assert_eq!(io.pop(), Some(skein_io::Request::Abort { entity: Token::new(9) }));
    assert!(!c.pump(&env, &mut io));
}
#[test]
fn a_listing_past_the_configured_page_is_refused_at_open() {
    use skein_forge::forgejo::request::{Page, Request};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: crate::tests::host::limits() };
    let forge = Forge {
        kind: Kind::Forgejo(fixtures::given()),
        address: skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()),
        transport: Transport::Plaintext { name: None },
    };
    let send = |limit| Send {
        call: Request::Tokens { login: b"kit".as_slice().into(), page: Page { page: Some(1), limit: Some(limit) } }
            .into(),
        auth: crate::call::Auth::Token(fixtures::secret(fixtures::ADMIN)),
    };
    let inner = env.limits.inner;
    open(Token::new(1), &forge, b"kit", &send(inner), &env).expect("a page at inner fits");
    assert_eq!(open(Token::new(1), &forge, b"kit", &send(inner + 1), &env).err(), Some(Refused::Unexpected));
}

#[test]
fn valid_accepts_test_limits() {
    let l = crate::tests::host::limits();
    let settings = crate::tests::host::settings(&[1]);
    assert_eq!(limits::check(&l, &settings), Ok(()));
}

fn refuses_limits(l: Limits, settings: Settings) {
    let error = limits::check(&l, &settings).expect_err("unrepresentable limits");
    assert_eq!(Host::new(&l, settings, crate::tests::host::crypto()).err(), Some(error));
}

#[test]
fn valid_refuses_a_json_wire_length_past_u32() {
    let l = Limits { body_bytes_in: u32::MAX, ..crate::tests::host::limits() };
    refuses_limits(l, crate::tests::host::settings(&[1]));
}

#[test]
fn valid_refuses_an_unrepresentable_http_head() {
    let l = Limits { head_bytes: 1, ..crate::tests::host::limits() };
    refuses_limits(l, crate::tests::host::settings(&[1]));
}

#[test]
fn valid_refuses_unrepresentable_tls_intake() {
    let mut l = crate::tests::host::limits();
    l.tls.read = u32::MAX;
    refuses_limits(l, crate::tests::host::settings(&[1]));
}

#[test]
fn valid_refuses_a_key_spelling_past_u32() {
    let mut settings = crate::tests::host::github_settings();
    for index in 0..settings.forges.len() {
        let forge = settings.forges.get_mut(index).unwrap();
        if let Kind::GitHub(given) = &mut forge.kind {
            given.limits.key_bytes = u32::MAX;
        }
    }
    refuses_limits(crate::tests::host::limits(), settings);
}

#[test]
fn valid_ignores_heap_run_plans() {
    let l = Limits { runs: u32::MAX, ..crate::tests::host::limits() };
    let mut settings = crate::tests::host::settings(&[1]);
    let Kind::Forgejo(given) = &mut settings.forges.get_mut(0).unwrap().kind else { unreachable!() };
    given.limits.files = u32::MAX;
    given.limits.file_bytes = u32::MAX;
    assert_eq!(limits::check(&l, &settings), Ok(()));
}

#[test]
fn valid_refuses_installation_slots_past_u32() {
    let mut settings = crate::tests::host::github_settings();
    for index in 0..settings.forges.len() {
        let forge = settings.forges.get_mut(index).unwrap();
        if let Kind::GitHub(given) = &mut forge.kind {
            given.limits.pages = u32::MAX;
        }
    }
    refuses_limits(crate::tests::host::limits(), settings);
}

#[test]
fn runtime_removal_forgets_only_its_forges_callback_states() {
    use super::states::Purpose;
    use skein_lib::Duration;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: crate::tests::host::limits() };
    let mut host = Host::new(&env.limits, crate::tests::host::github_settings(), crate::tests::host::crypto())
        .expect("valid settings");
    let removed = host
        .states
        .issue(4, Purpose::Convert, &[1; 16], env.now, Duration::from_secs(60), b"manifest")
        .expect("unique state");
    let kept = host
        .states
        .issue(1, Purpose::Convert, &[2; 16], env.now, Duration::from_secs(60), b"manifest")
        .expect("unique state");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    host.down(&env, Request::Remove { forge: 4 }, &mut up, &mut io);
    assert!(host.states.take(4, Purpose::Convert, &removed, env.now).is_err());
    assert_eq!(host.states.take(1, Purpose::Convert, &kept, env.now), Ok(()));
    assert!(up.is_empty());
    assert!(io.is_empty());
}

#[test]
fn runtime_removal_stops_an_added_github_forge_at_its_deadline_step() {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: crate::tests::host::limits() };
    let mut host = Host::new(&env.limits, crate::tests::host::settings(&[]), crate::tests::host::crypto())
        .expect("valid settings");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    let mut settings = crate::tests::host::github_settings();
    let forge = settings.forges.get_mut(1).expect("GitHub fixture").clone();
    assert_eq!(forge.number(), 4);
    host.down(&env, Request::Add(forge), &mut up, &mut io);
    host.down(&env, Request::Run { run: Token::new(1), plan: Plan::Install { forge: 4 } }, &mut up, &mut io);
    assert_eq!(up.pop(), Some(Event::Need { run: Token::new(1), need: Need::Read(crate::item::Item::AppSlug) }));
    host.down(&env, Request::Remove { forge: 4 }, &mut up, &mut io);
    assert_eq!(
        up.pop(),
        Some(Event::Stop {
            run: Token::new(1),
            failure: Failure { forge: 4, step: Stage::App, subject: b"slug".as_slice().into(), why: Why::Down },
        })
    );
    assert!(up.is_empty());
    assert!(io.is_empty());
    assert_eq!(host.next_deadline(), None);
}
