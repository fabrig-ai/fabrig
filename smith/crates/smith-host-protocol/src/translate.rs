//! Host-owned Start data becomes one checked channel frame (protocol/channel.md, section 3).
//! The host service supplies opening paths and credential values; the host domain
//! retains alias metadata and never credential values.
use crate::generated::channel as mapped;
use alloc::boxed::Box;
use skein_channel::Frame;
use skein_lib::List;
use smith_channel as wire;
use smith_channel::Record;
use smith_host_domain::{self as host, channel};

use crate::Error;

/// Values held by the host service beside its domain's names, in Start order.
#[derive(Debug)]
pub struct Values {
    /// One filesystem path for each directory, in domain order.
    pub paths: Box<[Box<[u8]>]>,
    /// One credential value for each grant, in domain order.
    pub credentials: Box<[smith_protocol_translate::GrantValue]>,
}

/// Encode a domain Start and its service-owned values as the first downlink frame.
pub fn encode_start(
    start: channel::Start,
    window: channel::Window,
    values: Values,
    limits: &wire::Limits,
    charter_limits: &smith_charter::v5::Limits,
    transcript_limits: &smith_transcript::v8::Limits,
    endpoints: &smith_protocol_translate::Endpoints,
) -> Result<Frame, Error> {
    if start.grants.len() != values.credentials.len() {
        return Err(Error::MissingValue);
    }
    for (grant, credential) in start.grants.iter().zip(values.credentials.iter()) {
        if grant_form(credential)? != grant.form {
            return Err(Error::Grants);
        }
    }
    let mut messages = List::with_capacity(limits.start_messages);
    for message in start.messages {
        let item = encode_message(message, limits)?;
        if messages.push(item).is_err() {
            return Err(Error::MissingValue);
        }
    }
    let workspace = encode_workspace(&start.directories, &values.paths, limits)?;
    let transcript = match start.transcript {
        Some(history) => {
            let (history, saved) = history.into_parts();
            match saved {
                Some(piece) => piece,
                None => {
                    let mut bodies = List::with_capacity(u32::try_from(history.turns.len()).or(Err(Error::Value))?);
                    for turn in &history.turns {
                        let Ok(body) = smith_protocol_translate::encode_turn(turn, transcript_limits, endpoints) else {
                            return Err(Error::Value);
                        };
                        if bodies.push(body).is_err() {
                            return Err(Error::MissingValue);
                        }
                    }
                    smith_protocol_translate::transcript_piece(bodies.as_slice()).ok_or(Error::Value)?
                }
            }
        }
        None => Box::default(),
    };
    let Ok(charter) = smith_protocol_translate::encode_charter(start.charter.value(), charter_limits, endpoints) else {
        return Err(Error::Value);
    };
    let mut answered = List::with_capacity(limits.start_answered);
    for call in start.answered {
        let item = mapped::answered_call_to_record(&call, limits).or(Err(Error::MissingValue))?;
        if answered.push(item).is_err() {
            return Err(Error::MissingValue);
        }
    }
    let mut grants = List::with_capacity(limits.start_grants);
    for (grant, credential) in start.grants.into_iter().zip(values.credentials) {
        let value = grant_value(credential, limits)?;
        let item = wire::Grant { account: grant.account, generation: grant.generation, valid: grant.valid, value };
        if grants.push(item).is_err() {
            return Err(Error::MissingValue);
        }
    }
    let window = mapped::window_to_record(&window);
    let live = match start.live {
        Some(ask) => Some(mapped::live_ask_to_record(&ask)),
        None => None,
    };
    let record = wire::Start {
        live,
        messages,
        activation: start.activation,
        identity: match start.identity {
            Some(identity) => Some(identity.bytes()),
            None => None,
        },
        charter,
        workspace,
        transcript,
        tell_parts: start.tell_parts,
        parts: encode_parts(start.parts, limits, transcript_limits, endpoints)?,
        answered,
        grants,
        window,
    };
    record.measure(limits)?;
    Ok(wire::frame(wire::Body::Start(record))?)
}

fn encode_parts(
    parts: Box<[channel::KeptPart]>,
    limits: &wire::Limits,
    transcript: &smith_transcript::v8::Limits,
    endpoints: &smith_protocol_translate::Endpoints,
) -> Result<Box<[u8]>, Error> {
    let mut bodies = List::with_capacity(u32::try_from(parts.len()).or(Err(Error::Value))?);
    for part in parts {
        let body =
            smith_protocol_translate::encode_part(part.body.value(), transcript, endpoints).or(Err(Error::Value))?;
        let kind = mapped::part_kind_to_record(&part.kind);
        let record = wire::KeptPart {
            activation: part.activation,
            turn: part.turn,
            thread: part.thread,
            call: part.call,
            kind,
            body,
        };
        let length = record.measure(limits)?;
        let mut writer = skein_lib::Writer::new(usize::try_from(length).or(Err(Error::Value))?);
        record.write(limits, &mut writer)?;
        bodies.push(writer.finish()).or(Err(Error::Value))?;
    }
    smith_protocol_translate::transcript_piece(bodies.as_slice()).ok_or(Error::Value)
}

