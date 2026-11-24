//! Scripted sign-in and turns for Claude Code (smith/docs/design/protocol/programs-claude-code.md, section 1, and smith/docs/design/protocol/programs.md, section 8;
//! smith/docs/design/testing.md, section 4). The fake keeps one command, its story and queued
//! I/O, and knows no person's home or credentials. [`Fake`] is the step
//! machine, [`hosted`] its world factory and [`main`] its kernel entry point.

#![forbid(unsafe_code)]

pub mod invocation;

mod act;
mod commands;
mod faces;
mod fake;
mod mcp;
mod output;
mod print;
mod server;
mod story;
mod stream;

pub use faces::{config_root, hosted, main, mcp_root, observation_append, simulated_launch};
pub use fake::{Faces, Fake};
pub use server::{Answer, Server, ServerScript, Tool, Verdict, served, server_main};
pub use story::{
    Decision, End, Fault, Init, Login, Logout, Observation, Resume, Script, Seen, Selection, Status, Step, Story, Turn,
    Usage, UsageReply, observations, received, seen, selections,
};

/// The environment variable naming this invocation's configuration directory.
pub const CONFIG_DIR: &str = "CLAUDE_CONFIG_DIR";

/// The configuration-directory file holding the signed-in authentication method.
pub const LOGIN_FILE: &str = "fake-claude.login";

/// The configuration-directory file holding the test's encoded story.
pub const STORY_FILE: &str = "fake-claude.story";

/// The configuration-directory file receiving appended invocation observations.
pub const OBSERVATIONS_FILE: &str = "fake-claude.observations";

/// The user settings file beneath the configuration directory.
pub const SETTINGS_FILE: &str = "settings.json";
