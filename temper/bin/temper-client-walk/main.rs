//! The walk tool's binary (temper/docs/design/testing/tools.md, 3).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    temper_client_walk_tool::main(temper_version::VERSION, temper_version::COMMIT)
}
