//! Composed plan accounts over the simulated service (jig/docs/design/protocol/service.md, 8).
use crate::world::{self, Proc, ServiceHost};
use crate::world_signals::SignalHost;
use jig_client::{Asker, Handle, Key, contract as c};
use jig_client_protocol_world::native::{NativeClient, Received, Script};
use jig_protocol_accounts_world::{fixtures, world::Story as AccountStory};
use skein_fake_peers::{Transport, llm, oauth};
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{Duration, Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{Inherited, Memory, StartupRoot};

// The process stories use this same browser source, without adding a second visitor.
#[expect(dead_code, reason = "these stories follow redirects rather than paste them")]
#[path = "../../protocol-accounts/src/visitor.rs"]
mod visitor;

const ASK: Asker = Asker(88);
const SIGN_IN: Key = Key { asker: ASK, number: 1 };
const SIGN_OUT: Key = Key { asker: ASK, number: 2 };
const ACCOUNTS: Handle = Handle { asker: ASK, number: 1 };
const WAIT: Duration = Duration::from_secs(20);
const WORDS: &[u8] = b"Hello";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Story {
    SignIn,
    Refused,
    Restart,
}
#[derive(Debug)]
pub struct Outcome {
    pub chats: u32,
    pub credentials: Vec<Option<u32>>,
    pub posts: usize,
    pub revokes: usize,
    pub restarts: u32,
    pub accounts_empty: bool,
    pub confirmed: bool,
    pub run_failure: Option<jig_domain_tasks::Class>,
    pub provider_failure: Option<smith_channel::RunResult>,
    pub queries: Vec<skein_fake_llm_domain::api::Query>,
}

enum Process {
    App { host: Box<ServiceHost>, spare: Option<(Fd, Fd)>, restarts: u32 },
    Other(Proc),
    Issuer(Box<oauth::Peer>),
    Visitor(Box<visitor::Visitor>),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::App { host, .. } => host.as_ref(),
            Self::Other(host) => host,
            Self::Issuer(host) => host.as_ref(),
            Self::Visitor(host) => host.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::App { host, .. } => host.as_mut(),
            Self::Other(host) => host,
            Self::Issuer(host) => host.as_mut(),
            Self::Visitor(host) => host.as_mut(),
        }
    }
}
impl Host for Process {
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
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
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
    fn is_empty(&self) -> bool {
        !matches!(self, Self::App { spare: Some(_), .. }) && self.host().is_empty()
    }
}
fn document() -> Vec<u8> {
    let mut document: serde_json::Value =
        serde_json::from_slice(&crate::fixtures::document(crate::fixtures::Chat::Finite)).expect("one-process fixture");
    document["engine"]["llm"]["providers"][0] = serde_json::json!({
        "name":"fake-provider", "number":1, "models":["fake"], "signed_in_plan":true, "manage_usage":"https://issuer.test/usage",
        "sources":{"sign_in":{
            "issuer":"http://127.0.0.1:31000", "client":"client", "scopes":["openid","read"],
            "name_hint":"jig-test", "authorization":"http://127.0.0.1:31000/authorize",
            "token":"http://127.0.0.1:31000/token", "revocation":"http://127.0.0.1:31000/revoke",
            "keys":"http://127.0.0.1:31000/keys", "transport":"plaintext",
            "must_grant":["openid","read"], "resource":"api", "redirect_path":"callback"
        }}
    });
    serde_json::to_vec(&document).expect("account document")
}
fn runtime() -> jig_test_service::AccountsRuntime {
    let endpoint = || skein_oauth_accounts::Endpoint {
        address: "127.0.0.1:31000".parse().expect("issuer address"),
        transport: skein_oauth_accounts::Transport::Plaintext,
    };
    let mut origins = skein_lib::List::with_capacity(1);
    origins
        .push(skein_oauth_accounts::Origin { uri: b"http://127.0.0.1:31000".as_slice().into(), endpoint: endpoint() })
        .expect("issuer origin");
    let mut providers = skein_lib::List::with_capacity(1);
    providers.push((1, endpoint(), origins)).expect("provider runtime");
    jig_test_service::AccountsRuntime { providers, trust_roots: 0, trust_root_bytes: 0 }
}
fn service(inherited: &Inherited) -> Process {
    let root = |name: &[u8]| inherited.roots.iter().find(|(n, _)| n.as_ref() == name).expect("startup root").1;
    Process::App {
        host: Box::new(world::start_service(
            root(b"state"),
            root(b"worker"),
            Some(inherited.signal),
            &document(),
            Some(runtime()),
        )),
        spare: None,
        restarts: 0,
    }
}
fn restarting_service(inherited: &Inherited) -> Process {
    let mut process = service(inherited);
    let Process::App { spare, .. } = &mut process else { unreachable!() };
    *spare = Some((
        inherited.roots.iter().find(|(name, _)| name.as_ref() == b"state-again").expect("restart state root").1,
        inherited.roots.iter().find(|(name, _)| name.as_ref() == b"worker-again").expect("restart worker root").1,
    ));
    process
}
fn agent(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Process {
    Process::Other(world::agent(spawn, inherited))
}
fn issuer(story: Story) -> oauth::Peer {
    let mut plans = fixtures::issuer_plans(AccountStory::AccountSignInVaryingPort);
    if story == Story::Restart {
        let skein_fake_oauth::Body::Token(token) = &mut plans[0].body else { panic!("code grant") };
        token.access_token = fixtures::TOKEN_FIRST.into();
        plans.insert(1, fixtures::rotate());
    }
    let profile = fixtures::limits();
    let limits = skein_fake_oauth::Limits {
        document: profile.client.document,
        uri_bytes: 256,
        request_bytes: 1024,
        codes: 1,
        rotations: 2,
        carried: 5,
        plans: u32::try_from(plans.len()).expect("plans fit"),
    };
    let response = oauth::least_response(&limits).expect("issuer response");
    let mut peer = oauth::Peer::new(
        "127.0.0.1:31000".parse().expect("issuer address"),
        Transport::Plaintext,
        skein_fake_peers::Limits {
            io: profile.io,
            connections: 2,
            queue: fixtures::ROOM,
            plaintext: 32768,
            ciphertext: 32768,
            observations: 64,
            observation_bytes: 4096,
        },
        skein_fake_oauth::Config {
            terms: fixtures::issuer_terms(AccountStory::AccountSignInVaryingPort),
            authorization_url: fixtures::AUTHORIZE.into(),
            token_endpoint: fixtures::TOKEN.into(),
            client_id: b"client".as_slice().into(),
            client_secret: None,
            redirect_uri: fixtures::PARTY_REDIRECT.into(),
            refresh_token: b"seed".as_slice().into(),
        },
        limits,
        skein_http::server::Limits { response, ..profile.server },
    )
    .expect("issuer");
    for plan in plans {
        peer.queue(plan).expect("issuer plan");
    }
    peer
}
fn provider(story: Story) -> llm::Peer {
    let mut script = jig_channel_world::fixtures::one_chat(WORDS);
    script.result = br#"{"form":"report","text":"World answer","fields":{}}"#.as_slice().into();
    jig_channel_world::fixtures::provider(
        Box::new([skein_fake_llm_domain::api::Script {
            cue: Box::default(),
            turns: jig_channel_world::fixtures::provider_turns(&script),
        }]),
        skein_llm::Credential::bearer(
            if story == Story::Refused { b"refused-plan".as_slice() } else { fixtures::TOKEN_SECOND }.into(),
        ),
        Duration::ZERO,
        Transport::Plaintext,
        skein_fake_llm_protocol::documents::Provider::OpenAi,
    )
}
#[derive(Debug)]
enum Phase {
    Admission,
    Address,
    Usable,
    Stopping,
    Restored,
    Answer,
    SignOut,
    Closing,
    Ended,
}
struct Judge {
    story: Story,
    phase: Phase,
    received: usize,
    handed: bool,
    source: Option<u32>,
    account: Option<u32>,
    accounts: Option<c::OwnerAccounts>,
    confirmed: bool,
    restarts: u32,
}
impl Judge {
    fn watch(native: &mut NativeClient) {
        native.account_request(jig_client::Event::Watch {
            handle: ACCOUNTS,
            subject: jig_client::Watchable::Jig(c::Subject::Accounts),
        });
    }
    fn receive(&mut self, native: &NativeClient) -> Option<c::Answer> {
        let mut answer = None;
        for received in &native.received[self.received..] {
            match received {
                Received::Delivery(c::Delivery::OwnerAccounts(accounts)) => {
                    self.accounts = Some(accounts.as_ref().clone());
                }
                Received::Answer { answer: value, .. } => answer = Some(value.clone()),
                Received::SignedIn { .. } | Received::Sent { .. } | Received::Delivery(_) | Received::Read(_) => {}
            }
        }
        self.received = native.received.len();
        answer
    }
    fn visit(&mut self, answer: Option<c::Answer>, peers: &mut [Process]) {
        if let Some(answer) = answer {
            let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted { source, address })) = answer
            else {
                panic!("sign-in: {answer:?}")
            };
            self.source = Some(source);
            let Process::Visitor(visitor) = &mut peers[1] else { panic!("visitor") };
            visitor.hand(&address);
            self.phase = Phase::Usable;
        }
    }
    fn sign_out(&mut self, native: &mut NativeClient, answer: Option<c::Answer>) {
        if let Some(answer) = answer {
            assert_eq!(
                answer,
                c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignedOut {
                    confirmed: true,
                    failed_hosts: Box::new([]),
                }))
            );
            self.confirmed = true;
        }
        if self.confirmed && self.accounts.as_ref().is_some_and(|accounts| accounts.accounts.is_empty()) {
            native.account_request(jig_client::Event::Unwatch { handle: ACCOUNTS });
            native.finish_account_sign_in();
            self.phase = Phase::Closing;
        }
    }
    fn usable(&mut self) -> bool {
        let Some(accounts) = &self.accounts else { return false };
        let Some(account) = accounts.accounts.iter().find(|a| {
            a.sources
                .iter()
                .any(|s| Some(s.number) == self.source && s.usable && s.state == c::OwnerAccountState::Granted)
        }) else {
            return false;
        };
        assert!(account.plan, "a signed-in plan is granted");
        assert!(matches!(account.holder, c::OwnerAccountHolder::Party(_)));
        assert!(account.sources.iter().all(|s| s.kind == c::OwnerAccountKind::SignIn));
        if let Some(previous) = self.account {
            assert_eq!(account.number, previous, "restored account identity");
        }
        self.account = Some(account.number);
        true
    }
}
impl skein_world::Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let (app, others) = procs.split_at_mut(1);
        let Process::App { host, spare, restarts } = &mut app[0] else { panic!("service") };
        let (person, peers) = others.split_at_mut(1);
        let Process::Other(Proc::Client(native)) = &mut person[0] else { panic!("native client") };
        if !self.handed
            && let Some(link) = host.service.links().pop()
        {
            native.sign_in(&link).expect("printed link");
            self.handed = true;
        }
        let answer = self.receive(native);
        match self.phase {
            Phase::Admission if native.account_sign_in_ready() => {
                Self::watch(native);
                native.ask_account_sign_in(SIGN_IN, c::Ask::SignIn { holder: c::AccountHolder::Own, provider: 1 });
                self.phase = Phase::Address;
            }
            Phase::Address => self.visit(answer, peers),
            Phase::Usable | Phase::Restored => {
                if !self.usable() {
                    return;
                }
                if self.story == Story::Restart && self.restarts == 0 {
                    host.service.shutdown();
                    self.accounts = None;
                    self.phase = Phase::Stopping;
                } else {
                    native.hold_after_script();
                    native.resume_after_account_setup();
                    self.phase = Phase::Answer;
                }
            }
            Phase::Stopping if host.is_empty() => {
                let (root, worker) = spare.take().expect("independent roots retained for the next service");
                let store = host.store.clone();
                store.borrow_mut().recovering();
                **host = world::start_service(root, worker, None, &document(), Some(runtime()));
                host.store = store;
                self.restarts += 1;
                *restarts += 1;
                self.phase = Phase::Restored;
            }
            Phase::Answer => {
                let failure = host.store.borrow().rows.values().find_map(|row| {
                    if let jig_test_domain::Record::Core(jig_domain_core::Record::Core(
                        jig_domain_core::CoreRecord::Terminal(terminal),
                    )) = row
                        && let jig_domain_tasks::End::Failed(class) = terminal.end
                    {
                        Some(class)
                    } else {
                        None
                    }
                });
                if self.story == Story::Refused && failure.is_some() {
                    native.account_request(jig_client::Event::Unwatch { handle: ACCOUNTS });
                    native.finish_account_sign_in();
                    self.phase = Phase::Closing;
                } else if native.script_finished() {
                    native.hold_for_account_sign_in();
                    native.ask_account_sign_in(
                        SIGN_OUT,
                        c::Ask::SignOut {
                            holder: c::AccountHolder::Own,
                            source: self.source.expect("signed-in source"),
                        },
                    );
                    self.phase = Phase::SignOut;
                }
            }
            Phase::SignOut => self.sign_out(native, answer),
            Phase::Closing if native.is_empty() => {
                host.service.shutdown();
                for proc in peers {
                    match proc {
                        Process::Issuer(peer) => peer.shutdown(),
                        Process::Other(Proc::Peer(peer)) => peer.shutdown(),
                        Process::Visitor(_) | Process::Other(_) => {}
                        Process::App { .. } => unreachable!(),
                    }
                }
                self.phase = Phase::Ended;
            }
            Phase::Admission | Phase::Stopping | Phase::Closing | Phase::Ended => {}
        }
    }
    fn observe(&mut self, _now: Time, _procs: &[Process]) {}
    fn next_deadline(&self) -> Option<Time> {
        (!self.passed()).then_some(Time::from_nanos(WAIT.as_nanos()))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now.as_nanos() >= WAIT.as_nanos() && !self.passed())
            .then(|| format!("account story {:?}: phase {:?}, accounts {:?}", self.story, self.phase, self.accounts))
    }
    fn passed(&self) -> bool {
        matches!(self.phase, Phase::Ended)
    }
}

