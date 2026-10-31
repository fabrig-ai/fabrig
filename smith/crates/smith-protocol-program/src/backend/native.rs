//! Native whole-file observations (domain/tools.md, 10; protocol/programs.md, 5.4).
//! Keeps one bounded load, separately owned from private startup files. Whole
//! reads hold Allow; allowed writes and edits hold their result. Failed loads
//! teach nothing. Closing waits for the load's terminal before private cleanup.

use super::Owner;
use super::{Backend, Below, Phase, policy, stop};
use crate::mcp::{self, json};
use alloc::boxed::Box;
use skein_io::{digest, file};
use skein_lib::{Queue, Time, Token};
use smith_domain::tools::{Place, Version};
use smith_protocol_llm::{FromDomain, ToDomain};

pub(super) enum After {
    Allow { call: mcp::CallId, input: Box<[u8]> },
    Result(ToDomain),
}

pub(super) struct Snapshot {
    owner: Token,
    place: Place,
    after: After,
    submitted: bool,
    cancelled: bool,
    deadline: Time,
}

impl Backend {
    pub(super) fn native_permission(
        &mut self,
        index: u32,
        name: &[u8],
        input: &[u8],
        call: mcp::CallId,
        updated_input: Box<[u8]>,
    ) {
        let place = policy::place(input, &self.mounts, self.settings.path_bytes);
        if let Some(place) = place {
            match policy::Builtin::named(name) {
                Some(policy::Builtin::Read) if whole(input) => {
                    let owner = self.ledger.get(index).expect("permitted call").owner.expect("completed call");
                    self.begin_snapshot(owner, place, After::Allow { call, input: updated_input });
                    return;
                }
                Some(policy::Builtin::Write | policy::Builtin::Edit) => {
                    let id = self.ledger.get(index).expect("permitted call").id.clone();
                    self.native_writes.insert(id, place).expect("allowed calls fit the ledger");
                }
                Some(policy::Builtin::Read | policy::Builtin::Glob | policy::Builtin::Grep | policy::Builtin::Bash)
                | None => {}
            }
        }
        self.server.as_mut().expect("permission server").verdict(
            call,
            mcp::Verdict::Allow { updated_input },
            &mut self.io,
        );
    }

    pub(super) fn begin_snapshot(&mut self, owner: Token, place: Place, after: After) {
        assert!(self.native.is_none(), "one native snapshot at a time");
        let deadline = match &self.request {
            Some(FromDomain::Complete { deadlines, .. }) => deadlines.whole,
            Some(FromDomain::Cancel { .. } | FromDomain::Yield { .. }) | None => {
                unreachable!("snapshot belongs to an allowed completion")
            }
        };
        self.native = Some(Snapshot { owner, place, after, submitted: false, cancelled: false, deadline });
    }

    pub(super) fn cancel_native(&mut self) {
        if let Some(snapshot) = &mut self.native {
            snapshot.cancelled = true;
        }
    }

    pub(super) fn load_native(&mut self, below: &mut Queue<Below>) {
        let owner = self.token(Owner::Native);
        if let Some(snapshot) = &mut self.native {
            if snapshot.submitted {
                return;
            }
            snapshot.submitted = true;
            let path = if snapshot.place.path.is_empty() { b".".as_slice() } else { &snapshot.place.path };
            below.push(Below::File {
                request: file::Request::Load {
                    owner,
                    root: snapshot.place.root,
                    path: path.into(),
                    max: self.settings.adapter.stated.file_bytes,
                    no_follow: false,
                },
                deadline: snapshot.deadline,
            });
        }
    }

    pub(super) fn native_event(&mut self, event: file::Event, up: &mut Queue<ToDomain>) {
        let snapshot = self.native.take().expect("one native file terminal");
        if snapshot.cancelled || self.phase == Phase::Ending || self.stop != stop::Stop::None {
            return;
        }
        match event {
            file::Event::Loaded { bytes, .. } => up.push(ToDomain::ReadVersion {
                owner: snapshot.owner,
                place: snapshot.place,
                version: Version::new(digest::digest(&bytes).0),
            }),
            file::Event::Locked { .. }
            | file::Event::Busy { .. }
            | file::Event::Made { .. }
            | file::Event::Opened { .. }
            | file::Event::Refused { .. }
            | file::Event::Scanned { .. }
            | file::Event::Stored { .. }
            | file::Event::Conflict { .. }
            | file::Event::TooLarge { .. }
            | file::Event::Stated { .. }
            | file::Event::Written { .. }
            | file::Event::Read { .. }
            | file::Event::Cut { .. }
            | file::Event::Synced { .. }
            | file::Event::Closed { .. }
            | file::Event::Renamed { .. }
            | file::Event::Removed { .. }
            | file::Event::Listed { .. }
            | file::Event::Failed { .. }
            | file::Event::Cancelled { .. } => {}
        }
        match snapshot.after {
            After::Allow { call, input } => self.server.as_mut().expect("snapshot permission server").verdict(
                call,
                mcp::Verdict::Allow { updated_input: input },
                &mut self.io,
            ),
            After::Result(result) => up.push(result),
        }
        self.flush_builtins(up);
        self.match_calls();
    }
}

fn whole(input: &[u8]) -> bool {
    match json::collect(input) {
        Some(document) => {
            json::field(&document, 0, b"offset").is_none() && json::field(&document, 0, b"limit").is_none()
        }
        None => false,
    }
}
