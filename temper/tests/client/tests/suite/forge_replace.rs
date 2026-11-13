//! Page and ctl replacement on the joined client/engine (temper/docs/design/client/ctl/testing.md, 2).
use jig_client_kit::harness::{ClientId, Until, World};
use skein_lib::{Duration, Token};
use skein_ui::{Act, Role};
use skein_world::domain::Verdict;
use temper_client_domain::Address;
use temper_client_world::{app::Temper, fixtures as f, front::Joint};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, walking};
type Pages = World<Temper, Joint>;
const FORGE: u16 = 1;
const WITHIN_REQUEST: Duration = Duration::from_millis(100);
fn browser(seed: u64) -> (temper_client_world::app::Root, temper_client_world::app::View) {
    let limits = f::domain_limits();
    let words = temper_client_view::words::Words::english(
        f::FORGE,
        limits.client.domain.text,
        temper_client_view::words::Shell::Browser,
    )
    .expect("browser words fit");
    let bounds = f::view_bounds(&limits, &words);
    (
        temper_client_world::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        temper_client_world::app::View::Person(Box::new(temper_client_view::View::new(
            &bounds,
            &limits,
            words,
            f::FORGE,
        ))),
    )
}
fn new_world(kind: forge::Kind) -> Pages {
    let mut world = super::forgejo_tokens::new_world();
    let front = world.front();
    let mut store = front.engine.take_store();
    store.seed(
        Box::new([
            engine::Record::Forge {
                row: Box::new(forge::Stored::Forge(forge::ForgeRecord {
                    person: None,
                    git_program: None,
                    number: FORGE,
                    kind,
                    mode: forge::Mode::Connected,
                    state: forge::ForgeState::Kept,
                    place: forge::Place {
                        api: forge::Address { host: b"forge.example".as_slice().into(), port: 443, tls: None },
                        git: b"https://forge.example/".as_slice().into(),
                        web: None,
                    },
                    read_only: false,
                    bot: forge::Bot { login: b"builder".as_slice().into(), name: Box::new([]), email: Box::new([]) },
                    asked: false,
                    first: None,
                })),
            },
            engine::Record::Forge { row: Box::new(forge::Stored::ForgeNumber { last: FORGE }) },
        ]),
        Box::new([]),
    );
    let limits = front.engine.env.limits;
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(f::SEED);
        config.core.connectors = Box::new([temper_client_protocol_wire::FORGE]);
        config.forges = Box::new([]);
        config.forge.writers[0].author = 7;
        config.under_box = false;
        config
    });
    front.engine = Driver::configured(store, configs, &limits);
    front.engine.settle();
    connected(&mut world, FORGE);
    world
}
fn send(world: &mut Pages, event: forge::Event) {
    world.front().engine.send(engine::Event::Forge(Box::new(event)));
}
fn connected(world: &mut Pages, number: u16) {
    send(
        world,
        forge::Event::Connection {
            forge: number,
            state: forge::Connection::Connected { id: 7, login: b"builder".as_slice().into() },
        },
    );
}
fn settle(world: &mut Pages) {
    let outcome = world.run(Until::Time(world.now().saturating_add(WITHIN_REQUEST)));
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn act(world: &mut Pages, client: ClientId, role: Role, name: &[u8], act: Act) {
    let node = world
        .tree(client)
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("named control {role:?} {name:?}"))
        .id;
    world.act(client, node, act);
    settle(world);
}
fn text(world: &Pages, client: ClientId) -> Vec<u8> {
    world
        .tree(client)
        .nodes()
        .iter()
        .flat_map(|node| {
            node.name
                .iter()
                .chain(node.description.iter())
                .map(AsRef::as_ref)
                .chain(node.value.iter().map(|value| value.text.as_ref()))
                .chain(node.text.iter().map(|run| run.text.as_ref()))
        })
        .flatten()
        .copied()
        .collect()
}
fn private(world: &mut Pages, client: ClientId, secret: &[u8]) {
    assert!(skein_lib::bytes::find(&text(world, client), secret).is_none());
    let front = world.front();
    for captured in [
        format!("{:?}", front.engine.store.rows),
        format!("{:?}", front.engine.transactions),
        format!("{:?}", front.engine.delivered),
        format!("{:?}", front.engine.immediate),
    ] {
        assert!(skein_lib::bytes::find(captured.as_bytes(), secret).is_none());
    }
}
fn secret(seed: u64) -> Box<[u8]> {
    let mut rng = skein_lib::Rng::new(seed);
    (0..32).map(|_| b'a' + u8::try_from(rng.next_u64() % 26).expect("alphabet fits")).collect()
}
fn finish(world: &mut Pages, number: u16, request: Token) {
    send(world, forge::Event::ConnectEnded { forge: number, request, connected: true, why: None, stop: None });
    connected(world, number);
    settle(world);
}
#[test]
fn the_page_replaces_a_git_token_and_an_app_key() {
    for (kind, field, button) in [
        (forge::Kind::Forgejo, b"Git token".as_slice(), b"Replace the tokens".as_slice()),
        (forge::Kind::GitHub, b"Private key", b"Replace the key"),
    ] {
        let mut world = new_world(kind);
        let client = world.client_with(f::PARTY, Address::Forges, f::domain_limits(), browser);
        let value = secret(f::SEED);
        settle(&mut world);
        act(&mut world, client, Role::Button, b"Replace", Act::Press);
        act(&mut world, client, Role::TextField, field, Act::Input { value: value.clone() });
        private(&mut world, client, &value);
        act(&mut world, client, Role::Button, button, Act::Press);
        let (number, request) = *world.front().replacements.last().expect("replacement sent");
        assert_eq!(number, FORGE);
        assert_eq!(world.front().replacements.len(), 1);
        assert!(skein_lib::bytes::find(&text(&world, client), b"Connecting").is_some());
        private(&mut world, client, &value);
        finish(&mut world, number, request);
        assert!(skein_lib::bytes::find(&text(&world, client), "builder · user 7".as_bytes()).is_some());
        private(&mut world, client, &value);
    }
}

