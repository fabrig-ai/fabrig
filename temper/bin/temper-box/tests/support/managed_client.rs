//! The sitting's fake person uses Ready's real domain, view and HTTP transport.
use super::{
    patience::Patience,
    relay_managed::{NAME, OWNER},
};
use jig_client_protocol_link::Signing;
use skein_fake_person::{Doing, Find, Next, Step, TreeFace};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_ui::Role;
use skein_world::{Referee, real};
use std::path::Path;
use temper_client_domain::{Address, Choice};
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};

struct Person {
    actor: skein_fake_person::Person<Address>,
    sign_in: bool,
    finished: bool,
    until: Patience,
    wake: Option<Time>,
    link: Box<[u8]>,
    retry: Option<Time>,
}
impl Referee<NativeClient> for Person {
    fn act(&mut self, now: Time, clients: &mut [NativeClient]) {
        let client = &mut clients[0];
        while let Some(signing) = client.signing() {
            match signing {
                Signing::SignedIn { party } => {
                    assert_eq!(party, 1, "the printed link signs in party 1");
                    self.retry = None;
                    if self.sign_in {
                        self.finished = true;
                        client.close();
                    }
                }
                Signing::Again | Signing::Busy => self.retry = Some(now.saturating_add(Duration::from_millis(100))),
                Signing::LinkUsed
                | Signing::LinkExpired
                | Signing::Approval(_)
                | Signing::LinkUnknown
                | Signing::Refused
                | Signing::Lost
                | Signing::SignedOut
                | Signing::Full { .. } => {
                    panic!("the sitting's sign-in was refused");
                }
            }
        }
        if self.retry.is_some_and(|at| at <= now) {
            self.retry = None;
            client.sign_in(&self.link).expect("retry the same printed link");
        }
        if !self.finished
            && let Some(figures) = self.until.failed()
        {
            panic!("signed-in Ready timed out: {figures}");
        }
        if self.until.deferred() || self.finished || !client.started() || self.sign_in {
            return;
        }
        self.wake = Some(now);
        match self.actor.next(&mut TreeFace::new(client.tree()), now) {
            Next::Do(Doing::Go { address }) => client.go(address),
            Next::Do(Doing::Act { node, act }) => assert!(client.act(node, act), "Ready offers the person's act"),
            Next::Do(Doing::Reload) => panic!("no reload in the sitting"),
            Next::Wait { until } => self.wake = until,
            Next::Done => {
                self.finished = true;
                client.close();
            }
            Next::Failed { step, why } => panic!("Ready failed at semantic step {step}: {why:?}"),
        }
    }
    fn observe(&mut self, _now: Time, _clients: &[NativeClient]) {
        if !self.finished
            && let Some(figures) = self.until.failed()
        {
            panic!("signed-in Ready timed out: {figures}");
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.finished { None } else { [self.until.until(), self.wake, self.retry].into_iter().flatten().min() }
    }
    fn passed(&self) -> bool {
        self.finished
    }
}

#[derive(Clone, Copy)]
pub enum Mode {
    SignIn,
    Adopt,
}

pub fn role() -> Option<Mode> {
    let mode = temper_test_env::value(&temper_test_env::MANAGED_CLIENT)
        .map(|mode| mode.into_string().expect("invalid fake sitting client mode"));
    match mode.as_deref() {
        Some("sign-in") => Some(Mode::SignIn),
        Some("adopt") => Some(Mode::Adopt),
        Some(_) => panic!("unknown fake sitting client mode"),
        None => None,
    }
}

pub fn run(mode: Mode) {
    let mut link = String::new();
    std::io::stdin().read_line(&mut link).expect("private printed link on stdin");
    ready(Path::new("."), link.trim_end(), mode);
}

pub fn ready(root: &Path, link: &str, mode: Mode) {
    let sign_in = match mode {
        Mode::SignIn => true,
        Mode::Adopt => false,
    };
    let address: std::net::SocketAddr = link
        .strip_prefix("http://")
        .and_then(|url| url.split('/').next())
        .expect("HTTP printed link")
        .parse()
        .expect("numeric client listener");
    assert!(address.ip().is_loopback(), "fake sitting stays on loopback");
    let within = Duration::from_secs(30);
    let steps = adoption_steps(within);
    let root = skein_shell::open_root(root).expect("sitting scratch root");
    let clock = Clock::new();
    let mut world = real::World::new(Person {
        actor: skein_fake_person::Person::new(steps),
        sign_in,
        finished: false,
        until: Patience::arm("signed-in Ready", within),
        wake: None,
        link: link.as_bytes().into(),
        retry: None,
    });
    world.spawn_with_fds(vec![root], || {
        NativeClient::new(
            Settings {
                engine: skein_io::Addr::Ip(address),
                directory: b"client".as_slice().into(),
                limits: temper_client_shell_terminal::limits::native(),
            },
            root,
            skein_shell::effective_user(),
            if sign_in { 91 } else { 92 },
            if sign_in { Start::Link(link.as_bytes().into()) } else { Start::Token },
        )
        .expect("sitting's native client")
    });
    let ended = world.run(&clock, Duration::from_secs(40));
    assert!(ended.procs.iter().all(Host::is_empty), "sitting client drained");
    println!("signed-in Ready client completed");
}

fn adoption_steps(within: Duration) -> Box<[Step<Address>]> {
    Box::new([
        Step::Go { address: Address::ConnectForge { forge: Some(Choice::Forge(1)) } },
        Step::See { find: Find::named(Role::Link, b"Adopt a repository"), within },
        Step::Press { find: Find::named(Role::Link, b"Adopt a repository") },
        Step::See { find: Find::named(Role::TextField, b"Owner"), within },
        Step::See { find: Find::exactly_named(Role::Check, format!("{OWNER}/{NAME}").as_bytes()), within },
        Step::Type { find: Find::named(Role::TextField, b"Owner"), words: OWNER.as_bytes().into() },
        Step::Type { find: Find::exactly_named(Role::TextField, b"Name"), words: NAME.as_bytes().into() },
        Step::Type { find: Find::named(Role::TextField, b"Landing branch"), words: b"main".as_slice().into() },
        Step::See { find: Find::named(Role::Button, format!("Adopt {OWNER}/{NAME}").as_bytes()), within },
        Step::Press { find: Find::named(Role::Button, b"Adopt") },
        Step::See { find: Find::text(format!("{OWNER}/{NAME}").as_bytes()), within },
    ])
}
