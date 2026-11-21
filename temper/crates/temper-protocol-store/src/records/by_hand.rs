//! The owner-shaped pairs called by generated maps (skein/docs/design/data/codec/maps.md, 4.2).
pub use super::answers::{answer_to_domain, answer_to_record};
pub use super::answers::{protection_to_domain, protection_to_record};
pub use super::answers::{pull_to_domain, pull_to_record};
pub use super::api::{error_to_domain, error_to_record};
pub use super::api::{summary_to_domain, summary_to_record};
pub use super::api::{write_to_domain, write_to_record};
pub use super::application::{core_refused_to_domain, core_refused_to_record};
pub use super::application::{several_repositories_to_domain, several_repositories_to_record};
pub use super::away::{reach_to_domain, reach_to_record};
pub use super::change::{
    authority_hold_to_domain, authority_hold_to_record, change_to_domain, change_to_record, delegate_ref_to_domain,
    delegate_ref_to_record, landing_below_to_domain, landing_below_to_record, llm_choice_to_domain,
    llm_choice_to_record, route_to_domain, route_to_record, stands_to_domain, stands_to_record,
};
pub use super::entries::{read_result_to_domain, read_result_to_record};
pub use super::findings::{finding_areas_to_domain, finding_areas_to_record};
pub use super::findings::{finding_result_to_domain, finding_result_to_record};
pub use super::findings::{finding_signature_to_domain, finding_signature_to_record};
pub use super::follows::{follow_files_to_domain, follow_files_to_record};
pub use super::forges::{forge_link_to_domain, forge_link_to_record};
pub use super::history::{
    history_job_outcome_to_domain, history_job_outcome_to_record, history_job_record_to_domain,
    history_job_record_to_record,
};
pub use super::landing::{
    landing_leave_to_domain, landing_leave_to_record, landing_search_tests_to_domain, landing_search_tests_to_record,
    landing_state_to_domain, landing_state_to_record,
};
pub use super::landing_log::{logged_landing_to_domain, logged_landing_to_record};
pub use super::landing_log::{logged_verdict_to_domain, logged_verdict_to_record};
pub use super::names::{segments_to_domain, segments_to_record};
pub use super::reds::{red_reply_to_domain, red_reply_to_record};
pub use super::reds::{red_to_domain, red_to_record};
pub use super::settings::{project_classes_to_domain, project_classes_to_record};
pub use super::settings::{project_gates_to_domain, project_gates_to_record};
pub use super::settings::{rule_approval_to_domain, rule_approval_to_record};
pub use super::settings::{rule_gate_to_domain, rule_gate_to_record};
pub use super::settings::{rule_pattern_to_domain, rule_pattern_to_record};
pub use super::settings::{setting_value_to_domain, setting_value_to_record};
pub use super::tiers::post::{tier_posted_blame_to_domain, tier_posted_blame_to_record};
pub use super::tiers::{failures_to_domain, failures_to_record};
pub use super::topics::{pending_release_to_domain, pending_release_to_record};
