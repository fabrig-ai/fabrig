#![expect(clippy::format_push_string, reason = "generator assembles reviewed source fragments")]

//! Split generated functions at data/codec/README.md, section 6's line bound.

use alloc::collections::BTreeMap;

use crate::{Record, emit_types::rust_type};

pub(crate) fn fold(signature: &str, name: &str, var: &str, statements: &[String], tail: &str, out: &mut String) {
    out.push_str(signature);
    let mut helpers = String::new();
    if statements.len() < 96 {
        out.extend(statements.iter().map(String::as_str));
    } else {
        for (index, chunk) in statements.chunks(90).enumerate() {
            out.push_str(&format!("    {var} = {name}_{index}(limits, {var})?;\n"));
            helpers.push_str(&format!(
                "\nfn {name}_{index}(limits: &Limits, mut {var}: u64) -> Option<u64> {{\n{}    Some({var})\n}}\n",
                chunk.concat()
            ));
        }
    }
    out.push_str(tail);
    out.push_str(&helpers);
}

pub(crate) struct Statement {
    pub text: String,
    pub boundary: bool,
    pub decoded: Option<String>,
}

impl Statement {
    pub fn new(text: String, boundary: bool) -> Self {
        Self { text, boundary, decoded: None }
    }
}

pub(crate) enum Tail {
    Line(String),
    Members(Vec<Member>),
}

pub(crate) enum Member {
    Field(String),
    Bools(Vec<String>),
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

/// Renders statements once, keeping balanced operations within each helper.
pub(crate) fn render(record: &Record, method: &Method, out: &mut String, helpers: &mut String) {
    out.extend(method.head.iter().map(String::as_str));
    let body = method.body.iter().map(|statement| &statement.text);
    let lines = method.head.iter().skip(1).chain(body).map(|text| text.lines().count()).sum::<usize>();
    let split = lines >= 98;
    let mut chunks = vec![Vec::new()];
    let mut count = 0_usize;
    for statement in &method.body {
        if split && statement.boundary && count >= 70 {
            chunks.push(Vec::new());
            count = 0;
        }
        count = count.checked_add(statement.text.lines().count()).expect("method lines");
        chunks.last_mut().expect("chunk").push(statement);
    }
    let mut decoded = BTreeMap::new();
    let types = record.fields.iter().map(|field| (field.name.as_str(), &field.ty)).collect::<BTreeMap<_, _>>();
    for (index, chunk) in chunks.iter().enumerate() {
        if !split {
            out.extend(chunk.iter().map(|statement| statement.text.as_str()));
            continue;
        }
        let name = method.name;
        let (parameters, arguments, result, finish, call) = match name {
            "measure" => ("&self, mut size: u32", "self, size", "u32", "size", "size = "),
            "encode" => (
                "&self, writer: &mut skein_lib::Writer",
                "self, writer",
                "Result<(), skein_lib::Overflow>",
                "Ok(())",
                "",
            ),
            "check" => ("&self, limits: &Limits", "self, limits", "Result<(), Problem>", "Ok(())", ""),
            "decode_from" => ("limits: &Limits, reader: &mut skein_lib::Reader<'_>", "limits, reader", "", "", ""),
            _ => unreachable!("record method selected"),
        };
        let (mut result, mut finish, mut call) = (result.to_owned(), finish.to_owned(), call.to_owned());
        if name == "decode_from" {
            let fragment = format!("{}Decoded{index}", record.name);
            let fields = chunk.iter().filter_map(|statement| statement.decoded.as_deref()).collect::<Vec<_>>();
            let members = fields.iter().map(|field| format!("{field}: decoded_{field}")).collect::<Vec<_>>().join(", ");
            helpers.push_str(&format!("\n#[derive(Debug)]\nstruct {fragment} {{\n"));
            for field in fields {
                decoded.insert(field, index);
                helpers.push_str(&format!("    {field}: {},\n", rust_type(types.get(field).expect("decoded field"))));
            }
            helpers.push_str("}\n");
            result = format!("Result<{fragment}, Problem>");
            finish = format!("Ok({fragment} {{ {members} }})");
            call = format!("let decoded_{index} = ");
        }
        let question = if name == "measure" { "" } else { "?" };
        out.push_str(&format!("        {call}Self::{name}_{index}({arguments}){question};\n"));
        helpers.push_str(&format!("\nimpl {} {{\n    fn {name}_{index}({parameters}) -> {result} {{\n", record.name));
        helpers.extend(chunk.iter().map(|statement| statement.text.as_str()));
        helpers.push_str(&format!("        {finish}\n    }}\n}}\n"));
    }
    match &method.tail {
        Tail::Line(line) => out.push_str(line),
        Tail::Members(members) => {
            let value = |field: &str| {
                decoded
                    .get(field)
                    .map_or_else(|| format!("decoded_{field}"), |index| format!("decoded_{index}.{field}"))
            };
            let members = members
                .iter()
                .map(|member| match member {
                    Member::Field(field) => format!("{field}: {}", value(field)),
                    Member::Bools(fields) => format!(
                        "skein_bools: [{}]",
                        fields.iter().map(|field| value(field)).collect::<Vec<_>>().join(", ")
                    ),
                })
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("        Ok(Self {{ {members} }})\n"));
        }
    }
    out.push_str("    }\n");
}
