use super::*;
use crate::tests::fixtures;
use skein_fake_forge_protocol as fake;
use skein_lib::List;
use temper_forge_setup::{place::Place, setup::Mode, spec};
use temper_protocol_records as records;
use tests::{Face, start, terminal};

// The existing setup stream face also speaks to the scope-enforcing peer.
impl Face {
    fn forge_reply(&mut self, service: &mut crate::Service, forge: &mut fake::Service, env: &Env<fake::Limits>) {
        self.sent(service);
        let mut server = fake::Server::new(Token::new(1), &env.limits).unwrap();
        let mut wire = skein_world::stream::Wire::new(17);
        wire.write(&self.wire.received);
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        fake::start(&mut server, forge, env, &mut above, &mut below);
        let mut closed = false;
        for _ in 0_u32..4096 {
            fake::resume(&mut server, forge, env, &mut above, &mut below);
            for _ in 0..below.len() {
                wire.take(below.pop().unwrap());
            }
            for _ in 0..above.len() {
                match above.pop().unwrap() {
                    fake::Event::Domain(_) => panic!("setup requests are served without domain calls"),
                    fake::Event::Close => fake::closed(&mut server, forge, env, &mut above, &mut below),
                    fake::Event::Closed => closed = true,
                }
            }
            if let Some(up) = wire.answer() {
                fake::up(&mut server, forge, env, up, &mut above, &mut below);
            }
        }
        assert!(closed, "fake completed the request and closed its connection");
        assert!(!wire.received.is_empty(), "fake answered");
        self.wire.write(&wire.received);
        for _ in 0_u32..1024 {
            self.turn(service);
        }
    }
}

fn start_adoption(
    service: &mut crate::Service,
    repository: core::num::NonZeroU64,
    token: Secret,
    access: skein_forge_connection::Access,
) {
    // Adoption reads the same records as every other setup plan.
    let engine = service.engine.as_mut().unwrap();
    for (item, value) in [
        (Item::AdminToken, token.expose()),
        (
            Item::ApiToken,
            match &access.credential {
                skein_forge::credential::Credential::Token(token) => token.expose(),
                skein_forge::credential::Credential::App { .. } => unreachable!(),
            },
        ),
    ] {
        engine.forge_records.rows.push(
            records::number(1, item),
            Secret::new(records::encode(item, value).unwrap().into_boxed_slice()),
            4096,
        );
    }
    // The sequential fake-start helpers use repositories 1 and 2; this scope fixture targets 7.
    service
        .engine
        .as_mut()
        .expect("engine")
        .setup
        .lifecycle
        .adoptions
        .insert(1, lifecycle::Adoption { next: repository, repositories: repository.get() })
        .unwrap();
    start(service, 1, host::Plan::Adopt { forge: 1, repository });
}

#[test]
#[expect(clippy::too_many_lines, reason = "one scope-enforcing peer walk checks adoption through its terminal answer")]
fn preparations_four_scopes_grant_adopt_and_without_repository_it_is_refused_at_repository() {
    for without_repository in [false, true] {
        let mut config = fixtures::engine();
        let entry = temper_forge_setup::setup::Entry::body_text(
            fixtures::owned_body(),
            1,
            &config.limits.setup.plan(),
            &config.directories,
        )
        .unwrap();
        let Mode::Owned(owned) = &entry.mode else { panic!("owned fixture") };
        let place = Place { directory: entry.directory.clone(), user: b"fixture".as_slice().into() };
        let spec = spec::admin_token(owned, &place, b"temper-admin-0123456789abcdef");
        // Preparation issues four scopes; the refusal case removes only repository access.
        let mut scopes = List::with_capacity(4);
        for args in spec.arguments.windows(2) {
            if args[0].as_ref() != b"--scopes" {
                continue;
            }
            let mut from = 0;
            for to in 0..=args[1].len() {
                if to < args[1].len() && args[1][to] != b',' {
                    continue;
                }
                let scope = &args[1][from..to];
                if !without_repository || scope != b"write:repository" {
                    scopes.push(scope).unwrap();
                }
                from = to.saturating_add(1);
            }
        }
        fixtures::fill_files(&mut config);
        let access = skein_forge_connection::Access {
            credential: skein_forge::credential::Credential::Token(Secret::new(b"api-token".as_slice().into())),
            user: 42,
            webhook_secret: None,
        };
        let mut service = crate::Service::engine(config, &fixtures::seeds(), None).unwrap();
        let mut record = fixtures::forge_record(1);
        record.setup = Some(fixtures::owned_body().into());
        fixtures::register_forges(&mut service, &[record]);
        service.engine.as_mut().expect("engine").store = fixtures::records_store();
        for _ in 0_u32..8 {
            crate::forge::records::advance(service.engine.as_mut().expect("engine"));
        }
        advance(service.engine.as_mut().expect("engine"));
        let limits = service.engine.as_ref().expect("engine").setup.env.limits;
        let env = Env {
            now: Time::ZERO,
            wall: Wall::EPOCH,
            limits: fake::Limits {
                calls: 4,
                connections: 4,
                raw_bytes: 4096,
                documents: host::documents(&limits),
                http: skein_http::server::Limits {
                    head: 4096,
                    headers: 16,
                    body: 65536,
                    read: 65536,
                    response: 4096,
                    send: 65536,
                },
            },
        };
        let mut repositories = List::with_capacity(1);
        repositories.push(fake::Repository { id: 7, full: b"acme/widget".as_slice().into(), protected: None }).unwrap();
        let mut forge = fake::Service::new(
            fake::Config {
                hooks: None,
                base: b"http://127.0.0.1:3000".as_slice().into(),
                tokens: List::with_capacity(0), // Only the issued scoped token authenticates.
                users: List::with_capacity(0),
                repositories,
                max_response_items: 2,
                default_paging_num: 30,
                faults: fake::Faults { reset: 0, drop: 0, server_error: 0 },
            },
            &env.limits,
            19,
        )
        .unwrap();
        let admin = forge.setup_mut().account(b"temper-admin", true, b"");
        forge.setup_mut().account(b"temper", false, b"");
        let token = Secret::new(b"0123456789abcdef0123456789abcdef01234567".as_slice().into());
        forge.setup_mut().token(admin, b"prepared", scopes.as_slice(), token.clone());
        let repository = core::num::NonZeroU64::new(7).unwrap();
        start_adoption(&mut service, repository, token, access);
        let mut face = Face::new();
        for _ in 0_u32..if without_repository { 2 } else { 4 } {
            face.forge_reply(&mut service, &mut forge, &env);
            if service.engine.as_ref().expect("engine").setup.runs.is_empty() {
                break;
            }
        }
        if without_repository {
            let expected = crate::Note::ForgeSetupStopped {
                number: 1,
                why: crate::SetupStopped::Stopped {
                    plan: crate::SetupPlan::Adopt,
                    step: crate::SetupStep { stage: reconcile::Stage::Repository, why: reconcile::Why::Refused(403) },
                },
            };
            let mut stopped = false;
            for note in &service.engine.as_ref().unwrap().notes {
                stopped |= *note == expected;
            }
            assert!(stopped);
            assert_eq!(forge.setup().collaborator(7, b"temper"), None);
        } else {
            let end = terminal(&mut service);
            let root::SetupEnd::Done { forge: 1, outcome: root::SetupOutcome::Done } = end else {
                panic!("adoption: {end:?}");
            };
            assert_eq!(forge.setup().collaborator(7, b"temper"), Some(skein_forge::forgejo::request::Grant::Write));
        }
    }
}
