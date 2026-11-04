//! Component tests of the credential owner and the login terminal projection.
//! The existing issuer/browser fakes meet Accounts over the simulated kernel;
//! no domain world or shipped process is added (testing-strategy.md, 2.1).
use super::*;
use crate::credentials::{self, tests::Files};
use skein_fake_machine::{How, Item, Machine, Opened};
use skein_lib::{Duration, List};
use smith_local_process_world::process::{BrowserProcess, IssuerProcess};

const SUBJECT: &[u8] = b"person-for-work";
const OTHER: &[u8] = b"person-for-home";
const FULL_SCOPE: &[u8] = b"openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";
// The bounded fixture needs at most the service test harness's 4000 iterations.
const STEPS: usize = 4000;
const SEED: u64 = 61;

fn saved(subject: &[u8]) -> Vec<u8> {
    skein_oauth::encode_record(
        &skein_oauth::SavedToken {
            key: 0,
            generation: 1,
            access_token: b"access-for-work".as_slice().into(),
            refresh_token: Some(b"refresh-for-work".as_slice().into()),
            issued_client_id: Some(b"oaiapp_for_work".as_slice().into()),
            granted_scopes: Some(FULL_SCOPE.into()),
            subject: Some(subject.into()),
            id_token: Some(b"prior-id-for-work".as_slice().into()),
            earliest_refresh_at: None,
            metadata: None,
            expires_at: Wall::from_nanos(u64::MAX),
        },
        &sign_in::document_limits(),
    )
    .expect("synthetic saved record")
    .into_vec()
}

fn keep(path: &std::path::Path, bytes: &[u8]) {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};
    let mut file =
        std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path).expect("private fixture");
    file.write_all(bytes).expect("synthetic record bytes");
}

struct Exchange {
    login: Login,
    sim: skein_sim::Sim,
    pid: skein_sim::Pid,
    issuer_pid: skein_sim::Pid,
    browser_pid: skein_sim::Pid,
    issuer: IssuerProcess,
    browser: BrowserProcess,
    files: Machine,
    root: Opened,
    arriving: Option<Vec<u8>>,
    urls: Vec<Vec<u8>>,
}

impl Exchange {
    fn new(prior: Option<&[u8]>, subject: &[u8], plan: bool, revocation: Option<bool>) -> Self {
        let (_, service) = smith_local_process_world::process::configuration(kernel::Fd::new(70), false);
        let limits = credentials::limits(service.limits.io, 1, Duration::from_secs(60));
        let mut files = Machine::for_user(7);
        let mut items = vec![Item::directory(b"tokens").mode(0o700)];
        if let Some(prior) = prior {
            items.push(Item::file(b"tokens/work", prior).mode(0o600));
        }
        let root = files.lay(&items);
        let mut sim = skein_sim::Sim::new(SEED, skein_sim::Config::calm());
        let pid = sim.spawn_process();
        let issuer_pid = sim.spawn_process();
        let browser_pid = sim.spawn_process();
        let owned_root = files.open(root, b".", How::Directory).expect("source root");
        let descriptor = sim.root(pid, skein_sim::Handle::new(owned_root.raw()));
        let origin = "http://127.0.0.1:444";
        let moved = registration::Moved {
            issuer: Some(origin.into()),
            authorization_url: Some(format!("{origin}/authorize")),
            token_url: Some(format!("{origin}/token")),
            keys_url: Some(format!("{origin}/keys")),
            revocation_url: Some(format!("{origin}/revoke")),
        };
        let mut sources = List::with_capacity(1);
        sources
            .push(smith_local_service::Source::SignIn {
                account: 0,
                root: descriptor,
                user: 7,
                directory: b"tokens".as_slice().into(),
                file: b"work".as_slice().into(),
                registration: registration::openai("host-for-tests", Some(2345), &moved),
                endpoint: skein_oauth_accounts::Endpoint {
                    address: ([127, 0, 0, 1], 444).into(),
                    transport: skein_oauth_accounts::Transport::Plaintext,
                },
                interactive: false,
            })
            .ok()
            .expect("one source");
        let mut driver = Accounts::shared(sources, limits, [61; 32]).expect("component driver");
        if revocation.is_some() {
            driver.sign_out(0);
        } else {
            let _started = driver.sign_in(0);
        }
        Self {
            login: Login {
                driver,
                name: "work".into(),
                outcome: None,
                state: if revocation.is_some() { State::SignOut } else { State::SignIn },
                closing: false,
                notice: false,
                others: Vec::new(),
                ssh: None,
                operations: 128,
                out: Queue::uncapped(),
                completions: Queue::with_capacity(64),
            },
            sim,
            pid,
            issuer_pid,
            browser_pid,
            issuer: IssuerProcess::provider(444, subject, plan, prior.is_some(), revocation),
            browser: BrowserProcess::new(),
            files,
            root,
            arriving: None,
            urls: Vec::new(),
        }
    }

