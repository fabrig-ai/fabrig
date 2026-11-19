//! The standalone start, composed from the engine and worker shells (protocol/services.md, 7).
//! Fixed starts follow temper/docs/design/protocol/setup/box.md, 5 and
//! temper/docs/design/client/journey-clock.md, 4.1.
use crate::{Service, path, prefixed, program_login, setup};
use skein_io::kernel::{Exit, Fd};
use skein_lib::{List, Secret};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use temper_client_protocol_wire::v1::Record;
use temper_engine_shell::serve::{self as engine, printed};
use temper_service as service;
use temper_service_shell::{close, jig_directories};
use temper_shell::directories::{Directories, EnvironmentEntry};
use temper_worker_shell::{agent, worker};
pub fn prepare(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
    programs: &temper_config_shell::Programs,
) -> Result<Service, String> {
    prepare_with_store(binary, environment, working, directories, bytes, owned, false, programs)
        .map(|(service, _)| service)
}
#[expect(clippy::too_many_arguments, reason = "the start carries file-only program paths beside its document")]
pub fn prepare_with_store(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
    sizing: bool,
    programs: &temper_config_shell::Programs,
) -> Result<(Service, Box<[u8]>), String> {
    prepare_with_limits(
        binary,
        environment,
        working,
        directories,
        bytes,
        owned,
        false,
        false,
        None,
        None,
        &service::stock::Given::Stock,
        None,
        sizing,
        &[],
        None,
        None,
        programs,
    )
    .map(|(service, store, _)| (service, store))
}
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "the standalone start supplies the binary beside the resolved process facts"
)]
pub(crate) fn prepare_with_limits(
    _binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
    under_box: bool,
    record: bool,
    link_secret: Option<Secret>,
    local_secret: Option<Secret>,
    given: &service::stock::Given,
    program: Option<&std::path::Path>,
    sizing: bool,
    projects: &[jig_protocol_config::Project],
    limits: Option<(&jig_protocol_config::Limits, &service::limits::Limits)>,
    fixed_start: Option<&skein_shell::FixedStart>,
    programs: &temper_config_shell::Programs,
) -> Result<(Service, Box<[u8]>, u32), String> {
    let given = crate::scripted::given(directories, given, crate::scripted::address(environment)?);
    let given = &given;
    let setup::Deployment { mut named, checked, limits: box_limits, demo } =
        setup::check_over(bytes, &jig_directories(directories), given, limits)?;
    let engine = engine::settings(bytes, directories, given);
    let engine::Runtimes { roots, github } = engine::runtimes(named.test_github.as_ref())?;
    let user = skein_shell::effective_user();
    let copy = path(&directories.data).join("agents.setting");
    let initial = if named.worker.is_some() { agents_setting(&copy, &mut std::io::stderr()) } else { None };
    let local_copy = path(&directories.data).join("local.setting");
    let local = if named.worker.is_some() { local_setting(&local_copy, &mut std::io::stderr()) } else { None };
    let places = local.as_ref().map_or(4, jig_client_protocol_wire::WorkerSetting::places);
    let agents = initial.clone().unwrap_or_else(service::stock::agents);
    let mut part = match limits {
        Some(limits) => service::startup::box_worker_with_limits(
            bytes,
            &jig_directories(directories),
            given,
            &agents,
            places,
            environment,
            None,
            limits,
        ),
        None => service::startup::box_worker(bytes, &jig_directories(directories), given, &agents, places, environment),
    }
    .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    let agent = part
        .as_mut()
        .map(|part| -> Result<_, String> {
            let prepared = prepare_agent(part, directories, environment, programs)?;
            let declaration = prepared.agent.declaration();
            *part = match limits {
                Some(limits) => service::startup::box_worker_with_limits(
                    bytes,
                    &jig_directories(directories),
                    given,
                    &agents,
                    places,
                    environment,
                    Some(&declaration),
                    limits,
                ),
                None => service::startup::box_worker_with_agent(
                    bytes,
                    &jig_directories(directories),
                    given,
                    &agents,
                    places,
                    environment,
                    &declaration,
                ),
            }
            .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?
            .expect("box worker already read");
            box_inputs_worker(part, directories)?;
            if let Some(program) = program {
                part.named.program = program.as_os_str().as_encoded_bytes().into();
                part.worker.jig.agent.as_mut().expect("derived agent").program.clone_from(&part.named.program);
            }
            let named = named.worker.as_mut().expect("box worker named");
            named.program.clone_from(&part.named.program);
            named.arguments.clone_from(&part.named.arguments);
            Ok(prepared)
        })
        .transpose()?;
    let engine::Inputs { files, google, bundle } = engine::before_root(
        &mut named.files,
        &mut named.asked_files,
        named.sign_in.as_ref(),
        named.bundle.as_deref(),
        named.bundle_limits,
        user,
        record,
    )?;
    let runtime = named
        .worker
        .as_ref()
        .map(|worker| worker::root(worker, &agent.as_ref().expect("local agent checked").bytes, user, owned))
        .transpose()?;
    let program_login = if named.worker.is_some() {
        program_login::line(directories, &printed::running_word(environment), skein_shell::HOME.value(environment))
            .map(|line| line.into_bytes().into_boxed_slice())
    } else {
        None
    };
    let accounts = engine::accounts(&engine)?;
    let seeds = match fixed_start {
        Some(start) => engine::seeds_from(start, link_secret, local_secret),
        None => engine::seeds(link_secret, local_secret)?,
    };
    let git = programs.git.clone().or_else(|| temper_shell::lookup::find(environment, b"git"));
    let landing = engine::landing_hold(path(&directories.data))?;
    let sources = agent
        .as_ref()
        .map(|agent| {
            let worker = named.worker.as_ref().expect("local agent checked");
            let source_inputs = if sizing { worker::sources_without_parent } else { worker::sources };
            source_inputs(
                &agent.agent,
                agent.environment.clone(),
                path(&worker.program),
                path(&directories.cache),
                user,
                environment,
            )
        })
        .transpose()?;
    let declaration = agent.as_ref().map(|prepared| prepared.agent.declaration());
    let inputs = service::startup::LimitInputs::new(
        bytes,
        &jig_directories(directories),
        &files,
        declaration.as_ref(),
        &seeds,
        given,
        named.worker.is_some(),
        limits,
    );
    let models = jig_protocol_config::core(&engine, seeds.deployment, seeds.seed).settings.models;
    let (mut store, state) =
        engine::open_limits(inputs, &models, path(&directories.data), &named.deployment, user, "", owned)?;
    if part.is_some() {
        let rebuilt = service::startup::box_worker_with_limits(
            bytes,
            &jig_directories(directories),
            given,
            &agents,
            places,
            environment,
            declaration.as_ref(),
            (&state.in_force.jig, &state.in_force.temper),
        );
        let mut rebuilt = match rebuilt {
            Ok(part) => part.expect("box worker already read"),
            Err(why) => {
                engine::close_limits_store(&mut store)?;
                return Err(String::from_utf8_lossy(&service::startup::lines(&why)).into_owned());
            }
        };
        box_inputs_worker(&mut rebuilt, directories)?;
        if let Some(program) = program {
            rebuilt.named.program = program.as_os_str().as_encoded_bytes().into();
            rebuilt.worker.jig.agent.as_mut().expect("derived agent").program.clone_from(&rebuilt.named.program);
        }
        part = Some(rebuilt);
    }
    let start = service::startup::checked_start_with_resolved(
        bytes,
        &jig_directories(directories),
        &files,
        agent.as_ref().map(|prepared| &prepared.agent),
        &seeds,
        List::with_capacity(0),
        Some(&checked),
        given,
        git.as_ref().map(|path| path.as_os_str().as_bytes()),
        part,
        &state.in_force,
    );
    let mut start = match start {
        Ok(start) => start,
        Err(why) => {
            engine::close_limits_store(&mut store)?;
            return Err(String::from_utf8_lossy(&service::startup::lines(&why)).into_owned());
        }
    };
    start.engine_mut().demo = demo.as_ref().map(setup::playground);
    start.engine_mut().owned_forgejo = programs.forgejo.as_ref().map(|path| path.as_os_str().as_bytes().into());
    start.engine_mut().owned_runner = programs.runner.as_ref().map(|path| path.as_os_str().as_bytes().into());
    if !projects.is_empty() {
        let engine = start.engine_mut();
        engine.jig.owners = engine
            .jig
            .owners
            .iter()
            .flat_map(|owner| {
                projects.iter().map(|project| jig_domain_people::InitialOwner {
                    project: project.number,
                    identity: owner.identity.clone(),
                })
            })
            .collect();
        engine.jig.projects = projects.into();
    }
    let service = service::startup::from_checked(
        start,
        &files,
        agent.as_ref().map(|prepared| &prepared.agent),
        &seeds,
        Some(accounts),
        runtime,
        if under_box { Some(box_limits.line_bytes) } else { None },
        program_login,
        engine::program_environment(directories, environment, working, &printed::running_word(environment)),
    );
    let mut service = match service {
        Ok(service) => service,
        Err(why) => {
            engine::close_limits_store(&mut store)?;
            return Err(String::from_utf8_lossy(&service::startup::lines(&why)).into_owned());
        }
    };
    service.engine_component().take_limit_store(store, state);
    if named.worker.is_some() {
        service.engine_component().keep_agents(copy.as_os_str().as_encoded_bytes().into(), initial);
        service.engine_component().keep_local(local_copy.as_os_str().as_encoded_bytes().into(), local);
    }
    if let Some(line) = landing {
        eprintln!("{line}");
        service.engine_component().hold_landings(true);
    }
    service.configure_forge_runtime(github, roots);
    service.configure_google_endpoints(google);
    if let Some(sources) = sources {
        if sources.reuse {
            service.reuse_workspaces();
        }
        if let Some(directory) = sources.dump {
            service.dump_llm(&directory);
        }
        if let Some(resources) = sources.resources
            && service.configure_program_sources(resources).is_err()
        {
            engine::close_limits_store(service.engine_component().store())?;
            return Err("worker refused program source resources".into());
        }
    }
    if let Some(bundle) = bundle {
        service.engine_component().adopt_bundle(bundle);
    }
    Ok((service, named.deployment, box_limits.line_bytes))
}
pub(crate) fn configured_agent(
    bytes: &[u8],
    directories: &Directories,
    environment: &[EnvironmentEntry],
    programs: &temper_config_shell::Programs,
) -> Result<Option<agent::Prepared>, String> {
    let initial = agents_setting(&path(&directories.data).join("agents.setting"), &mut std::io::stderr())
        .unwrap_or_else(service::stock::agents);
    let given =
        crate::scripted::given(directories, &service::stock::Given::Stock, crate::scripted::address(environment)?);
    let local = local_setting(&path(&directories.data).join("local.setting"), &mut std::io::stderr());
    let places = local.as_ref().map_or(4, jig_client_protocol_wire::WorkerSetting::places);
    let mut part =
        service::startup::box_worker(bytes, &jig_directories(directories), &given, &initial, places, environment)
            .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    part.as_mut().map(|part| prepare_agent(part, directories, environment, programs)).transpose()
}
fn prepare_agent(
    part: &mut service::startup::WorkerPart,
    directories: &Directories,
    environment: &[EnvironmentEntry],
    programs: &temper_config_shell::Programs,
) -> Result<agent::Prepared, String> {
    box_inputs_worker(part, directories)?;
    let configured = match &programs.claude {
        Some(claude) => Some(claude.clone()),
        None => crate::fake_backends::configured_claude(directories)?,
    };
    let claude = match configured {
        Some(claude) => Some(claude),
        None => part
            .claude
            .as_deref()
            .map(path)
            .map(std::path::Path::to_path_buf)
            .or_else(|| worker::find_claude(environment)),
    };
    let bytes = agent::figures::configuration(
        part.figures.as_ref().expect("box figures"),
        claude.as_deref(),
        path(&directories.data),
        &part.environment,
    )?;
    agent::prepare_bytes(bytes)
}
/// A bad copy never prevents the box from starting with the stock.
pub(crate) fn agents_setting(
    path: &std::path::Path,
    log: &mut impl std::io::Write,
) -> Option<temper_client_protocol_wire::v1::AgentFigures> {
    use std::io::Read;
    use temper_client_protocol_wire::v1::{AgentFigures, CEILINGS};
    let bound = AgentFigures::encoded_bound(&CEILINGS).expect("agents codec bound");
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return None,
        Err(why) => {
            let _ = writeln!(log, "agents.setting: {why}; using the default");
            return None;
        }
    };
    let mut bytes = Vec::new();
    if let Err(why) = file.take(bound.checked_add(1).expect("bounded codec")).read_to_end(&mut bytes) {
        let _ = writeln!(log, "agents.setting: {why}; using the default");
        return None;
    }
    let mut reader = skein_lib::Reader::new(&bytes);
    match AgentFigures::decode(&CEILINGS, &mut reader) {
        Ok(value) if reader.is_empty() => Some(value),
        Ok(_) | Err(_) => {
            let _ = writeln!(log, "agents.setting: cannot decode; using the default");
            None
        }
    }
}
/// The local worker's value is read before opening its store (configuration/figures.md, 4).
pub(crate) fn local_setting(
    path: &std::path::Path,
    log: &mut impl std::io::Write,
) -> Option<jig_client_protocol_wire::WorkerSetting> {
    use jig_client_protocol_wire::{CEILINGS, WorkerSetting};
    use std::io::Read;
    let bound = WorkerSetting::encoded_bound(&CEILINGS).expect("worker codec bound");
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return None,
        Err(why) => {
            let _ = writeln!(log, "local.setting: {why}; using the default");
            return None;
        }
    };
    let mut bytes = Vec::new();
    if let Err(why) = file.take(bound.checked_add(1).expect("bounded codec")).read_to_end(&mut bytes) {
        let _ = writeln!(log, "local.setting: {why}; using the default");
        return None;
    }
    let mut reader = skein_lib::Reader::new(&bytes);
    match WorkerSetting::decode(&CEILINGS, &mut reader) {
        Ok(value)
            if reader.is_empty()
                && value.number() == 1
                && value.name() == b"local"
                && !value.withdrawn()
                && value.digest().is_none()
                && value.places() > 0 =>
        {
            Some(value)
        }
        Ok(_) | Err(_) => {
            let _ = writeln!(log, "local.setting: cannot decode; using the default");
            None
        }
    }
}
pub(crate) fn box_inputs_worker(
    part: &mut service::startup::WorkerPart,
    directories: &Directories,
) -> Result<(), String> {
    match std::fs::symlink_metadata(path(&directories.config).join("agent.json")) {
        Ok(_) => {
            const MOVED: &str =
                "agent.json: moved: temper writes the agent's configuration under the worker's root; remove it";
            return Err(MOVED.into());
        }
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(format!("agent.json: {why}")),
    }
    if let Some(claude) = &part.claude {
        use std::os::unix::fs::PermissionsExt;
        if !std::fs::metadata(path(claude))
            .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        {
            return Err(format!(
                "application.workers[0].claude: {}: must be an absolute path to an executable file",
                path(claude).display()
            ));
        }
    }
    part.named.program = std::env::current_exe().map_err(|why| why.to_string())?.as_os_str().as_encoded_bytes().into();
    part.named.arguments =
        Box::new([b"internal".as_slice().into(), b"agent".as_slice().into(), part.named.agent.clone()]);
    part.worker.jig.agent = Some(jig_protocol_config::Agent {
        program: part.named.program.clone(),
        arguments: part.named.arguments.clone(),
        configuration: part.named.agent.clone(),
    });
    Ok(())
}
/// The kernel operations a standalone start keeps in flight at most (its ring's size),
/// derived without retaining its descriptors.
pub fn operations_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
) -> Result<u32, String> {
    operations_in_with(binary, environment, working, directories, config, &service::stock::Given::Stock)
}
/// The same for a boxed start.
pub fn boxed_operations_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
) -> Result<u32, String> {
    operations_in_with(binary, environment, working, directories, config, &service::stock::Given::Stock)
}
pub fn prepare_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
) -> Result<Service, String> {
    let read = temper_service_shell::configuration::read(directories, temper_config_shell::Shape::Standalone)
        .map_err(|why| prefixed(&format!("{binary} serve"), &why))?;
    let bytes = &read.document;
    let mut owned = Vec::new();
    let result = read
        .programs()
        .and_then(|programs| prepare(binary, environment, working, directories, bytes, &mut owned, &programs));
    if result.is_err() {
        close(owned);
    }
    result.map_err(|why| prefixed(&format!("{binary} serve"), &read.problems(&why)))
}
/// Prepare an in-process world's service with the supplied defaults.
pub fn prepare_in_with(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    given: &service::stock::Given,
) -> Result<Service, String> {
    prepare_in_with_program(binary, environment, working, directories, given, None)
}
/// A world's fake agent is a typed startup input, never a document member.
pub fn prepare_in_with_program(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    given: &service::stock::Given,
    program: Option<&std::path::Path>,
) -> Result<Service, String> {
    prepare_in_with_inputs(binary, environment, working, directories, given, program, &[])
}
/// Prepare typed project seeds supplied by a world.
pub fn prepare_in_with_projects(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    given: &service::stock::Given,
    projects: &[jig_protocol_config::Project],
) -> Result<Service, String> {
    prepare_in_with_inputs_and_limits(binary, environment, working, directories, given, None, projects, None)
}
#[expect(clippy::too_many_arguments, reason = "the world gives both typed limit sets beside its process facts")]
pub fn prepare_in_with_limits(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    given: &service::stock::Given,
    projects: &[jig_protocol_config::Project],
    jig: &jig_protocol_config::Limits,
    own: &service::limits::Limits,
) -> Result<Service, String> {
    prepare_in_with_inputs_and_limits(
        binary,
        environment,
        working,
        directories,
        given,
        None,
        projects,
        Some((jig, own)),
    )
}
pub fn prepare_in_with_inputs(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    given: &service::stock::Given,
    program: Option<&std::path::Path>,
    projects: &[jig_protocol_config::Project],
) -> Result<Service, String> {
    prepare_in_with_inputs_and_limits(binary, environment, working, directories, given, program, projects, None)
}
#[expect(clippy::too_many_arguments, reason = "the shared preparation carries both program and typed figure seams")]
pub fn prepare_in_with_inputs_and_limits(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    given: &service::stock::Given,
    program: Option<&std::path::Path>,
    projects: &[jig_protocol_config::Project],
    limits: Option<(&jig_protocol_config::Limits, &service::limits::Limits)>,
) -> Result<Service, String> {
    let read = temper_service_shell::configuration::read(directories, temper_config_shell::Shape::Standalone)
        .map_err(|why| prefixed(&format!("{binary} serve"), &why))?;
    let bytes = &read.document;
    let programs = read.programs().map_err(|why| prefixed(&format!("{binary} serve"), &why))?;
    let mut owned = Vec::new();
    let result = prepare_with_limits(
        binary,
        environment,
        working,
        directories,
        bytes,
        &mut owned,
        false,
        false,
        None,
        None,
        given,
        program,
        false,
        projects,
        limits,
        None,
        &programs,
    )
    .map(|(service, _, _)| service);
    if result.is_err() {
        close(owned);
    }
    result.map_err(|why| prefixed(&format!("{binary} serve"), &read.problems(&why)))
}
/// The same for an in-process world's service, with the supplied defaults.
pub fn operations_in_with(
    binary: &str,
    _environment: &[EnvironmentEntry],
    _working: &[u8],
    directories: &Directories,
    config: &[u8],
    given: &service::stock::Given,
) -> Result<u32, String> {
    operations_over(binary, directories, config, given, None)
}
#[expect(clippy::too_many_arguments, reason = "the world gives both typed limit sets beside its process facts")]
pub fn operations_in_with_limits(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
    given: &service::stock::Given,
    jig: &jig_protocol_config::Limits,
    own: &service::limits::Limits,
) -> Result<u32, String> {
    let _ = (environment, working);
    operations_over(binary, directories, config, given, Some((jig, own)))
}
fn operations_over(
    binary: &str,
    directories: &Directories,
    config: &[u8],
    given: &service::stock::Given,
    limits: Option<(&jig_protocol_config::Limits, &service::limits::Limits)>,
) -> Result<u32, String> {
    let dirs = jig_directories(directories);
    let convert = |why| prefixed(&format!("{binary} serve"), &String::from_utf8_lossy(&service::startup::lines(&why)));
    // Startup reports bad copies once; reserving the ring uses the same fallback silently.
    let local = local_setting(&path(&directories.data).join("local.setting"), &mut std::io::sink());
    let places = local.as_ref().map_or(4, jig_client_protocol_wire::WorkerSetting::places);
    let agents = agents_setting(&path(&directories.data).join("agents.setting"), &mut std::io::sink())
        .unwrap_or_else(service::stock::agents);
    let part = match limits {
        Some(limits) => {
            service::startup::box_worker_with_limits(config, &dirs, given, &agents, places, &[], None, limits)
        }
        None => service::startup::box_worker(config, &dirs, given, &agents, places, &[]),
    }
    .map_err(convert)?;
    service::startup::operations_with_worker(config, &dirs, given, limits, part.as_ref()).map_err(convert)
}
pub fn run(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    boxed: bool,
    record: bool,
    fixed_start: Option<&skein_shell::FixedStart>,
) -> Result<Exit, String> {
    temper_service_shell::run(
        binary,
        directories,
        temper_config_shell::Shape::Standalone,
        environment,
        crate::served::SERVED,
        &fixed_start.map_or_else(skein_shell::Clock::new, skein_shell::Clock::fixed),
        |listeners| box_inputs(boxed, listeners),
        |bytes, programs, owned, input| {
            prepare_with_limits(
                binary,
                environment,
                working,
                directories,
                bytes,
                owned,
                boxed,
                record,
                input.link,
                input.local,
                &service::stock::Given::Stock,
                None,
                false,
                &[],
                None,
                fixed_start,
                programs,
            )
            .map(|(service, store, line_bytes)| temper_service_shell::Prepared {
                service,
                store,
                adoption: (input.pipes, line_bytes),
            })
        },
        |service, owned, (pipes, line_bytes)| {
            if let Some((requests, answers)) = pipes {
                owned.extend([requests, answers]);
                service.adopt_box(requests, answers, line_bytes).map_err(box_refusal)?;
            }
            Ok(())
        },
    )
}

