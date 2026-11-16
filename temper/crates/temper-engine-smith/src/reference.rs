//! The host-tool declarations as their reference page
//! (protocol/documentation.md, 8).

use alloc::boxed::Box;
use skein_lib::{List, Overflow, Writer};

mod forms;
use forms::Form;
use smith_domain_run as run;

use crate::{
    calls::{SPECS, Spec},
    json::{self, Value},
};

/// The reference page `host-tools`, whole, as docs/help/reference holds it
/// (protocol/documentation.md, 8).
pub fn reference(out: &mut Writer) -> Result<(), Overflow> {
    render(&SPECS, out)
}

/// The page over any declarations, for the step tests' fixtures.
pub(crate) fn render(specs: &[Spec], out: &mut Writer) -> Result<(), Overflow> {
    let mut schemas = List::with_capacity(u32::try_from(specs.len()).expect("static tool list"));
    for spec in specs {
        let schema = json::parse(spec.whole()).expect("whole tool schema");
        schemas.push(schema).expect("static tool count");
    }
    let schemas = schemas.into_boxed();
    let forms = forms::discover(&schemas);
    out.put(b"# Host tools\n\nKind: reference\n\nEvery tool temper adds to its runs' own: what it does, as a run reads it, whether it reads or writes, and the members of its input.\n\nA run calls only the tools in its own list: a tool here that your list lacks is not yours.\n\nMembers are optional unless marked required. Numeric bounds in brackets are inclusive.\n\nFile, pull-file and compare replies fit min(core.call_answer_bytes minus tool name, agent.json profile.declared.tool_payload); raising the former cannot override the latter. Oversized comparisons give measured size and recovery. Forge refusals name the bound and exact size or at least bytes read; unknown checks say an unnamed bound, and figureless forge bounds say a bound of the forge's. All refusals say how to read on. Fitted file prefixes keep [file truncated] and size/recovery feedback if room permits. max_bytes selects a raw prefix without offset; cuts are incomplete. If metadata leaves no content room, shorten the path or raise the allowance. Oversized pull-file pages keep paths as room permits, distinguishing omitted contents from absent sides. Recover omitted paths by listing head/base directories; read paths at head/base commits, resolving base separately. Later pages list other files.\n")?;
    for (spec, schema) in specs.iter().zip(&schemas) {
        out.put(b"\n## ")?;
        out.put(spec.name)?;
        out.put(b"\n\n")?;
        out.put(spec.description)?;
        out.put(b"\n\n")?;
        out.put(match spec.effect {
            run::HostEffect::Read => b"A read.\n",
            run::HostEffect::Write => b"A write.\n",
        })?;
        spec.offer.words(out)?;
        out.put(b"\n")?;
        if !has_members(schema) {
            out.put(b"\nIts input is `{}`.\n")?;
            continue;
        }
        out.put(b"\n")?;
        members(schema, 0, &forms, out)?;
    }
    let mut heading = false;
    for order in 0..forms.len() {
        for form in &forms {
            if form.order != Some(u32::try_from(order).expect("static form index")) {
                continue;
            }
            if !heading {
                out.put(b"\n## Shared forms\n")?;
                heading = true;
            }
            out.put(b"\n### ")?;
            out.put(&form.name)?;
            out.put(b"\n\n")?;
            if field(&form.schema, b"enum").is_some() {
                member_type(&form.schema, false, out)?;
                out.put(b".\n")?;
            } else {
                members(&form.schema, 0, &forms, out)?;
            }
        }
    }
    Ok(())
}

fn properties(schema: &Value) -> &[(Box<[u8]>, Value)] {
    match field(schema, b"properties") {
        Some(Value::Object(properties)) => properties,
        None => &[],
        Some(Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null) => {
            unreachable!("whole properties are an object")
        }
    }
}

fn has_members(schema: &Value) -> bool {
    !properties(schema).is_empty()
}

fn indent(depth: u32, out: &mut Writer) -> Result<(), Overflow> {
    for _ in 0..depth {
        out.put(b"  ")?;
    }
    Ok(())
}

fn description(schema: &Value, out: &mut Writer) -> Result<(), Overflow> {
    if let Some(Value::String(description)) = field(schema, b"description") {
        out.put(b" ")?;
        out.put(description)?;
    }
    Ok(())
}

fn link(form: &Form, out: &mut Writer) -> Result<(), Overflow> {
    out.put(b" the `")?;
    out.put(&form.name)?;
    out.put(b"` form, under Shared forms.")
}

fn carried<'a>(schema: &Value, forms: &'a [Form]) -> Option<&'a Form> {
    if let Some(form) = forms::shared(schema, forms) {
        return Some(form);
    }
    match field(schema, b"items") {
        Some(items) => carried(items, forms),
        None => None,
    }
}

