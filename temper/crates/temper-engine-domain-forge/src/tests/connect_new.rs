//! New connects make records before asking the setup host (draft 3 and 6.4).
#![expect(clippy::disallowed_methods, reason = "ordinary test assertions inspect captured requests")]
#![expect(clippy::match_like_matches_macro, reason = "explicit request patterns; matches is banned by the subset")]
use super::*;
use crate::record as r;
use core::num::NonZeroU64;
use jig_domain_core::Carried;
use jig_domain_people::Role as PartyRole;

const VALUE: Carried = Carried { number: 1_u64 << 63_u32, digest: [17; 32] };

fn started() -> Domain {
    started_with(&LIMITS)
}

fn started_with(limits: &Limits) -> Domain {
    let mut d = Domain::new(
        limits,
        4,
        skein_forge_client::Config { namespace: b"connect-new".as_slice().into(), writers: Box::new([]) },
        &[],
        false,
    )
    .unwrap();
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall });
    d
}

fn application(
    d: &mut Domain,
    number: u64,
    ask: Ask,
    role: Option<PartyRole>,
    carried: Option<Carried>,
) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(number),
            token: Token::new(number),
            retention: Duration::from_secs(60),
            ask,
            role,
            carried,
        },
    )
}

fn ask(forge: ConnectForge, how: How) -> Ask {
    Ask::Connect { forge, how }
}

fn manifest() -> How {
    How::GitHubManifest { back: Some(b"/welcome".as_slice().into()) }
}

fn app() -> How {
    How::GitHubApp { app: NonZeroU64::new(7).unwrap() }
}

fn admin() -> How {
    How::ForgejoAdmin { first: None }
}

fn forgejo(address: &[u8]) -> ConnectForge {
    ConnectForge::Forgejo { address: address.into() }
}

fn github_record() -> Forge {
    Forge {
        number: 1,
        kind: Kind::GitHub,
        api: r::Address::Host { name: b"api.github.com".as_slice().into(), port: 443 },
        transport: r::Transport::Tls { server_name: b"api.github.com".as_slice().into(), trust: None },
        git: b"https://github.com".as_slice().into(),
        git_program: None,
        web: Some(b"https://github.com".as_slice().into()),
        read_only: false,
        identity: r::Identity { number: 1, login: Box::new([]), name: Box::new([]), email: Box::new([]) },
        files: None,
        setup: None,
        first: None,
        playground: false,
        removing: false,
    }
}

fn made_before_connect(out: &[Request], record: &Forge) {
    assert_eq!(out[0], Request::Save { record: Stored::Forge(record.clone()), release: None, read_afresh: None });
    assert_eq!(out[1], Request::Save { record: Stored::Forges { next: 2 }, release: None, read_afresh: None });
    assert_eq!(out[2], Request::MarkedForges);
    assert_eq!(out[3], Request::ForgeKept { forge: Box::new(record.clone()) });
}

fn refused(d: &mut Domain, ask: Ask, role: Option<PartyRole>, carried: Option<Carried>, why: ForgeRefusal) {
    let rows = d.forge_records.len();
    let next = d.next_forge();
    let out = application(d, 20, ask, role, carried);
    assert_eq!(d.application_answer(Token::new(20)), Some(Answered::Connect(ConnectAnswer::Refused(why))));
    assert_eq!(d.forge_records.len(), rows);
    assert_eq!(d.next_forge(), next);
    assert_eq!(out.len(), if carried.is_some() { 2 } else { 1 });
    assert_eq!(out.last(), Some(&Request::ApplicationDone { request: Token::new(20), token: Token::new(20) }));
    if let Some(value) = carried {
        assert_eq!(out[0], Request::CarriedUnused { number: value.number });
    }
}

#[test]
fn a_github_manifest_connect_makes_github_and_asks_its_manifest() {
    let mut d = started();
    let out = application(&mut d, 1, ask(ConnectForge::GitHub, manifest()), Some(PartyRole::Owner), Some(VALUE));
    let record = github_record();
    assert_eq!(d.forge_record(1), Some(&record));
    assert_eq!(d.configured(1), Some(&Configured::from_record(&record)));
    made_before_connect(&out, &record);
    assert!(out.contains(&Request::Manifest {
        forge: 1,
        request: Token::new(1),
        back: Some(b"/welcome".as_slice().into())
    }));
    assert!(out.contains(&Request::CarriedUnused { number: VALUE.number }));
    assert_eq!(d.application_answer(Token::new(1)), None);
}

