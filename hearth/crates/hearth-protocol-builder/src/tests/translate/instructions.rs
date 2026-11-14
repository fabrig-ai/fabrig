use crate::{render::evaluate::template as render, translate};
use hearth_protocol_template::{self as template, instruction as source};
#[test]
fn each_instruction_translates_to_render_and_back() {
    for (row, does) in super::super::fixtures::every_does().into_iter().enumerate() {
        let input = source::Instruction { at: 17, does };
        let translated = translate::instructions::instruction(input.clone());
        let back = source::Instruction { at: translated.at, does: does_back(translated.does) };
        assert_eq!(back, input, "row {row}: instruction and offset");
    }
}
fn does_back(value: render::Does) -> source::Does {
    use super::back::expression;
    match value {
        render::Does::Text(bytes) => source::Does::Text(bytes),
        render::Does::Output(value) => source::Does::Output(expression(value)),
        render::Does::Branch { condition, over } => source::Does::Branch { condition: expression(condition), over },
        render::Does::Jump { over } => source::Does::Jump { over },
        render::Does::Enter { names, iterable, over } => {
            source::Does::Enter { names: names_back(names), iterable: expression(iterable), over }
        }
        render::Does::Advance { back } => source::Does::Advance { back },
        render::Does::Set { name, value } => source::Does::Set { name, value: expression(value) },
    }
}
fn names_back(value: render::Names) -> source::Names {
    match value {
        render::Names::One { name } => source::Names::One { name },
        render::Names::Pair { first, second } => source::Names::Pair { first, second },
    }
}
#[test]
fn each_syntax_fault_translates_to_render_and_back() {
    for (row, input) in super::super::fixtures::every_syntax().into_iter().enumerate() {
        let back = syntax_back(translate::errors::syntax(input.clone()));
        assert_eq!(back, input, "row {row}: syntax fault");
    }
}
fn syntax_back(value: crate::render::errors::Syntax) -> template::error::ErrorKind {
    use crate::render::errors::Syntax as E;
    use template::error::ErrorKind as T;
    match value {
        E::TooLong => T::TooLong,
        E::Character => T::Character,
        E::Unterminated => T::Unterminated,
        E::Escape => T::Escape,
        E::Integer => T::Integer,
        E::TooDeep => T::TooDeep,
        E::Misplaced => T::Misplaced,
        E::UnknownFilter => T::UnknownFilter,
        E::UnknownTest => T::UnknownTest,
        E::Utf8 => T::Utf8,
        E::Return => T::Return,
        E::Unclosed => T::Unclosed,
        E::UnknownStatement => T::UnknownStatement,
        E::Unmatched => T::Unmatched,
        E::Unended => T::Unended,
        E::Shadows => T::Shadows,
        E::Interpolation => T::Interpolation,
        E::Expected(value) => T::Expected(expected_back(value)),
        E::Arity { filter, given } => T::Arity { filter: super::back::filter(filter), given },
    }
}
fn expected_back(value: crate::render::errors::syntax::Expected) -> template::error::Expected {
    use crate::render::errors::syntax::Expected as E;
    use template::error::Expected as T;
    match value {
        E::Operand => T::Operand,
        E::Operator => T::Operator,
        E::Name => T::Name,
        E::CloseParen => T::CloseParen,
        E::CloseBracket => T::CloseBracket,
        E::In => T::In,
        E::Else => T::Else,
        E::Statement => T::Statement,
        E::Assign => T::Assign,
        E::TagEnd => T::TagEnd,
    }
}