    fn record(&mut self) -> Option<Vec<u8>> {
        let file = self.files.open(self.root, b"tokens/work", How::Read).ok()?;
        let bytes = self.files.read(file, 0, sign_in::document_limits().record_bytes).expect("fake record");
        self.files.close(file);
        Some(bytes)
    }

    fn visits(&mut self) {
        while let Some(request) = self.login.out.pop() {
            if let CredentialRequest::Visit { url } = request {
                if let Some(bytes) = self.arriving.take() {
                    let file = self
                        .files
                        .open(self.root, b"tokens/work", How::Create { mode: 0o600 })
                        .expect("arriving record");
                    self.files.write(file, 0, &bytes).expect("winner bytes");
                    self.files.close(file);
                }
                self.login.visit(&url);
                self.urls.push(url.to_vec());
                self.browser.visit(&url);
            }
        }
    }

    fn step(&mut self) {
        let now = self.sim.now();
        let wall = self.sim.wall();
        self.sim.reap(self.issuer_pid, self.issuer.completions());
        self.issuer.iterate(now, wall);
        self.sim.submit(self.issuer_pid, self.issuer.submissions());
        self.sim.reap(self.browser_pid, self.browser.completions());
        self.browser.iterate(now, wall);
        self.sim.submit(self.browser_pid, self.browser.submissions());
        self.sim.reap(self.pid, &mut self.login.completions);
        while let Some(complete) = self.login.completions.pop() {
            self.login.driver.complete(complete);
        }
        self.login.driver.advance(now, wall, &mut self.login.out);
        self.visits();
        while let Some((_, terminal)) = self.login.driver.settled() {
            self.login.terminal(terminal, now);
        }
        if self.login.outcome.is_some() && !self.login.closing {
            self.login.driver.cancel(0);
            self.login.driver.close(smith_agent_service::Close::Graceful, now, wall);
            self.login.closing = true;
            self.issuer.shutdown();
            self.browser.shutdown();
        }
        self.sim.submit(self.pid, self.login.driver.submissions());
        skein_fake_machine::serve(&mut self.files, &mut self.sim);
        let pending = self.login.driver.work_pending(now)
            || self.issuer.work_pending(now)
            || self.browser.work_pending(now)
            || [self.pid, self.issuer_pid, self.browser_pid]
                .iter()
                .any(|pid| self.sim.ready(*pid) > 0 || self.sim.deferred(*pid));
        if !pending
            && let Some(next) = [
                self.sim.next_due(),
                self.login.driver.next_deadline(),
                self.issuer.next_deadline(),
                self.browser.next_deadline(),
            ]
            .into_iter()
            .flatten()
            .min()
        {
            self.sim.advance_to(next);
        }
    }

    fn finish(&mut self) -> LoginOutcome {
        for _ in 0..STEPS {
            self.step();
            if self.login.driver.closed()
                && self.issuer.is_empty()
                && self.browser.is_empty()
                && [self.pid, self.issuer_pid, self.browser_pid]
                    .iter()
                    .all(|pid| self.sim.in_flight(*pid) == 0 && self.sim.ready(*pid) == 0)
            {
                for pid in [self.pid, self.issuer_pid, self.browser_pid] {
                    self.sim.assert_quiescent(pid);
                    self.sim.assert_no_open_fds(pid);
                }
                return self.login.outcome.expect("closed after the terminal");
            }
        }
        panic!("credential component did not settle from seed {SEED}");
    }

