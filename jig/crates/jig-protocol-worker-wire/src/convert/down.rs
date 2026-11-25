use super::{codec_error, convert_part, envelope, part};
use crate::layouts::{Cause, Nested, Peer, Unconverted};
use crate::{Carried, Kind, v22 as wire};
use alloc::boxed::Box;
use skein_codec::{Reason, layout::Mode};
use skein_lib::{Buffer, List, Reader, Secret};
use wire::Record;

// Only these engine kinds carry parts of another layout. The others go
// directly to the frame plan. The current kind comes from KINDS.
pub(super) fn parts(
    peer: &mut Peer,
    kind: Kind,
    body: &[u8],
    limits: &wire::Limits,
) -> Result<Option<Box<[u8]>>, Unconverted> {
    let mut reader = Reader::new(body);
    let number = kind.number();
    let encoded = if kind == Kind::Assignment {
        let mut record = match wire::Assignment::decode(limits, &mut reader) {
            Ok(record) => record,
            Err(error) => return Err(codec_error(peer, Mode::Keep, number, error)),
        };
        assignment(peer, &mut record, limits)?;
        record.encode(limits)
    } else if kind == Kind::Job {
        let mut record = match wire::Job::decode(limits, &mut reader) {
            Ok(record) => record,
            Err(error) => return Err(codec_error(peer, Mode::Keep, number, error)),
        };
        workspace(peer, &mut record.workspace, limits)?;
        part(
            peer,
            Mode::Keep,
            Nested::Carried(Carried::JobSpec),
            &mut record.spec,
            limits.job_spec.min(wire::CEILINGS.job_spec),
        )?;
        grants(peer, &mut record.grants, limits)?;
        record.encode(limits)
    } else if kind == Kind::Grant {
        let mut record = match wire::Granted::decode(limits, &mut reader) {
            Ok(record) => record,
            Err(error) => return Err(codec_error(peer, Mode::Keep, number, error)),
        };
        grant(peer, &mut record.grant, limits)?;
        record.encode(limits)
    } else if kind == Kind::ReadList {
        let mut record = match wire::ReadList::decode(limits, &mut reader) {
            Ok(record) => record,
            Err(error) => return Err(codec_error(peer, Mode::Keep, number, error)),
        };
        grant(peer, &mut record.grant, limits)?;
        record.encode(limits)
    } else if kind == Kind::Asked {
        let mut record = match wire::Asked::decode(limits, &mut reader) {
            Ok(record) => record,
            Err(error) => return Err(codec_error(peer, Mode::Keep, number, error)),
        };
        match &mut record.answer {
            wire::AskAnswer::Bytes(value) => {
                part(
                    peer,
                    Mode::Keep,
                    Nested::Carried(Carried::Answer),
                    &mut value.bytes,
                    limits.ask_bytes_bytes.min(wire::CEILINGS.ask_bytes_bytes),
                )?;
            }
            wire::AskAnswer::Refused => {}
        }
        record.encode(limits)
    } else {
        return Ok(None);
    };
    match encoded {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) => Err(codec_error(peer, Mode::Keep, number, error)),
    }
}

fn assignment(peer: &mut Peer, record: &mut wire::Assignment, limits: &wire::Limits) -> Result<(), Unconverted> {
    workspace(peer, &mut record.workspace, limits)?;
    part(
        peer,
        Mode::Keep,
        Nested::Charter,
        &mut record.charter,
        limits.assignment_charter.min(wire::CEILINGS.assignment_charter),
    )?;
    if peer.converts_part(Mode::Keep, Nested::Turn) {
        record.transcript =
            window(peer, &record.transcript, limits.assignment_transcript.min(wire::CEILINGS.assignment_transcript))?;
        record.parts = kept_parts(peer, &record.parts, limits)?;
    }
    grants(peer, &mut record.grants, limits)
}

fn workspace(
    peer: &mut Peer,
    workspace: &mut Option<wire::Workspace>,
    limits: &wire::Limits,
) -> Result<(), Unconverted> {
    if let Some(workspace) = workspace {
        part(
            peer,
            Mode::Keep,
            Nested::Carried(Carried::Items),
            &mut workspace.items,
            limits.workspace_items.min(wire::CEILINGS.workspace_items),
        )?;
    }
    Ok(())
}

fn grants(peer: &mut Peer, grants: &mut List<wire::Grant>, limits: &wire::Limits) -> Result<(), Unconverted> {
    for value in grants.iter_mut() {
        grant(peer, value, limits)?;
    }
    Ok(())
}

fn grant(peer: &mut Peer, grant: &mut wire::Grant, limits: &wire::Limits) -> Result<(), Unconverted> {
    match &mut grant.value {
        wire::GrantValue::Application(value) => {
            let nested = Nested::Carried(Carried::Grant);
            if let Some(bytes) = convert_part(
                peer,
                Mode::Keep,
                nested,
                value.secret.expose(),
                limits.application_value_secret.min(wire::CEILINGS.application_value_secret),
            )? {
                value.secret = Secret::new(bytes);
            }
        }
        wire::GrantValue::Credential(_) | wire::GrantValue::Program(_) => {}
    }
    Ok(())
}

fn window(peer: &mut Peer, bytes: &[u8], bound: u32) -> Result<Box<[u8]>, Unconverted> {
    if bytes.is_empty() {
        return Ok(Box::new([]));
    }
    let mut reader = Reader::new(bytes);
    let mut window = Buffer::with_capacity(bound);
    let mut turn = Buffer::with_capacity(bound.saturating_sub(4));
    for _ in 0..=reader.remaining().saturating_div(4) {
        if reader.is_empty() {
            break;
        }
        let Some(length) = reader.u32() else {
            return Err(envelope("Assignment", "transcript", Cause::Malformed));
        };
        let Some(body) = reader.bytes(length) else {
            return Err(envelope("Assignment", "transcript", Cause::Malformed));
        };
        peer.down(Nested::Turn, body, &mut turn)?;
        if window.put(&turn.len().to_be_bytes()).is_err() || window.put(turn.as_slice()).is_err() {
            return Err(envelope("Assignment", "transcript", Cause::Large));
        }
    }
    Ok(window.to_boxed())
}

fn kept_parts(peer: &mut Peer, bytes: &[u8], limits: &wire::Limits) -> Result<Box<[u8]>, Unconverted> {
    if bytes.is_empty() {
        return Ok(Box::new([]));
    }
    let Ok(parts) = crate::parts::decode(bytes, limits) else {
        return Err(envelope("Assignment", "parts", Cause::Malformed));
    };
    let mut kept = List::with_capacity(parts.len());
    let mut out = Buffer::with_capacity(limits.kept_part_body.min(wire::CEILINGS.kept_part_body));
    for mut item in parts.into_boxed() {
        if peer.down(Nested::Turn, &item.body, &mut out).is_ok() {
            item.body = out.to_boxed();
            kept.push(item).expect("kept parts can only shrink in number");
        }
    }
    match crate::parts::encode(kept.as_slice(), limits) {
        Ok(bytes) => Ok(bytes),
        Err(problem) => {
            let cause = if problem.reason == Reason::Bound { Cause::Large } else { Cause::Malformed };
            Err(envelope("Assignment", "parts", cause))
        }
    }
}
