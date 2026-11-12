//! The binary's service adapter (protocol/services.md, 7).
use temper_service as service;
use temper_service_shell::{Served, host};
pub const SERVED: Served<service::Service> = Served {
    stages: &service::STAGES,
    iterate_stage: service::iterate_stage,
    resolve,
    completions: service::Service::completions,
    submissions: service::Service::submissions,
    operations: service::Service::operations,
    work_pending: service::Service::work_pending,
    next_mechanism_deadline: service::Service::next_deadline,
    next_policy_deadline: service::Service::next_policy_deadline,
    is_empty: service::Service::is_empty,
    shutdown: service::Service::shutdown,
    ending: host::service_ending,
    handover,
    listening: host::listening_line,
    note: host::next_note_line,
    link: |service| service.links().pop(),
    local_link: |service| service.local_links().pop(),
    check_wait_boundary: service::Service::check_wait_boundary,
    adopt_listener: service::Service::adopt_listener,
    adopt_signals: service::Service::adopt_signals,
};

/// Resolve the engine's forge names between its stages and drain.
pub fn resolve(service: &mut service::Service) {
    for (forge, name, port) in service.unresolved() {
        let address = std::str::from_utf8(&name)
            .ok()
            .and_then(|name| std::net::ToSocketAddrs::to_socket_addrs(&(name, port)).ok()?.next());
        service.resolved(forge, address);
    }
}
/// Read and remove only the handovers the engine asks for.
pub fn handover(service: &mut service::Service) {
    while let Some(read) = service.take_handover_read() {
        let answer = host::read_handover(&read);
        service.handover_read(read.forge, read.run, answer);
    }
    while let Some(remove) = service.take_handover_removal() {
        let removed = host::remove_handover(&remove);
        service.handover_removed(remove.forge, remove.run, removed);
    }
}
