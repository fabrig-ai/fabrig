//! The agent's binary (temper/docs/design/cli/README.md, 2).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    temper_agent_shell::main()
}
