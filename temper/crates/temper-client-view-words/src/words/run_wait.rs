//! A task's next-run refusal (temper/docs/design/client/pages/accounts-words.md, 1).
use super::{TimeForm, Times, Word as W, Words};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::phrases::Hole;

fn sentences(w: &Words, first: Box<[u8]>, next: Box<[u8]>) -> Box<[u8]> {
    w.say(W::RefusalSentences, &[Hole::Name(first), Hole::Name(next)])
}

impl Words {
    /// The task page's explanation of a wait on an account source.
    #[must_use]
    pub fn run_source_wait(&self, state: c::TaskSourceState) -> Box<[u8]> {
        use super::accounts::{Word as A, say};
        use c::TaskSourceState as S;
        let (word, until) = match state {
            S::Moving => return self.say(W::WaitMoving, &[]),
            S::Asking => (A::Asking, None),
            S::Granted => (A::Granted, None),
            S::Limited { until } => (A::Limited, until),
            S::Later { until } => (A::Later, until),
            S::NeedsSignIn => (A::NeedsSignIn, None),
            S::Refused => (A::Refused, None),
            S::Misconfigured => (A::Misconfigured, None),
            S::Cancelled => (A::Cancelled, None),
            S::Closing => (A::Closing, None),
            S::Unloaded { at, why } => {
                let place = match at {
                    c::AccountPlace::Directory => A::Directory,
                    c::AccountPlace::File => A::File,
                };
                let why = match why {
                    c::AccountUnread::Unreadable => A::Unreadable,
                    c::AccountUnread::Refused => A::Refused,
                    c::AccountUnread::Failed => A::Failed,
                    c::AccountUnread::Stalled => A::Stalled,
                    c::AccountUnread::TooLarge => A::TooLarge,
                };
                return skein_lib::bytes::concat(&[
                    &say(self, A::Unloaded, &[]),
                    &say(self, place, &[]),
                    b": ",
                    &say(self, why, &[]),
                ]);
            }
        };
        let first = say(self, word, &[]);
        match until {
            Some(until) => skein_lib::bytes::concat(&[&first, &say(self, A::Until, &[Hole::Wall(until)])]),
            None => first,
        }
    }

    /// The task page's explanation of a refused next run.
    #[must_use]
    pub fn run_wait_refused(&self, times: &mut Times, why: c::TaskWaitRefused) -> Box<[u8]> {
        let w = self;
        match why {
            c::TaskWaitRefused::Bounds { kind, why, figures } => bound_refusal(w, times, kind, why, figures, None),
            c::TaskWaitRefused::Job { worker, kind, why, figures } => w.job_wait(worker, kind, why, figures),
            c::TaskWaitRefused::Hello { kind, at, why, figures } => {
                match why {
                    c::HelloRefusal::Agent { why } if figures.is_some() => {
                        return bound_refusal(w, times, kind, Some(why), figures, Some(at));
                    }
                    c::HelloRefusal::Agent { .. }
                    | c::HelloRefusal::JobKinds
                    | c::HelloRefusal::ReservedChannel
                    | c::HelloRefusal::Position { .. }
                    | c::HelloRefusal::Workers { .. }
                    | c::HelloRefusal::StopBound { .. } => {}
                }
                let word = match kind {
                    c::HostKind::Engine => W::RefusedEngineAt,
                    c::HostKind::Worker => W::RefusedWorkerAt,
                };
                let first = w.say(word, &[Hole::Name(times.say(w, TimeForm::Clock, at))]);
                let cause = sentences(w, first, hello_refusal(w, why));
                w.say(W::WaitRefused, &[Hole::Name(cause)])
            }
        }
    }
}

