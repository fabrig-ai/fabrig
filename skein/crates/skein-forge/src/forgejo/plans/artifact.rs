//! A named artifact, its run commit, and one bounded ZIP (forge/forgejo.md, 1.2).
use super::{
    Body, Heard,
    plan::{Bounds, Step},
    tables::{Repository, Tables},
};
use crate::api::Excess;
use crate::{
    api,
    forgejo::{document as d, request::Request},
};
use alloc::boxed::Box;

#[derive(Debug)]
enum State {
    Listing { id: Option<u64> },
    Run { id: u64 },
    Zip { head: api::Commit },
    Ended,
}
#[derive(Debug)]
pub(super) struct Plan {
    row: Repository,
    run: u64,
    name: Box<[u8]>,
    max: u32,
    state: State,
}
impl Plan {
    pub(super) fn new(row: Repository, run: u64, name: Box<[u8]>, max: u32) -> Self {
        Self { row, run, name, max, state: State::Listing { id: None } }
    }
    pub(super) fn begin(&self) -> Step {
        Step::send(Request::Artifacts { at: self.row.at(), run: self.run, name: Some(self.name.clone()) })
    }
    pub(super) fn heard(
        &mut self,
        heard: Heard,
        bounds: Bounds,
        repository: api::Repository,
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        match heard {
            Heard::Head { status, fields } => {
                if status != 200 {
                    return Err(super::failure::refused(
                        super::failure::Asked::Leaf,
                        status,
                        None,
                        fields.retry_after,
                        repository,
                        tables,
                    ));
                }
                Ok(None)
            }
            Heard::Failed(fault) => Err(super::failure::transport(fault)),
            Heard::Row(row) => {
                self.received(row, bounds)?;
                Ok(None)
            }
            Heard::Done(body) => self.done(body, bounds),
        }
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        clippy::manual_let_else,
        reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
    )]
    fn received(&mut self, row: d::Row, bounds: Bounds) -> Result<(), api::Error> {
        let value = match row {
            d::Row::Artifact(value) => value,
            _ => return Err(api::Error::InvalidAnswer),
        };
        match &mut self.state {
            State::Listing { id } => {
                if id.is_some() || value.id == 0 || value.run_id != self.run || value.name != self.name {
                    return Err(api::Error::InvalidAnswer);
                }
                if self.max > bounds.answer_bytes || value.size_in_bytes > u64::from(bounds.answer_bytes) {
                    return Err(api::Error::TooLarge(Excess::unnamed()));
                }
                *id = Some(value.id);
                Ok(())
            }
            State::Run { .. } | State::Zip { .. } => Err(api::Error::InvalidAnswer),
            State::Ended => unreachable!("an artifact plan hears nothing after its end"),
        }
    }
    fn done(&mut self, body: Body, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        let step;
        self.state = match core::mem::replace(&mut self.state, State::Ended) {
            State::Listing { id } => match body {
                Body::Document(d::Document::Listed { skipped: 0 }) => {
                    let id = id.ok_or(api::Error::Missing)?;
                    step = Step::send(Request::Run { at: self.row.at(), run: self.run });
                    State::Run { id }
                }
                Body::Empty | Body::Raw { .. } | Body::Document(_) | Body::Malformed(_) => return Err(invalid(body)),
            },
            State::Run { id } => match body {
                Body::Document(d::Document::Run(run)) => {
                    if run.id != self.run {
                        return Err(api::Error::InvalidAnswer);
                    }
                    let head = super::reading::commit(run.commit_sha, self.row.object_format)
                        .ok_or(api::Error::InvalidAnswer)?;
                    step = Step::send(Request::ArtifactZip { at: self.row.at(), id });
                    State::Zip { head }
                }
                Body::Empty | Body::Raw { .. } | Body::Document(_) | Body::Malformed(_) => return Err(invalid(body)),
            },
            State::Zip { head } => match body {
                Body::Raw { bytes, more } => {
                    if more {
                        return Err(api::Error::TooLarge(Excess::unnamed()));
                    }
                    let file = crate::forgejo::zip::read(&bytes, self.max, bounds.answer_bytes)?;
                    step = Step::Done(Ok(api::Answer::File {
                        head,
                        path: file.path,
                        bytes: file.bytes,
                        truncated: file.truncated,
                    }));
                    State::Ended
                }
                Body::Empty | Body::Document(_) | Body::Malformed(_) => return Err(invalid(body)),
            },
            State::Ended => unreachable!("an artifact plan hears nothing after its end"),
        };
        Ok(Some(step))
    }
}
fn invalid(body: Body) -> api::Error {
    match body {
        Body::Malformed(error) => super::failure::malformed(error),
        Body::Empty | Body::Raw { .. } | Body::Document(_) => api::Error::InvalidAnswer,
    }
}
