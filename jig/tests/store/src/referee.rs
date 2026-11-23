//! Whole-prefix, header, page and transcript promises (protocol/store/testing.md, 1).
use crate::{Restored, model::Model, root};
use jig_domain_core as core;

#[derive(Debug)]
pub struct Violation(pub &'static str);

pub struct Referee;

impl Referee {
    /// A checkpoint recovers exactly its selected prefix, not the later serving state.
    pub fn checkpoint(model: &Model, restored: &Restored, cut: u64) -> Result<(), Violation> {
        if restored.last != cut {
            return Err(Violation("checkpoint recovers exactly its C"));
        }
        let state = model.prefixes.get(&cut).ok_or(Violation("checkpoint has a selected model prefix"))?.clone();
        let selected = Model { acknowledged: cut, prefixes: std::collections::BTreeMap::from([(cut, state)]) };
        Self::check(&selected, restored)
    }

    /// Check recovered data against independently retained commit prefixes.
    ///
    /// # Errors
    /// Returns the first broken store promise.
    pub fn check(model: &Model, restored: &Restored) -> Result<(), Violation> {
        let submitted = *model.prefixes.keys().next_back().expect("initial prefix");
        if restored.last < model.acknowledged || restored.last > submitted {
            return Err(Violation("last acknowledged or a whole submitted prefix"));
        }
        let expected = model.prefixes.get(&restored.last).ok_or(Violation("recovery has a staged prefix"))?;
        let mut live_rows = 0_usize;
        for (key, row) in &expected.rows {
            if matches!(key, core::Key::Tasks(jig_domain_tasks::Key::LiveCold(_))) {
                continue;
            }
            live_rows += 1;
            if restored.rows.get(key) != Some(row) {
                return Err(Violation("restored live rows and fetched task histories equal one whole model prefix"));
            }
        }
        if restored.rows.len() != live_rows || restored.application != expected.application {
            return Err(Violation("restored live rows and fetched task histories equal one whole model prefix"));
        }
        for (task, turns) in &expected.turns {
            if restored.turns(*task) != turns {
                return Err(Violation("every committed turn is readable whole"));
            }
        }
        for (task, turns) in &restored.transcripts {
            if turns.as_slice() != expected.turns.get(task).map_or(&[][..], Vec::as_slice) {
                return Err(Violation("no readable turn past its commit"));
            }
        }
        if restored.last > 0 {
            let key = core::Key::Core(core::CoreKey::Deployment);
            if restored.rows.get(&key) != Some(&root::header(restored.last)) {
                return Err(Violation("deployment header agrees exactly with store number"));
            }
        }
        Ok(())
    }
}
