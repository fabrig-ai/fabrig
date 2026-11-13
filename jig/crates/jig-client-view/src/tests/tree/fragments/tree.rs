#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests inspect checked sayings and public tree events"
)]
use crate::tests::fixtures::{AnsweredByEngine, Fixture, Watched};
use crate::tests::kit::Goldens;
use crate::tests::tree::cards::{CLIENT, document, drive, start, words};
use crate::words::Words;
use crate::{JigBindings, tree::fragments::tree as view};
use skein_ui::phrases::{PhraseLimits, Phrasebook, Segment};
use skein_ui::{Builder, Role, Tree};

use crate::{
    Event, Request,
    state::{
        action::Act as ClientAct,
        address::{Address, Place, TaskSection},
    },
};
use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn chip(n: u64) -> Chip {
    let mut c = crate::tests::tree::cards_objects::task_doc(&[]).chip;
    c.task = n;
    c.title.cut = true;
    c
}
fn counts() -> PhaseCounts {
    PhaseCounts {
        waiting: 1,
        idle: 2,
        starting: 3,
        running: 4,
        backing_off: 5,
        closing: 6,
        held: 7,
        done: 8,
        failed: 9,
        cancelled: u32::MAX,
    }
}
fn source() -> jig_client::contract::Tree {
    let mut rows = List::with_capacity(2);
    let proposals =
        crate::tests::kit::list_in(2, [ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX }]);
    let dependencies = crate::tests::kit::list_in(2, [TaskLink::Stored { number: u64::MAX, status: None }]);
    rows.push(TreeRow {
        chip: chip(9),
        parent: 8,
        depth: u32::MAX,
        proposals,
        spent: u64::MAX,
        dependencies,
        cut: true,
    })
    .unwrap();
    let folded = crate::tests::kit::list([
        TreeFold { parent: 0, phases: counts(), held_below: 11 },
        TreeFold { parent: u64::MAX, phases: PhaseCounts { idle: u32::MAX, ..counts() }, held_below: 17 },
    ]);
    jig_client::contract::Tree {
        root: chip(8),
        rows,
        folded,
        held_below: 19,
        next: Some(Cursor::new(copy_of(UTF8_TEXTS[0]), 128).unwrap()),
    }
}
fn domain(value: &jig_client::contract::Tree) -> Fixture {
    domain_and_watch(value).0
}
fn domain_and_watch(value: &jig_client::contract::Tree) -> (Fixture, jig_client::Handle) {
    let (mut d, frame) = start();
    drive(&mut d, Watched { watch: frame, event: WatchEvent::Delivered(Delivery::Me(Box::new(document()))) });
    let asks = drive(&mut d, Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Plan }) });
    let task =
        asks.iter()
            .find_map(|r| match r {
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
    let asks = drive(
        &mut d,
        Watched {
            watch: task,
            event: WatchEvent::Delivered(Delivery::Task(Box::new(crate::tests::tree::cards_objects::task_doc(&[])))),
        },
    );
    let watch =
        asks.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Tree { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    drive(&mut d, Watched { watch, event: WatchEvent::Delivered(Delivery::Tree(Box::new(value.clone()))) });
    (d, watch)
}
fn build(value: &jig_client::contract::Tree, d: &Fixture, w: &Words, nodes: Option<u32>) -> Tree {
    let f = crate::tests::render_limits(&CLIENT, w).unwrap();
    let mut l = crate::tests::render_limits(&CLIENT, w).unwrap();
    l.nodes = nodes.unwrap_or(f.nodes);
    l.depth = f.depth;
    l.text_bytes = f.text_bytes;
    l.text_per_node = f.text_per_node;
    let mut builder = Builder::new(&l);
    let mut bindings = JigBindings::new(f.bindings, CLIENT.client.text);
    view::render(&mut builder, value, &d.task_page().plan, &d.view(), w, &mut bindings);
    builder.finish()
}
#[test]
fn owner_tree_view_names_root_parent_depth_spend_loss_proposal_and_all_ten_counts_per_parent() {
    let mut goldens = Goldens::new("tree/fragments/tree/owner_tree_view_names");
    let value = source();
    let d = domain(&value);
    let w = words();
    let tree = build(&value, &d, w, None);
    goldens.check(&tree);
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::TreeItem).count(), 2);
    assert_eq!(d.task_page().plan.owner, Some(&value));
}
#[test]
fn owner_tree_view_preserves_all_link_metadata_without_fabricated_stored_chip_or_time() {
    let mut goldens = Goldens::new("tree/fragments/tree/owner_tree_view_preserves");
    let mut value = source();
    let w = words();
    for link in [
        TaskLink::Live(chip(17)),
        TaskLink::Retained { chip: chip(17), at: None },
        TaskLink::Retained { chip: chip(17), at: Some(Wall::EPOCH) },
        TaskLink::Stored { number: u64::MAX, status: None },
        TaskLink::Stored { number: u64::MAX, status: Some(TaskStatus::Done) },
        TaskLink::Stored { number: u64::MAX, status: Some(TaskStatus::Failed) },
        TaskLink::Stored { number: u64::MAX, status: Some(TaskStatus::Cancelled) },
    ] {
        value.rows.get_mut(0).unwrap().dependencies.clear();
        value.rows.get_mut(0).unwrap().dependencies.push(link).unwrap();
        let d = domain(&value);
        let tree = build(&value, &d, w, None);
        goldens.check(&tree);
        assert_eq!(d.task_page().plan.owner, Some(&value));
    }
}
#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn owner_tree_view_uses_checked_selected_language_and_finite_node_text_scratch_envelopes() {
    let mut goldens = Goldens::new("tree/fragments/tree/owner_tree_view_uses");
    let mut value = source();
    let first = value.rows.get_mut(0).unwrap();
    first.proposals.push(ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX }).unwrap();
    first.dependencies.push(TaskLink::Retained { chip: chip(17), at: Some(Wall::EPOCH) }).unwrap();
    let mut second = first.clone();
    second.chip.task = 10;
    second.parent = 9;
    second.depth = 2;
    value.rows.push(second).unwrap();
    let mut metadata = crate::tests::tree::fragments::chip::details();
    metadata.closing_connectors = Some(List::with_capacity(CLIENT.client.rows));
    *value.root.details = ChipPhaseFacts {
        phase: ChipPhase::Active(RunStage::Running { attempt: u64::MAX }),
        details: Box::new(metadata),
    };
    value.root.phase = Phase::Running;
    let d = domain(&value);
    let english = crate::words::phrases::english::language();
    let mut selected = english.clone();
    for phrase in &mut selected.phrases {
        if (8300..8340).contains(&phrase.id.number) {
            for form in &mut phrase.forms {
                for segment in &mut form.segments {
                    if let Segment::Text(bytes) = segment {
                        *bytes = copy_of(b"selected tree ");
                    }
                }
            }
        }
    }
    let w = Words::new(
        Phrasebook::load(
            crate::words::phrases::jig::shapes(256),
            Box::new([english, selected]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::words::document_words::phrase_count(),
                holes: 4,
                segments: crate::tests::english_phrase_segments(2),
                text_bytes: 100_000,
                hole_bytes: 256,
            },
        )
        .unwrap(),
        1,
        0,
    );
    let tree = build(&value, &d, &w, None);
    goldens.check(&tree);
    let f = crate::tests::render_limits(&CLIENT, &w).unwrap();
    assert!(tree.len() <= f.nodes);

    let exact = tree.len();
    assert_eq!(build(&value, &d, &w, Some(exact)).len(), exact);
    let fail = std::panic::catch_unwind(|| build(&value, &d, &w, Some(exact.checked_sub(1).unwrap())));
    let _panic = fail.unwrap_err();
    let mut bytes = 0_u32;
    for node in tree.nodes() {
        assert!(node.value.is_none() && node.external.is_none() && node.drawing.is_none());
        if let Some(name) = &node.name {
            bytes += u32::try_from(name.len()).unwrap();
        }
        for run in &node.text {
            bytes += u32::try_from(run.text.len()).unwrap();
        }
    }
    let mut cap = crate::tests::render_limits(&CLIENT, &w).unwrap();
    cap.nodes = exact;
    cap.depth = f.depth;
    cap.text_bytes = bytes;
    cap.text_per_node = f.text_per_node;
    let render = |cap: skein_ui::Limits| {
        let mut b = Builder::new(&cap);
        let mut bindings = JigBindings::new(f.bindings, CLIENT.client.text);
        view::render(&mut b, &value, &d.task_page().plan, &d.view(), &w, &mut bindings);
        b.finish()
    };
    assert_eq!(render(cap).len(), exact);
    cap.text_bytes = bytes.checked_sub(1).unwrap();
    let _panic = std::panic::catch_unwind(|| render(cap)).unwrap_err();
    let mut invalid = value.clone();
    invalid.rows = List::with_capacity(CLIENT.client.rows + 1);
}
#[test]
fn owner_tree_local_filters_folds_cycles_and_empty_loss_continuations_remain_bounded_and_independent() {
    let mut goldens = Goldens::new("tree/fragments/tree/owner_tree_local");
    let mut value = source();
    let w = words();
    value.rows.get_mut(0).unwrap().parent = 9;
    let d = domain(&value);
    let tree = build(&value, &d, w, None);
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::TreeItem).count(), 2);
    for next in [None, Some(Cursor::new(copy_of(b""), 128).unwrap())] {
        value.rows = List::with_capacity(2);
        value.folded = List::with_capacity(2);
        value.next = next.clone();
        let d = domain(&value);
        let tree = build(&value, &d, w, None);
        goldens.check(&tree);
        assert_eq!(d.task_page().plan.owner, Some(&value));
    }
}

