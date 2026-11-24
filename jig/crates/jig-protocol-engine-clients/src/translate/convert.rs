//! The client's authority bit sets (skein/docs/design/data/codec/maps.md, 5).
use alloc::boxed::Box;
use jig_domain_authority as a;
use jig_domain_tasks as t;

#[expect(clippy::trivially_copy_pass_by_ref, reason = "client maps pass domain values by reference")]
pub(crate) fn tools_to_record(value: &t::Tools) -> u64 {
    value.0
}

pub(crate) fn tools_to_domain(value: u64) -> t::Tools {
    t::Tools(value)
}

pub(super) fn scopes_to_record(value: t::Scopes) -> u8 {
    value.0
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "client maps pass domain values by reference")]
pub(crate) fn requests_to_record(value: &a::Requests) -> u16 {
    value.0
}

pub(super) fn requests_to_domain(value: u16) -> a::Requests {
    a::Requests(value)
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "client maps pass domain values by reference")]
pub(crate) fn proposals_to_record(value: &a::Proposals) -> u8 {
    value.0
}

pub(super) fn proposals_to_domain(value: u8) -> a::Proposals {
    a::Proposals(value)
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "client maps pass domain values by reference")]
pub(crate) fn reasons_to_record(value: &t::Reasons) -> u32 {
    value.0
}

pub(crate) fn reasons_to_domain(value: u32) -> t::Reasons {
    t::Reasons(value)
}

pub(crate) fn login_method_to_record(value: &jig_domain_link::LoginMethod) -> Box<[u8]> {
    skein_lib::bytes::copy_of(value.as_bytes())
}
