use super::walk::Walk;
use super::{Halt, codec_error};
use crate::layouts::{Nested, Peer, Unconverted};
use crate::{Carried, Kind, v22 as wire};
use alloc::boxed::Box;
use skein_codec::layout::Mode;
use skein_lib::Reader;
use wire::Record;

// Only these engine kinds carry parts of another layout. The others go
// directly to the frame pass, without decoding their parts.
#[derive(Debug)]
pub(super) enum Body {
    Assignment(wire::Assignment),
    Job(wire::Job),
    Grant(wire::Granted),
    ReadList(wire::ReadList),
    Asked(wire::Asked),
}

impl Body {
    pub(super) fn decode(
        peer: &Peer,
        kind: Kind,
        bytes: &[u8],
        limits: &wire::Limits,
    ) -> Result<Option<Self>, Unconverted> {
        let mut reader = Reader::new(bytes);
        let body = if kind == Kind::Assignment {
            match wire::Assignment::decode(limits, &mut reader) {
                Ok(record) => Ok(Self::Assignment(record)),
                Err(problem) => Err(problem),
            }
        } else if kind == Kind::Job {
            match wire::Job::decode(limits, &mut reader) {
                Ok(record) => Ok(Self::Job(record)),
                Err(problem) => Err(problem),
            }
        } else if kind == Kind::Grant {
            match wire::Granted::decode(limits, &mut reader) {
                Ok(record) => Ok(Self::Grant(record)),
                Err(problem) => Err(problem),
            }
        } else if kind == Kind::ReadList {
            match wire::ReadList::decode(limits, &mut reader) {
                Ok(record) => Ok(Self::ReadList(record)),
                Err(problem) => Err(problem),
            }
        } else if kind == Kind::Asked {
            match wire::Asked::decode(limits, &mut reader) {
                Ok(record) => Ok(Self::Asked(record)),
                Err(problem) => Err(problem),
            }
        } else {
            return Ok(None);
        };
        match body {
            Ok(body) => Ok(Some(body)),
            Err(error) => Err(codec_error(peer, Mode::Keep, kind.number(), error)),
        }
    }

    pub(super) fn encode(&self, limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
        match self {
            Self::Assignment(record) => record.encode(limits),
            Self::Job(record) => record.encode(limits),
            Self::Grant(record) => record.encode(limits),
            Self::ReadList(record) => record.encode(limits),
            Self::Asked(record) => record.encode(limits),
        }
    }

    pub(super) fn parts(&mut self, walk: &mut Walk<'_>, limits: &wire::Limits) -> Result<(), Halt> {
        let grant_bound = limits.application_value_secret.min(wire::CEILINGS.application_value_secret);
        match self {
            Self::Assignment(record) => {
                workspace(walk, &mut record.workspace, limits)?;
                walk.part(
                    Nested::Charter,
                    &mut record.charter,
                    limits.assignment_charter.min(wire::CEILINGS.assignment_charter),
                )?;
                walk.window(
                    &mut record.transcript,
                    limits.assignment_transcript.min(wire::CEILINGS.assignment_transcript),
                )?;
                walk.kept(&mut record.parts, limits)?;
                walk.grants(&mut record.grants, grant_bound)?;
            }
            Self::Job(record) => {
                workspace(walk, &mut record.workspace, limits)?;
                walk.part(
                    Nested::Carried(Carried::JobSpec),
                    &mut record.spec,
                    limits.job_spec.min(wire::CEILINGS.job_spec),
                )?;
                walk.grants(&mut record.grants, grant_bound)?;
            }
            Self::Grant(record) => walk.grant(&mut record.grant, grant_bound)?,
            Self::ReadList(record) => walk.grant(&mut record.grant, grant_bound)?,
            Self::Asked(record) => match &mut record.answer {
                wire::AskAnswer::Bytes(value) => walk.part(
                    Nested::Carried(Carried::Answer),
                    &mut value.bytes,
                    limits.ask_bytes_bytes.min(wire::CEILINGS.ask_bytes_bytes),
                )?,
                wire::AskAnswer::Refused => {}
            },
        }
        Ok(())
    }
}

fn workspace(walk: &mut Walk<'_>, value: &mut Option<wire::Workspace>, limits: &wire::Limits) -> Result<(), Halt> {
    if let Some(value) = value {
        walk.part(
            Nested::Carried(Carried::Items),
            &mut value.items,
            limits.workspace_items.min(wire::CEILINGS.workspace_items),
        )?;
    }
    Ok(())
}
