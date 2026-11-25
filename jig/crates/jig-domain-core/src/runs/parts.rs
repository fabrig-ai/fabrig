//! Parts kept between turns (domain/engine/executors.md, 2).
use crate::{Core, CoreKey, CoreRecord, Held, Key, PartRecord, Record, Request, Write, start};
use alloc::boxed::Box;
use jig_domain_link::KeptPart;
use skein_lib::{List, Map, Queue, Set, length::len_u64};

impl Core {
    pub(crate) fn restore_part(&mut self, record: PartRecord) -> bool {
        self.start_input();
        assert!(self.start_loading(), "parts load before Link");
        assert!(!self.kept_parts.contains_key(&record.key), "duplicate part");
        if !self.start_room(
            start::LiveSet::Parts,
            Key::Core(CoreKey::Part(record.key)),
            self.kept_parts.len(),
            self.kept_parts.capacity(),
        ) {
            return false;
        }
        self.kept_parts.insert(record.key, record).is_ok()
    }

    pub(crate) fn keep_part(&mut self, record: PartRecord) -> Queue<Request> {
        let mut out = Queue::uncapped();
        let key = record.key;
        if !self.current_proof(key.task, key.attempt) {
            return out;
        }
        let proof = self.proofs.get(&key.task).expect("current proof");
        if key.number <= proof.kept_part {
            out.push(Request::Held(Box::new(Held::TaskPartKept { key })));
            return out;
        }
        if proof.kept_part.checked_add(1) != Some(key.number) {
            out.push(Request::Now(Box::new(crate::Now::PartBusy { key })));
            return out;
        }
        let committed = match proof.turn {
            Some(turn) => turn.turn,
            None => 0,
        };
        if record.turn > committed && !self.kept_parts.contains_key(&key) {
            if self.kept_parts.len() >= self.kept_parts.capacity() {
                // A full room never stops the run: its task's parts go and this
                // one is acknowledged unkept, so a resume finds no answer for
                // the cut turn and plans it afresh (domain/engine/executors-transcripts.md, 1).
                self.retire_parts(key.task, None, &mut out);
            } else {
                assert!(self.kept_parts.insert(key, record.clone()).is_ok(), "kept part room checked");
                out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Part(record)))));
            }
        }
        let proof = self.proofs.get_mut(&key.task).expect("current part proof");
        if key.number > proof.kept_part {
            proof.kept_part = key.number;
            out.push(Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(proof.clone())))));
        }
        out.push(Request::Held(Box::new(Held::TaskPartKept { key })));
        out
    }

    pub(crate) fn retire_parts(&mut self, task: u64, through: Option<(u64, u32)>, out: &mut Queue<Request>) {
        let mut retired = List::with_capacity(self.kept_parts.len());
        for (&key, part) in &self.kept_parts {
            let moot = match through {
                Some((attempt, turn)) => key.attempt < attempt || (key.attempt == attempt && part.turn <= turn),
                None => true,
            };
            if key.task == task && moot {
                retired.push(key).expect("retired parts fit retained parts");
            }
        }
        for key in retired.into_boxed() {
            self.kept_parts.remove(&key);
            out.push(Request::Write(Write::Erase(Key::Core(CoreKey::Part(key)))));
        }
    }

    pub(crate) fn last_part(&self, task: u64, attempt: u64) -> u32 {
        let mut last = match self.proofs.get(&task) {
            Some(proof) if proof.attempt == attempt => proof.kept_part,
            Some(_) | None => 0,
        };
        for (key, _) in &self.kept_parts {
            if key.task == task && key.attempt == attempt {
                last = last.max(key.number);
            }
        }
        last
    }

    /// Main's parts first, then whole sub-agent calls in call order while they fit.
    pub(crate) fn resumed_parts(&self, task: u64, attempt: u64, window_bytes: u64) -> Box<[KeptPart]> {
        let mut out = List::with_capacity(self.kept_parts.len());
        let Some(mut room) = u64::from(self.settings.resume_bytes).checked_sub(window_bytes) else {
            return out.into_boxed();
        };
        let mut main = 0_u64;
        let mut calls = Map::<(u64, u32), u64>::with_capacity(self.kept_parts.len());
        for (key, part) in &self.kept_parts {
            if key.task != task || key.attempt >= attempt {
                continue;
            }
            let size = len_u64(part.body.len());
            if part.thread == 0 {
                main = main.checked_add(size).expect("retained main bytes fit u64");
            } else {
                let group = (key.attempt, part.call);
                let bytes = calls.get(&group).copied().unwrap_or(0).checked_add(size).expect("call bytes fit u64");
                calls.insert(group, bytes).expect("one group per retained part");
            }
        }
        if main > room {
            return out.into_boxed();
        }
        room = room.checked_sub(main).expect("main fits");
        let mut selected = Set::with_capacity(calls.len());
        for (group, size) in &calls {
            if *size > room {
                break;
            }
            room = room.checked_sub(*size).expect("whole call fits");
            selected.insert(*group).expect("one selection per call");
        }
        for (key, part) in &self.kept_parts {
            if key.task == task
                && key.attempt < attempt
                && (part.thread == 0 || selected.contains(&(key.attempt, part.call)))
            {
                out.push(KeptPart {
                    attempt: key.attempt,
                    turn: part.turn,
                    thread: part.thread,
                    call: part.call,
                    kind: part.kind,
                    body: part.body.clone(),
                })
                .expect("selected parts fit retained parts");
            }
        }
        out.into_boxed()
    }
}
