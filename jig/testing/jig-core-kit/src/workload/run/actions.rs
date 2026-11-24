//! Each injected act uses the authenticated people's route.
use super::{
    Driver, Kind, Moment, Party, Struck, Sweep, Token, Wall, World, core, hold_room, open, people, requests, row, tasks,
};

pub(super) struct Receipt {
    kind: Kind,
    pub(super) target: usize,
    key: u8,
    duplicate: bool,
}

impl Driver {
    pub(super) fn act(&mut self, world: &mut World, sweep: &Sweep) -> Result<(), String> {
        self.check_receipts(world)?;
        if !self.receipts.is_empty() {
            return Ok(());
        }
        for index in 0..self.acts.len() {
            if self.acts[index].fired {
                continue;
            }
            let act = self.acts[index].act.clone();
            if self.script_target.is_some_and(|target| self.related(target, act.target)) {
                continue;
            }
            let Some(number) = self.progress[act.target].number else { continue };
            let Some(current) = row(world, number) else { continue };
            if act.kind == people::HOLDS_A_TASK && !hold_room(world) {
                continue;
            }
            let parent_ended = self.workload.tasks[act.target]
                .parent
                .and_then(|parent| self.progress[parent].number)
                .and_then(|parent| row(world, parent))
                .is_some_and(|row| row.last_answer.is_some());
            let allowed = holds(act.moment, act.kind, current, parent_ended);
            if !allowed {
                continue;
            }
            let since = *self.acts[index].seen.get_or_insert(world.iterations());
            if world.iterations().saturating_sub(since) < act.after {
                continue;
            }
            let mut holding = Vec::new();
            if act.kind == open::AMENDS_A_PASSED_DEADLINE
                && current.requester == tasks::Party::Person(self.persons[act.party.index()])
            {
                holding.push("the amending party started the goal");
            }
            if act.kind == open::WIDENS_GOAL_AUTHORITY && act.party == Party::Owner {
                holding.push("the widening party is the project's owner");
            }
            if let Some(exclusion) = sweep.leaves_out(act.kind, &holding) {
                self.ledger.leave_out(exclusion.scenario, act.kind, world.clocks().0);
                self.acts[index].fired = true;
                continue;
            }
            let request = request(&act, current, world)?;
            self.acts[index].fired = true;
            let now = world.clocks().0;
            let mut affected = Some(act.target);
            while let Some(target) = affected {
                self.progress[target].last = (now, world.iterations());
                affected = self.workload.tasks[target].parent;
            }
            let entry = self.ledger.inject(
                act.kind,
                now,
                Struck { request: Some(self.workload.tasks[act.target].name.clone()), ..Struck::default() },
            );
            self.ledger.stop(entry, now)?;
            if let Some(request) = request {
                let key = self.ask(world, act.party, request.clone())?;
                self.receipts.push(Receipt { kind: act.kind, target: act.target, key, duplicate: false });
                if act.kind == people::ANSWERS_TWICE {
                    let key = self.ask(world, act.party, request)?;
                    self.receipts.push(Receipt { kind: act.kind, target: act.target, key, duplicate: true });
                }
            }
            // One act per iteration leaves its revision and committed outcome observable.
            return Ok(());
        }
        Ok(())
    }

