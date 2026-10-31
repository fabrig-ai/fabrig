//! Person contracts shown on an inbox card beside their Answer confirmation.
use super::*;

/// Person verdict and report contracts with their editable forms.
#[derive(Clone, Copy)]
pub enum Form {
    OwnerChoice,
    Choice,
    OwnerReport,
    Words,
}

impl Form {
    /// The choice's public caption, or no choice for a words contract.
    #[must_use]
    pub fn choice(self) -> Option<&'static [u8]> {
        match self {
            Self::OwnerChoice | Self::Choice => Some(b"Choice code: 2"),
            Self::OwnerReport | Self::Words => None,
        }
    }
    /// The complete request the person's confirmation will submit.
    #[must_use]
    pub fn ask(self) -> Ask {
        let answer = match self {
            Self::OwnerChoice | Self::Choice => PersonAnswer::Choice { option: 2, reason: text(b"second report") },
            Self::OwnerReport | Self::Words => PersonAnswer::Words(text(b"my complete answer")),
        };
        Ask::Answer { project: 1, on: AnswerOn::Person { task: 9, revision: Revision { value: 17 }, answer } }
    }
}

fn result(number: u64, words: &[u8]) -> Result {
    let mut task = super::task();
    task.chip.task = number;
    Result {
        task: task.chip,
        ending: Ending::Done,
        value: Some(ResultValue::Report(DocumentText::complete(text(words)))),
        at: Some(Wall::EPOCH),
        acts: list([]),
        details: None,
    }
}

fn chip(_form: Form) -> Chip {
    let mut chip = super::task().chip;
    chip.executor = Executor::Party { party: 7 };
    chip.phase = Phase::Active;
    chip.details = Box::new(ChipPhase::Owner {
        phase: OwnerChipPhase::Person { address: PersonAddress::Party(7), taken_by: Some(7), since: Some(Wall::EPOCH) },
        details: Box::new(OwnerChipDetails {
            dependencies: skein_lib::List::with_capacity(0),
            project: 1,
            executor: OwnerExecutor::Person(PersonAddress::Party(7)),
            phase_since: Wall::EPOCH,
            run_since: None,
            attempt: None,
            retry: None,
            wake: OwnerWakePolicy {
                words: WakeRule::Never,
                notices: WakeRule::Never,
                news: WakeRule::Never,
                results: ResultsWake::Never,
                questions: false,
                answers: true,
                timers: false,
            },
            person_wait: None,
            hold: None,
            budget: OwnerBudget {
                numbers: OwnerNumbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
                left: 0,
                funder: Funder::Pool { project: 1, person: 7, period: 1 },
                deadline: None,
            },
            activity: 0,
            resources: list([]),
            resources_cut: false,
            resources_observed: false,
            procedure: None,
            live_delegates: 0,
            closing_connectors: None,
            ending: None,
            ended_at: None,
            observation: None,
        }),
    });
    chip
}

fn document(form: Form) -> ReadValue {
    let chip = chip(form);
    let acts = list([Offered { effect: None, act: ActKind::Answer, admission: Admission::Allowed }]);
    let first = result(101, b"first whole report");
    let second = result(102, b"second whole report");
    ReadValue::OwnerPersonTask(Box::new(OwnerPersonTask {
        task: chip,
        revision: Revision { value: 17 },
        requester: OwnerRequester::Person(7),
        spec: super::specification(text(b"compare both reports")),
        address: PersonAddress::Party(7),
        taken_by: Some(7),
        since: Some(Wall::EPOCH),
        stall: Some(Duration::from_secs(120)),
        deadline: None,
        contract: match form {
            Form::OwnerChoice | Form::Choice => TaskContractDocument::Verdict {
                choices: list([
                    VerdictChoice { code: 1, words: 32, followups: 0, fields: list([]) },
                    VerdictChoice { code: 2, words: 32, followups: 0, fields: list([]) },
                ]),
                cut: false,
            },
            Form::OwnerReport | Form::Words => TaskContractDocument::Report { words: 32 },
        },
        results: list([
            OwnerShownResult { task: OwnerTaskLink::Live(first.task.clone()), result: Some(Box::new(first)) },
            OwnerShownResult { task: OwnerTaskLink::Live(second.task.clone()), result: Some(Box::new(second)) },
        ]),
        cut: false,
        acts,
    }))
}

/// A whole two-input person task with one answer act and a durably scripted reply.
#[must_use]
pub fn world(seed: u64, form: Form) -> World<Test, Scripted> {
    let mut world = super::world(seed);
    for change in [
        jig_client_scripted_engine::Change::Object { project: 1, key: ObjectKey::PersonTask(9), value: document(form) },
        jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
            party: 7,
            read: Read::Inbox { group: InboxReadGroup::Needs, project: None, kind: None, goal: None, after: None },
            rows: jig_client_scripted_engine::Rows::Inbox(vec![crate::stories::inbox_entry(
                ObjectKey::PersonTask(9),
                1,
                InboxGroup::PersonTasks,
                1,
                Wall::EPOCH,
                None,
            )]),
        }),
    ] {
        world.front().engine.act(skein_lib::Time::ZERO, Wall::EPOCH, change).expect("whole person facts fit");
    }
    world
        .front()
        .engine
        .script(jig_client_scripted_engine::Rule {
            party: Some(7),
            ask: form.ask(),
            answer: Answer::Done(Outcome::Owner(OwnerOutcome::PersonAnswered { task: 9 })),
            changes: vec![],
        })
        .expect("whole confirmed answer fits");
    world
}

use jig_client::contract::AnswerOn;
