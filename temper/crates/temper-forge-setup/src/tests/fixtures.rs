//! Runtime setup values used by preparation and process specification tests.
use crate::{
    Setup,
    setup::{Account, Entry, Fake, Fetch, Mode, Owned, Repository, Runner},
};
use alloc::boxed::Box;
use core::net::{Ipv4Addr, SocketAddr};
use skein_json::Document;
use skein_lib::{Writer, bytes};
pub fn document(bytes: &[u8]) -> Document {
    jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .expect("test JSON")
}
pub fn setup() -> Setup {
    let bot = Account { login: b"temper".as_slice().into(), email: b"temper@example.invalid".as_slice().into() };
    Setup {
        entries: Box::new([
            Entry {
                forge: 1,
                directory: b"forge-1".as_slice().into(),
                bot: bot.clone(),
                mode: Mode::Managed { admin_token: Some(b"secrets/forgejo-admin".as_slice().into()) },
                asked: false,
            },
            Entry { forge: 2, directory: b"forge-2".as_slice().into(), bot, mode: Mode::Owned(owned()), asked: false },
        ]),
    }
}
pub fn fake_setup() -> Setup {
    let mut entries = skein_lib::List::with_capacity(3);
    for entry in setup().entries {
        entries.push(entry).expect("three fixture entries");
    }
    entries
        .push(Entry {
            forge: 3,
            directory: b"demo/forge".as_slice().into(),
            bot: Account { login: b"temper".as_slice().into(), email: b"temper@example.invalid".as_slice().into() },
            asked: false,
            mode: Mode::Fake(Fake {
                listen: ([127, 0, 0, 1], 8369).into(),
                git: b"demo/bin/git".as_slice().into(),
                owner: b"demo".as_slice().into(),
                repositories: Box::new([Repository {
                    name: b"widget".as_slice().into(),
                    landing: b"main".as_slice().into(),
                }]),
            }),
        })
        .expect("three fixture entries");
    Setup { entries: entries.into_boxed() }
}
pub fn owned() -> Owned {
    Owned {
        public: None,
        fetch: Fetch { forgejo: false, runner: false, mirror: None },
        forgejo: b"/opt/forgejo/forgejo".as_slice().into(),
        runner: Some(Runner {
            binary: b"/opt/forgejo/forgejo-runner".as_slice().into(),
            labels: Box::new([b"temper-host:host".as_slice().into()]),
            capacity: 1,
            path: b"/usr/local/bin:/usr/bin:/bin".as_slice().into(),
        }),
        listen: ([127, 0, 0, 1], 3000).into(),
        person: Account { login: b"ada".as_slice().into(), email: b"ada@example.invalid".as_slice().into() },
    }
}
pub fn place() -> crate::place::Place {
    crate::place::Place { directory: b"/srv/forge-2".as_slice().into(), user: b"ada".as_slice().into() }
}
pub fn secrets() -> crate::state::Secrets {
    use skein_lib::Secret;
    crate::state::Secrets {
        secret_key: Secret::new(b"key-secret".as_slice().into()),
        internal_token: Secret::new(b"internal-secret".as_slice().into()),
        jwt_secret: Secret::new(b"jwt-secret".as_slice().into()),
    }
}

