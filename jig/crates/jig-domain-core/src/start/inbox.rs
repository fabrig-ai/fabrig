//! Inbox projections use the selected people entrance (domain/engine.md, 6, 11.4).
use super::StartMode;
use crate::{Core, Limits, people};
use skein_lib::{Env, Map, Queue, Time, Wall};

impl Core {
    pub(crate) fn project(&mut self, limits: &Limits, input: people::RestoreProjection) {
        match self.start.mode {
            StartMode::LoadAndLink => {
                let projected = self.people.start_project(&limits.people, input);
                self.start_collect_failure();
                assert!(projected || self.start_failure().is_some(), "people projection refusal names its fit");
            }
            StartMode::Default => {
                let event = match input {
                    people::RestoreProjection::Waiting { task, entries } => people::Event::Waiting { task, entries },
                    people::RestoreProjection::ChatState { party, project, task, last_reply, live, seen, replies } => {
                        people::Event::ChatState { party, project, task, last_reply, live, seen, replies }
                    }
                    people::RestoreProjection::Result { party, entry } => {
                        self.people.remember_result(&limits.people, party, entry);
                        return;
                    }
                    people::RestoreProjection::EndedConversation { task } => {
                        self.people.restore_ended_conversation(task);
                        return;
                    }
                };
                let mut out = Queue::uncapped();
                people::step(
                    &mut self.people,
                    &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
                    event,
                    &mut out,
                );
                assert!(out.is_empty(), "restored inbox projections decide nothing");
            }
        }
    }

    pub(super) fn start_inbox_done(&mut self) {
        self.census_progress = Map::with_capacity(self.census_progress.capacity());
    }
}
