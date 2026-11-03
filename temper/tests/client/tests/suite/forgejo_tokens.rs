//! Bot tokens on the joined client/engine (temper/docs/design/client/forge.md, 5.4).
use jig_client_world::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step as PersonStep};
use skein_lib::Duration;
use skein_ui::{Act, Role};
use skein_world::domain::Verdict;
use temper_client_domain::{Address, ChatsFilter, Choice};
use temper_client_world::{app::Temper, fixtures as f, front::Joint};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, walking};
type Pages = World<Temper, Joint>;
const WITHIN_REQUEST: Duration = Duration::from_millis(100);
fn new_world() -> Pages {
    let mut world = super::forges_edges::admin_world(forge::Mode::Connected);
    let front = world.front();
    let mut store = front.engine.take_store();
    let forges = store
        .rows
        .iter()
        .filter(|(_, row)| matches!(row, engine::Record::Forge { .. }))
        .map(|(key, _)| key.clone())
        .collect();
    store.seed(Box::new([]), forges);
    let limits = front.engine.env.limits;
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(f::SEED);
        config.core.connectors = Box::new([temper_client_protocol_wire::FORGE]);
        config.forges = Box::new([]);
        config.under_box = false;
        config
    });
    front.engine = Driver::configured(store, configs, &limits);
    front.engine.settle();
    world
}
fn see(role: Role, name: &[u8]) -> PersonStep<Address> {
    PersonStep::See { find: Find::exactly_named(role, name), within: f::WITHIN }
}
fn follow(world: &mut Pages, client: ClientId, steps: Vec<PersonStep<Address>>) {
    world.person(client, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn node(world: &Pages, client: ClientId, role: Role, name: &[u8]) -> skein_ui::NodeId {
    world
        .tree(client)
        .nodes()
        .iter()
        .find(|n| n.role == role && n.name.as_deref() == Some(name))
        .expect("named node")
        .id
}
fn settle(world: &mut Pages) {
    let _ = world.run(Until::Time(world.now().saturating_add(WITHIN_REQUEST)));
}
fn press(world: &mut Pages, client: ClientId, role: Role, name: &[u8]) {
    let id = node(world, client, role, name);
    world.act(client, id, Act::Press);
    settle(world);
}
fn input(world: &mut Pages, client: ClientId, field: &[u8], value: &[u8]) {
    let id = node(world, client, Role::TextField, field);
    world.act(client, id, Act::Input { value: value.into() });
    settle(world);
}
fn tokens() -> [Box<[u8]>; 3] {
    let mut rng = skein_lib::Rng::new(f::SEED);
    std::array::from_fn(|_| {
        (0..32).map(|_| b'a' + u8::try_from(rng.next_u64() % 26).expect("alphabet index fits")).collect()
    })
}
fn private(world: &mut Pages, client: ClientId, secrets: &[Box<[u8]>]) {
    for node in world.tree(client).nodes() {
        for value in [node.name.as_deref(), node.description.as_deref(), node.value.as_ref().map(|v| v.text.as_ref())]
            .into_iter()
            .flatten()
        {
            for secret in secrets {
                assert!(skein_lib::bytes::find(value, secret).is_none());
            }
        }
        for run in &node.text {
            for secret in secrets {
                assert!(skein_lib::bytes::find(&run.text, secret).is_none());
            }
        }
    }
    let engine = &world.front().engine;
    for captured in [
        format!("{:?}", engine.store.rows),
        format!("{:?}", engine.transactions),
        format!("{:?}", engine.delivered),
        format!("{:?}", engine.immediate),
    ] {
        for secret in secrets {
            assert!(skein_lib::bytes::find(captured.as_bytes(), secret).is_none());
        }
    }
}
fn paste(world: &mut Pages, client: ClientId, values: &[Box<[u8]>]) {
    for (name, value) in
        [(b"API token".as_slice(), &values[0]), (b"Git token", &values[1]), (b"Webhook secret", &values[2])]
    {
        input(world, client, name, value);
    }
}
#[test]
fn your_forgejo_connects_with_bot_tokens_retries_a_stop_and_is_ready() {
    let mut world = new_world();
    let values = tokens();
    let client = world.client(f::PARTY, Address::Chats { filter: ChatsFilter::All, start: false });
    follow(
        &mut world,
        client,
        vec![
            see(Role::Heading { level: 1 }, b"Chats"),
            PersonStep::Go { address: Address::ConnectForge { forge: None } },
            see(Role::Link, b"Your Forgejo"),
        ],
    );
    press(&mut world, client, Role::Link, b"Your Forgejo");
    follow(
        &mut world,
        client,
        vec![
            see(Role::Heading { level: 2 }, b"Your Forgejo"),
            see(Role::Button, b"Connect as its admin"),
            see(Role::TextField, b"API token"),
        ],
    );
    input(&mut world, client, b"Its address", b"forge.example");
    input(&mut world, client, b"Its git base", b"https://forge.example/");
    input(&mut world, client, b"Its web address", b"https://forge.example");
    paste(&mut world, client, &values);
    private(&mut world, client, &values);
    press(&mut world, client, Role::Button, b"Connect with its tokens");
    let (number, request) = *world.front().admin_connects.last().expect("tokens Connect sent");
    assert_eq!(world.front().admin_connects.len(), 1);
    let connecting = format!("Connecting forge {number} with its bot’s tokens.");
    follow(
        &mut world,
        client,
        vec![
            see(Role::Heading { level: 2 }, connecting.as_bytes()),
            see(Role::Paragraph, b"Checking that both answer as one user, not a site admin."),
        ],
    );
    private(&mut world, client, &values);
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::ConnectEnded {
        forge: number,
        request,
        connected: false,
        why: Some(b"the git token answers as another user than the API token".as_slice().into()),
        stop: None,
    })));
    follow(
        &mut world,
        client,
        vec![
            see(Role::Button, b"Connect with its tokens"),
            see(Role::Alert, b"The last connect stopped: the git token answers as another user than the API token"),
        ],
    );
    for name in [b"API token".as_slice(), b"Git token", b"Webhook secret"] {
        let id = node(&world, client, Role::TextField, name);
        let tree = world.tree(client);
        let entry = tree.nodes().iter().find(|n| n.id == id).expect("token field");
        assert!(entry.value.as_ref().expect("token field value").text.is_empty());
    }
    private(&mut world, client, &values);
    paste(&mut world, client, &values);
    press(&mut world, client, Role::Button, b"Connect with its tokens");
    let (kept, retry) = *world.front().admin_connects.last().expect("retry Connect sent");
    assert_eq!(kept, number);
    assert_ne!(retry, request);
    follow(&mut world, client, vec![see(Role::Heading { level: 2 }, connecting.as_bytes())]);
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::BotLogin {
        forge: number,
        login: b"builder".as_slice().into(),
    })));
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::ConnectEnded {
        forge: number,
        request: retry,
        connected: true,
        why: None,
        stop: None,
    })));
    world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge: number,
        state: forge::Connection::Connected { id: 7, login: b"builder".as_slice().into() },
    })));
    follow(
        &mut world,
        client,
        vec![see(Role::Heading { level: 2 }, b"Forgejo is connected"), see(Role::Paragraph, b"Connected as builder.")],
    );
    private(&mut world, client, &values);
    assert_eq!(world.front().admin_connects.len(), 2);
    // The next addition opens fresh addresses/forms rather than the completed forge.
    follow(
        &mut world,
        client,
        vec![
            PersonStep::Go { address: Address::ConnectForge { forge: Some(Choice::Managed) } },
            see(Role::Heading { level: 2 }, b"Your Forgejo"),
            see(Role::TextField, b"API token"),
        ],
    );
}
