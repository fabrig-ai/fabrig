//! Live run proofs and their restoration bounds (`jig/docs/design/domain/engine.md`, sections 6 and 7).
use crate::{Core, CoreKey, Key, RunProof, TerminalRecord, start};
use jig_domain_tasks as tasks;
use skein_lib::Token;

impl Core {
    /// Whether a live task still holds this exact fenced attempt.
    #[must_use]
    pub fn current_proof(&self, task: u64, attempt: u64) -> bool {
        match self.proofs.get(&task) {
            Some(proof) => proof.attempt == attempt,
            None => false,
        }
    }

    /// Retain the canonical terminal of an unpriced attempt before the task
    /// child makes its next lifecycle decision.
    pub fn remember_unpriced_terminal(&mut self, run: Token, attempt: Token, end: tasks::End) {
        self.start_runtime_input();
        if self.committed_terminal(run.raw(), attempt.raw()).is_some() {
            return;
        }
        let proof = self.proofs.get_mut(&run.raw()).expect("current fleet terminal has pre-reserved proof");
        assert!(proof.attempt == attempt.raw(), "fleet terminal belongs to current proof");
        let cumulative = match proof.turn {
            Some(turn) => turn.cumulative,
            None => 0,
        };
        proof.terminal = Some(TerminalRecord { task: run.raw(), attempt: attempt.raw(), cumulative, end });
    }

    pub(crate) fn committed_terminal(&self, task: u64, attempt: u64) -> Option<TerminalRecord> {
        if self.tasks.task(task)?.last_answer != Some(attempt) {
            return None;
        }
        let proof = self.proofs.get(&task)?;
        if proof.attempt != attempt {
            return None;
        }
        proof.terminal.clone()
    }

    /// Restore one current claim only when it matches the live task's
    /// transient correlation. An ended row can never enter this live map.
    pub(crate) fn restore_proof(&mut self, proof: RunProof, result_bytes: u32) -> bool {
        self.start_input();
        {
            assert!(self.start_loading(), "proofs load before Link");
            let key = Key::Core(CoreKey::RunProof(proof.task));
            assert!(!self.proofs.contains_key(&proof.task), "duplicate {key:?}");
            if !self.start_room(start::LiveSet::RunProofs, key.clone(), self.proofs.len(), self.proofs.capacity()) {
                return false;
            }
            let mut bytes = u64::try_from(proof.outstanding.len()).expect("allocated proof fits u64");
            let mut limit = u64::from(self.run_inbox);
            if bytes <= limit {
                bytes = u64::try_from(proof.accounts.len()).expect("allocated accounts fit u64");
                limit = u64::from(tasks::MODELS);
            }
            if bytes <= limit {
                bytes = u64::try_from(proof.credentials.len()).expect("allocated credentials fit u64");
                limit = u64::from(self.accounts.credential_limit());
            }
            if bytes <= limit
                && let Some(terminal) = &proof.terminal
            {
                bytes = tasks::terminal_bytes(&terminal.end).expect("allocated terminal has a representable measure");
                limit = u64::from(result_bytes).checked_mul(2).expect("result bound");
            }
            if bytes > limit {
                let _refused = self.start_refuse(start::Failure::LiveSet {
                    set: start::LiveSet::RunProofs,
                    key,
                    held: bytes,
                    limit,
                });
                return false;
            }
        }
        self.proofs.insert(proof.task, proof).is_ok()
    }
}
