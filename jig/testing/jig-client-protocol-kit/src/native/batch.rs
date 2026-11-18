//! Native HTTP to the reference service: one iteration, one batch, and kept stream replacement.
//! jig/docs/design/protocol/client/README.md, 11.
use super::{Machine, Service, config, files, settings};
use jig_client::{self as c, Event, Handle, Message, Request, Told};
use jig_client_shell_native::Shell;
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{Duration, List, Queue, Time, Wall};
use skein_shell::{Delegate, Host};
use skein_world::{Memory, Referee};

const READS: [Handle; 3] = [
    Handle { asker: c::Asker(9), number: 1 },
    Handle { asker: c::Asker(9), number: 2 },
    Handle { asker: c::Asker(9), number: 3 },
];
const WATCH: Handle = Handle { asker: c::Asker(9), number: 4 };
struct Client {
    shell: Shell,
    machine: c::Client,
    completions: Queue<Complete>,
    out: Queue<Request>,
    kept: Vec<c::Record>,
    pages: Vec<(Handle, c::Readout)>,
    batches: Vec<u32>,
    singles: u32,
    gone_calls: u32,
    gone_events: u32,
    replaces: Vec<Option<[u8; 16]>>,
    loaded: bool,
    closing: bool,
}
impl Client {
    fn new(root: Fd, seed: u64) -> Self {
        let mut shell = Shell::new(settings());
        shell.adopt(root, 1000, seed);
        Self {
            shell,
            machine: c::Client::new(&c::Limits { domain: crate::fixtures::domain_limits(), exchanges: 8 }, seed)
                .expect("client limits"),
            completions: Queue::uncapped(),
            out: Queue::uncapped(),
            kept: vec![],
            pages: vec![],
            batches: vec![],
            singles: 0,
            gone_calls: 0,
            gone_events: 0,
            replaces: vec![],
            loaded: false,
            closing: false,
        }
    }
    fn event(&mut self, now: Time, event: Event) {
        if matches!(event, Event::StreamGone) {
            self.gone_events += 1;
        }
        self.machine.step(now, Wall::EPOCH, event, &mut self.out);
    }
    fn ask(&mut self, now: Time) {
        for handle in READS {
            self.event(now, Event::Read { handle, read: c::Readable::Jig(c::contract::Read::Me) });
        }
        self.event(
            now,
            Event::Watch { handle: WATCH, subject: c::Watchable::Jig(c::contract::Subject::Inbox { window: 2 }) },
        );
    }
    fn start(&mut self, now: Time, kept: List<c::Record>, credential: skein_lib::Secret) {
        self.event(now, Event::Start { kept });
        self.event(now, Event::SignIn { with: c::With::Held(credential) });
    }
}
impl Host for Client {
    fn iterate(&mut self, now: Time, wall: Wall) {
        while let Some(event) = self.completions.pop() {
            self.shell.completions().push(event);
        }
        self.shell.iterate(now, wall);
        if self.shell.loaded().is_some() {
            self.loaded = true;
        }
        while let Some(event) = self.shell.events().pop() {
            self.event(now, event);
        }
        if !self.closing {
            self.machine.fire(now, wall, &mut self.out);
        }
        self.machine.settle(now, wall, &mut self.out);
        while let Some(request) = self.out.pop() {
            match &request {
                Request::Exchange { message: Message::Batch(bytes), .. } => {
                    let batch = jig_client_protocol_wire::Batch::decode(
                        &jig_client_protocol_wire::CEILINGS,
                        &mut skein_lib::Reader::new(bytes),
                    )
                    .expect("batch record");
                    self.batches.push(batch.members().len());
                }
                Request::Exchange { message: Message::Read(_), .. } => self.singles += 1,
                Request::Exchange { message: Message::StreamGone(_), .. } => self.gone_calls += 1,
                Request::OpenStream { replaces, .. } => self.replaces.push(*replaces),
                Request::Save { records } => {
                    for record in records {
                        self.kept.retain(|r| r.key != record.key);
                        self.kept.push(record.clone());
                    }
                }
                Request::Told { told: Told::Page { handle, page } } => self.pages.push((*handle, page.clone())),
                _ => {}
            }
            self.shell.down(request);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.shell.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty() || !self.out.is_empty() || self.shell.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.shell.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        if self.closing { None } else { self.machine.next_deadline() }
    }
    fn is_empty(&self) -> bool {
        self.closing && self.shell.is_empty()
    }
    fn operations(&self) -> u32 {
        self.shell.operations()
    }
}
enum Process {
    Service(Box<Service>),
    Client(Box<Client>),
}
impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Service(s) => s.as_ref(),
            Self::Client(c) => c.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Service(s) => s.as_mut(),
            Self::Client(c) => c.as_mut(),
        }
    }
}
#[expect(clippy::struct_excessive_bools, reason = "scenario choice and three referee milestones")]
struct Judge {
    replacement: bool,
    signed: bool,
    asked: bool,
    done: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let [Process::Service(service), Process::Client(first), Process::Client(second)] = procs else {
            panic!("service and clients")
        };
        if !self.signed
            && first.loaded
            && second.loaded
            && let Some(link) = service.0.links().pop()
        {
            let secret = first.shell.sign_in(&link).expect("printed link");
            first.event(now, Event::Start { kept: List::with_capacity(0) });
            first.event(now, Event::SignIn { with: c::With::Link { secret, held: None } });
            self.signed = true;
        }
        if !self.asked && first.machine.link().state == c::Link::Live {
            if self.replacement {
                let credential = first.machine.link().credential.cloned().expect("signed in");
                let kept = jig_client_kit::test::list(first.kept.clone());
                assert!(first.kept.iter().any(|r| r.key.as_ref() == c::STREAM_KEY));
                second.start(now, kept, credential);
            } else {
                first.ask(now);
            }
            self.asked = true;
        }
        let answered = if self.replacement {
            second.machine.link().state == c::Link::Live && first.gone_events > 0
        } else {
            first.pages.len() == READS.len() && first.machine.document(WATCH).is_some_and(|d| d.state == c::State::Live)
        };
        if !self.done && self.asked && answered {
            for client in [first, second] {
                client.closing = true;
                client.shell.close();
            }
            service.0.shutdown();
            self.done = true;
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Process]) -> Result<(), String> {
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.done { None } else { Some(Time::ZERO.saturating_add(Duration::from_secs(5))) }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        if !self.done && now >= Time::ZERO.saturating_add(Duration::from_secs(5)) {
            Some("native batch story did not finish".into())
        } else {
            None
        }
    }
    fn passed(&self) -> bool {
        self.done
    }
}
/// Run the native batching or replacement story through jig-test-service.
pub fn run(seed: u64, replacement: bool) {
    let mut machine = skein_fake_machine::Machine::for_user(1000);
    let roots = [machine.lay(&[]), machine.lay(&[]), machine.lay(&[])];
    let mut world = skein_world::World::new(
        seed,
        skein_sim::Config::calm(),
        Judge { replacement, signed: false, asked: false, done: false },
        Memory::Unchecked,
    )
    .with_machine(Machine(machine));
    world.spawn_root(skein_sim::Handle::new(roots[0].raw()), |root| {
        let seeds = jig_test_service::Seeds {
            deployment: [7; 16],
            installation: [9; 16],
            seed,
            crypto: skein_world::crypto_seed(seed),
        };
        let mut service = jig_test_service::startup(&config::document(), &files(), None, &seeds, None, None)
            .expect("engine configuration");
        service.adopt_store(root, 1000).expect("store directory");
        Process::Service(Box::new(Service(service)))
    });
    for root in &roots[1..] {
        world.spawn_root(skein_sim::Handle::new(root.raw()), |root| Process::Client(Box::new(Client::new(root, seed))));
    }
    let outcome = world.run();
    let [Process::Service(_), Process::Client(first), Process::Client(second)] = outcome.procs.as_slice() else {
        panic!("service and clients")
    };
    assert!(outcome.procs.iter().all(Host::is_empty), "native hosts drained");
    assert_eq!(first.gone_calls + second.gone_calls, 0, "no /v1/stream-gone");
    if replacement {
        assert_eq!(second.replaces.len(), 1);
        let record = first.kept.iter().find(|r| r.key.as_ref() == c::STREAM_KEY).expect("kept stream");
        let kept = jig_client_protocol_wire::Kept::decode(
            &jig_client_protocol_wire::CEILINGS,
            &mut skein_lib::Reader::new(&record.bytes),
        )
        .expect("kept stream record");
        assert_eq!(second.replaces[0].as_ref().map(<[u8; 16]>::as_slice), Some(kept.record()));
        assert!(first.gone_events > 0, "the predecessor was ended");
    } else {
        assert_eq!(first.batches, [4], "one /v1/batch for this iteration");
        assert_eq!(first.singles, 0, "no /v1/read beside it");
        for handle in READS {
            let pages: Vec<_> = first.pages.iter().filter(|(h, _)| *h == handle).collect();
            assert!(matches!(
                pages.as_slice(),
                [(_, c::Readout::Jig(c::contract::ReadResult::Value(c::contract::ReadValue::Me(_))))]
            ));
        }
    }
}
