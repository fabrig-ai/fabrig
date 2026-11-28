//! Safety checks observe the component's boundaries only.
use crate::io::Below;
use skein_lib::Token;
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Passed,
}
#[derive(Debug)]
pub struct Expected {
    done: Option<git::Done>,
    fault: Option<git::Fault>,
    removed: Option<Box<[u8]>>,
}
impl Expected {
    #[must_use]
    pub fn done(done: git::Done) -> Self {
        Self { done: Some(done), fault: None, removed: None }
    }
    #[must_use]
    pub fn fault(fault: git::Fault) -> Self {
        Self { done: None, fault: Some(fault), removed: None }
    }
    /// For an operation with no background work, cleanup must finish before Done.
    #[must_use]
    pub fn after_remove(mut self, path: &[u8]) -> Self {
        self.removed = Some(path.into());
        self
    }
}
#[derive(Debug, Default)]
pub struct Referee {
    pub terminals: Vec<(Token, git::Done)>,
    stopped_at: Option<u32>,
}
impl Referee {
    pub fn stop(&mut self, spawns: u32) {
        self.stopped_at = Some(spawns);
    }
    pub fn observe(&mut self, event: Event, below: &Below, owner: Token, expected: &Expected) {
        let (got, done) = match event {
            Event::Done { owner, done } => (owner, done),
            Event::Unswept { .. } => return,
            Event::Started | Event::Unusable { .. } => panic!("operation returned a startup event"),
        };
        assert_eq!(got, owner);
        assert!(self.terminals.is_empty(), "one terminal per operation");
        assert_eq!(below.closed_children, below.spawns, "all children closed before Done");
        if let Some(path) = &expected.removed {
            assert!(!below.tree.contains_key(path.as_ref()), "temporary file removed before Done: {path:?}");
            assert!(below.file_completions_drained(), "cleanup completions drained before Done");
        }
        self.check(below);
        self.terminals.push((got, done));
    }
    pub fn check(&self, below: &Below) {
        if let Some(at) = self.stopped_at {
            assert_eq!(below.spawns, at, "no invocation starts after stop");
        }
        for arguments in &below.arguments {
            for argument in arguments {
                assert!(!argument.windows(11).any(|part| part == b"secret-git!"));
            }
        }
    }
    #[must_use]
    pub fn finish(&self, expected: &Expected) -> Verdict {
        assert_eq!(self.terminals.len(), 1, "one terminal reached");
        if let Some(done) = &expected.done {
            assert_eq!(&self.terminals[0].1, done);
        }
        if let Some(fault) = &expected.fault {
            match &self.terminals[0].1 {
                git::Done::Failed { fault: got } | git::Done::FailedWithOutput { fault: got, .. } => {
                    assert_eq!(got, fault);
                }
                other @ (git::Done::ParentUnreachable { .. }
                | git::Done::Looked { .. }
                | git::Done::Results { .. }
                | git::Done::Paths { .. }
                | git::Done::Message { .. }
                | git::Done::Brings { .. }
                | git::Done::IsAncestor { .. }
                | git::Done::Merged
                | git::Done::Conflicted { .. }
                | git::Done::Succeeded
                | git::Done::Fetched { .. }
                | git::Done::Committed { .. }
                | git::Done::Unchanged
                | git::Done::Exists
                | git::Done::Rejected) => panic!("expected {fault:?}, got {other:?}"),
            }
        }
        Verdict::Passed
    }
}
