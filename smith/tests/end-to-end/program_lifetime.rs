//! Aborting an agent ends its program spend (smith/docs/design/protocol/programs.md, 5.1).
#![cfg(target_os = "linux")]
use super::support;
use smith_fake_claude::{STORY_FILE, Step, Story, Turn};
use smith_real_world::Scratch;

#[test]
fn an_aborted_agent_takes_its_program_and_relay_with_it() {
    let scratch = Scratch::new();
    let executable = support::fake_claude();
    let settings = format!(
        "model='anthropic/claude-opus-5-5'\nsub_agents=true\n[programs]\nclaude_code='{}'\n\
         [accounts.work]\nprovider='anthropic'\nsource='program'\n[endpoints.anthropic]\naccount='work'\nprogram='claude_code'\n",
        executable.display()
    );
    let _file = support::user_layer(&scratch, settings.as_bytes());
    let login = support::run_product(&scratch, &["login", "work"], b"", None);
    assert_eq!(login.code, 0, "{}", String::from_utf8_lossy(&login.stderr));
    let story = Story { turns: vec![Turn::new(vec![Step::Hang])], ..Story::default() };
    std::fs::write(scratch.path().join("state/smith/programs/work").join(STORY_FILE), story.encode())
        .expect("scratch story");
    support::lifetime::abort_agent(&scratch);
}