/// Bound figures keep their units and the fleet's largest offer (temper/docs/design/client/pages/accounts-words.md, 1).
fn bound_refusal(
    w: &Words,
    times: &mut Times,
    kind: c::HostKind,
    why: Option<c::AgentShortfall>,
    figures: Option<c::Figures>,
    at: Option<skein_lib::Wall>,
) -> Box<[u8]> {
    use c::AgentShortfall as S;
    let (lead, alone) = match kind {
        c::HostKind::Worker => (W::BoundWorker, W::BoundNoWorker),
        c::HostKind::Engine => (W::BoundEngine, W::BoundNoEngine),
    };
    let Some(why) = why else {
        return w.say(lead, &[Hole::Name(w.say(W::BoundDifferent, &[]))]);
    };
    let Some(figures) = figures else {
        return sentences(w, w.say(alone, &[]), agent_shortfall(w, why));
    };
    let charter = match why {
        S::Model { charter, .. }
        | S::Window { charter, .. }
        | S::Output { charter, .. }
        | S::Turns { charter }
        | S::Time { charter }
        | S::Waiting { charter }
        | S::Checks { charter } => charter,
        S::Spend | S::Message | S::Inbox | S::Unacknowledged => 0,
    };
    let charter = match at {
        Some(_) => w.say(W::BoundHelloCharter, &[Hole::Number(u64::from(charter))]),
        None => w.say(W::BoundCharter, &[]),
    };
    let required = bound_figure(w, why, figures.required);
    let offered = bound_figure(w, why, figures.offered);
    let clause = match why {
        S::Turns { .. } => w.say(W::BoundTurns, &[Hole::Name(charter), Hole::Name(required)]),
        S::Time { .. } => w.say(W::BoundTime, &[Hole::Name(charter), Hole::Name(required)]),
        S::Waiting { .. } => w.say(W::BoundWaiting, &[Hole::Name(charter), Hole::Name(required)]),
        S::Checks { .. } => w.say(W::BoundChecks, &[Hole::Name(charter), Hole::Name(required)]),
        S::Window { model, .. } => {
            w.say(W::BoundWindow, &[Hole::Name(charter), Hole::Number(u64::from(model)), Hole::Name(required)])
        }
        S::Output { model, .. } => {
            w.say(W::BoundOutput, &[Hole::Name(charter), Hole::Number(u64::from(model)), Hole::Name(required)])
        }
        S::Spend => w.say(W::BoundSpend, &[Hole::Name(required)]),
        S::Message => w.say(W::BoundMessage, &[Hole::Name(required)]),
        S::Inbox => w.say(W::BoundInbox, &[Hole::Name(required)]),
        S::Unacknowledged => w.say(W::BoundUnacknowledged, &[Hole::Name(required)]),
        S::Model { .. } => return sentences(w, w.say(alone, &[]), agent_shortfall(w, why)),
    };
    let tail = match kind {
        c::HostKind::Engine => w.say(W::BoundTakes, &[Hole::Name(offered)]),
        c::HostKind::Worker => match at {
            Some(at) => w.say(W::BoundRefusedAt, &[Hole::Name(times.say(w, TimeForm::Clock, at)), Hole::Name(offered)]),
            None => {
                let tail = match why {
                    S::Checks { .. } => W::BoundQuickest,
                    S::Model { .. }
                    | S::Window { .. }
                    | S::Output { .. }
                    | S::Turns { .. }
                    | S::Spend
                    | S::Time { .. }
                    | S::Waiting { .. }
                    | S::Message
                    | S::Inbox
                    | S::Unacknowledged => W::BoundLargest,
                };
                w.say(tail, &[Hole::Name(offered)])
            }
        },
    };
    w.say(lead, &[Hole::Name(sentences(w, clause, tail))])
}
fn bound_figure(w: &Words, why: c::AgentShortfall, value: u64) -> Box<[u8]> {
    use c::AgentShortfall as S;
    match why {
        S::Time { .. } | S::Waiting { .. } | S::Checks { .. } => {
            w.time_span(value.checked_div(60_000).expect("minute is nonzero"))
        }
        S::Model { .. }
        | S::Window { .. }
        | S::Output { .. }
        | S::Turns { .. }
        | S::Spend
        | S::Message
        | S::Inbox
        | S::Unacknowledged => crate::words::grouped(value),
    }
}

fn hello_refusal(w: &Words, why: c::HelloRefusal) -> Box<[u8]> {
    match why {
        c::HelloRefusal::JobKinds => w.say(W::HelloJobKinds, &[]),
        c::HelloRefusal::ReservedChannel => w.say(W::HelloReservedChannel, &[]),
        c::HelloRefusal::Position { worker } => w.say(W::HelloPosition, &[Hole::Number(u64::from(worker))]),
        c::HelloRefusal::Workers { limit } => {
            sentences(w, w.say(W::HelloWorkers, &[]), w.say(W::HelloLimit, &[Hole::Number(u64::from(limit))]))
        }
        c::HelloRefusal::StopBound { grace } => {
            sentences(w, w.say(W::HelloStopBound, &[]), w.say(W::HelloAllowedGrace, &[Hole::Span(grace)]))
        }
        c::HelloRefusal::Agent { why } => agent_shortfall(w, why),
    }
}
fn agent_shortfall(w: &Words, why: c::AgentShortfall) -> Box<[u8]> {
    use c::AgentShortfall as S;
    let word = match why {
        S::Model { .. } => W::AgentModel,
        S::Window { .. } => W::AgentWindow,
        S::Output { .. } => W::AgentOutput,
        S::Turns { .. } => W::AgentTurns,
        S::Spend => W::AgentSpend,
        S::Time { .. } => W::AgentTime,
        S::Waiting { .. } => W::AgentWaiting,
        S::Checks { .. } => W::AgentChecks,
        S::Message => W::AgentMessage,
        S::Inbox => W::AgentInbox,
        S::Unacknowledged => W::AgentUnacknowledged,
    };
    match why {
        S::Checks { charter } => w.say(word, &[Hole::Number(u64::from(charter))]),
        S::Model { charter, model } | S::Window { charter, model } | S::Output { charter, model } => sentences(
            w,
            w.say(word, &[Hole::Number(u64::from(charter))]),
            w.say(W::AgentModelPosition, &[Hole::Number(u64::from(model))]),
        ),
        S::Turns { .. } | S::Spend | S::Time { .. } | S::Waiting { .. } | S::Message | S::Inbox | S::Unacknowledged => {
            w.say(word, &[])
        }
    }
}
