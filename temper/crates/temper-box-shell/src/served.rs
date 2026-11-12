//! The box's service and adapter (protocol/services.md, 7).
use temper_service as service;
use temper_service_shell::{Served, host};

/// A document's service, driven without joining its vocabularies.
#[expect(clippy::large_enum_variant, reason = "the shell owns one service, never a population of variants")]
pub enum Service {
    Joined(service::Service),
    Worker(service::worker_alone::Service),
}
impl Service {
    #[must_use]
    pub fn operations(&self) -> u32 {
        match self {
            Self::Joined(service) => service.operations(),
            Self::Worker(service) => service.operations(),
        }
    }
    /// A worker has no box pipe; refusal retains both descriptors with the caller.
    pub fn adopt_box(
        &mut self,
        requests: skein_io::kernel::Fd,
        answers: skein_io::kernel::Fd,
        line_bytes: u32,
    ) -> Result<(), service::system::BoxRefusal> {
        match self {
            Self::Joined(service) => service.adopt_box(requests, answers, line_bytes),
            Self::Worker(_) => Err(service::system::BoxRefusal::NoEngine),
        }
    }
}

pub const SERVED: Served<Service> = Served {
    stages: &service::STAGES,
    iterate_stage: |service, stage, now, wall| match service {
        Service::Joined(service) => service::iterate_stage(service, stage, now, wall),
        Service::Worker(service) => service::worker_alone::iterate_stage(service, stage, now, wall),
    },
    resolve: |service| match service {
        Service::Joined(service) => temper_engine_shell::served::resolve(service),
        Service::Worker(_) => {}
    },
    handover: |service| match service {
        Service::Joined(service) => temper_engine_shell::served::handover(service),
        Service::Worker(_) => {}
    },
    completions: |service| match service {
        Service::Joined(service) => service.completions(),
        Service::Worker(service) => service.completions(),
    },
    submissions: |service| match service {
        Service::Joined(service) => service.submissions(),
        Service::Worker(service) => service.submissions(),
    },
    work_pending: |service, now| match service {
        Service::Joined(service) => service.work_pending(now),
        Service::Worker(service) => service.work_pending(now),
    },
    next_mechanism_deadline: |service| match service {
        Service::Joined(service) => service.next_deadline(),
        Service::Worker(service) => service.next_deadline(),
    },
    next_policy_deadline: |service| match service {
        Service::Joined(service) => service.next_policy_deadline(),
        Service::Worker(service) => service.next_policy_deadline(),
    },
    is_empty: |service| match service {
        Service::Joined(service) => service.is_empty(),
        Service::Worker(service) => service.is_empty(),
    },
    shutdown: |service| match service {
        Service::Joined(service) => service.shutdown(),
        Service::Worker(service) => service.shutdown(),
    },
    check_wait_boundary: |service, now| match service {
        Service::Joined(service) => service.check_wait_boundary(now),
        Service::Worker(service) => service.check_wait_boundary(now),
    },
    adopt_listener: |service, fd, address| match service {
        Service::Joined(service) => service.adopt_listener(fd, address),
        Service::Worker(service) => service.adopt_listener(fd, address),
    },
    adopt_signals: |service, fd| match service {
        Service::Joined(service) => service.adopt_signals(fd),
        Service::Worker(service) => service.adopt_signals(fd),
    },
    operations: Service::operations,
    ending: |service, binary, failed, directories, store| match service {
        Service::Joined(service) => host::service_ending(service, binary, failed, directories, store),
        Service::Worker(service) => host::worker_service_ending(service, binary, failed, directories, store),
    },
    listening: |service, binary| match service {
        Service::Joined(service) => host::listening_line(service, binary),
        Service::Worker(service) => host::worker_listening_line(service, binary),
    },
    note: |service, binary| match service {
        Service::Joined(service) => host::next_note_line(service, binary),
        Service::Worker(service) => host::worker_note_line(service, binary),
    },
    link: |service| match service {
        Service::Joined(service) => service.links().pop(),
        Service::Worker(_) => None,
    },
    local_link: |service| match service {
        Service::Joined(service) => service.local_links().pop(),
        Service::Worker(_) => None,
    },
};
