//! The binary's service adapter (protocol/services.md, 7).
use temper_service::worker_alone as service;
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
    ending: host::worker_service_ending,
    handover: |_| {},
    listening: host::worker_listening_line,
    note: host::worker_note_line,
    link: |_| None,
    local_link: |_| None,
    check_wait_boundary: service::Service::check_wait_boundary,
    adopt_listener: service::Service::adopt_listener,
    adopt_signals: service::Service::adopt_signals,
};