struct BoxInputs {
    link: Option<Secret>,
    local: Option<Secret>,
    pipes: Option<(Fd, Fd)>,
}
fn box_inputs(boxed: bool, listeners: usize) -> Result<BoxInputs, String> {
    if !boxed {
        return Ok(BoxInputs { link: None, local: None, pipes: None });
    }
    let requests = 3 + i32::try_from(listeners).expect("bounded inherited listeners");
    let answers = requests + 1;
    if cfg!(target_os = "linux") {
        for fd in [requests, answers] {
            if !std::fs::metadata(format!("/proc/self/fd/{fd}")).is_ok_and(|m| m.file_type().is_fifo()) {
                return Err(format!("--box: fd {fd} is not an open pipe"));
            }
        }
    }
    let (link, local) = given_secret(&mut std::io::stdin().lock())?;
    Ok(BoxInputs { link, local, pipes: Some((Fd::new(requests), Fd::new(answers))) })
}
fn box_refusal(why: service::system::BoxRefusal) -> String {
    use service::system::BoxRefusal;
    let words = match why {
        BoxRefusal::Sockets { limit } => {
            format!("the box's pipes need two free socket slots; the service socket limit is {limit}")
        }
        BoxRefusal::Adopted => "the box's pipes were already adopted".into(),
        BoxRefusal::NoEngine => "the box's pipes need an engine".into(),
        BoxRefusal::ZeroLine => "the box's pipes need a nonzero application.limits.overrides.box.line_bytes".into(),
    };
    format!("--box: {words}")
}
/// The first links' secrets on a boxed serve's standard input (protocol/setup/box.md, 1).
fn given_secret(input: &mut impl std::io::Read) -> Result<(Option<Secret>, Option<Secret>), String> {
    use std::io::Read;
    let both = service::clients::LINK_SECRET_BYTES.checked_mul(2).expect("two secrets");
    let limit = both.checked_add(1).expect("small secret bound");
    let mut bytes = Vec::with_capacity(limit);
    input
        .take(u64::try_from(limit).expect("small secret bound"))
        .read_to_end(&mut bytes)
        .map_err(|_| "--box: stdin: expected the first link's secret".to_string())?;
    if bytes.is_empty() {
        Ok((None, None))
    } else if bytes.len() == service::clients::LINK_SECRET_BYTES {
        Ok((Some(Secret::new(bytes.into_boxed_slice())), None))
    } else if bytes.len() == both {
        let local = bytes.split_off(service::clients::LINK_SECRET_BYTES);
        Ok((Some(Secret::new(bytes.into_boxed_slice())), Some(Secret::new(local.into_boxed_slice()))))
    } else {
        Err("--box: stdin: expected the first link's secret".into())
    }
}

