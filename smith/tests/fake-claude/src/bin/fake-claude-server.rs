//! Scripted MCP peer entry point (protocol/programs.md, sections 5 and 8).

fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    smith_fake_claude::server_main()
}
