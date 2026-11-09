use crate::directories::jig_directories;
use std::os::unix::ffi::OsStrExt;
use temper_service as service;
/// Prepare a worker's own directories and program sources, with no deployment.
pub(super) fn prepare(
    directories: &crate::Directories,
    bytes: &[u8],
    owned: &mut Vec<skein_io::kernel::Fd>,
) -> Result<service::Service, String> {
    let mut part = service::startup::worker_part(bytes, &jig_directories(directories))
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    let agent = crate::agent::preflight_worker(&part.named)?;
    let configuration = crate::agent::configuration(crate::path(&part.named.agent))?;
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
    let agent_program = part.named.program.clone();
    let mut service = service::startup::worker_alone(part, &files, &agent, runtime, seed)
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    if crate::workspace_reuse() {
        service.reuse_workspaces();
    }
    if let Some(directory) = crate::dump_llm() {
        service.dump_llm(directory.as_os_str().as_bytes());
    }
    if let Some(program) = agent.programs.first() {
        if crate::agent::launcher(crate::path(&program.directory)).as_deref() == Some(crate::path(&program.path)) {
            crate::claude_code::place(
                crate::path(&program.path).parent().expect("launcher parent"),
                crate::path(&directories.cache),
                crate::path(&agent_program),
            )
            .map_err(|why| format!("programs.claude_code: {why}"))?;
        }
        let parent =
            skein_shell::open_root(crate::path(&program.directory).parent().unwrap_or(std::path::Path::new("/"))).ok();
        let resources = service::program_sources::configured(
            program,
            configuration.service.environment.command_entries(),
            user,
            parent,
        )
        .ok_or("workers[0].agent.configuration: program source resources exceed their bounds")?;
        service.configure_program_sources(resources).map_err(|_| "worker refused program source resources")?;
    }
    Ok(service)
}
