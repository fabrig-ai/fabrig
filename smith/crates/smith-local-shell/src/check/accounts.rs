//! Check's account projection (shell/commands.md, 4; protocol/host.md, 5.2).
//! `program_accounts` names the inspections to make; `report` projects their
//! answers to redacted states. Neither starts a program or a sign-in.
use crate::{credentials, credentials::program_check, settings::resolve::Resolved};
use skein_lib::Wall;
use smith_agent_shell::config::ProgramName;
use std::collections::BTreeMap;

pub(super) type ProgramAnswers = BTreeMap<String, program_check::Inspection>;

pub(super) fn program_accounts(resolved: &Resolved) -> Vec<String> {
    credentials::account_report::accounts(resolved)
        .into_values()
        .filter(|account| account.source() == crate::settings::resolve::AccountSource::Program)
        .map(|account| account.name)
        .collect()
}

pub(super) fn report(
    resolved: &Resolved,
    wall: Wall,
    answers: ProgramAnswers,
) -> BTreeMap<String, credentials::AccountReport> {
    let mut states = program_check::ProgramStates::new();
    for (account, answer) in answers {
        states.insert(account, program_check::state(ProgramName::ClaudeCode, answer));
    }
    credentials::report(resolved, wall, &states)
}
