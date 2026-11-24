//! Transfer program bindings after memory admission (shell/credentials.md, section 1).
use crate::credentials::{Location, Source, program_source};
use skein_io::kernel::Fd;
use skein_lib::List;
use smith_host_domain::SourceName;
use smith_protocol_program::Programs;

pub(crate) fn hand(
    service: &mut smith_local_service::Service,
    programs: Programs,
    locations: &List<Location>,
    base: Box<[Box<[u8]>]>,
    root: Fd,
    startup_checked: bool,
) -> Result<(), crate::startup::Failure> {
    let mut sources = List::with_capacity(locations.len());
    for location in locations {
        if matches!(location.source, Source::Program { .. }) {
            let source =
                SourceName::new(location.name.as_bytes()).ok_or_else(|| failure("invalid program source name"))?;
            sources.push((location.account, source, startup_checked)).expect("selected program accounts");
        }
    }
    service
        .handed_programs(
            sources,
            programs,
            base,
            root,
            program_source::DETAIL_BYTES,
            smith_agent_service::profile::standard().policy.check_timeout,
        )
        .map_err(|why| failure(&format!("program check binding: {why:?}")))
}

fn failure(detail: &str) -> crate::startup::Failure {
    crate::startup::Failure::Host { detail: detail.into() }
}
