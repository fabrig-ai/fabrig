//! The live layer's person-driven sign-in, bounded answer and confirmed sign-out.
use crate::support::{self, Person, Process};
use jig_client::{Asker, Handle, Key, contract as c};
use jig_client_protocol_kit::native::{NativeClient, Received, Script};
use jig_protocol_accounts_kit::fixtures;
use jig_protocol_config as config;
use jig_protocol_store::{codec, transcripts};
use skein_io::kernel::Signal;
use skein_lib::{Duration, Queue, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::{fs, net::ToSocketAddrs, path::Path};

const SIGN_IN: Key = Key { asker: Asker(77), number: 1 };
const SIGN_OUT: Key = Key { asker: Asker(77), number: 2 };
const ACCOUNTS: Handle = Handle { asker: Asker(77), number: 1 };
const CHAT: Handle = Handle { asker: Asker(77), number: 2 };
/// After the chat's run ends with a result, the person reads it within this.
const READ: Duration = Duration::from_secs(5);
const PROVIDER: u32 = 1;
const MODEL: &str = "gpt-6-luna";
/// smith's openai preset window. The engine's model asks no more than the agent holds, or the
/// worker's hello is refused.
pub const WINDOW: u32 = 272_000;
// configuration.md's small profile: 300 seconds, plus exchange/observation margin.
const SIGN_IN_WAIT: Duration = Duration::from_secs(660);
const WHOLE: Duration = Duration::from_secs(1200);

pub fn configuration(directory: &Path) -> codec::View {
    let address = ("api.openai.com", 443)
        .to_socket_addrs()
        .expect("resolve public Responses endpoint")
        .find(std::net::SocketAddr::is_ipv4)
        .expect("public Responses IPv4 endpoint")
        .to_string();
    configuration_at(directory, &address, Peers::Real)
}

pub fn configuration_at(directory: &Path, address: &str, peers: Peers) -> codec::View {
    let mut value = support::configuration_value(directory);
    let path = directory.join("config.json");
    let engine = &mut value["engine"];
    engine["llm"]["providers"][0] = serde_json::json!({
        "name":"ChatGPT", "number":PROVIDER, "models":[MODEL],
        "signed_in_plan":true, "manage_usage":"https://chatgpt.com/settings/usage",
        "sources":{"sign_in":peers.sign_in()}
    });
    let endpoint = &mut engine["llm"]["endpoints"][0];
    endpoint["provider"] = "openai".into();
    endpoint["dialect"] = 3.into();
    endpoint["address"] = serde_json::json!({"socket":address});
    endpoint["transport"] = "tls".into();
    endpoint["server_name"] = peers.server_name().into();
    endpoint["headers"] = serde_json::json!([]);
    peers.trust(endpoint, "trust", directory);
    engine["llm"]["models"][0]["name"] = MODEL.into();
    engine["llm"]["models"][0]["output"] = 4096.into();
    engine["llm"]["models"][0]["efforts"] = serde_json::json!(["low"]);
    engine["llm"]["models"][0]["window"] = WINDOW.into();
    engine["charters"][0]["model"] = MODEL.into();
    engine["charters"][0]["turns"] = 5.into();
    engine["charters"][0]["time_ms"] = 240_000.into();
    engine["charters"][0]["waiting_ms"] = 10_000.into();
    let bytes = serde_json::to_vec(&value).expect("live config JSON");
    support::private(&path, &bytes);
    let limits = store_limits(&bytes);
    let path = directory.join("agent.json");
    let mut agent = support::agent_value(directory);
    // The 64 KiB reasoning item and complete trace exceed standard's memory ceiling.
    agent["profile"]["declared"]["memory"] = (3_u64 << 30).into();
    let endpoint = &mut agent["endpoints"][0];
    endpoint["provider"] = "openai".into();
    endpoint["dialect"] = 3.into();
    endpoint["address"] = address.into();
    endpoint["transport"] = "tls".into();
    endpoint["server_name"] = peers.server_name().into();
    peers.trust(endpoint, "trust_der", directory);
    endpoint["headers"] = serde_json::json!([]);
    endpoint["connect"] = 10_000.into();
    endpoint["handshake"] = 10_000.into();
    endpoint["limits"]["output"] = 4096.into();
    endpoint["limits"]["reasoning_item"] = 65_536.into();
    endpoint["limits"]["window"] = WINDOW.into();
    endpoint["limits"]["idle"] = 300_000.into();
    agent["trace"] = support::trace::settings(directory);
    support::private(&path, &serde_json::to_vec(&agent).expect("live agent JSON"));
    support::private(&directory.join("charters/chat.txt"), INSTRUCTIONS.as_bytes());
    peers.describe(address);
    limits
}

#[derive(Debug)]
enum Phase {
    Admission,
    Address,
    Signing { source: u32 },
    Answer { source: u32 },
    SignOut,
    Closing,
    Ended,
}
struct Judge {
    finding: Option<String>,
    phase: Phase,
    peers: Peers,
    deadline: Time,
    sign_in_deadline: Option<Time>,
    received: usize,
    accounts: Option<c::Accounts>,
    state: String,
    chat: Option<u64>,
    task: Option<c::Task>,
    run: String,
    ended: Option<Time>,
}
fn client(procs: &mut [Process]) -> Result<&mut NativeClient, String> {
    let Process::Person(person) = procs.get_mut(1).ok_or("native person")? else {
        return Err(String::from("native person"));
    };
    person.client.as_mut().ok_or_else(|| String::from("native client"))
}
fn progress(previous: &mut String, document: &c::Accounts, source: u32) {
    let state = if let Some(end) = document.last_end.as_ref().filter(|end| end.source == source) {
        format!("{:?}", end.end)
    } else if document.signing.iter().any(|signing| signing.source == source) {
        "signing".into()
    } else {
        return;
    };
    let facts = document
        .accounts
        .iter()
        .find_map(|account| account.sources.iter().find(|grant| grant.number == source).map(|grant| (account, grant)));
    let state = if let Some((account, grant)) = facts {
        format!(
            "{state}; account={}; plan={}; state={:?}; usable={}",
            account.number, account.plan, grant.state, grant.usable
        )
    } else {
        state
    };
    if *previous != state {
        eprintln!("ChatGPT sign-in source={source}: {state}");
        *previous = state;
    }
}
/// The chat's run in words: its phase, attempt, retry, work, hold, ending and run wait. A
/// result's words are the page's; no grant or credential is in the task document.
pub(super) fn run_line(task: &c::Task) -> String {
    let c::ChipPhaseFacts { phase, details } = task.chip.details.as_ref();
    format!(
        "phase={phase:?}; attempt={:?}; retry={:?}; work={:?}; hold={:?}; ending={}; wait={:?}",
        details.attempt,
        details.retry,
        details.observation.as_ref().map(|observation| &observation.work),
        details.hold.as_ref().map(|hold| (&hold.reason, &hold.tries)),
        details.ending.as_ref().map_or_else(|| "none".to_owned(), ending),
        task.run_wait,
    )
}
fn ending(ending: &c::ChipEnding) -> String {
    let text = |text: &c::DocumentText| String::from_utf8_lossy(text.bytes.bytes()).into_owned();
    match ending {
        c::ChipEnding::Done(c::ResultValue::Report(words) | c::ResultValue::Words(words)) => {
            format!("Done({:?})", text(words))
        }
        c::ChipEnding::Done(c::ResultValue::Failure(words)) => format!("Done(Failure({:?}))", text(words)),
        c::ChipEnding::Done(_) => "Done(another result)".to_owned(),
        c::ChipEnding::Failed(words) => format!("Failed({:?})", text(words)),
        c::ChipEnding::Cancelled { reason, .. } => format!("Cancelled({:?})", text(reason)),
    }
}
/// Whether the run can no longer bring the answer: any failed or refused attempt, a hold, an
/// ending other than a report, or a host's refusal. Each fails the test at once, in words.
pub(super) fn failed(task: &c::Task) -> bool {
    let refused = matches!(task.run_wait, Some(c::TaskRunWait::Refused(_)));
    let c::ChipPhaseFacts { phase, details } = task.chip.details.as_ref();
    refused
        || details.retry.is_some()
        || details.hold.is_some()
        || matches!(phase, c::ChipPhase::Held { .. })
        || details.ending.as_ref().is_some_and(|ending| {
            !matches!(ending, c::ChipEnding::Done(c::ResultValue::Report(_) | c::ResultValue::Words(_)))
        })
}
fn reported(task: &c::Task) -> bool {
    task.chip.details.details.ending.is_some()
}
impl Judge {
    /// One line per change of the chat's run; a run that can't answer fails here, with its words.
    fn running(&mut self, now: Time) -> Result<(), String> {
        let (Some(chat), Some(task)) = (self.chat, &self.task) else { return Ok(()) };
        let line = run_line(task);
        if line != self.run {
            eprintln!("Live chat task={chat}: {line}");
            self.run = line;
        }
        if failed(task) {
            return Err(format!("live chat task={chat} can't answer: {}", self.run));
        }
        if reported(task) && self.ended.is_none() {
            self.ended = Some(now);
        }
        Ok(())
    }
    fn receive(&mut self, native: &NativeClient) -> Option<c::Answer> {
        let mut answer = None;
        for received in &native.received[self.received..] {
            match received {
                Received::Delivery(c::Delivery::Accounts(document)) => {
                    self.accounts = Some(document.as_ref().clone());
                }
                Received::Delivery(c::Delivery::Task(document)) if Some(document.chip.task) == self.chat => {
                    self.task = Some(document.as_ref().clone());
                }
                Received::Answer { answer: value, .. } => answer = Some(value.clone()),
                Received::SignedIn { .. } | Received::Sent { .. } | Received::Delivery(_) | Received::Read(_) => {}
            }
        }
        self.received = native.received.len();
        answer
    }
    fn signing(&mut self, procs: &mut [Process], source: u32) -> Result<(), String> {
        let Some(document) = &self.accounts else { return Ok(()) };
        progress(&mut self.state, document, source);
        let Some(end) = document.last_end.as_ref().filter(|end| end.source == source) else { return Ok(()) };
        match end.end {
            c::AccountEnd::Made { account, declined } | c::AccountEnd::Renewed { account, declined } => {
                if declined {
                    return Err(String::from("ChatGPT sign-in made, but plan usage was declined"));
                }
                let account = document.accounts.iter().find(|line| line.number == account).ok_or("made account")?;
                let grant = account.sources.iter().find(|grant| grant.number == source).ok_or("signed-in source")?;
                if !account.plan {
                    return Err(String::from("ChatGPT sign-in made without plan usage"));
                }
                if grant.usable && grant.state == c::AccountState::Granted {
                    if !matches!(account.holder, c::AccountHolder::Party(_)) {
                        return Err(String::from("a plan is held by its person"));
                    }
                    {
                        let (observed_value, wanted) = (&(grant.kind), &(c::AccountKind::SignIn));
                        if observed_value != wanted {
                            return Err(format!(
                                "assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                            ));
                        }
                    }
                    self.sign_in_deadline = None;
                    let native = client(procs)?;
                    native.account_request(jig_client::Event::Unwatch { handle: ACCOUNTS });
                    native.hold_after_script();
                    native.resume_after_account_setup();
                    self.phase = Phase::Answer { source };
                }
            }
            c::AccountEnd::Failed { failure } => return Err(format!("ChatGPT sign-in failed: {failure:?}")),
            c::AccountEnd::Refused { refusal } => return Err(format!("ChatGPT sign-in refused: {refusal:?}")),
        }
        Ok(())
    }
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.finding.is_none() {
            self.finding = self.check_act(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, procs: &[Process]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        let Process::Binary(binary) = procs.first().ok_or("binary")? else { return Err(String::from("binary")) };
        if binary.exit_status().is_some() && !matches!(self.phase, Phase::Ended) {
            return Err(format!(
                "jig-test failed {:?}: {}",
                binary.exit_status(),
                String::from_utf8_lossy(binary.stderr())
            ));
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if matches!(self.phase, Phase::Ended) {
            None
        } else {
            let read = self.ended.map(|ended| ended.saturating_add(READ));
            [Some(self.deadline), self.sign_in_deadline, read].into_iter().flatten().min()
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.next_deadline()
            .filter(|at| now >= *at)
            .map(|_| format!("live ChatGPT phase {:?}: {}; chat: {}", self.phase, self.state, self.run))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && matches!(self.phase, Phase::Ended)
    }
}

pub fn run(directory: &Path, started: Time, peers: Peers) -> Result<(u32, u32), String> {
    let clock = Clock::new();
    let mut world = real::World::new(Judge {
        finding: None,
        phase: Phase::Admission,
        peers,
        deadline: started.saturating_add(peers.deadline()),
        sign_in_deadline: None,
        received: 0,
        accounts: None,
        state: String::new(),
        chat: None,
        task: None,
        run: String::new(),
        ended: None,
    });
    let fakes = if let Peers::Fake | Peers::Refusing = peers {
        let (issuer, issuer_address, issuer_fd) = support::issuer();
        let (peer, peer_address, peer_fd) = fake_peer(peers);
        support::ports::configure(directory, Some(issuer_address), Some(peer_address));
        Some((issuer, issuer_fd, peer, peer_fd))
    } else {
        None
    };
    let _client_directory = spawn_client(
        &mut world,
        directory,
        Script::answered_report(WORDS.as_bytes(), support::ANSWER, Duration::from_secs(240)),
        support::CLIENT_SEED,
    )?;
    if let Some((issuer, issuer_fd, peer, peer_fd)) = fakes {
        world.spawn_with_fds(vec![issuer_fd], || Process::Issuer(Box::new(issuer)));
        world.spawn(|| Process::Visitor(Box::default()));
        world.spawn_with_fds(vec![peer_fd], || Process::Peer(Box::new(peer)));
    }
    let remaining = started.saturating_add(peers.whole()).saturating_since(clock.now().now);
    let outcome = world.try_run(&clock, remaining)?;
    if !outcome.procs.iter().all(Host::is_empty) {
        return Err(String::from("all live sign-in io settles"));
    }
    let Process::Person(person) = &outcome.procs[1] else { return Err(String::from("native person")) };
    let native = person.client.as_ref().ok_or("signed-in client")?;
    Ok((native.signed_in, native.chats_made))
}

pub const INSTRUCTIONS: &str = "Report exactly: Process answer";
pub const WORDS: &str = "Follow the first sentence.";

pub fn store_limits(bytes: &[u8]) -> codec::View {
    let document = config::document(bytes, &config::DocumentLimits { bytes: 1 << 20, tokens: 1 << 20, depth: 32 })
        .expect("live document");
    let at = config::read::Object::new(&document, 0)
        .expect("document object")
        .member(&document, b"engine")
        .expect("engine member");
    let engine = config::engine(&document, at, &config::Directories::working(), config::Defaults::Document)
        .expect("live engine configuration");
    codec::limits(&engine.limits.core).expect("live store codec limits")
}

pub fn completions(directory: &Path, limits: &codec::View) -> Result<u32, String> {
    let path = directory.join("state/transcripts/0000000000000000/0000000000000001");
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(format!("live transcript: {error}")),
    };
    let mut at = 0;
    let mut count = 0;
    while at + 4 <= bytes.len() {
        let length =
            u32::from_be_bytes(bytes[at..at + 4].try_into().map_err(|why| format!("frame length: {why:?}"))?) as usize;
        if length < 12 || at + length > bytes.len() {
            break;
        }
        let turn = transcripts::frame::decode(&bytes[at..at + length], limits)
            .map_err(|why| format!("durable live turn: {why:?}"))?;
        {
            let (observed_value, wanted) = (&(turn.task, turn.attempt, turn.turn), &(1, 1, count + 1));
            if observed_value != wanted {
                return Err(format!("assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"));
            }
        }
        // Smith's transcript.md, 1: a turn is one completion, including compaction.
        count += 1;
        at += length;
    }
    Ok(count)
}

pub fn capture(directory: &Path, destination: &Path) -> Result<bool, String> {
    let bytes =
        fs::read(directory.join("agent-trace.jsonl")).map_err(|why| format!("Smith provider trace: {why:?}"))?;
    let mut requests = Vec::new();
    for line in bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let record: serde_json::Value =
            serde_json::from_slice(line).map_err(|why| format!("Smith event JSON: {why:?}"))?;
        if let Some(prompt) = record.get("prompt") {
            requests.push(serde_json::json!({"instructions":prompt["system"],"input":prompt["messages"]}));
        }
    }
    fs::write(
        destination,
        serde_json::to_vec_pretty(&requests).map_err(|why| format!("request capture JSON: {why:?}"))?,
    )
    .map_err(|why| format!("system/input capture file: {why:?}"))?;
    let present = requests.iter().any(|request| {
        request["instructions"].as_str().is_some_and(|system| system.contains(INSTRUCTIONS) && system.contains(WORDS))
    });
    eprintln!(
        "live request capture={}; brief={}; file={}",
        requests.len(),
        if present { "present" } else { "absent" },
        destination.display()
    );
    if requests.is_empty() {
        return Err(String::from("provider request captured before interpreting the answer"));
    }
    Ok(present)
}

pub fn check(directory: &Path, limits: &codec::View, outcome: (u32, u32), peers: Peers) -> Result<(), String> {
    let count = completions(directory, limits)?;
    eprintln!("live completions={count}");
    if !capture(directory, &directory.join("live-request.json"))? {
        return Err(String::from("the provider request includes charter and task words"));
    }
    support::trace::try_check(directory, peers.credential())?;
    if !(1..=5).contains(&count) {
        return Err(String::from("at most five real completions, and at least one answer"));
    }
    {
        let (observed_value, wanted) = (&(outcome), &(1, 1));
        if observed_value != wanted {
            return Err(format!("assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"));
        }
    }
    Ok(())
}

/// The live story's real backends or independent fake peers on loopback.
#[derive(Clone, Copy)]
pub enum Peers {
    Real,
    Fake,
    /// The fake peers, with a provider that refuses the plan's grant.
    Refusing,
}
impl Peers {
    fn sign_in(self) -> serde_json::Value {
        match self {
            Self::Real => serde_json::json!({
                "issuer":"https://auth.openai.com", "client":"dynamic_agent_client",
                "authorization":"https://auth.openai.com/api/accounts/authorize",
                "token":"https://auth.openai.com/api/accounts/oauth/token",
                "revocation":"https://auth.openai.com/api/accounts/oauth/revoke",
                "keys":"https://auth.openai.com/.well-known/jwks.json",
                "transport":"tls", "server_name":"auth.openai.com",
                "scopes":["openid","profile","email","offline_access","resource.invoke","chatgpt.tokens.use.direct"],
                "must_grant":["chatgpt.tokens.use.direct"], "resource":"https://api.openai.com/v1",
                "name_hint":"jig", "redirect_path":"auth/callback", "redirect_port":1455
            }),
            Self::Fake | Self::Refusing => serde_json::json!({
                "issuer":"http://127.0.0.1:0", "client":"client", "scopes":["openid","read"],
                "name_hint":"jig-test", "authorization":"http://127.0.0.1:0/authorize",
                "token":"http://127.0.0.1:0/token", "revocation":"http://127.0.0.1:0/revoke",
                "keys":"http://127.0.0.1:0/keys", "transport":"plaintext",
                "must_grant":["openid","read"], "resource":"api", "redirect_path":"callback"
            }),
        }
    }
    fn server_name(self) -> &'static str {
        match self {
            Self::Real => "api.openai.com",
            Self::Fake | Self::Refusing => "skein.test",
        }
    }
    fn trust(self, endpoint: &mut serde_json::Value, field: &str, directory: &Path) {
        match self {
            Self::Real => {
                endpoint.as_object_mut().expect("endpoint").remove(field);
            }
            Self::Fake | Self::Refusing => endpoint[field] = serde_json::json!(directory.join("root.der")),
        }
    }
    fn describe(self, address: &str) {
        match self {
            Self::Real => {
                eprintln!("Live model={MODEL}; Responses={address}; TLS server_name=api.openai.com; machine trust");
            }
            Self::Fake | Self::Refusing => {
                eprintln!("Live model={MODEL}; Responses={address}; TLS server_name=skein.test; fixture trust");
            }
        }
    }
    fn sign_in_wait(self) -> Duration {
        match self {
            Self::Real => SIGN_IN_WAIT,
            Self::Fake | Self::Refusing => Duration::from_secs(20),
        }
    }
    fn deadline(self) -> Duration {
        match self {
            Self::Real => Duration::from_secs(1180),
            Self::Fake | Self::Refusing => Duration::from_secs(20),
        }
    }
    fn whole(self) -> Duration {
        match self {
            Self::Real => WHOLE,
            Self::Fake | Self::Refusing => Duration::from_secs(30),
        }
    }
    fn credential(self) -> Option<&'static [u8]> {
        match self {
            Self::Real => None,
            Self::Fake | Self::Refusing => Some(fixtures::TOKEN_SECOND),
        }
    }
    fn visit(self, procs: &mut [Process], address: &[u8]) -> Result<(), String> {
        match self {
            Self::Real => {
                eprintln!(
                    "Sign in at: {}",
                    std::str::from_utf8(address).map_err(|why| format!("authorization URL: {why:?}"))?
                );
                eprintln!("(the browser must reach 127.0.0.1:1455 on this machine: ssh -L 1455:127.0.0.1:1455 HOST)");
            }
            Self::Fake | Self::Refusing => {
                let Process::Visitor(visitor) = procs.get_mut(3).ok_or("browser")? else {
                    return Err(String::from("browser"));
                };
                visitor.hand(address);
            }
        }
        Ok(())
    }
    fn shutdown(self, procs: &mut [Process]) {
        if let Self::Fake | Self::Refusing = self {
            for proc in procs {
                match proc {
                    Process::Issuer(issuer) => issuer.shutdown(),
                    Process::Peer(peer) => peer.shutdown(),
                    Process::Binary(_) | Process::Person(_) | Process::Visitor(_) => {}
                }
            }
        }
    }
}

/// Shared fake model and credential check for the process stories.
pub(super) fn fake_peer(peers: Peers) -> (skein_fake_peers::llm::Peer, std::net::SocketAddr, skein_io::kernel::Fd) {
    let mut script = jig_channel_kit::fixtures::one_chat(WORDS.as_bytes());
    script.result = br#"{"form":"report","text":"Process answer","fields":{}}"#.as_slice().into();
    support::ports::provider(
        Box::new([skein_fake_llm_domain::api::Script {
            cue: Box::default(),
            turns: jig_channel_kit::fixtures::provider_turns(&script),
        }]),
        skein_llm::Credential::bearer(match peers {
            Peers::Refusing => b"not the plan's grant".as_slice().into(),
            Peers::Real | Peers::Fake => fixtures::TOKEN_SECOND.into(),
        }),
    )
}

/// The process stories share a native client held for account setup.
pub(super) fn spawn_client<R: Referee<Process>>(
    world: &mut real::World<Process, R>,
    directory: &Path,
    script: Script,
    seed: u64,
) -> Result<tempfile::TempDir, String> {
    let mut binary = support::start(directory);
    let streams = binary.take_streams().ok_or("binary pipes")?;
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { return Err(String::from("pipes mode")) };
    let client_directory =
        tempfile::tempdir_in(directory).map_err(|why| format!("native client directory: {why:?}"))?;
    let root = skein_shell::open_root(client_directory.path()).map_err(|why| format!("client root: {why:?}"))?;
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds.into_iter().chain([root]).collect(), || {
        Process::Person(Box::new(Person {
            seed,
            client: None,
            root: Some(root),
            script: Some(script),
            account_sign_in: true,
            llm_key: None,
            input: Some(input),
            output: Some(output),
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            completions: Queue::with_capacity(512),
            submissions: Queue::with_capacity(512),
        }))
    });
    Ok(client_directory)
}

impl Judge {
    fn check_act(&mut self, now: Time, procs: &mut [Process]) -> Result<(), String> {
        let Process::Person(person) = procs.get_mut(1).ok_or("native person")? else {
            return Err(String::from("native person"));
        };
        let Some(native) = &mut person.client else { return Ok(()) };
        let answer = self.receive(native);
        match self.phase {
            Phase::Admission => {
                if native.signed_in == 1 && native.account_sign_in_ready() {
                    native.account_request(jig_client::Event::Watch {
                        handle: ACCOUNTS,
                        subject: jig_client::Watchable::Jig(c::Subject::Accounts),
                    });
                    native.ask_account_sign_in(
                        SIGN_IN,
                        c::Ask::SignIn { holder: c::AskAccountHolder::Own, provider: PROVIDER },
                    );
                    self.sign_in_deadline = Some(now.saturating_add(self.peers.sign_in_wait()));
                    self.phase = Phase::Address;
                }
            }
            Phase::Address => {
                if let Some(answer) = answer {
                    let c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::AccountSignInStarted {
                        source,
                        address,
                    })) = answer
                    else {
                        return Err(format!("ChatGPT sign-in start: {answer:?}"));
                    };
                    self.peers.visit(procs, &address)?;
                    self.phase = Phase::Signing { source };
                    self.signing(procs, source)?;
                }
            }
            Phase::Signing { source } => self.signing(procs, source)?,
            Phase::Answer { source } => {
                match answer {
                    Some(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Created { first, .. })))
                        if self.chat.is_none() =>
                    {
                        self.chat = Some(first);
                        native.account_request(jig_client::Event::Watch {
                            handle: CHAT,
                            subject: jig_client::Watchable::Jig(c::Subject::Task { task: first }),
                        });
                    }
                    Some(refused @ c::Answer::Refused { .. }) => return Err(format!("live chat refused: {refused:?}")),
                    Some(_) | None => {}
                }
                self.running(now)?;
                if native.script_finished() {
                    native.account_request(jig_client::Event::Unwatch { handle: CHAT });
                    self.ended = None;
                    native.hold_for_account_sign_in();
                    native.ask_account_sign_in(SIGN_OUT, c::Ask::SignOut { holder: c::AskAccountHolder::Own, source });
                    self.phase = Phase::SignOut;
                }
            }
            Phase::SignOut => {
                if let Some(answer) = answer {
                    {
                        let (observed_value, wanted) = (
                            &(answer),
                            &(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::AccountSignedOut {
                                confirmed: true,
                                failed_hosts: Box::new([]),
                            }))),
                        );
                        if observed_value != wanted {
                            return Err(format!(
                                "ChatGPT sign-out confirmed by the issuer and every host; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                            ));
                        }
                    }
                    eprintln!("ChatGPT sign-out confirmed=true; failed_hosts=[]");
                    native.finish_account_sign_in();
                    self.phase = Phase::Closing;
                }
            }
            Phase::Closing => {
                if native.is_empty() {
                    let Process::Binary(binary) = procs.get_mut(0).ok_or("binary")? else {
                        return Err(String::from("binary"));
                    };
                    binary.signal(Signal::Kill);
                    self.peers.shutdown(procs);
                    self.phase = Phase::Ended;
                }
            }
            Phase::Ended => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The diagnostic checked by the refused-plan process story is formatted without a process.
    #[test]
    fn a_refused_plan_is_reported_with_the_run_failure() {
        let mut task = jig_client_protocol_link::translate::documents::task(
            jig_client_test_records::task_with_hold_record(),
            &jig_client_protocol_kit::fixtures::link_limits(),
        )
        .expect("owner task fixture");
        let c::ChipPhaseFacts { phase, details } = task.chip.details.as_mut();
        *phase = c::ChipPhase::Active(c::RunStage::BackingOff { until: skein_lib::Wall::from_nanos(17) });
        details.attempt = Some(1);
        details.retry =
            Some(c::RetryNotice::OwnerFailed { class: c::HoldFailure::Run, count: 1, allowance: 3, reason: None });
        details.hold = None;
        details.ending = None;
        details.observation = None;
        task.run_wait = None;
        assert!(failed(&task));
        assert_eq!(
            run_line(&task),
            "phase=Active(BackingOff { until: Wall(17) }); attempt=Some(1); retry=Some(OwnerFailed { class: Run, count: 1, allowance: 3, reason: None }); work=None; hold=None; ending=none; wait=None"
        );
    }
}
