//! Visitor projections from retained forge facts (temper/docs/design/client/forge/README.md, 9).
pub use temper_engine_domain_forge_vocabulary::public::*;
mod change;
mod landing;
mod landings;
mod merges;
use crate::{ChangeRow, Domain, Limits};
use alloc::boxed::Box;
use skein_lib::{List, Wall, Writer};
use temper_client_protocol_wire::public_v1 as w;
use temper_engine_domain_forge_landing as policy;

struct Names {
    tasks: List<u64>,
}
impl Names {
    fn new() -> Self {
        // jig's application envelope admits at most 1024 task references.
        Self { tasks: List::with_capacity(1024) }
    }
    fn place(&self, task: u64) -> Option<u16> {
        for (place, previous) in self.tasks.iter().enumerate() {
            if *previous == task {
                return u16::try_from(place).ok();
            }
        }
        None
    }
    fn name(&mut self, task: u64) -> Option<u16> {
        for (place, previous) in self.tasks.iter().enumerate() {
            if *previous == task {
                return u16::try_from(place).ok();
            }
        }
        let place = u16::try_from(self.tasks.len()).ok()?;
        self.tasks.push(task).ok()?;
        Some(place)
    }
}
impl Domain {
    /// Make only the visitor's record, or the single unknown result (client/forge/README.md, 9.2).
    #[must_use]
    pub fn public(&self, ask: &PublicAsk<'_>, limits: &Limits) -> Option<PublicMade> {
        if ask.version != w::PUBLIC_VERSION {
            return None;
        }
        let mut names = Names::new();
        let record = match ask.kind {
            w::PUBLIC_CHANGE => {
                let row = self.public_change(ask, ask.task?)?;
                let record = self.make_change(ask, row, limits, &mut names)?;
                let mut writer = Writer::new(usize::try_from(record.measure()).ok()?);
                record.encode(&mut writer).ok()?;
                writer.finish()
            }
            w::PUBLIC_LANDINGS => {
                if ask.task.is_some() {
                    return None;
                }
                let record = self.make_landings(ask, limits, &mut names)?;
                let mut writer = Writer::new(usize::try_from(record.measure()).ok()?);
                record.encode(&mut writer).ok()?;
                writer.finish()
            }
            w::PUBLIC_LANDING => {
                let row = self.public_change(ask, ask.task?)?;
                let record = match self.newest_landing(row.task) {
                    Some(landing) => {
                        if !self.public_landing(ask, landing) {
                            return None;
                        }
                        self.make_landing(landing, &mut names)?
                    }
                    None => self.make_merge(ask, row, &mut names)?,
                };
                let mut writer = Writer::new(usize::try_from(record.measure()).ok()?);
                record.encode(&mut writer).ok()?;
                writer.finish()
            }
            _ => return None,
        };
        Some(PublicMade { tasks: names.tasks.into_boxed(), record })
    }
    fn public_change(&self, ask: &PublicAsk<'_>, task: u64) -> Option<&ChangeRow> {
        ask.handed(task)?;
        let row = self.changes.get(&task)?;
        if self.repositories.get(&row.repository)?.project != ask.project {
            return None;
        }
        Some(row)
    }
    fn public_landing(&self, ask: &PublicAsk<'_>, landing: &policy::Landing) -> bool {
        if landing.project != ask.project {
            return false;
        }
        for member in &landing.members {
            if self.public_change(ask, member.change).is_none() {
                return false;
            }
        }
        true
    }
    pub(crate) fn newest_landing(&self, change: u64) -> Option<&policy::Landing> {
        let mut newest: Option<&policy::Landing> = None;
        for (_, row) in &self.landing.rows {
            for member in &row.landing.members {
                if member.change == change
                    && match newest {
                        Some(last) => last.task < row.landing.task,
                        None => true,
                    }
                {
                    newest = Some(&row.landing);
                }
            }
        }
        newest
    }
    pub(crate) fn repository_address(&self, row: &ChangeRow) -> Option<Box<[u8]>> {
        let repo = self.repositories.get(&row.repository)?;
        let Some(forge) = self.configured.get(&row.repository.forge) else {
            return Some(Box::default());
        };
        if forge.mode == crate::Mode::Fake {
            return Some(Box::default());
        }
        let mut base = match forge.kind {
            crate::Kind::GitHub => forge.web.as_deref().unwrap_or_default(),
            crate::Kind::Forgejo => &forge.git,
        };
        while base.last() == Some(&b'/') {
            base = base.get(..base.len().checked_sub(1)?)?;
        }
        if base.is_empty() {
            return Some(Box::default());
        }
        let parts = [base, b"/", &repo.owner, b"/", &repo.name];
        let mut length = 0_usize;
        for part in parts {
            length = length.checked_add(part.len())?;
        }
        if length > usize::try_from(crate::PULL_ADDRESS_BYTES).ok()? {
            return None;
        }
        let mut writer = Writer::new(length);
        for part in parts {
            writer.put(part).ok()?;
        }
        Some(writer.finish())
    }
}
fn phase(phase: &policy::Phase) -> w::PublicLandingPhase {
    match phase {
        policy::Phase::Waiting => w::PublicLandingPhase::Waiting,
        policy::Phase::Preparing => w::PublicLandingPhase::Preparing,
        policy::Phase::Proving => w::PublicLandingPhase::Proving,
        policy::Phase::Proved { .. } => w::PublicLandingPhase::Proved,
        policy::Phase::Turn { .. } => w::PublicLandingPhase::Turn,
        policy::Phase::Landed => w::PublicLandingPhase::Landed,
        policy::Phase::Held { .. } => w::PublicLandingPhase::Held,
        policy::Phase::Cancelled => w::PublicLandingPhase::Cancelled,
    }
}

/// Select the panels' order over exactly the supplied candidates (domain/forge/queues.md).
pub(crate) fn order_ready(
    mut candidates: List<temper_engine_domain_forge_change::Ready>,
    repair: Option<u64>,
    limits: &Limits,
    now: Wall,
) -> Option<List<temper_engine_domain_forge_change::Ready>> {
    let mut ordered = List::with_capacity(candidates.capacity());
    // Each selection removes one admitted candidate.
    while !candidates.is_empty() {
        let mut first = 0;
        for (index, candidate) in candidates.iter().enumerate() {
            if crate::panel::queue_before(candidate, candidates.get(first)?, repair, limits, now) {
                first = u32::try_from(index).ok()?;
            }
        }
        let candidate = candidates.swap_remove(first)?;
        ordered.push(candidate).ok()?;
    }
    Some(ordered)
}
