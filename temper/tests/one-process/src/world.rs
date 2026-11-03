//! One real startup, native person, hosted agent and git, and loopback byte peers.
use crate::{
    fixtures,
    referee::{self, Capture, Store, Wire},
    stories,
};
use jig_client_protocol_link::Signing;
use skein_fake_peers::forge::{Delivery, Observation};
use skein_fake_person::{Doing, Face, Find, Found, Next, Step, TreeFace};
use skein_io::kernel::{Complete, Exit, Fd, Submit};
use skein_lib::{Duration, Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{Inherited, Memory};
use std::{cell::RefCell, net::SocketAddr, rc::Rc};
use temper_client_shell_terminal::client::{NativeClient, Settings as ClientSettings, Start};
use temper_playground::{
    Links,
    machine::{self, Machine, SignalHost},
    programs::{self, Harness, Parameters, Role},
    service::Observe,
};
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub seed: u64,
    pub config: skein_sim::Config,
    pub memory: Memory,
}
#[derive(Clone, Copy, Debug)]
pub enum Story {
    Chat { words: &'static [u8] },
    ChatReadThenWait { words: &'static [u8] },
    Adoption,
    Adopt,
    Merge,
    HeldChange,
    PlaygroundA,
}
#[derive(Debug)]
pub struct Outcome {
    pub verdict: skein_world::domain::Verdict,
    pub peaks: Option<Vec<u64>>,
    pub signed_in: u32,
    pub settings_revision: u32,
    pub chats_answered: u32,
    pub adopted: u32,
    pub pushed: Vec<Pushed>,
    pub deliveries: Vec<Delivery>,
    pub drained: bool,
    pub end: Option<temper_service::End>,
    pub restarts: u32,
    pub claims_settled_at_restart: u32,
    pub adoptions_answered: u32,
    pub adoption_pending_at_restart: bool,
    pub playground: Option<stories::playground::Outcome>,
}
#[derive(Clone, Copy)]
pub(crate) enum Phase {
    Story,
    SignIn,
    Drain,
    Restart,
    Refused,
    Fault { cut: skein_world::Cut, before: u32 },
}
#[derive(Default)]
struct Progress {
    setup_party: Option<u64>,
    at: usize,
    life: u32,
    signed: bool,
    sent: Vec<(u32, [u8; 16])>,
    answers: Vec<(u32, [u8; 16])>,
    press_had_frame: Vec<(u32, bool)>,
    cache: referee::client::cache::History,
    playground_answers: u32,
}
#[derive(Debug)]
pub struct Pushed {
    pub branch: Box<[u8]>,
    pub commit: u64,
    pub parents: Vec<u64>,
}
// Smaller than the fixture's stored stock policy, while its shape still decodes.
const REFUSED_POLICY_BYTES: u32 = 512;
thread_local! {
    static REFUSE_START: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static STORE: RefCell<Option<Rc<RefCell<Store>>>> = const { RefCell::new(None) };
    static PROGRESS: RefCell<Progress> = RefCell::new(Progress::default());
}
pub(crate) fn setup_signed(party: u64) {
    PROGRESS.with(|progress| progress.borrow_mut().setup_party = Some(party));
}
pub(crate) fn playground_answers() -> u32 {
    PROGRESS.with(|progress| progress.borrow().playground_answers)
}
pub(crate) fn playground_records() {
    STORE.with(|store| {
        let store = store.borrow();
        for row in store.as_ref().expect("world store").borrow().rows.values() {
            if let temper_engine_domain::Record::Forge { row } = row {
                eprintln!("story A durable forge: {row:?}");
            }
        }
    });
}
/// The one-process world's harness (temper/docs/design/playground.md, 3).
pub(crate) struct OneProcess;
pub(crate) type Proc = programs::Process<OneProcess>;
pub(crate) type ServiceHost = temper_playground::service::ServiceHost<OneProcess>;
impl Harness for OneProcess {
    type Service = ServiceObserver;
    type Process = HarnessProcess;
    fn github() -> bool {
        crate::lower::github::active()
    }
    fn configure(document: &mut serde_json::Value) {
        if REFUSE_START.with(std::cell::Cell::get) {
            document["engine"]["projects"] = serde_json::json!([]);
            document["application"]["webhooks"]["listener"] =
                serde_json::json!({"socket": crate::box_listener::ADDRESS.to_string()});
            // No seed policies on this start; its persisted policy exceeds the smaller read bound.
            document["engine"]["limits"]["overrides"]["core.policy_bytes"] = REFUSED_POLICY_BYTES.into();
        }
    }
    fn spawned(role: Role, inherited: &Inherited, base: &[u8]) {
        use crate::m15::observer::{self, Role as Observed};
        match role {
            Role::Service => {
                for (name, fd) in &inherited.roots {
                    observer::root(Observed::Service, *fd, name);
                }
            }
            Role::Agent => observer::process(Observed::Agent, inherited, base),
            Role::Git => observer::process(Observed::Git, inherited, base),
            Role::Shell => observer::process(Observed::Shell, inherited, base),
            Role::Provider | Role::Forge | Role::Visitor | Role::Other => {}
        }
    }
    fn complete(role: Role, complete: &Complete) {
        use crate::m15::observer::{self, Role as Observed};
        let role = match role {
            Role::Service => Observed::Service,
            Role::Agent => Observed::Agent,
            Role::Git => Observed::Git,
            Role::Shell => Observed::Shell,
            Role::Provider => Observed::Provider,
            Role::Forge => Observed::Forge,
            Role::Visitor | Role::Other => return,
        };
        observer::complete(role, complete);
    }
    fn git_limits() -> skein_fake_forge_protocol::Limits {
        crate::lower::github::protocol()
    }
    fn seeds(_one: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }
    fn agent_seed(_one: u64) -> u64 {
        fixtures::SEED
    }
    fn provider(_one: u64) -> skein_fake_peers::llm::Peer {
        fixtures::provider()
    }
    fn forge_seed(_one: u64) -> u64 {
        fixtures::SEED
    }
    fn files() -> temper_service::settings::Files {
        fixtures::files()
    }
    fn given(document: &serde_json::Value) -> temper_service::stock::Given {
        if stories::playground() {
            return temper_playground::documents::given(document);
        }
        stories::given(&serde_json::from_slice(&fixtures::document()).expect("deployment JSON"))
    }
}
/// Scope the harness's scenario documents and shared forge around its world.
pub(crate) fn parameters(machine: &Machine, forge: temper_fake_git_hosted::Typed) -> Parameters {
    Parameters {
        deployment: fixtures::document().into(),
        seed: fixtures::SEED,
        links: Links::Keep,
        forge,
        disk: machine.0.clone(),
    }
}
/// Captures, faults and assertions beside the composition's service.
pub(crate) struct ServiceObserver {
    pub(crate) hooks: Option<SocketAddr>,
    bound_hooks: Option<(Fd, SocketAddr)>,
    capture: Capture,
    store: Rc<RefCell<Store>>,
    pub(crate) commits: Vec<crate::referee::durable::Commit>,
    pub(crate) wire: Wire,
    pub(crate) attached: bool,
    pub(crate) cut_before: Option<u32>,
    pub(crate) submitted: u32,
    pub(crate) before_iteration: u32,
    in_flight: std::collections::BTreeSet<skein_lib::Token>,
    reserve: Option<crate::box_listener::Listener>,
    codec: jig_protocol_store::codec::View,
    families: temper_protocol_store::Bounds,
}
impl Observe for ServiceObserver {
    fn started(root: Fd, _inherited: &Inherited) -> Self {
        let mut capture = Capture::default();
        capture.root(root);
        let store = STORE.with(|store| store.borrow().as_ref().expect("world store").clone());
        Self {
            hooks: None,
            bound_hooks: None,
            capture,
            store: store.clone(),
            commits: Vec::new(),
            wire: Wire::new(store),
            attached: false,
            cut_before: None,
            submitted: 0,
            before_iteration: 0,
            in_flight: std::collections::BTreeSet::new(),
            reserve: REFUSE_START.with(std::cell::Cell::get).then(crate::box_listener::Listener::new),
            codec: if stories::playground() { stories::playground::codec() } else { fixtures::codec() },
            families: temper_protocol_store::bounds(
                &temper_service::limits::profile(temper_service::limits::Profile::Small).forge,
                jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.tasks.spec_bytes,
            )
            .expect("application codec bounds"),
        }
    }
    fn complete(&mut self, complete: Complete, service: &mut temper_service::Service) -> Option<Complete> {
        assert!(self.in_flight.remove(&complete.op), "completion has an operation in flight");
        if complete.op == crate::box_listener::OWNER {
            if let Some(fd) = self.reserve.as_mut().expect("box listener setup").complete(complete) {
                service.adopt_listener(fd, crate::box_listener::ADDRESS).expect("held webhook listener");
                self.reserve = None;
            }
            return None;
        }
        if let Some(bound) = hooks_address(&mut self.bound_hooks, &complete) {
            self.hooks = Some(bound);
        }
        self.capture.complete(&complete);
        self.wire.complete(&complete);
        Some(complete)
    }
    fn before_iterate(&mut self, service: &mut temper_service::Service, now: Time, _wall: Wall) -> bool {
        if stories::playground() {
            while let Some(note) = service.notes().pop() {
                eprintln!("story A at {} ns: service {note:?}", now.as_nanos());
            }
        }
        self.before_iteration = self.submitted;
        self.commits.extend(self.store.borrow_mut().observe(
            &mut self.capture.events,
            &self.codec,
            fixtures::store_names(),
            &self.families,
        ));
        if let Some(reserve) = &mut self.reserve {
            reserve.submit(service.submissions());
            true
        } else {
            false
        }
    }
    fn submitted(&mut self, submit: &Submit) {
        assert!(self.in_flight.insert(submit.op), "one operation per token");
        self.capture.pending(&submit.kind);
    }
    fn after_iterate(&mut self, service: &mut temper_service::Service, submitted: u32) {
        self.submitted += submitted;
        if self.cut_before.is_some_and(|before| self.before_iteration <= before && before < self.submitted) {
            referee::Referee::check_roots(service);
            self.cut_before = None;
        }
    }
    fn handover_read(
        &mut self,
        read: &temper_service::HandoverRead,
    ) -> Option<Result<Option<skein_lib::Secret>, temper_service::HandoverUnread>> {
        if stories::playground() {
            eprintln!("story A: handover read {}", String::from_utf8_lossy(&read.path));
            return None;
        }
        Some(Ok(stories::first_start().then(|| skein_lib::Secret::new(crate::restart_control::ADMIN_TOKEN.into()))))
    }
    fn drain(&mut self, service: &mut temper_service::Service, now: Time) {
        assert!(
            service.is_empty()
                || service.work_pending(now)
                || service.next_deadline().is_some()
                || !self.in_flight.is_empty(),
            "service would wait forever with nothing in flight: end={:?}; state={:?}; pending={:?}",
            service.end(),
            service.wait_state(),
            service.pending(now)
        );
    }
    fn work_pending(&self, _now: Time) -> Option<bool> {
        self.reserve.as_ref().map(crate::box_listener::Listener::ready)
    }
}
pub(crate) enum HarnessProcess {
    Ctl(Box<crate::ctl::Recorded>),
    Calls(Box<crate::ctl::Caller>),
    Client(Box<SignalHost<Person>>),
}
impl HarnessProcess {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Ctl(h) => h.as_ref(),
            Self::Calls(h) => h.as_ref(),
            Self::Client(h) => h.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Ctl(h) => h.as_mut(),
            Self::Calls(h) => h.as_mut(),
            Self::Client(h) => h.as_mut(),
        }
    }
}
impl Host for HarnessProcess {
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        if crate::m15::observer::enabled() {
            use crate::m15::observer::{self, Role};
            let role = match self {
                Self::Ctl(_) => Role::Ctl,
                Self::Calls(_) => Role::Caller,
                Self::Client(_) => Role::Client,
            };
            for complete in self.host_mut().completions().iter() {
                observer::complete(role, complete);
            }
        }
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
pub(crate) struct Person {
    pub(crate) wire: referee::client::Wire,
    cache: referee::client::cache::Cache,
    client: Option<NativeClient>,
    root: Option<Fd>,
    actor: skein_fake_person::Person<temper_client_domain::Address>,
    provision: Option<skein_fake_person::Person<temper_client_domain::Address>>,
    wake: Option<Time>,
    pub(crate) signed_in: u32,
    pub(crate) finished: bool,
    project_selected: bool,
    setup: Option<crate::ctl::Caller>,
    printed_link: Box<[u8]>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    offset: usize,
    pub(crate) life: u32,
    cutting: bool,
    playground_replies: Vec<u32>,
}
const CHAT_WITHIN: Duration = Duration::from_secs(5);
impl Person {
    pub(crate) fn pages(root: Fd, steps: Box<[Step<temper_client_domain::Address>]>) -> Self {
        let mut person = Self::new(root, Story::Chat { words: b"" });
        person.actor = skein_fake_person::Person::new(steps);
        person.provision = None;
        person.wire = referee::client::Wire::capturing();
        crate::m15::observer::root(crate::m15::observer::Role::Client, root, b"client");
        person
    }
    pub(crate) fn saved(&mut self) {
        self.client = Some(
            NativeClient::new(
                ClientSettings {
                    engine: skein_io::Addr::Ip(fixtures::CLIENTS),
                    directory: b"private".as_slice().into(),
                    limits: stories::client_limits(),
                },
                self.root.take().expect("saved client root"),
                1000,
                fixtures::SEED,
                Start::Token,
            )
            .expect("saved native client"),
        );
    }
    fn new(root: Fd, scenario: Story) -> Self {
        let (playground_steps, playground_replies) = if matches!(scenario, Story::PlaygroundA) {
            stories::playground::person()
        } else {
            (Vec::new(), Vec::new())
        };
        let steps = match scenario {
            Story::Chat { words } | Story::ChatReadThenWait { words } => vec![
                Step::Go {
                    address: temper_client_domain::Address::Chats {
                        filter: temper_client_domain::ChatsFilter::All,
                        start: false,
                    },
                },
                Step::See { find: Find::named(skein_ui::Role::TextArea, b"Start a chat"), within: CHAT_WITHIN },
                Step::Type { find: Find::named(skein_ui::Role::TextArea, b"Start a chat"), words: words.into() },
                Step::Press { find: Find::named(skein_ui::Role::Button, b"Start") },
                Step::See { find: Find::text(fixtures::ANSWER), within: CHAT_WITHIN },
            ],
            Story::Adoption => stories::adoption_person(),
            Story::Adopt | Story::Merge => stories::person(),
            Story::HeldChange if stories::first_start() => stories::person(),
            Story::HeldChange => Vec::new(),
            Story::PlaygroundA => playground_steps,
        };
        let mut offset = PROGRESS.with(|progress| progress.borrow().at);
        if matches!(scenario, Story::Adoption) && offset < fixtures::ADOPTION_REPLY_STEP {
            offset = 0;
        }
        let mut remaining: Vec<_> = steps.into_iter().skip(offset).collect();
        if matches!(scenario, Story::Adoption) && offset >= fixtures::ADOPTION_REPLY_STEP {
            remaining.clear();
        }
        let life = PROGRESS.with(|progress| {
            progress.borrow_mut().life += 1;
            progress.borrow().life
        });
        Self {
            wire: referee::client::Wire::default(),
            cache: referee::client::cache::Cache::new(
                root,
                life,
                PROGRESS.with(|progress| progress.borrow().cache.clone()),
            ),
            client: None,
            root: Some(root),
            actor: skein_fake_person::Person::new(remaining.into_boxed_slice()),
            provision: (offset == 0 && !matches!(scenario, Story::PlaygroundA))
                .then(|| skein_fake_person::Person::new(fixtures::account_steps())),
            wake: None,
            signed_in: 0,
            finished: false,
            project_selected: matches!(scenario, Story::PlaygroundA),
            setup: None,
            printed_link: Box::new([]),
            completions: Queue::with_capacity(512),
            submissions: Queue::with_capacity(512),
            offset,
            life,
            cutting: matches!(scenario, Story::Adoption),
            playground_replies,
        }
    }
    pub(crate) fn sign_in(&mut self, link: &[u8]) {
        assert!(self.client.is_none(), "one printed link");
        self.printed_link = link.into();
        if stories::needs_settings() || stories::held_change() || stories::first_start() {
            self.setup = Some(crate::ctl::Caller::new(self.root.expect("client parent"), link));
            return;
        }
        self.client = Some(
            NativeClient::new(
                ClientSettings {
                    engine: skein_io::Addr::Ip(fixtures::CLIENTS),
                    directory: b"private".as_slice().into(),
                    limits: stories::client_limits(),
                },
                self.root.take().expect("private root"),
                1000,
                fixtures::SEED,
                Start::Link(link.into()),
            )
            .expect("service's printed link"),
        );
    }
    pub(crate) fn finish(&mut self) {
        self.finished = true;
        if let Some(client) = &mut self.client {
            client.close();
        }
    }
}
// Select the story's project and provision its real account before its own acts.
/// The banner's button for a project `me` leaves unnamed, "Project N"
/// (client/README.md, 5.1).
fn an_unnamed_project(name: Option<&[u8]>) -> bool {
    let Some(number) = name.and_then(|name| name.strip_prefix(b"Project ".as_slice())) else { return false };
    !number.is_empty() && number.iter().all(u8::is_ascii_digit)
}
fn prepare_account(
    client: &mut NativeClient,
    selected: &mut bool,
    provision: &mut Option<skein_fake_person::Person<temper_client_domain::Address>>,
    wake: &mut Option<Time>,
    now: Time,
) -> bool {
    if !*selected
        && let Some(node) = client
            .tree()
            .nodes()
            .iter()
            .find(|node| node.role == skein_ui::Role::Button && an_unnamed_project(node.name.as_deref()))
            .map(|node| (node.id, node.states.iter().any(|state| *state == skein_ui::State::Pressed)))
    {
        let (node, pressed) = node;
        if !pressed {
            assert!(client.act(node, skein_ui::Act::Press), "offered project selection");
        }
        *selected = true;
    }
    let Some(actor) = provision else { return false };
    match actor.next(&mut TreeFace::new(client.tree()), now) {
        Next::Do(Doing::Act { node, act }) => assert!(client.act(node, act), "offered account setup act"),
        Next::Do(Doing::Go { address }) => client.go(address),
        Next::Do(Doing::Reload) => panic!("account setup has no reload"),
        Next::Wait { until } => *wake = until,
        Next::Done => {
            *provision = None;
            *wake = Some(now);
        }
        Next::Failed { step, why } => panic!("account setup step {step}: {why:?}"),
    }
    true
}
impl Person {
    fn prepare_client(&mut self, now: Time, wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            if crate::ctl::Caller::owns(complete.op) {
                self.setup.as_mut().expect("ctl setup").completions().push(complete);
                continue;
            }
            let client = self.client.as_mut().expect("native completion");
            self.cache.complete(&complete);
            let received = self.wire.received.len();
            self.wire.complete(&complete);
            for event in &self.wire.received[received..] {
                if let referee::client::Received::ApplicationSent { key } = event {
                    PROGRESS.with(|progress| progress.borrow_mut().sent.push((self.life, *key)));
                } else if let referee::client::Received::Application { key, .. } = event {
                    PROGRESS.with(|progress| progress.borrow_mut().answers.push((self.life, *key)));
                }
            }
            client.completions().push(complete);
        }
        if let Some(setup) = &mut self.setup {
            setup.iterate(now, wall);
            while let Some(submit) = setup.submissions().pop() {
                self.submissions.push(submit);
            }
            if setup.is_empty() {
                self.setup = None;
                let party = PROGRESS.with(|progress| progress.borrow().setup_party.expect("observed CLI sign-in"));
                self.wire.received.push(referee::client::Received::SignedIn(party));
                self.signed_in = 1;
                PROGRESS.with(|progress| progress.borrow_mut().signed = true);
                self.client = Some(
                    NativeClient::new(
                        ClientSettings {
                            engine: skein_io::Addr::Ip(fixtures::CLIENTS),
                            directory: b"private".as_slice().into(),
                            limits: stories::client_limits(),
                        },
                        self.root.take().expect("private parent"),
                        1000,
                        fixtures::SEED,
                        Start::Token,
                    )
                    .expect("saved sign-in"),
                );
            }
        }
    }
}
impl Host for Person {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.prepare_client(now, wall);
        let Some(client) = &mut self.client else { return };
        client.iterate(now, wall);
        if crate::m15::observer::enabled() {
            crate::m15::observer::page(format!("{:?}", client.tree().nodes()).as_bytes());
        }
        while let Some(signing) = client.signing() {
            if matches!(signing, Signing::Again) {
                client.sign_in(&self.printed_link).expect("retry the printed link while startup restores settings");
                continue;
            }
            assert!(matches!(signing, Signing::SignedIn { .. }), "sign-in failed: {signing:?}");
            self.signed_in += 1;
            PROGRESS.with(|progress| progress.borrow_mut().signed = true);
        }
        while let Some(notice) = client.notices() {
            if self.cutting
                && matches!(
                    notice,
                    temper_client_shell_terminal::client::Shown::Notice(
                        temper_client_shell_terminal::route::Notice::Fact(
                            jig_client_protocol_link::Fact::StreamDropped
                        )
                    )
                )
            {
                continue;
            }
            assert!(
                !matches!(
                    notice,
                    temper_client_shell_terminal::client::Shown::Notice(
                        temper_client_shell_terminal::route::Notice::Fact(_)
                    )
                ),
                "client protocol fact: {notice:?}"
            );
        }
        self.wake = None;
        let preparing = client.started()
            && !self.finished
            && prepare_account(client, &mut self.project_selected, &mut self.provision, &mut self.wake, now);
        if client.started() && self.project_selected && !self.finished && !preparing {
            let before = self.actor.at();
            let mut face = TreeFace::new(client.tree());
            let next = if self.cutting
                && self.life > 1
                && !self
                    .wire
                    .received
                    .iter()
                    .any(|event| matches!(event, referee::client::Received::Application { .. }))
            {
                Next::Wait { until: None }
            } else if self.cutting && self.offset + before as usize + 1 == fixtures::ADOPTION_REPLY_STEP {
                match face.find(&Find::exactly_named(skein_ui::Role::Button, b"Adopt")) {
                    Found::Node(node) if face.allows(node, &skein_ui::Act::Press) => self.actor.next(&mut face, now),
                    Found::Node(_) | Found::Absent | Found::Pending => Next::Wait { until: None },
                }
            } else {
                self.actor.next(&mut face, now)
            };
            match next {
                Next::Do(Doing::Act { node, act }) => {
                    if self.cutting && self.offset + before as usize + 1 == fixtures::ADOPTION_REPLY_STEP {
                        let frame = self.wire.received.iter().any(|event| {
                            matches!(
                                event,
                                referee::client::Received::Delivery(jig_client::contract::Delivery::Me(_))
                                    | referee::client::Received::Read(jig_client::contract::ReadValue::Me(_))
                            )
                        });
                        PROGRESS.with(|progress| progress.borrow_mut().press_had_frame.push((self.life, frame)));
                    }
                    assert!(client.act(node, act), "offered person act");
                }
                Next::Do(Doing::Go { address }) => client.go(address),
                Next::Do(Doing::Reload) => panic!("chat script has no reload"),
                Next::Wait { until } => self.wake = until,
                Next::Done => {
                    self.finished = true;
                    client.close();
                }
                Next::Failed { step, why } => {
                    stories::playground::failed(now, step, &why, client.tree());
                    person_failed(step, &why, &self.wire, self.life)
                }
            }
            if self.actor.at() != before {
                playground_progress(now, before, &self.playground_replies);
                PROGRESS.with(|progress| {
                    progress.borrow_mut().at = self.offset + usize::try_from(self.actor.at()).expect("person step");
                });
                self.wake = Some(now);
            }
        }
        while let Some(submit) = client.submissions().pop() {
            self.submissions.push(submit);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty()
            || self.wake.is_some_and(|at| at <= now)
            || self.client.as_ref().is_some_and(|client| client.work_pending(now))
            || self.setup.as_ref().is_some_and(|setup| setup.work_pending(now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        [self.client.as_ref().and_then(Host::next_mechanism_deadline), self.wake].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [self.client.as_ref().and_then(Host::next_policy_deadline), self.wake].into_iter().flatten().min()
    }
    fn is_empty(&self) -> bool {
        self.finished && self.client.as_ref().is_some_and(Host::is_empty)
    }
    fn operations(&self) -> u32 {
        512
    }
}
fn playground_progress(now: Time, before: u32, replies: &[u32]) {
    if stories::playground() {
        eprintln!("story A at {} ns: person step {before} complete", now.as_nanos());
        if replies.contains(&before) {
            PROGRESS.with(|progress| progress.borrow_mut().playground_answers += 1);
        }
    }
}
fn person_failed(step: u32, why: &skein_fake_person::Why, wire: &referee::client::Wire, life: u32) -> ! {
    let sent: Vec<_> = wire
        .received
        .iter()
        .filter_map(|event| match event {
            referee::client::Received::ApplicationSent { key } => Some(*key),
            referee::client::Received::SignedIn(_)
            | referee::client::Received::Answer { .. }
            | referee::client::Received::Application { .. }
            | referee::client::Received::Delivery(_)
            | referee::client::Received::Read(_) => None,
        })
        .collect();
    let answered: Vec<_> = wire
        .received
        .iter()
        .filter_map(|event| match event {
            referee::client::Received::Application { key, .. } => Some(*key),
            referee::client::Received::SignedIn(_)
            | referee::client::Received::Answer { .. }
            | referee::client::Received::ApplicationSent { .. }
            | referee::client::Received::Delivery(_)
            | referee::client::Received::Read(_) => None,
        })
        .collect();
    let durable = STORE.with(|store| store.borrow().as_ref().expect("world store").borrow().rows.values().filter(|row| matches!(row, temper_engine_domain::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Application { .. }))).count());
    PROGRESS.with(|progress| {
        let progress = progress.borrow();
        let before: Vec<_> = progress.sent.iter().filter(|(sent_life, _)| *sent_life < life).collect();
        panic!("person step {step}: {why:?}; adoption keys sent {sent:?}, answered {answered:?}, durable outcomes {durable}; pre-cut sent {before:?}; Adopt presses had frame {:?}; private cache {:?}", progress.press_had_frame, progress.cache.borrow());
    })
}
#[must_use]
pub fn run(settings: Settings, scenario: Story) -> Outcome {
    run_phase(
        settings,
        scenario,
        Phase::Story,
        machine::fresh(),
        Store::new(),
        Rc::new(RefCell::new(referee::Run::default())),
    )
    .0
}
#[must_use]
pub fn drain_mid_run(settings: Settings, restart: bool) -> Outcome {
    let run = Rc::new(RefCell::new(referee::Run::default()));
    let (mut outcome, disk, store) = run_phase(
        settings,
        Story::ChatReadThenWait { words: b"Hello" },
        Phase::Drain,
        machine::fresh(),
        Store::new(),
        run.clone(),
    );
    if restart {
        let (next, _, _) =
            run_phase(settings, Story::ChatReadThenWait { words: b"Hello" }, Phase::Restart, disk, store, run);
        assert_eq!(outcome.settings_revision, 2, "one hosted ctl settings change before the signal");
        assert_eq!(next.settings_revision, outcome.settings_revision, "the changed settings survive restart");
        outcome.restarts = next.restarts;
        outcome.claims_settled_at_restart = next.claims_settled_at_restart;
        outcome.drained &= next.drained;
    }
    outcome
}
#[must_use]
pub fn crash(settings: Settings, cut: skein_world::Cut, draw: &mut skein_lib::Rng) -> (u32, Outcome) {
    // The pinned world arms cuts before running. Observe the deterministic
    // pending window on fresh disk, then replay that scenario with its cut.
    let setup = Rc::new(RefCell::new(referee::Run::default()));
    run_phase(settings, Story::Adoption, Phase::Story, machine::fresh(), Store::new(), setup.clone());
    let (start, end) = setup.borrow().window.expect("saved pending adoption window");
    let length = end.checked_sub(start).expect("pending window ordering");
    assert!(length > 0, "pending window has a submission");
    let before = start + u32::try_from(draw.below(u64::from(length))).expect("bounded pending draw");
    crate::oracle::begin();
    let run = Rc::new(RefCell::new(referee::Run::default()));
    let (outcome, disk, store) =
        run_phase(settings, Story::Adoption, Phase::Fault { cut, before }, machine::fresh(), Store::new(), run.clone());
    crate::oracle::cold(settings, disk, store, &run);
    (before, outcome)
}
pub(crate) fn saved_adoptions() -> std::collections::BTreeSet<[u8; 16]> {
    PROGRESS.with(|progress| referee::client::cache::saved(&progress.borrow().cache))
}
pub fn require_pending_at_cut(outcome: &Outcome, seed: u64, before: u32) {
    if !outcome.adoption_pending_at_restart {
        PROGRESS.with(|progress| {
            let progress = progress.borrow();
            panic!("seed {seed}, before {before}: pending adoption at cut; original sent {:?}; answers {:?}; frames {:?}; private cache {:?}", progress.sent, progress.answers, progress.press_had_frame, progress.cache.borrow());
        });
    }
}
fn person(inherited: &Inherited) -> Proc {
    let root = inherited.roots.first().expect("client root").1;
    let signed = PROGRESS.with(|progress| progress.borrow().signed);
    let mut person = Person::new(root, Story::Adoption);
    if signed {
        person.client = Some(
            NativeClient::new(
                ClientSettings {
                    engine: skein_io::Addr::Ip(fixtures::CLIENTS),
                    directory: b"private".as_slice().into(),
                    limits: stories::client_limits(),
                },
                person.root.take().expect("private root"),
                1000,
                fixtures::SEED,
                Start::Token,
            )
            .expect("stored sign-in"),
        );
        person.signed_in = 1;
    }
    Proc::Other(HarnessProcess::Client(Box::new(SignalHost::new(person, Some(inherited.signal)))))
}
fn hooks_address(bound: &mut Option<(Fd, SocketAddr)>, complete: &Complete) -> Option<SocketAddr> {
    use skein_io::kernel::{Done, Op};
    match (&complete.kind, &complete.result) {
        (Op::Bind { fd, addr: asked }, Ok(Done::Bound(addr))) if asked.port() == 0 => {
            *bound = Some((*fd, *addr));
            None
        }
        (Op::Listen { fd, .. }, Ok(_)) if bound.is_some_and(|(at, _)| at == *fd) => bound.take().map(|(_, addr)| addr),
        _ => None,
    }
}
fn deliveries(procs: &[Proc]) -> Vec<Delivery> {
    procs
        .iter()
        .filter_map(|proc| if let Proc::Forge(peer) = proc { Some(peer.observations()) } else { None })
        .flatten()
        .filter_map(|observation| {
            if let Observation::Delivery { outcome, .. } = observation {
                Some(match outcome {
                    Delivery::Written => Delivery::Written,
                    Delivery::NoAddress => Delivery::NoAddress,
                    Delivery::NoRoom => Delivery::NoRoom,
                    Delivery::Refused => Delivery::Refused,
                    Delivery::Reset => Delivery::Reset,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Two native starts on one disk, checking the held sign-in's stored identity and credential.
#[must_use]
pub fn sign_in_again(settings: Settings) -> [Outcome; 2] {
    let scenario = Story::Chat { words: b"Hello" };
    let run = Rc::new(RefCell::new(referee::Run::default()));
    let (first, mut disk, store) =
        run_phase(settings, scenario, Phase::SignIn, machine::fresh(), Store::new(), run.clone());
    let before = sign_ins(&store);
    assert_eq!(before.len(), 1, "one live sign-in before the fresh link");
    let token = token_bytes(&mut disk);
    assert_eq!(token.len(), 40, "the native bearer is saved on disk");
    let (again, mut disk, store) = run_phase(settings, scenario, Phase::SignIn, disk, store, run);
    assert_eq!(token_bytes(&mut disk), token, "same directory keeps every token byte");
    assert_eq!(sign_ins(&store), before, "same sign-in number and expiry, pool unchanged");
    [first, again]
}

#[expect(clippy::wildcard_enum_match_arm, reason = "the snapshot selects only durable sign-in rows")]
fn sign_ins(store: &Store) -> Vec<(u64, u64, Wall)> {
    store
        .rows
        .values()
        .filter_map(|row| match row {
            temper_engine_domain::Record::Core(jig_domain_core::Record::People(
                jig_domain_people::Stored::SignIn { number, person, expires },
            )) => Some((*number, *person, *expires)),
            _ => None,
        })
        .collect()
}

fn token_bytes(disk: &mut skein_fake_machine::Machine) -> Vec<u8> {
    let root = disk.reopen_root(2);
    let file = disk.open(root, b"private/token", skein_fake_machine::How::ReadNoFollow).expect("native token file");
    let size = u32::try_from(disk.stat(file).size).expect("bounded token");
    let bytes = disk.read(file, 0, size).expect("read native token");
    disk.close(file);
    disk.close(root);
    bytes
}

fn write_settings(disk: &mut skein_fake_machine::Machine) {
    if stories::needs_settings() || stories::first_start() {
        let root = disk.lay(&[
            skein_fake_machine::Item::file(b"settings.json", &stories::settings_file()),
            skein_fake_machine::Item::file(b"admin.token", crate::restart_control::ADMIN_TOKEN).mode(0o600),
        ]);
        disk.close(root);
    }
}

fn settings_revision(records: &Store) -> u32 {
    records
        .rows
        .values()
        .filter_map(|row| {
            if let temper_engine_domain::Record::Core(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Settings(jig_domain_core::settings::Record::Revision {
                    history: false,
                    revision,
                }),
            )) = row
            {
                Some(revision.head.number)
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0)
}

fn forge_for_run(run: &Rc<RefCell<referee::Run>>, scenario: Story, phase: Phase) -> temper_fake_git_hosted::Typed {
    let transport = run.borrow().forge.clone().unwrap_or_else(|| stories::forge(scenario));
    if matches!(scenario, Story::HeldChange) || matches!(phase, Phase::Fault { .. }) {
        run.borrow_mut().forge = Some(transport.clone());
    }
    transport
}
/// Restore a persisted policy past a smaller read bound, with the box's held listener.
#[must_use]
pub fn refused_restore(settings: Settings) -> Outcome {
    let scenario = Story::Chat { words: b"Hello" };
    let run = Rc::new(RefCell::new(referee::Run::default()));
    let (_, disk, store) = run_phase(settings, scenario, Phase::SignIn, machine::fresh(), Store::new(), run.clone());
    run_phase(settings, scenario, Phase::Refused, disk, store, run).0
}

fn ended(procs: &[Proc], phase: Phase) -> Option<temper_service::End> {
    assert!(procs.iter().all(Host::is_empty), "all processes drain");
    for proc in procs {
        if let Proc::Service(service) = proc {
            if matches!(phase, Phase::Refused) {
                let pending = service.service.pending(Time::ZERO);
                assert!(!pending.engine.expect("engine").release_pending, "an ended engine owes no release");
                assert!(pending.worker.expect("in-process worker").aborted);
            }
            return service.service.end().cloned();
        }
    }
    None
}

#[expect(clippy::too_many_lines, reason = "the scenario owns its registry, restartable processes and outcome together")]
fn run_phase(
    settings: Settings,
    scenario: Story,
    phase: Phase,
    mut disk: skein_fake_machine::Machine,
    previous: Store,
    run: Rc<RefCell<referee::Run>>,
) -> (Outcome, skein_fake_machine::Machine, Store) {
    stories::set(scenario);
    REFUSE_START.with(|refused| refused.set(matches!(phase, Phase::Refused)));
    stories::setup_settings(phase);
    write_settings(&mut disk);
    PROGRESS.with(|progress| *progress.borrow_mut() = Progress::default());
    let client = matches!(phase, Phase::Story | Phase::SignIn | Phase::Drain).then(|| disk.reopen_root(2));
    let store = Rc::new(RefCell::new(previous));
    STORE.with(|cell| {
        *cell.borrow_mut() = Some(store.clone());
    });
    let start = matches!(scenario, Story::PlaygroundA).then(|| {
        let mut start = temper_playground::stories::A.start(fixtures::SEED, settings.config.wall);
        start.links = Links::Keep;
        machine::setup(&mut disk);
        start
    });
    let transport =
        start.as_ref().map_or_else(|| forge_for_run(&run, scenario, phase), |start| start.fakes.forge.clone());
    let observed_forge = Rc::new(RefCell::new(referee::forge::Forge::new(transport.clone(), scenario)));

    let machine = Machine::new(disk);
    let mut parameters = parameters(&machine, transport.clone());
    let peers = start.map(|start| {
        parameters.deployment = start.deployment;
        parameters.seed = start.seed;
        parameters.links = start.links;
        (start.fakes.llm, start.fakes.peer)
    });
    let outcome = programs::with(parameters, || {
        let mut world =
            skein_world::World::new_controlled(settings.seed, settings.config, settings.memory, |controls| {
                referee::Referee::new(controls, store.clone(), scenario, observed_forge.clone(), run.clone(), phase)
            })
            .with_machine(machine);
        for (program, roots) in programs::programs::<OneProcess>() {
            world.host_roots(program, roots);
        }
        let limits = temper_client_shell_cli::limits::native();
        world.host_roots(
            skein_world::HostedProgram {
                program: crate::ctl::PROGRAM.into(),
                make: |spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited)))),
                instances: 1,
                operations: skein_io::operations(&limits.io).expect("ctl operations") * 2 + 5,
            },
            crate::ctl::roots,
        );
        world.spawn_restartable(programs::service_roots(), programs::service::<OneProcess>);
        match phase {
            Phase::Fault { cut, before } => {
                world.spawn_restartable(
                    vec![skein_world::StartupRoot {
                        name: b"private".as_slice().into(),
                        path: b"client".as_slice().into(),
                    }],
                    person,
                );
                world.spawn_restartable(vec![], programs::llm::<OneProcess>);
                world.spawn_restartable(vec![], programs::forge::<OneProcess>);
                crate::oracle::spawn(&mut world);
                world.cut(0, before, cut);
            }
            Phase::Restart | Phase::Refused => {
                world.spawn(|| Proc::Peer(Box::new(SignalHost::new(stories::provider(scenario), None))));
                world.spawn(|| {
                    Proc::Forge(Box::new(SignalHost::new(
                        temper_playground::fakes::forge_peer(&transport, fixtures::SEED),
                        None,
                    )))
                });
            }
            Phase::Story | Phase::SignIn | Phase::Drain => {
                world.spawn_root(skein_sim::Handle::new(client.expect("story client root").raw()), |root| {
                    Proc::Other(HarnessProcess::Client(Box::new(SignalHost::new(Person::new(root, scenario), None))))
                });
                let (llm, forge) = peers.unwrap_or_else(|| {
                    (stories::provider(scenario), temper_playground::fakes::forge_peer(&transport, fixtures::SEED))
                });
                world.spawn(|| Proc::Peer(Box::new(SignalHost::new(llm, None))));
                world.spawn(|| Proc::Forge(Box::new(SignalHost::new(forge, None))));
            }
        }
        world.run()
    });
    if matches!(scenario, Story::Adopt) {
        observed_forge.borrow().require_hooks(stories::ADOPT_HOOKS);
    }
    let signed_in = outcome
        .procs
        .iter()
        .filter_map(|proc| match proc {
            Proc::Other(HarnessProcess::Client(person)) => Some(person.signed_in),
            Proc::Service(_)
            | Proc::Agent(_)
            | Proc::GitRoute(_)
            | Proc::Git(_)
            | Proc::Shell(_)
            | Proc::Peer(_)
            | Proc::Forge(_)
            | Proc::Visitor(_)
            | Proc::Other(HarnessProcess::Ctl(_) | HarnessProcess::Calls(_)) => None,
        })
        .sum();
    let chats_answered = if matches!(scenario, Story::ChatReadThenWait { .. }) {
        u32::from(referee::chat_waited(&store.borrow()))
    } else {
        referee::terminals(&store.borrow())
    };
    let adopted = stories::adopted(&store.borrow().rows);
    if matches!(phase, Phase::Fault { .. }) {
        referee::require_replayed(&run.borrow(), &store.borrow());
    }
    let pushed = std::mem::take(&mut observed_forge.borrow_mut().pushed);
    let deliveries = deliveries(&outcome.procs);
    let end = ended(&outcome.procs, phase);

    STORE.with(|cell| {
        *cell.borrow_mut() = None;
    });
    let records = std::mem::take(&mut *store.borrow_mut());
    let result = Outcome {
        peaks: outcome.peaks(),
        verdict: skein_world::domain::Verdict::Passed,
        signed_in,
        settings_revision: settings_revision(&records),
        chats_answered,
        adopted,
        pushed,
        deliveries,
        drained: run.borrow().drained,
        end,
        restarts: run.borrow().restarts,
        claims_settled_at_restart: run.borrow().settled,
        adoptions_answered: u32::try_from(run.borrow().answers.len()).expect("adoption answers"),
        adoption_pending_at_restart: run.borrow().pending_adoption,
        playground: matches!(scenario, Story::PlaygroundA).then(|| stories::playground::outcome(&transport)),
    };
    drop(run);
    (result, std::mem::replace(&mut outcome.machine.0.borrow_mut(), machine::fresh()), records)
}

#[cfg(test)]
mod tests;

pub(crate) fn lower_store(store: Option<Rc<RefCell<Store>>>) {
    STORE.with(|cell| *cell.borrow_mut() = store);
}
