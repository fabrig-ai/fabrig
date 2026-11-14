//! Private sign-in from an absent root, cut before every submission (runtime/io/README.md, 5).
use super::*;
use crate::private::FileSystem;
use crate::restart::Restartable;
use skein_io::kernel::Fd;
use skein_world::{Cut, Delegate, Inherited, StartupRoot};

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;
type Process = Restartable<CutHost>;

struct CutHost {
    process: super::Process,
    root: Option<Fd>,
    start_sign_in: bool,
    ending: bool,
    opened_at: Option<u32>,
    stored_at: Option<u32>,
}
impl CutHost {
    fn wrap(process: super::Process, inherited: &Inherited) -> Process {
        Restartable::new(
            Self {
                process,
                root: inherited.roots.first().map(|(_, fd)| *fd),
                start_sign_in: false,
                ending: false,
                opened_at: None,
                stored_at: None,
            },
            inherited.signal,
            true,
            Self::prepare,
            Self::mark,
        )
    }
    fn prepare(&mut self) {
        if self.start_sign_in {
            self.start_sign_in = false;
            let mut client = super::client_with_limits(Story::Confidential, false, &limits(), 8);
            client.file_driver = Some(skein_world::files::Driver::new(
                self.root.expect("owner startup root"),
                4,
                client.limits.client.document.record_bytes,
                client.limits.file_stall,
                1234,
            ));
            self.process = super::Process::Client(Box::new(client));
        }
    }
    fn mark(&mut self, issued: u32) {
        if let super::Process::Client(client) = &self.process
            && let Some(owner) = &client.sign_in
        {
            if owner.opened_private.is_some() && self.opened_at.is_none() {
                self.opened_at = Some(issued);
            }
            if owner.stored_private.is_some() && self.stored_at.is_none() {
                self.stored_at = Some(issued);
            }
        }
    }
}
impl Delegate for CutHost {
    fn host(&self) -> &dyn Host {
        &self.process
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        &mut self.process
    }
}

fn limits() -> accounts::Limits {
    let mut limits = crate::world::limits();
    limits.accounts = 1;
    limits.exchanges = 1;
    limits.listeners = 0;
    limits.io.sockets = 1;
    limits.io.intake = skein_tls::client::LARGEST_READ;
    limits.io.receive = 256;
    limits.io.output = 2048;
    limits.io.sends = 1;
    limits.http.request = 1024;
    limits.http.head = 512;
    limits.http.headers = 8;
    limits.http.read = 128;
    limits.http.send = 128;
    limits.tls.read = 512;
    limits.tls.send = 1024;
    limits.tls.records = skein_tls::client::MAX_RECORD;
    limits.client.document.document_bytes = 512;
    limits.client.document.string_bytes = 128;
    limits.client.document.token_bytes = 64;
    limits.client.document.record_bytes = 512;
    limits.client.document.id_token_bytes = 64;
    limits.client.document.tokens = 32;
    limits
}
fn recovery(root: Fd) -> Client {
    let mut profile = limits();
    profile.refresh_lead = Duration::ZERO;
    let mut client = Client::with_limits(crate::world::Story::NotKept, Transport::Plaintext, &profile, 8);
    client.file_driver =
        Some(skein_world::files::Driver::new(root, 3, profile.client.document.record_bytes, profile.file_stall, 1234));
    client
}
fn issuer(inherited: &Inherited) -> Process {
    CutHost::wrap(super::issuer::with_access_token(Story::Confidential, b"new"), inherited)
}
fn browser(inherited: &Inherited) -> Process {
    CutHost::wrap(super::Process::Browser(Box::new(Browser::new(Story::Confidential))), inherited)
}
fn owner(inherited: &Inherited) -> Process {
    let root = inherited.roots.first().expect("owner startup root").1;
    CutHost::wrap(super::Process::Client(Box::new(recovery(root))), inherited)
}

