//! Decode an address read (temper/docs/design/domain/agent.md, 8.4.1).

use crate::{Problem, json::Value};
use smith_domain_run::HostAnswer;
use temper_engine_domain::Tool;

pub(crate) fn decode(value: &Value) -> Result<Result<Tool, HostAnswer>, Problem> {
    let Value::Object(fields) = value else { return Err(Problem::Type(b"see")) };
    for (name, _) in fields {
        if name.as_ref() != b"address" {
            return Err(Problem::Range(b"see"));
        }
    }
    let address = value.required(b"address")?.text(b"address")?;
    let line = address.trim_ascii();
    if line.is_empty() || line.len() > 512 {
        return Ok(Err(HostAnswer::new(b"see: address takes 1 to 512 bytes, trimmed.".as_slice().into(), true)
            .ok_or(Problem::TooLarge)?));
    }
    Ok(Ok(Tool::See { line: line.into(), from: 0 }))
}
