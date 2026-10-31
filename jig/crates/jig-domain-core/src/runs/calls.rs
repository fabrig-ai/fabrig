//! Named calls retained by live runs (`jig/docs/design/domain/engine.md`, sections 4 and 7).
use crate::{CallKey, CallPart, Core, CoreKey, Key, start};
use alloc::boxed::Box;
use skein_lib::List;

impl Core {
    /// Whether a named call can proceed or replay its retained answer.
    /// Pending calls wait; new calls share the configured record bound.
    #[must_use]
    pub fn call_room(&self, key: CallKey) -> bool {
        !self.pending_calls.contains_key(&key)
            && (self.call_parts.contains_key(&key)
                || self.call_parts.len().saturating_add(self.pending_calls.len()) < self.call_parts.capacity())
    }

    /// Retain one newly accepted named call decision. The application saves
    /// its store row and holds the outward answer behind the same commit.
    pub fn record_call(&mut self, key: CallKey, part: CallPart) {
        self.start_runtime_input();
        assert!(self.call_parts.insert(key, part) == Ok(None), "named call room reserved");
    }

    /// Restore one live named call only after its current task proof was
    /// restored. The connector's separately owned answer is validated by the
    /// application before it asks the core to retain this part.
    pub(crate) fn restore_call(&mut self, key: CallKey, part: CallPart) -> bool {
        self.start_input();
        {
            assert!(self.start_loading(), "calls load before Link");
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
        self.call_parts.insert(key, part) == Ok(None)
    }

    /// Forget named calls whose answer entered a committed later turn. The
    /// application erases these keys in the same decision and drops any
    /// connector-owned answer for them.
    pub fn retire_calls(&mut self, task: u64, attempt: u64, turn: u32, room: u32) -> Box<[CallKey]> {
        self.start_runtime_input();
        let mut retired = List::with_capacity(self.call_parts.len());
        for (&key, _) in &self.call_parts {
            if key.task == task && (key.attempt < attempt || (key.attempt == attempt && key.completion < turn)) {
                retired.push(key).expect("all retained call names fit their configured bound");
            }
        }
        assert!(retired.len() <= room, "retired calls fit their configured bound");
        for &key in &retired {
            self.call_settled.remove(&key);
            match self.call_parts.remove(&key) {
                Some(CallPart::Effect { entry, .. }) => {
                    self.effect_replies.remove(&entry);
                }
                Some(
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
                    | CallPart::NoteWritten { .. }
                    | CallPart::NoteRecalled { .. }
                    | CallPart::NoteRefused(_)
                    | CallPart::Revised { .. }
                    | CallPart::Moved { .. }
                    | CallPart::Again { .. }
                    | CallPart::PassedUp { .. }
                    | CallPart::Unavailable,
                )
                | None => {}
            }
        }
        retired.into_boxed()
    }
}
