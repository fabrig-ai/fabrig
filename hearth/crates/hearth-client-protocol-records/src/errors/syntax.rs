use hearth_domain_render_vocabulary::{errors::syntax::Expected, evaluate::Filter};
use hearth_domain_render_vocabulary::errors::{Syntax};
//! Template syntax faults (hearth/docs/design/protocol/format/templates.md, 2 to 5).


use super::{Box, Encoder, Lines, field};

pub(super) fn syntax_text(item: &Syntax, lines: &mut Lines) {
    match item {
        Syntax::TooLong => lines.put(b"a template past its length limit"),
        Syntax::Character => lines.put(b"an unexpected character"),
        Syntax::Unterminated => lines.put(b"a string with no end"),
        Syntax::Escape => lines.put(b"an invalid escape"),
        Syntax::Integer => lines.put(b"an integer outside i64"),
        Syntax::TooDeep => lines.put(b"too deep"),
        Syntax::Misplaced => lines.put(b"a misplaced expression"),
        Syntax::UnknownFilter => lines.put(b"an unknown filter"),
        Syntax::UnknownTest => lines.put(b"an unknown test"),
        Syntax::Utf8 => lines.put(b"not UTF-8"),
        Syntax::Return => lines.put(b"a carriage return"),
        Syntax::Unclosed => lines.put(b"a tag with no end"),
        Syntax::UnknownStatement => lines.put(b"an unknown statement"),
        Syntax::Unmatched => lines.put(b"a close with no open block"),
        Syntax::Unended => lines.put(b"a block left open"),
        Syntax::Shadows => lines.put(b"a root of the context bound"),
        Syntax::Interpolation => lines.put(b"a statement in an interpolated string"),
        Syntax::Expected(expected) => {
            lines.put(b"expected ");
            lines.put(expected_name(*expected));
        }
        Syntax::Arity { filter, given } => {
            lines.quote(filter_name(*filter));
            lines.put(b" takes ");
            lines.decimal(u64::from(filter.arity()));
            lines.put(if filter.arity() == 1 { b" argument, given " } else { b" arguments, given " });
            lines.decimal(u64::from(*given));
        }
    }
}
pub(super) fn syntax_json(item: &Syntax, json: &mut Encoder) {
    json.object_start();
    field(json, b"kind", b"syntax");
    match item {
        Syntax::TooLong => field(json, b"fault", b"too_long"),
        Syntax::Character => field(json, b"fault", b"character"),
        Syntax::Unterminated => field(json, b"fault", b"unterminated"),
        Syntax::Escape => field(json, b"fault", b"escape"),
        Syntax::Integer => field(json, b"fault", b"integer"),
        Syntax::TooDeep => field(json, b"fault", b"too_deep"),
        Syntax::Misplaced => field(json, b"fault", b"misplaced"),
        Syntax::UnknownFilter => field(json, b"fault", b"unknown_filter"),
        Syntax::UnknownTest => field(json, b"fault", b"unknown_test"),
        Syntax::Utf8 => field(json, b"fault", b"utf8"),
        Syntax::Return => field(json, b"fault", b"return"),
        Syntax::Unclosed => field(json, b"fault", b"unclosed"),
        Syntax::UnknownStatement => field(json, b"fault", b"unknown_statement"),
        Syntax::Unmatched => field(json, b"fault", b"unmatched"),
        Syntax::Unended => field(json, b"fault", b"unended"),
        Syntax::Shadows => field(json, b"fault", b"shadows"),
        Syntax::Interpolation => field(json, b"fault", b"interpolation"),
        Syntax::Expected(expected) => {
            field(json, b"fault", b"expected");
            field(json, b"expected", expected_name(*expected));
        }
        Syntax::Arity { filter, given } => {
            field(json, b"fault", b"arity");
            field(json, b"filter", filter_name(*filter));
            json.key(b"expected");
            json.unsigned(u64::from(filter.arity()));
            json.key(b"given");
            json.unsigned(u64::from(*given));
        }
    }
    json.object_end();
}


pub(super) fn expected_name(item: Expected) -> &'static [u8] {
    match item {
        Expected::Operand => b"operand",
        Expected::Operator => b"operator",
        Expected::Name => b"name",
        Expected::CloseParen => b")",
        Expected::CloseBracket => b"]",
        Expected::In => b"in",
        Expected::Else => b"else",
        Expected::Statement => b"statement",
        Expected::Assign => b"=",
        Expected::TagEnd => b"tag end",
    }
}


fn filter_name(value: hearth_domain_render_vocabulary::evaluate::Filter) -> &'static [u8] {
    match value {
        Filter::Default => b"default",
        Filter::Join => b"join",
        Filter::Split => b"split",
        Filter::First => b"first",
        Filter::Last => b"last",
        Filter::Length => b"length",
        Filter::Replace => b"replace",
        Filter::Items => b"items",
        Filter::Keys => b"keys",
        Filter::Values => b"values",
        Filter::Sort => b"sort",
        Filter::Tojson => b"tojson",
        Filter::ShellQuote => b"shell_quote",
        Filter::SystemdEscape => b"systemd_escape",
        Filter::SystemdEscapePath => b"systemd_escape_path",
        Filter::Basename => b"basename",
        Filter::Dirname => b"dirname",
    }
}
