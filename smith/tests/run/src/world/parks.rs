//! Parks drawn against the noisy run world. The fake session owns opaque bodies;
//! the run domain owns their numbering and grouping, its drain, and its answer.
//! Contracts: smith/docs/design/parks.md, 2 and 3; smith/docs/design/domain/turn-parts.md, 2 to 4.

use skein_world::{
    faults::{FaultLedger, Kind, Struck},
    streams::{Stream, Streams},
};
use smith_faults::parks as kinds;

use super::{Delivery, Duration, Time, Token, World, run};
use crate::partner::{ParkTurn, Partner};
use run::Request as R;

const GRACE: Duration = Duration::from_millis(250);
const FAST: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scenario {
    Wait,
    Cut,
    Answered,
    Unanswered,
    Child,
}

impl Scenario {
    fn kind(self) -> Kind {
        match self {
            Self::Wait => kinds::WITHHELD_WAIT,
            Self::Cut => kinds::CUT_TURN,
            Self::Answered => kinds::HOST_CALL_ANSWERED,
            Self::Unanswered => kinds::HOST_CALL_UNANSWERED,
            Self::Child => kinds::SUB_AGENT_KEPT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Body {
    Answer,
    Result { position: u32, host_answered: bool },
}

#[derive(Debug, PartialEq, Eq)]
struct Part {
    number: u32,
    turn: u32,
    thread: u32,
    call: u32,
    body: Body,
}

pub(super) struct Parks {
    draws: Vec<Park>,
    ledger: FaultLedger,
}

impl Parks {
    fn get(&self, host_run: Token) -> &Park {
        &self.draws[usize::try_from(host_run.raw() - 1).expect("bounded host jobs")]
    }

    fn get_mut(&mut self, host_run: Token) -> &mut Park {
        &mut self.draws[usize::try_from(host_run.raw() - 1).expect("bounded host jobs")]
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Drain {
    #[default]
    None,
    Requested,
    Running {
        at: Time,
        cut: Option<Time>,
    },
}

struct Park {
    host_run: Token,
    scenario: Scenario,
    eligible: bool,
    child: Option<(Token, u32)>,
    answer_part: Option<Token>,
    held: Option<run::RelayName>,
    waiting: Option<Time>,
    drain: Drain,
    interrupted: bool,
    answered: Option<Time>,
    last_turn: Option<Token>,
    parts: Vec<(Time, Part)>,
}

impl Park {
    fn check(&self, holds: bool, what: &str) {
        assert!(holds, "run {} / {}: {what}; parts: {:?}", self.host_run.raw(), self.scenario.kind().name, self.parts);
    }

    fn check_observed(&self, answer: &run::Answer, turns: u32) {
        let parts = |expected: &[Part]| self.parts.iter().map(|(_, part)| part).eq(expected);
        let main = Part { number: 1, turn: 1, thread: 0, call: 0, body: Body::Answer };
        if self.scenario == Scenario::Wait {
            self.check(matches!(answer, run::Answer::Parked { .. }), "the withheld wait did not park");
            self.check(
                self.last_turn == Some(Partner::WITHHELD_WAIT_RECORD) && turns == 1,
                "last turn was not told with its wait withheld",
            );
            self.check(parts(&[main]), "the withheld wait told a result or lost its answer part");
            return;
        }

        self.check(
            matches!(answer, run::Answer::Failed { failure: run::RunFailure::Cancelled, .. }),
            "the cut did not answer cancelled",
        );
        self.check(turns == 0, "told a whole turn for the cut");
        let Drain::Running { at, cut: Some(cut) } = self.drain else {
            self.check(false, "did not observe the cut at grace expiry");
            return;
        };
        let last = self.parts.last().map_or(Time::ZERO, |(told, _)| *told);
        let grace_start = if self.scenario == Scenario::Child { last.max(at) } else { at };
        self.check(
            cut == grace_start.saturating_add(GRACE) && last <= cut,
            "the cut did not keep the parts through the grace",
        );
        match self.scenario {
            Scenario::Cut | Scenario::Answered => {
                let result = Part { number: 2, body: Body::Result { position: 0, host_answered: true }, ..main };
                self.check(
                    parts(&[main, result]),
                    "expected main's answer followed by only the answered host call's result",
                );
                if self.scenario == Scenario::Cut {
                    self.check(at < last, "the host call did not answer during the grace");
                } else {
                    self.check(last <= at, "the host call was not already answered at the drain");
                }
            }
            Scenario::Unanswered => {
                self.check(parts(&[main]), "an unanswered host call told a result or lost its answer part");
            }
            Scenario::Child => {
                let child = Part { number: 2, thread: 1, call: 1, ..main };
                self.check(parts(&[main, child]), "child's answer was not kept on its thread at the call's position");
            }
            Scenario::Wait => unreachable!("checked separately"),
        }
    }
}

impl World {
    /// Draw the noisy world's parks in the host's numbered order, before eligibility or arrival order is known.
    pub fn draw_parks(&mut self) {
        assert_eq!(self.stats.starts, 0, "draw before the host starts runs");
        let mut rng = Streams::new(self.settings.seed).rng(Stream::Faults, "parks");
        let choices = [Scenario::Wait, Scenario::Cut, Scenario::Answered, Scenario::Unanswered, Scenario::Child];
        let draws = (1..=self.settings.host.jobs)
            .map(|name| Park {
                host_run: Token::new(u64::from(name)),
                scenario: choices[usize::try_from(rng.below(5)).expect("five park states")],
                eligible: false,
                child: None,
                answer_part: None,
                held: None,
                waiting: None,
                drain: Drain::None,
                interrupted: false,
                answered: None,
                last_turn: None,
                parts: Vec::new(),
            })
            .collect();
        self.parks = Some(Parks { draws, ledger: FaultLedger::new() });
        // This fixture checks typed boundary observations, not a text replay.
        self.trace = None;
    }

    /// The case's observed parks and its draws without an eligible target.
    #[must_use]
    pub fn park_ledger(&self) -> &FaultLedger {
        &self.parks.as_ref().expect("draw_parks enabled the park fixture").ledger
    }

    pub(super) fn park_start(&mut self, host_run: Token, charter: &run::Charter) -> bool {
        let Some(parks) = self.parks.as_mut() else { return false };
        let park = parks.get_mut(host_run);
        park.eligible = match park.scenario {
            Scenario::Wait => charter.grants.wait && self.carried_messages.is_empty(),
            Scenario::Cut | Scenario::Answered => !charter.grants.host_tools.is_empty() && self.settings.run.calls >= 2,
            Scenario::Unanswered => !charter.grants.host_tools.is_empty(),
            Scenario::Child => charter.grants.agents && self.settings.run.run_conversations >= 2,
        };
        park.eligible
    }

    pub(super) fn park_turn(&mut self, conversation: Token, opening: &run::Opening) -> Option<ParkTurn> {
        let parks = self.parks.as_mut()?;
        let run = self.run_of_conversation[&conversation];
        let view = &self.views[&run];
        let park = parks.get_mut(view.host_run);
        if !park.eligible {
            return None;
        }
        if view.main != Some(conversation) {
            park.child = Some((conversation, opening.ordinal));
            return Some(ParkTurn::Child);
        }
        Some(match park.scenario {
            Scenario::Wait => ParkTurn::Wait,
            Scenario::Cut | Scenario::Answered => ParkTurn::HostPair,
            Scenario::Unanswered => ParkTurn::HostSingle,
            Scenario::Child => ParkTurn::Delegate,
        })
    }

    pub(super) fn park_note(&mut self, owner: Token, event: &run::Event) {
        let Some(parks) = self.parks.as_mut() else { return };
        let Some(view) = self.views.get(&owner) else { return };
        let park = parks.get_mut(view.host_run);
        if park.answered.is_some() {
            return;
        }
        match event {
            run::Event::Cancel { .. } => park.interrupted = true,
            run::Event::Drain { .. } => park.drain = Drain::Running { at: self.now, cut: None },
            run::Event::Used { conversation, .. } | run::Event::Ended { conversation, .. } if park.eligible => {
                run::completion_settled(&mut self.run, *conversation, self.now);
                if park.answer_part == Some(*conversation) && matches!(event, run::Event::Used { .. }) {
                    park.answer_part = None;
                    self.park_part(owner, *conversation, Body::Answer);
                }
            }
            _ => {}
        }
    }

    fn park_part(&mut self, owner: Token, conversation: Token, body: Body) {
        let park = self.parks.as_mut().expect("parts enabled by the park fixture").get_mut(self.views[&owner].host_run);
        if let Some((host_run, number, turn, call, _spent)) = run::part(&mut self.run, conversation) {
            park.check(park.answered.is_none(), "told a part after its final answer");
            park.check(host_run == park.host_run, "part names another host run");
            park.check(number as usize == park.parts.len() + 1, "parts are not consecutively numbered");
            let thread = park.child.filter(|(child, _)| *child == conversation).map_or(0, |(_, ordinal)| ordinal);
            park.parts.push((self.now, Part { number, turn, thread, call, body }));
            self.run_stage.push(run::Event::PartKept { run: owner, number });
        }
    }

    fn park_drain(&mut self, run: Token) {
        let park = self.parks.as_mut().expect("the host armed a park").get_mut(self.views[&run].host_run);
        if park.drain == Drain::None && !park.interrupted {
            park.drain = Drain::Requested;
            self.run_stage.push(run::Event::Drain { run, grace: Some(GRACE) });
        }
    }

    pub(super) fn park_completion(&mut self, conversation: Token, owner: Token) {
        let Some(parks) = self.parks.as_ref() else { return };
        let park = parks.get(self.views[&owner].host_run);
        park.check(park.answered.is_none(), "started a completion after its final answer");
        park.check(park.drain == Drain::None, "started a new completion during the grace");
        if park.child.is_some_and(|(child, _)| child == conversation) {
            self.park_drain(owner);
        }
    }

    pub(super) fn park_relay(&mut self, run: Token, relay: run::RelayName, position: u32) -> bool {
        let Some(parks) = self.parks.as_mut() else { return false };
        let park = parks.get_mut(self.views[&run].host_run);
        if !park.eligible {
            return false;
        }
        park.check(park.answered.is_none(), "asked a host call after its final answer");
        match park.scenario {
            Scenario::Cut | Scenario::Answered if position == 0 => {
                let result =
                    run::HostAnswer::new(b"recorded during park".as_slice().into(), false).expect("bounded text");
                let reply = run::HostReply::Answered(result);
                self.schedule(self.now.saturating_add(FAST), Delivery::Host(run::Event::HostReturned { relay, reply }));
            }
            Scenario::Cut | Scenario::Answered | Scenario::Unanswered => {
                park.check(park.held.is_none(), "reasked a held host call");
                park.held = Some(relay);
                if park.scenario != Scenario::Answered {
                    self.park_drain(run);
                }
            }
            Scenario::Wait | Scenario::Child => return false,
        }
        true
    }

    pub(super) fn park_cut_seen(&mut self, owner: Token, conversation: Token) {
        let Some(parks) = self.parks.as_mut() else { return };
        let park = parks.get_mut(self.views[&owner].host_run);
        if let Drain::Running { at, cut: None } = park.drain
            && run::cutting(&self.run, conversation)
        {
            park.drain = Drain::Running { at, cut: Some(self.now) };
        }
    }

    pub(super) fn park_withdraw(&mut self, owner: Token, relay: run::RelayName) {
        if let Some(park) = self
            .parks
            .as_mut()
            .map(|parks| parks.get_mut(self.views[&owner].host_run))
            .filter(|park| park.held == Some(relay))
        {
            park.held = None;
            let reply = run::HostReply::Withdrawn;
            self.schedule(self.now, Delivery::Host(run::Event::HostReturned { relay, reply }));
        }
    }

    pub(super) fn park_return(&mut self, call: Token, result: &run::Returned) -> bool {
        let Some(parks) = self.parks.as_mut() else { return false };
        let ledger = &self.calls[&call];
        let conversation = ledger.conversation;
        let owner = self.run_of_conversation[&conversation];
        let park = parks.get_mut(self.views[&owner].host_run);
        if !park.eligible {
            return false;
        }
        if *result == run::Returned::Withheld {
            return false;
        }
        if run::cutting(&self.run, conversation) {
            // All calls scripted here are delegated; a cut leaves them without a result.
            return true;
        }
        let host_answered = matches!(result, run::Returned::HostAnswered(_));
        let position = ledger.position;
        let drain = park.scenario == Scenario::Answered && host_answered && park.held.is_some();
        self.park_part(owner, conversation, Body::Result { position, host_answered });
        if drain {
            self.park_drain(owner);
        }
        false
    }

    pub(super) fn park_told_turn(&mut self, host_run: Token, record: Token) {
        if let Some(parks) = self.parks.as_mut() {
            let park = parks.get_mut(host_run);
            park.last_turn = Some(record);
            if park.scenario != Scenario::Wait && park.drain == Drain::None {
                // The scripted first turn could close whole if the other call
                // or child was refused. Do not try a new park on a later turn.
                park.eligible = false;
            }
        }
    }

    pub(super) fn park_waiting(&mut self, host_run: Token) {
        if let Some(parks) = self.parks.as_mut() {
            parks.get_mut(host_run).waiting = Some(self.now);
        }
    }

    pub(super) fn park_answer_part(&mut self, conversation: Token) {
        let owner = self.run_of_conversation[&conversation];
        let park =
            self.parks.as_mut().expect("the park fixture emits answer parts").get_mut(self.views[&owner].host_run);
        park.check(park.answer_part.is_none(), "another answer part is pending");
        park.answer_part = Some(conversation);
    }

    pub(super) fn guard_park_request(&self, request: &run::Request, current: Option<Token>) {
        let Some(parks) = self.parks.as_ref() else { return };
        let (owner, what) = match request {
            R::HostCall { host_run, .. } | R::Deliver { host_run, .. } | R::Save { host_run, .. } => {
                (self.run_of_owner.get(host_run).copied(), "asked a host call after its final answer")
            }
            R::Waiting { host_run, .. }
            | R::Turn { host_run, .. }
            | R::Checking { host_run, .. }
            | R::ChecksEnded { host_run }
            | R::MessageRefused { host_run, .. } => {
                (self.run_of_owner.get(host_run).copied(), "told a host record after its final answer")
            }
            R::Answer { .. } | R::Admitted { .. } => return,
            R::CompletionReady { conversation } => (
                self.run_of_conversation.get(conversation).copied(),
                "released completion credit after its final answer",
            ),
            R::Say { peer, .. } | R::Yield { peer } | R::Close { peer } | R::CancelJobs { peer } => (
                self.conversation_of_peer.get(peer).and_then(|c| self.run_of_conversation.get(c)).copied(),
                "asked its session after its final answer",
            ),
            R::Return { call, .. } => (
                self.calls.get(call).and_then(|c| self.run_of_conversation.get(&c.conversation)).copied(),
                "returned a call after its final answer",
            ),
            R::WithdrawHost { relay } => {
                (self.relays.owners.get(&relay.owner).copied(), "withdrew a host call after its final answer")
            }
            R::Open { .. }
            | R::Read { .. }
            | R::Probe { .. }
            | R::ProbeHooks { .. }
            | R::Check { .. }
            | R::Abort { .. } => (current, "requested more work after its final answer"),
        };
        if let Some(view) = owner.and_then(|owner| self.views.get(&owner)) {
            let park = parks.get(view.host_run);
            park.check(park.answered.is_none(), what);
        }
    }
}

pub(super) fn observe_answer(parks: &mut Option<Parks>, host_run: Token, now: Time, answer: &run::Answer, turns: u32) {
    let Some(parks) = parks.as_mut() else { return };
    let park = parks.get_mut(host_run);
    park.answered = Some(now);
    let kind = park.scenario.kind();
    let armed = if park.scenario == Scenario::Wait { park.waiting.is_some() } else { park.drain != Drain::None };
    if !park.eligible || !armed || park.interrupted {
        parks.ledger.ineligible(kind);
        return;
    }
    // The final answer is where the host can see what this activation kept.
    park.check_observed(answer, turns);
    let entry = parks.ledger.inject(
        kind,
        now,
        Struck { request: Some(format!("run {}", host_run.raw())), ..Struck::default() },
    );
    parks.ledger.stop(entry, now).expect("the park is observed at its answer");
}
