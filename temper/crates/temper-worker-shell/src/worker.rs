//! A worker's start: a worker alone's whole, and what a standalone start's worker takes (protocol/binaries.md, 5, and protocol/cluster.md, 1).
use crate::{agent, path};
use std::os::unix::ffi::OsStrExt;
use temper_service as service;
use temper_service_shell::jig_directories;
use temper_shell::directories::{Directories, EnvironmentEntry, private_root};
/// Prepare a worker's own directories and program sources, with no deployment.
pub fn alone(
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<skein_io::kernel::Fd>,
    environment: &[EnvironmentEntry],
) -> Result<service::Service, String> {
    let mut part = service::startup::worker_part(bytes, &jig_directories(directories))
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    let agent = crate::agent::preflight_worker(&part.named)?;
    let user = skein_shell::effective_user();
    let files = temper_service_shell::read_files(core::mem::replace(&mut part.files, Box::new([])), user)?;
    let runtime = root(&part.named.root, user, owned)?;
    let seed = skein_shell::seed().map_err(|errno| format!("random seed: errno {errno}"))?;
    let agent_program = part.named.program.clone();
    let configuration = part.named.agent.clone();
    let mut service = service::startup::worker_alone(part, &files, &agent, runtime, seed)
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    let sources =
        sources(&agent, path(&configuration), path(&agent_program), path(&directories.cache), user, environment)?;
    if sources.reuse {
        service.reuse_workspaces();
    }
    if let Some(directory) = sources.dump {
        service.dump_llm(&directory);
    }
    if let Some(resources) = sources.resources {
        service.configure_program_sources(resources).map_err(|_| "worker refused program source resources")?;
    }
    Ok(service)
}

pub fn root(path: &[u8], user: u32, owned: &mut Vec<skein_io::kernel::Fd>) -> Result<service::WorkerRuntime, String> {
    let root_path = crate::path(path);
    if std::fs::symlink_metadata(root_path).is_ok() {
        private_root(root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
    }
    let root = skein_shell::make_root(root_path, 0o700)
        .map_err(|errno| format!("application.workers[0].root: errno {errno}"))?;
    owned.push(root);
    private_root(root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
    Ok(service::WorkerRuntime { root, path: root_path.as_os_str().as_bytes().into() })
}

/// Resources and switches the caller installs on its own service.
pub struct Sources {
    pub resources: Option<service::program_sources::Resources>,
    pub reuse: bool,
    pub dump: Option<Box<[u8]>>,
}
pub fn sources(
    agent: &service::settings::Agent,
    configuration: &std::path::Path,
    agent_program: &std::path::Path,
    cache: &std::path::Path,
    user: u32,
    environment: &[EnvironmentEntry],
) -> Result<Sources, String> {
    let reuse = crate::workspace_reuse(environment);
    let dump = crate::dump_llm(environment).map(|directory| directory.as_os_str().as_bytes().into());
    let resources = if let Some(program) = agent.programs.first() {
        if agent::launcher(path(&program.directory)).as_deref() == Some(path(&program.path)) {
            temper_launcher::place(path(&program.path).parent().expect("launcher parent"), cache, agent_program)
                .map_err(|why| format!("programs.claude_code: {why}"))?;
        }
        // Source commands use the agent's explicit command environment.
        let environment = agent::configuration(configuration)?.service.environment.command_entries();
        let parent =
            skein_shell::open_root(path(&program.directory).parent().unwrap_or(std::path::Path::new("/"))).ok();
        let resources = service::program_sources::configured(program, environment, user, parent)
            .ok_or("workers[0].agent.configuration: program source resources exceed their bounds")?;
        Some(resources)
    } else {
        None
    };
    Ok(Sources { resources, reuse, dump })
}
