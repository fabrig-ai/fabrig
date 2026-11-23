//! Scripted MCP peer entry point (smith/docs/design/protocol/programs-claude-code.md, section 1, and smith/docs/design/protocol/programs.md, section 8).

fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    smith_fake_claude::server_main()
}
