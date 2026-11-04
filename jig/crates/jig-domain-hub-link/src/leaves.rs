//! Shared root translations (jig/docs/design/domain/root.md, 5 and 14;
//! jig/docs/design/domain/hosts.md, 6.1 and 6.9).
use crate::{JobStart, Link};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_host as host;
use skein_lib::{List, Time, Token};

/// Translate a claimed job into the host's assignment vocabulary.
#[must_use]
pub fn job_assignment(job: JobStart) -> host::JobAssignment {
    host::JobAssignment {
        run: Token::new(job.task),
        attempt: Token::new(job.attempt),
        kind: job.kind,
        spec: job.spec,
        deadline: job.deadline,
        workspace: match job.workspace {
            Some(_) => Some(host::Workspace { workstream: job.workstream, items: Token::new(job.task) }),
            None => None,
        },
        grants: job.grants,
    }
}

/// Relay a host answer; delivery evidence stays with the root's protocol hand-out.
pub fn settled_answer(answer: core::SettledAnswer) -> Result<host::Relayed, core::SettledAnswer> {
    match answer {
        core::SettledAnswer::Host { error, body } => Ok(host::Relayed::Answer { error, text: body }),
        answer @ core::SettledAnswer::Delivery { .. } => Err(answer),
    }
}

/// Plan the fixed core sections within the root's budget and deadline.
#[must_use]
pub fn brief_plan(task: u64, sections: List<brief::Planned>, budget: u32, deadline: Time) -> core::Event {
    core::Event::Brief(brief::GatherEvent::Plan {
        brief: Token::new(task),
        budget,
        deadline,
        sections: sections.into_boxed(),
    })
}

/// Prepare a task with no workspace, credentials or held names.
#[must_use]
pub fn no_workspace(task: u64, attempt: u64) -> core::Event {
    core::Event::WorkspacePrepared {
        task,
        attempt,
        holds: Box::new([]),
        workspace: None,
        credentials: Box::new([]),
        writes: Some(Box::new([])),
    }
}

impl Link {
    /// Keep the brief's order until its claim and report that it is ready.
    #[must_use]
    pub fn complete_brief(&mut self, brief: Token, order: Box<[brief::GatherPlaced]>) -> core::Event {
        self.keep_brief(brief.raw(), order);
        core::Event::BriefAssembled { task: brief.raw(), ready: true }
    }
}