    fn revocations(&self) -> usize {
        self.issuer
            .peer()
            .observations()
            .iter()
            .filter(|observation| matches!(observation, skein_fake_peers::oauth::Observation::Revoke { .. }))
            .count()
    }
}

#[test]
fn a_second_login_reuses_registration_and_subject_hints() {
    let mut exchange = Exchange::new(Some(&saved(SUBJECT)), SUBJECT, true, None);
    assert_eq!(exchange.finish(), LoginOutcome::Usable, "second login");
    let url = String::from_utf8_lossy(&exchange.urls[0]);
    for parameter in ["client_id=oaiapp_for_work", "ext_agent_host_id=host-for-tests", "id_token_hint="] {
        assert!(url.contains(parameter), "authorization carries {parameter}");
    }
    assert!(!url.contains("agent_name_hint"), "an issued registration omits the new-registration hint");
}

#[test]
fn a_replacement_subject_keeps_the_saved_bytes() {
    let prior = saved(SUBJECT);
    let mut exchange = Exchange::new(Some(&prior), OTHER, true, None);
    assert_eq!(exchange.finish(), LoginOutcome::Refused, "another subject is refused");
    assert_eq!(exchange.urls.len(), 1, "one authorization visit");
    assert_eq!(exchange.record(), Some(prior), "saved bytes remain identical");
}

#[test]
fn declined_plan_scope_is_saved_and_reported_unavailable() {
    let files = Files::new("smith-credentials");
    let mut exchange = Exchange::new(None, SUBJECT, false, None);
    assert_eq!(exchange.finish(), LoginOutcome::Unavailable, "declining plan usage maps to exit 3");
    let bytes = exchange.record().expect("the declined-scope record is kept");
    let file = files.path().join("record");
    keep(&file, &bytes);
    let state = sign_in::inspect(&file, Wall::EPOCH, Some(b"chatgpt.tokens.use.direct"));
    assert_eq!(
        serde_json::to_value(&state).expect("state JSON"),
        serde_json::json!({"state":"without_scope","scope":"chatgpt.tokens.use.direct"}),
        "check reports missing plan usage"
    );
    assert!(state.line("work").contains("again and allow it, or use a key"), "actionable scope report");
}

#[test]
fn a_duplicate_subject_rolls_back_only_the_new_account() {
    let files = Files::new("smith-credentials");
    let first = files.path().join("first");
    let prior = saved(SUBJECT);
    keep(&first, &prior);
    let mut exchange = Exchange::new(None, SUBJECT, true, None);
    exchange.login.others.push(("first".into(), first.clone()));
    assert_eq!(exchange.finish(), LoginOutcome::Refused, "duplicate subject refusal");
    assert_eq!(exchange.revocations(), 1, "new grant is revoked once");
    assert!(exchange.record().is_none(), "the new account is removed");
    assert_eq!(std::fs::read(first).expect("first account"), prior, "the existing account remains unchanged");
}

#[test]
fn unconfirmed_revocation_still_removes_the_record() {
    let mut exchange = Exchange::new(Some(&saved(SUBJECT)), SUBJECT, true, Some(false));
    assert_eq!(exchange.finish(), LoginOutcome::Usable, "unconfirmed revocation does not refuse local sign-out");
    assert_eq!(exchange.revocations(), 1, "one revocation request");
    assert!(exchange.record().is_none(), "local sign-out removes the record");
}

#[test]
fn an_arriving_record_wins_before_the_login_save() {
    let winner = saved(OTHER);
    let mut exchange = Exchange::new(None, SUBJECT, true, None);
    exchange.arriving = Some(winner.clone());
    assert_eq!(exchange.finish(), LoginOutcome::Refused, "another subject arrived before save");
    assert_eq!(exchange.urls.len(), 1, "one authorization visit");
    assert_eq!(exchange.record(), Some(winner), "the winner stays byte-for-byte");
}

