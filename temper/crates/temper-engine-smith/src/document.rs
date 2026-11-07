//! Decode the chat's document write (temper/docs/design/domain/agent.md, 8.4.3).

use crate::{Problem, json::Value};
use temper_engine_domain::{DocumentAction, Tool};

pub(crate) fn decode(value: &Value) -> Result<Tool, Problem> {
    let Value::Object(fields) = value else { return Err(Problem::Type(b"document")) };
    for (name, _) in fields {
        if name.as_ref() != b"action" && name.as_ref() != b"body" {
            return Err(Problem::Range(b"document"));
        }
    }
    let action = match value.required(b"action")?.text(b"action")?.as_ref() {
        b"write" => DocumentAction::Write,
        b"append" => DocumentAction::Append,
        b"replace" => DocumentAction::Replace,
        _ => return Err(Problem::Range(b"action")),
    };
    Ok(Tool::Document { action, body: value.required(b"body")?.text(b"body")? })
}
