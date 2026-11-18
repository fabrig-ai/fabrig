//! Configuration and drive shared by stopped deployment tools (protocol/checkpoints.md, 2).
use crate::setup;
use skein_shell::{Clock, Config, Kernel};
use temper_engine_service as service;
use temper_service_shell::jig_directories;
use temper_shell::directories::Directories;

/// No worker root, listener or external system is started.
pub fn prepare(binary: &str, directories: &Directories) -> Result<(service::Service, Box<[u8]>), String> {
    let bytes = directories.read()?;
    if matches!(
        service::startup::nearest(&bytes),
        Some(service::startup::Shape::Standalone | service::startup::Shape::WorkerAlone)
    ) {
        return Err(format!("temper.json names a worker, and {binary} runs none"));
    }
    let providers = temper_service_shell::providers::read(
        &crate::path(&directories.data).join("providers.setting"),
        service::stock::providers(),
        &mut std::io::stderr(),
    );
    let given = service::stock::Given::Stock.with_providers(providers, Box::new([]));
    let mut named = setup::check_with(&bytes, &jig_directories(directories), &given)?.named;
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
    let engine = crate::serve::settings(&bytes, directories, &given);
    let accounts = crate::serve::accounts(&engine)?;
    let seeds = crate::serve::seeds(None, None)?;
    let mut service = service::startup::restoring_with(
        &bytes,
        &jig_directories(directories),
        &files,
        None,
        &seeds,
        Some(accounts),
        &given,
    )
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
    copy_providers: bool,
) -> Result<u8, String> {
    let mut kernel = Kernel::open(Config { operations: service.operations() }).map_err(|why| why.to_string())?;
    let mut served = crate::served::SERVED;
    if !copy_providers {
        served.handover = |service| {
            while service.engine_component().take_providers_write().is_some() {}
            crate::served::handover(service.engine_component());
        };
    }
    let mut host = temper_service_shell::Host::for_upgrade(
        binary,
        service,
        served,
        directories.clone(),
        deployment.into(),
        report,
    );
    Ok(match skein_shell::drive(&mut kernel, &Clock::new(), &mut host) {
        skein_io::kernel::Exit::Code(code) => code,
        skein_io::kernel::Exit::Signal(_) => 1,
    })
}
