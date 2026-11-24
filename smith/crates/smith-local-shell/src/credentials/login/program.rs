//! Hand the terminal to program sign-in, then ask the host half (shell/commands.md, 5–6).
use super::super::{Directory, program_source};
use super::{Environment, LoginOutcome};
use crate::settings::Settings;
use program_source::detail;
use skein_shell::Clock;
use smith_host_domain::{ProgramCheck, ProgramSignOut};
use smith_host_protocol::{SourceAnswer, SourceAsk};

pub(in crate::credentials) fn run(
    name: &str,
    settings: &Settings,
    logout: bool,
    clock: &Clock,
    environment: &Environment,
) -> Result<LoginOutcome, String> {
    let directory = Directory::of(settings).ok_or("an absolute credentials directory is required")?;
    let configured = settings.values.get("programs.claude_code").and_then(|value| value.value.as_str());
    let Ok(program) = crate::settings::resolve::programs::locate(configured, settings.paths.path.as_deref()) else {
        eprintln!("account {name}: program is missing");
        return Ok(LoginOutcome::Unavailable);
    };
    let configuration =
        smith_agent_shell::program::prepare(&directory.programs(), name).map_err(|why| why.to_string())?;
    if !logout {
        let result =
            smith_agent_shell::program::sign_in(&program, &configuration, environment.variables.clone()).status();
        if !result.is_ok_and(|status| status.success()) {
            eprintln!("account {name}: program sign-in failed");
            return Ok(LoginOutcome::Unavailable);
        }
    }
    let base: Vec<Box<[u8]>> = environment
        .variables
        .iter()
        .map(|(name, value)| {
            let mut entry = name.as_encoded_bytes().to_vec();
            entry.push(b'=');
            entry.extend_from_slice(value.as_encoded_bytes());
            entry.into()
        })
        .collect();
    if logout {
        match program_source::run(SourceAsk::SignOut, &program, name, &directory, clock, &base)? {
            SourceAnswer::SignOut(answer) => {
                if let Some(line) = sign_out_line(name, &answer) {
                    eprintln!("{line}");
                }
            }
            SourceAnswer::Check(_) => unreachable!("sign-out answers sign-out"),
        }
    }
    match program_source::run(SourceAsk::Check, &program, name, &directory, clock, &base)? {
        SourceAnswer::Check(answer) => {
            let (outcome, line) = checked(name, logout, &answer);
            eprintln!("{line}");
            Ok(outcome)
        }
        SourceAnswer::SignOut(_) => unreachable!("check answers check"),
    }
}

fn sign_out_line(name: &str, answer: &ProgramSignOut) -> Option<String> {
    match answer {
        ProgramSignOut::SignedOut => None,
        ProgramSignOut::Failed(failure) => {
            Some(format!("account {name}: program sign-out failed: {}", detail(&failure.detail)))
        }
    }
}

fn checked(name: &str, logout: bool, answer: &ProgramCheck) -> (LoginOutcome, String) {
    match answer {
        ProgramCheck::Usable { .. } => {
            (if logout { LoginOutcome::Unavailable } else { LoginOutcome::Usable }, format!("account {name}: usable"))
        }
        ProgramCheck::NotSignedIn => (
            if logout { LoginOutcome::Usable } else { LoginOutcome::Unavailable },
            format!("account {name}: not signed in"),
        ),
        ProgramCheck::SignedInOtherwise(method) => (
            LoginOutcome::Unavailable,
            format!("account {name}: signed in otherwise ({})", String::from_utf8_lossy(method.as_bytes())),
        ),
        ProgramCheck::Failed(failure) => {
            (LoginOutcome::Unavailable, format!("account {name}: program check failed: {}", detail(&failure.detail)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smith_host_domain::{FailureReason, ProgramFailure};

    #[test]
    fn a_refused_program_logout_keeps_the_login_and_escapes_its_bounded_detail() {
        // The host retained 1024 bytes, including the 15-byte suffix and its final LF.
        const RETAINED_FILL: usize = 1009;
        let fill = "x".repeat(RETAINED_FILL);
        let detail = format!("{fill}tail\x1b[2J\r\n\x07end\n");
        assert_eq!(detail.len(), usize::try_from(program_source::DETAIL_BYTES).expect("detail bound"));
        let answer = ProgramSignOut::Failed(ProgramFailure {
            reason: FailureReason::Exited,
            detail: detail.into_bytes().into_boxed_slice(),
        });
        let line = sign_out_line("work", &answer).expect("failure line");
        assert!(!line.bytes().any(|byte| byte.is_ascii_control()), "no raw terminal controls");
        assert_eq!(line, format!(r"account work: program sign-out failed: {fill}tail\x1b[2J\r\n\x07end\n"));

        let still_signed_in = ProgramCheck::Usable { email: Some(b"person@example.com".as_slice().into()) };
        let (outcome, line) = checked("work", true, &still_signed_in);
        assert_eq!(outcome.code(), 3, "the program is still signed in after logout");
        assert_eq!(line, "account work: usable", "no login details from the status");
    }
}
