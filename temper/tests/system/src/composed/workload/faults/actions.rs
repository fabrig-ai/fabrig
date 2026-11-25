//! The composed party route, with logical moments read by the engine world's watch.
use super::{Act, Faults, Struck, World};
use crate::composed::{people, tasks};
use temper_engine_domain_world::acts;

impl World {
    pub(super) fn workload_actions_tick(&mut self, f: &mut Faults) {
        for action in f.watch.due(&self.store, self.workload_names(), self.iteration) {
            eprintln!("act {action:?} at {:?}", self.env.now);
            f.actions.retain(|pending| pending != &action);
            let target = self.workload_names().task(&action.target).expect("drawn member");
            let ask = match action.act {
                Act::HoldMember => {
                    people::Ask::Stop { project: 1, task: target, reason: Box::from(b"Parking the task".as_slice()) }
                }
                Act::ReleaseMember => {
                    let row = self.root.tasks().task(target).expect("held member");
                    assert!(matches!(row.phase, tasks::Phase::Held { .. }), "release follows hold");
                    people::Ask::Release { project: 1, task: target, revision: row.revisions.hold, again: None }
                }
                _ => panic!("only the train's hold and release are drawn"),
            };
            let injection = f.ledger.inject(
                acts::kind(action.act),
                self.env.now,
                Struck { request: Some(action.target), ..Struck::default() },
            );
            f.ledger.stop(injection, self.env.now).expect("an act is an instant");
            f.stopped(self.env.now);
            let sign_in = match action.party {
                temper_workload::draw::Party::First => self.party.sign_in,
                temper_workload::draw::Party::Second => self.workload.as_ref().expect("workload").other.sign_in,
            }
            .expect("drawn party signed in");
            let mut key = [0; 16];
            key[..8].copy_from_slice(&self.workload.as_ref().expect("workload").next_reply.to_le_bytes());
            self.workload.as_mut().expect("workload").next_reply += 1;
            let reply = self.notes_act_as(sign_in, ask, key);
            assert!(
                !matches!(reply, people::Reply::Outcome(people::Outcome::Refused(_)) | people::Reply::Refused(_)),
                "drawn act refused: {reply:?}"
            );
        }
    }
}
