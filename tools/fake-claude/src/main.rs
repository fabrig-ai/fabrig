//! Process-tier entry point for smith's fake (domain/testing.md, 2.6).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    smith_fake_claude::main()
}
