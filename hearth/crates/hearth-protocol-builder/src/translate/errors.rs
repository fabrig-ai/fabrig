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
        config::Fault::Declaration(value) => target::Decode::Declaration(value),
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
            config::Shape::Wrong { field } => target::Decode::Wrong { field },
            config::Shape::Missing { field } => target::Decode::Missing { field },
            config::Shape::Misplaced { field } => target::Decode::Misplaced { field },
        },
    };
    Some((fault, None))
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
