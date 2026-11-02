//! Drive the account owner's terminals through close (protocol/hosts.md, 5.4).
//! Subject comparisons emit account names only; no subject enters diagnostics.
use super::super::super::{registration, sign_in};
use super::super::LoginOutcome;
use skein_io::kernel;
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use smith_local_service::{Accounts, CredentialRequest, Settled};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum State {
    SignIn,
    /// A Visit was shown; retain its deadline to distinguish earlier exchange failures.
    Waiting {
        deadline: Time,
    },
    SignOut,
    Rollback,
}

pub(super) struct Login {
    pub driver: Accounts,
    pub name: String,
    pub outcome: Option<LoginOutcome>,
    pub state: State,
    pub closing: bool,
    pub notice: bool,
    pub others: Vec<(String, std::path::PathBuf)>,
    pub ssh: Option<String>,
    pub operations: u32,
    pub out: Queue<CredentialRequest>,
    pub completions: Queue<kernel::Complete>,
}

impl Login {
    fn signed_in(&mut self, subject: Option<Box<[u8]>>, missing: &[u8]) {
        if let Some(subject) = subject {
            for (name, file) in &self.others {
                match sign_in::record(file) {
                    Ok(Some(record)) if record.subject.as_deref() == Some(subject.as_ref()) => {
                        eprintln!("smith: account {}: already signed in as account {name}", self.name);
                        self.state = State::Rollback;
                        self.driver.sign_out(0);
                        return;
                    }
                    Err(_) => {
                        eprintln!("smith: account {name}: unreadable sign-in record");
                        self.state = State::Rollback;
                        self.driver.sign_out(0);
                        return;
                    }
                    Ok(Some(_) | None) => {}
                }
            }
        }
        if missing.is_empty() {
            if self.notice {
                eprintln!("{}", registration::NOTICE);
            }
            eprintln!("account {}: signed in", self.name);
            self.outcome = Some(LoginOutcome::Usable);
        } else {
            eprintln!("Signed in without plan usage: run `smith login {}` again and allow it, or use a key", self.name);
            self.outcome = Some(LoginOutcome::Unavailable);
        }
    }

    fn visit(&mut self, url: &[u8]) {
        let deadline = self.driver.next_deadline().expect("a Visit keeps the sign-in deadline");
        self.state = State::Waiting { deadline };
        eprintln!("{}", visit_lines(url, self.ssh.as_deref()));
    }

    fn terminal(&mut self, terminal: Settled, now: Time) {
        match terminal {
            Settled::SignedIn { subject, missing } => self.signed_in(subject, &missing),
            Settled::SignedOut { revoked } => {
                if !revoked {
                    eprintln!("account {}: disconnect smith at {}", self.name, registration::DISCONNECT);
                }
                eprintln!("account {}: signed out", self.name);
                self.outcome =
                    Some(if self.state == State::Rollback { LoginOutcome::Refused } else { LoginOutcome::Usable });
            }
            Settled::Failed { failure, .. } => {
                let (line, refused) = failure_line(&self.name, self.state, failure, now);
                eprintln!("{line}");
                self.outcome = Some(if refused || self.state == State::Rollback {
                    LoginOutcome::Refused
                } else {
                    LoginOutcome::Unavailable
                });
            }
        }
    }
}

impl Host for Login {
    fn iterate(&mut self, now: Time, wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.driver.complete(complete);
        }
        self.driver.advance(now, wall, &mut self.out);
        while let Some(request) = self.out.pop() {
            match request {
                CredentialRequest::Visit { url } => self.visit(&url),
                CredentialRequest::SignedIn { .. }
                | CredentialRequest::Failed { .. }
                | CredentialRequest::Closed
                | CredentialRequest::Notice { .. }
                | CredentialRequest::Ready { .. } => {}
            }
        }
        while let Some((_, terminal)) = self.driver.settled() {
            self.terminal(terminal, now);
        }
        if self.outcome.is_some() && !self.closing {
            self.driver.cancel(0);
            self.driver.close(smith_agent_service::Close::Graceful, now, wall);
            self.closing = true;
        }
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        self.driver.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.driver.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.driver.next_deadline()
    }
    fn is_empty(&self) -> bool {
        self.outcome.is_some() && self.driver.closed()
    }
    fn operations(&self) -> u32 {
        self.operations
    }
    fn exit(&self) -> Option<kernel::Exit> {
        self.outcome.map(|outcome| kernel::Exit::Code(outcome.code()))
    }
}

fn sign_in_minutes() -> u64 {
    super::super::super::product::client_limits()
        .sign_in_time
        .as_nanos()
        .div_euclid(skein_lib::Duration::from_secs(60).as_nanos())
}

fn visit_lines(url: &[u8], ssh: Option<&str>) -> String {
    let forwarding = if let Some(host) = ssh
        && let Some(port) = redirect_port(url)
    {
        format!("ssh -L {port}:127.0.0.1:{port} {host}\n")
    } else {
        String::new()
    };
    format!(
        "Sign in: {}\n{forwarding}Waiting up to {} minutes for the browser.",
        String::from_utf8_lossy(url),
        sign_in_minutes()
    )
}

fn browser_deadline_passed(state: State, now: Time) -> bool {
    match state {
        State::Waiting { deadline } => now >= deadline,
        State::SignIn | State::SignOut | State::Rollback => false,
    }
}

fn failure_line(name: &str, state: State, failure: skein_oauth_accounts::Failure, now: Time) -> (String, bool) {
    use skein_oauth_accounts::Failure;
    match failure {
        Failure::Exchange(skein_oauth::Failure::ClientRejected) => {
            (format!("smith: account {name}: the provider refused smith's registration"), true)
        }
        Failure::AnotherSubject => (format!("smith: account {name}: another subject was refused"), true),
        Failure::Exchange(skein_oauth::Failure::TimedOut) if browser_deadline_passed(state, now) => (
            format!("account {name}: not signed in: no answer from the browser within {} minutes", sign_in_minutes()),
            false,
        ),
        Failure::Busy
        | Failure::NotGranted
        | Failure::NotKept
        | Failure::Expired
        | Failure::Unloaded { .. }
        | Failure::Exchange(_) => (
            format!("account {name}: {}", if state == State::SignOut { "not signed out" } else { "not signed in" }),
            false,
        ),
    }
}

fn redirect_port(url: &[u8]) -> Option<u16> {
    let url = std::str::from_utf8(url).ok()?;
    let redirect = url.split(['?', '&']).find_map(|field| field.strip_prefix("redirect_uri="))?;
    let port = redirect.split("%3A").nth(2)?.split("%2F").next()?;
    port.parse().ok()
}

#[cfg(test)]
mod tests;
