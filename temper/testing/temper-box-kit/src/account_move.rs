//! client/ctl/testing.md, 4: ctl moves an account between two shipped deployments.
use super::{
    Process, ctl, sign_in,
    story_socket::{Scope, Socket},
};
use skein_fake_peers::{llm, oauth};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{
    fs,
    io::Write,
    net::{SocketAddr, TcpStream},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::mpsc,
};

const WITHIN: Duration = Duration::from_secs(20);
const FILE_REFRESH: &[u8] = b"fake-refresh";
const ROTATED_REFRESH: &[u8] = b"moved-refresh";
const A: usize = 0;
const B: usize = 2;
const VISITOR: usize = 4;

fn document(directory: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(directory.join("temper.json")).expect("configuration")).expect("JSON")
}

fn configure(directory: &Path, llm: SocketAddr) -> oauth::Peer {
    let issuer = sign_in::configuration(directory);
    let mut value = document(directory);
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket": llm.to_string()});
    let agent_path = directory.join("agent.json");
    let mut agent: serde_json::Value = serde_json::from_slice(&fs::read(&agent_path).expect("agent")).expect("JSON");
    agent["endpoints"][0]["address"] = llm.to_string().into();
    super::write_private(&agent_path, &serde_json::to_vec(&agent).expect("agent JSON"));
    // Keep init's LLM registration, charters and worker, with no forge records
    // (temper/docs/design/protocol/setup/init.md, 3).
    let (_, fixture) = super::configuration::documents(
        directory,
        skein_scratch::paths::binary("temper-box").to_str().expect("a UTF-8 path"),
    );
    let fixture: serde_json::Value = serde_json::from_slice(&fixture).expect("fixture JSON");
    for field in ["machine", "demo", "webhooks", "workers", "clients"] {
        value["application"][field] = fixture["application"][field].clone();
    }
    super::write_private(&directory.join("temper.json"), &serde_json::to_vec(&value).expect("configuration JSON"));
    issuer
}

fn calls(procs: &[Process], deployment: usize) -> Result<&ctl::Calls, String> {
    Ok({
        let Some(Process::Calls(calls)) = procs.get(deployment + 1) else { return Err(String::from("ctl calls")) };
        calls
    })
}
fn queue(procs: &mut [Process], deployment: usize, arguments: &[&str]) -> Result<(), String> {
    let Some(Process::Calls(calls)) = procs.get_mut(deployment + 1) else { return Err(String::from("ctl calls")) };
    calls.push(ctl::Step::Run(ctl::Call::new(arguments)));

    Ok(())
}
fn success(ended: &super::Ended) -> Result<(), String> {
    if ended.exit != Some(Exit::Code(0)) {
        return Err(format!("ctl: {}", String::from_utf8_lossy(&ended.stderr)));
    }

    Ok(())
}
fn account(ended: &super::Ended) -> Result<serde_json::Value, String> {
    Ok({
        let value = super::check_ctl_json(ended)?;
        let rows = value["accounts"]["party"].as_array().ok_or(String::from("holder's accounts"))?;
        let [account] = rows.as_slice() else { return Err(String::from("one signed-in account")) };
        {
            let (left, right) = (&(account["provider"]), &(1));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(account["plan"]), &(true));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        if !account["sources"].as_array().ok_or(String::from("sources"))?.iter().any(|source| {
            source["kind"] == "sign_in" && source["usable"] == true && source["state"]["kind"] == "granted"
        }) {
            return Err(String::from("usable signed-in source"));
        }
        account.clone()
    })
}

