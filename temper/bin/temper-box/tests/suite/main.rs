#[path = "../support/listeners.rs"]
pub mod listeners;
use temper_box_kit as support;

mod agent_modes;
mod box_front;
mod box_listeners;
mod configuration;
mod fake;
mod fake_opening;
mod forge;
mod forge_land;
mod inbox_reply;
mod ports;
mod relay;
mod startup;
mod table;

// Admission exercises the checked start, not its configuration-only ring sizing.
fn admission(directories: &temper_box_shell::directories::Directories, bytes: &[u8]) -> Result<u32, String> {
    let mut owned = Vec::new();
    let result =
        temper_box_shell::prepare(&[], b"/", directories, bytes, &mut owned).map(|service| service.operations());
    for fd in owned {
        skein_shell::close_startup_fd(fd);
    }
    result
}

mod admission_edges;
mod producer_policy;
mod provider_admission;
mod support_tests;
mod wiring;