struct Judge {
    original: bool,
    sign_in: super::Judge,
    wake: Option<Time>,
    passed: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        if !self.original {
            procs[1].inner.start_sign_in = true;
            self.original = true;
        }
        let address = procs[0].inner.process.issuer().expect("issuer slot").address();
        procs[1].inner.process.client_mut().expect("owner slot").address = address;
        if procs[1].inner.process.client().is_some_and(|client| client.sign_in.is_some()) {
            let mut visit = [0; 1024];
            let mut visit_len = 0;
            let mut redirect = [0; 1024];
            let mut redirect_len = 0;
            let browser = procs[2].inner.process.browser().expect("browser slot");
            if let Some(uri) = &browser.redirect {
                redirect_len = uri.len();
                redirect[..redirect_len].copy_from_slice(uri);
            }
            let client = procs[1].inner.process.client_mut().expect("owner slot");
            if let Some(url) = super::owner(client).visits.first() {
                visit_len = url.len();
                visit[..visit_len].copy_from_slice(url);
            }
            if !self.sign_in.redirected && redirect_len > 0 {
                super::owner_mut(client).redirected[..redirect_len].copy_from_slice(&redirect[..redirect_len]);
                super::owner_mut(client).redirected_len = redirect_len;
                self.sign_in.redirected = true;
            }
            if self.sign_in.delivered == 0
                && visit_len > 0
                && let super::Process::Browser(browser) = &mut procs[2].inner.process
            {
                browser.inbox[..visit_len].copy_from_slice(&visit[..visit_len]);
                browser.inbox_len = visit_len;
                self.sign_in.delivered = 1;
            }
        }
        if procs[1].inner.process.is_empty() && procs[2].inner.process.is_empty() && !procs[0].inner.ending {
            procs[0].inner.process.issuer_mut().expect("issuer slot").shutdown();
            procs[0].inner.ending = true;
        }
    }
    fn watch(&mut self, now: Time, procs: &[Process]) -> Result<(), String> {
        self.passed = procs.iter().all(Host::is_empty);
        let address_ready = procs[0].inner.process.issuer().expect("issuer slot").address().is_some();
        let client = procs[1].inner.process.client().expect("owner slot");
        let owner_ready = if let Some(owner) = &client.sign_in {
            (client.component.is_none() && address_ready) || (!owner.visits.is_empty() && self.sign_in.delivered == 0)
        } else {
            client.unstarted() && address_ready
        };
        let browser = procs[2].inner.process.browser().expect("browser slot");
        let redirect_ready = browser.redirect.is_some() && !self.sign_in.redirected && client.sign_in.is_some();
        let shutdown_ready =
            procs[1].inner.process.is_empty() && procs[2].inner.process.is_empty() && !procs[0].inner.ending;
        self.wake = (owner_ready || redirect_ready || shutdown_ready).then_some(now);
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake.or(Some(Time::from_nanos(180_000_000_000)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now.as_nanos() >= 180_000_000_000 && !self.passed).then(|| "private sign-in cut did not settle".to_owned())
    }
    fn passed(&self) -> bool {
        self.passed
    }
}
fn world() -> World<Process, Judge, FileSystem> {
    let mut world = World::new(
        0,
        skein_sim::Config::calm(),
        Judge {
            original: false,
            sign_in: super::Judge {
                delivered: 0,
                redirected: false,
                shutdown: false,
                wake: None,
                passed: false,
                story: Story::Confidential,
                private_failure: false,
            },
            wake: None,
            passed: false,
        },
        Memory::Checked,
    )
    .with_machine(FileSystem::absent_for_cuts());
    world.spawn_restartable(Vec::new(), issuer);
    world.spawn_restartable(vec![StartupRoot { name: bytes::copy_of(b"root"), path: bytes::copy_of(b"root") }], owner);
    world.spawn_restartable(Vec::new(), browser);
    world
}
#[test]
fn an_accounts_private_root_starting_absent_survives_every_acknowledged_power_loss_cut() {
    let mut uncut = world().run();
    let opened = uncut.procs[1].inner.opened_at.expect("private open answered");
    let stored = uncut.procs[1].inner.stored_at.expect("private store answered");
    let points: Vec<_> = uncut
        .trace
        .iter()
        .filter(|entry| entry.pid.raw() == 1)
        .filter_map(|entry| if let skein_sim::Event::Submit { op, .. } = &entry.event { Some(*op) } else { None })
        .enumerate()
        .filter(|(_, op)| skein_world::files::Driver::owns(*op))
        .map(|(before, _)| u32::try_from(before).expect("bounded submissions"))
        .collect();
    for before in points {
        {
            let (label, cut) = ("power loss", Cut::PowerLoss);
            let mut world = world();
            world.cut(1, before, cut);
            let mut outcome = world.run();
            assert_eq!(outcome.machine.cuts, 1, "{label}, before {before}");
            if before >= opened {
                assert!(outcome.machine.root_at_cut, "{label}, root acknowledged before {before}");
            }
            if before >= stored {
                let record = oauth::decode_record(
                    outcome.machine.record_at_cut.as_deref().expect("acknowledged record"),
                    &crate::world::limits().client.document,
                )
                .expect("whole acknowledged record");
                assert_eq!(record.generation, 1, "{label}, store before {before}");
                assert_eq!(record.access_token.expose(), b"new", "{label}, store before {before}");
            }
            outcome.machine.finish();
        }
    }
    uncut.machine.finish();
}