#[test]
fn owner_tree_departure_keeps_a_separately_named_deciding_card_until_its_answer_once() {
    let mut goldens = Goldens::new("tree/fragments/tree/owner_tree_departure");
    let mut value = source();
    let key = value.rows.get(0).unwrap().proposals.get(0).copied().unwrap();
    let (mut d, tree_watch) = domain_and_watch(&value);
    let objects = crate::tests::kit::list([ObjectKey::Proposal(key)]);
    let asks = drive(&mut d, Event::Named { objects, sections: List::with_capacity(0) });
    let proposal_watch =
        asks.iter()
            .find_map(|ask| match ask {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Inbox { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let acts = crate::tests::kit::list([Offered { effect: None, act: ActKind::Accept, admission: Admission::Allowed }]);
    let proposal = Proposal {
        projects: List::with_capacity(0),
        key,
        project: 4,
        reason: DocumentText::complete(crate::tests::tree::cards::text(b"supplied pending reason")),
        action: ProposalAction::Release { task: TaskLink::Live(chip(9)), hold: None },
        as_holder: true,
        waiting: ProposalWaiting { holder: HistoryHolder::Task(9), since: Wall::EPOCH },
        funding: List::with_capacity(0),
        acts,
    };
    drive(
        &mut d,
        Watched { watch: proposal_watch, event: WatchEvent::Delivered(Delivery::Proposal(Box::new(proposal))) },
    );
    let id = d.objects().find(ObjectKey::Proposal(key)).unwrap();
    drive(&mut d, Event::Act { action: ClientAct::Intend { object: id.token(), act: ActKind::Accept } });
    let sent = drive(&mut d, Event::Act { action: ClientAct::Confirm });
    let request = sent
        .iter()
        .find_map(|ask| match ask {
            Request::Client { asked: jig_client::Event::Send { key: request, .. } } => Some(*request),
            _ => None,
        })
        .unwrap();
    let card_key = crate::tree::pages::object_key(ObjectKey::Proposal(key));
    let before = deciding_tree_page(&d);
    assert_eq!(before.nodes().iter().filter(|node| node.key == Some(card_key)).count(), 1);
    let row = d.objects().find(ObjectKey::Task(9)).unwrap();
    drive(&mut d, Event::Act { action: ClientAct::Fold { object: row.token(), open: false } });
    assert_eq!(
        deciding_tree_page(&d).nodes().iter().filter(|node| node.key == Some(card_key)).count(),
        0,
        "person fold stays hidden"
    );
    drive(&mut d, Event::Act { action: ClientAct::Fold { object: row.token(), open: true } });
    value.rows.get_mut(0).unwrap().proposals.clear();
    drive(&mut d, Watched { watch: tree_watch, event: WatchEvent::Delivered(Delivery::Tree(Box::new(value.clone()))) });
    assert!(!d.objects().get(id).unwrap().sources.is_empty());
    assert!(d.task_page().plan.owner.as_ref().unwrap().rows.get(0).unwrap().proposals.is_empty());
    let pending = deciding_tree_page(&d);
    assert_eq!(pending.nodes().iter().filter(|node| node.key == Some(card_key)).count(), 1);
    goldens.check(&pending);
    drive(
        &mut d,
        AnsweredByEngine {
            request,
            answer: Answer::Done(Outcome::Document(DocumentOutcome::ProposalDecided {
                proposal: key,
                by: 1,
                at: Wall::EPOCH,
                choice: ProposalChoice::Accepted,
                repeated: false,
            })),
        },
    );
    let answered = deciding_tree_page(&d);
    assert_eq!(answered.nodes().iter().filter(|node| node.key == Some(card_key)).count(), 1);
    goldens.check(&answered);
}

const UTF8_TEXTS: [&[u8]; 1] = [b"~\0/"];

fn deciding_tree_page(domain: &Fixture) -> Tree {
    let w = words();
    let mut cap = crate::tests::render_limits(&CLIENT, w).unwrap();
    cap.nodes = cap.nodes.max(4096);
    // This fragment needs the minimum scratch; whole-page text is priced by of_words.
    cap.text_bytes = cap.text_bytes.max(4096_u32.checked_mul(cap.text_per_node).unwrap());
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.client.text);
    b.open(Role::Page);
    view::render(
        &mut b,
        domain.task_page().plan.owner.as_ref().unwrap(),
        &domain.task_page().plan,
        &domain.view(),
        w,
        &mut bindings,
    );
    crate::tree::pages::after_page(&mut b, &domain.view(), w, &mut bindings);
    b.close();
    b.finish()
}

#[test]
fn a_tree_keeps_a_waiting_card_held_by_another_source_after_its_row_leaves() {
    let mut value = source();
    let key = ObjectKey::Proposal(*value.rows.get(0).unwrap().proposals.get(0).unwrap());
    let (mut domain, watch) = domain_and_watch(&value);
    domain.ui.objects.name(key, crate::state::objects::Source::Window { number: 23 }).unwrap();
    let card = crate::tree::pages::object_key(key);
    assert_eq!(deciding_tree_page(&domain).nodes().iter().filter(|node| node.key == Some(card)).count(), 1);
    value.rows.get_mut(0).unwrap().proposals.clear();
    drive(&mut domain, Watched { watch, event: WatchEvent::Delivered(Delivery::Tree(Box::new(value))) });
    assert_eq!(deciding_tree_page(&domain).nodes().iter().filter(|node| node.key == Some(card)).count(), 1);
}

#[test]
fn a_waiting_tree_card_is_still_owed_when_the_task_stops_offering_its_plan() {
    let value = source();
    let key = ObjectKey::Proposal(*value.rows.get(0).unwrap().proposals.get(0).unwrap());
    let (mut domain, _) = domain_and_watch(&value);
    let task = domain
        .ui
        .handles
        .watches
        .iter()
        .find_map(|(handle, held)| (held.subject == Subject::Task { task: 8 }).then_some(*handle))
        .unwrap();
    let mut finished = crate::tests::tree::cards_objects::task_doc(&[]);
    finished.delegates.clear();
    finished.context = None;
    drive(&mut domain, Watched { watch: task, event: WatchEvent::Delivered(Delivery::Task(Box::new(finished))) });
    assert!(!domain.shows(crate::state::page::Section::Task { task: 8, section: TaskSection::Plan }));
    let id = domain.objects().find(key).unwrap();
    assert!(domain.owes(id));
}

#[test]
fn a_task_end_keeps_its_other_documents_visible_while_the_final_header_is_read() {
    let value = source();
    let key = ObjectKey::Proposal(*value.rows.get(0).unwrap().proposals.get(0).unwrap());
    let (mut domain, _) = domain_and_watch(&value);
    let task = domain
        .ui
        .handles
        .watches
        .iter()
        .find_map(|(handle, held)| (held.subject == Subject::Task { task: 8 }).then_some(*handle))
        .unwrap();
    drive(&mut domain, crate::tests::fixtures::Ended { watch: task, end: WatchEnd::SubjectEnded });
    let w = words();
    let cap = crate::tests::tree::cards::limits();
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.client.text);
    b.open(Role::Page);
    crate::tree::pages::main(&mut b, &domain.view(), w, &mut bindings);
    crate::tree::pages::after_page(&mut b, &domain.view(), w, &mut bindings);
    b.close();
    let tree = b.finish();
    assert_eq!(tree.nodes().iter().filter(|node| node.key == Some(crate::tree::pages::object_key(key))).count(), 1);
}
