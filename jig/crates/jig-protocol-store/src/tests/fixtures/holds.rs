//! Full values for the store's held declarations (protocol/store.md, 5.1).
use jig_domain_people as people;
use jig_domain_tasks as tasks;

pub fn member() -> people::Member {
    let mut member = super::people_member();
    let spec = super::people_amendment().spec.expect("full amendment");
    member.parameters = spec.parameters;
    member.inputs = spec.inputs;
    member.named = member.authority.grants.clone();
    member.dependencies = Box::new([people::Dependency::Batch(0), people::Dependency::Existing(2)]);
    member.contract = people::Contract::Verdict {
        choices: Box::new([people::Verdict {
            code: 7,
            words: 16,
            followups: 3,
            fields: Box::new([people::FieldRule { name: b"field".as_slice().into(), max: 11 }]),
        }]),
    };
    member
}

pub fn result_fields() -> Box<[tasks::ResultField]> {
    Box::new([tasks::ResultField { name: b"field".as_slice().into(), value: b"value".as_slice().into() }])
}

pub fn task() -> tasks::TaskRecord {
    let mut task = super::task_record();
    task.contract = super::task_contract();
    task.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Verdict {
        code: 7,
        fields: result_fields(),
        words: b"done".as_slice().into(),
    }));
    task.inbox[0].kind = tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 7, fields: result_fields() });
    task
}

pub fn edit_note() -> people::Ask {
    people::Ask::EditNote {
        project: 1,
        name: 2,
        scope: Box::new(people::NoteScope::Resources { connector: 1, pattern: super::people_pattern() }),
        change: Box::new(people::NoteChange::Correct {
            description: Box::new([2]),
            body: Box::new([0, 255]),
            references: Box::new([3]),
            recalled: 4,
        }),
    }
}

pub fn adopt() -> people::Ask {
    people::Ask::Adopt {
        project: 1,
        adoption: people::Adoption {
            resource: people::ResourceName { connector: 1, path: Box::new([Box::new([2])]) },
            role: people::ResourceRole::Owned,
            options: Box::new([Box::new([3])]),
        },
    }
}
