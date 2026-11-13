#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary proofs query checked source trees and select public action requests"
)]
use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
use crate::words::Words;
use crate::{JigBindings, tree::fragments::proposal as view};
use skein_ui::phrases::{Hole, PhraseLimits, Phrasebook, SpanUnit, WallFormat};
use skein_ui::{Builder, Tree};

use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn doc(bytes: &[u8], cut: bool) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), 32).unwrap(), cut }
}
fn limits() -> crate::tests::fixtures::Profile {
    crate::tests::fixtures::Profile {
        view: crate::Config { ..(crate::tests::tree::cards::CLIENT).view },
        client: jig_client::DomainLimits { rows: 3, text: 32, ..(crate::tests::tree::cards::CLIENT).client },
    }
}
fn part() -> Part {
    let mut segments = List::with_capacity(3);
    for s in [b"a/b".as_slice(), &[255, 0, 254], b""] {
        segments.push(copy_of(s)).unwrap();
    }
    Part { connector: u16::MAX, kind: PartKind::Outbox, key: PartKey::Path(segments) }
}
fn authority() -> AuthorityDocument {
    let mut grants = List::with_capacity(3);
    let mut notes = List::with_capacity(3);
    for _ in 0_u32..3 {
        let mut segments = List::with_capacity(3);
        for _ in 0_u32..3 {
            segments.push(copy_of(&[255; 32])).unwrap();
        }
        let pattern = Pattern { segments, last: PatternEnd::Open(copy_of(&[0; 32])) };
        grants.push(Grant { connector: u16::MAX, kind: u16::MAX, pattern: pattern.clone() }).unwrap();
        notes.push(ResourceScope { connector: u16::MAX, pattern }).unwrap();
    }
    let mut kinds = List::with_capacity(3);
    for v in [
        AuthorityExecutor::Charter(u32::MAX),
        AuthorityExecutor::Procedure(u32::MAX),
        AuthorityExecutor::Role(u32::MAX),
    ] {
        kinds.push(v).unwrap();
    }
    AuthorityDocument {
        authority: Authority {
            models: None,
            tools: u64::MAX,
            grants,
            delegation: Delegation { kinds, tasks: u32::MAX, depth: u32::MAX },
            spend: u64::MAX,
            deadline: Some(Wall::from_nanos(u64::MAX)),
            notes: u8::MAX,
            note_resources: notes,
        },
        cut: true,
    }
}
fn state(words: &[u8], cut: bool) -> AmendmentState {
    let mut parameters = List::with_capacity(3);
    let mut inputs = List::with_capacity(3);
    let mut dependencies = List::with_capacity(3);
    for n in 0..3 {
        parameters.push(ParameterDocument::Bytes { name: n, value: doc(UTF8_TEXTS[0], true) }).unwrap();
        inputs.push(u64::MAX.checked_sub(u64::from(n)).unwrap()).unwrap();
        dependencies.push(u64::from(n)).unwrap();
    }
    AmendmentState {
        addressed: Some(PersonAddress::Role(u32::MAX)),
        taken_by: Some(u64::MAX),
        instructions: Some(doc(words, cut)),
        spec: SpecificationDocument {
            specification: DocumentSpecification {
                asks: false,
                words: doc(b"# supplied spec", true),
                parameters,
                inputs,
            },
            cut: true,
        },
        authority: authority(),
        wake: crate::tests::tree::cards_objects::task_doc(&[]).wake,
        dependencies,
        cut: true,
    }
}
fn hold() -> ChipHold {
    ChipHold {
        words: None,
        placed: None,
        circle: None,
        account: None,
        source: Some(part()),
        reason: HoldCategory::Uncertain { entry: u64::MAX },
        revision: u64::MAX,
        before: PriorPhase::Active(RunStage::Running { attempt: 17 }),
        escalation: Some(HoldEscalation::Rejected { by: 19, reason: doc(b"hold explanation", true) }),
        tries: FailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: 6, infrastructure: 7 },
        hold_wait_since: Some(Wall::from_nanos(23)),
    }
}
fn source(action: ProposalAction) -> Proposal {
    Proposal {
        projects: List::with_capacity(0),
        key: ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX },
        project: u32::MAX,
        reason: doc(b"# whole proposal reason", true),
        action,
        as_holder: true,
        waiting: ProposalWaiting { holder: HistoryHolder::Task(31), since: Wall::from_nanos(37) },
        funding: List::with_capacity(3),
        acts: List::with_capacity(3),
    }
}
fn words(selected: bool) -> Words {
    let mut first = crate::words::phrases::english::language();
    first.wall = WallFormat::Nanos;
    first.span_unit = SpanUnit::Nanos;
    let languages: Box<[skein_ui::phrases::Language]> = if selected {
        let second = crate::tests::kit::translated_language(b"second ");
        Box::new([first, second])
    } else {
        Box::new([first])
    };
    Words::new(
        Phrasebook::load(
            crate::words::phrases::jig::shapes(256),
            languages,
            &PhraseLimits {
                languages: 2,
                phrases: crate::words::document_words::phrase_count(),
                holes: 4,
                segments: 10_000,
                text_bytes: 200_000,
                hole_bytes: 256,
            },
        )
        .unwrap(),
        u16::from(selected),
        0,
    )
}

