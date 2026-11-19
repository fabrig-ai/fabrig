//! Complete connector documents and their nested collections.
use super::parts::{
    decode_application_object, decode_clientdocumentdeployment_outbox, decode_clientdocumenthold_effects,
    decode_clientdocumenthold_resources, decode_clientdocumentpanel_effects, decode_clientdocumentpanel_resources,
    decode_clientdocumentproject_resources, decode_description, decode_diagnostic_availability, decode_effect,
    decode_outbox_entry, decode_procedure_state, encode_application_object, encode_clientdocumentdeployment_outbox,
    encode_clientdocumenthold_effects, encode_clientdocumenthold_resources, encode_clientdocumentpanel_effects,
    encode_clientdocumentpanel_resources, encode_clientdocumentproject_resources, encode_description,
    encode_diagnostic_availability, encode_effect, encode_outbox_entry, encode_procedure_state,
};
use super::{Undecodable, d, w};
pub(super) fn encode_client_document(value: &d::ClientDocument) -> w::ClientDocument {
    match value {
        d::ClientDocument::Object { object, now } => {
            w::ClientDocument::Object(w::ClientDocumentObject { object: *object, now: encode_application_object(now) })
        }
        d::ClientDocument::Panel { task, procedure, result, resources, effects } => {
            w::ClientDocument::Panel(w::ClientDocumentPanel {
                task: *task,
                procedure: match procedure {
                    Some(value) => Some(encode_procedure_state(value)),
                    None => None,
                },
                result: match result {
                    Some(value) => Some(*value),
                    None => None,
                },
                resources: encode_clientdocumentpanel_resources(resources),
                effects: encode_clientdocumentpanel_effects(effects),
            })
        }
        d::ClientDocument::Result { task, code } => {
            w::ClientDocument::Result(w::ClientDocumentResult { task: *task, code: *code })
        }
        d::ClientDocument::Effect { proposal, task, effect, description } => {
            w::ClientDocument::Effect(w::ClientDocumentEffect {
                proposal: *proposal,
                task: *task,
                effect: encode_effect(effect),
                description: encode_description(description),
            })
        }
        d::ClientDocument::Outbox { entry, description } => w::ClientDocument::Outbox(w::ClientDocumentOutbox {
            entry: encode_outbox_entry(entry),
            description: encode_description(description),
        }),
        d::ClientDocument::Hold { task, resources, effects } => w::ClientDocument::Hold(w::ClientDocumentHold {
            task: *task,
            resources: encode_clientdocumenthold_resources(resources),
            effects: encode_clientdocumenthold_effects(effects),
        }),
        d::ClientDocument::Project { project, resources } => w::ClientDocument::Project(w::ClientDocumentProject {
            project: *project,
            resources: encode_clientdocumentproject_resources(resources),
        }),
        d::ClientDocument::Deployment { outbox, subject_limit, staged_limit, request_budget, system_last_response } => {
            w::ClientDocument::Deployment(w::ClientDocumentDeployment {
                outbox: encode_clientdocumentdeployment_outbox(outbox),
                subject_limit: *subject_limit,
                staged_limit: *staged_limit,
                request_budget: encode_diagnostic_availability(*request_budget),
                system_last_response: encode_diagnostic_availability(*system_last_response),
            })
        }
    }
}
pub(super) fn decode_client_document(
    value: w::ClientDocument,
    limits: &d::Limits,
) -> Result<d::ClientDocument, Undecodable> {
    Ok(match value {
        w::ClientDocument::Object(value) => {
            let w::ClientDocumentObject { object, now } = value;
            d::ClientDocument::Object { object, now: decode_application_object(now, limits) }
        }
        w::ClientDocument::Panel(value) => {
            let w::ClientDocumentPanel { task, procedure, result, resources, effects } = value;
            d::ClientDocument::Panel {
                task,
                procedure: match procedure {
                    Some(value) => Some(decode_procedure_state(value, limits)?),
                    None => None,
                },
                result: match result {
                    Some(value) => Some(value),
                    None => None,
                },
                resources: decode_clientdocumentpanel_resources(resources, limits)?,
                effects: decode_clientdocumentpanel_effects(effects, limits)?,
            }
        }
        w::ClientDocument::Result(value) => {
            let w::ClientDocumentResult { task, code } = value;
            d::ClientDocument::Result { task, code }
        }
        w::ClientDocument::Effect(value) => {
            let w::ClientDocumentEffect { proposal, task, effect, description } = value;
            d::ClientDocument::Effect {
                proposal,
                task,
                effect: decode_effect(effect, limits)?,
                description: decode_description(description, limits)?,
            }
        }
        w::ClientDocument::Outbox(value) => {
            let w::ClientDocumentOutbox { entry, description } = value;
            d::ClientDocument::Outbox {
                entry: decode_outbox_entry(entry, limits)?,
                description: decode_description(description, limits)?,
            }
        }
        w::ClientDocument::Hold(value) => {
            let w::ClientDocumentHold { task, resources, effects } = value;
            d::ClientDocument::Hold {
                task,
                resources: decode_clientdocumenthold_resources(resources, limits)?,
                effects: decode_clientdocumenthold_effects(effects, limits)?,
            }
        }
        w::ClientDocument::Project(value) => {
            let w::ClientDocumentProject { project, resources } = value;
            d::ClientDocument::Project {
                project,
                resources: decode_clientdocumentproject_resources(resources, limits)?,
            }
        }
        w::ClientDocument::Deployment(value) => {
            let w::ClientDocumentDeployment {
                outbox,
                subject_limit,
                staged_limit,
                request_budget,
                system_last_response,
            } = value;
            d::ClientDocument::Deployment {
                outbox: decode_clientdocumentdeployment_outbox(outbox, limits)?,
                subject_limit,
                staged_limit,
                request_budget: decode_diagnostic_availability(request_budget, limits),
                system_last_response: decode_diagnostic_availability(system_last_response, limits),
            }
        }
    })
}
