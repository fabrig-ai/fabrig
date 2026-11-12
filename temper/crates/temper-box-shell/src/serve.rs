//! The standalone start, composed from the engine and worker shells (protocol/services.md, 7).
use crate::{Service, path, prefixed, program_login, setup};
use skein_io::kernel::{Exit, Fd};
use skein_lib::{List, Secret};
use std::os::unix::fs::FileTypeExt;
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
) -> Result<Service, String> {
    prepare_with_store(binary, environment, working, directories, bytes, owned).map(|(service, _)| service)
}
pub fn prepare_with_store(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
) -> Result<(Service, Box<[u8]>), String> {
    prepare_with_limits(
        binary,
        environment,
        working,
        directories,
        bytes,
        owned,
        false,
        None,
        None,
        &service::stock::Given::Stock,
    )
    .map(|(service, store, _)| (service, store))
}
#[expect(
    clippy::too_many_arguments,
    reason = "the standalone start supplies the binary beside the resolved process facts"
)]
fn prepare_with_limits(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
    under_box: bool,
    link_secret: Option<Secret>,
    local_secret: Option<Secret>,
    given: &service::stock::Given,
) -> Result<(Service, Box<[u8]>, u32), String> {
    if service::startup::shape_with(bytes, &jig_directories(directories), given)
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?
        == service::startup::Shape::WorkerAlone
    {
        return worker::alone(directories, bytes, owned, environment)
            .map(|service| (Service::Worker(service), Box::new([]) as Box<[u8]>, 0));
    }
    let setup::Deployment { mut named, checked, limits, .. } =
        setup::check_with(bytes, &jig_directories(directories), given)?;
    let engine = engine::settings(bytes, directories, given);
    let engine::Runtimes { roots, github } = engine::runtimes(binary, &named.machine)?;
    let user = skein_shell::effective_user();
    let agent = named.worker.as_ref().map(agent::preflight_worker).transpose()?;
    let engine::Inputs { files, google, bundle } = engine::before_root(
        &mut named.files,
        &mut named.asked_files,
        named.sign_in.as_ref(),
        named.bundle.as_deref(),
        named.bundle_limits,
        user,
    )?;
    let runtime = named.worker.as_ref().map(|worker| worker::root(&worker.root, user, owned)).transpose()?;
    let program_login = if named.worker.is_some() {
        program_login::line(directories, &printed::running_word(environment), skein_shell::HOME.value(environment))
            .map(|line| line.into_bytes().into_boxed_slice())
    } else {
        None
    };
    let accounts = engine::accounts(&engine)?;
    let seeds = engine::seeds(link_secret, local_secret)?;
    let mut service = service::startup::with_setup_given(
        bytes,
        &jig_directories(directories),
        &files,
        agent.as_ref(),
        &seeds,
        Some(accounts),
        runtime,
        List::with_capacity(0),
        Some(&checked),
        if under_box { Some(limits.line_bytes) } else { None },
        program_login,
        given,
        engine::program_environment(directories, environment, working, &printed::running_word(environment)),
    )
    .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    service.configure_forge_runtime(github, roots);
    service.configure_google_endpoints(google);
    if let Some(agent) = &agent {
        let worker = named.worker.as_ref().expect("local agent checked");
        let sources = worker::sources(
            agent,
            path(&worker.agent),
            path(&worker.program),
            path(&directories.cache),
            user,
            environment,
        )?;
        if sources.reuse {
            service.reuse_workspaces();
        }
        if let Some(directory) = sources.dump {
            service.dump_llm(&directory);
        }
        if let Some(resources) = sources.resources {
            service.configure_program_sources(resources).map_err(|_| "worker refused program source resources")?;
        }
    }
    engine::adopt(service.engine_component(), &named.deployment, bundle, user, owned)?;
    Ok((Service::Joined(service), named.deployment, limits.line_bytes))
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
    operations_in_mode(binary, environment, working, directories, config, false)
}
/// The same for a boxed start.
pub fn boxed_operations_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
) -> Result<u32, String> {
    operations_in_mode(binary, environment, working, directories, config, true)
}
fn operations_in_mode(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
    boxed: bool,
) -> Result<u32, String> {
    directories.open(skein_shell::effective_user())?;
    let mut owned = Vec::new();
    let result = prepare_with_limits(
        binary,
        environment,
        working,
        directories,
        config,
        &mut owned,
        boxed,
        None,
        None,
        &service::stock::Given::Stock,
    )
    .map(|(service, _, _)| service.operations());
    close(owned);
    result.map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
pub fn prepare_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
) -> Result<Service, String> {
    let bytes = directories.read().map_err(|why| prefixed(&format!("{binary} serve"), &why))?;
    let mut owned = Vec::new();
    let result = prepare(binary, environment, working, directories, &bytes, &mut owned);
    if result.is_err() {
        close(owned);
    }
    result.map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
/// Prepare an in-process world's service with the supplied defaults.
pub fn prepare_in_with(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    given: &service::stock::Given,
) -> Result<Service, String> {
    let bytes = directories.read().map_err(|why| prefixed(&format!("{binary} serve"), &why))?;
    let mut owned = Vec::new();
    let result =
        prepare_with_limits(binary, environment, working, directories, &bytes, &mut owned, false, None, None, given)
            .map(|(service, _, _)| service);
    if result.is_err() {
        close(owned);
    }
    result.map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
/// The same for an in-process world's service, with the supplied defaults.
pub fn operations_in_with(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
    given: &service::stock::Given,
) -> Result<u32, String> {
    directories.open(skein_shell::effective_user())?;
    let mut owned = Vec::new();
    let result =
        prepare_with_limits(binary, environment, working, directories, config, &mut owned, false, None, None, given)
            .map(|(service, _, _)| service.operations());
    close(owned);
    result.map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
pub fn run(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    boxed: bool,
) -> Result<Exit, String> {
    temper_service_shell::run(
        binary,
        directories,
        environment,
        crate::served::SERVED,
        |listeners| box_inputs(boxed, listeners),
        |bytes, owned, input| {
            prepare_with_limits(
                binary,
                environment,
                working,
                directories,
                bytes,
                owned,
                boxed,
                input.link,
                input.local,
                &service::stock::Given::Stock,
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
