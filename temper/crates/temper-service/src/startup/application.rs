//! The joined application's engine and worker rows (protocol/services.md, 6).
use super::Problems;
use crate::{limits, settings};
use alloc::boxed::Box;
pub use engine::{
    Clients, Forge, Identity, Origin, Resolved, Scheme, Transport, USER_BYTES, User, Webhooks, push, sign_in,
};
use jig_protocol_config as config;
use skein_json::Document;
use skein_lib::List;
use temper_engine_service::startup::application as engine;
pub use temper_worker_service::startup::worker::application::Worker;
#[derive(Debug)]
pub struct Members {
    pub clients: Clients,
    pub sign_in: Option<sign_in::SignIn>,
    pub push: Option<push::Push>,
    pub webhooks: Option<Webhooks>,
    pub machine: temper_forge_setup::machine::Machine,
    pub test_github: Option<temper_forge_setup::machine::GitHubMachine>,
    pub demo: Option<settings::Demo>,
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
    let mut workers = List::with_capacity(u32::try_from(engine.workers.len()).expect("bounded workers"));
    for worker in engine.workers {
        workers.push(Worker { root: worker.root, git: None }).expect("bounded workers");
    }
    Some(Members {
        clients: engine.clients,
        sign_in: engine.sign_in,
        push: engine.push,
        webhooks: engine.webhooks,
        machine: engine.machine,
        test_github: engine.test_github,
        demo: engine.demo,
        landings: engine.landings,
        landing_projects: engine.landing_projects,
        limits: engine.limits,
        overrides: engine.overrides,
        workers: workers.into_boxed(),
    })
}
#[must_use]
pub fn files(members: &Members) -> Box<[config::File]> {
    engine::named_files(members.sign_in.as_ref(), members.push.as_ref(), &members.limits)
}
pub fn resolve(members: &Members, files: &settings::Files, problems: &mut Problems) -> Option<Resolved> {
    engine::resolve_parts(
        members.sign_in.as_ref(),
        members.push.as_ref(),
        members.webhooks.as_ref(),
        &members.limits,
        files,
        problems,
    )
}
