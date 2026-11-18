#![expect(clippy::format_push_string, reason = "generator assembles reviewed source fragments")]

//! Split generated methods by field (temper/docs/design/draft/codec-shape.md, 6.1).

use crate::{Record, emit_codec::uses_limits, emit_types::rust_type};

pub(crate) struct Statement {
    pub field: String,
    pub text: String,
}

impl Statement {
    pub fn new(field: &str, text: String) -> Self {
        Self { field: field.into(), text }
    }
}

pub(crate) enum Tail {
    Line(String),
    Members(Vec<String>),
}

pub(crate) struct Method {
    pub name: &'static str,
    pub head: Vec<String>,
    pub body: Vec<Statement>,
    pub tail: Tail,
}

impl Method {
    pub fn new(name: &'static str, signature: &str, tail: &str) -> Self {
        Self { name, head: vec![signature.into()], body: Vec::new(), tail: Tail::Line(tail.into()) }
    }
}

fn helper(record: &Record, method: &str, statement: &Statement, out: &mut String, helpers: &mut String) {
    let field = &statement.field;
    let name = format!("{method}_{field}_field");
    let (parameters, arguments, result, finish, call) = match method {
        "measure" => {
            let (parameters, arguments) = if statement.text.contains("self.") {
                ("&self, mut size: u32", "self, size")
            } else {
                ("mut size: u32", "size")
            };
            (parameters, arguments, "u32", "size", "size = ")
        }
        "encode" => (
            "&self, writer: &mut skein_lib::Writer",
            "self, writer",
            "Result<(), skein_lib::Overflow>",
            "Ok(())",
            "",
        ),
        "check" => ("&self, limits: &Limits", "self, limits", "Result<(), Problem>", "Ok(())", ""),
        "decode_from" => {
            let ty = &record.fields.iter().find(|candidate| candidate.name == *field).expect("decoded field").ty;
            let limits = if uses_limits(ty) { "limits" } else { "_limits" };
            out.push_str(&format!("        let decoded_{field} = Self::{name}(limits, reader)?;\n"));
            helpers.push_str(&format!(
                "\nimpl {} {{\n    fn {name}({limits}: &Limits, reader: &mut skein_lib::Reader<'_>) -> Result<{}, Problem> {{\n{}        Ok(decoded_{field})\n    }}\n}}\n",
                record.name, rust_type(ty), statement.text
            ));
            return;
        }
        _ => unreachable!("record method selected"),
    };
    let question = if method == "measure" { "" } else { "?" };
    out.push_str(&format!("        {call}Self::{name}({arguments}){question};\n"));
    helpers.push_str(&format!(
        "\nimpl {} {{\n    fn {name}({parameters}) -> {result} {{\n{}        {finish}\n    }}\n}}\n",
        record.name, statement.text
    ));
}

/// Renders a field's operations together; a split never crosses a field boundary.
pub(crate) fn render(record: &Record, method: &Method, out: &mut String, helpers: &mut String) {
    out.extend(method.head.iter().map(String::as_str));
    let lines = method.head.iter().chain(method.body.iter().map(|statement| &statement.text))
        .map(|text| text.lines().count()).sum::<usize>();
    let split = lines >= 98;
    for statement in &method.body {
        if split {
            helper(record, method.name, statement, out, helpers);
        } else {
            out.push_str(&statement.text);
        }
    }
    match &method.tail {
        Tail::Line(line) => out.push_str(line),
        Tail::Members(members) => {
            let members = members.iter().map(|field| format!("{field}: decoded_{field}")).collect::<Vec<_>>().join(", ");
            out.push_str(&format!("        Ok(Self {{ {members} }})\n"));
        }
    }
    out.push_str("    }\n");
}
