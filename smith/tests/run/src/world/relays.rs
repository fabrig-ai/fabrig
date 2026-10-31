//! Scripted host attempts and retained decisions for immutable call names.
//! Contract: smith/docs/design/domain/run.md, sections 5.2 and 13.

use std::collections::{BTreeMap, BTreeSet};

use skein_lib::Token;
use smith_domain_run as run;

use super::{Delivery, Stats, World};

#[derive(Default)]
pub(super) struct Relays {
    pub(super) active: BTreeSet<(Token, u32)>,
    pub(super) owners: BTreeMap<Token, Token>,
    pub(super) stages: BTreeMap<Token, &'static str>,
    pub(super) decisions: BTreeMap<(Token, u64, u32, u32), run::HostAnswer>,
}

impl Relays {
    pub(super) fn returned(&mut self, relay: run::RelayName, reply: &run::HostReply, stats: &mut Stats) {
        assert!(self.active.remove(&(relay.owner, relay.attempt)), "relay returns once");
        let run = self.owners[&relay.owner];
        match reply {
            run::HostReply::Answered(_) | run::HostReply::TooLarge => {
                stats.relay_answered += 1;
                self.stages.remove(&run);
            }
            run::HostReply::Busy => {
                stats.relay_busy += 1;
                self.stages.insert(run, "relay backoff");
            }
            run::HostReply::Unanswered(_) | run::HostReply::Withdrawn => {
                stats.relay_lost += 1;
                self.stages.insert(run, "relay backoff");
            }
        }
    }
}

impl World {
    pub(super) fn host_call(&mut self, request: run::Request) {
        let run::Request::HostCall { host_run, relay, name, tool, effect, input, deadline } = request else {
            unreachable!("only a host call reaches this handler")
        };
        assert_eq!(tool.as_ref(), b"comment");
        assert_eq!(effect, run::HostEffect::Read);
        assert_eq!(input.bytes(), b"{}");
        assert!(deadline <= self.now.saturating_add(self.settings.run.host_timeout));
        let run = self.run_of_owner[&host_run];
        assert!(self.relays.active.insert((relay.owner, relay.attempt)), "one terminal per host relay");
        self.relays.owners.insert(relay.owner, run);
        self.relays.stages.insert(run, "relay sending");
        let key = (host_run, name.activation, name.completion, name.position);
        let reply = if let Some(answer) = self.relays.decisions.get(&key) {
            run::HostReply::Answered(answer.clone())
        } else if self.rng.chance(self.settings.host.relay_busy) {
            run::HostReply::Busy
        } else {
            let answer = run::HostAnswer::new(b"recorded".as_slice().into(), false).expect("bounded host text");
            self.relays.decisions.insert(key, answer.clone());
            if self.rng.chance(self.settings.host.relay_lost) {
                run::HostReply::Unanswered(run::Unanswered::Lost)
            } else {
                run::HostReply::Answered(answer)
            }
        };
        let delay = self.draw(self.settings.host.relay);
        self.schedule(self.now.saturating_add(delay), Delivery::Host(run::Event::HostReturned { relay, reply }));
    }

    pub(super) fn withdraw_host(&mut self, relay: run::RelayName) {
        assert!(self.relays.active.contains(&(relay.owner, relay.attempt)), "withdraw only a live relay");
        let run = self.relays.owners[&relay.owner];
        self.relays.stages.insert(run, "relay withdrawing");
    }
}