pub struct Drive {
    runner: bool,
    machine: crate::prepare::Prepare,
    out: skein_lib::Queue<crate::prepare::Request>,
}
impl Drive {
    pub fn new(owned: Owned) -> Self {
        Self {
            runner: owned.runner.is_some(),
            machine: crate::prepare::Prepare::new(owned, place(), None),
            out: skein_lib::Queue::uncapped(),
        }
    }
    pub fn with_source_address(mut self, address: Option<Box<[u8]>>) -> Self {
        self.machine = self.machine.with_source_address(address, skein_lib::Duration::from_secs(7));
        self
    }
    pub fn send(&mut self, event: crate::prepare::Event) -> crate::prepare::Request {
        crate::prepare::step(&mut self.machine, event, &mut self.out);
        let request = self.out.pop().expect("one request");
        assert!(self.out.is_empty());
        request
    }
    pub fn versions(&mut self, found: crate::prepare::Found, runner: bool) -> crate::prepare::Request {
        use crate::{
            prepare::{Event, Request},
            spec::Which,
        };
        assert_eq!(
            self.send(Event::Found(found)),
            Request::Version { which: Which::Forgejo, spec: crate::spec::version(Which::Forgejo, &owned(), &place()) }
        );
        let mut request = self.send(Event::Printed(line(Which::Forgejo)));
        if runner {
            assert_eq!(
                request,
                Request::Version {
                    which: Which::Runner,
                    spec: crate::spec::version(Which::Runner, &owned(), &place())
                }
            );
            request = self.send(Event::Printed(line(Which::Runner)));
        }
        request
    }
    pub fn reach(&mut self, found: crate::prepare::Found) -> crate::prepare::Request {
        self.versions(found, self.runner)
    }
    pub fn drawn(&mut self, runner: bool) -> crate::prepare::Request {
        use crate::prepare::{Event, Request};
        let count = if runner { 5 } else { 4 };
        for value in 0..count {
            let next = self.send(Event::Drew(draw(value)));
            if value.checked_add(1).unwrap() == count {
                return next;
            }
            assert_eq!(next, Request::Draw);
        }
        unreachable!("four or five draws")
    }
}
pub fn draw(value: u8) -> skein_lib::Secret {
    skein_lib::Secret::new(Box::new([value; 32]))
}
pub fn line(which: crate::spec::Which) -> Box<str> {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    let (prefix, version) = match which {
        crate::spec::Which::Forgejo => (b"forgejo version ".as_slice(), VERSION),
        crate::spec::Which::Runner => (b"forgejo-runner version v".as_slice(), RUNNER_VERSION),
    };
    let mut w = Writer::new(prefix.len().checked_add(version.len()).unwrap());
    w.put(prefix).unwrap();
    w.put(version.as_bytes()).unwrap();
    let bytes = w.finish();
    let mut chunks = bytes.utf8_chunks();
    let chunk = chunks.next().unwrap();
    assert_eq!(chunk.invalid(), []);
    assert!(chunks.next().is_none());
    chunk.valid().into()
}
pub fn first() -> crate::prepare::Found {
    crate::prepare::Found {
        state: None,
        admin_token: false,
        person_headers: [false; 2],
        runner_secret: false,
        runner_configuration: None,
        files: false,
    }
}
pub fn later() -> crate::prepare::Found {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    crate::prepare::Found {
        state: Some(crate::state::encode(&crate::state::State {
            forgejo: VERSION.as_bytes().into(),
            runner: Some(RUNNER_VERSION.as_bytes().into()),
            prepared: true,
            secrets: secrets(),
        })),
        admin_token: true,
        person_headers: [true; 2],
        runner_secret: true,
        runner_configuration: Some(runner_configuration(b"kept-uuid")),
        files: true,
    }
}
pub fn person_created() -> crate::prepare::Event {
    ran(b"generated random password is 'person-password'\nNew user 'ada' has been successfully created!\n")
}
pub fn ran(stdout: &[u8]) -> crate::prepare::Event {
    use crate::prepare::{Event, Exit, Ran};
    Event::Ran(Ran {
        exit: Exit::Code(0),
        stdout: skein_lib::Secret::new(stdout.into()),
        stderr: skein_lib::Secret::new(Box::new([])),
    })
}
pub fn secret(value: &[u8]) -> skein_lib::Secret {
    skein_lib::Secret::new(bytes::copy_of(value))
}
pub fn prepared() -> crate::state::State {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    crate::state::State {
        forgejo: VERSION.as_bytes().into(),
        runner: Some(RUNNER_VERSION.as_bytes().into()),
        prepared: true,
        secrets: secrets(),
    }
}
pub fn launch(which: crate::spec::Which) -> crate::run::Launch {
    crate::run::launch(&owned(), &place(), which).expect("configured process")
}
pub fn suffixed_line(which: crate::spec::Which, suffix: &str) -> Box<str> {
    let line = line(which);
    let mut writer = Writer::new(line.len().checked_add(suffix.len()).unwrap());
    writer.put(line.as_bytes()).unwrap();
    writer.put(suffix.as_bytes()).unwrap();
    let bytes = writer.finish();
    let mut chunks = bytes.utf8_chunks();
    let chunk = chunks.next().unwrap();
    assert_eq!(chunk.invalid(), []);
    assert!(chunks.next().is_none());
    chunk.valid().into()
}

pub fn older(forgejo: Option<&[u8]>, runner: Option<&[u8]>) -> crate::prepare::Found {
    let mut state = prepared();
    if let Some(version) = forgejo {
        state.forgejo = version.into();
    }
    if let Some(version) = runner {
        state.runner = Some(version.into());
    }
    crate::prepare::Found { state: Some(crate::state::encode(&state)), ..later() }
}
pub fn backup(recorded: &[u8]) -> Box<[u8]> {
    let pin = skein_forge::forgejo::pin::VERSION.as_bytes();
    let mut name = Writer::new(recorded.len().checked_add(pin.len()).unwrap().checked_add(16).unwrap());
    for piece in [b"forgejo-".as_slice(), recorded, b"-to-", pin, b".zip"] {
        name.put(piece).unwrap();
    }
    name.finish()
}
pub fn ran_ok() -> crate::prepare::Event {
    ran(b"runner-uuid\n")
}
pub fn run_command(command: crate::spec::Command) -> crate::prepare::Request {
    crate::prepare::Request::Run { command, spec: crate::spec::command(command, &owned(), &place()) }
}
pub fn runner_added() -> crate::prepare::Found {
    let mut state = prepared();
    state.runner = None;
    crate::prepare::Found {
        state: Some(crate::state::encode(&state)),
        runner_secret: false,
        runner_configuration: None,
        ..later()
    }
}
pub fn runner_configuration(uuid: &[u8]) -> skein_lib::Secret {
    let owned = owned();
    skein_lib::Secret::new(crate::runner::configuration(owned.runner.as_ref().unwrap(), &owned, &place(), uuid))
}
pub fn failed() -> crate::prepare::Event {
    crate::prepare::Event::Ran(crate::prepare::Ran {
        exit: crate::prepare::Exit::Code(1),
        stdout: secret(b""),
        stderr: secret(b"broken"),
    })
}

pub fn listener() -> SocketAddr {
    SocketAddr::from((Ipv4Addr::LOCALHOST, 9898))
}