fn members(schema: &Value, depth: u32, forms: &[Form], out: &mut Writer) -> Result<(), Overflow> {
    for (name, member) in properties(schema) {
        indent(depth, out)?;
        out.put(b"- `")?;
        out.put(name)?;
        out.put(b"`: ")?;
        let form = carried(member, forms);
        member_type(member, form.is_some(), out)?;
        let mut required = false;
        if let Some(Value::Array(names)) = field(schema, b"required") {
            for item in names {
                if let Value::String(text) = item {
                    required |= text == name;
                }
            }
        }
        out.put(if required { b"; required." } else { b"." })?;
        description(member, out)?;
        if let Some(items) = field(member, b"items") {
            description(items, out)?;
        }
        if let Some(form) = form {
            link(form, out)?;
        }
        out.put(b"\n")?;
        if form.is_none() {
            members(member, depth.checked_add(1).expect("schema depth"), forms, out)?;
        }
    }
    if let Some(items) = field(schema, b"items") {
        members(items, depth, forms, out)?;
    }
    for alternative in [b"oneOf".as_slice(), b"anyOf", b"allOf"] {
        if let Some(Value::Array(variants)) = field(schema, alternative) {
            for variant in variants {
                let form = forms::shared(variant, forms);
                if form.is_some() || field(variant, b"enum").is_some() || field(variant, b"description").is_some() {
                    indent(depth, out)?;
                    out.put(b"- Alternative: ")?;
                    member_type(variant, form.is_some(), out)?;
                    out.put(b".")?;
                    description(variant, out)?;
                    if let Some(form) = form {
                        link(form, out)?;
                    }
                    out.put(b"\n")?;
                }
                if form.is_none() {
                    members(variant, depth, forms, out)?;
                }
            }
        }
    }
    Ok(())
}

fn field<'a>(value: &'a Value, name: &'static [u8]) -> Option<&'a Value> {
    value.field(name).expect("declared schema member is an object")
}

fn member_type(member: &Value, shared: bool, out: &mut Writer) -> Result<(), Overflow> {
    if let Some(Value::Array(variants)) = field(member, b"oneOf").or(field(member, b"anyOf")) {
        for (index, variant) in variants.iter().enumerate() {
            if index != 0 {
                out.put(b" or ")?;
            }
            match field(variant, b"type") {
                Some(Value::String(kind)) => out.put(kind)?,
                None => out.put(b"any value")?,
                Some(Value::Object(_) | Value::Array(_) | Value::Number(_) | Value::Boolean(_) | Value::Null) => {
                    unreachable!("whole variant type is text")
                }
            }
        }
    } else if let Some(values) = field(member, b"enum")
        && !shared
    {
        let values = values.array(b"enum").expect("declared enum is an array");
        out.put(b"one of `")?;
        for (index, value) in values.iter().enumerate() {
            if index != 0 {
                out.put(b"|")?;
            }
            match value {
                Value::String(text) => out.put(text)?,
                Value::Object(_) | Value::Array(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
                    unreachable!("declared enum values are strings");
                }
            }
        }
        out.put(b"`")?;
    } else {
        let kind = match field(member, b"type") {
            Some(Value::String(kind)) => kind.as_ref(),
            None if field(member, b"enum").is_some() => b"string",
            None | Some(Value::Object(_) | Value::Array(_) | Value::Number(_) | Value::Boolean(_) | Value::Null) => b"",
        };
        out.put(match kind {
            b"string" => b"text",
            b"integer" => b"integer",
            b"number" => b"number",
            b"boolean" => b"boolean",
            b"object" => b"object",
            b"array" => b"array",
            _ => b"any value",
        })?;
        if kind == b"integer" || kind == b"number" {
            match (field(member, b"minimum"), field(member, b"maximum")) {
                (Some(minimum), Some(maximum)) => {
                    out.put(b" [")?;
                    out.put(number_text(minimum))?;
                    out.put(b", ")?;
                    out.put(number_text(maximum))?;
                    out.put(b"]")?;
                }
                (Some(minimum), None) => {
                    out.put(b" >=")?;
                    out.put(number_text(minimum))?;
                }
                (None, Some(maximum)) => {
                    out.put(b" <=")?;
                    out.put(number_text(maximum))?;
                }
                (None, None) => {}
            }
        }
    }
    if let Some(default) = field(member, b"default") {
        out.put(b", ")?;
        match default {
            Value::Number(text) => out.put(text)?,
            Value::Object(_) | Value::Array(_) | Value::String(_) | Value::Boolean(_) | Value::Null => {
                unreachable!("declared default is a number")
            }
        }
        out.put(b" when absent")?;
    }
    Ok(())
}

fn number_text(value: &Value) -> &[u8] {
    match value {
        Value::Number(text) => text,
        Value::Object(_) | Value::Array(_) | Value::String(_) | Value::Boolean(_) | Value::Null => {
            unreachable!("declared numeric bound is a number")
        }
    }
}
