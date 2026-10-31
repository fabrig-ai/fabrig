#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary view proofs drive public events and inspect selected semantic trees"
)]
use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
use crate::tests::tree::cards::{CLIENT, text};
use crate::words::{Words, chip_words::ChipPhrase as P};
use crate::{
    Request,
    state::address::{Address, TaskSection},
};
use jig_client::contract::*;
use skein_lib::{Token, Wall, bytes::copy_of};
use skein_ui::Tree;

fn words() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| {
        let mut language = crate::words::phrases::english::language();
        language.wall = skein_ui::phrases::WallFormat::Nanos;
        Words::new(
            skein_ui::phrases::Phrasebook::load(
                crate::words::phrases::jig::shapes(CLIENT.client.text),
                Box::new([language]),
                &skein_ui::phrases::PhraseLimits {
                    languages: 1,
                    phrases: crate::words::owner_words::phrase_count(),
                    holes: 4,
                    segments: crate::tests::english_phrase_segments(1),
                    text_bytes: 100_000,
                    hole_bytes: CLIENT.client.text,
                },
            )
            .unwrap(),
            0,
            0,
        )
    })
}
fn selected() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| {
        let mut language = crate::words::phrases::english::language();
        language.wall = skein_ui::phrases::WallFormat::Nanos;
        for phrase in &mut language.phrases {
            if [P::Starting.id(), P::Writing.id(), P::Tool.id()].contains(&phrase.id) {
                for form in &mut phrase.forms {
                    for segment in &mut form.segments {
                        if let skein_ui::phrases::Segment::Text(bytes) = segment {
                            *bytes = copy_of(b"Selected work report");
                        }
                    }
                }
            }
        }
        Words::new(
            skein_ui::phrases::Phrasebook::load(
                crate::words::phrases::jig::shapes(CLIENT.client.text),
                Box::new([crate::words::phrases::english::language(), language]),
                &skein_ui::phrases::PhraseLimits {
                    languages: 2,
                    phrases: crate::words::owner_words::phrase_count(),
                    holes: 4,
                    segments: crate::tests::english_phrase_segments(2),
                    text_bytes: 100_000,
                    hole_bytes: CLIENT.client.text,
                },
            )
            .unwrap(),
            1,
            0,
        )
    })
}
fn report(work: OwnerChipWork, started: Option<Wall>) -> OwnerChipObservation {
    OwnerChipObservation {
        channel: Token::new(u64::MAX),
        work,
        since: Wall::from_nanos(31),
        reported_at: Wall::from_nanos(47),
        started,
    }
}
fn task(observation: Option<OwnerChipObservation>, context: bool) -> Task {
    let mut task = crate::tests::tree::cards_objects::task_doc(&[]);
    task.chip.executor = Executor::Agent { charter: 1 };
    let mut details = crate::tests::tree::fragments::chip::details();
    details.project = task.project;
    details.executor = OwnerExecutor::Agent { charter: 1 };
    details.observation = observation;
    task.chip.phase = Phase::Running;
    task.chip.details = Box::new(ChipPhase {
        phase: OwnerChipPhase::Active(RunStage::Running { attempt: u64::MAX }),
        details: Box::new(details),
    });
    task.hold = None;
    task.goal = None;
    task.delegates = crate::tests::tree::work_pages::list([]);
    if context {
        task.context.as_mut().unwrap().runtime = Runtime::Running {
            attempt: u64::MAX,
            since: Wall::from_nanos(59),
            tool: Some(text(b"independent synthetic runtime")),
        };
    } else {
        task.context = None;
    }
    task
}
fn page(task: Task, section: TaskSection) -> (Fixture, Vec<Request>) {
    let (mut domain, requests) = crate::tests::tree::work_pages::entered(Address::Task { task: 8, section });
    let watch = requests
        .iter()
        .find_map(|request| match request {
            Request::Client {
                asked:
                    jig_client::Event::Watch {
                        handle: watch,
                        subject: jig_client::Watchable::Jig(Subject::Task { task: 8 }),
                    },
            } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let requests = crate::tests::tree::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(task)));
    (domain, requests)
}
fn render(domain: &Fixture, words: &Words) -> Tree {
    let (tree, _) = crate::tests::tree::pages::build(domain, 3, words);
    crate::tests::tree::cards_access::check(&tree);
    assert!(tree.len() <= crate::task_nodes(&CLIENT, &(CLIENT).client, 0).unwrap());
    tree
}
#[test]
fn owner_work_activity_without_synthetic_context_shows_all_seven_reports_and_independent_clocks() {
    let mut goldens = Goldens::new("state/conversation_work/owner_work_activity");
    for work in [
        OwnerChipWork::Preparing,
        OwnerChipWork::Starting,
        OwnerChipWork::Active,
        OwnerChipWork::Writing,
        OwnerChipWork::Tool { call: copy_of(&[255, 0]), tool: copy_of(b"literal/tool") },
        OwnerChipWork::Waiting,
        OwnerChipWork::Ending,
    ] {
        for started in [None, Some(Wall::EPOCH), Some(Wall::from_nanos(u64::MAX))] {
            let source = report(work.clone(), started);
            let doc = task(Some(source.clone()), false);
            assert!(doc.context.is_none());
            let (domain, _) = page(doc.clone(), TaskSection::Activity);
            let tree = render(&domain, words());
            goldens.check(&tree);
            let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
            let Some(crate::state::objects::Body::Task(retained)) = &domain.objects().get(id).unwrap().body else {
                panic!("whole task retained");
            };
            assert_eq!(retained.as_ref(), &doc);
        }
    }
}
#[test]
fn owner_work_absence_does_not_fabricate_report_and_preserves_independent_synthetic_runtime() {
    let mut goldens = Goldens::new("state/conversation_work/owner_work_absence");
    for context in [false, true] {
        for observation in [None, Some(report(OwnerChipWork::Writing, None))] {
            let (domain, _) = page(task(observation, context), TaskSection::Activity);
            let tree = render(&domain, words());
            goldens.check(&tree);
            assert!(tree.nodes().iter().any(|node| node.role == skein_ui::Role::Log));
        }
    }
}
#[test]
fn selected_owner_work_activity_preserves_conversation_and_opaque_provisional_text() {
    let mut goldens = Goldens::new("state/conversation_work/selected_owner_work");
    for work in [
        OwnerChipWork::Starting,
        OwnerChipWork::Writing,
        OwnerChipWork::Tool { call: copy_of(&[255, 0]), tool: copy_of(b"selected/tool") },
    ] {
        let (mut domain, requests) = page(task(Some(report(work, None)), false), TaskSection::Activity);
        let run =
            requests
                .iter()
                .find_map(|request| match request {
                    Request::Client {
                        asked:
                            jig_client::Event::Watch {
                                handle: watch,
                                subject: jig_client::Watchable::Jig(Subject::Run { .. }),
                            },
                    } => Some(*watch),
                    _ => None,
                })
                .unwrap();
        crate::tests::tree::pages::deliver(
            &mut domain,
            run,
            crate::tests::fixtures::run(RunText {
                writing: None,
                attempt: u64::MAX,
                began: None,
                turn: ProvisionalTurnName { retry: u32::MAX, attempt: u64::MAX, number: 1 },
                from: 0,
                through: 17,
                text: text(b"provisional reply"),
                cut: false,
                call: None,
            }),
        );
        let tree = render(&domain, selected());
        goldens.check(&tree);
    }
}
#[test]
fn owner_work_overview_and_activity_keep_the_same_supplied_observation_without_full_metadata_in_activity() {
    let mut goldens = Goldens::new("state/conversation_work/owner_work_overview");
    let value = task(Some(report(OwnerChipWork::Writing, None)), false);
    for section in [TaskSection::Overview, TaskSection::Activity] {
        let (domain, _) = page(value.clone(), section);
        let tree = render(&domain, words());
        goldens.check(&tree);
    }
}