fn encode_workspace(
    directories: &[channel::Directory],
    paths: &[Box<[u8]>],
    limits: &wire::Limits,
) -> Result<Option<wire::Workspace>, Error> {
    if directories.is_empty() {
        if paths.is_empty() {
            return Ok(None);
        }
        return Err(Error::MissingValue);
    }
    if directories.len() != paths.len() {
        return Err(Error::MissingValue);
    }
    let mut items = List::with_capacity(limits.workspace_directories);
    for (directory, path) in directories.iter().zip(paths) {
        let mut conflicts = List::with_capacity(limits.directory_conflicts);
        for conflict in &directory.conflicts {
            if conflicts.push(conflict.clone()).is_err() {
                return Err(Error::MissingValue);
            }
        }
        let item = wire::Directory {
            name: directory.name.clone(),
            path: path.clone(),
            writable: directory.writable,
            git: directory.git,
            conflicts,
            results: directory.results.clone(),
            hooks: match &directory.hooks {
                Some(hooks) => Some(mapped::hook_times_to_record(hooks)),
                None => None,
            },
        };
        if items.push(item).is_err() {
            return Err(Error::MissingValue);
        }
    }
    Ok(Some(wire::Workspace { directories: items }))
}

/// Move one bounded channel call into the host domain's typed operation face.
pub(crate) fn decode_ask(call: &wire::Call) -> Result<Option<channel::Ask>, Error> {
    let ask = match &call.ask {
        wire::Ask::Host(host) => {
            let effect = mapped::effect_to_domain(call.effect.clone());
            channel::Ask::Host { tool: Box::from(host.tool.as_ref()), effect, body: Box::from(host.input.as_ref()) }
        }
        wire::Ask::Save => {
            if call.effect != wire::Effect::Write {
                return Ok(None);
            }
            channel::Ask::Save
        }
        wire::Ask::Deliver(deliver) => {
            if call.effect != wire::Effect::Write {
                return Ok(None);
            }
            let value = mapped::deliver_ask_to_domain(deliver.clone());
            channel::Ask::Deliver { fields: host::Fields::new(value, u64::MAX).ok_or(Error::Value)? }
        }
    };
    Ok(Some(ask))
}

/// Encode one host-domain terminal under the durable call name it answers.
pub(crate) fn encode_reply(
    name: channel::CallName,
    reply: channel::Reply,
    limits: &wire::Limits,
) -> Result<Frame, Error> {
    let name = mapped::call_name_to_record(&name);
    let reply = mapped::reply_to_record(&reply, limits).or(Err(Error::MissingValue))?;
    let record = wire::HostAnswer { name, reply };
    record.measure(limits)?;
    Ok(wire::frame(wire::Body::HostAnswer(record))?)
}

/// The form a service-owned value lends, checked against its grant
/// (protocol/host.md, section 2).
pub fn grant_form(value: &smith_protocol_translate::GrantValue) -> Result<host::Form, Error> {
    use smith_protocol_translate::GrantValue;
    match value {
        GrantValue::Bearer(_) => Ok(host::Form::Credential(host::Scheme::Bearer)),
        GrantValue::Key(_) => Ok(host::Form::Credential(host::Scheme::Key)),
        GrantValue::Program(name) => {
            let name = host::SourceName::new(name).ok_or(Error::Grants)?;
            Ok(host::Form::Program(name))
        }
    }
}

pub(crate) fn grant_value(
    value: smith_protocol_translate::GrantValue,
    limits: &wire::Limits,
) -> Result<wire::GrantValue, Error> {
    use smith_protocol_translate::GrantValue;
    match value {
        GrantValue::Bearer(value) => credential_value(value, wire::Scheme::Bearer, limits),
        GrantValue::Key(value) => credential_value(value, wire::Scheme::Key, limits),
        GrantValue::Program(name) => {
            smith_protocol_translate::program_name(&name).or(Err(Error::Grants))?;
            let value = wire::ProgramValue { name };
            value.measure(limits).or(Err(Error::Grants))?;
            Ok(wire::GrantValue::Program(value))
        }
    }
}

fn credential_value(bytes: Box<[u8]>, scheme: wire::Scheme, limits: &wire::Limits) -> Result<wire::GrantValue, Error> {
    let value = wire::CredentialValue { scheme, bytes: skein_lib::Secret::new(bytes) };
    value.measure(limits).or(Err(Error::Grants))?;
    Ok(wire::GrantValue::Credential(value))
}

/// Encode the same named control message in Start and on the live channel.
pub(crate) fn encode_message(message: channel::Message, limits: &wire::Limits) -> Result<wire::Message, Error> {
    let record = mapped::message_to_record(&message);
    record.measure(limits)?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use smith_protocol_translate::GrantValue;

    #[test]
    fn grant_encoding_refusals_keep_the_grant_error() {
        let mut limits = wire::CEILINGS;
        limits.credential_value_bytes = 1;
        limits.program_value_name = 1;
        for value in [GrantValue::Bearer(Box::from(*b"key")), GrantValue::Program(Box::from(*b"source"))] {
            assert_eq!(grant_value(value, &limits).err(), Some(Error::Grants));
        }
    }
}
