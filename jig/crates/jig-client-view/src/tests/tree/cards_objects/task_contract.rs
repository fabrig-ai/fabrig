use crate::state::address::{Address, TaskSection};
use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;

pub(in crate::tests) fn page(row: Task) -> (Tree, JigBindings) {
    let (mut domain, out) = crate::tests::tree::work_pages::entered(Address::task(8));
    let watch =
        out.iter()
            .find_map(|request| match request {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    crate::tests::tree::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row)));
    let built = crate::tests::tree::pages::build(&domain, 3, words());
    crate::tests::tree::cards_access::check(&built.0);
    assert!(built.0.len() <= crate::task_nodes(&CLIENT, &(CLIENT).client, 0).unwrap());
    built
}

#[test]
fn task_contract_overview_preserves_opaque_approval_and_named_verdict_caps_without_option_labels() {
    let mut goldens = Goldens::new("tree/cards_objects/task_contract/task_contract_overview");
    let reference = Part { connector: u16::MAX, kind: PartKind::Result, key: PartKey::Number(u64::MAX) };
    let mut choices = List::with_capacity(2);
    let fields = crate::tests::kit::list_in(2, [FieldRule { name: Box::from(*b"field\xff/"), max: u32::MAX }]);
    choices.push(VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields }).unwrap();
    for contract in [
        TaskContractDocument::Report { words: u32::MAX },
        TaskContractDocument::Approval { state: Box::from(*b"\xff\0/"), part: reference.clone() },
        TaskContractDocument::Verdict { choices: choices.clone(), cut: false },
        TaskContractDocument::Verdict { choices, cut: true },
        TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX },
    ] {
        let mut row = task_doc(&[]);
        row.spec.specification.words = DocumentText::complete(text(b""));
        row.result_contract = contract.clone();
        row.cut = false;
        let (tree, bindings) = page(row);
        goldens.check(&tree);
        match contract {
            TaskContractDocument::Report { .. } => goldens.check(&tree),
            TaskContractDocument::Approval { state, .. } => {
                assert_eq!(&*state, b"\xff\0/");
                goldens.check(&tree);
                let slots: Vec<_> = tree.nodes().iter().filter_map(|node| node.slot).collect();
                assert_eq!(slots.len(), 1);
                assert_eq!(bindings.owner_part(slots[0].token), Some(&reference));
            }
            TaskContractDocument::Verdict { .. } | TaskContractDocument::Change { .. } => {
                goldens.check(&tree);
            }
        }
    }
}

#[test]
fn task_header_links_the_supplied_served_goal_without_turning_ancestor_identity_into_self_goal() {
    let mut goldens = Goldens::new("tree/cards_objects/task_contract/task_header_links");
    for goal in [None, Some(8), Some(0), Some(u64::MAX)] {
        let mut row = task_doc(&[]);
        row.goal = goal;
        let (tree, mut bindings) = page(row);
        let mut goals = Vec::new();
        for node in tree.nodes() {
            if let Some(binding) = node.binding
                && let Some(Action::Go { address: Address::Task { task, section: TaskSection::Overview } }) =
                    decode(&mut bindings, binding.token, Act::Press)
                && (task == 0 || task == u64::MAX)
            {
                goals.push(task);
            }
        }
        assert_eq!(goals, goal.filter(|number| *number != 8).into_iter().collect::<Vec<_>>());
        goldens.check(&tree);
    }
}

#[test]
fn maximum_nested_task_contracts_fit_the_page_bound_and_preserve_every_raw_slot_and_field() {
    let mut goldens = Goldens::new("tree/cards_objects/task_contract/maximum_nested_task");
    let raw = vec![0xff; usize::try_from(CLIENT.client.text).unwrap()].into_boxed_slice();
    let mut segments = List::with_capacity(CLIENT.client.rows);
    for _ in 0..CLIENT.client.rows {
        segments.push(raw.clone()).unwrap();
    }
    let reference = Part { connector: u16::MAX, kind: PartKind::Resource, key: PartKey::Path(segments) };
    let mut fields = List::with_capacity(CLIENT.client.rows);
    for _ in 0..CLIENT.client.rows {
        fields.push(FieldRule { name: raw.clone(), max: u32::MAX }).unwrap();
    }
    let mut choices = List::with_capacity(CLIENT.client.rows);
    for _ in 0..CLIENT.client.rows {
        choices
            .push(VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields: fields.clone() })
            .unwrap();
    }
    for contract in [
        TaskContractDocument::Approval { state: raw.clone(), part: reference.clone() },
        TaskContractDocument::Verdict { choices, cut: true },
    ] {
        let mut row = task_doc(&[]);
        row.goal = Some(u64::MAX);
        row.spec.specification.words = DocumentText::complete(text(b""));
        row.result_contract = contract.clone();
        let (tree, bindings) = page(row);

        goldens.check(&tree);
        match contract {
            TaskContractDocument::Approval { state, .. } => {
                assert_eq!(state, raw);
                let slots: Vec<_> = tree.nodes().iter().filter_map(|node| node.slot).collect();
                assert_eq!(slots.len(), 1);
                assert_eq!(bindings.owner_part(slots[0].token), Some(&reference));
            }
            TaskContractDocument::Verdict { choices, .. } => {
                assert_eq!(choices.len(), CLIENT.client.rows);
                let fields = tree
                    .nodes()
                    .iter()
                    .filter(|node| {
                        node.text
                            .iter()
                            .flat_map(|run| run.text.iter().copied())
                            .eq(b"Field value byte limit 4294967295".iter().copied())
                    })
                    .count();
                assert_eq!(fields, usize::try_from(CLIENT.client.rows * CLIENT.client.rows).unwrap());
                goldens.check(&tree);
            }
            TaskContractDocument::Report { .. } | TaskContractDocument::Change { .. } => unreachable!(),
        }
    }
}

#[test]
fn refused_chat_header_shows_the_supplied_cause_and_receipt_time() {
    let mut goldens = Goldens::new("tree/cards_objects/task_contract/refused_chat_header");
    for cause in [
        TaskWaitRefused::Program(OpenProgram::Model),
        TaskWaitRefused::Program(OpenProgram::Source { account: 17, source: 23 }),
        TaskWaitRefused::Hello { kind: HostKind::Worker, at: Wall::EPOCH, why: HelloRefusal::Position { worker: 7 } },
    ] {
        let mut row = task_doc(&[]);
        row.flags.conversation = true;
        row.waits_on = WaitsOn::Nothing;
        row.run_wait = Some(TaskRunWait::Refused(cause));
        row.chip.phase = Phase::Due;
        row.chip.details = Box::new(ChipPhaseFacts {
            phase: ChipPhase::Active(RunStage::Due),
            details: Box::new(crate::tests::tree::fragments::chip::details()),
        });
        let (tree, _) = page(row);
        goldens.check(&tree);
    }
}