/// Run one account lifecycle with every peer on the same simulated kernel.
#[must_use]
pub fn run(seed: u64, story: Story) -> Outcome {
    let mut machine = world::machine();
    let client_root = machine.reopen_root(2);
    let mut world = skein_world::World::new_controlled(seed, skein_sim::Config::calm(), Memory::Unchecked, |_| Judge {
        story,
        phase: Phase::Admission,
        received: 0,
        handed: false,
        source: None,
        account: None,
        accounts: None,
        confirmed: false,
        restarts: 0,
    })
    .with_machine(world::Machine(machine));
    world.host(skein_world::HostedProgram {
        program: b"agent-program".as_slice().into(),
        make: agent,
        instances: 1,
        operations: smith_agent_process_world::limits().routes.checked_add(2).expect("agent operations"),
    });
    let mut roots = vec![
        StartupRoot { name: b"state".as_slice().into(), path: b"state".as_slice().into() },
        StartupRoot { name: b"worker".as_slice().into(), path: b"agent-root".as_slice().into() },
    ];
    if story == Story::Restart {
        roots.push(StartupRoot { name: b"state-again".as_slice().into(), path: b"state".as_slice().into() });
        roots.push(StartupRoot { name: b"worker-again".as_slice().into(), path: b"agent-root".as_slice().into() });
    }
    world.spawn_restartable(roots, if story == Story::Restart { restarting_service } else { service });
    world.spawn_root(skein_sim::Handle::new(client_root.raw()), |root| {
        let mut client = NativeClient::new(
            crate::fixtures::client_settings(),
            root,
            1000,
            seed,
            Script::answered_chat(WORDS, crate::fixtures::ANSWER),
        );
        client.hold_for_account_sign_in();
        Process::Other(Proc::Client(Box::new(SignalHost::new(client, None))))
    });
    world.spawn(|| Process::Issuer(Box::new(issuer(story))));
    world.spawn(|| Process::Visitor(Box::default()));
    world.spawn(|| Process::Other(Proc::Peer(Box::new(SignalHost::new(provider(story), None)))));
    let outcome = world.run();
    assert!(outcome.procs.iter().all(Host::is_empty));
    evidence(&outcome.procs)
}
fn evidence(procs: &[Process]) -> Outcome {
    let Process::App { host, restarts, .. } = &procs[0] else { panic!("service") };
    assert!(host.store.borrow().pending.is_empty(), "shutdown leaves no commit pending");
    let Process::Other(Proc::Client(native)) = &procs[1] else { panic!("client") };
    let Process::Issuer(issuer) = &procs[2] else { panic!("issuer") };
    let Process::Other(Proc::Peer(peer)) = &procs[4] else { panic!("provider") };
    Outcome {
        chats: native.chats_made,
        credentials: peer
            .observations()
            .iter()
            .filter_map(|o| if let llm::Observation::Credential { index, .. } = o { Some(*index) } else { None })
            .collect(),
        queries: peer
            .observations()
            .iter()
            .filter_map(|o| if let llm::Observation::Query { query, .. } = o { Some(query.clone()) } else { None })
            .collect(),
        posts: issuer.observations().iter().filter(|o| matches!(o, oauth::Observation::Post { .. })).count(),
        revokes: issuer.observations().iter().filter(|o| matches!(o, oauth::Observation::Revoke { .. })).count(),
        restarts: *restarts,
        confirmed: native.received.iter().any(|r| {
            matches!(
                r,
                Received::Answer {
                    answer: c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignedOut {
                        confirmed: true,
                        ..
                    })),
                    ..
                }
            )
        }),
        accounts_empty: native
            .received
            .iter()
            .rev()
            .find_map(|r| {
                if let Received::Delivery(c::Delivery::OwnerAccounts(a)) = r {
                    Some(a.accounts.is_empty())
                } else {
                    None
                }
            })
            .expect("accounts document"),
        run_failure: host.store.borrow().rows.values().find_map(|row| {
            if let jig_test_domain::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Terminal(
                t,
            ))) = row
                && let jig_domain_tasks::End::Failed(class) = t.end
            {
                Some(class)
            } else {
                None
            }
        }),
        provider_failure: host.wire.answers.last().cloned(),
    }
}
