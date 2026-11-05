use crate::tests::*;
const PARENT: u64 = 1;
const CHILD: u64 = 7;
const CHARTER: u32 = 1;
const ATTEMPT: u64 = 1;
const DELEGATES: [(tasks::Phase, &[u8]); 2] = [
    (
        tasks::Phase::Active(tasks::Active::Running { attempt: ATTEMPT }),
        b"delegate 7: running, spec revision 1\n".as_slice(),
    ),
    (
        tasks::Phase::Held { was: tasks::Was::Waiting, why: tasks::Hold::Budget, source: None },
        b"delegate 7: held, spec revision 1, hold revision 2\n".as_slice(),
    ),
];
#[test]
fn the_briefs_spec_shows_the_contract_only_when_the_result_ends_the_task() {
    let (mut core, _) = make_tree();
    for (ended_by, expected) in [
        (
            tasks::EndedBy::Result,
            b"What does this project hold? Answer in two sentences.\n[Report: at most 32 bytes]\n[This task: 1, in project 1]\n[Requested by person 1]\n"
                .as_slice(),
        ),
        (
            tasks::EndedBy::Requester,
            b"What does this project hold? Answer in two sentences.\n[This task: 1, in project 1]\n[Requested by person 1]\n".as_slice(),
        ),
    ] {
        let mut context = settings::charters::context(PARENT, CHARTER);
        context.ended_by = ended_by;
        context.spec.words = b"What does this project hold? Answer in two sentences.".as_slice().into();
        context.contract = tasks::Contract::Report { words: 32 };
        core.contexts.insert(PARENT, context).expect("context room");
        let text = read_brief_part(
            &core,
            BriefTextLimits { parts: 2, read_bytes: 256, brief_bytes: 256 },
            PARENT,
            BriefPart::Spec,
        )
        .expect("spec section");
        assert_eq!(text.as_ref(), expected, "task ended by {ended_by:?}");
    }
}

#[test]
fn a_briefs_task_section_names_its_number_and_project() {
    let (mut core, _) = make_tree();
    for contract in [
        tasks::Contract::Job { bytes: 32 },
        tasks::Contract::Approval {
            state: Box::new([]),
            part: Part { connector: 1, kind: PartKind::Resource, key: PartKey::Number(1) },
        },
        tasks::Contract::Report { words: 32 },
        tasks::Contract::Verdict {
            choices: Box::new([tasks::Verdict { code: 1, words: 32, followups: 0, fields: Box::new([]) }]),
        },
        tasks::Contract::Change { connector: 1, kind: 1, words: 32 },
    ] {
        for (requester, label) in [
            (tasks::Party::Person(19), b"[Requested by person 19]\n".as_slice()),
            (tasks::Party::Task(23), b"[Requested by task 23]\n".as_slice()),
            (tasks::Party::Deployment { project: 29 }, b"[Requested by deployment for project 29]\n".as_slice()),
        ] {
            for ended_by in [tasks::EndedBy::Result, tasks::EndedBy::Requester] {
                for (task, project, line) in [
                    (12, 3, b"[This task: 12, in project 3]\n".as_slice()),
                    (u64::MAX, u32::MAX, b"[This task: 18446744073709551615, in project 4294967295]\n".as_slice()),
                ] {
                    let mut context = settings::charters::context(task, CHARTER);
                    context.project = project;
                    context.contract = contract.clone();
                    context.requester = requester;
                    context.ended_by = ended_by;
                    core.contexts.insert(task, context).expect("context room");
                    let text = read_brief_part(
                        &core,
                        BriefTextLimits { parts: 2, read_bytes: 256, brief_bytes: 256 },
                        task,
                        BriefPart::Spec,
                    )
                    .expect("spec section");
                    let before_requester = text.len().checked_sub(label.len()).expect("requester line");
                    assert!(
                        text.ends_with(label)
                            && text.get(..before_requester).expect("through task line").ends_with(line),
                        "task {task}, project {project}, {contract:?}, {requester:?}, {ended_by:?}: {text:?}"
                    );
                    let before = before_requester.checked_sub(line.len()).expect("task line");
                    assert_eq!(text.get(before.checked_sub(1).expect("spec before task line")), Some(&b'\n'));
                }
            }
        }
    }

    let mut context = settings::charters::context(12, CHARTER);
    context.project = 3;
    context.ended_by = tasks::EndedBy::Requester;
    core.contexts.insert(12, context).expect("context room");
    for (parts, read_bytes, expected) in [
        (2, 5, b"task\n[54 bytes cut]\n".as_slice()),
        (2, 10, b"task\n[This\n[49 bytes cut]\n".as_slice()),
        (2, 33, b"task\n[This task: 12, in project 3\n[26 bytes cut]\n".as_slice()),
        (2, 55, b"task\n[This task: 12, in project 3]\n[Requested by person\n[4 bytes cut]\n".as_slice()),
        (1, 256, b"task\n[54 bytes cut]\n".as_slice()),
    ] {
        let text = read_brief_part(&core, BriefTextLimits { parts, read_bytes, brief_bytes: 256 }, 12, BriefPart::Spec)
            .expect("cut spec section");
        assert_eq!(text.as_ref(), expected, "{parts} parts, {read_bytes} bytes");
    }
}

#[test]
fn the_briefs_delegates_show_their_spec_and_hold_revisions() {
    let (mut core, _) = make_tree();
    for (phase, expected) in DELEGATES {
        let mut context = settings::charters::context(PARENT, CHARTER);
        context.delegates = Box::new([tasks::DelegateState {
            task: CHILD,
            phase,
            revisions: tasks::Revisions { spec: 1, hold: 2, person: 0 },
        }]);
        core.contexts.insert(PARENT, context).expect("context room");
        let text = read_brief_part(
            &core,
            BriefTextLimits { parts: 1, read_bytes: 256, brief_bytes: 256 },
            PARENT,
            BriefPart::Delegates,
        )
        .expect("delegates section");
        assert_eq!(text.as_ref(), expected);
    }
}

#[test]
fn a_brief_names_its_task_and_project() {
    let (mut core, _) = make_tree();
    let mut context = settings::charters::context(CHILD, CHARTER);
    context.project = 23;
    context.spec.words = b"Keep the task words first".as_slice().into();
    context.ended_by = tasks::EndedBy::Requester;
    core.contexts.insert(CHILD, context).expect("context room");
    let text =
        read_brief_part(&core, BriefTextLimits { parts: 2, read_bytes: 256, brief_bytes: 256 }, CHILD, BriefPart::Spec)
            .expect("spec section");
    assert_eq!(text.as_ref(), b"Keep the task words first\n[This task: 7, in project 23]\n[Requested by person 1]\n");
}
