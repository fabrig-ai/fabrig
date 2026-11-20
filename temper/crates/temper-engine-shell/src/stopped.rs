//! Configuration and drive shared by stopped deployment tools (protocol/checkpoints.md, 2).
use crate::setup;
use skein_shell::{Clock, Config, Kernel};
use temper_engine_service as service;
use temper_service_shell::jig_directories;
use temper_shell::directories::Directories;

/// No worker root, listener or external system is started.
pub fn prepare(_binary: &str, directories: &Directories) -> Result<(service::Service, Box<[u8]>), String> {
    prepare_with(directories, None::<(&'static str, fn(&mut service::store::Store) -> Result<(), String>)>)
}
/// The check's engine takes the limits a start would take from its copy
/// (configuration/limit-settings.md, 3): once the configuration is checked, `adopt`
/// gives a store at the stock's figures the copy's roots, the copy opens, and its two
/// kinds resolve before the engine is sized. A copy that doesn't open is reported by
/// an engine at the stock's figures, or `limits-held`'s.
pub fn prepare_rehearsal(
    directories: &Directories,
    build_name: &'static str,
    adopt: impl FnOnce(&mut service::store::Store) -> Result<(), String>,
) -> Result<(service::Service, Box<[u8]>), String> {
    prepare_with(directories, Some((build_name, adopt)))
}
fn prepare_with(
    directories: &Directories,
    rehearsal: Option<(&'static str, impl FnOnce(&mut service::store::Store) -> Result<(), String>)>,
) -> Result<(service::Service, Box<[u8]>), String> {
    let read = temper_service_shell::configuration::read(directories, temper_config_shell::Shape::EngineAlone)?;
    read.prepare(|bytes| {
        let held = crate::serve::limits_hold(crate::path(&directories.data))?;
        let providers = temper_service_shell::providers::read(
            &crate::path(&directories.data).join("providers.setting"),
            service::stock::providers(),
            &mut std::io::stderr(),
        );
        let given = service::stock::Given::Stock.with_providers(providers, Box::new([]));
        let mut named = setup::check_with(bytes, &jig_directories(directories), &given)?.named;
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
        let engine = crate::serve::settings(bytes, directories, &given);
        let accounts = crate::serve::accounts(&engine)?;
        let seeds = crate::serve::seeds(None, None)?;
        let inputs = service::startup::LimitInputs::new(
            bytes,
            &jig_directories(directories),
            &files,
            None,
            &seeds,
            &given,
            false,
            None,
        );
        let (jig, temper) = held.as_ref().map_or((&[][..], &[][..]), |held| (held.jig.as_ref(), held.temper.as_ref()));
        let stock = service::startup::resolve_limits(&inputs, jig, temper).map_err(|why| {
            let line = String::from_utf8_lossy(&service::startup::lines(&why)).into_owned();
            held.as_ref().map_or_else(|| line.clone(), |held| crate::serve::held_failure(&line, held))
        })?;
        let mut opening = match rehearsal {
            Some((build_name, adopt)) => {
                let models = jig_protocol_config::core(&engine, seeds.deployment, seeds.seed).settings.models;
                Some(crate::serve::rehearse_limits(inputs, &models, held.as_ref(), build_name, adopt)?)
            }
            None => None,
        };
        if let Some(held) = &held {
            eprintln!("{}", held.line);
        }
        let resolved = match &opening {
            Some(crate::serve::Opening { settings: Some(settings), .. }) => &settings.in_force,
            Some(crate::serve::Opening { settings: None, .. }) | None => &stock,
        };
        let built = service::startup::checked_start_with_resolved(
            bytes,
            &jig_directories(directories),
            &files,
            None,
            &seeds,
            skein_lib::List::with_capacity(0),
            None,
            &given,
            resolved,
        )
        .and_then(|service::startup::CheckedStart { engine, io, forges, .. }| {
            service::startup::restoring_prepared(engine, &seeds, Some(accounts), forges, io)
        });
        let mut service = match built {
            Ok(service) => service,
            Err(why) => {
                if let Some(opening) = &mut opening {
                    crate::serve::close_limits_store(&mut opening.store)?;
                }
                return Err(String::from_utf8_lossy(&service::startup::lines(&why)).into_owned());
            }
        };
        if let Some(crate::serve::Opening { store, settings, began }) = opening {
            service.engine_component().rehearse_opened(store, settings, began);
        }
        service.configure_forge_runtime(github, roots);
        service.configure_google_endpoints(google);
        if let Some(bundle) = bundle {
            service.adopt_bundle(bundle);
        }
        Ok((service, named.deployment))
    })
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
