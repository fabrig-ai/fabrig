//! By-hand note and resource pieces (skein/docs/design/data/codec/maps.md, 3.3).
use super::{decode_numbers, decode_segments, domain, encode_numbers, encode_segments, wire};
use crate::generated::people as mapped;

pub(crate) fn note_change_to_record(
    value: &domain::NoteChange,
    limits: &wire::Limits,
) -> Result<wire::NoteChange, wire::Problem> {
    Ok(match value {
        domain::NoteChange::Correct { description, body, references, recalled } => {
            wire::NoteChange::Correct(wire::NoteChangeCorrect {
                description: description.clone(),
                body: body.clone(),
                references: encode_numbers(
                    references,
                    limits,
                    limits.parameter_bytes_value,
                    limits.note_change_correct_references,
                )?,
                recalled: *recalled,
            })
        }
        domain::NoteChange::Delete { recalled } => {
            wire::NoteChange::Delete(wire::NoteChangeDelete { recalled: *recalled })
        }
    })
}

pub(crate) fn note_change_to_domain(
    value: wire::NoteChange,
    limits: &wire::Limits,
) -> Result<domain::NoteChange, wire::Problem> {
    Ok(match value {
        wire::NoteChange::Correct(value) => {
            let wire::NoteChangeCorrect { description, body, references, recalled } = value;
            domain::NoteChange::Correct {
                description,
                body,
                references: decode_numbers(references, limits, limits.parameter_bytes_value)?,
                recalled,
            }
        }
        wire::NoteChange::Delete(value) => {
            let wire::NoteChangeDelete { recalled } = value;
            domain::NoteChange::Delete { recalled }
        }
    })
}

pub(crate) fn resource_name_to_record(
    value: &domain::ResourceName,
    limits: &wire::Limits,
) -> Result<wire::ResourceName, wire::Problem> {
    let domain::ResourceName { connector, path } = value;
    Ok(wire::ResourceName {
        connector: *connector,
        path: encode_segments(path, limits, limits.parameter_bytes_value, limits.resource_name_path)?,
    })
}

pub(crate) fn resource_name_to_domain(
    value: wire::ResourceName,
    limits: &wire::Limits,
) -> Result<domain::ResourceName, wire::Problem> {
    let wire::ResourceName { connector, path } = value;
    Ok(domain::ResourceName { connector, path: decode_segments(path, limits, limits.parameter_bytes_value)? })
}

pub(crate) fn adoption_to_record(
    value: &domain::Adoption,
    limits: &wire::Limits,
) -> Result<wire::Adoption, wire::Problem> {
    let domain::Adoption { resource, role, options } = value;
    Ok(wire::Adoption {
        resource: resource_name_to_record(resource, limits)?,
        role: mapped::resource_role_to_record(role),
        options: encode_segments(options, limits, limits.parameter_bytes_value, limits.adoption_options)?,
    })
}

pub(crate) fn adoption_to_domain(
    value: wire::Adoption,
    limits: &wire::Limits,
) -> Result<domain::Adoption, wire::Problem> {
    let wire::Adoption { resource, role, options } = value;
    Ok(domain::Adoption {
        resource: resource_name_to_domain(resource, limits)?,
        role: mapped::resource_role_to_domain(role),
        options: decode_segments(options, limits, limits.parameter_bytes_value)?,
    })
}

pub fn encode_ask(value: &domain::Ask, limits: &wire::Limits) -> Result<wire::Ask, wire::Problem> {
    mapped::ask_to_record(value, limits)
}

pub fn decode_ask(value: wire::Ask, limits: &wire::Limits) -> Result<domain::Ask, wire::Problem> {
    mapped::ask_to_domain(value, limits)
}