#[cfg(test)]
mod input_tests {
    #[test]
    fn each_box_adoption_refusal_renders_one_line_naming_what_it_lacks() {
        use temper_service::system::BoxRefusal;
        for (why, expected) in [
            (
                BoxRefusal::Sockets { limit: 1 },
                "the box's pipes need two free socket slots; the service socket limit is 1",
            ),
            (BoxRefusal::Adopted, "the box's pipes were already adopted"),
            (BoxRefusal::NoEngine, "the box's pipes need an engine"),
            (BoxRefusal::ZeroLine, "the box's pipes need a nonzero application.limits.overrides.box.line_bytes"),
        ] {
            assert_eq!(
                crate::prefixed("temper serve", &super::box_refusal(why)),
                format!("temper serve: --box: {expected}\n")
            );
        }
    }

    #[test]
    fn a_boxed_serve_reads_none_or_a_whole_secret() {
        use std::io::Cursor;
        use temper_service::clients::LINK_SECRET_BYTES;

        let mut empty = Cursor::new([]);
        let (printed, local) = super::given_secret(&mut empty).expect("empty input");
        assert!(printed.is_none() && local.is_none());
        let bytes = [23; LINK_SECRET_BYTES];
        let mut whole = Cursor::new(bytes);
        let (printed, local) = super::given_secret(&mut whole).expect("whole secret");
        assert!(printed.expect("given secret").expose() == bytes, "the secret is taken whole");
        assert!(local.is_none());
        let mut both = Cursor::new([vec![23; LINK_SECRET_BYTES], vec![37; LINK_SECRET_BYTES]].concat());
        let (printed, local) = super::given_secret(&mut both).expect("both given secrets");
        assert!(printed.expect("printed secret").expose() == bytes, "the printed secret comes first");
        assert!(local.expect("local secret").expose() == [37; LINK_SECRET_BYTES], "the local secret comes second");
        for length in [LINK_SECRET_BYTES - 1, LINK_SECRET_BYTES + 1, 63, 65] {
            let mut input = Cursor::new(vec![23; length]);
            assert_eq!(
                super::given_secret(&mut input).expect_err("partial or oversized secret"),
                "--box: stdin: expected the first link's secret"
            );
        }
        let mut oversized = Cursor::new(vec![23; LINK_SECRET_BYTES * 3]);
        assert!(super::given_secret(&mut oversized).is_err(), "oversized input is refused");
        assert_eq!(oversized.position(), 65);
    }
}
