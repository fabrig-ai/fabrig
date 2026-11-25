//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use crate::generated::client;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;

pub(crate) fn wake_policy(value: &w::WakePolicy) -> c::WakePolicy {
    client::wake_policy_to_domain(value.clone())
}