    pub(super) fn check_outcomes(&self, world: &World) -> Result<(), String> {
        for act in self
            .acts
            .iter()
            .filter(|act| act.fired && [people::ANSWERS_TWICE, people::ANSWERS_AN_ENDED_RUN].contains(&act.act.kind))
        {
            let number = self.progress[act.act.target].number.expect("answered question task");
            let reports = world.store.rows.values().filter(|record| match record {
                super::root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) =>
                    matches!(&line.line, tasks::Line::Heard(message) if message.from == tasks::Party::Task(number) && matches!(message.kind, tasks::MessageKind::Result(_))),
                _ => false,
            }).count();
            if reports != 1 {
                return Err(format!(
                    "seed {}: task {} ({number}), {:?} at {:?}, iteration {}: {reports} reports reached its requester, expected one",
                    self.seed,
                    self.workload.tasks[act.act.target].name,
                    act.act.kind,
                    world.clocks().0,
                    world.iterations()
                ));
            }
        }
        Ok(())
    }

    fn check_receipts(&mut self, world: &World) -> Result<(), String> {
        let mut pending = Vec::new();
        for receipt in std::mem::take(&mut self.receipts) {
            let Some((_, reply)) =
                world.people_replies.iter().find(|(token, _)| *token == Token::new(8200 + u64::from(receipt.key)))
            else {
                pending.push(receipt);
                continue;
            };
            let requests::Reply::Outcome(outcome) = reply else {
                return Err(format!("act reply: {reply:?}"));
            };
            let accepted = if receipt.duplicate {
                matches!(outcome, requests::Outcome::Refused(requests::Refusal::Ended { .. }))
            } else if receipt.kind == people::WORDS_AFTER_ITS_RUN_ENDED {
                matches!(
                    outcome,
                    requests::Outcome::Said { .. } | requests::Outcome::Refused(requests::Refusal::Unknown)
                )
            } else if receipt.kind == people::CANCELS_A_TASK {
                matches!(outcome, requests::Outcome::Cancelled { .. })
            } else if receipt.kind == people::HOLDS_A_TASK {
                matches!(outcome, requests::Outcome::Stopped { .. })
            } else if receipt.kind == people::RELEASES_A_TASK {
                matches!(outcome, requests::Outcome::Released { .. })
            } else if receipt.kind == open::AMENDS_A_PASSED_DEADLINE || receipt.kind == open::WIDENS_GOAL_AUTHORITY {
                matches!(outcome, requests::Outcome::Amended { .. })
            } else {
                matches!(outcome, requests::Outcome::PersonAnswered { .. })
            };
            if !accepted {
                return Err(format!(
                    "seed {}: task {} ({:?}), {:?} at {:?}, iteration {}: {:?}",
                    self.seed,
                    self.workload.tasks[receipt.target].name,
                    self.progress[receipt.target].number,
                    receipt.kind,
                    world.clocks().0,
                    world.iterations(),
                    outcome
                ));
            }
        }
        self.receipts = pending;
        Ok(())
    }
}

fn request(act: &super::Act, current: &tasks::TaskRecord, world: &World) -> Result<Option<requests::Ask>, String> {
    let number = current.number;
    let request = if act.kind == people::CANCELS_A_TASK {
        Some(requests::Ask::Cancel {
            project: 1,
            task: number,
            reason: b"workload cancel".as_slice().into(),
            grace: None,
        })
    } else if act.kind == people::HOLDS_A_TASK {
        Some(requests::Ask::Stop { project: 1, task: number })
    } else if act.kind == people::RELEASES_A_TASK {
        Some(requests::Ask::Release { project: 1, task: number, revision: current.revisions.hold, again: None })
    } else if act.kind == people::NEVER_ANSWERS {
        None
    } else if [people::ANSWERS_LATE, people::ANSWERS_TWICE, people::ANSWERS_AN_ENDED_RUN].contains(&act.kind) {
        Some(requests::Ask::AnswerPerson {
            project: 1,
            task: number,
            revision: current.revisions.person,
            result: requests::PersonResult::Report { words: b"answer".as_slice().into() },
        })
    } else if act.kind == people::WORDS_AFTER_ITS_RUN_ENDED {
        Some(requests::Ask::Say { project: 1, task: number, words: b"after the run".as_slice().into() })
    } else if act.kind == open::AMENDS_A_PASSED_DEADLINE || act.kind == open::WIDENS_GOAL_AUTHORITY {
        let mut amendment = jig_test_profile::fixtures::person_unchanged(b"more time");
        let mut authority =
            core::authority_to_people(&core::tasks_authority(&current.authority)).expect("bounded task authority");
        authority.deadline = Some(Wall::from_nanos(world.clocks().1.as_nanos() + 600_000_000_000));
        amendment.authority = Some(authority);
        Some(requests::Ask::Amend {
            project: 1,
            task: number,
            revision: current.revisions.spec,
            amendment,
            release: (act.kind == open::AMENDS_A_PASSED_DEADLINE).then_some(current.revisions.hold),
        })
    } else {
        return Err(format!("unimplemented workload act {:?}", act.kind));
    };
    Ok(request)
}

