//! The joined application's engine and worker rows (protocol/services.md, 6).
use super::Problems;
use crate::{limits, settings};
use alloc::boxed::Box;
pub use engine::{Clients, Forge, Identity, Origin, Resolved, Scheme, Transport, USER_BYTES, User, Webhooks, sign_in};
use jig_protocol_config::{self as config, Path, read};
use skein_json::Document;
use skein_lib::List;
use temper_engine_service::startup::application as engine;
pub use temper_worker_service::startup::worker::application::Worker;
#[derive(Debug)]
pub struct Members {
    pub clients: Clients,
    pub sign_in: Option<sign_in::SignIn>,
    pub webhooks: Option<Webhooks>,
    pub machine: temper_forge_setup::machine::Machine,
    pub demo: Option<settings::Demo>,
    pub welcome: bool,
    pub landings: skein_lib::Map<u32, temper_engine_domain::ProjectLanding>,
    pub landing_projects: Box<[u32]>,
    pub limits: limits::Limits,
    pub overrides: Box<[(Box<[u8]>, u64)]>,
    pub workers: Box<[Worker]>,
}
pub fn parse(
    document: &Document,
    at: u32,
    jig: &config::Engine,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Members> {
    let engine = engine::parse(document, at, jig, directories, problems)?;
    let mut object = read::Object::new(document, at).expect("checked application object");
    let workers = object.member(document, b"workers").expect("checked workers member");
    let ats = read::list(document, workers, jig.limits.core.fleet.workers).expect("checked application workers");
    let mut workers = List::with_capacity(ats.len());
    for (index, at) in ats.iter().enumerate() {
        let mut found = temper_worker_service::startup::Problems::empty();
        let worker = temper_worker_service::startup::worker::application::member(
            document,
            index,
            *at,
            directories,
            &Path::root().member(b"application"),
            &mut found,
        );
        assert!(found.is_empty(), "the engine's document grammar checked this member");
        workers.push(worker.expect("checked worker row")).expect("application worker count");
    }
    Some(Members {
        clients: engine.clients,
        sign_in: engine.sign_in,
        webhooks: engine.webhooks,
        machine: engine.machine,
        demo: engine.demo,
        welcome: engine.welcome,
        landings: engine.landings,
        landing_projects: engine.landing_projects,
        limits: engine.limits,
        overrides: engine.overrides,
        workers: workers.into_boxed(),
    })
}
#[must_use]
pub fn files(members: &Members) -> Box<[config::File]> {
    engine::named_files(members.sign_in.as_ref(), &members.limits)
}
pub fn resolve(members: &Members, files: &settings::Files, problems: &mut Problems) -> Option<Resolved> {
    engine::resolve_parts(members.sign_in.as_ref(), members.webhooks.as_ref(), &members.limits, files, problems)
}