#[derive(Debug)]
enum Phase {
    Connect,
    Account,
    Export,
    Withdrawn,
    Import,
    Empty,
    Stop,
}
struct Move {
    failed: Option<String>,
    phase: Phase,
    file: PathBuf,
    record: Vec<u8>,
    number: u64,
    moved: u64,
    deadline: Time,
    settled: bool,
    result: mpsc::Sender<(u64, Vec<u8>)>,
}
impl Referee<Process> for Move {
    #[expect(clippy::too_many_lines, reason = "the account transfer keeps each phase's finding before advancing")]
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            for deployment in [A, B] {
                for ended in calls(procs, deployment)?.ends() {
                    success(ended)?;
                }
            }
            match self.phase {
                Phase::Connect if calls(procs, A)?.ends().len() == 2 => {
                    let address = sign_in::authorization_address(&calls(procs, A)?.ends()[1])
                        .ok_or(String::from("connect's authorization address"))?
                        .to_vec();
                    let Some(Process::Visitor(visitor)) = procs.get_mut(VISITOR) else {
                        return Err(String::from("browser"));
                    };
                    visitor.hand(&address);
                    queue(procs, A, &["account", "list", "--until", "settled", "--json"])?;
                    queue(procs, A, &["account", "list", "--json"])?;
                    self.phase = Phase::Account;
                }
                Phase::Account if calls(procs, A)?.ends().len() == 4 => {
                    self.number = account(&calls(procs, A)?.ends()[3])?["number"]
                        .as_u64()
                        .ok_or(String::from("account number"))?;
                    if self.file.exists() {
                        return Err(String::from("export creates a new file"));
                    }
                    queue(
                        procs,
                        A,
                        &[
                            "account",
                            "export",
                            &self.number.to_string(),
                            self.file.to_str().ok_or(String::from("file path"))?,
                        ],
                    )?;
                    self.phase = Phase::Export;
                }
                Phase::Export if calls(procs, A)?.ends().len() == 5 => {
                    let expected = format!("moved out: account {} to {}\n", self.number, self.file.display());
                    {
                        let (left, right) = (&(calls(procs, A)?.ends()[4].stdout), &(expected.as_bytes()));
                        if left != right {
                            return Err(format!(
                                "assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"
                            ));
                        }
                    }
                    {
                        let (left, right) = (
                            &(fs::metadata(&self.file)
                                .map_err(|error| format!("exported file: {error:?}"))?
                                .permissions()
                                .mode()
                                & 0o777),
                            &(0o600),
                        );
                        if left != right {
                            return Err(format!(
                                "assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"
                            ));
                        }
                    }
                    self.record = fs::read(&self.file)
                        .map_err(|error| format!("keep file bytes only for the stale-token check: {error:?}"))?;
                    queue(procs, A, &["account", "list", "--json"])?;
                    self.phase = Phase::Withdrawn;
                }
                Phase::Withdrawn if calls(procs, A)?.ends().len() == 6 => {
                    let empty = super::check_ctl_json(&calls(procs, A)?.ends()[5])?;
                    if !empty["accounts"]["party"].as_array().ok_or(String::from("A's accounts"))?.is_empty() {
                        return Err(String::from("A holds nothing before import"));
                    }
                    let Some(Process::Calls(b)) = procs.get_mut(B + 1) else { return Err(String::from("B's calls")) };
                    b.push(ctl::Step::Run(ctl::Call::sign_in()));
                    queue(
                        procs,
                        B,
                        &[
                            "account",
                            "import",
                            self.file.to_str().ok_or(String::from("file path"))?,
                            "--key",
                            "move-in",
                        ],
                    )?;
                    self.phase = Phase::Import;
                }
                Phase::Import if calls(procs, B)?.ends().len() == 2 => {
                    let output = std::str::from_utf8(&calls(procs, B)?.ends()[1].stdout)
                        .map_err(|error| format!("import output: {error:?}"))?;
                    self.moved = output
                        .strip_prefix("moved in: account ")
                        .ok_or(String::from("import names its account"))?
                        .strip_suffix('\n')
                        .ok_or(String::from("one output line"))?
                        .parse()
                        .map_err(|error| format!("B's account number: {error:?}"))?;
                    if self.file.exists() {
                        return Err(String::from("import removes the file"));
                    }
                    queue(procs, B, &["account", "list", "--until", "settled", "--json"])?;
                    queue(procs, B, &["account", "list", "--json"])?;
                    self.phase = Phase::Empty;
                }
                Phase::Empty if calls(procs, A)?.ends().len() == 6 && calls(procs, B)?.ends().len() == 4 => {
                    if account(&calls(procs, B)?.ends()[3])?["number"] != self.moved {
                        return Err(String::from("B holds the imported account"));
                    }
                    let empty = super::check_ctl_json(&calls(procs, A)?.ends()[5])?;
                    if !empty["accounts"]["party"].as_array().ok_or(String::from("A's accounts"))?.is_empty() {
                        return Err(String::from("A holds nothing"));
                    }
                    self.result
                        .send((self.moved, std::mem::take(&mut self.record)))
                        .map_err(|error| format!("move result: {error:?}"))?;
                    for deployment in [A, B] {
                        let Some(Process::Calls(calls)) = procs.get_mut(deployment + 1) else {
                            return Err(String::from("ctl calls"));
                        };
                        calls.release();
                        let Some(Process::Binary(binary)) = procs.get_mut(deployment) else {
                            return Err(String::from("serve"));
                        };
                        binary.signal(Signal::Terminate);
                    }
                    self.phase = Phase::Stop;
                }
                Phase::Connect
                | Phase::Account
                | Phase::Export
                | Phase::Withdrawn
                | Phase::Import
                | Phase::Empty
                | Phase::Stop => {}
            }

