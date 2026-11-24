//! Source-specific removal (shell/commands.md, 6). Holds no credential values.
use super::{Account, Environment, LoginOutcome, Source};
use crate::settings::Settings;
use skein_shell::Clock;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
enum Decision {
    Revoke(PathBuf),
    Program(String),
    Local { remove: Option<PathBuf>, line: String, outcome: LoginOutcome },
}

fn decide(name: &str, source: Source) -> Decision {
    match source {
        Source::SignIn { file, .. } => Decision::Revoke(file),
        Source::Program { name } => Decision::Program(name),
        Source::Key { file } => Decision::Local {
            remove: Some(file),
            line: format!("account {name}: key removed"),
            outcome: LoginOutcome::Usable,
        },
        Source::Env { .. } | Source::Nothing => Decision::Local {
            remove: None,
            line: format!("account {name}: smith holds nothing"),
            outcome: LoginOutcome::Usable,
        },
    }
}

pub(super) fn run(
    account: &Account,
    source: Source,
    settings: &Settings,
    clock: &Clock,
    environment: &Environment,
) -> Result<LoginOutcome, String> {
    match decide(&account.name, source) {
        Decision::Revoke(file) => super::login::provider::run(
            account,
            &file,
            settings,
            super::login::provider::Operation::SignOut,
            clock,
            environment,
        ),
        Decision::Program(name) => super::login::program::run(&name, settings, true, clock, environment),
        Decision::Local { remove, line, outcome } => {
            if let Some(file) = remove {
                remove_key(&file)?;
            }
            eprintln!("{line}");
            Ok(outcome)
        }
    }
}

pub(super) fn remove_key(file: &std::path::Path) -> Result<(), String> {
    match std::fs::remove_file(file) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("cannot remove private key".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_logout_removes_only_a_kept_key() {
        let file = PathBuf::from("/credentials/keys/work");
        for (name, source, removed, words) in [
            ("work", Source::Key { file: file.clone() }, Some(file), "account work: key removed"),
            ("empty", Source::Nothing, None, "account empty: smith holds nothing"),
            ("variable", Source::Env { variable: "WORK_KEY".into() }, None, "account variable: smith holds nothing"),
        ] {
            let Decision::Local { remove, line, outcome } = decide(name, source) else {
                unreachable!("{name}: no provider or program operation");
            };
            assert_eq!(remove, removed, "{name}: file to remove");
            assert_eq!(line, words, "{name}: only the account name and disposition, never a credential");
            assert_eq!(outcome.code(), 0, "{name}: sign-out is complete");
        }
    }
}
