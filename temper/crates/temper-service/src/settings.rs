//! Settings of the composed service (temper/docs/design/protocol/services.md, 6).
pub use temper_engine_service::settings::*;
pub use temper_worker_service::settings::Worker;
pub use temper_worker_service::worker_settings::Agent;

#[cfg(test)]
#[derive(Clone, Debug)]
pub struct Setup { pub entry: temper_forge_setup::setup::Entry, pub handover: Option<skein_lib::Secret> }
