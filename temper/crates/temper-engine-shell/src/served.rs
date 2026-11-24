//! The binary's service adapter (protocol/services.md, 7).
use crate::lines;
use temper_engine_service as service;
use temper_service_shell::Served;
pub const SERVED: Served<service::Service> = Served {
    set_local_places: None,
    stages: &service::STAGES,
    iterate_stage: service::iterate_stage,
    resolve: |service| resolve(service.engine_component()),
    completions: service::Service::completions,
    submissions: service::Service::submissions,
    operations: service::Service::operations,
    work_pending: service::Service::work_pending,
    next_mechanism_deadline: service::Service::next_deadline,
    next_policy_deadline: service::Service::next_policy_deadline,
    is_empty: service::Service::is_empty,
    shutdown: service::Service::shutdown,
    ending: lines::service_ending,
    handover: |service| handover(service.engine_component()),
    listening: lines::listening_line,
    note: lines::next_note_line,
    link: |service| service.links().pop(),
    local_link: |service| service.local_links().pop(),
    check_wait_boundary: service::Service::check_wait_boundary,
    adopt_listener: service::Service::adopt_listener,
    adopt_signals: service::Service::adopt_signals,
};

/// Resolve the engine's forge names between its stages and drain.
pub fn resolve(service: &mut service::Engine) {
    for (forge, name, port) in service.unresolved() {
        let address = std::str::from_utf8(&name)
            .ok()
            .and_then(|name| std::net::ToSocketAddrs::to_socket_addrs(&(name, port)).ok()?.next());
        service.resolved(forge, address);
    }
    for (token, name, port) in service.unresolved_pushes() {
        let address = std::str::from_utf8(&name)
            .ok()
            .and_then(|name| std::net::ToSocketAddrs::to_socket_addrs(&(name, port)).ok()?.next());
        service.resolved_push(token, address);
    }
    if let Some((name, port)) = service.unresolved_github() {
        let address = std::str::from_utf8(&name)
            .ok()
            .and_then(|name| std::net::ToSocketAddrs::to_socket_addrs(&(name, port)).ok()?.next());
        service.resolved_github(&name, address);
    }
}
/// Read and remove only the handovers the engine asks for.
pub fn handover(service: &mut service::Engine) {
    while let Some((path, providers)) = service.take_providers_write() {
        if let Err(why) = temper_service_shell::providers::write(crate::path(&path), &providers) {
            service.notes().push(service::Note::StartupWarning {
                line: format!("providers.setting: not written: {why}").into_bytes().into_boxed_slice(),
            });
        }
    }
    while let Some(io) = service.take_place_io() {
        match io {
            service::PlaceIo::Choose { request } => {
                let port = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
                    .ok()
                    .and_then(|listener| listener.local_addr().ok())
                    .map(|address| address.port());
                service.place_port(request, port);
            }
            service::PlaceIo::Read { forge, kind, directories } => {
                service.place_file(
                    forge,
                    temper_forge_setup_shell::setup_file::read(&directories, forge, kind)
                        .map_err(|why| why.into_bytes().into_boxed_slice()),
                );
            }
            service::PlaceIo::Write { forge, setup, directories } => {
                let answer = temper_forge_setup_shell::setup_file::write(&directories, forge, &setup)
                    .map(|()| *setup)
                    .map_err(|why| why.into_bytes().into_boxed_slice());
                service.place_file(forge, answer);
            }
        }
    }
    while let Some(read) = service.take_handover_read() {
        let answer = lines::read_handover(&read);
        service.handover_read(read.forge, read.run, answer);
    }
    while let Some(remove) = service.take_handover_removal() {
        let removed = lines::remove_handover(&remove);
        service.handover_removed(remove.forge, remove.run, removed);
    }
}
