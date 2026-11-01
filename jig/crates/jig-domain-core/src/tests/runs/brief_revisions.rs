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
            b"What does this project hold? Answer in two sentences.\n[Report: at most 32 bytes]\n[Requested by person 1]\n"
                .as_slice(),
        ),
        (
            tasks::EndedBy::Requester,
            b"What does this project hold? Answer in two sentences.\n[Requested by person 1]\n".as_slice(),
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
