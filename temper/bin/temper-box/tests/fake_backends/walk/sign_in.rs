//! The `ChatGPT` plan on the fake box: the fake sign-in leaves a usable model for Ready.
//! (temper/docs/design/protocol/setup/box.md, 5; temper/docs/design/client/pages/README.md.)
use super::open_default::{browser_bounds, named};
use super::{Act, Call, Walk};
use crate::support::WITHIN;
use skein_lib::Time;
use skein_shell::{Clock, Host};
use skein_ui::{Role, State};
use skein_world::{Referee, real};
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};
use temper_client_domain::{Address, Step};
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};

#[derive(Debug)]
enum Page {
    Starting,
    Ready,
    Done,
}
struct Judge {
    failed: Option<String>,
    page: Page,
    until: Time,
}
impl Referee<NativeClient> for Judge {
    fn act(&mut self, now: Time, clients: &mut [NativeClient]) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            let client = clients.first_mut().ok_or(String::from("native client"))?;
            match self.page {
                Page::Starting if client.started() && named(client, Role::Status, b"Owner").is_some() => {
                    client.go(Address::Welcome { step: Step::Ready });
                    self.page = Page::Ready;
                }
                Page::Ready => {
                    let model = client.tree().nodes().iter().any(|node| {
                        node.role == Role::Paragraph
                            && node.name.as_deref().is_some_and(|name| name.starts_with(b"ChatGPT plan"))
                    });
                    let create = named(client, Role::Button, b"Create your project")
                        .is_some_and(|node| !node.states.as_slice().contains(&State::Disabled));
                    if model && create {
                        client.close();
                        self.page = Page::Done;
                    }
                }
                Page::Starting | Page::Done => {}
            }
            if !matches!(self.page, Page::Done) && now >= self.until {
                return Err(format!(
                    "Ready missing ChatGPT plan and enabled Create your project at {:?}; tree names: {:?}",
                    self.page,
                    client
                        .tree()
                        .nodes()
                        .iter()
                        .filter_map(|node| node.name.as_deref())
                        .map(String::from_utf8_lossy)
                        .collect::<Vec<_>>()
                ));
            }

            Ok(())
        })();
        if let Err(why) = result {
            self.failed = Some(why);
        }
    }
    fn watch(&mut self, _now: Time, _clients: &[NativeClient]) -> Result<(), String> {
        if let Some(why) = self.failed.take() {
            return Err(why);
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!matches!(self.page, Page::Done)).then_some(self.until)
    }
    fn passed(&self) -> bool {
        if self.failed.is_some() {
            return false;
        }
        matches!(self.page, Page::Done)
    }
}

/// Ready, by the browser's bounds: the plan's model line, and Create enabled. It presses nothing.
fn ready_can_create(config: &Path, client: &Path) {
    let document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(config.join("temper.json")).expect("configuration")).expect("JSON");
    let engine = document
        .pointer("/application/clients/listener/socket")
        .and_then(serde_json::Value::as_str)
        .expect("clients listener")
        .parse()
        .expect("socket");
    let root = skein_shell::open_root(client).expect("signed-in client's parent root");
    let clock = Clock::new();
    let mut world =
        real::World::new(Judge { failed: None, page: Page::Starting, until: clock.now().now.saturating_add(WITHIN) });
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
    assert!(ended.procs.iter().all(Host::is_empty), "Ready's model check drained");
}

#[test]
fn a_chatgpt_browser_tab_on_the_fake_box_leaves_ready_able_to_create() {
    let walk = Walk::owned("walk-browser-sign-in");
    walk.run_then(vec![Act::Now(Call::new(&["forge", "setup", "--key", "walk-here", "--json"]))], browser_sign_in);
    walk.scratch.remove();
}

fn browser_socket(url: &[u8]) -> TcpStream {
    let (authority, _) = browser_target(url);
    let address: std::net::SocketAddr = authority.parse().expect("literal browser address");
    assert!(address.ip().is_loopback(), "browser stays on loopback");
    let socket = TcpStream::connect_timeout(&address, std::time::Duration::from_nanos(WITHIN.as_nanos()))
        .expect("browser connection");
    socket.set_read_timeout(Some(std::time::Duration::from_nanos(WITHIN.as_nanos()))).expect("browser read timeout");
    socket
}

fn browser_target(url: &[u8]) -> (&str, &str) {
    let url = std::str::from_utf8(url).expect("browser URL text");
    url.strip_prefix("http://").expect("plain HTTP browser URL").split_once('/').expect("browser URL path")
}