            Ok(())
        })();
        if let Err(why) = result {
            self.failed = Some(why);
        }
    }
    fn watch(&mut self, _now: Time, procs: &[Process]) -> Result<(), String> {
        if let Some(why) = self.failed.take() {
            return Err(why);
        }
        for deployment in [A, B] {
            let Some(Process::Binary(binary)) = procs.get(deployment) else { return Err(String::from("serve")) };
            if !matches!(self.phase, Phase::Stop) && binary.exit_status().is_some() {
                return Err(format!("serve failed: {}", String::from_utf8_lossy(binary.stderr())));
            }
        }
        self.settled = matches!(self.phase, Phase::Stop) && procs.iter().all(Host::is_empty);

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| format!("account move stalled at {:?}", self.phase))
    }
    fn passed(&self) -> bool {
        if self.failed.is_some() {
            return false;
        }
        self.settled
    }
}

fn move_account(a: &Path, b: &Path, file: &Path) -> (u64, Vec<u8>) {
    let clock = Clock::new();
    let (result, received) = mpsc::channel();
    let mut world = real::World::new(Move {
        failed: None,
        phase: Phase::Connect,
        file: file.into(),
        record: Vec::new(),
        number: 0,
        moved: 0,
        deadline: clock.now().now.saturating_add(WITHIN),
        settled: false,
        result,
    });
    for (directory, steps) in [
        (
            a,
            vec![
                ctl::Step::Run(ctl::Call::sign_in()),
                ctl::Step::Run(ctl::Call::new(&["account", "connect", "1", "--key", "connect-move"])),
            ],
        ),
        (b, Vec::new()),
    ] {
        let mut binary = super::serve(directory);
        let streams = binary.take_streams().expect("serve pipes");
        let root = skein_shell::open_root(directory).expect("deployment root");
        let mut fds = streams.descriptors();
        fds.push(root);
        world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
        world.spawn_with_fds(fds, || {
            let mut calls = ctl::Calls::new(root, steps, Some(&streams), None);
            calls.hold();
            Process::Calls(Box::new(calls))
        });
    }
    world.spawn(|| Process::Visitor(Box::default()));
    let outcome = world.run(&clock, WITHIN);
    for deployment in [A, B] {
        let Process::Binary(binary) = &outcome.procs[deployment] else { panic!("serve") };
        assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "healthy stop before B's restart");
    }
    assert!(outcome.procs.iter().all(Host::is_empty), "both deployments drained");
    received.recv().expect("completed move")
}

struct Peers {
    failed: Option<String>,
    stop: mpsc::Receiver<()>,
    ended: bool,
    deadline: Time,
    wake: Time,
}
impl Referee<Process> for Peers {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            self.wake = now.saturating_add(Duration::from_millis(10));
            if !self.ended && !matches!(self.stop.try_recv(), Err(mpsc::TryRecvError::Empty)) {
                let [Process::Issuer(issuer), Process::Peer(peer)] = procs else {
                    return Err(String::from("independent peers"));
                };
                issuer.shutdown();
                peer.shutdown();
                self.ended = true;
            }

