//! A worker's start: a worker alone's whole, and what a standalone start's worker takes (protocol/binaries.md, 5, and protocol/cluster.md, 1).
use crate::{agent, path};
use std::{
    fs,
    io::Write,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::Path,
};
use temper_service_shell::jig_directories;
use temper_shell::directories::{Directories, EnvironmentEntry, private_root};
use temper_worker_service as service;
/// Prepare a worker's own directories and program sources, with no deployment.
pub fn alone(
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<skein_io::kernel::Fd>,
    environment: &[EnvironmentEntry],
) -> Result<service::worker_alone::Service, String> {
    let mut part = service::startup::worker_part(
        bytes,
        &jig_directories(directories),
        temper_shell::lookup::find(environment, b"git").as_ref().map(|path| path.as_os_str().as_bytes()),
        environment,
    )
    .map_err(|why| String::from_utf8_lossy(&temper_service_core::startup::problems::lines(&why)).into_owned())?;
    inputs(&mut part, directories, &std::env::current_exe().map_err(|why| why.to_string())?)?;
    let files = temper_service_shell::read_files(core::mem::take(&mut part.files), skein_shell::effective_user())?;
    let seed = skein_shell::seed().map_err(|errno| format!("random seed: errno {errno}"))?;
    let fetch = service::fetch::Fetch::new(&part.worker, &files, seed).map_err(|why| {
        String::from_utf8_lossy(&service::startup::lines(&service::startup::Problems::jig(
            why,
            &jig_protocol_config::Path::root(),
        )))
        .into_owned()
    })?;
    let figures = crate::fetch::take(fetch, directories, environment)?;
    eprint!("{}", figures_line(&part.worker.jig.engine, &figures));
    from_figures(part, figures, &files, directories, owned, environment)
}
/// Construct from figures fetched on the shell's or a world's ring.
pub fn from_figures(
    mut part: service::startup::WorkerPart,
    figures: jig_protocol_worker_wire::Figures,
    files: &service::settings::Files,
    directories: &Directories,
    owned: &mut Vec<skein_io::kernel::Fd>,
    environment: &[EnvironmentEntry],
) -> Result<service::worker_alone::Service, String> {
    let unknown = service::figures::apply(&mut part, figures)
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    for name in &unknown {
        eprintln!("the engine's figures: unknown limit {}; skipped", String::from_utf8_lossy(name));
    }
    let claude = part.claude.as_deref().map(path).map(Path::to_path_buf).or_else(|| find_claude(environment));
    let bytes = agent::figures::configuration(
        part.figures.as_ref().expect("applied figures"),
        claude.as_deref(),
        path(&directories.data),
        &part.environment,
    )?;
    let agent::Prepared { agent, bytes, environment: command_environment } =
        agent::prepare_bytes(bytes).map_err(|why| format!("the engine's figures: {why}"))?;
    let user = skein_shell::effective_user();
    let runtime = root(&part.named, &bytes, user, owned)?;
    let seed = skein_shell::seed().map_err(|errno| format!("random seed: errno {errno}"))?;
    let agent_program = part.named.program.clone();
    let mut service = service::startup::worker_alone(part, files, &agent, runtime, seed)
        .map_err(|why| String::from_utf8_lossy(&temper_service_core::startup::problems::lines(&why)).into_owned())?;
    let sources =
        sources(&agent, command_environment, path(&agent_program), path(&directories.cache), user, environment)?;
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

pub fn root(
    worker: &service::startup::NamedWorker,
    configuration: &[u8],
    user: u32,
    owned: &mut Vec<skein_io::kernel::Fd>,
) -> Result<service::WorkerRuntime, String> {
    let root_path = path(&worker.root);
    if std::fs::symlink_metadata(root_path).is_ok() {
        private_root(root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
    }
    let root = skein_shell::make_root(root_path, 0o700)
        .map_err(|errno| format!("application.workers[0].root: errno {errno}"))?;
    owned.push(root);
    private_root(root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
    let agent = pin(
        root_path,
        &jig_protocol_config::Agent {
            program: worker.program.clone(),
            arguments: worker.arguments.clone(),
            configuration: worker.agent.clone(),
        },
        configuration,
    )?;
    Ok(service::WorkerRuntime { root, path: root_path.as_os_str().as_bytes().into(), agent, wires: None })
}

/// Write a fresh, private agent from the start's checked bytes
/// (domain/worker/README.md, 5). Only exact configuration arguments are replaced.
pub fn pin(
    root: &Path,
    agent: &jig_protocol_config::Agent,
    configuration: &[u8],
) -> Result<jig_protocol_config::Agent, String> {
    let directory = std::path::absolute(root).map_err(|why| format!("{}: {why}", root.display()))?.join("agent");
    match fs::symlink_metadata(&directory) {
        Ok(metadata) => {
            let removed = if metadata.is_dir() { fs::remove_dir_all(&directory) } else { fs::remove_file(&directory) };
            removed.map_err(|why| format!("{}: {why}", directory.display()))?;
        }
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(format!("{}: {why}", directory.display())),
    }
    fs::DirBuilder::new().mode(0o700).create(&directory).map_err(|why| format!("{}: {why}", directory.display()))?;
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
        .map_err(|why| format!("{}: {why}", directory.display()))?;
    let program = path(&agent.program);
    let document = path(&agent.configuration);
    let program_copy =
        directory.join(program.file_name().ok_or_else(|| format!("{}: no file name", program.display()))?);
    let document_copy =
        directory.join(document.file_name().ok_or_else(|| format!("{}: no file name", document.display()))?);
    if program_copy == document_copy {
        return Err(format!("{}: program and configuration have the same file name", document.display()));
    }
    fs::copy(program, &program_copy)
        .map_err(|why| format!("{} -> {}: {why}", program.display(), program_copy.display()))?;
    fs::set_permissions(&program_copy, fs::Permissions::from_mode(0o700))
        .map_err(|why| format!("{}: {why}", program_copy.display()))?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&document_copy)
        .map_err(|why| format!("{}: {why}", document_copy.display()))?;
    file.write_all(configuration).map_err(|why| format!("{}: {why}", document_copy.display()))?;
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|why| format!("{}: {why}", document_copy.display()))?;
    let pinned_configuration: Box<[u8]> = document_copy.as_os_str().as_bytes().into();
    let arguments = agent
        .arguments
        .iter()
        .map(|argument| if *argument == agent.configuration { pinned_configuration.clone() } else { argument.clone() })
        .collect();
    Ok(jig_protocol_config::Agent {
        program: program_copy.as_os_str().as_bytes().into(),
        arguments,
        configuration: pinned_configuration,
    })
}

/// Resources and switches the caller installs on its own service.
pub struct Sources {
    pub resources: Option<service::program_sources::Resources>,
    pub reuse: bool,
    pub dump: Option<Box<[u8]>>,
}
pub fn sources(
    agent: &service::settings::Agent,
    command_environment: Box<[Box<[u8]>]>,
    agent_program: &std::path::Path,
    cache: &std::path::Path,
    user: u32,
    environment: &[EnvironmentEntry],
) -> Result<Sources, String> {
    source_inputs(agent, command_environment, agent_program, cache, user, environment, true)
}
/// Compute source resources without opening the borrowed parent used only while running.
pub fn sources_without_parent(
    agent: &service::settings::Agent,
    command_environment: Box<[Box<[u8]>]>,
    agent_program: &std::path::Path,
    cache: &std::path::Path,
    user: u32,
    environment: &[EnvironmentEntry],
) -> Result<Sources, String> {
    source_inputs(agent, command_environment, agent_program, cache, user, environment, false)
}
fn source_inputs(
    agent: &service::settings::Agent,
    command_environment: Box<[Box<[u8]>]>,
    agent_program: &std::path::Path,
    cache: &std::path::Path,
    user: u32,
    environment: &[EnvironmentEntry],
    open_parent: bool,
) -> Result<Sources, String> {
    let reuse = crate::workspace_reuse(environment);
    let dump = crate::dump_llm(environment).map(|directory| directory.as_os_str().as_bytes().into());
    let resources = if let Some(program) = agent.programs.first() {
        if agent::launcher(path(&program.directory)).as_deref() == Some(path(&program.path)) {
            temper_launcher::place(path(&program.path).parent().expect("launcher parent"), cache, agent_program)
                .map_err(|why| format!("programs.claude_code: {why}"))?;
        }
        // Source commands use the agent's explicit command environment.
        let environment = command_environment;
        let parent = if open_parent {
            skein_shell::open_root(path(&program.directory).parent().unwrap_or(std::path::Path::new("/"))).ok()
        } else {
            None
        };
        let resources = service::program_sources::configured(program, environment, user, parent)
            .ok_or("workers[0].agent.configuration: program source resources exceed their bounds")?;
        Some(resources)
    } else {
        None
    };
    Ok(Sources { resources, reuse, dump })
}

/// Machine-only checks before the opening. No agent document is read.
pub fn inputs(
    part: &mut service::startup::WorkerPart,
    directories: &Directories,
    executable: &Path,
) -> Result<(), String> {
    match fs::symlink_metadata(path(&directories.config).join("agent.json")) {
        Ok(_) => {
            return Err(
                "agent.json: moved: temper writes the agent's configuration under the worker's root; remove it".into(),
            );
        }
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(format!("agent.json: {why}")),
    }
    if let Some(claude) = &part.claude {
        executable_file(path(claude), "application.workers[0].claude")?;
    }
    let program = executable.parent().ok_or("worker executable has no directory")?.join("temper-agent");
    executable_file(&program, "the derived agent program")?;
    part.named.program = program.as_os_str().as_bytes().into();
    part.named.arguments = Box::new([part.named.agent.clone()]);
    Ok(())
}
pub(crate) fn executable_file(program: &Path, name: &str) -> Result<(), String> {
    if !program.is_absolute()
        || !fs::metadata(program).is_ok_and(|metadata| metadata.is_file() && metadata.mode() & 0o111 != 0)
    {
        return Err(format!("{name}: {}: must be an absolute path to an executable file", program.display()));
    }
    Ok(())
}
#[must_use]
pub fn find_claude(environment: &[EnvironmentEntry]) -> Option<std::path::PathBuf> {
    let value = temper_shell::environment::PATH.value(environment)?;
    value
        .split(|byte| *byte == b':')
        .filter(|entry| entry.starts_with(b"/"))
        .map(|entry| path(entry).join("claude"))
        .find(|candidate| executable_file(candidate, "claude").is_ok())
}
#[must_use]
pub fn figures_line(engine: &jig_protocol_config::Address, figures: &jig_protocol_worker_wire::Figures) -> String {
    let engine = match engine {
        jig_protocol_config::Address::Addr(skein_io::Addr::Ip(address)) => address.to_string(),
        jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(name)) => format!("{name:?}"),
        jig_protocol_config::Address::Host { name, port } => format!("{}:{port}", String::from_utf8_lossy(name)),
    };
    format!(
        "figures taken from {engine}: {} places, {} limits, {} endpoints\n",
        figures.places(),
        figures.limits().len(),
        figures.agents().endpoints().len()
    )
}