#[test]
fn a_visit_names_the_wait_from_the_sign_in_limit() {
    let limit = credentials::product::client_limits().sign_in_time;
    assert_eq!(limit, Duration::from_secs(600), "the product allows ten minutes");
    let minutes = limit.as_nanos().div_euclid(Duration::from_secs(60).as_nanos());
    let url = b"http://127.0.0.1/authorize?redirect_uri=http%3A%2F%2F127.0.0.1%3A2345%2Fcallback";
    assert_eq!(
        visit_lines(url, None),
        format!("Sign in: {}\nWaiting up to {minutes} minutes for the browser.", String::from_utf8_lossy(url)),
        "a local visit states the configured wait"
    );
    assert_eq!(
        visit_lines(url, Some("login-host")),
        format!(
            "Sign in: {}\nssh -L 2345:127.0.0.1:2345 login-host\nWaiting up to {minutes} minutes for the browser.",
            String::from_utf8_lossy(url)
        ),
        "the waiting line follows the forwarding line"
    );
}

#[test]
fn a_sign_in_deadline_names_the_browser_wait() {
    let failure = skein_oauth_accounts::Failure::Exchange(skein_oauth::Failure::TimedOut);
    let deadline = Time::ZERO.saturating_add(credentials::product::client_limits().sign_in_time);
    assert_eq!(
        failure_line("work", State::Waiting { deadline }, failure, deadline),
        ("account work: not signed in: no answer from the browser within 10 minutes".into(), false),
        "the deadline explains the unavailable account"
    );
    let mut exchange = Exchange::new(None, SUBJECT, true, None);
    for _ in 0..STEPS {
        exchange.step();
        if !exchange.urls.is_empty() {
            break;
        }
    }
    assert_eq!(exchange.urls.len(), 1, "the listener shows its authorization address");
    let State::Waiting { deadline } = exchange.login.state else {
        panic!("the Visit records the browser deadline");
    };
    assert_eq!(
        deadline,
        Time::ZERO.saturating_add(credentials::product::client_limits().sign_in_time),
        "the Visit retains the sign-in deadline rather than a shorter I/O deadline"
    );
    exchange.sim.advance_to(deadline);
    assert_eq!(exchange.finish(), LoginOutcome::Unavailable, "the browser misses its deadline");
    assert_eq!(exchange.login.outcome, Some(LoginOutcome::Unavailable), "deadline keeps exit 3");
}

#[test]
fn other_credential_failures_keep_their_words() {
    use skein_oauth_accounts::Failure;
    for failure in [
        Failure::Busy,
        Failure::NotGranted,
        Failure::NotKept,
        Failure::Expired,
        Failure::Exchange(skein_oauth::Failure::Refused),
    ] {
        assert_eq!(
            failure_line("work", State::SignIn, failure, Time::ZERO),
            ("account work: not signed in".into(), false),
            "other failures keep the sign-in diagnostic"
        );
    }
    assert_eq!(
        failure_line("work", State::SignOut, Failure::Exchange(skein_oauth::Failure::TimedOut), Time::ZERO),
        ("account work: not signed out".into(), false),
        "a sign-out timeout does not name the browser"
    );
    assert_eq!(
        failure_line("work", State::SignIn, Failure::Exchange(skein_oauth::Failure::ClientRejected), Time::ZERO),
        ("smith: account work: the provider refused smith's registration".into(), true),
        "registration refusal keeps its diagnostic and exit classification"
    );
}

#[test]
fn an_exchange_timeout_before_the_browser_deadline_keeps_its_words() {
    let deadline = Time::ZERO.saturating_add(credentials::product::client_limits().sign_in_time);
    let state = State::Waiting { deadline };
    let failure = skein_oauth_accounts::Failure::Exchange(skein_oauth::Failure::TimedOut);
    let before = Time::from_nanos(deadline.as_nanos().checked_sub(1).expect("positive deadline"));
    assert_eq!(
        failure_line("work", state, failure, before),
        ("account work: not signed in".into(), false),
        "an earlier exchange timeout does not claim the browser wait elapsed"
    );
}
