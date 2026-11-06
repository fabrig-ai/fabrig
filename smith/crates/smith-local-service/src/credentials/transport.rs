//! The account driver's socket pass; io and file operations keep their lanes.
//! Completions return by file_layer::owns (skein/docs/design/io.md, 5).
use super::Accounts;
use skein_io as io;
use skein_lib::{Env, Time, Wall};

impl Accounts {
    pub(super) fn socket_up(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits.io };
        io::stage::up(
            &mut self.sockets,
            &env,
            &mut self.socket_completions,
            &mut self.socket_events,
            &mut self.socket_submissions,
            now,
        );
        if let Some(event) = self.socket_events.pop() {
            let env = Env { now, wall, limits: self.limits };
            self.driver.up(&env, event, &mut self.out, &mut self.socket_requests, &mut self.file_requests);
        }
    }

    pub(super) fn socket_down(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits.io };
        if self.sockets.takes()
            && let Some(request) = self.socket_requests.pop()
        {
            io::down(&mut self.sockets, &env, request, &mut self.socket_submissions);
        }
        // Forward only submissions staged at entry; the kernel-facing queue keeps its credit.
        for _ in 0..self.socket_submissions.len() {
            if self.submissions.room() == 0 {
                break;
            }
            let Some(submit) = self.socket_submissions.pop() else { break };
            self.submissions.push(submit);
        }
        self.sockets.reclaim();
        self.driver.reclaim();
    }

    pub(super) fn socket_pending(&self, now: Time) -> bool {
        self.sockets.is_ready()
            || self.sockets.is_due(now)
            || !self.socket_completions.is_empty()
            || !self.socket_submissions.is_empty()
            || !self.socket_events.is_empty()
            || !self.socket_requests.is_empty()
    }

    pub(super) fn socket_empty(&self) -> bool {
        self.sockets.is_empty()
            && self.socket_completions.is_empty()
            && self.socket_submissions.is_empty()
            && self.socket_events.is_empty()
            && self.socket_requests.is_empty()
    }
}

