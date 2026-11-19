//! Wire projections of specification facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, w};
use super::wire_document_text;
pub(in crate::codec) fn wire_specification_document(
    value: &c::SpecificationDocument,
    limits: &w::Limits,
) -> Result<w::SpecificationDocument, w::Problem> {
    Ok(w::SpecificationDocument {
        specification: wire_document_specification(&value.specification, limits)?,
        cut: value.cut,
    })
}

pub(in crate::codec) fn wire_document_specification(
    value: &c::DocumentSpecification,
    limits: &w::Limits,
) -> Result<w::DocumentSpecification, w::Problem> {
    Ok(w::DocumentSpecification {
        words: wire_document_text(&value.words, limits)?,
        parameters: list(value.parameters.iter().map(|item| wire_parameter_document(item, limits)))?,
        inputs: list(value.inputs.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
        asks: value.asks,
    })
}

pub(in crate::codec) fn wire_parameter_document(
    value: &c::ParameterDocument,
    limits: &w::Limits,
) -> Result<w::ParameterDocument, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ParameterDocument::Number { name, value } => {
            w::ParameterDocument::Number(w::ParameterDocumentNumber { name: *(name), value: *(value) })
        }
        c::ParameterDocument::Bytes { name, value } => w::ParameterDocument::Bytes(w::ParameterDocumentBytes {
            name: *(name),
            value: wire_document_text(value, limits)?,
        }),
        c::ParameterDocument::Resource { name, connector, resource } => {
            w::ParameterDocument::Resource(w::ParameterDocumentResource {
                name: *(name),
                connector: *(connector),
                resource: *(resource),
            })
        }
    })
}

pub(in crate::codec) fn wire_specification(
    value: &c::Specification,
    limits: &w::Limits,
) -> Result<w::Specification, w::Problem> {
    Ok(w::Specification {
        words: bytes::copy_of(value.words.bytes()),
        parameters: list(value.parameters.iter().map(|item| wire_parameter(item, limits)))?,
        inputs: list(value.inputs.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
        asks: value.asks,
    })
}

pub(in crate::codec) fn wire_parameter(value: &c::Parameter, _limits: &w::Limits) -> Result<w::Parameter, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Parameter::Number { name, value } => {
            w::Parameter::Number(w::ParameterNumber { name: *(name), value: *(value) })
        }
        c::Parameter::Bytes { name, value } => {
            w::Parameter::Bytes(w::ParameterBytes { name: *(name), value: bytes::copy_of((value).bytes()) })
        }
        c::Parameter::Resource { name, connector, resource } => w::Parameter::Resource(w::ParameterResource {
            name: *(name),
            connector: *(connector),
            resource: *(resource),
        }),
    })
}
