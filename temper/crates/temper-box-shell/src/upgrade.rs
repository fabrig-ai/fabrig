//! The box's standalone preflight for an upgrade, without constructing its worker (protocol/binaries.md).
use temper_engine_shell::{serve as engine, upgrade::Request};
use temper_service as service;
use temper_service_shell::jig_directories;
use temper_shell::directories::Directories;

pub(crate) fn run(directories: &Directories, request: &Request) -> Result<u8, String> {
    let (service, deployment) = prepare(directories).map_err(|why| crate::prefixed("temper-box upgrade", &why))?;
    temper_engine_shell::upgrade::run_prepared("temper-box", directories, request, service, &deployment)
}
fn prepare(directories: &Directories) -> Result<(temper_engine_service::Service, Box<[u8]>), String> {
    let bytes = directories.read()?;
    let address = crate::scripted::address(&[])?;
    let given = crate::scripted::given(directories, &service::stock::Given::Stock, address);
    let mut named = crate::setup::check_with(&bytes, &jig_directories(directories), &given)?.named;
    let environment = [(Box::from(crate::scripted::ADDRESS), address.to_string().into_bytes().into_boxed_slice())];
    let agent = crate::serve::configured_agent(&bytes, directories, &environment)?;
    let engine::Inputs { files, google, bundle } = engine::before_root(
        &mut named.files,
        &mut named.asked_files,
        named.sign_in.as_ref(),
        named.bundle.as_deref(),
        named.bundle_limits,
        skein_shell::effective_user(),
        false,
    )?;
    let engine::Runtimes { roots, github } = engine::runtimes(named.test_github.as_ref())?;
    let config = engine::settings(&bytes, directories, &given);
    let accounts = engine::accounts(&config)?;
    let seeds = engine::seeds(None, None)?;
    let mut service = service::startup::restoring_engine_with(
        &bytes,
        &jig_directories(directories),
        &files,
        agent.as_ref().map(|prepared| &prepared.agent),
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