fn holds(moment: Moment, kind: Kind, current: &tasks::TaskRecord, parent_ended: bool) -> bool {
    let question = current.person_wait.is_some()
        && current.person_answer.is_none()
        && matches!(current.phase, tasks::Phase::Active(_));
    match moment {
        Moment::Exists => !matches!(current.phase, tasks::Phase::Closing(_) | tasks::Phase::Ended(_)),
        Moment::QuestionOpen => question,
        Moment::Held => {
            current.placed.is_some() || matches!(current.phase, tasks::Phase::Held { why: tasks::Hold::Stopped, .. })
        }
        Moment::DeadlineHeld => matches!(current.phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. }),
        Moment::RunEnded if kind == people::ANSWERS_AN_ENDED_RUN => parent_ended && question,
        Moment::RunEnded => current.last_answer.is_some() || matches!(current.phase, tasks::Phase::Ended(_)),
    }
}

#[cfg(test)]
mod tests {
    use super::{Moment, holds, people, tasks};
    use crate::{
        fixtures,
        world::{Setup, World},
    };
    use skein_lib::{Duration, Wall};

    #[test]
    fn workload_moments_follow_the_committed_task_state() {
        let world = World::new(Setup { seed: 27, ..Setup::default() });
        let mut row = fixtures::live_row(&world, fixtures::run(&world).0).clone();
        assert!(holds(Moment::Exists, people::HOLDS_A_TASK, &row, false));
        assert!(!holds(Moment::Held, people::RELEASES_A_TASK, &row, false));
        assert!(!holds(Moment::QuestionOpen, people::ANSWERS_LATE, &row, false));
        assert!(!holds(Moment::RunEnded, people::ANSWERS_AN_ENDED_RUN, &row, true));
        row.person_wait = Some(tasks::PersonWait {
            since: Wall::EPOCH,
            stall: Duration::from_secs(60),
            deadline: Wall::from_nanos(60_000_000_000),
            notice: None,
        });
        assert!(holds(Moment::QuestionOpen, people::ANSWERS_LATE, &row, false));
        assert!(!holds(Moment::RunEnded, people::ANSWERS_AN_ENDED_RUN, &row, false));
        assert!(holds(Moment::RunEnded, people::ANSWERS_AN_ENDED_RUN, &row, true));
        row.phase = tasks::Phase::Held {
            was: tasks::Was::Active(tasks::Active::Idle),
            why: tasks::Hold::Stopped,
            source: None,
        };
        assert!(holds(Moment::Held, people::RELEASES_A_TASK, &row, false));
        assert!(!holds(Moment::QuestionOpen, people::ANSWERS_LATE, &row, false));
        assert!(!holds(Moment::DeadlineHeld, crate::open::AMENDS_A_PASSED_DEADLINE, &row, false));
        row.phase = tasks::Phase::Held {
            was: tasks::Was::Active(tasks::Active::Idle),
            why: tasks::Hold::Deadline,
            source: None,
        };
        assert!(holds(Moment::DeadlineHeld, crate::open::AMENDS_A_PASSED_DEADLINE, &row, false));
        row.last_answer = Some(row.attempt);
        assert!(holds(Moment::RunEnded, people::WORDS_AFTER_ITS_RUN_ENDED, &row, false));
    }
}
