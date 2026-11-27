//! The ctl act through the root and the world's real memory store (temper/docs/design/protocol/checkpoints.md, 5).
use jig_client::{self as client, contract as c};
use jig_domain_core::{documents::Document, reads};
use jig_domain_people as people;
use jig_protocol_engine_clients::translate::owner;
use skein_lib::{Env, List, Queue, ReplyTo, RequestEnvelope, Token, Wall};
use temper_client_protocol_wire as wire;
use temper_client_tools::{self as tools, root};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, store::Store};
use temper_engine_service_vocabulary::family;

fn driver(owner: bool) -> Driver {
    let mut driver = Driver::new(Store::new());
    driver.settle();
    driver.sign_in();
    driver.settle();
    if !owner {
        let owner_session = driver.session();
        driver.delivered.clear();
        driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
            until: None,
            held: None,
            reply_to: ReplyTo::new(Token::new(600)),
            identity: people::Identity {
                key: temper_engine_domain_forge::user_key(1, 8),
                login: b"member".as_slice().into(),
                name: b"Member".as_slice().into(),
            },
        }));
        driver.settle();
        let person = driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| {
                if let engine::Delivery::WebReply { reply: people::Reply::SignedIn { person, .. }, .. } = delivery {
                    Some(*person)
                } else {
                    None
                }
            })
            .expect("second party signed in");
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(601)),
            sign_in: owner_session,
            key: [6; 16],
            ask: people::Ask::SetRoles {
                project: 1,
                holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
            },
        }));
        driver.settle();
        for (project, held) in [(people::DEPLOYMENT, false), (1, true)] {
            let row = driver
                .store
                .rows
                .get(&engine::Key::Core(jig_domain_core::Key::People(people::Key::Roles(project))))
                .expect("durable roster");
            let engine::Record::Core(jig_domain_core::Record::People(people::Stored::Roles { holdings, .. })) = row
            else {
                panic!("roles")
            };
            assert_eq!(holdings.contains(&people::Holding { person, role: people::Role::Owner }), held);
        }
    }
    driver
}
fn me(driver: &mut Driver, sign_in: u64) -> Box<c::Me> {
    driver.delivered.clear();
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(500)),
        sign_in,
        read: Box::new(reads::Read::Frame),
    }));
    driver.settle();
    let answer = driver
        .delivered
        .iter()
        .find_map(|delivery| if let engine::Delivery::Read { answer, .. } = delivery { Some(answer) } else { None })
        .expect("checkpoint world fixture");
    let reads::Answer::Document(document) = answer else { panic!("me document") };
    let Document::Frame(frame) = document.as_ref() else { panic!("me frame") };
    let preset = tools::limits::client().client;
    let limits =
        jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("checkpoint world fixture");
    let wire = owner::frame(frame, Wall::from_nanos(u64::MAX), &[], limits.wire()).expect("me on wire");
    Box::new(jig_client_protocol_link::translate::documents::frame(wire, &limits).expect("client me"))
}
fn act(driver: &mut Driver) -> tools::jig::Printed {
    let sign_in = driver.session();
    let me = me(driver, sign_in);
    let env = Env {
        now: driver.env.now,
        wall: driver.env.wall,
        limits: root::Limits::new(tools::limits::client(), tools::limits::tools()),
    };
    let mut tool = root::Root::unlinked(&env.limits, 7).expect("checkpoint world fixture");
    let mut out = Queue::uncapped();
    for event in [
        client::Event::StartUnlinked { kept: List::with_capacity(0) },
        client::Event::SignIn { with: client::With::Transport },
        client::Event::Up { up: client::Up::Served { served: true } },
        client::Event::Up {
            up: client::Up::Watched {
                watch: Token::new(0),
                event: c::WatchEvent::Delivered(c::Delivery::Me(me.clone())),
            },
        },
    ] {
        root::step(&mut tool, &env, root::Event::Client { event }, &mut out);
    }
    let tools::jig::call::Parsed::Call { call, json } =
        tools::table::parse(&[b"deployment", b"checkpoint"]).expect("checkpoint world fixture")
    else {
        panic!("checkpoint call")
    };
    root::step(&mut tool, &env, root::Event::Call { number: 1, call, json, value: None }, &mut out);
    for _ in 0..64 {
        let request = out.pop().expect("ctl progresses");
        let event = match request {
            root::Request::Printed { printed, .. } => return printed,
            root::Request::Unlinked {
                request: client::UnlinkedRequest::Down { down: client::Down::Open { watch, .. } },
            } => client::Event::Up {
                up: client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(me.clone())) },
            },
            root::Request::Unlinked {
                request:
                    client::UnlinkedRequest::Down {
                        down: client::Down::Send { attempt, key, request: client::Ask::Family { record, .. }, .. },
                    },
            } => {
                let event = family::to_root(
                    &jig_protocol_config::Directories::working(),
                    jig_protocol_engine_clients::ToApplication::Request {
                        reply_to: ReplyTo::new(Token::new(501)),
                        sign_in,
                        key,
                        family: record.family,
                        version: record.version,
                        scope: people::ApplicationScope::Deployment,
                        record: record.bytes,
                        digest: [9; 32],
                        carried: None,
                    },
                    &temper_engine_service_vocabulary::forges::Forges::with_capacity(0),
                    None,
                    &family::Deployment { welcome: false, owned: false, program_login: None, push: None },
                    None,
                    &wire::v1::CEILINGS,
                )
                .expect("checkpoint world fixture");
                driver.delivered.clear();
                driver.send(event);
                driver.settle();
                let outcome = driver
                    .delivered
                    .iter()
                    .find_map(|delivery| {
                        if let engine::Delivery::Application { outcome, .. } = delivery {
                            Some(outcome.clone())
                        } else {
                            None
                        }
                    })
                    .expect("root's terminal");
                let bytes = family::answer(outcome, &wire::v1::CEILINGS, &jig_client_protocol_wire::CEILINGS)
                    .expect("checkpoint world fixture");
                client::Event::Up {
                    up: client::Up::Answer {
                        attempt,
                        envelope: RequestEnvelope::Final,
                        answer: Some(client::Answered::Family(client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes,
                        })),
                        why: None,
                    },
                }
            }
            _ => continue,
        };
        root::step(&mut tool, &env, root::Event::Client { event }, &mut out);
    }
    panic!("checkpoint did not answer")
}
#[test]
fn the_owners_act_prints_the_memory_stores_unavailable_refusal() {
    let printed = act(&mut driver(true));
    assert_eq!(printed.status, 2);
    assert_eq!(printed.stderr.as_ref(), b"temper ctl: checkpoint refused: the store is not serving\n");
}
#[test]
fn a_project_only_owners_act_prints_not_owner_instead_of_asking_the_store() {
    let mut driver = driver(false);
    let printed = act(&mut driver);
    assert_eq!(printed.status, 2);
    assert_eq!(
        printed.stderr.as_ref(),
        b"temper ctl: checkpoint refused: only an owner of the deployment takes a checkpoint\n"
    );
}
