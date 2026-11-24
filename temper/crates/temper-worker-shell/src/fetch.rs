//! Run the figures service to completion before constructing the worker.
use skein_shell::{Clock, Config, Kernel};
use temper_service_shell::{Host, Served};
use temper_shell::directories::{Directories, EnvironmentEntry};
use temper_worker_service::fetch::{self, Fetch};
pub const SERVED: Served<Fetch> = Served {
    set_local_places: None,
    stages: &temper_worker_service::STAGES,
    iterate_stage: fetch::iterate_stage,
    resolve: |_| {},
    completions: Fetch::completions,
    submissions: Fetch::submissions,
    operations: Fetch::operations,
    work_pending: Fetch::work_pending,
    next_mechanism_deadline: Fetch::next_deadline,
    next_policy_deadline: |_| None,
    is_empty: Fetch::is_empty,
    shutdown: Fetch::shutdown,
    ending: |service, _, _, _, _| (String::new(), u8::from(service.figures().is_none())),
    handover: |_| {},
    listening: |_, _| None,
    note: |service, binary| service.notes().pop().map(|note| crate::lines::note_line(binary, note)),
    link: |_| None,
    local_link: |_| None,
    check_wait_boundary: Fetch::check_wait_boundary,
    adopt_listener: |_, fd, _| Err(fd),
    adopt_signals: Fetch::adopt_signals,
};
pub fn take(
    mut service: Fetch,
    directories: &Directories,
    environment: &[EnvironmentEntry],
) -> Result<jig_protocol_worker_wire::Figures, String> {
    let signals = skein_shell::open_termination_signals().map_err(|errno| format!("signals: errno {errno}"))?;
    if let Err(fd) = service.adopt_signals(signals) {
        skein_shell::close_startup_fd(fd);
        return Err("signals refused".into());
    }
    let mut kernel = match Kernel::open(Config { operations: service.operations() }) {
        Ok(kernel) => kernel,
        Err(why) => {
            skein_shell::close_startup_fd(signals);
            return Err(why.to_string());
        }
    };
    let mut host = Host::with_binary("temper-worker", service, SERVED, directories.clone(), environment);
    let _exit = skein_shell::drive(&mut kernel, &Clock::new(), &mut host);
    host.service.take().ok_or_else(|| "stopped before taking the engine's figures".into())
}