fn tokens_file(items: &[(&[u8], &[u8])]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for (name, value) in items {
        bytes.extend_from_slice(name);
        bytes.push(b' ');
        bytes.extend_from_slice(value);
        bytes.push(b'\n');
    }
    bytes.into_boxed_slice()
}
fn ctl(world: &mut Pages, agent: ClientId, words: &[&[u8]], value: Box<[u8]>) -> temper_client_tools::jig::Printed {
    let _ = world.run(Until::Steps(32));
    assert!(world.root(agent).agent().ready());
    let number = world.root(agent).agent().calls() + 1;
    let temper_client_world::app::Event::Call { call, json, .. } = f::call(words) else { unreachable!() };
    // As with the key-file calls in account_tools, the shell supplies a one-shot value.
    // Its private-file read strips one final newline before the tools root parses it.
    let mut value = value.into_vec();
    if value.ends_with(b"\n") {
        let _ = value.pop();
    }
    world.event(
        agent,
        temper_client_world::app::Event::Call {
            number,
            call,
            json,
            value: Some(skein_lib::Secret::new(value.into_boxed_slice())),
        },
    );
    f::answered(world, agent, number)
}
fn printed_private(printed: &temper_client_tools::jig::Printed, value: &[u8]) {
    for bytes in [&printed.stdout, &printed.stderr] {
        assert!(skein_lib::bytes::find(bytes, value).is_none());
    }
}
#[test]
fn ctl_connects_and_replaces_a_single_token_on_the_same_world() {
    let mut world = new_world(forge::Kind::Forgejo);
    let agent = f::agent(&mut world);
    let api = secret(f::SEED);
    let git = secret(f::SEED + 1);
    let answer = ctl(
        &mut world,
        agent,
        &[
            b"forge",
            b"connect",
            b"forgejo",
            b"tokens",
            b"--api",
            b"forge.example:443",
            b"--git",
            b"https://forge.example/",
            b"--key",
            b"tokens-connect",
        ],
        tokens_file(&[(b"api", &api), (b"git", &git)]),
    );
    assert_eq!(answer.status, 0);
    assert!(skein_lib::bytes::find(&answer.stdout, b"connecting").is_some());
    printed_private(&answer, &api);
    printed_private(&answer, &git);
    let (number, request) = *world.front().admin_connects.last().expect("ctl Connect recorded");
    assert_eq!(world.front().admin_connects.len(), 1);
    finish(&mut world, number, request);
    let replacement = secret(f::SEED + 2);
    let number_text = skein_lib::Decimal::of(u64::from(number));
    let answer = ctl(
        &mut world,
        agent,
        &[b"forge", b"replace", number_text.as_bytes(), b"tokens", b"--key", b"tokens-replace"],
        tokens_file(&[(b"git", &replacement)]),
    );
    assert_eq!(answer.status, 0);
    assert!(skein_lib::bytes::find(&answer.stdout, b"connecting").is_some());
    printed_private(&answer, &git);
    assert_eq!(world.front().replacements.len(), 1);
    let (replaced, request) = *world.front().replacements.last().expect("ctl Replace recorded");
    assert_eq!(replaced, number);
    finish(&mut world, number, request);
    private(&mut world, agent, &api);
    private(&mut world, agent, &git);
    private(&mut world, agent, &replacement);
}
#[test]
fn ctl_refuses_a_disconnected_forge_and_a_broken_tokens_line() {
    let mut world = new_world(forge::Kind::Forgejo);
    send(&mut world, forge::Event::Connection { forge: FORGE, state: forge::Connection::NotConnected });
    let agent = f::agent(&mut world);
    let git = secret(f::SEED + 1);
    let answer = ctl(
        &mut world,
        agent,
        &[b"forge", b"replace", b"1", b"tokens", b"--key", b"not-connected"],
        tokens_file(&[(b"git", &git)]),
    );
    assert_eq!(answer.status, 2);
    assert!(skein_lib::bytes::find(&answer.stderr, b"not connected").is_some());
    printed_private(&answer, &git);
    assert_eq!(world.front().replacements.len(), 0);
    let answer = ctl(
        &mut world,
        agent,
        &[
            b"forge",
            b"connect",
            b"forgejo",
            b"tokens",
            b"--api",
            b"forge.example:443",
            b"--git",
            b"https://forge.example/",
            b"--key",
            b"broken-line",
        ],
        tokens_file(&[(b"api", &git), (b"broken", &git)]),
    );
    assert_eq!(answer.status, 1);
    assert!(skein_lib::bytes::find(&answer.stderr, b"tokens file line 2").is_some());
    printed_private(&answer, &git);
    assert_eq!(world.front().admin_connects.len(), 0);
    assert_eq!(world.front().replacements.len(), 0);
}

