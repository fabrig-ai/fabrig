//! The binary's service adapter (protocol/services.md, 7).
use temper_service as service;
use temper_service_shell::{Served, host};
pub const SERVED: Served<service::Service> = Served {
    stages: &service::STAGES,
    iterate_stage: service::iterate_stage,
    resolve: |_| {},
    completions: service::Service::completions,
    submissions: service::Service::submissions,
    operations: service::Service::operations,
    work_pending: service::Service::work_pending,
    next_mechanism_deadline: service::Service::next_deadline,
    next_policy_deadline: service::Service::next_policy_deadline,
    is_empty: service::Service::is_empty,
    shutdown: service::Service::shutdown,
    ending: host::service_ending,
    handover: |_| {},
    listening: host::listening_line,
    note: host::next_note_line,
    link: |service| service.links().pop(),
    local_link: |service| service.local_links().pop(),
    check_wait_boundary: service::Service::check_wait_boundary,
    adopt_listener: service::Service::adopt_listener,
    adopt_signals: service::Service::adopt_signals,
};
