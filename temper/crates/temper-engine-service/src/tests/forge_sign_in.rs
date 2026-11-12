//! Forge sign-in and its page follow kept records (temper/docs/design/protocol/openid-provider.md, 7, 8 and 9).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary protocol fixtures construct textual requests and inspect page bytes"
)]
use super::{
    fixtures::{self, http::Peer},
    sign_in,
};
use crate::clients::forge_sign_in::{Entries, Unserved, refresh};
use jig_protocol_config::clients::ProviderClient;
use skein_lib::{Duration, Map, Time, Token, Wall};
use temper_engine_domain_forge as forge;

const PAGE: &[u8] = b"<head></head><body>temper</body>";
const FIRST_META: &[u8] = b"<meta name=\"temper-forge-sign-in\" content=\"1 forgejo https://git.example.com other\">";
const SECOND_META: &[u8] = b"<meta name=\"temper-forge-sign-in\" content=\"2 two https://git.example.com other\">";
const FILE_BYTES: &[u8] = b"boot();";
const LIMIT: u32 = 65_536;
const CALLBACK: &[u8] = b"https://git.example.com/user/oauth2/temper/callback";
const REQUEST: &[u8] = b"GET / HTTP/1.1\r\nHost: temper.example.com\r\nConnection: close\r\n\r\n";
// The turn bound of fixtures::http::Peer, on the same seeded wire.
const PAGE_TURNS: u32 = 256;

fn world() -> crate::Service {
    let (mut service, _) = sign_in::world_settings(4, Duration::from_secs(600), true, None, &[]);
    let engine = service.engine.as_mut().unwrap();
    engine.workers.forges.remove(1);
    engine.notes.clear();
    let mut bundle = fixtures::bundle::bundle();
    bundle.page = PAGE.into();
    bundle.files = Map::with_capacity(1);
    bundle
        .files
        .insert(
            b"boot.js".as_slice().into(),
            crate::clients::bundle::File { kind: b"text/javascript", bytes: FILE_BYTES.into(), gzip: None },
        )
        .unwrap();
    service.adopt_bundle(bundle);
    service
}
fn entries(
    service: &mut crate::Service,
    rows: &[(u16, &[u8])],
    public: Option<&temper_forge_setup::setup::Origin>,
    limit: u32,
) {
    let rows: Vec<_> = rows
        .iter()
        .map(|(forge, client)| crate::startup::application::sign_in::ForgeSignIn {
            forge: *forge,
            client: (*client).into(),
        })
        .collect();
    let clients: Vec<_> = rows
        .iter()
        .map(|row| ProviderClient {
            id: row.client.clone(),
            secret: Some(b"forgejo.secret".as_slice().into()),
            redirect_uri: CALLBACK.into(),
        })
        .collect();
    service.engine.as_mut().unwrap().clients.forge_sign_in = Entries::new(&rows, &clients, public, None, limit);
}
fn configured() -> crate::Service {
    let mut service = world();
    entries(&mut service, &[(1, b"forgejo")], None, LIMIT);
    refresh(service.engine.as_mut().unwrap());
    service
}
fn record(number: u16) -> forge::ForgeRecord {
    let mut record = fixtures::forge_record(number);
    record.mode = forge::Mode::Managed;
    record.place.git = b"http://127.0.0.1:3000".as_slice().into();
    record.place.web = Some(b"https://git.example.com".as_slice().into());
    record
}
fn get(service: &mut crate::Service, tag: Option<&[u8]>) -> (Peer, u16, Box<[skein_http::Header]>) {
    let request = match tag {
        Some(tag) => format!(
            "GET / HTTP/1.1\r\nHost: temper.example.com\r\nIf-None-Match: {}\r\nConnection: close\r\n\r\n",
            core::str::from_utf8(tag).unwrap()
        )
        .into_bytes(),
        None => REQUEST.to_vec(),
    };
    let mut peer = fixtures::request(service, &request);
    let (status, headers) = fixtures::answered(service, &mut peer);
    (peer, status, headers)
}
fn page(meta: &[u8]) -> Box<[u8]> {
    [b"<head>".as_slice(), meta, b"</head><body>temper</body>"].concat().into()
}
fn notes(service: &mut crate::Service) -> Vec<(u16, Box<[u8]>, Unserved)> {
    let mut notes = Vec::new();
    while let Some(note) = service.notes().pop() {
        if let crate::Note::ForgeSignIn { number, client, why } = note {
            notes.push((number, client, why));
        }
    }
    notes
}

