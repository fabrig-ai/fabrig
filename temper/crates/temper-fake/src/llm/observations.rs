//! Lossless query records for the reference case (domain/testing.md, 5.3).
use crate::{Refusal, limits, read};
use alloc::boxed::Box;
use jig_protocol_config::Path;
use skein_fake_llm_domain::api::{self, Message, Part, Role};
use skein_json::{
    Document,
    writer::{Encoder, Limits},
};
use skein_lib::{List, Overflow, Writer, base64};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub credential: Option<u32>,
    pub model: Box<[u8]>,
    pub system: Box<[u8]>,
    pub messages: Box<[Message]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation {
    Query(Query),
    Checkpoint { lost: u64 },
    Full { capacity: u32 },
}
/// Appends one complete record, or none when the output has insufficient room.
pub fn encode(query: &api::Query, out: &mut List<u8>) -> Result<(), Overflow> {
    let limits = Limits { depth: 8, length: limits::OBSERVATION_BYTES };
    let mut measure = Encoder::measure(&limits);
    render(query, &mut measure);
    let len = measure.measured().or(Err(Overflow))?;
    if out.room() < len.checked_add(1).ok_or(Overflow)? {
        return Err(Overflow);
    }
    let mut write = Encoder::write(len, &limits);
    render(query, &mut write);
    for byte in write.finish() {
        out.push(byte).expect("measured query record");
    }
    out.push(b'\n').expect("measured newline");
    Ok(())
}
fn binary(json: &mut Encoder, name: &[u8], bytes: &[u8]) {
    json.key(name);
    let len = base64::encoded_len(base64::Alphabet::Standard, u32::try_from(bytes.len()).expect("bounded query"))
        .expect("base64 size");
    let mut out = Writer::new(usize::try_from(len).expect("bounded encoding"));
    base64::encode(base64::Alphabet::Standard, bytes, &mut out).expect("measured base64");
    json.string(&out.finish());
}
fn render(query: &api::Query, json: &mut Encoder) {
    json.object_start();
    json.key(b"observation");
    json.string(b"query");
    json.key(b"credential");
    match query.credential {
        Some(n) => json.unsigned(u64::from(n)),
        None => json.null(),
    }
    binary(json, b"model", &query.model);
    binary(json, b"system", &query.system);
    json.key(b"messages");
    json.array_start();
    for message in &query.messages {
        json.object_start();
        json.key(b"role");
        json.string(match message.role {
            Role::User => b"user",
            Role::Assistant => b"assistant",
        });
        json.key(b"parts");
        json.array_start();
        for part in &message.parts {
            json.object_start();
            match part {
                Part::Text { text } => binary(json, b"text", text),
                Part::Opaque { bytes } => binary(json, b"opaque", bytes),
                Part::ToolCall { id, name, arguments } => {
                    json.key(b"tool_call");
                    json.object_start();
                    binary(json, b"id", id);
                    binary(json, b"name", name);
                    binary(json, b"arguments", arguments);
                    json.object_end();
                }
                Part::ToolOutput { id, output, is_error } => {
                    json.key(b"tool_output");
                    json.object_start();
                    binary(json, b"id", id);
                    binary(json, b"output", output);
                    json.key(b"is_error");
                    json.boolean(*is_error);
                    json.object_end();
                }
            }
            json.object_end();
        }
        json.array_end();
        json.object_end();
    }
    json.array_end();
    json.object_end();
}
fn items(d: &Document, at: u32, p: &Path) -> Result<List<u32>, Refusal> {
    match jig_protocol_config::read::list(d, at, limits::LLM_HISTORY) {
        Ok(items) => Ok(items),
        Err(why) => Err(read::kind(p, why)),
    }
}
fn binary_field(
    o: &mut jig_protocol_config::read::Object,
    d: &Document,
    name: &[u8],
    p: &Path,
) -> Result<Box<[u8]>, Refusal> {
    let text = read::field(o, d, name, limits::OBSERVATION_BYTES, p)?;
    match base64::decode(base64::Alphabet::Standard, &text, limits::LLM_REQUEST_BYTES) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(read::why(&p.member(name), b"expected standard base64")),
    }
}
/// Decodes the peer's public records into the fake domain's original messages.
pub fn decode(bytes: &[u8]) -> Result<Observation, Refusal> {
    let p = Path::root();
    let d = match jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits {
            bytes: limits::OBSERVATION_BYTES,
            tokens: limits::LLM_TOKENS,
            depth: limits::JSON_DEPTH,
        },
    ) {
        Ok(d) => d,
        Err(why) => return Err(read::kind(&why.path, why.what)),
    };
    let mut o = read::object(&d, 0, &p)?;
    if let Some(at) = o.member(&d, b"observations_lost") {
        let lost = read::integer(&d, at, u64::MAX, &p.member(b"observations_lost"))?;
        read::finish(o, &d, &p)?;
        return Ok(Observation::Checkpoint { lost });
    }
    let kind = read::field(&mut o, &d, b"observation", 64, &p)?;
    if kind.as_ref() == b"records_full" {
        let at = read::required(&mut o, &d, b"capacity", &p)?;
        let capacity = read::integer(&d, at, u64::from(u32::MAX), &p.member(b"capacity"))?;
        read::finish(o, &d, &p)?;
        return Ok(Observation::Full { capacity: u32::try_from(capacity).expect("bounded capacity") });
    }
    if kind.as_ref() != b"query" {
        return Err(read::why(&p.member(b"observation"), b"expected query"));
    }
    let at = read::required(&mut o, &d, b"credential", &p)?;
    let credential = if read::is_null(&d, at) {
        None
    } else {
        Some(
            u32::try_from(read::integer(&d, at, u64::from(u32::MAX), &p.member(b"credential"))?)
                .expect("bounded credential"),
        )
    };
    let model = binary_field(&mut o, &d, b"model", &p)?;
    let system = binary_field(&mut o, &d, b"system", &p)?;
    let at = read::required(&mut o, &d, b"messages", &p)?;
    read::finish(o, &d, &p)?;
    let p = p.member(b"messages");
    let mut messages = List::with_capacity(limits::LLM_HISTORY);
    for (i, at) in items(&d, at, &p)?.iter().enumerate() {
        let p = read::index(&p, i);
        let mut o = read::object(&d, *at, &p)?;
        let role = read::field(&mut o, &d, b"role", 16, &p)?;
        let role = match role.as_ref() {
            b"user" => Role::User,
            b"assistant" => Role::Assistant,
            _ => return Err(read::why(&p.member(b"role"), b"expected user or assistant")),
        };
        let at = read::required(&mut o, &d, b"parts", &p)?;
        read::finish(o, &d, &p)?;
        let p = p.member(b"parts");
        let mut parts = List::with_capacity(limits::LLM_HISTORY);
        for (i, at) in items(&d, at, &p)?.iter().enumerate() {
            parts.push(part(&d, *at, &read::index(&p, i))?).expect("bounded parts");
        }
        messages.push(Message { role, parts: parts.into_boxed() }).expect("bounded messages");
    }
    Ok(Observation::Query(Query { credential, model, system, messages: messages.into_boxed() }))
}
fn part(d: &Document, at: u32, p: &Path) -> Result<Part, Refusal> {
    let mut o = read::object(d, at, p)?;
    let part = if o.member(d, b"text").is_some() {
        // Use a fresh object so binary_field also consumes the member.
        let mut o = read::object(d, at, p)?;
        let text = binary_field(&mut o, d, b"text", p)?;
        read::finish(o, d, p)?;
        return Ok(Part::Text { text });
    } else if o.member(d, b"opaque").is_some() {
        let mut o = read::object(d, at, p)?;
        let bytes = binary_field(&mut o, d, b"opaque", p)?;
        read::finish(o, d, p)?;
        return Ok(Part::Opaque { bytes });
    } else if let Some(at) = o.member(d, b"tool_call") {
        let p = p.member(b"tool_call");
        let mut inner = read::object(d, at, &p)?;
        let id = binary_field(&mut inner, d, b"id", &p)?;
        let name = binary_field(&mut inner, d, b"name", &p)?;
        let arguments = binary_field(&mut inner, d, b"arguments", &p)?;
        read::finish(inner, d, &p)?;
        Part::ToolCall { id, name, arguments }
    } else {
        let at = read::required(&mut o, d, b"tool_output", p)?;
        let p = p.member(b"tool_output");
        let mut inner = read::object(d, at, &p)?;
        let id = binary_field(&mut inner, d, b"id", &p)?;
        let output = binary_field(&mut inner, d, b"output", &p)?;
        let at = read::required(&mut inner, d, b"is_error", &p)?;
        let is_error = match d.token(at) {
            Some(token) if token.kind == skein_json::document::Kind::True => true,
            Some(token) if token.kind == skein_json::document::Kind::False => false,
            Some(_) | None => return Err(read::why(&p.member(b"is_error"), b"expected boolean")),
        };
        read::finish(inner, d, &p)?;
        Part::ToolOutput { id, output, is_error }
    };
    read::finish(o, d, p)?;
    Ok(part)
}
