//! Exhaustive config and template faults (hearth/docs/design/domain/render/errors.md, 1).
use crate::{
    config,
    render::{self, errors as target},
};
use skein_json::tokenizer;
#[must_use]
pub fn fault(
    fault: config::Fault,
    limits: &config::limits::Limits,
) -> Option<(target::Decode, Option<render::revision::Position>)> {
    let fault = match fault {
        config::Fault::Json(error) => match error {
            tokenizer::Error::Unexpected => target::Decode::Unexpected,
            tokenizer::Error::Trailing => target::Decode::Trailing,
            tokenizer::Error::TooDeep => target::Decode::TooDeep,
            tokenizer::Error::StringTooLong => target::Decode::StringTooLong,
            tokenizer::Error::NumberTooLong => target::Decode::NumberTooLong,
            tokenizer::Error::Number => target::Decode::Number,
            tokenizer::Error::Escape => target::Decode::Escape,
            tokenizer::Error::Surrogate => target::Decode::Surrogate,
            tokenizer::Error::Utf8 => target::Decode::Utf8,
            tokenizer::Error::Control => target::Decode::Control,
            tokenizer::Error::Truncated => target::Decode::Truncated,
            tokenizer::Error::TooLong => {
                target::Decode::TooLarge { limit: target::Limit::Document, max: u64::from(limits.document) }
            }
            tokenizer::Error::Stream(_) => return None,
        },
        config::Fault::Stream(_) => return None,
        config::Fault::Declaration(value) => target::Decode::Declaration(declared(value)),
        config::Fault::Duplicate { key, first } => {
            return Some((target::Decode::Duplicate { key }, Some(super::documents::position(first))));
        }
        config::Fault::NotObject => target::Decode::NotObject,
        config::Fault::NotInteger => target::Decode::NotInteger,
        config::Fault::Range => target::Decode::Range,
        config::Fault::Nul => target::Decode::Nul,
        config::Fault::Keys => target::Decode::Keys,
        config::Fault::Note => target::Decode::Note,
        config::Fault::Shape(shape) => match shape {
            config::Shape::ListNull => target::Decode::ListNull,
            config::Shape::Name => target::Decode::Name,
            config::Shape::Placement => target::Decode::Placement,
            config::Shape::Members => target::Decode::Members,
            config::Shape::Member => target::Decode::Member,
            config::Shape::Settings { task } => target::Decode::Settings { task },
            config::Shape::Unknown { field } => target::Decode::Unknown { field },
            config::Shape::Wrong { field } => target::Decode::Wrong { field: config_field(field) },
            config::Shape::Missing { field } => target::Decode::Missing { field: config_field(field) },
            config::Shape::Misplaced { field } => target::Decode::Misplaced { field: config_field(field) },
        },
    };
    Some((fault, None))
}
fn config_field(field: config::Field) -> target::Field {
    match field {
        config::Field::Kind => target::Field::Kind,
        config::Field::Firmware => target::Field::Firmware,
        config::Field::Hardware => target::Field::Hardware,
        config::Field::On => target::Field::On,
        config::Field::Mutable => target::Field::Mutable,
    }
}
#[must_use]
pub fn declared(value: config::Declared) -> target::Declared {
    match value {
        config::Declared::Unknown { of } => target::Declared::Unknown { of: object(of) },
        config::Declared::Later => target::Declared::Later,
        config::Declared::Wrong { expected } => target::Declared::Wrong { expected: config_expected(expected) },
        config::Declared::Missing { key } => target::Declared::Missing { key: required(key) },
        config::Declared::Name => target::Declared::Name,
        config::Declared::Doc => target::Declared::Doc,
        config::Declared::Order => target::Declared::Order,
        config::Declared::Optional => target::Declared::Optional,
        config::Declared::Bound => target::Declared::Bound,
        config::Declared::Above => target::Declared::Above,
        config::Declared::Enum => target::Declared::Enum,
        config::Declared::Settings => target::Declared::Settings,
        config::Declared::Secret => target::Declared::Secret,
    }
}

#[must_use]
pub fn object(value: config::Object) -> target::Object {
    match value {
        config::Object::Declaration => target::Object::Declaration,
        config::Object::Read => target::Object::Read,
        config::Object::Guard => target::Object::Guard,
        config::Object::Script => target::Object::Script,
        config::Object::Bless => target::Object::Bless,
        config::Object::Entry(map) => target::Object::Entry(declaration_map(map)),
        config::Object::Range => target::Object::Range,
        config::Object::Spec => target::Object::Spec,
        config::Object::Export => target::Object::Export,
        config::Object::Type => target::Object::Type,
    }
}