#[test]
fn a_kept_forgejo_at_its_redirect_gets_its_meta_and_the_page_a_new_etag() {
    let mut service = configured();
    let (before, status, headers) = get(&mut service, None);
    assert_eq!(status, 200);
    assert_eq!(fixtures::bundle::body(&before), PAGE);
    let tag = fixtures::header(&headers, b"etag").unwrap();
    assert_eq!(notes(&mut service), [(1, b"forgejo".as_slice().into(), Unserved::NotKept)]);
    fixtures::keep_forge(&mut service, record(1));
    let (after, status, headers) = get(&mut service, Some(tag));
    assert_eq!(status, 200, "the old tag no longer revalidates");
    assert_ne!(fixtures::header(&headers, b"etag"), Some(tag));
    assert_eq!(fixtures::bundle::body(&after), page(FIRST_META).as_ref());
    assert!(notes(&mut service).is_empty());
}

#[test]
fn an_owned_forge_takes_the_machines_public_and_says_owned() {
    let mut service = world();
    let public = temper_forge_setup::setup::Origin { host: b"git.example.com".as_slice().into(), port: 443 };
    entries(&mut service, &[(1, b"forgejo")], Some(&public), LIMIT);
    let mut owned = record(1);
    owned.mode = forge::Mode::Owned;
    owned.place.web = Some(b"https://internal.example.com".as_slice().into());
    fixtures::keep_forge(&mut service, owned);
    let (reply, status, _) = get(&mut service, None);
    assert_eq!(status, 200);
    assert_eq!(
        fixtures::bundle::body(&reply),
        page(b"<meta name=\"temper-forge-sign-in\" content=\"1 forgejo https://git.example.com owned\">").as_ref()
    );
    assert!(notes(&mut service).is_empty());
}

#[test]
fn an_entry_that_does_not_serve_is_named_once_and_the_others_serve() {
    let mut github = record(1);
    github.kind = forge::Kind::GitHub;
    let mut elsewhere = record(1);
    elsewhere.place.web = None;
    // The second meta fits exactly; the first client is one byte longer, putting its meta one past the bound.
    let room = u32::try_from(PAGE.len().checked_add(FILE_BYTES.len()).unwrap().checked_add(SECOND_META.len()).unwrap())
        .unwrap();
    for (name, first, limit, why) in [
        ("absent", None, LIMIT, Unserved::NotKept),
        ("github", Some(github), LIMIT, Unserved::NotForgejo),
        ("origin", Some(elsewhere), LIMIT, Unserved::Origin { origin: b"http://127.0.0.1:3000".as_slice().into() }),
        ("room", Some(record(1)), room, Unserved::NoRoom),
    ] {
        let mut service = world();
        if let Some(first) = first {
            fixtures::keep_forge(&mut service, first);
        }
        fixtures::keep_forge(&mut service, record(2));
        entries(&mut service, &[(1, b"four"), (2, b"two")], None, limit);
        refresh(service.engine.as_mut().unwrap());
        refresh(service.engine.as_mut().unwrap());
        let (reply, status, _) = get(&mut service, None);
        assert_eq!(status, 200, "{name}");
        assert_eq!(fixtures::bundle::body(&reply), page(SECOND_META).as_ref(), "{name}");
        assert_eq!(notes(&mut service), [(1, b"four".as_slice().into(), why)], "{name}");
    }
}

#[test]
fn a_removed_forge_loses_its_meta() {
    let mut service = configured();
    assert_eq!(notes(&mut service), [(1, b"forgejo".as_slice().into(), Unserved::NotKept)]);
    fixtures::keep_forge(&mut service, record(1));
    let (before, _, headers) = get(&mut service, None);
    assert_eq!(fixtures::bundle::body(&before), page(FIRST_META).as_ref());
    let tag = fixtures::header(&headers, b"etag").unwrap();
    crate::forge::gone::request(service.engine.as_mut().unwrap(), 1);
    let (after, status, _) = get(&mut service, Some(tag));
    assert_eq!(status, 200);
    assert_eq!(fixtures::bundle::body(&after), PAGE);
    refresh(service.engine.as_mut().unwrap());
    assert_eq!(
        notes(&mut service),
        [(1, b"forgejo".as_slice().into(), Unserved::NotKept)],
        "serving clears the named reason"
    );
}

