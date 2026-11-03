//! Source-specific credential commands (shell.md, sections 4.5–4.6).
//! The edge supplies the environment; dispatch keeps names and redacted outcomes.
use super::{Directory, Source};
use crate::{command::Command, settings};
use skein_shell::Clock;

mod key;
pub(super) mod program;
pub(super) mod provider;

/// The environment snapshot supplied by main, never rendered with its values.
#[derive(Clone, Default)]
pub struct Environment {
    pub variables: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Environment")
    }
}

/// The named source's terminal after one credential command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginOutcome {
    /// The requested credential state is ready.
    Usable,
    /// The requested credential state could not be reached.
    Unavailable,
    /// A declaration or account identity was refused.
    Refused,
}

impl LoginOutcome {
    /// The product's credential command exit code.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Usable => 0,
            Self::Unavailable => 3,
            Self::Refused => 2,
        }
    }
}

/// Handle login or logout before the run and check front ends.
#[must_use]
pub fn command(arguments: &[std::ffi::OsString], clock: &Clock, environment: &Environment) -> Option<u8> {
    let (shared, name, logout) = match crate::command::parse(arguments.iter().skip(1).cloned().collect()) {
        Ok(Command::Login { shared, account }) => (shared, account, false),
        Ok(Command::Logout { shared, account }) => (shared, account, true),
        Ok(Command::Interactive { .. } | Command::Check { .. } | Command::Exec(_)) | Err(_) => return None,
    };
    Some(match run(&shared, &name, logout, clock, environment) {
        Ok(outcome) => outcome.code(),
        Err(why) => {
            eprintln!("smith: {why}");
            2
        }
    })
}

fn run(
    shared: &crate::command::Shared,
    name: &str,
    logout: bool,
    clock: &Clock,
    environment: &Environment,
) -> Result<LoginOutcome, String> {
    let settings = settings::read_in(shared, &environment.variables).map_err(|why| why.line())?;
    let settings = settings::resolve::checked(&settings).map_err(|why| why.line())?;
    let account = settings::accounts(&settings).remove(name).ok_or_else(|| format!("unknown account {name}"))?;
    if !account.fields.contains_key("source") {
        return Err(format!("unknown account {name}"));
    }
    let directory = Directory::of(&settings);
    let source = super::source(&account, directory.as_ref()).map_err(|credential| credential.line(name))?;
    if logout {
        return super::logout::run(&account, &source, &settings, clock, environment);
    }
    match source {
        Source::SignIn { file, .. } => {
            provider::run(&account, &file, &settings, provider::Operation::SignIn, clock, environment)
        }
        Source::Key { file } => key::run(name, &file),
        Source::Program { .. } => program::run(name, &settings, false, clock, environment),
        Source::Nothing => {
            eprintln!("account {name}: needs nothing");
            Ok(LoginOutcome::Usable)
        }
        Source::Env { variable } => {
            let wall = clock.now().wall;
            let borrowed = environment
                .variables
                .iter()
                .find(|(key, _)| key.as_os_str() == std::ffi::OsStr::new(&variable))
                .and_then(|(_, value)| super::env_record(value.as_encoded_bytes(), wall));
            let credential = super::env_state(&variable, borrowed.as_ref());
            eprintln!("{}", credential.line(name));
            Ok(if credential.usable() { LoginOutcome::Usable } else { LoginOutcome::Unavailable })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{command::Shared, credentials::tests::Files};

    fn invocation(files: &Files, declarations: &[&str]) -> Shared {
        Shared {
            directory: Some(files.path().to_path_buf()),
            overrides: std::iter::once(format!("credentials='{}'", files.path().join("unopened").display()))
                .chain(declarations.iter().map(|declaration| (*declaration).into()))
                .collect(),
            ..Shared::default()
        }
    }

    #[test]
    fn an_unknown_account_is_refused_without_creating_credentials() {
        let files = Files::new("smith-credentials");
        let shared = invocation(&files, &["accounts.work.source='none'"]);
        let error =
            run(&shared, "missing", false, &Clock::new(), &Environment::default()).expect_err("unknown account");
        assert_eq!(error, "unknown account missing", "lookup diagnostic");
        let mut arguments: Vec<std::ffi::OsString> =
            vec!["smith".into(), "login".into(), "missing".into(), "-C".into(), files.path().to_path_buf().into()];
        for declaration in &shared.overrides {
            arguments.extend(["-c".into(), declaration.into()]);
        }
        assert_eq!(
            command(&arguments, &Clock::new(), &Environment::default()),
            Some(2),
            "credential command refusal code"
        );
        assert!(!files.path().join("unopened").exists(), "lookup starts no credential store");
    }

    #[test]
    fn account_only_login_needs_no_main_model() {
        let files = Files::new("smith-credentials");
        let shared = invocation(
            &files,
            &["accounts.work.source='none'", "endpoints.custom.preset='openai'", "endpoints.custom.account='work'"],
        );
        let outcome = run(&shared, "work", false, &Clock::new(), &Environment::default()).expect("account command");
        assert_eq!(outcome, LoginOutcome::Usable, "account selection does not select a run model");
        assert!(!files.path().join("unopened").exists(), "a none source starts no credential store");
    }
}
