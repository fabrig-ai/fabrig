//! LLM account sign-in against the shipped process and the existing independent issuer/browser.
use crate::support::{Person, Process, issuer, private, start};
use jig_client::{Asker, Key, contract as c};
use jig_client_protocol_kit::native::Received;
use jig_client_protocol_kit::native::{NativeClient, Script};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Queue, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::{fs, path::Path};

const ACCOUNT_SIGN_IN: Key = Key { asker: Asker(7), number: 1 };

pub fn configuration(directory: &Path) {
    crate::support::configuration(directory);
    let path = directory.join("config.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["engine"]["llm"]["providers"] = serde_json::json!([{
        "name":"Fake", "signed_in_plan":true, "manage_usage":"https://usage.example/", "number":3, "models":["fake"], "sources":{"sign_in":{
            "issuer":"http://127.0.0.1:0", "client":"client", "scopes":["openid","read"],
            "name_hint":"jig-test", "authorization":"http://127.0.0.1:0/authorize",
            "token":"http://127.0.0.1:0/token", "revocation":"http://127.0.0.1:0/revoke",
            "keys":"http://127.0.0.1:0/keys", "transport":"plaintext",
            "must_grant":["openid","read"], "resource":"api", "redirect_path":"callback"
        }}
    }]);
    private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
}
fn client(procs: &mut [Process]) -> Result<&mut NativeClient, String> {
    let Process::Person(person) = procs.get_mut(1).ok_or("native person")? else {
        return Err(String::from("native person"));
    };
    person.client.as_mut().ok_or_else(|| String::from("native client"))
}
fn answer(procs: &[Process]) -> Result<Option<c::Answer>, String> {
    let Process::Person(person) = procs.get(1).ok_or("native person")? else {
        return Err(String::from("native person"));
    };
    let Some(native) = &person.client else { return Ok(None) };
    Ok(native.received.iter().rev().find_map(|event| match event {
        Received::Answer { answer, .. } => Some(answer.clone()),
        Received::SignedIn { .. } | Received::Sent { .. } | Received::Delivery(_) | Received::Read(_) => None,
    }))
}
fn run_world<R: Referee<Process>>(
    directory: &Path,
    judge: R,
    clock: &Clock,
    patience: Duration,
) -> real::Outcome<Process> {
    let (issuer, address, fd) = issuer();
    crate::support::ports::configure(directory, Some(address), None);
    let mut binary = start(directory);
    let streams = binary.take_streams().expect("binary pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let client_directory = tempfile::tempdir_in(directory).expect("client directory");
    let root = skein_shell::open_root(client_directory.path()).expect("client root");
    let mut world = real::World::new(judge);
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds.into_iter().chain([root]).collect(), || {
        Process::Person(Box::new(Person {
            seed: crate::support::CLIENT_SEED,
            client: None,
            root: Some(root),
            script: Some(Script::account_sign_in()),
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
    world.spawn_with_fds(vec![fd], || Process::Issuer(Box::new(issuer)));
    world.spawn(|| Process::Visitor(Box::default()));
    world.run(clock, patience)
}

// Shutdown is bounded by the focused world's shared Deadline.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(10);
const SETUP_WAIT: Duration = Duration::from_secs(20);
const WORLD_WAIT: Duration = Duration::from_secs(40);

struct Shutdown {
    finding: Option<String>,
    asked: bool,
    sent: Option<Time>,
    killed: bool,
    exit: Option<Exit>,
    deadline: Time,
}
impl Referee<Process> for Shutdown {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.finding.is_none() {
            self.finding = self.check_act(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, procs: &[Process]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        let Process::Visitor(visitor) = procs.get(3).ok_or("browser")? else { return Err(String::from("browser")) };
        if visitor.handed() {
            return Err(String::from("no authorization visit"));
        }
        let Process::Binary(binary) = procs.first().ok_or("binary")? else { return Err(String::from("binary")) };
        if binary.exit_status().is_some() && self.sent.is_none() {
            return Err(String::from("binary exited before SIGTERM"));
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.exit.is_some() {
            None
        } else if let Some(sent) = self.sent {
            Some(if self.killed { self.deadline } else { sent.saturating_add(SHUTDOWN_WAIT) })
        } else {
            Some(self.deadline)
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.exit.is_none() && now >= self.deadline).then(|| "SIGTERM process setup or cleanup did not settle".into())
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.exit.is_some()
    }
}

/// Leave the authorization address unvisited and stop the owned binary.
pub fn sigterm() {
    let directory = crate::support::scratch();
    configuration(directory.path());
    let clock = Clock::new();
    let judge = Shutdown {
        finding: None,
        asked: false,
        sent: None,
        killed: false,
        exit: None,
        deadline: clock.now().now.saturating_add(SETUP_WAIT),
    };
    let outcome = run_world(directory.path(), judge, &clock, WORLD_WAIT);
    assert!(outcome.procs.iter().all(Host::is_empty), "all SIGTERM test io settles");
    let Process::Binary(binary) = &outcome.procs[0] else { panic!("binary") };
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "SIGTERM must end without SIGKILL");
}

impl Shutdown {
    fn check_act(&mut self, now: Time, procs: &mut [Process]) -> Result<(), String> {
        if self.exit.is_some() {
            return Ok(());
        }
        if let Some(sent) = self.sent {
            let Process::Binary(binary) = procs.get_mut(0).ok_or("binary")? else { return Err(String::from("binary")) };
            if let Some(status) = binary.exit_status() {
                let elapsed = now.saturating_since(sent);
                eprintln!("SIGTERM elapsed={elapsed:?} killed={} status={status:?}", self.killed);
                let Process::Issuer(issuer) = procs.get_mut(2).ok_or("issuer")? else {
                    return Err(String::from("issuer"));
                };
                // Keep the fake alive until the binary exits; it cannot help teardown.
                issuer.shutdown();
                self.exit = Some(status);
            } else if !self.killed && now >= sent.saturating_add(SHUTDOWN_WAIT) {
                eprintln!("SIGTERM still running after {SHUTDOWN_WAIT:?}; killing owned child");
                binary.signal(Signal::Kill);
                self.killed = true;
            }
            return Ok(());
        }
        let Process::Person(person) = procs.get_mut(1).ok_or("native person")? else {
            return Err(String::from("native person"));
        };
        let Some(native) = &mut person.client else { return Ok(()) };
        if native.signed_in != 1 || !native.account_sign_in_ready() {
            return Ok(());
        }
        if !self.asked {
            native
                .ask_account_sign_in(ACCOUNT_SIGN_IN, c::Ask::SignIn { holder: c::AskAccountHolder::Own, provider: 3 });
            self.asked = true;
            return Ok(());
        }
        {
            let Some(answer) = answer(procs)? else { return Ok(()) };
            let c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::AccountSignInStarted { address, .. })) =
                answer
            else {
                return Err(format!("account sign-in answer: {answer:?}"));
            };
            if address.is_empty() {
                return Err(String::from("the pending sign-in has its authorization address"));
            }
        }
        // Deliberately never hand the address to Visitor.
        client(procs)?.finish_account_sign_in();
        let Process::Binary(binary) = procs.get_mut(0).ok_or("binary")? else { return Err(String::from("binary")) };
        binary.signal(Signal::Terminate);
        self.sent = Some(now);

        Ok(())
    }
}
