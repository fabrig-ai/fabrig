//! Preserve the task's next-run wait and its unknown deadlines.
use crate::generated::client;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;

#[must_use]
pub fn waiting(value: Option<&w::TaskRunWait>) -> Option<c::TaskRunWait> {
    Some(client::task_run_wait_to_domain(value?.clone()))
}