fn build(value: &Proposal, w: &Words) -> (Tree, JigBindings) {
    let f = crate::tests::render_limits(&limits(), w).unwrap();
    let mut b = Builder::new(&f);
    let mut bindings = JigBindings::new(f.bindings, 256);
    let client = Fixture::new(&limits());
    view::render(&mut b, &client.view(), value, w, &mut bindings);
    let tree = b.finish();
    crate::tests::tree::cards_access::check(&tree);
    assert!(tree.len() <= f.nodes);
    let total: u64 = tree
        .nodes()
        .iter()
        .map(|n| {
            assert!(n.text_bytes() <= f.text_per_node);
            u64::from(n.text_bytes())
        })
        .sum();
    assert!(total <= u64::from(f.text_bytes));
    (tree, bindings)
}

fn batch() -> List<ProposedTask> {
    let mut tasks = List::with_capacity(3);
    for n in 0..3 {
        let a = state(b"task effective context", true);
        let mut resources = List::with_capacity(3);
        for _ in 0_u32..3 {
            resources.push(part()).unwrap();
        }
        tasks
            .push(ProposedTask {
                deadline: NewDeadline::Named,
                number: n,
                spec: a.spec,
                executor: Executor::Person(PersonAddress::Party(u64::MAX)),
                contract: TaskContractDocument::Approval { state: copy_of(&[255; 32]), part: part() },
                authority: a.authority,
                budget: u64::MAX,
                funder: Funder::Task(u64::MAX),
                dependencies: a.dependencies,
                resources,
                wake: a.wake,
                priority: Some(u32::MAX),
                cut: true,
            })
            .unwrap();
    }
    tasks
}
#[test]
fn owner_proposal_five_whole_actions_keep_effect_coordinates_release_hold_links_and_unscoped_outbox() {
    let mut goldens = Goldens::new("tree/fragments/proposal/owner_proposal_five");
    let w = words(false);
    let task = TaskLink::Stored { number: u64::MAX, status: None };
    let cases = [
        ProposalAction::Batch(batch()),
        ProposalAction::Effect {
            part: part(),
            authority: authority(),
            attempt: u64::MAX,
            completion: u32::MAX,
            position: u32::MAX,
            purpose: u64::MAX,
        },
        ProposalAction::ProcedureEffect { part: part(), authority: authority(), step: u64::MAX, purpose: u64::MAX },
        ProposalAction::Amend {
            task: task.clone(),
            addressed: None,
            before: Box::new(state(b"before context", true)),
            after: Box::new(state(b"after context", false)),
            reason: doc(b"why amend", true),
            release: None,
        },
        ProposalAction::Widen { task: task.clone(), before: authority(), after: authority() },
        ProposalAction::Release { task, hold: Some(Box::new(hold())) },
    ];
    for action in cases {
        let v = source(action);
        let original = v.clone();
        let (tree, bindings) = build(&v, &w);
        goldens.check(&tree);
        for node in tree.nodes() {
            if let Some(slot) = node.slot
                && let Some(raw) = bindings.owner_part(slot.token)
            {
                assert_eq!(raw.connector, u16::MAX);
                assert_eq!(raw.kind, PartKind::Outbox);
                assert_eq!(raw, &part());
            }
        }
        match &v.action {
            ProposalAction::Effect { .. } | ProposalAction::ProcedureEffect { .. } => {
                goldens.check(&tree);
                assert_eq!(v.funding.len(), 0);
            }
            ProposalAction::Release { .. } => {
                goldens.check(&tree);
            }
            _ => {}
        }
        assert_eq!(v, original);
    }
}
#[test]
fn owner_proposal_amendment_contexts_address_and_release_presence_never_join_a_current_task() {
    let mut goldens = Goldens::new("tree/fragments/proposal/owner_proposal_amendment");
    amendment_contexts(None, &mut goldens);
}

#[test]
fn owner_proposal_addressed_amendment_contexts_and_release_presence_never_join_a_current_task() {
    let mut goldens = Goldens::new("tree/fragments/proposal/owner_proposal_addressed");
    amendment_contexts(
        Some(AddressChange { revision: Revision { value: u64::MAX }, address: PersonAddress::Party(u64::MAX) }),
        &mut goldens,
    );
}

