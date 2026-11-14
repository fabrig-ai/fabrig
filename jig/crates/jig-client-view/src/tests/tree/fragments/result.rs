#![expect(
    clippy::disallowed_methods,
    reason = "ordinary tests inspect standalone trees and mutate checked language data"
)]
use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
use crate::words::Words;
use crate::words::{result_words, result_words::ResultPhrase};
use crate::{JigBindings, tree::fragments::result as view};
use jig_client::contract::{ChipPhase, RunStage};
use skein_ui::phrases::{Invalid, Language, PhraseLimits, Phrasebook, Segment, WallFormat};
use skein_ui::{Builder, Tree};

use jig_client::contract::{
    Chip, ChipExecutor, Executor, Part, PartKey, PartKind, PersonAddress, Phase, ProposalKey, Proposer, Requester,
    ResultDetails, TaskContractDocument, TaskLink, TaskStatus, Text,
};
use skein_lib::{List, Wall, bytes::copy_of};

fn client_limits() -> crate::tests::fixtures::Profile {
    crate::tests::fixtures::Profile {
        view: crate::Config { ..(crate::tests::tree::cards::CLIENT).view },
        client: jig_client::DomainLimits { rows: 8, ..(crate::tests::tree::cards::CLIENT).client },
    }
}
fn phrase_limits(languages: &[Language]) -> PhraseLimits {
    let mut segments = 0_u32;
    for language in languages {
        for phrase in &language.phrases {
            for form in &phrase.forms {
                segments = segments
                    .checked_add(u32::try_from(form.segments.len()).expect("fixture segment count fits u32"))
                    .expect("whole fixture segment count fits u32");
            }
        }
    }
    PhraseLimits {
        languages: 2,
        phrases: crate::words::document_words::phrase_count(),
        holes: 4,
        segments,
        text_bytes: 100_000,
        hole_bytes: 128,
    }
}
fn words() -> Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| Words::new(crate::words::phrases::english::english(128).unwrap(), 0, 0)).clone()
}
fn chip() -> Chip {
    Chip {
        task: u64::MAX,
        title: jig_client::contract::DocumentText::complete(Text::new(copy_of(b"supplied title"), 128).unwrap()),
        executor: ChipExecutor::Agent { charter: u32::MAX },
        phase: Phase::Due,
        details: Box::new(crate::tests::fixtures::chip_phase(ChipPhase::Active(RunStage::Preparing))),
    }
}
fn details() -> ResultDetails {
    ResultDetails {
        contract: TaskContractDocument::Report { words: u32::MAX },
        requester: Requester::Deployment { project: u32::MAX },
        executor: Executor::Person(PersonAddress::Role(u32::MAX)),
        chosen_by: None,
        delegates: List::with_capacity(0),
        followups: None,
        proposal: None,
        cut: false,
    }
}
fn build(value: &ResultDetails, words: &Words) -> Tree {
    let client = Fixture::new(&client_limits());
    let limits = view::limits(&client_limits(), &(client_limits()).client, words).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, 128);
    view::render(&mut b, &client.view(), value, words, &mut bindings);
    b.finish()
}

