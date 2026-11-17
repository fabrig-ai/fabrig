//! A deployment starts without a repository (testing/qa/setups.md, 5).
use super::{forge, forge_document};
use serde_json::{Value, json};
use skein_fake_forge_domain as domain;
use skein_fake_forge_forgejo as wire;
use skein_fake_peers::forge::Forge;
use skein_lib::{
    Env, Intake, Queue, Time, Token, Wall,
    stream::{Down, Read, Up},
};

fn request(service: &mut wire::Service, forge: &mut Forge, method: &str, target: &str, body: &[u8]) -> Vec<u8> {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge::git_limits() };
    let fake_env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge.config.clone() };
    let mut server = wire::Server::new(Token::new(1), &env.limits).expect("HTTP server");
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut replies = Queue::uncapped();
    let mut input = Intake::with_capacity(65536);
    input.append(format!("{method} {target} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: token {}\r\nContent-Length: {}\r\n\r\n",
        "a".repeat(40), body.len()).as_bytes()).expect("request head");
    input.append(body).expect("request body");
    let mut output = Vec::new();
    wire::start(&mut server, service, &env, &mut above, &mut below);
    for _ in 0..256 {
        wire::resume_with_domain(
            &mut server,
            service,
            wire::Hosted { domain: &mut forge.domain, config: &forge.config },
            &env,
            &mut above,
            &mut below,
        );
        while let Some(down) = below.pop() {
            let up = match down {
                Down::Demand { read: Read::Nothing, room: 0 } | Down::Finish => None,
                Down::Demand { read, room } => {
                    input.meet(read).map(Up::Bytes).or_else(|| (room > 0).then_some(Up::Room))
                }
                Down::Send(bytes) => {
                    output.extend_from_slice(&bytes);
                    None
                }
            };
            if let Some(up) = up {
                wire::up_with_domain(
                    &mut server,
                    service,
                    wire::Hosted { domain: &mut forge.domain, config: &forge.config },
                    &env,
                    up,
                    &mut above,
                    &mut below,
                );
            }
        }
        while let Some(event) = above.pop() {
            match event {
                wire::Event::Domain(event) => domain::step(&mut forge.domain, &fake_env, event, &mut replies),
                wire::Event::Close | wire::Event::Closed => panic!("HTTP closed before response"),
            }
        }
        domain::fire(&mut forge.domain, &fake_env, &mut replies);
        while let Some(reply) = replies.pop() {
            if let Ok((_, reply)) = service.target(reply) {
                wire::down(&mut server, service, &env, reply, &mut above, &mut below);
            }
        }
        service.reclaim();
        forge.domain.reclaim();
        if !output.is_empty() && server.is_settled() && below.is_empty() {
            return output;
        }
    }
    panic!("HTTP did not settle")
}

#[test]
fn an_empty_forge_serves_the_repository_its_admin_makes() {
    let mut value = forge_document();
    value["repositories"] = json!([]);
    let document = forge::Document::decode(&serde_json::to_vec(&value).expect("JSON")).expect("zero repositories");
    let (mut forge, config) = document.seed(23);
    let mut service = wire::Service::new(config, &forge::git_limits(), 23).expect("empty forge service");
    forge::setup(service.setup_mut(), &[b'a'; 40]);
    service.setup_mut().org(b"work", b"temper-admin");
    let made = request(
        &mut service,
        &mut forge,
        "POST",
        "/api/v1/orgs/work/repos",
        br#"{"name":"hello","default_branch":"main","private":false,"auto_init":true}"#,
    );
    assert!(made.starts_with(b"HTTP/1.1 201"), "{}", String::from_utf8_lossy(&made));
    let read = request(&mut service, &mut forge, "GET", "/api/v1/repos/work/hello", b"");
    assert!(read.starts_with(b"HTTP/1.1 200"), "{}", String::from_utf8_lossy(&read));
    let split = read.windows(4).position(|bytes| bytes == b"\r\n\r\n").expect("HTTP head");
    let repository: Value = serde_json::from_slice(&read[split + 4..]).expect("repository JSON");
    assert_eq!(repository["id"], 1);
    assert_eq!(repository["owner"]["login"], "work");
    assert_eq!(repository["name"], "hello");
    assert!(forge.domain.branch(b"work/hello", b"main").is_some());
}
