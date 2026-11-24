//! Program checks for check and startup (shell/commands.md, 4, and shell/settings.md, 1.3;
//! protocol/programs.md, 3). Keeps redacted states, never a program's login.
//! `check` asks sequentially, with the runner's deadline and service environment.
use super::{Credential, Directory, program_source};
use crate::settings::resolve::Resolved;
use skein_shell::Clock;
use smith_agent_shell::config::ProgramName;
use smith_host_domain::ProgramCheck;
use smith_host_protocol::{SourceAnswer, SourceAsk};
use std::collections::BTreeMap;

pub(crate) type ProgramStates = BTreeMap<String, Credential>;

/// The program's typed status or a shell failure before it could supply one.
pub(crate) type Inspection = Result<ProgramCheck, String>;

fn check<'a>(resolved: &Resolved, accounts: impl Iterator<Item = &'a str>, clock: &Clock) -> ProgramStates {
    accounts
        .map(|account| (account.to_owned(), state(ProgramName::ClaudeCode, inspect(resolved, account, clock))))
        .collect()
}

pub(crate) fn needed(resolved: &Resolved, clock: &Clock) -> ProgramStates {
    check(
        resolved,
        resolved
            .accounts
            .iter()
            .filter(|account| account.source() == crate::settings::resolve::AccountSource::Program)
            .map(|account| account.name.as_str()),
        clock,
    )
}

/// Inspect the source without creating its directory or changing its login.
pub(crate) fn inspect(resolved: &Resolved, account: &str, clock: &Clock) -> Inspection {
    let Some(directory) = Directory::of(&resolved.settings) else {
        return Err("an absolute credentials directory is required".into());
    };
    // Inspect both levels without creating or following a source directory.
    for path in [directory.programs(), directory.program(account)] {
        match smith_agent_shell::program::private_directory(&path) {
            Ok(false) => return Ok(ProgramCheck::NotSignedIn),
            Ok(true) => {}
            Err(why) => return Err(why.to_string()),
        }
    }
    let settings = &resolved.settings;
    let configured = settings.values.get("programs.claude_code").and_then(|field| field.value.as_str());
    let Ok(executable) = crate::settings::resolve::programs::locate(configured, settings.paths.path.as_deref()) else {
        return Err("program is missing".into());
    };
    match program_source::run(
        SourceAsk::Check,
        &executable,
        account,
        &directory,
        clock,
        &resolved.agent.service.environment.command_entries(),
    )? {
        SourceAnswer::Check(answer) => Ok(answer),
        SourceAnswer::SignOut(_) => unreachable!("check answers check"),
    }
}

/// Project one inspection to a redacted state shared by check and startup.
pub(crate) fn state(program: ProgramName, answer: Inspection) -> Credential {
    match answer {
        Ok(answer) => credential(program, answer),
        Err(why) => Credential::ProgramFailed { program, detail: program_source::detail(why.as_bytes()) },
    }
}

pub(crate) fn credential(program: ProgramName, answer: ProgramCheck) -> Credential {
    match answer {
        ProgramCheck::Usable { .. } => Credential::Usable { until: None },
        ProgramCheck::NotSignedIn => Credential::ProgramNotSignedIn { program },
        ProgramCheck::SignedInOtherwise(method) => {
            Credential::ProgramSignedInOtherwise { program, method: program_source::detail(method.as_bytes()) }
        }
        ProgramCheck::Failed(failure) => {
            Credential::ProgramFailed { program, detail: program_source::detail(&failure.detail) }
        }
    }
}

pub(crate) const fn key(program: ProgramName) -> &'static str {
    match program {
        ProgramName::ClaudeCode => "claude_code",
    }
}

pub(super) const fn shown(program: ProgramName) -> &'static str {
    match program {
        ProgramName::ClaudeCode => "Claude Code",
    }
}

pub(super) fn serialize<S: serde::Serializer>(
    program: impl std::borrow::Borrow<ProgramName>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(key(*program.borrow()))
}

#[cfg(test)]
mod tests;