#[test]
fn a_restart_makes_the_meta_from_the_restored_record() {
    let mut service = configured();
    let saved = record(1);
    let engine = service.engine.as_mut().unwrap();
    fixtures::commit(
        &mut engine.store,
        2,
        Box::new([temper_engine_domain::Write::Save(temper_engine_domain::Record::Forge {
            row: Box::new(forge::Stored::Forge(saved)),
        })]),
    );
    assert!(engine.store.host.above.pop().is_some());
    let previous = service.engine.take().unwrap();
    let mut restarted = fixtures::restarting_from(previous);
    entries(&mut restarted, &[(1, b"forgejo")], None, LIMIT);
    let mut bundle = fixtures::bundle::bundle();
    bundle.page = PAGE.into();
    bundle.files = Map::with_capacity(0);
    restarted.adopt_bundle(bundle);
    refresh(restarted.engine.as_mut().unwrap());
    assert!(notes(&mut restarted).is_empty(), "no absent verdict before the records load");
    fixtures::restart(&mut restarted);
    let (reply, status, _) = get(&mut restarted, None);
    assert_eq!(status, 200);
    assert_eq!(fixtures::bundle::body(&reply), page(FIRST_META).as_ref());
    assert!(notes(&mut restarted).is_empty(), "restored forge never named absent");
}

#[test]
fn a_full_note_queue_holds_the_reason_until_it_can_be_named() {
    let mut service = world();
    entries(&mut service, &[(1, b"forgejo")], None, LIMIT);
    let engine = service.engine.as_mut().unwrap();
    for _ in 0..engine.notes.room() {
        engine.notes.push(crate::Note::ForgeLacking { number: 7, names: Box::new([]) });
    }
    refresh(engine);
    assert!(notes(&mut service).is_empty(), "the existing notes hold all the room");
    crate::forge::records::advance(service.engine.as_mut().unwrap());
    assert_eq!(notes(&mut service), [(1, b"forgejo".as_slice().into(), Unserved::NotKept)]);
    crate::forge::records::advance(service.engine.as_mut().unwrap());
    assert!(notes(&mut service).is_empty());
}

/// A byte peer that can pause between page chunks, without running the root.
struct PageReply {
    wire: skein_world::stream::Wire,
    owner: Option<Token>,
    closed: bool,
}
impl PageReply {
    fn new(service: &mut crate::Service) -> Self {
        let mut wire = skein_world::stream::Wire::new(7);
        wire.write(REQUEST);
        let engine = service.engine.as_mut().unwrap();
        assert!(engine.clients.up(skein_io::Event::Accepted {
            owner: Token::new(crate::tokens::SERVER),
            socket: fixtures::http::SOCKET,
            peer: fixtures::http::address(),
        }));
        Self { wire, owner: None, closed: false }
    }
    fn turn(&mut self, service: &mut crate::Service) {
        let engine = service.engine.as_mut().unwrap();
        engine.clients.iterate(
            Time::from_nanos(1),
            Wall::from_nanos(1),
            &mut engine.events,
            &engine.workers.forges,
            None,
            &mut engine.accounts,
        );
        while let Some(request) = engine.clients.io.pop() {
            match request {
                skein_io::Request::Bind { socket, owner } => {
                    assert_eq!(socket, fixtures::http::SOCKET);
                    self.owner = Some(owner);
                }
                skein_io::Request::Stream { down, .. } => self.wire.take(down),
                skein_io::Request::Close { .. } | skein_io::Request::Abort { .. } => {
                    self.closed = true;
                    assert!(engine.clients.up(skein_io::Event::Closed { owner: self.owner.unwrap() }));
                }
                other @ (skein_io::Request::Listen { .. }
                | skein_io::Request::Connect { .. }
                | skein_io::Request::Reject { .. }
                | skein_io::Request::Output { .. }
                | skein_io::Request::Spawn { .. }
                | skein_io::Request::SpawnAlone { .. }
                | skein_io::Request::Signal { .. }
                | skein_io::Request::Usage { .. }
                | skein_io::Request::Window { .. }) => panic!("unexpected page io: {other:?}"),
            }
        }
        if !self.closed
            && let Some(owner) = self.owner
            && let Some(up) = self.wire.answer()
        {
            assert!(engine.clients.up(skein_io::Event::Stream { owner, up }));
        }
    }
    fn body(&self) -> Option<&[u8]> {
        let at = skein_lib::bytes::find(&self.wire.received, b"\r\n\r\n")?.checked_add(4).unwrap();
        self.wire.received.get(at..)
    }
}