#[must_use]
pub fn config_expected(value: config::Expected) -> target::Expected {
    match value {
        config::Expected::Object => target::Expected::Object,
        config::Expected::String => target::Expected::String,
        config::Expected::Strings => target::Expected::Strings,
        config::Expected::Bool => target::Expected::Bool,
        config::Expected::Int => target::Expected::Int,
        config::Expected::Field(takes) => target::Expected::Field(config_takes(takes)),
        config::Expected::True => target::Expected::True,
        config::Expected::Type => target::Expected::Type,
    }
}

#[must_use]
pub fn required(value: config::Required) -> target::Required {
    match value {
        config::Required::If => target::Required::If,
        config::Required::Message => target::Required::Message,
        config::Required::Type => target::Required::Type,
        config::Required::Doc => target::Required::Doc,
        config::Required::Value => target::Required::Value,
    }
}

#[must_use]
pub fn config_takes(value: config::entries::Takes) -> target::Takes {
    match value {
        config::entries::Takes::Text => target::Takes::Text,
        config::entries::Takes::Int => target::Takes::Int,
        config::entries::Takes::Bool => target::Takes::Bool,
        config::entries::Takes::Strings => target::Takes::Strings,
        config::entries::Takes::Range => target::Takes::Range,
    }
}

#[must_use]
pub fn declaration_map(value: config::entries::Map) -> render::revision::declaration::Map {
    match value {
        config::entries::Map::Packages => render::revision::declaration::Map::Packages,
        config::entries::Map::Preseeds => render::revision::declaration::Map::Preseeds,
        config::entries::Map::Initramfs => render::revision::declaration::Map::Initramfs,
        config::entries::Map::Software => render::revision::declaration::Map::Software,
        config::entries::Map::Files => render::revision::declaration::Map::Files,
        config::entries::Map::Users => render::revision::declaration::Map::Users,
        config::entries::Map::Groups => render::revision::declaration::Map::Groups,
        config::entries::Map::Presets => render::revision::declaration::Map::Presets,
        config::entries::Map::Tmpfiles => render::revision::declaration::Map::Tmpfiles,
        config::entries::Map::Data => render::revision::declaration::Map::Data,
        config::entries::Map::Secrets => render::revision::declaration::Map::Secrets,
        config::entries::Map::Identity => render::revision::declaration::Map::Identity,
        config::entries::Map::Cmdline => render::revision::declaration::Map::Cmdline,
        config::entries::Map::Live => render::revision::declaration::Map::Live,
    }
}

#[must_use]
pub fn syntax(value: hearth_protocol_template::error::ErrorKind) -> target::Syntax {
    use hearth_protocol_template::error::ErrorKind as Source;
    match value {
        Source::TooLong => target::Syntax::TooLong,
        Source::Character => target::Syntax::Character,
        Source::Unterminated => target::Syntax::Unterminated,
        Source::Escape => target::Syntax::Escape,
        Source::Integer => target::Syntax::Integer,
        Source::TooDeep => target::Syntax::TooDeep,
        Source::Misplaced => target::Syntax::Misplaced,
        Source::UnknownFilter => target::Syntax::UnknownFilter,
        Source::UnknownTest => target::Syntax::UnknownTest,
        Source::Utf8 => target::Syntax::Utf8,
        Source::Return => target::Syntax::Return,
        Source::Unclosed => target::Syntax::Unclosed,
        Source::UnknownStatement => target::Syntax::UnknownStatement,
        Source::Unmatched => target::Syntax::Unmatched,
        Source::Unended => target::Syntax::Unended,
        Source::Shadows => target::Syntax::Shadows,
        Source::Interpolation => target::Syntax::Interpolation,
        Source::Expected(value) => target::Syntax::Expected(syntax_expected(value)),
        Source::Arity { filter, given } => target::Syntax::Arity { filter: super::operations::filter(filter), given },
    }
}
#[must_use]
pub fn syntax_expected(value: hearth_protocol_template::error::Expected) -> target::syntax::Expected {
    use hearth_protocol_template::error::Expected as Source;
    match value {
        Source::Operand => target::syntax::Expected::Operand,
        Source::Operator => target::syntax::Expected::Operator,
        Source::Name => target::syntax::Expected::Name,
        Source::CloseParen => target::syntax::Expected::CloseParen,
        Source::CloseBracket => target::syntax::Expected::CloseBracket,
        Source::In => target::syntax::Expected::In,
        Source::Else => target::syntax::Expected::Else,
        Source::Statement => target::syntax::Expected::Statement,
        Source::Assign => target::syntax::Expected::Assign,
        Source::TagEnd => target::syntax::Expected::TagEnd,
    }
}
