//! Parent pipe cuts triggered by independent wire or kernel observations.
//! The parent keeps stderr and its wait right solely to observe settlement.

use super::{HangUp, Parent, World, agent_configuration};
use skein_io::kernel::Op;
use skein_lib::Time;
use smith_channel::{Fact, FactKind, Record, kind};

pub(super) const HELD_ANSWER: &[u8] = b"orphan-answer";

pub(super) struct Loss {
    pub(super) at: Time,
    pub(super) queries: usize,
}

pub(super) fn reached(case: HangUp, parent: &Parent, command_spawned: bool) -> bool {
    match case {
        HangUp::Never | HangUp::BeforeStart | HangUp::MidCall => false,
        HangUp::Command => command_spawned,
        HangUp::Waiting => parent.peer.observed().iter().any(|frame| frame.kind == kind::WAITING),
        HangUp::Live | HangUp::ReadEnd => parent.peer.observed().iter().any(|frame| {
            frame.kind == kind::FACT
                && matches!(
                    Fact::decode(&smith_channel::CEILINGS, &mut skein_lib::Reader::new(&frame.body)),
                    Ok(Fact { kind: FactKind::TextArrived, .. })
                )
        }),
    }
}

impl Parent {
    pub(super) fn lose_host(&mut self, now: Time, queries: usize, output_only: bool) {
        self.loss = Some(Loss { at: now, queries });
        if !output_only {
            assert!(!self.writing && self.pending.is_none(), "the host has written its last word");
            let input = self.input.take().expect("host's writing end");
            self.close(input);
        }
        if let Some(target) = self.reading {
            self.submit(Op::Cancel { target });
        }
    }
}

impl World {
    /// Name the fake provider hold released only after the agent exits.
    pub const ORPHAN_ANSWER: &'static [u8] = HELD_ANSWER;

    /// Close both host ends after text arrives, before the held live piece flushes.
    pub fn hang_up_with_live_text(&mut self) {
        self.hang_up = HangUp::Live;
    }

    /// Close both host ends after the simulated shell and its group start.
    pub fn hang_up_during_command(&mut self) {
        self.hang_up = HangUp::Command;
    }

    /// Close both host ends once the run is waiting for its next message.
    pub fn hang_up_while_waiting(&mut self) {
        self.hang_up = HangUp::Waiting;
    }

    /// Close only the host's reading end before the next live write.
    pub fn close_reading_end(&mut self) {
        self.hang_up = HangUp::ReadEnd;
        self.hold_channel = true;
    }

    /// The link's figures as the agent derives them from this world's configuration.
    #[must_use]
    pub fn link(&self) -> smith_host_domain::Limits {
        let setup = self.launch_setup();
        let mut configuration = agent_configuration(setup);
        if setup.tracing.is_some() {
            crate::tracing::configure_events(&mut configuration, setup.capture);
        }
        smith_agent_profile::host_limits(&crate::profile(), &configuration.limits, &configuration.domain.limits)
            .expect("world link derives")
    }

    /// When the agent's process exited, on the world's clock.
    #[must_use]
    pub fn exited_at(&self) -> Option<Time> {
        self.outcome.as_ref()?;
        self.parent().exited_at
    }

    /// Whether the host still held its writing end when it observed the agent's exit.
    #[must_use]
    pub fn writing_end_at_exit(&self) -> bool {
        self.parent().writing_end_at_exit
    }

    /// When the parent cut the selected channel ends and how many requests it had seen.
    #[must_use]
    pub fn loss(&self) -> (Time, usize) {
        let loss = self.parent().loss.as_ref().expect("story reached its pipe cut");
        (loss.at, loss.queries)
    }
}