#[test]
fn a_page_reply_on_its_way_ends_with_the_page_it_began() {
    let mut service = configured();
    notes(&mut service);
    // More than two sends, so a record can arrive between body chunks.
    let send = crate::limits::server(&fixtures::engine().jig.limits).http.send;
    let mut original = b"<head></head><body>".to_vec();
    original.extend(vec![b'x'; usize::try_from(send).unwrap().checked_mul(2).unwrap()]);
    original.extend_from_slice(b"</body>");
    let mut bundle = fixtures::bundle::bundle();
    bundle.page = original.clone().into();
    bundle.files = Map::with_capacity(0);
    service.adopt_bundle(bundle);
    let mut peer = PageReply::new(&mut service);
    for _ in 0..PAGE_TURNS {
        peer.turn(&mut service);
        if peer.body().is_some_and(|body| !body.is_empty()) {
            break;
        }
    }
    let partial = peer.body().expect("first body chunk");
    assert!(!partial.is_empty() && partial.len() < original.len(), "record arrives between chunks");
    fixtures::keep_forge(&mut service, record(1));
    // Fresh routing still sees the old page until its active replies end.
    assert!(
        service.engine.as_ref().unwrap().clients.bundle.as_ref().unwrap().page.as_ref() == original,
        "the active page is retained"
    );
    for _ in 0..PAGE_TURNS {
        peer.turn(&mut service);
        if peer.closed {
            break;
        }
    }
    let response = core::str::from_utf8(&peer.wire.received).unwrap();
    let length = response
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: ").or_else(|| line.strip_prefix("content-length: ")))
        .unwrap()
        .parse::<usize>()
        .unwrap();
    assert_eq!(peer.body().unwrap().len(), length);
    assert_eq!(peer.body().unwrap(), original);
    assert!(peer.closed);
    let (reply, status, _) = get(&mut service, None);
    assert_eq!(status, 200);
    assert!(fixtures::bundle::body(&reply).windows(FIRST_META.len()).any(|bytes| bytes == FIRST_META));
}

