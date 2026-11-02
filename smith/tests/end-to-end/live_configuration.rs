//! Offline controls for live configuration and the credential-directory guard.
pub use support::{live, live_report};

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

#[test]
fn the_claude_live_story_reads_its_agent_trace() {
    let directory = std::path::Path::new("/unused/smith-live-story");
    let trace = directory.join("trace-1.jsonl");
    let mut arguments = live::arguments("claude", directory, true, false);
    live::with_trace(&mut arguments, &trace);
    let at = arguments.len().checked_sub(1).expect("prompt argument");
    arguments.splice(at..at, ["-c".into(), "programs.claude_code='/bin/true'".into()]);
    let settings = live_settings(arguments, directory);
    let (resolved, _document) =
        smith_local_shell::settings::resolve::resolve_startup(&settings).unwrap_or_else(|why| panic!("{}", why.line()));
    assert_eq!(resolved.agent.trace.expect("configured trace").path, trace);
    for (account, program) in [("claude", true), ("chatgpt", false)] {
        let observing = trace_observer(account, &trace);
        assert_eq!(observing.prepared.program_source(), program, "{account}'s record source");
    }
}

fn trace_observer(account: &str, trace: &std::path::Path) -> smith_bench::agent::smith::Observing {
    use smith_bench::agent::smith::{FaceSince, Observing, Prepared, Receiving, events::Shape};
    Observing::new(Prepared {
        face: Shape::Exec,
        registry: FaceSince {
            commit: String::new(),
            shape: Shape::Exec,
            receiving: Receiving { string: 64, content: 4096, items: 32, encoded_line: 65_536 },
        },
        settings: "/unused/config/smith/config.toml".into(),
        serial: None,
        trace: Some(trace.into()),
        rendered: live::rendered(account),
        program: "/unused/smith".into(),
        state: "/unused/state".into(),
        teardown: Duration::ZERO,
    })
    .expect("outside observer")
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
fn the_refresh_story_chooses_its_case_from_the_record() {
    use live::RefreshCase::{Kept, Refreshed, Wait};
    use skein_lib::Wall;
    use smith_local_shell::credentials::RefreshFacts;

    let now = Wall::from_nanos(10_000_000_000_000);
    for (refreshable, earliest, expected) in [
        (true, None, Ok(Refreshed)),
        (true, Some(9_999_000_000_000), Ok(Refreshed)),
        (true, Some(10_000_000_000_000), Ok(Refreshed)),
        (true, Some(12_336_000_000_000), Ok(Kept)),
        (true, Some(10_301_000_000_000), Ok(Kept)),
        (true, Some(10_300_000_000_001), Ok(Kept)),
        (true, Some(10_300_000_000_000), Ok(Wait { seconds: 301 })),
        (true, Some(10_120_500_000_000), Ok(Wait { seconds: 122 })),
        (false, None, Err(live::NOT_REFRESHABLE)),
        (false, Some(12_336_000_000_000), Err(live::NOT_REFRESHABLE)),
    ] {
        let facts = RefreshFacts {
            generation: 7,
            refreshable,
            expires_at: Wall::from_nanos(u64::MAX),
            earliest_refresh_at: earliest.map(Wall::from_nanos),
        };
        assert_eq!(live::refresh_case(&facts, now), expected, "refresh facts: {facts:?}");
    }
    const {
        assert!(
            live::EARLIEST_MARGIN_SECONDS > live::CHECK_SECONDS + live::EXEC_SECONDS,
            "the earliest refresh margin outlasts the check and exec bounds together"
        );
    }
}

#[test]
fn the_refresh_report_names_facts_without_token_bytes() {
    use skein_lib::Wall;
    use smith_local_shell::credentials::{RefreshFacts, env_record, refresh_facts, token_limits};
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    const ACCESS: &str = "synthetic-refresh-report-access";
    const REFRESH: &str = "synthetic-refresh-report-refresh";
    let scratch = skein_scratch::Scratch::new("smith-refresh-report");
    let now = Wall::from_nanos(10_000_500_000_000);
    let mut record = env_record(ACCESS.as_bytes(), false, now).expect("synthetic record").record;
    record.generation = 7;
    record.refresh_token = Some(REFRESH.as_bytes().into());
    record.expires_at = Wall::from_nanos(13_127_900_000_000);
    record.earliest_refresh_at = Some(Wall::from_nanos(11_840_900_000_000));
    let bytes = skein_oauth::encode_record(&record, &token_limits()).expect("bounded synthetic record");
    let path = scratch.path().join("record");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .expect("private synthetic record");
    file.write_all(&bytes).expect("write synthetic record");
    let before = refresh_facts(&path).expect("readable synthetic record").expect("synthetic record exists");
    let after = RefreshFacts {
        generation: 8,
        refreshable: false,
        expires_at: Wall::from_nanos(9_988_900_000_000),
        earliest_refresh_at: None,
    };
    let report = live::refresh_report(&before, &after, now);
    assert_eq!(
        report,
        "record before: generation=7 refreshable=true expires_in=3127s earliest_in=1840s; after: generation=8 refreshable=false expires_in=-11s earliest_in=none",
        "the report names only fixed words and whole seconds relative to the story clock"
    );
    for token in [ACCESS, REFRESH] {
        assert!(!report.contains(token), "token bytes stay out of the refresh report");
    }
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
