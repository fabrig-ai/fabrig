use std::os::unix::ffi::OsStrExt;
use temper_service as service;
/// Prepare a worker's own directories and program sources, with no deployment.
pub(super) fn prepare(
    directories: &crate::Directories,
    bytes: &[u8],
    owned: &mut Vec<skein_io::kernel::Fd>,
) -> Result<service::Service, String> {
    let mut part = service::startup::worker_part(bytes, &directories.jig())
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    let agent = crate::agent::preflight_worker(&part.named)?;
    let configuration = smith_agent_shell::config::read(crate::path(&part.named.agent))?;
    let user = skein_shell::effective_user();
    let files = crate::read_files(core::mem::replace(&mut part.files, Box::new([])), user)?;
    let root_path = crate::path(&part.named.root);
    if std::fs::symlink_metadata(root_path).is_ok() {
        crate::private_root(root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
    }
    let root = skein_shell::make_root(root_path, 0o700)
        .map_err(|errno| format!("application.workers[0].root: errno {errno}"))?;
    owned.push(root);
    crate::private_root(root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
    let runtime = service::WorkerRuntime { root, path: root_path.as_os_str().as_bytes().into() };
    let seed = skein_shell::seed().map_err(|errno| format!("random seed: errno {errno}"))?;
    let mut service = service::startup::worker_alone(part, &files, &agent, runtime, seed)
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    if let Some(program) = agent.programs.first() {
        let resources = service::program_sources::Resources::configured(
            program,
            configuration.service.environment.command_entries(),
        )
        .ok_or("workers[0].agent.configuration: program source resources exceed their bounds")?;
        service.configure_program_sources(resources).map_err(|_| "worker refused program source resources")?;
    }
    Ok(service)
}