fn amendment_contexts(addressed: Option<AddressChange>, goldens: &mut Goldens) {
    let w = words(false);
    for release in [None, Some(0), Some(u64::MAX)] {
        for after in [None, Some(doc(b"", false)), Some(doc(b"exact effective after", true))] {
            let mut a = state(b"before supplied context", false);
            a.instructions = after.clone();
            let v = source(ProposalAction::Amend {
                task: TaskLink::Stored { number: 8, status: None },
                addressed,
                before: Box::new(state(b"before supplied context", false)),
                after: Box::new(a),
                reason: doc(b"own action reason", true),
                release,
            });
            let tree = build(&v, &w).0;
            goldens.check(&tree);
        }
    }
}

#[test]
fn owner_proposal_all_funders_independent_optional_amounts_and_holder_variants_preserve_supplied_identity() {
    let mut goldens = Goldens::new("tree/fragments/proposal/owner_proposal_all");
    let w = words(false);
    for funder in [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX },
        Funder::Recurring { project: u32::MAX, task: u64::MAX, period: u64::MAX },
        Funder::Period { project: u32::MAX, period: u64::MAX },
    ] {
        for left in [None, Some(0), Some(u64::MAX)] {
            for after in [None, Some(0), Some(u64::MAX)] {
                for as_holder in [false, true] {
                    let mut v = source(ProposalAction::Release {
                        task: TaskLink::Stored { number: 8, status: None },
                        hold: None,
                    });
                    v.as_holder = as_holder;
                    v.funding.push(FundingOffer { source: funder, needed: u64::MAX, left, after }).unwrap();
                    let original = v.clone();
                    let tree = build(&v, &w).0;
                    goldens.check(&tree);
                    assert_eq!(v, original);
                }
            }
        }
    }
    for kind in [
        MessageProposalKind::Batch,
        MessageProposalKind::Effect,
        MessageProposalKind::Amend,
        MessageProposalKind::Widen,
        MessageProposalKind::Release,
    ] {
        let mut v = source(ProposalAction::Batch(List::with_capacity(3)));
        v.waiting.holder = HistoryHolder::Policy { project: u32::MAX, kind };
        goldens.check(&build(&v, &w).0);
    }
}
#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn owner_proposal_exact_checked_envelopes_keep_selected_language_whole_loss_and_one_under_refusals() {
    let mut goldens = Goldens::new("tree/fragments/proposal/owner_proposal_exact");
    let w = words(true);
    let v = source(ProposalAction::Batch(batch()));
    let unchanged = v.clone();
    let (tree, _) = build(&v, &w);
    goldens.check(&tree);
    let f = crate::tests::render_limits(&limits(), &w).unwrap();

    assert_eq!(v, unchanged);
    assert!(
        view::max_nodes(
            &crate::tests::fixtures::Profile {
                view: crate::Config { ..(limits()).view },
                client: jig_client::DomainLimits { rows: u32::MAX, ..(limits()).client }
            },
            &(crate::tests::fixtures::Profile {
                view: crate::Config { ..(limits()).view },
                client: jig_client::DomainLimits { rows: u32::MAX, ..(limits()).client }
            })
            .client
        )
        .is_none()
    );

    let id = crate::words::proposal_words::ProposalPhrase::Attempt.id();
    assert_eq!(w.book().say(1, id, &[Hole::Wall(Wall::EPOCH)]).unwrap_err(), skein_ui::phrases::Invalid::Values { id });
    assert_eq!(w.book().say(1, id, &[Hole::Number(u64::MAX)]).unwrap().text.as_ref(), b"second 18446744073709551615");
    let short = skein_ui::Limits { nodes: tree.len() - 1, ..f };
    let _refusal = std::panic::catch_unwind(|| {
        let mut b = Builder::new(&short);
        let mut bindings = JigBindings::new(f.bindings, 256);
        view::render(&mut b, &Fixture::new(&limits()).view(), &v, &w, &mut bindings);
        b.finish()
    })
    .unwrap_err();
}
#[test]
fn owner_proposal_link_presence_and_same_keyed_card_across_contexts_keep_current_facts_and_checked_confirmation() {
    use crate::{Event, state::action::Act};
    let mut goldens = Goldens::new("tree/fragments/proposal/owner_proposal_link");
    let w = words(false);
    let mut chip = crate::tests::tree::cards_objects::task_doc(&[]).chip;
    chip.title = doc(b"supplied linked title", true);
    let links = [
        TaskLink::Live(chip.clone()),
        TaskLink::Retained { chip: chip.clone(), at: None },
        TaskLink::Retained { chip, at: Some(Wall::EPOCH) },
        TaskLink::Stored { number: 8, status: None },
        TaskLink::Stored { number: 8, status: Some(TaskStatus::Done) },
        TaskLink::Stored { number: 8, status: Some(TaskStatus::Failed) },
        TaskLink::Stored { number: 8, status: Some(TaskStatus::Cancelled) },
    ];
    for link in links {
        let v = source(ProposalAction::Release { task: link.clone(), hold: None });
        let tree = build(&v, &w).0;
        goldens.check(&tree);
    }
    whole_linked_owner_carrier(&w);
    let mut maker_keys = List::with_capacity(2);
    for maker in [Proposer::Task(u64::MAX), Proposer::Party(u64::MAX)] {
        let mut v = source(ProposalAction::Release { task: TaskLink::Stored { number: 8, status: None }, hold: None });
        v.key.proposer = maker;
        v.funding = List::with_capacity(2);
        v.acts = List::with_capacity(2);
        v.acts.push(Offered { effect: None, act: ActKind::Accept, admission: Admission::Allowed }).unwrap();
        let (mut client, id) = crate::tests::tree::cards_objects::loaded(
            ObjectKey::Proposal(v.key),
            ReadValue::Proposal(Box::new(v.clone())),
        );
        let f = crate::tests::render_limits(&limits(), &w).unwrap();
        let mut expected = None;
        for place in [
            crate::tree::cards::Place::Inbox { lineage: List::with_capacity(0) },
            crate::tree::cards::Place::Conversation,
            crate::tree::cards::Place::TaskPage,
            crate::tree::cards::Place::Elsewhere,
        ] {
            let mut b = Builder::new(&skein_ui::Limits { nodes: f.nodes + 100, ..f });
            let mut bindings = JigBindings::new(f.bindings, 256);
            assert!(crate::tree::cards::card(&mut b, &client.view(), id, &place, &w, &mut bindings));
            let tree = b.finish();
            crate::tests::tree::cards_access::check(&tree);
            goldens.check(&tree);
            let key = tree.nodes()[0].key.unwrap();
            if let Some(previous) = expected {
                assert_eq!(key, previous);
            } else {
                expected = Some(key);
            }
            assert_eq!(crate::state::objects::proposal::owner_facts(&client.objects().get(id).unwrap()), Some(&v));
        }
        maker_keys.push(expected.unwrap()).unwrap();
        crate::tests::tree::cards::drive(
            &mut client,
            Event::Act { action: Act::Intend { object: id.token(), act: ActKind::Accept } },
        );
        assert_eq!(client.confirmation().unwrap().revision, None);
        let mut b = Builder::new(&skein_ui::Limits { nodes: f.nodes + 100, ..f });
        let mut bindings = JigBindings::new(f.bindings, 256);
        crate::tree::confirmation::confirmation(&mut b, &client.view(), &w, &mut bindings);
        let tree = b.finish();
        crate::tests::tree::cards_access::check(&tree);
        goldens.check(&tree);
        crate::tests::tree::cards::drive(&mut client, Event::Act { action: Act::Dismiss });
        assert!(client.confirmation().is_none());
    }
    assert_ne!(maker_keys.get(0).unwrap(), maker_keys.get(1).unwrap());
}

