//! The shipped tools render the family's retained records (temper/docs/design/client/tools.md, 5.5).
use super::*;
use jig_client::{self as client, contract as c};
use temper_client_tools::{self as tools, root as cli};

enum Record {
    Answer(w::Outcome),
    Forges(Box<[u8]>),
}

fn text(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(bytes.into(), 64).unwrap())
}

fn me() -> c::Delivery {
    c::Delivery::Me(Box::new(c::Me {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 256,
        party: c::MeParty { number: 1, kind: c::PartyKind::Person, login: text(b"owner"), name: text(b"Owner") },
        projects: List::with_capacity(0),
        needs: c::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: c::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: LIFE,
        deployment_role: Some(c::HeldRole::Owner),
    }))
}

fn render(words: &[&[u8]], json: bool, record: &Record) -> tools::jig::Printed {
    let env = Env {
        now: Time::ZERO,
        wall: Wall::EPOCH,
        limits: cli::Limits::new(tools::limits::client(), tools::limits::tools()),
    };
    let mut root = cli::Root::unlinked(&env.limits, 7).unwrap();
    let mut out = Queue::uncapped();
    for event in [
        client::Event::StartUnlinked { kept: List::with_capacity(0) },
        client::Event::SignIn { with: client::With::Transport },
        client::Event::Up { up: client::Up::Served { served: true, versions: None } },
    ] {
        cli::step(&mut root, &env, cli::Event::Client { event }, &mut out);
    }
    let tools::jig::call::Parsed::Call { call, .. } = tools::table::parse(words).unwrap() else { panic!("tool call") };
    cli::step(&mut root, &env, cli::Event::Call { number: 1, call, json, value: None }, &mut out);
    for _ in 0..PASSES {
        let Some(request) = out.pop() else { break };
        let event = match request {
            cli::Request::Printed { printed, .. } => return printed,
            cli::Request::Client {
                request:
                    client::Request::Down {
                        down: client::Down::Open { watch, subject: client::Watchable::Jig(c::Subject::Me) },
                    },
            } => Some(client::Event::Up { up: client::Up::Watched { watch, event: c::WatchEvent::Delivered(me()) } }),
            cli::Request::Client { request: client::Request::Down { down: client::Down::Open { watch, .. } } } => {
                let Record::Forges(bytes) = record else { panic!("forge document") };
                Some(client::Event::Up {
                    up: client::Up::FamilyDelivered {
                        watch,
                        record: client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: bytes.clone(),
                        },
                    },
                })
            }
            cli::Request::Client { request: client::Request::Down { down: client::Down::Send { attempt, .. } } } => {
                let Record::Answer(outcome) = record else { panic!("family answer") };
                let bytes =
                    wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: outcome.clone() }).unwrap());
                Some(client::Event::Up {
                    up: client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        answer: Some(client::Answered::Family(client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes,
                        })),
                        why: None,
                    },
                })
            }
            cli::Request::Client { .. }
            | cli::Request::Ready
            | cli::Request::Record { .. }
            | cli::Request::Kept { .. }
            | cli::Request::Progress { .. }
            | cli::Request::Fact { .. } => None,
        };
        if let Some(event) = event {
            cli::step(&mut root, &env, cli::Event::Client { event }, &mut out);
        }
    }
    panic!("tool printed its record")
}

pub(super) fn status(service: &crate::Service, json: bool) -> tools::jig::Printed {
    let bytes = crate::family::forges(service.engine.as_ref().unwrap().domain.forges(), &w::CEILINGS).unwrap();
    render(&[b"forge", b"list"], json, &Record::Forges(bytes))
}

#[test]
fn retained_install_link_renders_in_human_and_json_replays() {
    let mut service = app_service();
    let answer = install(&mut service, 1);
    let w::Outcome::Link(link) = &answer else { panic!("install link") };
    let human = render(&[b"forge", b"install", b"1", b"--key", b"install"], false, &Record::Answer(answer.clone()));
    let replay = install(&mut service, 1);
    let json = render(&[b"forge", b"install", b"1", b"--key", b"install"], true, &Record::Answer(replay.clone()));
    assert_eq!(replay, answer);
    assert_eq!((human.status, json.status), (0, 0));
    assert!(contains(&human.stdout, b"Open this address in a browser on the engine's machine:"));
    assert!(contains(&human.stdout, link.address()));
    assert!(contains(&json.stdout, link.address()));
}