            Ok(())
        })();
        if let Err(why) = result {
            self.failed = Some(why);
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Process]) -> Result<(), String> {
        if let Some(why) = self.failed.take() {
            return Err(why);
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.deadline.min(self.wake))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline).then(|| "account move peers did not settle".into())
    }
    fn passed(&self) -> bool {
        if self.failed.is_some() {
            return false;
        }
        self.ended
    }
}

pub mod tests {
    use super::{Peers, Referee, Time, mpsc};

    pub fn a_judge_keeps_an_action_finding_for_watch() {
        let (stop, receive) = mpsc::channel();
        let mut judge = Peers { failed: None, stop: receive, ended: false, deadline: Time::ZERO, wake: Time::ZERO };
        stop.send(()).expect("stop requested");
        judge.act(Time::ZERO, &mut []);
        judge.ended = true;
        assert!(!judge.passed(), "a kept finding prevents a pass");
        assert_eq!(judge.watch(Time::ZERO, &[]), Err(String::from("independent peers")));
        assert_eq!(judge.watch(Time::ZERO, &[]), Ok(()), "watch takes the kept finding");
    }
}

fn refuse_file_token(address: SocketAddr, client: &str, record: &[u8]) {
    // The fake's known token is checked against the actual exported bytes;
    // neither the moved form nor any credential is printed on a failure.
    assert!(skein_lib::bytes::find(record, FILE_REFRESH).is_some(), "file holds the fixture's refresh token");
    let body = format!(
        "grant_type=refresh_token&client_id={client}&refresh_token={}",
        std::str::from_utf8(FILE_REFRESH).expect("fixture token")
    );
    let mut socket = Socket::new(TcpStream::connect(address).expect("same fake issuer"), Scope::World);
    socket.stream().write_all(format!("POST /api/accounts/oauth/token HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).expect("replay sent");
    let response =
        socket.read_to_end("the moved token's replay refusal", Duration::from_secs(5), 8192).expect("issuer refusal");
    assert!(response.starts_with(b"HTTP/1.1 400 "), "stale token refused");
    let body = response.split(|byte| *byte == b'\n').next_back().expect("refusal body");
    let error: serde_json::Value = serde_json::from_slice(body).expect("issuer error JSON");
    assert_eq!(error["error"], "invalid_grant");
}

fn assert_peers(issuer: &oauth::Peer, peer: &llm::Peer) {
    let observations = issuer.observations();
    assert_eq!(
        observations.iter().filter(|o| matches!(o, oauth::Observation::Post { .. })).count(),
        3,
        "A's code grant, B's one refresh, and the stale file replay"
    );
    assert_eq!(
        observations.iter().filter(|o| matches!(o, oauth::Observation::Answered { status: 200, .. })).count(),
        2
    );
    assert_eq!(
        observations.iter().filter(|o| matches!(o, oauth::Observation::Answered { status: 400, .. })).count(),
        1
    );
    assert_eq!(observations.iter().filter(|o| matches!(o, oauth::Observation::Revoke { .. })).count(), 0);
    let credentials: Vec<_> = peer
        .observations()
        .iter()
        .filter_map(|o| if let llm::Observation::Credential { index, .. } = o { Some(*index) } else { None })
        .collect();
    assert_eq!(credentials, [Some(0)], "B alone uses the refreshed account's grant");
    assert_eq!(
        peer.observations().iter().filter(|o| matches!(o, llm::Observation::Answered { result: Ok(()), .. })).count(),
        1,
        "one chat answer"
    );
}

fn rotation() -> skein_fake_oauth::Plan {
    skein_fake_oauth::Plan {
        status: 200,
        body: skein_fake_oauth::Body::Token(skein_oauth::TokenResponse {
            access_token: skein_lib::Secret::new(sign_in::ACCESS.as_ref().into()),
            refresh_token: Some(skein_lib::Secret::new(ROTATED_REFRESH.into())),
            expires_in: 300,
            scope: None,
            id_token: None,
            earliest_refresh_at: None,
        }),
        delay: Duration::ZERO,
        retry_after: Duration::ZERO,
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "the fake peers' world serves the synchronous account-transfer driver on a scoped thread"
)]
pub fn run(directory: &Path) {
    let peer = sign_in::plan_peer(Box::new([skein_fake_llm_domain::api::Script {
        cue: Box::default(),
        turns: Box::new([skein_fake_llm_domain::api::Turn {
            lines: Box::new([skein_fake_llm_domain::api::Line::Text { text: super::ANSWER.into() }]),
            finish: skein_fake_llm_domain::api::Finish::Stop,
            tokens: 1,
        }]),
    }]));
    let llm = peer.address();
    let a = directory.join("a");
    let b = directory.join("b");
    for deployment in [&a, &b] {
        fs::create_dir(deployment).expect("separate deployment");
    }
    let mut issuer = configure(&a, llm);
    drop(configure(&b, llm));
    let registration = document(&a)["engine"]["llm"]["providers"][0]["sources"]["sign_in"].clone();
    let mut second = document(&b);
    second["engine"]["llm"]["providers"][0]["sources"]["sign_in"] = registration.clone();
    super::write_private(&b.join("temper.json"), &serde_json::to_vec(&second).expect("B's configuration"));
    let address: SocketAddr = registration["issuer"]
        .as_str()
        .expect("issuer")
        .strip_prefix("http://")
        .expect("local issuer")
        .parse()
        .expect("issuer address");
    // A spare success makes stale-token refusal an issuer validation check.
    for _ in 0..2 {
        issuer.queue(rotation()).expect("rotation plan");
    }
    std::thread::scope(|scope| {
        let (stop, receiver) = mpsc::channel();
        let (ready, listening) = mpsc::channel();
        let peers = scope.spawn(move || {
            let clock = Clock::new();
            let mut world = real::World::new(Peers {
                failed: None,
                stop: receiver,
                ended: false,
                deadline: clock.now().now.saturating_add(WITHIN),
                wake: clock.now().now,
            });
            world.spawn(|| Process::Issuer(Box::new(issuer)));
            peer.spawn(&mut world);
            ready.send(()).expect("peer world ready");
            let outcome = world.run(&clock, WITHIN);
            assert!(outcome.procs.iter().all(Host::is_empty), "peer io drained");
            let [Process::Issuer(issuer), Process::Peer(peer)] = outcome.procs.as_slice() else { panic!("peers") };
            assert_peers(issuer, peer);
        });
        listening.recv().expect("peers constructed");
        let file = directory.join("account.move");
        let (number, record) = move_account(&a, &b, &file);
        let seen = ctl::run(ctl::Story::new(
            &b,
            vec![
                ctl::Step::Run(ctl::Call::sign_in()),
                ctl::Step::Run(ctl::Call::new(&["account", "list", "--json"])),
                ctl::Step::Run(ctl::Call::new(&[
                    "chat",
                    "start",
                    "--project",
                    "1",
                    "--model",
                    "gpt-6-luna",
                    "--key",
                    "moved-chat",
                    "Hello",
                    "--json",
                ])),
                ctl::Step::Run(ctl::Call::new(&["task", "show", "1", "--until", "replied", "--json"])),
                // Stop holds the parked chat immediately, without a closing grace turn.
                ctl::Step::Run(ctl::Call::new(&["task", "stop", "1", "--json"])),
                ctl::Step::Run(ctl::Call::new(&["task", "show", "1", "--until", "held", "--json"])),
            ],
        ));
        for ended in &seen.calls {
            success(ended).expect("successful call");
        }
        assert_eq!(
            account(&seen.calls[1]).expect("imported account")["number"],
            number,
            "B kept the imported account after restart"
        );
        assert_eq!(ctl::json(&seen.calls[3])["met"], true, "the moved account answered");
        refuse_file_token(address, registration["client"].as_str().expect("client id"), &record);
        assert!(!file.exists(), "the imported file stays gone");
        drop(stop);
        peers.join().expect("independent peers");
    });
}