fn whole_linked_owner_carrier(w: &Words) {
    let mut goldens = Goldens::new("tree/fragments/proposal/whole_linked_owner");
    let mut details = crate::tests::tree::fragments::chip::details();
    details.hold = Some(hold());
    details.ending = Some(ChipEnding::Failed(doc(b"whole linked failure", true)));
    details.resources = List::with_capacity(3);
    for _ in 0_u32..3 {
        details
            .resources
            .push(ResourceLine {
                part: part(),
                kind: u16::MAX,
                taken: false,
                holder: None,
                writer: Some(ResourceWriter::Effect { entry: u64::MAX }),
                writer_lost: true,
            })
            .unwrap();
    }
    details.resources_cut = true;
    let mut owner_chip = crate::tests::tree::cards_objects::task_doc(&[]).chip;
    owner_chip.details = Box::new(ChipPhaseFacts { phase: ChipPhase::Ended, details: Box::new(details) });
    let linked = source(ProposalAction::Release {
        task: TaskLink::Retained { chip: owner_chip, at: Some(Wall::EPOCH) },
        hold: Some(Box::new(hold())),
    });
    let linked_tree = build(&linked, w).0;
    goldens.check(&linked_tree);
    let (linked_client, linked_id) = crate::tests::tree::cards_objects::loaded(
        ObjectKey::Proposal(linked.key),
        ReadValue::Proposal(Box::new(linked.clone())),
    );
    assert_eq!(
        crate::state::objects::proposal::owner_facts(&linked_client.objects().get(linked_id).unwrap()),
        Some(&linked),
    );
}

const UTF8_TEXTS: [&[u8]; 1] = [&[b'~'; 32]];
