//! The asking family's see call (temper/docs/design/domain/agent.md, 8.4.1).

use super::{Domain, ReplyTo};

pub(crate) fn call(domain: &mut Domain, to: ReplyTo) {
    super::host_route::rendered(
        domain,
        to,
        jig_domain_core::SettledAnswer::Host { error: true, body: b"see: not there yet.".as_slice().into() },
    );
}
