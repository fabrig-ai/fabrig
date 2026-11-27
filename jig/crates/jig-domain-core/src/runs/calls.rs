//! Named calls retained by live runs (`jig/docs/design/domain/engine/vocabulary.md`, section 1 and `jig/docs/design/domain/engine/executors.md`).
use crate::{CallKey, CallPart, Core, CoreKey, Key, start};
use alloc::boxed::Box;
use skein_lib::List;

/// The route that continues a pending named call (`jig/docs/design/domain/engine/executors-tools.md`, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum CallHolder {
    Routed,
    Application,
    DelegateInput,
    DecidedProposal(u64),
}

impl Core {
    /// A decided effect must settle before its task resumes.
    pub(crate) fn unsettled_effect(&self, task: u64) -> bool {
        for (key, part) in &self.call_parts {
            if key.task != task {
                continue;
            }
            match part {
                CallPart::Effect { outcome, .. } => match outcome {
                    None
                    | Some(
                        crate::effects::connector::OutboxOutcome::Uncertain
                        | crate::effects::connector::OutboxOutcome::Held { .. },
                    ) => return true,
                    Some(
                        crate::effects::connector::OutboxOutcome::Made
                        | crate::effects::connector::OutboxOutcome::Failed
                        | crate::effects::connector::OutboxOutcome::Withdrawn
                        | crate::effects::connector::OutboxOutcome::NotMade,
                    ) => {}
                },
                CallPart::Connector { .. }
                | CallPart::EffectRefused(_)
                | CallPart::EffectDenied { .. }
                | CallPart::ToolDenied { .. }
                | CallPart::EscalationDecided { .. }
                | CallPart::EscalationRefused(_)
                | CallPart::Proposed { .. }
                | CallPart::ProposalDecided { .. }
                | CallPart::ProposalRefused(_)
                | CallPart::SettingsRefused(_)
                | CallPart::HoldOwner { .. }
                | CallPart::Controlled
                | CallPart::ControlRefused(_)
                | CallPart::ControlDenied { .. }
                | CallPart::Sent { .. }
                | CallPart::Introduced
                | CallPart::MessageRefused(_)
                | CallPart::Subscribed { .. }
                | CallPart::Unsubscribed
                | CallPart::SubscriptionRefused(_)
                | CallPart::Delegated(_)
                | CallPart::DelegationDenied { .. }
                | CallPart::DelegationRefused(_)
                | CallPart::NoteWritten { .. }
                | CallPart::NoteRecalled { .. }
                | CallPart::NoteRefused(_)
                | CallPart::NoteDeleted { .. }
                | CallPart::WordsRecorded { .. }
                | CallPart::Revised { .. }
                | CallPart::Moved { .. }
                | CallPart::Again { .. }
                | CallPart::PassedUp { .. }
                | CallPart::Unavailable => {}
            }
        }
        false
    }

    /// Whether a named call holds a pending place.
    #[must_use]
    pub fn call_pending(&self, key: CallKey) -> bool {
        self.pending_calls.contains_key(&key)
    }

    /// Whether a named call can proceed or replay its retained answer.
    /// Pending calls wait; new calls share the configured record bound with
    /// each other alone, never with kept parts (domain/engine/executors-transcripts.md, 1).
    #[must_use]
    pub fn call_room(&self, key: CallKey) -> bool {
        !self.pending_calls.contains_key(&key) && (self.call_parts.contains_key(&key) || self.call_places_available())
    }

    fn new_call_room(&self, key: CallKey) -> bool {
        !self.pending_calls.contains_key(&key) && !self.call_parts.contains_key(&key) && self.call_places_available()
    }

    fn call_places_available(&self) -> bool {
        match self.pending_calls.len().checked_add(self.call_parts.len()) {
            Some(held) => held < self.call_parts.capacity(),
            None => false,
        }
    }

    /// Admit a new name, never a replay or a replacement.
    #[must_use]
    pub(crate) fn reserve_call(&mut self, key: CallKey, owner: CallHolder) -> bool {
        if !self.new_call_room(key) {
            return false;
        }
        self.record_pending_call(key, owner);
        true
    }

    fn record_pending_call(&mut self, key: CallKey, owner: CallHolder) {
        assert!(self.new_call_room(key), "named call {key:?} must be new with room to go pending");
        let prior = self.pending_calls.insert(key, owner);
        assert!(prior == Ok(None), "named call {key:?} takes its pending place once");
    }

