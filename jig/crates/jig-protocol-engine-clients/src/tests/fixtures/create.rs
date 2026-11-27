use super::*;
use jig_domain_people as p;
pub const PROJECT: u32 = 3;
pub const CHARTER: u32 = 7;
pub const ROLE: u32 = 2;
pub const FIRST_TASK: u64 = 7;
pub const LAST_TASK: u64 = 8;
pub fn create_ask() -> (w::Ask, p::Ask) {
    let (wire, people) = authority_ask();
    let w::Ask::ChangePolicy(wire) = wire else { panic!("policy fixture") };
    let w::PolicyEdit::Role(wire) = wire.edit else { panic!("role fixture") };
    let authority = wire.value.authority;
    let p::Ask::ChangePolicy { change: p::PolicyChange::Role(expected_role), .. } = people else {
        panic!("authority fixture")
    };
    let mut tasks = List::with_capacity(2);
    let mut batch = List::with_capacity(2);
    for (role, deadline) in [(false, None), (true, Some(DEADLINE))] {
        let mut wire = authority.clone();
        wire.deadline = None;
        if let Some(wall) = deadline {
            wire.deadline = Some(wall.as_nanos());
        }
        let mut expected = expected_role.authority.clone();
        expected.deadline = deadline;
        let (task, member) = member(role, wire, expected);
        tasks.push(task).unwrap();
        batch.push(member).unwrap();
    }
    let mut encoded = List::with_capacity(tasks.len());
    for task in &tasks {
        let mut out = Writer::new(
            usize::try_from(task.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
        );
        task.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
        encoded.push(out.finish()).unwrap();
    }
    let bytes = pieces(encoded.as_slice());

    (
        w::Ask::Create(w::Create { project: PROJECT, tasks: bytes }),
        p::Ask::Create { project: PROJECT, batch: batch.into_boxed() },
    )
}
fn member(role: bool, authority: w::Authority, expected: p::Authority) -> (w::NewTask, p::Member) {
    let dependencies = List::from_boxed(Box::new([
        w::TaskRef::Member(w::TaskRefMember { value: 1 }),
        w::TaskRef::Task(w::TaskRefTask { value: u64::MAX }),
    ]));
    let spec = w::Specification {
        asks: false,
        words: b"new words".as_slice().into(),
        parameters: List::from_boxed(Box::new([w::Parameter::Number(w::ParameterNumber { name: 9, value: 12 })])),
        inputs: List::from_boxed(Box::new([77])),
    };
    let field = w::FieldRule { name: b"field".as_slice().into(), max: 13 };
    let choice = w::VerdictChoice { code: 5, words: 16, followups: 2, fields: List::from_boxed(Box::new([field])) };
    let executor = if role {
        w::Executor::Person(w::ExecutorPerson { value: w::PersonAddress::Role(w::PersonAddressRole { value: ROLE }) })
    } else {
        w::Executor::Agent(w::ExecutorAgent { charter: CHARTER })
    };
    let task = w::NewTask {
        symbols: List::with_capacity(0),
        llm: if role { None } else { Some(careful_and_quick()) },
        deadline: w::NewDeadline::Named,
        executor,
        spec,
        contract: w::NewContract::Verdict(w::ContractVerdict { choices: List::from_boxed(Box::new([choice])) }),
        authority: authority.clone(),
        budget: w::NewBudget::AsAllowed(w::BudgetAsAllowed { cap: Some(42) }),
        wake: None,
        dependencies,
        named: authority.grants.as_ref().into(),
        tracked: Some(14),
        label: 15,
        flags: w::TaskFlags {
            ended_by: w::EndedBy::Result,
            conversation: false,
            listed: true,
            retention: w::Retention::ForgottenOnceRead,
        },
    };
    (task, expected_member(role, expected))
}
fn expected_member(role: bool, expected: p::Authority) -> p::Member {
    p::Member {
        symbols: Box::new([]),
        deadline: jig_domain_people::Deadline::Named,
        executor: if role {
            p::TaskExecutor::Person(p::PersonAddress::Role(ROLE))
        } else {
            p::TaskExecutor::Agent { charter: CHARTER }
        },
        words: b"new words".as_slice().into(),
        parameters: Box::new([p::Parameter::Number { name: 9, value: 12 }]),
        inputs: Box::new([77]),
        contract: p::Contract::Verdict {
            choices: Box::new([p::Verdict {
                code: 5,
                words: 16,
                followups: 2,
                fields: Box::new([p::FieldRule { name: b"field".as_slice().into(), max: 13 }]),
            }]),
        },
        authority: expected.clone(),
        budget: p::Budget::AsAllowed { cap: Some(42) },
        wake: None,
        dependencies: Box::new([p::Dependency::Batch(1), p::Dependency::Existing(u64::MAX)]),
        named: expected.grants,
        tracked: Some(14),
        label: 15,
        flags: p::Flags {
            ended_by: p::EndedBy::Result,
            conversation: false,
            listed: true,
            retention: p::Retention::ForgottenOnceRead,
        },
        llm: if role {
            None
        } else {
            Some(p::LlmChoice {
                later: None,
                account: None,
                main: p::ModelEffort { model: 0, effort: Some(1), speed: None },
                subagents: Box::new([p::ModelEffort { model: 1, effort: None, speed: None }]),
            })
        },
    }
}
/// The fixture's careful model at its high effort, its sub-agents the quick one.
pub fn careful_and_quick() -> w::LlmChoice {
    llm_choice(b"careful", Some(b"high"), b"quick")
}
/// A main model and effort, and one sub-agent model at its default effort, by name.
pub fn llm_choice(main: &[u8], effort: Option<&[u8]>, subagent: &[u8]) -> w::LlmChoice {
    let main = w::ModelEffort {
        model: main.into(),
        effort: match effort {
            Some(effort) => Some(effort.into()),
            None => None,
        },
    };
    let subagent = w::ModelEffort { model: subagent.into(), effort: None };
    w::LlmChoice { later: None, account: None, main, subagents: List::from_boxed(Box::new([subagent])) }
}

pub fn create_refusals() -> [(&'static str, p::Refusal, w::Refusal); 5] {
    [
        (
            "two budgets",
            p::Refusal::NotWellFormed { at: 1, why: p::Malformed::AsAllowedTwice },
            w::Refusal::Malformed(w::RefusalMalformed { at: 1, why: w::Malformed::TwoAsAllowed }),
        ),
        (
            "person conversation",
            p::Refusal::NotWellFormed { at: 2, why: p::Malformed::Conversation },
            w::Refusal::Malformed(w::RefusalMalformed { at: 2, why: w::Malformed::ConversationNotAgent }),
        ),
        (
            "cycle",
            p::Refusal::NotWellFormed { at: 3, why: p::Malformed::Cycle },
            w::Refusal::Malformed(w::RefusalMalformed { at: 3, why: w::Malformed::Cycle }),
        ),
        (
            "proposed AsAllowed",
            p::Refusal::NotWellFormed { at: 4, why: p::Malformed::ProposedAsAllowed },
            w::Refusal::Malformed(w::RefusalMalformed { at: 4, why: w::Malformed::ProposedAsAllowed }),
        ),
        (
            "goals",
            p::Refusal::Limit { which: p::LimitKind::Goals },
            w::Refusal::Limit(w::RefusalLimit {
                which: w::Limit::Admission(w::LimitAdmission { which: w::AdmissionLimit::Goals }),
            }),
        ),
    ]
}
pub fn created_outcome() -> w::Outcome {
    w::Outcome::Document(w::OutcomeDocument {
        value: w::DocumentOutcome::Created(w::OutcomeCreated {
            first: FIRST_TASK,
            last: LAST_TASK,
            given: Some(w::CreatedGiven { task: LAST_TASK, amount: 42 }),
        }),
    })
}

// A finite wall distinct from policy fixture timestamps.
const DEADLINE: Wall = Wall::from_nanos(101);
