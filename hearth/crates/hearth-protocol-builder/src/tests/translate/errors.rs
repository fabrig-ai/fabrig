use crate::render::errors as render_errors;
use crate::render::{self, revision as target};

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
const DECODE_FIELDS: &[(crate::config::Field, render_errors::Field)] = &[
    (crate::config::Field::Kind, render_errors::Field::Kind),
    (crate::config::Field::Firmware, render_errors::Field::Firmware),
    (crate::config::Field::Hardware, render_errors::Field::Hardware),
    (crate::config::Field::On, render_errors::Field::On),
    (crate::config::Field::Mutable, render_errors::Field::Mutable),
];
#[test]
fn shape_field_faults_translate_each_field() {
    for (source, input) in DECODE_FIELDS {
        assert_eq!(
            crate::translate::errors::fault(
                crate::config::Fault::Shape(crate::config::Shape::Wrong { field: *source }),
                &crate::config::limits::SHIPPED
            ),
            Some((render_errors::Decode::Wrong { field: *input }, None))
        );
    }
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

fn declared_config(value: render_errors::Declared) -> crate::config::Declared {
    match value {
        render_errors::Declared::Unknown { of } => crate::config::Declared::Unknown { of: object_config(of) },
        render_errors::Declared::Later => crate::config::Declared::Later,
        render_errors::Declared::Wrong { expected } => {
            crate::config::Declared::Wrong { expected: expected_config(expected) }
        }
        render_errors::Declared::Missing { key } => crate::config::Declared::Missing { key: required_config(key) },
        render_errors::Declared::Name => crate::config::Declared::Name,
        render_errors::Declared::Doc => crate::config::Declared::Doc,
        render_errors::Declared::Order => crate::config::Declared::Order,
        render_errors::Declared::Optional => crate::config::Declared::Optional,
        render_errors::Declared::Bound => crate::config::Declared::Bound,
        render_errors::Declared::Above => crate::config::Declared::Above,
        render_errors::Declared::Enum => crate::config::Declared::Enum,
        render_errors::Declared::Settings => crate::config::Declared::Settings,
        render_errors::Declared::Secret => crate::config::Declared::Secret,
    }
}

fn object_config(value: render_errors::Object) -> crate::config::Object {
    match value {
        render_errors::Object::Declaration => crate::config::Object::Declaration,
        render_errors::Object::Read => crate::config::Object::Read,
        render_errors::Object::Guard => crate::config::Object::Guard,
        render_errors::Object::Script => crate::config::Object::Script,
        render_errors::Object::Bless => crate::config::Object::Bless,
        render_errors::Object::Entry(map) => crate::config::Object::Entry(declaration_map_config(map)),
        render_errors::Object::Range => crate::config::Object::Range,
        render_errors::Object::Spec => crate::config::Object::Spec,
        render_errors::Object::Export => crate::config::Object::Export,
        render_errors::Object::Type => crate::config::Object::Type,
    }
}

fn expected_config(value: render_errors::Expected) -> crate::config::Expected {
    match value {
        render_errors::Expected::Object => crate::config::Expected::Object,
        render_errors::Expected::String => crate::config::Expected::String,
        render_errors::Expected::Strings => crate::config::Expected::Strings,
        render_errors::Expected::Bool => crate::config::Expected::Bool,
        render_errors::Expected::Int => crate::config::Expected::Int,
        render_errors::Expected::Field(takes) => crate::config::Expected::Field(takes_config(takes)),
        render_errors::Expected::True => crate::config::Expected::True,
        render_errors::Expected::Type => crate::config::Expected::Type,
    }
}

fn required_config(value: render_errors::Required) -> crate::config::Required {
    match value {
        render_errors::Required::If => crate::config::Required::If,
        render_errors::Required::Message => crate::config::Required::Message,
        render_errors::Required::Type => crate::config::Required::Type,
        render_errors::Required::Doc => crate::config::Required::Doc,
        render_errors::Required::Value => crate::config::Required::Value,
    }
}

fn takes_config(value: render_errors::Takes) -> crate::config::entries::Takes {
    match value {
        render_errors::Takes::Text => crate::config::entries::Takes::Text,
        render_errors::Takes::Int => crate::config::entries::Takes::Int,
        render_errors::Takes::Bool => crate::config::entries::Takes::Bool,
        render_errors::Takes::Strings => crate::config::entries::Takes::Strings,
        render_errors::Takes::Range => crate::config::entries::Takes::Range,
    }
}

fn declaration_map_config(value: render::revision::declaration::Map) -> crate::config::entries::Map {
    match value {
        render::revision::declaration::Map::Packages => crate::config::entries::Map::Packages,
        render::revision::declaration::Map::Preseeds => crate::config::entries::Map::Preseeds,
        render::revision::declaration::Map::Initramfs => crate::config::entries::Map::Initramfs,
        render::revision::declaration::Map::Software => crate::config::entries::Map::Software,
        render::revision::declaration::Map::Files => crate::config::entries::Map::Files,
        render::revision::declaration::Map::Users => crate::config::entries::Map::Users,
        render::revision::declaration::Map::Groups => crate::config::entries::Map::Groups,
        render::revision::declaration::Map::Presets => crate::config::entries::Map::Presets,
        render::revision::declaration::Map::Tmpfiles => crate::config::entries::Map::Tmpfiles,
        render::revision::declaration::Map::Data => crate::config::entries::Map::Data,
        render::revision::declaration::Map::Secrets => crate::config::entries::Map::Secrets,
        render::revision::declaration::Map::Identity => crate::config::entries::Map::Identity,
        render::revision::declaration::Map::Cmdline => crate::config::entries::Map::Cmdline,
        render::revision::declaration::Map::Live => crate::config::entries::Map::Live,
    }
}

const DECLARED_FAULTS: &[crate::config::Declared] = &[
    crate::config::Declared::Later,
    crate::config::Declared::Name,
    crate::config::Declared::Doc,
    crate::config::Declared::Order,
    crate::config::Declared::Optional,
    crate::config::Declared::Bound,
    crate::config::Declared::Above,
    crate::config::Declared::Enum,
    crate::config::Declared::Settings,
    crate::config::Declared::Secret,
    crate::config::Declared::Unknown { of: crate::config::Object::Declaration },
    crate::config::Declared::Unknown { of: crate::config::Object::Read },
    crate::config::Declared::Unknown { of: crate::config::Object::Guard },
    crate::config::Declared::Unknown { of: crate::config::Object::Script },
    crate::config::Declared::Unknown { of: crate::config::Object::Bless },
    crate::config::Declared::Unknown { of: crate::config::Object::Range },
    crate::config::Declared::Unknown { of: crate::config::Object::Spec },
    crate::config::Declared::Unknown { of: crate::config::Object::Export },
    crate::config::Declared::Unknown { of: crate::config::Object::Type },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Packages) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Preseeds) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Initramfs) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Software) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Files) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Users) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Groups) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Presets) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Tmpfiles) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Data) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Secrets) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Identity) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Cmdline) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Live) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Object },
    crate::config::Declared::Wrong { expected: crate::config::Expected::String },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Strings },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Bool },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Int },
    crate::config::Declared::Wrong { expected: crate::config::Expected::True },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Type },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Text) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Int) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Bool) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Strings) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Range) },
    crate::config::Declared::Missing { key: crate::config::Required::If },
    crate::config::Declared::Missing { key: crate::config::Required::Message },
    crate::config::Declared::Missing { key: crate::config::Required::Type },
    crate::config::Declared::Missing { key: crate::config::Required::Doc },
    crate::config::Declared::Missing { key: crate::config::Required::Value },
];
#[test]
fn each_declared_fault_translates_to_render_and_back() {
    for (row, &input) in DECLARED_FAULTS.iter().enumerate() {
        let rendered = crate::translate::errors::declared(input);
        let back = declared_config(rendered);
        assert_eq!(back, input, "row {row}: declaration fault");
    }
}