/// One GET of `url` on `socket`, read to the end of the response's head, with its
/// status and Location. It sends `host`, and no `connection: close`.
fn hop(socket: &mut TcpStream, url: &[u8]) -> (u16, Vec<u8>) {
    let (authority, target) = browser_target(url);
    write!(socket, "GET /{target} HTTP/1.1\r\nhost: {authority}\r\n\r\n").expect("browser GET");
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        assert!(head.len() < 16_384, "bounded browser response head");
        let mut byte = [0];
        socket.read_exact(&mut byte).expect("browser response head");
        head.extend_from_slice(&byte);
    }
    let mut lines = std::str::from_utf8(&head).expect("response head text").split("\r\n");
    let status = lines
        .next()
        .expect("status line")
        .split_whitespace()
        .nth(1)
        .expect("status code")
        .parse()
        .expect("numeric status");
    let location = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("location"))
        .map_or_else(Vec::new, |(_, value)| value.trim().as_bytes().to_vec());
    (status, location)
}

#[derive(Debug)]
enum Departure {
    Starting,
    Model,
    Signing,
    Done,
}
struct Depart<'a> {
    failed: Option<String>,
    page: Departure,
    until: Time,
    address: &'a std::cell::Cell<Option<Box<[u8]>>>,
}
impl Referee<NativeClient> for Depart<'_> {
    fn act(&mut self, now: Time, clients: &mut [NativeClient]) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            let client = clients.first_mut().ok_or(String::from("native client"))?;
            match self.page {
                Departure::Starting if client.started() && named(client, Role::Status, b"Owner").is_some() => {
                    client.go(Address::Welcome { step: Step::Model { way: None } });
                    self.page = Departure::Model;
                }
                Departure::Model => {
                    if let Some(button) = named(client, Role::Button, b"Continue with ChatGPT") {
                        let id = button.id;
                        if !client.act(id, skein_ui::Act::Press) {
                            return Err(String::from("Continue with ChatGPT accepts its press"));
                        }
                        self.page = Departure::Signing;
                    }
                }
                Departure::Starting | Departure::Signing | Departure::Done => {}
            }
            while let Some(notice) = client.notices() {
                if let temper_client_shell_terminal::client::Shown::Notice(
                    temper_client_shell_terminal::route::Notice::Leave { to },
                ) = notice
                {
                    self.address.set(Some(to));
                    client.close();
                    self.page = Departure::Done;
                }
            }
            if !matches!(self.page, Departure::Done) && now >= self.until {
                return Err(format!("browser sign-in stalled at {:?}", self.page));
            }

            Ok(())
        })();
        if let Err(why) = result {
            self.failed = Some(why);
        }
    }
    fn watch(&mut self, _now: Time, _clients: &[NativeClient]) -> Result<(), String> {
        if let Some(why) = self.failed.take() {
            return Err(why);
        }
        Ok(())
    }
    fn passed(&self) -> bool {
        if self.failed.is_some() {
            return false;
        }
        matches!(self.page, Departure::Done)
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.passed()).then_some(self.until)
    }
}
fn browser_sign_in(config: &Path, client: &Path) {
    const PAGE: &[u8] = b"Sign-in received. You can close this tab.\n";
    let document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(config.join("temper.json")).expect("configuration")).expect("JSON");
    let engine = document
        .pointer("/application/clients/listener/socket")
        .and_then(serde_json::Value::as_str)
        .expect("clients listener")
        .parse()
        .expect("socket");
    let root = skein_shell::open_root(client).expect("signed-in client's parent root");
    let clock = Clock::new();
    let address = std::cell::Cell::new(None);
    let mut world = real::World::new(Depart {
        failed: None,
        page: Departure::Starting,
        until: clock.now().now.saturating_add(WITHIN),
        address: &address,
    });
    world.spawn_with_fds(vec![root], || {
        NativeClient::new(
            Settings {
                engine: skein_io::Addr::Ip(engine),
                directory: b"client".as_slice().into(),
                limits: browser_bounds(),
            },
            root,
            skein_shell::effective_user(),
            95,
            Start::Token,
        )
        .expect("browser-bounded root")
    });
    let ended = world.run(&clock, WITHIN);
    assert!(ended.procs.iter().all(Host::is_empty), "tab address reader drained");
    let address = address.take().expect("sign-in tab address");
    let mut issuer = browser_socket(&address);
    let (status, location) = hop(&mut issuer, &address);
    assert_eq!(status, 302, "fake issuer's callback");
    let mut tab = browser_socket(&location);
    let (status, _) = hop(&mut tab, &location);
    assert_eq!(status, 200, "the sign-in tab ends on the listener's fixed page");
    let mut body = vec![0; PAGE.len()];
    tab.read_exact(&mut body).expect("listener page");
    assert_eq!(body, PAGE);
    ready_can_create(config, client);
}
