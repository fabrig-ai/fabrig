//! Forgejo's fetch host with its bottle extraction (protocol/setup/owned.md, 6).
pub use temper_fetch_shell::fetch::Failed;

/// Runs the machine with Forgejo's bounded bottle extraction.
pub fn run(
    limits: &skein_io::Limits,
    trust: &skein_shell::Trust,
    machine: temper_forge_setup::fetch::Fetch,
    out: &mut dyn std::io::Write,
) -> Result<(), Failed> {
    temper_fetch_shell::fetch::run(limits, trust, machine, crate::process::extract, out)
}