    /// Continue the same place, without readmitting the name (executors-tools.md, 1).
    pub(crate) fn handoff_call(&mut self, key: CallKey, from: CallHolder, to: CallHolder) {
        let Some(owner) = self.pending_calls.get_mut(&key) else {
            unreachable!("named call {key:?} holds a pending place for handoff from {from:?}")
        };
        assert!(*owner == from, "named call {key:?} handoff belongs to {from:?}, not {owner:?}");
        *owner = to;
    }

    pub(crate) fn release_pending_call(&mut self, key: CallKey) {
        assert!(!self.call_parts.contains_key(&key), "named call {key:?} gives back a pending, not a retained, place");
        let pending = self.pending_calls.remove(&key);
        assert!(pending.is_some(), "named call {key:?} gives back its pending place once");
    }

    /// Move the newly decided call's place from pending to retained. The
    /// application saves the row and holds the answer behind that commit.
    pub fn record_call(&mut self, key: CallKey, part: CallPart) {
        self.start_runtime_input();
        self.release_pending_call(key);
        assert!(self.new_call_room(key), "named call {key:?} must be new with room to be retained");
        let prior = self.call_parts.insert(key, part);
        assert!(prior == Ok(None), "named call {key:?} takes its retained place once");
    }

    /// Restore one live named call only after its current task proof was
    /// restored. The connector's separately owned answer is validated by the
    /// application before it asks the core to retain this part.
    pub(crate) fn restore_call(&mut self, key: CallKey, part: CallPart) -> bool {
        self.start_input();
        {
            assert!(self.start_loading(), "named call {key:?} loads before Link");
            assert!(self.pending_calls.is_empty(), "named call {key:?} loads before any runtime reservation");
            assert!(!self.call_parts.contains_key(&key), "duplicate call {:?}", Key::Core(CoreKey::Call(key)));
            if !self.start_room(
                start::LiveSet::Calls,
                Key::Core(CoreKey::Call(key)),
                self.call_parts.len(),
                self.call_parts.capacity(),
            ) {
                return false;
            }
        }
        let prior = self.call_parts.insert(key, part);
        assert!(prior == Ok(None), "restored named call {key:?} takes its retained place once");
        true
    }

    /// Forget named calls whose answer entered a committed later turn. The
    /// application erases these keys in the same decision and drops any
    /// connector-owned answer for them.
    pub fn retire_calls(&mut self, task: u64, attempt: u64, turn: u32, room: u32) -> Box<[CallKey]> {
        self.start_runtime_input();
        let mut retired = List::with_capacity(self.call_parts.len().min(room));
        for (&key, _) in &self.call_parts {
            if key.task == task && (key.attempt < attempt || (key.attempt == attempt && key.completion < turn)) {
                let added = retired.push(key);
                assert!(added.is_ok(), "retained call {key:?} fits the retirement list");
            }
        }
        for &key in &retired {
            assert!(!self.pending_calls.contains_key(&key), "retired call {key:?} is retained, not pending");
            let Some(part) = self.call_parts.remove(&key) else {
                unreachable!("retired call {key:?} still holds its enumerated place")
            };
            self.call_settled.remove(&key);
            match part {
                CallPart::Effect { entry, .. } => {
                    self.effect_replies.remove(&entry);
                }
                CallPart::Connector { .. }
                | CallPart::EffectRefused(_)
                | CallPart::EffectDenied { .. }
                | CallPart::ToolDenied { .. }
                | CallPart::EscalationDecided { .. }
                | CallPart::EscalationRefused(_)
                | CallPart::Proposed { .. }
                | CallPart::ProposalDecided { .. }
                | CallPart::SettingsRefused(_)
                | CallPart::ProposalRefused(_)
                | CallPart::HoldOwner { .. }
                | CallPart::Controlled
                | CallPart::ControlRefused(_)
                | CallPart::ControlDenied { .. }
                | CallPart::Sent { .. }
                | CallPart::Introduced
                | CallPart::MessageRefused(_)
                | CallPart::Subscribed { .. }
                | CallPart::Unsubscribed
                | CallPart::SubscriptionRefused(_)
                | CallPart::Delegated(_)
                | CallPart::DelegationDenied { .. }
                | CallPart::DelegationRefused(_)
                | CallPart::WordsRecorded { .. }
                | CallPart::NoteWritten { .. }
                | CallPart::NoteDeleted { .. }
                | CallPart::NoteRecalled { .. }
                | CallPart::NoteRefused(_)
                | CallPart::Revised { .. }
                | CallPart::Moved { .. }
                | CallPart::Again { .. }
                | CallPart::PassedUp { .. }
                | CallPart::Unavailable => {}
            }
        }
        retired.into_boxed()
    }
}
