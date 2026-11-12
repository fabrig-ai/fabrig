//! The stage-one tool's binary (temper/docs/design/testing/tools.md, 3).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    temper_stage_one_tool::main(temper_version::VERSION, temper_version::COMMIT)
}