#[test]
fn a_github_app_connect_makes_github_and_connects_with_its_key() {
    let mut d = started();
    let out = application(&mut d, 1, ask(ConnectForge::GitHub, app()), Some(PartyRole::Owner), Some(VALUE));
    assert_eq!(d.forge_record(1), Some(&github_record()));
    made_before_connect(&out, &github_record());
    assert!(out.contains(&Request::Connect { forge: 1, request: Token::new(1), how: app(), carried: VALUE }));
    assert_eq!(d.connection(1), Connection::Connecting);
    assert!(!out.contains(&Request::CarriedUnused { number: VALUE.number }));
}

#[test]
fn a_forgejo_admin_connect_makes_a_managed_forgejo_from_its_origin() {
    for first in [
        None,
        Some(FirstAsk {
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            project: SetUpProject::Existing(1),
            label: b"docker".as_slice().into(),
        }),
    ] {
        let mut d = started();
        let how = How::ForgejoAdmin { first: first.clone() };
        let out = application(
            &mut d,
            1,
            ask(forgejo(b"https://forge.example:8443"), how.clone()),
            Some(PartyRole::Owner),
            Some(VALUE),
        );
        let expected = Forge {
            kind: Kind::Forgejo,
            api: r::Address::Host { name: b"forge.example".as_slice().into(), port: 8443 },
            transport: r::Transport::Tls { server_name: b"forge.example".as_slice().into(), trust: None },
            git: b"https://forge.example:8443/".as_slice().into(),
            web: None,
            identity: r::Identity {
                number: 1,
                login: b"temper".as_slice().into(),
                name: b"Temper".as_slice().into(),
                email: b"temper@localhost".as_slice().into(),
            },
            setup: Some(r::Setup {
                directory: Box::new([]),
                bot: r::Account { login: b"temper".as_slice().into(), email: b"temper@localhost".as_slice().into() },
                orgs: Box::new([]),
                mode: r::SetupMode::Managed { admin_token: None },
                asked: false,
            }),
            ..github_record()
        };
        assert_eq!(d.forge_record(1), Some(&expected));
        assert_eq!(d.configured(1), Some(&Configured::from_record(&expected)));
        made_before_connect(&out, &expected);
        assert!(out.contains(&Request::Connect { forge: 1, request: Token::new(1), how, carried: VALUE }));
        assert_eq!(d.connection(1), Connection::Connecting);
        assert_eq!(
            d.forges().rows[0].first,
            first.map(|f| First {
                playground: false,
                org: f.org,
                repository: f.repository,
                project: f.project,
                label: f.label,
                state: FirstState::Making,
            })
        );
    }
}

#[test]
fn a_loopback_http_origin_is_plaintext() {
    let mut d = started();
    application(&mut d, 1, ask(forgejo(b"http://127.0.0.1:3000"), admin()), Some(PartyRole::Owner), Some(VALUE));
    let record = d.forge_record(1).unwrap();
    assert_eq!(record.api, r::Address::Socket(core::net::SocketAddr::from(([127, 0, 0, 1], 3000))));
    assert_eq!(record.transport, r::Transport::Plaintext);
    assert_eq!(record.git.as_ref(), b"http://127.0.0.1:3000/");
}

#[test]
fn an_http_origin_off_loopback_is_refused_and_makes_nothing() {
    for address in [b"http://192.0.2.1:3000".as_slice(), b"http://localhost:3000"] {
        refused(
            &mut started(),
            ask(forgejo(address), admin()),
            Some(PartyRole::Owner),
            Some(VALUE),
            ForgeRefusal::NotTaken,
        );
    }
}

#[test]
fn a_refused_role_makes_no_forge() {
    for role in [None, Some(PartyRole::Member), Some(PartyRole::Maintainer), Some(PartyRole::Observer)] {
        for ask in [
            ask(ConnectForge::GitHub, manifest()),
            ask(ConnectForge::GitHub, app()),
            ask(forgejo(b"https://forge.example"), admin()),
        ] {
            refused(&mut started(), ask, role, Some(VALUE), ForgeRefusal::Role);
        }
    }
}

