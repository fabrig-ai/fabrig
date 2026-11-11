//! Ready's Create and reload on the fake box (temper/docs/design/testing/README.md, 2;
//! temper/docs/design/client/README.md, 2).
use crate::support::WITHIN;
use skein_lib::Time;
use skein_shell::{Clock, Host};
use skein_ui::{Role, State};
use skein_world::{Referee, real};
use std::path::Path;
use temper_client_domain::{Address, ChatsFilter, Step};
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};

const HOME_HEADLINE: &[u8] = b"My project is ready.";

/// The browser's client bounds over the native transport (client/README.md, 2).
pub(super) fn browser_bounds() -> temper_client_shell_terminal::limits::Limits {
    temper_client_shell_terminal::limits::Limits {
        client: temper_client_shell_web::limits::client(),
        ..temper_client_shell_terminal::limits::native()
    }
}

#[derive(Debug)]
enum Page {
    Starting,
    Ready,
    Home,
    Chats,
    Forges,
    Accounts,
    Done,
}
struct Judge {
    page: Page,
    press: bool,
    until: Time,
}
pub(super) fn named<'a>(client: &'a NativeClient, role: Role, name: &[u8]) -> Option<&'a skein_ui::Node> {
    client.tree().nodes().iter().find(|node| node.role == role && node.name.as_deref() == Some(name))
}
fn live(client: &NativeClient) -> bool {
    named(client, Role::Status, b"Live").is_some() && named(client, Role::Status, b"Behind").is_none()
}
fn chats() -> Address {
    Address::Chats { filter: ChatsFilter::All }
}
impl Referee<NativeClient> for Judge {
    fn act(&mut self, now: Time, clients: &mut [NativeClient]) {
        let client = &mut clients[0];
        match self.page {
            Page::Starting if client.started() && named(client, Role::Status, b"Owner").is_some() => {
                client.go(if self.press { Address::Welcome { step: Step::Ready } } else { chats() });
                self.page = if self.press { Page::Ready } else { Page::Chats };
            }
            Page::Ready => {
                if let Some(button) = named(client, Role::Button, b"Create your project")
                    && !button.states.as_slice().contains(&State::Disabled)
                {
                    let node = button.id;
                    assert!(
                        client.act(node, skein_ui::Act::Press),
                        "Welcome Ready: Create your project accepts its press"
                    );
                    self.page = Page::Home;
                }
            }
            Page::Home
                if named(client, Role::Heading { level: 1 }, HOME_HEADLINE).is_some()
                    && named(client, Role::Button, b"My project")
                        .is_some_and(|node| node.states.as_slice().contains(&State::Pressed))
                    && live(client) =>
            {
                client.close();
                self.page = Page::Done;
            }
            Page::Chats
                if named(client, Role::Heading { level: 1 }, b"Talk to an agent.").is_some()
                    && named(client, Role::Button, b"My project")
                        .is_some_and(|node| !self.press || node.states.as_slice().contains(&State::Pressed))
                    && live(client) =>
            {
                if self.press {
                    client.close();
                    self.page = Page::Done;
                } else {
                    client.go(Address::Forges);
                    self.page = Page::Forges;
                }
            }
            Page::Forges
                if named(client, Role::Heading { level: 1 }, b"Where your code lives.").is_some()
                    && named(client, Role::Disclosure, b"Forgejo on this machine").is_some() =>
            {
                client.go(Address::Accounts);
                self.page = Page::Accounts;
            }
            Page::Accounts
                if named(client, Role::Heading { level: 1 }, b"What agents run on.").is_some()
                    && named(client, Role::Disclosure, "OpenAI API key · held by the deployment".as_bytes())
                        .is_some()
                    && live(client) =>
            {
                client.close();
                self.page = Page::Done;
            }
            Page::Starting | Page::Home | Page::Chats | Page::Forges | Page::Accounts | Page::Done => {}
        }
        let missing = match self.page {
            Page::Starting => "Inbox: native client started and Status Owner",
            Page::Ready => "Welcome Ready: enabled Button Create your project",
            Page::Home => {
                "Home: Heading My project is ready. level 1, pressed Button My project, Status Live and no Status Behind"
            }
            Page::Chats => {
                "Chats: Heading Talk to an agent. level 1, Button My project, Status Live and no Status Behind"
            }
            Page::Forges => "Forges: Heading Forges level 1 and Disclosure Forgejo on this machine",
            Page::Accounts => {
                "Accounts: Heading Accounts level 1, Disclosure OpenAI API key · held by the deployment, Status Live and no Status Behind"
            }
            Page::Done => "nothing",
        };
        assert!(
            matches!(self.page, Page::Done) || now < self.until,
            "Ready's Create session {} missing {missing}; tree names: {:?}",
            if self.press { 1 } else { 2 },
            client
                .tree()
                .nodes()
                .iter()
                .filter_map(|node| node.name.as_deref())
                .map(String::from_utf8_lossy)
                .collect::<Vec<_>>()
        );
    }
    fn observe(&mut self, _now: Time, _clients: &[NativeClient]) {}
    fn next_deadline(&self) -> Option<Time> {
        (!matches!(self.page, Page::Done)).then_some(self.until)
    }
    fn passed(&self) -> bool {
        matches!(self.page, Page::Done)
    }
}

/// Session 1, the press; then session 2, the reload. Panics naming the page
/// and the missing node when a session does not reach its end within WITHIN.
pub(super) fn open_and_reload(config: &Path, client: &Path) {
    let document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(config.join("temper.json")).expect("configuration")).expect("JSON");
    let engine = document
        .pointer("/application/clients/listener/socket")
        .and_then(serde_json::Value::as_str)
        .expect("clients listener")
        .parse()
        .expect("socket");
    for press in [true, false] {
        let root = skein_shell::open_root(client).expect("signed-in client's parent root");
        let clock = Clock::new();
        let mut world =
            real::World::new(Judge { page: Page::Starting, press, until: clock.now().now.saturating_add(WITHIN) });
        world.spawn_with_fds(vec![root], || {
            NativeClient::new(
                Settings {
                    engine: skein_io::Addr::Ip(engine),
                    directory: b"client".as_slice().into(),
                    limits: browser_bounds(),
                },
                root,
                skein_shell::effective_user(),
                94,
                Start::Token,
            )
            .expect("browser-bounded native client")
        });
        let ended = world.run(&clock, WITHIN);
        assert!(ended.procs.iter().all(Host::is_empty), "Ready's Create session {} drained", if press { 1 } else { 2 });
    }
}
