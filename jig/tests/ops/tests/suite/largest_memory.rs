//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        jig_ops_client_protocol_wire::SCHEMA,
        jig_ops_client_protocol_wire::v1, sealed,
        limits [
            infrastructure_service_environment,
            infrastructure_service_name,
            infrastructure_environment_pool,
            infrastructure_environment_name,
            infrastructure_pool_value,
            infrastructure_effect_rollback_from,
            infrastructure_effect_rollback_to,
            infrastructure_description_resources,
            infrastructure_service_fact_version,
            infrastructure_drift_account_by_some,
            infrastructure_value_panel_resources,
            infrastructure_value_panel_effects,
            infrastructure_value_result_resources,
            infrastructure_value_hold_resources,
            infrastructure_value_hold_drift,
            infrastructure_value_hold_effects,
            infrastructure_value_project_resources,
            infrastructure_value_project_environments,
            infrastructure_value_deployment_effects,
            observability_service_environment,
            observability_service_name,
            observability_fact_load,
            observability_incident_alerts,
            observability_incident_timeline,
            observability_effect_rule,
            observability_value_deployment_effects,
            observability_value_project_services,
            observability_value_project_incidents,
        ],
        roots [
            ObservabilitySubjectRecord,
            InfrastructureAskRecord,
            InfrastructureAnswerRecord,
            InfrastructureSubjectRecord,
            ObservabilityDocumentRecord,
            InfrastructureDocumentRecord,
            InfrastructureReadAnswer,
            ObservabilityReadAnswer,
        ]
    );
}