#[test]
fn a_failed_connect_leaves_its_forge_kept() {
    for ask in [ask(ConnectForge::GitHub, app()), ask(forgejo(b"https://forge.example"), admin())] {
        let mut d = started();
        application(&mut d, 1, ask, Some(PartyRole::Owner), Some(VALUE));
        let record = d.forge_record(1).unwrap().clone();
        outputs(&mut d, Event::ConnectAnswered { forge: 1, request: Token::new(1), answer: ConnectAnswer::Connecting });
        let out = outputs(
            &mut d,
            Event::ConnectEnded {
                forge: 1,
                request: Token::new(1),
                connected: false,
                why: Some(b"connection refused".as_slice().into()),
                stop: None,
            },
        );
        assert_eq!(d.forge_record(1), Some(&record));
        assert_eq!(d.connection(1), Connection::NotConnected);
        assert_eq!(d.next_forge(), 2);
        assert!(!d.forges().rows[0].finished);
        assert!(out.contains(&Request::EraseCarried { number: VALUE.number }));
        assert!(!out.iter().any(|r| if let Request::Erase { .. } = r { true } else { false }));
    }
}

#[test]
fn missing_keys_mismatched_kinds_and_bad_origins_make_nothing() {
    for ask in [ask(ConnectForge::GitHub, app()), ask(forgejo(b"https://forge.example"), admin())] {
        refused(&mut started(), ask, Some(PartyRole::Owner), None, ForgeRefusal::NoKey);
    }
    for ask in [
        ask(ConnectForge::GitHub, admin()),
        ask(forgejo(b"https://forge.example"), app()),
        ask(forgejo(b"https://forge.example"), manifest()),
        ask(forgejo(b"https://forge.example/path"), admin()),
    ] {
        refused(&mut started(), ask, Some(PartyRole::Owner), Some(VALUE), ForgeRefusal::NotTaken);
    }
}

#[test]
fn a_first_organization_named_as_the_bot_makes_nothing() {
    let how = How::ForgejoAdmin {
        first: Some(FirstAsk {
            org: b"Temper".as_slice().into(),
            repository: b"hello".as_slice().into(),
            project: SetUpProject::Existing(1),
            label: b"docker".as_slice().into(),
        }),
    };
    refused(
        &mut started(),
        ask(forgejo(b"https://forge.example"), how),
        Some(PartyRole::Owner),
        Some(VALUE),
        ForgeRefusal::FirstIsBot,
    );
}

#[test]
fn full_and_busy_admission_make_nothing() {
    let mut full = started();
    let mut out = Queue::uncapped();
    for _ in 0..LIMITS.forges {
        crate::make::make(&mut full, &env(), github_record(), &mut out).unwrap();
    }
    refused(&mut full, ask(ConnectForge::GitHub, manifest()), Some(PartyRole::Owner), None, ForgeRefusal::Busy);
    let capacity_env = Env { limits: Limits { forges: 8, ..LIMITS }, ..env() };
    let mut busy = started_with(&capacity_env.limits);
    for number in 1..=u64::from(LIMITS.applications) {
        application(&mut busy, number, ask(ConnectForge::GitHub, manifest()), Some(PartyRole::Owner), None);
    }
    let mut requests = Queue::uncapped();
    step(
        &mut busy,
        &capacity_env,
        Event::Application {
            request: Token::new(20),
            token: Token::new(20),
            retention: Duration::from_secs(60),
            ask: ask(forgejo(b"https://forge.example"), admin()),
            role: Some(PartyRole::Owner),
            carried: Some(VALUE),
        },
        &mut requests,
    );
    assert_eq!(
        busy.application_answer(Token::new(20)),
        Some(Answered::Connect(ConnectAnswer::Refused(ForgeRefusal::Busy)))
    );
    assert_eq!(busy.forge_records.len(), LIMITS.forges);
    assert_eq!(busy.next_forge(), 5);
    assert_eq!(requests.pop(), Some(Request::CarriedUnused { number: VALUE.number }));
    assert_eq!(requests.pop(), Some(Request::ApplicationDone { request: Token::new(20), token: Token::new(20) }));
    assert!(requests.is_empty());
}

#[test]
fn a_service_refusal_leaves_the_record_kept_and_releases_the_key() {
    let mut d = started();
    application(&mut d, 1, ask(ConnectForge::GitHub, app()), Some(PartyRole::Owner), Some(VALUE));
    let out = outputs(
        &mut d,
        Event::ConnectAnswered {
            forge: 1,
            request: Token::new(1),
            answer: ConnectAnswer::Refused(ForgeRefusal::NotTaken),
        },
    );
    assert_eq!(d.forge_record(1), Some(&github_record()));
    assert!(!d.forges().rows[0].finished);
    assert!(out.contains(&Request::CarriedUnused { number: VALUE.number }));
}