#[test]
fn a_stopped_replacement_keeps_the_connection_and_its_last_line() {
    let mut world = new_world(forge::Kind::Forgejo);
    let client = world.client_with(f::PARTY, Address::Forges, f::domain_limits(), browser);
    let value = secret(f::SEED);
    settle(&mut world);
    act(&mut world, client, Role::Button, b"Replace", Act::Press);
    act(&mut world, client, Role::TextField, b"Git token", Act::Input { value: value.clone() });
    act(&mut world, client, Role::Button, b"Replace the tokens", Act::Press);
    let (number, request) = *world.front().replacements.last().expect("replacement sent");
    send(
        &mut world,
        forge::Event::ConnectEnded {
            forge: number,
            request,
            connected: false,
            why: Some(b"Forgejo refused the git token".as_slice().into()),
            stop: None,
        },
    );
    settle(&mut world);
    let rendered = text(&world, client);
    assert!(skein_lib::bytes::find(&rendered, b"Connected as builder").is_some());
    assert!(skein_lib::bytes::find(&rendered, b"The last connect stopped: Forgejo refused the git token").is_some());
    assert!(
        !world
            .root(client)
            .person()
            .forge()
            .replacement(FORGE)
            .expect("open draft")
            .has(temper_client_domain::forge::replace::Field::Git)
    );
    private(&mut world, client, &value);
    let retry = secret(f::SEED + 1);
    act(&mut world, client, Role::TextField, b"Git token", Act::Input { value: retry.clone() });
    act(&mut world, client, Role::Button, b"Replace the tokens", Act::Press);
    let (number, next) = *world.front().replacements.last().expect("retry sent");
    assert_ne!(next, request);
    finish(&mut world, number, next);
    private(&mut world, client, &retry);
}
