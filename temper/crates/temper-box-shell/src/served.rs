//! The box's service and adapter (protocol/services.md, 7).
use crate::lines::note_line;
use temper_service as service;
use temper_service_shell::Served;
use temper_worker_shell::lines as worker_lines;

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
        Service::Joined(service) => temper_engine_shell::served::resolve(service.engine_component()),
        Service::Worker(_) => {}
    },
    handover: |service| match service {
        Service::Joined(service) => temper_engine_shell::served::handover(service.engine_component()),
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
        Service::Joined(service) => ending(service, binary, failed, directories, store),
        Service::Worker(service) => worker_lines::worker_service_ending(service, binary, failed, directories, store),
    },
    listening: |service, binary| match service {
        Service::Joined(service) => listening(service, binary),
        Service::Worker(service) => worker_lines::worker_listening_line(service, binary),
    },
    note: |service, binary| match service {
        Service::Joined(service) => service.notes().pop().map(|note| note_line(binary, note)),
        Service::Worker(service) => worker_lines::worker_note_line(service, binary),
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
