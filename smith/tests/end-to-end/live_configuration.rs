//! Offline controls for live configuration and the credential-directory guard.
#[path = "support/live.rs"]
pub mod live;
#[path = "support/live_report.rs"]
pub mod live_report;

#[test]
fn the_live_arguments_resolve_only_the_requested_account() {
    assert_account("claude");
}

#[test]
fn chatgpt_live_arguments_select_only_chatgpt() {
    assert_account("chatgpt");
}

#[test]
fn key_live_arguments_select_only_the_key() {
    assert_account("anthropic-key");
}

fn assert_account(account: &str) {
    let directory = std::path::Path::new("/unused/smith-live-story");
    let settings = live_settings(live::arguments(account, directory, true, false), directory);
    let resolved =
        smith_local_shell::settings::resolve::select(&settings).unwrap_or_else(|why| panic!("{}", why.line()));
    assert!(
        resolved.accounts.len() == 1 && resolved.accounts.first().is_some_and(|selected| selected.name == account),
        "only this story's account is selected"
    );
}

fn live_settings(
    arguments: Vec<std::ffi::OsString>,
    directory: &std::path::Path,
) -> smith_local_shell::settings::Settings {
    use smith_local_shell::{command, settings};
    let command::Command::Exec(execution) = command::parse(arguments).expect("live command parses") else {
        panic!("exec command")
    };
    let source = settings::Source { layer: settings::Layer::User, path: "/unused/live/config/smith/config.toml".into(), text: "credentials='/unused/live'\n[accounts.chatgpt]\nprovider='openai'\nsource='sign_in'\n[accounts.claude]\nprovider='anthropic'\nsource='program'\n[accounts.anthropic-key]\nprovider='anthropic'\nsource='env'\nvariable='ANTHROPIC_API_KEY'\n".into() };
    settings::from_sources(&execution.shared, directory.into(), &[source])
        .unwrap_or_else(|why| panic!("{}", why.line()))
}

#[test]
fn the_refresh_story_lead_outlives_an_access_token() {
    let directory = std::path::Path::new("/unused/smith-live-story");
    let mut arguments = live::arguments("chatgpt", directory, true, false);
    live::with_refresh_lead(&mut arguments, live::REFRESH_LEAD_SECONDS);
    let settings = live_settings(arguments, directory);
    let (resolved, _document) =
        smith_local_shell::settings::resolve::resolve_startup(&settings).unwrap_or_else(|why| panic!("{}", why.line()));
    assert_eq!(
        resolved.agent.profile.policy.refresh_lead,
        Duration::from_secs(400 * 86_400),
        "the refresh story's lead is 400 days"
    );
}

#[test]
fn the_live_arguments_carry_private_program_homes() {
    let directory = std::path::Path::new("/unused/smith-live-story");
    let command = smith_local_shell::command::parse(live::arguments("claude", directory, false, false))
        .expect("live command parses");
    let smith_local_shell::command::Command::Exec(execution) = command else { panic!("exec command") };
    for (name, path) in [("HOME", directory.join("home")), ("XDG_STATE_HOME", directory.join("state"))] {
        let expected =
            format!("environment.set.{name}={}", toml::Value::String(path.to_str().expect("synthetic path").into()));
        assert!(
            execution.shared.overrides.contains(&expected),
            "the program receives its scratch home by an explicit setting"
        );
    }
}

#[path = "support/live_tests.rs"]
mod live_tests;

pub mod support;

use skein_lib::{Duration, Time};
use skein_world::Referee;
use support::{
    Settling,
    window::{Story, WORLD_PATIENCE},
};

#[test]
fn a_sign_in_address_waits_for_its_whole_piped_line() {
    let partial = b"Sign in: http://127.0.0.1:34444/authorize?state=";
    assert_eq!(support::login::sign_in_url(partial), None);
    let mut complete = partial.to_vec();
    complete.extend_from_slice(b"whole-state\n");
    assert_eq!(
        support::login::sign_in_url(&complete),
        Some(b"http://127.0.0.1:34444/authorize?state=whole-state".to_vec())
    );
}

#[test]
fn resize_referee_uses_world_patience() {
    let now = Time::ZERO;
    let referee = Settling::new(now, now.saturating_add(Story::Resize.budget(false)));
    assert_eq!(referee.overdue(now.saturating_add(Duration::from_millis(1118))), None);
    let deadline = now.saturating_add(WORLD_PATIENCE);
    assert_eq!(referee.next_deadline(), Some(deadline));
    assert!(referee.overdue(deadline).is_some());
}

#[test]
fn program_stories_allow_thirty_seconds_total() {
    for story in [Story::ProgramRun, Story::ProgramCheck] {
        assert_eq!(story.budget(false), Duration::from_secs(30));
    }
}
