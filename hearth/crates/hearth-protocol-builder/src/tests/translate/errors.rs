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
fn a_decode_fault_passes_through_unchanged_without_a_position() {
    let decode = render_errors::Decode::Settings { task: skein_lib::bytes::copy_of(b"motd") };
    assert_eq!(
        crate::translate::errors::fault(crate::config::Fault::Decode(decode.clone()), &crate::config::limits::SHIPPED),
        Some((decode, None))
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
