//! The owner-shaped pairs called by generated maps (skein/docs/design/data/codec/maps.md, 4.2).
pub(crate) use super::change::{
    authority_hold_to_domain, authority_hold_to_record, change_to_domain, change_to_record, delegate_ref_to_domain,
    delegate_ref_to_record, landing_below_to_domain, landing_below_to_record, llm_choice_to_domain,
    llm_choice_to_record, route_to_domain, route_to_record, stands_to_domain, stands_to_record,
};
pub(crate) use super::follows::{follow_files_to_domain, follow_files_to_record};
pub(crate) use super::history::{
    history_job_outcome_to_domain, history_job_outcome_to_record, history_job_record_to_domain,
    history_job_record_to_record,
};
pub(crate) use super::landing::{
    landing_leave_to_domain, landing_leave_to_record, landing_search_tests_to_domain, landing_search_tests_to_record,
    landing_state_to_domain, landing_state_to_record,
};
pub(crate) use super::landing_log::{logged_landing_to_domain, logged_landing_to_record};
pub(crate) use super::landing_log::{logged_verdict_to_domain, logged_verdict_to_record};
pub(crate) use super::tiers::post::{tier_posted_blame_to_domain, tier_posted_blame_to_record};
pub(crate) use super::tiers::{failures_to_domain, failures_to_record};
