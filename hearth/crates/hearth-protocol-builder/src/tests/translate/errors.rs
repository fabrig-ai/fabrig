use crate::render::errors as render_errors;
use crate::render::revision as target;

const CONFIG_FAULTS: &[(crate::config::Fault, render_errors::Decode)] = &[
    (crate::config::Fault::Json(skein_json::tokenizer::Error::Unexpected), render_errors::Decode::Unexpected),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::Trailing), render_errors::Decode::Trailing),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::TooDeep), render_errors::Decode::TooDeep),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::StringTooLong), render_errors::Decode::StringTooLong),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::NumberTooLong), render_errors::Decode::NumberTooLong),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::Number), render_errors::Decode::Number),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::Escape), render_errors::Decode::Escape),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::Surrogate), render_errors::Decode::Surrogate),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::Utf8), render_errors::Decode::Utf8),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::Control), render_errors::Decode::Control),
    (crate::config::Fault::Json(skein_json::tokenizer::Error::Truncated), render_errors::Decode::Truncated),
    (crate::config::Fault::Shape(crate::config::Shape::ListNull), render_errors::Decode::ListNull),
    (crate::config::Fault::NotObject, render_errors::Decode::NotObject),
    (crate::config::Fault::NotInteger, render_errors::Decode::NotInteger),
    (crate::config::Fault::Range, render_errors::Decode::Range),
    (crate::config::Fault::Nul, render_errors::Decode::Nul),
    (crate::config::Fault::Keys, render_errors::Decode::Keys),
    (crate::config::Fault::Note, render_errors::Decode::Note),
    (crate::config::Fault::Shape(crate::config::Shape::Name), render_errors::Decode::Name),
    (crate::config::Fault::Shape(crate::config::Shape::Placement), render_errors::Decode::Placement),
    (crate::config::Fault::Shape(crate::config::Shape::Members), render_errors::Decode::Members),
    (crate::config::Fault::Shape(crate::config::Shape::Member), render_errors::Decode::Member),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::TooLong),
        render_errors::Decode::TooLarge {
            limit: render_errors::Limit::Document,
            max: super::super::fixtures::bounds::DOCUMENT_BYTES,
        },
    ),
];
#[test]
fn each_config_fault_translates_to_its_decode() {
    for (fault, input) in CONFIG_FAULTS {
        assert_eq!(
            crate::translate::errors::fault(fault.clone(), &crate::config::limits::SHIPPED),
            Some((input.clone(), None))
        );
    }
}
#[test]
fn a_shape_field_fault_keeps_its_field() {
    assert_eq!(
        crate::translate::errors::fault(
            crate::config::Fault::Shape(crate::config::Shape::Wrong { field: crate::config::Field::Kind }),
            &crate::config::limits::SHIPPED
        ),
        Some((render_errors::Decode::Wrong { field: render_errors::Field::Kind }, None))
    );
}
#[test]
fn a_duplicate_fault_translates_the_key_and_first_position() {
    let fault = crate::config::Fault::Duplicate {
        key: skein_lib::bytes::copy_of(b"motd"),
        first: crate::config::Position { offset: 1, line: 1, column: 2 },
    };
    assert_eq!(
        crate::translate::errors::fault(fault, &crate::config::limits::SHIPPED),
        Some((
            render_errors::Decode::Duplicate { key: skein_lib::bytes::copy_of(b"motd") },
            Some(target::Position { line: 1, column: 2 })
        ))
    );
}

const SHAPE_FAULTS: &[(crate::config::Shape, render_errors::Decode)] = &[
    (
        crate::config::Shape::Wrong { field: crate::config::Field::Kind },
        render_errors::Decode::Wrong { field: render_errors::Field::Kind },
    ),
    (
        crate::config::Shape::Missing { field: crate::config::Field::Kind },
        render_errors::Decode::Missing { field: render_errors::Field::Kind },
    ),
    (
        crate::config::Shape::Misplaced { field: crate::config::Field::Kind },
        render_errors::Decode::Misplaced { field: render_errors::Field::Kind },
    ),
];
#[test]
fn each_shape_field_fault_translates_its_kind() {
    for (shape, input) in SHAPE_FAULTS {
        assert_eq!(
            crate::translate::errors::fault(
                crate::config::Fault::Shape(shape.clone()),
                &crate::config::limits::SHIPPED
            ),
            Some((input.clone(), None))
        );
    }
}
#[test]
fn a_task_settings_fault_translates_its_task_name() {
    assert_eq!(
        crate::translate::errors::fault(
            crate::config::Fault::Shape(crate::config::Shape::Settings { task: skein_lib::bytes::copy_of(b"motd") }),
            &crate::config::limits::SHIPPED
        ),
        Some((render_errors::Decode::Settings { task: skein_lib::bytes::copy_of(b"motd") }, None))
    );
}
#[test]
fn an_unknown_field_fault_translates_its_field_name() {
    assert_eq!(
        crate::translate::errors::fault(
            crate::config::Fault::Shape(crate::config::Shape::Unknown { field: skein_lib::bytes::copy_of(b"ids") }),
            &crate::config::limits::SHIPPED
        ),
        Some((render_errors::Decode::Unknown { field: skein_lib::bytes::copy_of(b"ids") }, None))
    );
}
