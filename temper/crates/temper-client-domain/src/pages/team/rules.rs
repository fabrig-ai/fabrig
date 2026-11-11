//! The selected role's charter, using the Role page's reader (client/pages/team.md, 1).
use crate::{
    Address, forge,
    pages::{Request, roles},
};
use alloc::boxed::Box;
use jig_client::{Client, contract as c};
use skein_lib::Queue;
use temper_client_protocol_wire::v1 as w;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub charter: u32,
    pub asked: Box<[u8]>,
}
#[derive(Debug)]
pub(super) struct Reader {
    role: Option<u64>,
    charter: Option<u32>,
    read: roles::Roles,
}
#[expect(clippy::derivable_impls, reason = "the step subset names each initial read state")]
impl Default for Reader {
    fn default() -> Self {
        Self { role: None, charter: None, read: roles::Roles::default() }
    }
}
impl Reader {
    pub(super) fn configure(
        &mut self,
        task: Option<&c::Task>,
        client: &Client,
        limits: forge::Limits,
        next: &mut u64,
        out: &mut Queue<Request>,
    ) {
        let Some(task) = task else {
            self.leave(out);
            return;
        };
        let charter = match task.chip.executor {
            c::ChipExecutor::Agent { charter } => charter,
            c::ChipExecutor::Procedure { .. }
            | c::ChipExecutor::Party { .. }
            | c::ChipExecutor::Role { .. }
            | c::ChipExecutor::Job { .. } => {
                self.leave(out);
                return;
            }
        };
        if self.role != Some(task.chip.task) {
            self.leave(out);
        }
        self.role = Some(task.chip.task);
        self.charter = Some(charter);
        self.read.configure(&Address::Role { project: task.project, charter }, client, limits, next, out);
    }
    pub(super) fn leave(&mut self, out: &mut Queue<Request>) {
        self.read.leave(out);
        self.role = None;
        self.charter = None;
    }
    pub(super) fn told(&mut self, told: &jig_client::Told, limits: forge::Limits) -> bool {
        self.read.told(told, limits)
    }
    pub(super) fn facts(&self) -> Option<Rules> {
        asked(self.charter?, self.read.read())
    }
}
/// Only the charter read's Asked to fact supplies rules; never a placeholder.
#[must_use]
pub fn asked(charter: u32, read: roles::Read<'_>) -> Option<Rules> {
    let roles::Read::Drawn(w::View::Charter(view)) = read else { return None };
    if view.charter() != charter {
        return None;
    }
    for fact in view.card().facts() {
        if fact.label() == b"Asked to" && !fact.value().is_empty() {
            return Some(Rules { charter, asked: fact.value().into() });
        }
    }
    None
}
