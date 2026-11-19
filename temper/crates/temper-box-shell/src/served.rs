//! The box's service and adapter (protocol/services.md, 7).
use crate::lines::note_line;
use temper_service as service;
pub use temper_service::Service;
use temper_service_shell::Served;

pub const SERVED: Served<Service> = Served {
    stages: &service::STAGES,
    iterate_stage: service::iterate_stage,
    resolve: |service| temper_engine_shell::served::resolve(service.engine_component()),
    handover: |service| {
        temper_engine_shell::served::handover(service.engine_component());
        keep_settings(service);
    },
    completions: Service::completions,
    submissions: Service::submissions,
    work_pending: Service::work_pending,
    next_mechanism_deadline: Service::next_deadline,
    next_policy_deadline: Service::next_policy_deadline,
    is_empty: Service::is_empty,
    shutdown: Service::shutdown,
    check_wait_boundary: Service::check_wait_boundary,
    adopt_listener: Service::adopt_listener,
    adopt_signals: Service::adopt_signals,
    operations: Service::operations,
    ending,
    listening,
    note: |service, binary| service.notes().pop().map(|note| note_line(binary, note)),
    link: |service| service.links().pop(),
    local_link: |service| service.local_links().pop(),
};

fn listening(service: &mut service::Service, binary: &str) -> Option<(String, bool)> {
    let note = service.listening().pop()?;
    let clients = matches!(
        &note,
        service::Note::Engine(service::EngineNote::Listening { listener: service::Listening::Clients, .. })
    );
    Some((note_line(binary, note), clients))
}
fn ending(
    service: &service::Service,
    binary: &str,
    failed: bool,
    directories: &temper_shell::directories::Directories,
    store: &[u8],
) -> (String, u8) {
    crate::lines::ending_with_binary(binary, service.end(), failed, directories, store, service.unsettled())
}

/// The engine asks; the shell alone replaces its data-directory copy.
pub(crate) fn keep_settings(service: &mut service::Service) {
    while let Some((path, bytes)) = service.engine_component().take_agents_write() {
        if let Err(why) = write_setting(crate::path(&path), &bytes) {
            service.notes().push(service::Note::Engine(temper_engine_service::Note::StartupWarning {
                line: format!("agents.setting: {why}").into_bytes().into_boxed_slice(),
            }));
        }
    }
    while let Some((path, bytes)) = service.engine_component().take_local_write() {
        if let Err(why) = write_setting(crate::path(&path), &bytes) {
            service.notes().push(service::Note::Engine(temper_engine_service::Note::StartupWarning {
                line: format!("local.setting: {why}").into_bytes().into_boxed_slice(),
            }));
        }
    }
}
pub(crate) fn write_setting(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};
    let temporary = path.with_extension("setting.new");
    match std::fs::remove_file(&temporary) {
        Ok(()) => {}
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(why),
    }
    let result = (|| {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        std::fs::File::open(path.parent().expect("data directory"))?.sync_all()
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}
