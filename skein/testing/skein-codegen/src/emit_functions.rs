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

    pub(crate) fn splits(&self) -> bool {
        self.head
            .iter()
            .chain(self.body.iter().map(|statement| &statement.text))
            .map(|text| text.lines().count())
            .sum::<usize>()
            >= 98
    }
}

fn helper(
    record: &Record,
    method: &Method,
    statement: &Statement,
    index: usize,
    out: &mut String,
    helpers: &mut String,
) {
    let field = &statement.field;
    let method_name = method.name;
    let name = format!("{method_name}_{field}_field");
    let (parameters, arguments, result, finish, call) = match method_name {
        "measure" => {
            let (parameters, arguments) = if statement.text.contains("self.") {
                ("&self, mut size: u32", "self, size")
            } else {
                ("mut size: u32", "size")
            };
            (parameters, arguments, "u32", "size", "size = ")
        }
        "encode" => {
            ("&self, writer: &mut skein_lib::Writer", "self, writer", "Result<(), skein_lib::Overflow>", "Ok(())", "")
        }
        "check" => ("&self, limits: &Limits", "self, limits", "Result<(), Problem>", "Ok(())", ""),
        "decode_from" => {
            let ty = &record.fields.get(index).expect("decode statements follow field order").ty;
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
    let question = if method_name == "measure" { "" } else { "?" };
    out.push_str(&format!("        {call}Self::{name}({arguments}){question};\n"));
    helpers.push_str(&format!(
        "\nimpl {} {{\n    fn {name}({parameters}) -> {result} {{\n{}        {finish}\n    }}\n}}\n",
        record.name, statement.text
    ));
}

/// Packs only dispatch lines in a wrapper that would exceed the function bound.
pub(crate) fn pack_dispatches(source: String, marker: &str) -> String {
    if source.trim_end().lines().count() <= 100 {
        return source;
    }
    let mut out = String::new();
    let mut pending = false;
    for line in source.lines() {
        if line.contains(marker) {
            if pending {
                out.pop().expect("previous dispatch newline");
                out.push(' ');
                out.push_str(line.trim_start());
            } else {
                out.push_str(line);
            }
            pending = !pending;
        } else {
            out.push_str(line);
            pending = false;
        }
        out.push('\n');
    }
    out
}

/// Renders a field's operations together; a split never crosses a field boundary.
pub(crate) fn render(record: &Record, method: &Method, out: &mut String, helpers: &mut String) {
    let mut function = String::new();
    let split = method.splits();
    for head in &method.head {
        if split && method.name == "decode_from" {
            function.push_str(&head.replace("_limits:", "limits:"));
        } else {
            function.push_str(head);
        }
    }
    for (index, statement) in method.body.iter().enumerate() {
        if split {
            helper(record, method, statement, index, &mut function, helpers);
        } else {
            function.push_str(&statement.text);
        }
    }
    match &method.tail {
        Tail::Line(line) => function.push_str(line),
        Tail::Members(members) => {
            let members =
                members.iter().map(|field| format!("{field}: decoded_{field}")).collect::<Vec<_>>().join(", ");
            function.push_str(&format!("        Ok(Self {{ {members} }})\n"));
        }
    }
    function.push_str("    }\n");
    out.push_str(&pack_dispatches(function, "Self::"));
}
