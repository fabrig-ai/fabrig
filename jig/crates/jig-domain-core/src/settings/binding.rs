//! Resolve a creation's requested charter before checking its effective executor
//! (jig/docs/design/domain/engine/project-charters.md, 4 and 6).
use super::{Overlay, overlays::instruction_join};
use crate::{Charter, Core, Limits, ProjectCharter};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, len_u64, usize_of};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BindingFault {
    NotOffered,
    Project { name: Box<[u8]>, settings: Box<[u8]>, revision: u64, bound: Option<Bound> },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Bound {
    Parts(u32),
    RunBytes(u32),
}

impl Core {
    pub(crate) fn bind_charter(
        &self,
        limits: &Limits,
        project: u32,
        executor: tasks::Executor,
        scope: tasks::CharterScope,
    ) -> Result<Option<tasks::CharterBinding>, BindingFault> {
        let number = match executor {
            tasks::Executor::Agent { charter } => charter,
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => {
                return Ok(None);
            }
        };
        let requested = self.charter(number).ok_or(BindingFault::NotOffered)?;
        let revision = self.project_charters(project);
        if scope == tasks::CharterScope::Deployment || revision == 0 {
            return Ok(None);
        }
        let Some(name) = requested.name.as_deref() else { return Ok(None) };
        let Some(words) = self.project_charter_words(project, revision, name) else { return Ok(None) };
        let Some(charter) = self.charter_named(&words.settings) else { return Err(fault(words, revision, None)) };
        let effective = self.charter(charter).expect("named charter offered");
        let overlay = self.overlay_at(charter, self.current.newest());
        if let Some(bound) = composed_bound(limits, words, effective, overlay) {
            return Err(fault(words, revision, Some(bound)));
        }
        Ok(Some(tasks::CharterBinding { charter, scope: words.name.clone() }))
    }

    #[expect(clippy::result_large_err, reason = "task admission retains the member's problem")]
    pub(crate) fn bind_batch(
        &self,
        limits: &Limits,
        batch: &[tasks::New],
        checked: Option<&[tasks::Executor]>,
    ) -> Result<Box<[Option<tasks::CharterBinding>]>, tasks::Problem> {
        let size = u32::try_from(batch.len()).unwrap_or(u32::MAX);
        if size > limits.tasks.batch {
            return Err(tasks::Problem::new(None, tasks::Refusal::Batch));
        }
        let mut bindings = None;
        for (at, member) in batch.iter().enumerate() {
            let Ok(binding) = self.bind_charter(limits, member.project, member.executor, member.charter_scope) else {
                return Err(tasks::Problem::new(Some(member.number), tasks::Refusal::Executor));
            };
            let effective = executor(member.executor, binding.as_ref());
            if let Some(checked) = checked
                && checked.get(at) != Some(&effective)
            {
                return Err(tasks::Problem::new(Some(member.number), tasks::Refusal::Executor));
            }
            if binding.is_some() && bindings.is_none() {
                let mut slots = List::with_capacity(size);
                for _ in 0..at {
                    slots.push(None).expect("earlier unbound member");
                }
                bindings = Some(slots);
            }
            if let Some(slots) = &mut bindings {
                slots.push(binding).expect("one binding per batch member");
            }
        }
        Ok(match bindings {
            Some(bindings) => bindings.into_boxed(),
            None => Box::new([]),
        })
    }

    #[expect(clippy::manual_find, reason = "step code scans the retained set without a closure")]
    pub(crate) fn project_charter_words(&self, project: u32, revision: u64, name: &[u8]) -> Option<&ProjectCharter> {
        let set = self.project_charter_set(project, revision)?;
        for charter in &set.charters {
            if charter.name.as_ref() == name {
                return Some(charter);
            }
        }
        None
    }
}

pub(crate) fn executor(requested: tasks::Executor, binding: Option<&tasks::CharterBinding>) -> tasks::Executor {
    match binding {
        Some(binding) => tasks::Executor::Agent { charter: binding.charter },
        None => requested,
    }
}

pub(crate) fn executors(batch: &[tasks::New], bindings: &[Option<tasks::CharterBinding>]) -> Box<[tasks::Executor]> {
    let mut checked = List::with_capacity(u32::try_from(batch.len()).expect("bounded batch"));
    for (at, member) in batch.iter().enumerate() {
        checked.push(tasks::bound_executor(member.executor, bindings, at)).expect("one executor per member");
    }
    checked.into_boxed()
}

fn fault(words: &ProjectCharter, revision: u64, bound: Option<Bound>) -> BindingFault {
    BindingFault::Project { name: words.name.clone(), settings: words.settings.clone(), revision, bound }
}

fn composed_bound(
    limits: &Limits,
    words: &ProjectCharter,
    charter: &Charter,
    overlay: Option<&Overlay>,
) -> Option<Bound> {
    let (added, parts) = match overlay {
        Some(overlay) => (overlay.instructions.as_ref(), overlay.parts.as_ref()),
        None => (&[][..], &[][..]),
    };
    if words.parts.len().saturating_add(parts.len()) > usize_of(limits.tasks.charter_parts) {
        return Some(Bound::Parts(limits.tasks.charter_parts));
    }
    match composed_bytes(words, charter, added, parts) {
        Some(bytes) if bytes <= u64::from(limits.run_bytes) => None,
        Some(_) | None => Some(Bound::RunBytes(limits.run_bytes)),
    }
}

fn composed_bytes(
    words: &ProjectCharter,
    charter: &Charter,
    added: &[u8],
    parts: &[crate::CharterPart],
) -> Option<u64> {
    jig_domain_core_vocabulary::policy_owned_bytes(&charter.policy)?
        .checked_sub(len_u64(charter.policy.instructions.len()))?
        .checked_add(len_u64(words.instructions.len()))?
        .checked_add(len_u64(instruction_join(&words.instructions, added).len()))?
        .checked_add(len_u64(added.len()))?
        .checked_add(jig_domain_core_vocabulary::parts_owned_bytes(&words.parts)?)?
        .checked_add(jig_domain_core_vocabulary::parts_owned_bytes(parts)?)
}
