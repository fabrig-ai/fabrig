//! Configuration and drive shared by stopped deployment tools (protocol/checkpoints.md, 2).
use crate::setup;
use skein_shell::{Clock, Config, Kernel};
use temper_engine_service as service;
use temper_service_shell::jig_directories;
use temper_shell::directories::Directories;

/// No worker root, listener or external system is started.
pub fn prepare(binary: &str, directories: &Directories) -> Result<(service::Service, Box<[u8]>), String> {
    let bytes = directories.read()?;
    let held = crate::serve::limits_hold(crate::path(&directories.data))?;
    if matches!(
        service::startup::nearest(&bytes),
        Some(service::startup::Shape::Standalone | service::startup::Shape::WorkerAlone)
    ) {
        return Err(format!("temper.json names a worker, and {binary} runs none"));
    }
    let mut named = setup::check(&bytes, &jig_directories(directories))?.named;
    let crate::serve::Inputs { files, google, bundle } = crate::serve::before_root(
        &mut named.files,
        &mut named.asked_files,
        named.sign_in.as_ref(),
        named.bundle.as_deref(),
        named.bundle_limits,
        skein_shell::effective_user(),
        false,
    )?;
    let crate::serve::Runtimes { roots, github } = crate::serve::runtimes(named.test_github.as_ref())?;
    let engine = crate::serve::settings(&bytes, directories, &service::stock::Given::Stock);
    let accounts = crate::serve::accounts(&engine)?;
    let seeds = crate::serve::seeds(None, None)?;
    let inputs = service::startup::LimitInputs::new(
        &bytes,
        &jig_directories(directories),
        &files,
        None,
        &seeds,
        &service::stock::Given::Stock,
        false,
        None,
    );
    let (jig, temper) = held.as_ref().map_or((&[][..], &[][..]), |held| (held.jig.as_ref(), held.temper.as_ref()));
    let resolved = service::startup::resolve_limits(&inputs, jig, temper).map_err(|why| {
        let line = String::from_utf8_lossy(&service::startup::lines(&why)).into_owned();
        held.as_ref().map_or_else(|| line.clone(), |held| crate::serve::held_failure(&line, held))
    })?;
    if let Some(held) = &held {
        eprintln!("{}", held.line);
    }
    let service::startup::CheckedStart { engine, io, forges, .. } = service::startup::checked_start_with_limits(
        &bytes,
        &jig_directories(directories),
        &files,
        None,
        &seeds,
        skein_lib::List::with_capacity(0),
        None,
        &service::stock::Given::Stock,
        &resolved.jig,
        &resolved.temper,
    )
    .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    let mut service = service::startup::restoring_prepared(engine, &seeds, Some(accounts), forges, io)
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    service.configure_forge_runtime(github, roots);
    service.configure_google_endpoints(google);
    if let Some(bundle) = bundle {
        service.adopt_bundle(bundle);
    }
    Ok((service, named.deployment))
}

/// Run only the prepared engine and render its terminal after teardown.
pub fn drive(
    binary: &str,
    directories: &Directories,
    service: service::Service,
    deployment: &[u8],
    report: temper_service_shell::host::UpgradeReport<service::Service>,
) -> Result<u8, String> {
    let mut kernel = Kernel::open(Config { operations: service.operations() }).map_err(|why| why.to_string())?;
    let mut host = temper_service_shell::Host::for_upgrade(
        binary,
        service,
        crate::served::SERVED,
        directories.clone(),
        deployment.into(),
        report,
    );
    Ok(match skein_shell::drive(&mut kernel, &Clock::new(), &mut host) {
        skein_io::kernel::Exit::Code(code) => code,
        skein_io::kernel::Exit::Signal(_) => 1,
    })
}