#[test]
fn machine_owned_sign_in_is_refused_at_its_path() {
    for (owned_sign_in, sign_in, expected) in [
        ("unknown", "null", "application.machine.owned.sign_in: unknown reference\n"),
        (
            "forgejo",
            r#"{"google":{"provider":3,"client":"client","secret":"google.secret","endpoints":null},"members":[{"address":"ada@localhost","login":"ada"}]}"#,
            "application.machine.owned.sign_in: a box's sign-in takes application.sign_in null\n",
        ),
    ] {
        let input = super::stock::generated();
        let input = fixtures::with(
            &input,
            b"engine.providers",
            br#"[{"number":1,"kind":"link"},{"number":2,"kind":"deployment"},{"number":3,"kind":"application"}]"#,
        );
        let input = fixtures::with(&input, b"engine.owners", br#"[{"provider":3,"subject":"ada@localhost"}]"#);
        let input = fixtures::with(&input, b"engine.clients.provider", br#"{"key":"provider.key","addresses":[1],"clients":[{"id":"forgejo","secret":null,"redirect_uri":"https://forge.example/user/oauth2/temper/callback"}]}"#);
        let owned = format!(
            r#"{{"listen":{{"socket":"127.0.0.1:8368"}},"remote":"http://127.0.0.1:8368","person":{{"login":"ada","email":"ada@localhost"}},"forgejo":"fetched","runner":null,"sign_in":"{owned_sign_in}"}}"#
        );
        let input = fixtures::with(&input, b"application.machine.owned", owned.as_bytes());
        let input = fixtures::with(&input, b"application.sign_in", sign_in.as_bytes());
        let problems = crate::startup::named(&fixtures::engine_part(&input), &jig_protocol_config::Directories::working()).unwrap_err();
        assert_eq!(core::str::from_utf8(&crate::startup::lines(&problems)).unwrap(), expected);
    }
}

#[test]
fn sign_in_members_login_refuses_the_bot_without_any_forge_entries() {
    for login in ["temper", "TEMPER"] {
        let input = super::stock::generated();
        let input = fixtures::with(
            &input,
            b"engine.providers",
            br#"[{"number":1,"kind":"link"},{"number":2,"kind":"deployment"},{"number":3,"kind":"application"}]"#,
        );
        let input = fixtures::with(&input, b"engine.owners", br#"[{"provider":3,"subject":"ada@localhost"}]"#);
        let sign_in = format!(
            r#"{{"google":{{"provider":3,"client":"client","secret":"google.secret","endpoints":null}},"members":[{{"address":"ada@localhost","login":"{login}"}}]}}"#
        );
        let input = fixtures::with(&input, b"application.sign_in", sign_in.as_bytes());
        let problems = crate::startup::named(&fixtures::engine_part(&input), &jig_protocol_config::Directories::working()).unwrap_err();
        assert!(
            core::str::from_utf8(&crate::startup::lines(&problems))
                .unwrap()
                .contains("application.sign_in.members[0].login:")
        );
    }
}

#[test]
fn a_box_document_starts_and_its_printed_link_supplies_the_persons_verified_claims() {
    use super::provider;
    use jig_client_protocol_wire as wire;
    use skein_lib::{Secret, Writer};
    let mut configuration = provider::configuration();
    configuration.provider.addresses = Box::new([1]);
    configuration.provider.clients[0].secret = None;
    let input = super::stock::generated();
    let input = fixtures::with(&input, b"engine.clients.provider", br#"{"key":"provider.key","addresses":[1],"clients":[{"id":"forgejo","secret":null,"redirect_uri":"https://forge.example/user/oauth2/temper/callback"}]}"#);
    let input = fixtures::with(&input, b"application.machine.owned", br#"{"listen":{"socket":"127.0.0.1:8368"},"remote":"http://127.0.0.1:8368","person":{"login":"ada","email":"ada@localhost"},"forgejo":"fetched","runner":null,"sign_in":"forgejo"}"#);
    let named = crate::startup::named(&fixtures::engine_part(&input), &jig_protocol_config::Directories::working()).unwrap();
    assert!(named.sign_in.is_none());
    let mut config = fixtures::engine();
    config.jig = super::stock::engine(&input);
    config.jig.clients.provider = Some(configuration.provider);
    config.clients.public = Some(crate::startup::application::Origin {
        scheme: crate::startup::application::Scheme::Https,
        host: b"temper.example.com".as_slice().into(),
        port: None,
        name: b"temper.example.com".as_slice().into(),
    });
    let mut seeds = fixtures::seeds();
    seeds.link_secret = Some(Secret::new(Box::new([7; 32])));
    let mut clients = crate::clients::Clients::new(
        &jig_protocol_config::Directories::working(),
        &config.jig,
        &config.clients,
        &seeds,
        &config.limits.root.journal,
        None,
        named.machine.owned.as_ref(),
        &configuration.files,
    )
    .unwrap();
    clients.starting = None;
    let mut service = fixtures::ready();
    service.engine.as_mut().unwrap().clients = clients;
    fixtures::listening(&mut service);
    let limits = service.engine.as_ref().unwrap().clients.env.limits.wire;
    let record = wire::LinkSignIn::new(&limits, wire::LinkSignInParts { secret: seeds.link_secret.unwrap() }).unwrap();
    let mut out = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut out).unwrap();
    let signed = provider::request(
        &mut service,
        "POST",
        "/jig/v1/sign-in/link",
        "Content-Type: application/octet-stream\r\nJig-Client: browser\r\n",
        &out.finish(),
    );
    let cookie = sign_in::header(&signed, "set-cookie").split(|byte| *byte == b';').next().unwrap().to_vec();
    let code = provider::authorize(&mut service, &cookie);
    let form = format!(
        "grant_type=authorization_code&code={}&client_id=forgejo&redirect_uri=https%3A%2F%2Fforge.example%2Fuser%2Foauth2%2Ftemper%2Fcallback&code_verifier={}",
        core::str::from_utf8(&code).unwrap(),
        provider::VERIFIER
    );
    let token = provider::request(
        &mut service,
        "POST",
        "/oauth/token",
        "Content-Type: application/x-www-form-urlencoded\r\n",
        form.as_bytes(),
    );
    assert!(token.received().starts_with(b"HTTP/1.1 200"));
    let access = provider::text(&provider::json(&token), b"access_token");
    let user = provider::request(
        &mut service,
        "GET",
        "/oauth/userinfo",
        &format!("Authorization: Bearer {}\r\n", core::str::from_utf8(&access).unwrap()),
        &[],
    );
    let claims = provider::json(&user);
    assert_eq!(provider::text(&claims, b"preferred_username").as_ref(), b"ada");
    assert_eq!(provider::text(&claims, b"name").as_ref(), b"Owner");
    assert_eq!(provider::text(&claims, b"email").as_ref(), b"ada@localhost");
    assert_eq!(provider::text(&claims, b"sub").as_ref(), b"1:ada@localhost");
    let mut object = jig_protocol_config::read::Object::new(&claims, 0).unwrap();
    assert!(jig_protocol_config::read::boolean(&claims, object.member(&claims, b"email_verified").unwrap()).unwrap());
}

fn box_entries(service: &mut crate::Service, key: bool) {
    let mut settings = fixtures::engine();
    let input = super::stock::generated();
    let named = crate::startup::named(&fixtures::engine_part(&input), &jig_protocol_config::Directories::working()).unwrap();
    settings.machine = named.machine;
    if key {
        settings.machine.owned.as_mut().unwrap().sign_in = Some(b"forgejo".as_slice().into());
        settings.jig.clients.provider = Some(jig_protocol_config::clients::Provider {
            key: b"provider.key".as_slice().into(),
            addresses: Box::new([1]),
            clients: Box::new([ProviderClient {
                id: b"forgejo".as_slice().into(),
                secret: None,
                redirect_uri: CALLBACK.into(),
            }]),
        });
    }
    settings.machine.owned.as_mut().unwrap().public =
        Some(temper_forge_setup::setup::Origin { host: b"git.example.com".as_slice().into(), port: 443 });
    let engine = service.engine.as_mut().unwrap();
    engine.clients.deployment.owned = true;
    engine.clients.configure_forge_sign_in(&settings);
}
#[test]
fn a_box_owned_forge_has_an_off_fact_until_its_source_is_added_whatever_its_number() {
    let mut service = world();
    box_entries(&mut service, true);
    let mut owned = record(9);
    owned.mode = forge::Mode::Owned;
    fixtures::keep_forge(&mut service, owned);
    let (reply, status, _) = get(&mut service, None);
    assert_eq!(status, 200);
    assert_eq!(
        fixtures::bundle::body(&reply),
        page(b"<meta name=\"temper-forge-sign-in-off\" content=\"9\">").as_ref()
    );
    assert_eq!(notes(&mut service), [(9, b"forgejo".as_slice().into(), Unserved::NoSource)]);
    let (_, _, headers) = get(&mut service, None);
    let tag = fixtures::header(&headers, b"etag").unwrap().to_vec();
    let engine = service.engine.as_mut().unwrap();
    let mut out = skein_lib::Queue::uncapped();
    engine.box_link = crate::route::box_link::Link::new(256, true);
    engine.box_link.ask(9, Token::new(991), &mut out);
    out.clear();
    engine.box_link.read(b"source 9 failed discovery unavailable\n", &mut out);
    assert_eq!(out.pop(), Some(crate::route::box_link::Out::Source { forge: 9, added: false }));
    refresh(engine);
    assert!(notes(&mut service).is_empty(), "the same unserved reason is named once");
    let engine = service.engine.as_mut().unwrap();
    engine.box_link.read(b"source 8 added\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let (reply, _, _) = get(&mut service, None);
    assert_eq!(
        fixtures::bundle::body(&reply),
        page(b"<meta name=\"temper-forge-sign-in-off\" content=\"9\">").as_ref(),
        "another forge cannot enable this one"
    );
    let engine = service.engine.as_mut().unwrap();
    engine.box_link.read(b"source 9 added\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let (reply, status, headers) = get(&mut service, Some(&tag));
    assert_eq!(status, 200);
    assert_ne!(fixtures::header(&headers, b"etag"), Some(tag.as_slice()));
    assert_eq!(
        fixtures::bundle::body(&reply),
        page(b"<meta name=\"temper-forge-sign-in\" content=\"9 forgejo https://git.example.com owned\">").as_ref()
    );
    assert!(notes(&mut service).is_empty());
}
#[test]
fn a_source_added_after_started_turns_the_box_forges_links_back_on() {
    let mut service = world();
    box_entries(&mut service, true);
    let mut owned = record(9);
    owned.mode = forge::Mode::Owned;
    fixtures::keep_forge(&mut service, owned);
    let engine = service.engine.as_mut().unwrap();
    engine.box_link = crate::route::box_link::Link::new(256, true);
    let mut out = skein_lib::Queue::uncapped();
    engine.box_link.ask(9, Token::new(991), &mut out);
    out.clear();
    engine.box_link.read(b"source 9 failed temper's discovery did not answer\nstarted 9\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let (reply, _, headers) = get(&mut service, None);
    assert_eq!(
        fixtures::bundle::body(&reply),
        page(b"<meta name=\"temper-forge-sign-in-off\" content=\"9\">").as_ref()
    );
    let tag = fixtures::header(&headers, b"etag").unwrap().to_vec();
    let engine = service.engine.as_mut().unwrap();
    engine.box_link.ask(8, Token::new(992), &mut out);
    out.clear();
    engine.box_link.read(b"source 9 added\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let (reply, status, headers) = get(&mut service, Some(&tag));
    assert_eq!(status, 200, "later added invalidates the off page even during another group's ask");
    service.engine.as_mut().unwrap().box_link.read(b"started 8\n", &mut out);
    assert_eq!(
        out.pop(),
        Some(crate::route::box_link::Out::Started { forge: 8 }),
        "source did not consume the other ask"
    );
    assert_ne!(fixtures::header(&headers, b"etag"), Some(tag.as_slice()));
    assert_eq!(
        fixtures::bundle::body(&reply),
        page(b"<meta name=\"temper-forge-sign-in\" content=\"9 forgejo https://git.example.com owned\">").as_ref()
    );
}

#[test]
fn a_restored_box_source_fact_serves_without_an_active_setup_ask() {
    let mut service = world();
    box_entries(&mut service, true);
    let mut owned = record(9);
    owned.mode = forge::Mode::Owned;
    fixtures::keep_forge(&mut service, owned);
    fixtures::keep_forge(&mut service, record(8));
    notes(&mut service);
    let engine = service.engine.as_mut().unwrap();
    engine.box_link = crate::route::box_link::Link::new(256, true);
    let mut out = skein_lib::Queue::uncapped();
    engine.box_link.read(b"source 9 added\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let (reply, _, _) = get(&mut service, None);
    let ready = page(b"<meta name=\"temper-forge-sign-in\" content=\"9 forgejo https://git.example.com owned\">");
    assert_eq!(fixtures::bundle::body(&reply), ready.as_ref(), "a restored owned source restores its page fact");
    for foreign in [b"source 7 added\n".as_slice(), b"source 8 added\n"] {
        let engine = service.engine.as_mut().unwrap();
        engine.box_link.read(foreign, &mut out);
        crate::route::box_link::drain(engine, &mut out);
        let (reply, _, _) = get(&mut service, None);
        assert_eq!(
            fixtures::bundle::body(&reply),
            ready.as_ref(),
            "unknown and managed forges cannot replace the owned source"
        );
        notes(&mut service);
    }
    let engine = service.engine.as_mut().unwrap();
    engine.box_link.read(b"source 9 failed unavailable\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let (reply, _, _) = get(&mut service, None);
    assert_eq!(
        fixtures::bundle::body(&reply),
        page(b"<meta name=\"temper-forge-sign-in-off\" content=\"9\">").as_ref()
    );
}

#[test]
fn a_box_without_a_provider_key_has_only_its_owned_forges_off_fact() {
    let mut service = world();
    box_entries(&mut service, false);
    let mut owned = record(7);
    owned.mode = forge::Mode::Owned;
    fixtures::keep_forge(&mut service, owned);
    fixtures::keep_forge(&mut service, record(8));
    let (reply, _, _) = get(&mut service, None);
    assert_eq!(
        fixtures::bundle::body(&reply),
        page(b"<meta name=\"temper-forge-sign-in-off\" content=\"7\">").as_ref()
    );
}

#[test]
fn the_refused_person_identity_is_the_forge_domains_bot() {
    use temper_engine_domain_forge::{BOT_EMAIL, BOT_LOGIN};
    use temper_forge_setup::sign_in::refused_person;
    assert!(refused_person(BOT_LOGIN, b"person@example.invalid"));
    assert!(refused_person(b"person", BOT_EMAIL));
}
