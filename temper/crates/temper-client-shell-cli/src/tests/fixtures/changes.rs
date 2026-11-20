//! A change panel on the HTTP fake, with not-ready supplemental reads.
use super::*;
use temper_client_protocol_wire::{self as wire, v1 as f};
// jig-client-test-records' task chip is 7 in project 3.
const TASK: u64 = 7;
pub const TIMEOUT: Duration = Duration::from_secs(12);
impl Frame {
    pub fn change_wait(timeout: &str) -> Self {
        Self::change_read(&["change", "show", "7", "--project", "3", "--until", "ci:failed", "--timeout", timeout])
    }
    /// Sign in and deliver a queue change on the requested forge watch.
    pub fn change_read(words: &[&str]) -> Self {
        let mut frame = Self::read(words);
        frame.change = true;
        frame.sign_in_then_deliver_me();
        frame
    }
    pub(super) fn deliver_change(&mut self) {
        let used = f::Used { used: 0, limit: 3 };
        let panel = f::ChangePanel {
            wait: None,
            source: None,
            way: f::Way::Queue,
            in_landing: None,
            task: TASK,
            forge: 3,
            owner: b"qa".as_slice().into(),
            name: b"restart".as_slice().into(),
            branch: b"temper/7".as_slice().into(),
            landing: b"main".as_slice().into(),
            head: None,
            pull: None,
            step: f::Step::Gating,
            hold: Some(f::Hold::Stalled),
            runs: None,
            ci_required: true,
            ci: f::Ci::Pending,
            approvals: skein_lib::List::with_capacity(0),
            unmapped_reviews: skein_lib::List::with_capacity(0),
            gates: skein_lib::List::with_capacity(0),
            repairs: used.clone(),
            resolutions: used.clone(),
            updates: used,
            questions: f::Questions {
                route: f::Route::None,
                taken: f::Used { used: 0, limit: 4 },
                open: skein_lib::List::with_capacity(0),
            },
        };
        let document = w::Owned {
                owner: w::Owner::Family(
                    w::Family { number: wire::FORGE },
                ),
                version: wire::CLIENT_VERSION,
                record: wire::change_panel(&panel),
            };
        let event = w::StreamEvent::Delivery(
            w::Delivery { watch: self.wait_task.expect("change watch"), document },
        );
        self.sockets.get_mut(&self.stream.expect("stream")).expect("stream socket").reply.extend(sse(&event));
    }
    pub fn change_task_not_ready(&mut self) {
        self.read_unreachable = true;
        let event = w::StreamEvent::Ended(w::WatchEnded {
            watch: self.wait_task.expect("change watch"),
            why: w::WatchEnd::Unknown,
        });
        self.sockets.get_mut(&self.stream.expect("stream")).expect("stream socket").reply.extend(sse(&event));
        self.run();
    }
    pub fn change_task_ready(&mut self) {
        self.read_unreachable = false;
        self.advance_past(crate::limits::native().root.domain.client.domain.backoff.first);
    }
    pub fn engine_restarted(&mut self) {
        self.stopping = None;
        self.advance_past(crate::limits::native().root.domain.client.domain.backoff.first);
    }
}