#[test]
fn result_metadata_keeps_requester_variants_chooser_and_task_or_party_makers() {
    let mut goldens = Goldens::new("tree/fragments/result/result_metadata_keeps");
    let mut value = details();
    let words = words();
    for requester in
        [Requester::Task(u64::MAX), Requester::Person(u64::MAX), Requester::Deployment { project: u32::MAX }]
    {
        value.requester = requester;
        goldens.check(&build(&value, &words));
    }
    goldens.check(&build(&value, &words));
    value.chosen_by = Some(u64::MAX);
    value.proposal = Some(ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX });
    let task = build(&value, &words);
    goldens.check(&task);
    value.proposal = Some(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX });
    let party = build(&value, &words);
    goldens.check(&party);
}
#[test]
fn result_task_links_keep_live_retained_stored_optional_times_and_all_statuses() {
    let mut goldens = Goldens::new("tree/fragments/result/result_task_links");
    let mut value = details();
    value.delegates = List::with_capacity(7);
    for link in [
        TaskLink::Live(chip()),
        TaskLink::Retained { chip: chip(), at: None },
        TaskLink::Retained { chip: chip(), at: Some(Wall::from_nanos(120_000_000_000)) },
        TaskLink::Stored { number: u64::MAX, status: None },
        TaskLink::Stored { number: u64::MAX, status: Some(TaskStatus::Done) },
        TaskLink::Stored { number: u64::MAX, status: Some(TaskStatus::Failed) },
        TaskLink::Stored { number: u64::MAX, status: Some(TaskStatus::Cancelled) },
    ] {
        value.delegates.push(link).unwrap();
    }
    let tree = build(&value, &words());
    goldens.check(&tree);
    assert_eq!(
        tree.nodes().iter().filter(|n| n.binding.is_some()).count(),
        3,
        "stored references acquire no fabricated chip link"
    );
    assert_eq!(value.delegates.get(1), Some(&TaskLink::Retained { chip: chip(), at: None }));
}
#[test]
fn result_followups_distinguish_absent_empty_nonempty_and_preserve_aggregate_cut() {
    let mut goldens = Goldens::new("tree/fragments/result/result_followups_distinguish");
    let mut value = details();
    let words = words();
    goldens.check(&build(&value, &words));
    value.followups = Some(List::with_capacity(2));
    let empty = build(&value, &words);
    goldens.check(&empty);
    value.followups.as_mut().unwrap().push(TaskLink::Stored { number: 42, status: None }).unwrap();
    value.cut = true;
    let cut = build(&value, &words);
    goldens.check(&cut);
}
#[test]
fn result_metadata_reuses_whole_contract_opaque_parts_and_owner_executor() {
    let mut goldens = Goldens::new("tree/fragments/result/result_metadata_reuses");
    let mut value = details();
    let segments = crate::tests::kit::list([copy_of(b"a/b"), copy_of(&[255, 0])]);
    value.contract = TaskContractDocument::Approval {
        state: copy_of(&[128, 0]),
        part: Part { connector: u16::MAX, kind: PartKind::Outbox, key: PartKey::Path(segments) },
    };
    value.executor = Executor::Procedure { connector: u16::MAX, code: u32::MAX };
    let tree = build(&value, &words());
    goldens.check(&tree);
    assert_eq!(tree.nodes().iter().filter(|n| n.slot.is_some()).count(), 1);
    value.contract = TaskContractDocument::Verdict { choices: List::with_capacity(0), cut: true };
    let cut = build(&value, &words());
    goldens.check(&cut);
}
#[test]
fn result_words_check_shapes_select_language_and_fit_exact_tree_capacity() {
    let mut goldens = Goldens::new("tree/fragments/result/result_words_check");
    let normal = crate::words::phrases::english::language();
    let mut translated = crate::words::phrases::english::language();
    translated.wall = WallFormat::Clock;
    for phrase in &mut translated.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                match segment {
                    Segment::Text(bytes) => *bytes = copy_of(b"selected "),
                    Segment::Hole { .. } => {}
                }
            }
        }
    }
    let languages = Box::new([normal, translated]);
    let limits = phrase_limits(languages.as_slice());
    let book = Phrasebook::load(crate::words::phrases::jig::shapes(128), languages, &limits).unwrap();
    let words = Words::new(book, 1, 0);
    let tree = build(&details(), &words);
    goldens.check(&tree);
    let mut language = result_words::language();
    language.phrases[0].id = ResultPhrase::Contract.id();
    assert_eq!(
        Phrasebook::load(result_words::shapes(), Box::new([language]), &limits).err(),
        Some(Invalid::Duplicate { id: ResultPhrase::Contract.id() })
    );
    let mut language = result_words::language();
    language.phrases[2].forms[0].segments = Box::new([Segment::Text(copy_of(b"missing hole"))]);
    assert_eq!(
        Phrasebook::load(result_words::shapes(), Box::new([language]), &limits).err(),
        Some(Invalid::Holes { id: ResultPhrase::RequesterTask.id() })
    );
    let value = details();
    let client = Fixture::new(&client_limits());
    let mut capacity = view::limits(&client_limits(), &(client_limits()).client, &words).unwrap();
    assert_eq!(tree.len(), 9, "one region and eight independent metadata leaves");
    capacity.nodes = 9;
    capacity.depth = 2;
    capacity.text_per_node = 19;
    capacity.text_bytes = 112;
    let mut b = Builder::new(&capacity);
    let mut bindings = JigBindings::new(capacity.bindings, 128);
    view::render(&mut b, &client.view(), &value, &words, &mut bindings);
    let used = b.finish().used();
    assert_eq!(used.nodes, capacity.nodes);
    assert_eq!(used.depth, capacity.depth);
    assert_eq!(used.text_per_node, capacity.text_per_node);
    assert_eq!(used.text_bytes, capacity.text_bytes);
    assert_eq!(
        view::nodes(
            &crate::tests::fixtures::Profile {
                view: crate::Config { ..(client_limits()).view },
                client: jig_client::DomainLimits { rows: u32::MAX, ..(client_limits()).client }
            },
            &(crate::tests::fixtures::Profile {
                view: crate::Config { ..(client_limits()).view },
                client: jig_client::DomainLimits { rows: u32::MAX, ..(client_limits()).client }
            })
            .client
        ),
        None
    );
}
