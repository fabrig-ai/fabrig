//! Authenticated person-task standing and task handoff
//! (domain/engine.md, section 4.4; domain/people.md, section 5).

use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::ReplyTo;

use crate::{Core, PersonTaskRoute};

#[derive(Clone)]
pub(crate) enum Action {
    Take { task: u64 },
    HandBack { task: u64 },
    Answer { task: u64, revision: u64, result: people::PersonResult },
}
impl Action {
    pub(crate) fn task(&self) -> u64 {
        match self {
            Self::Take { task } | Self::HandBack { task } | Self::Answer { task, .. } => *task,
        }
    }
    fn act(&self) -> crate::acts::Act {
        match self {
            Self::Take { .. } => crate::acts::Act::Take,
            Self::HandBack { .. } => crate::acts::Act::HandBack,
            Self::Answer { .. } => crate::acts::Act::Answer,
        }
    }
}

fn approval_matches(contract: &tasks::Contract, result: &people::PersonResult) -> bool {
    match result {
        people::PersonResult::Approved { state, .. } => match contract {
            tasks::Contract::Approval { state: expected, .. } => state == expected,
            tasks::Contract::Report { .. } | tasks::Contract::Verdict { .. } | tasks::Contract::Change { .. } => true,
        },
        people::PersonResult::Report { .. }
        | people::PersonResult::Verdict { .. }
        | people::PersonResult::Failure { .. } => true,
    }
}

fn person_result(result: people::PersonResult) -> tasks::TaskResult {
    match result {
        people::PersonResult::Approved { state, words } => tasks::TaskResult::Approved { state, words },
        people::PersonResult::Report { words } => tasks::TaskResult::Report { words },
        people::PersonResult::Verdict { code, words } => {
            tasks::TaskResult::Verdict { code, words, fields: Box::new([]) }
        }
        people::PersonResult::Failure { reason } => tasks::TaskResult::Failure { reason },
    }
}

impl Core {
    /// Recheck the stored owner and current address before answering an act on an ended task.
    pub(crate) fn person_task_ended(
        &self,
        party: crate::routing::party::Party,
        task: u64,
        row: Option<Box<tasks::TaskRecord>>,
        busy: bool,
    ) -> people::Outcome {
        let crate::routing::party::Party { person, project, .. } = party;
        if busy {
            return people::Outcome::Again(people::Again::Busy);
        }
        let Some(row) = row else { return people::Outcome::Refused(people::Refusal::Unknown) };
        let role = self.people.role(person, project);
        let allowed = row.number == task
            && task != 0
            && task <= self.counters.deployment().tasks
            && row.project == project
            && role.is_some()
            && match row.executor {
                tasks::Executor::Person(tasks::PersonAddress::Person(address)) => address == person,
                tasks::Executor::Person(tasks::PersonAddress::Role(address)) => {
                    role.expect("membership").number() == address
                }
                tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => false,
            };
        if !allowed {
            return people::Outcome::Refused(people::Refusal::Unknown);
        }
        let ending = match &row.phase {
            tasks::Phase::Ended(ending) => ending,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                return people::Outcome::Refused(people::Refusal::Unknown);
            }
        };
        let refusal = match row.person_answer {
            Some(answer) if answer.by != 0 && answer.by <= self.counters.deployment().people => {
                people::Refusal::Ended { how: people::Ending::Decided { by: answer.by, at: answer.at } }
            }
            Some(_) => people::Refusal::Unknown,
            None => crate::routing::refusals::ended(ending),
        };
        people::Outcome::Refused(refusal)
    }

    pub(crate) fn person_task(
        &mut self,
        party: crate::routing::party::Party,
        action: Action,
    ) -> Result<tasks::Event, people::Refusal> {
        let crate::routing::party::Party { request, person, project, .. } = party;
        self.start_input();
        let task = action.task();
        if self.tasks.task(task).ok_or(people::Refusal::Unknown)?.project != project {
            return Err(people::Refusal::Unknown);
        }
        let act = action.act();
        let revision = match &action {
            Action::Answer { revision, .. } => Some(*revision),
            Action::Take { .. } | Action::HandBack { .. } => None,
        };
        match crate::acts::person(self, person, act, task, revision) {
            Ok(()) => {}
            // The task owner supplies its exact capacity observation.
            Err(people::Refusal::Limit { which: people::LimitKind::LiveDelegates })
                if act == crate::acts::Act::Answer => {}
            Err(why) => return Err(why),
        }
        let (route, event) = match action {
            Action::Take { .. } => (
                PersonTaskRoute::Take(task),
                tasks::Event::TakePerson { reply_to: ReplyTo::new(request), task, person },
            ),
            Action::HandBack { .. } => (
                PersonTaskRoute::HandBack(task),
                tasks::Event::HandBackPerson { reply_to: ReplyTo::new(request), task, person },
            ),
            Action::Answer { result, .. } => {
                let row = self.tasks.task(task).expect("person act admission checked");
                if !approval_matches(&row.contract, &result) {
                    return Err(people::Refusal::Moved {
                        now: people::Moved::Person { revision: row.revisions.person },
                    });
                }
                (
                    PersonTaskRoute::Answer(task),
                    tasks::Event::AnswerPerson {
                        reply_to: ReplyTo::new(request),
                        task,
                        person,
                        result: person_result(result),
                    },
                )
            }
        };
        let request = self.fly_person_task(request, route);
        Ok(crate::flight::bind(event, request))
    }
}
