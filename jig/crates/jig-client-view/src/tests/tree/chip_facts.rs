#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests inspect public trees and select supplied watch requests"
)]
use crate::JigBindings;
use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
pub(in crate::tests) use crate::tests::kit::list;
use crate::tests::tree::cards::{CLIENT, text, words};
use crate::{Request, state::address::Address};
use jig_client::contract::*;
use skein_ui::{Builder, Tree};

fn chip(phase: OwnerChipPhase, details: OwnerChipDetails) -> Chip {
    Chip {
        task: 8,
        title: DocumentText { bytes: text(b"partial title"), cut: true },
        executor: Executor::Agent { charter: 1 },
        phase: phase.compact(),
        details: Box::new(ChipPhase { phase, details: Box::new(details) }),
    }
}
fn brief(value: &Chip) -> Tree {
    let client = Fixture::new(&CLIENT);
    let limits = crate::tests::render_limits(&CLIENT, words()).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.client.text);
    crate::tree::cards::chip::chip(&mut b, &client.view(), value, words(), &mut bindings);
    let tree = b.finish();
    assert!(tree.len() <= crate::chip_nodes(&CLIENT, &(CLIENT).client).unwrap());
    tree
}
#[test]
fn brief_exact_chips_show_title_loss_and_distinct_phase_without_inventing_ending_clock() {
    let mut goldens = Goldens::new("tree/chip_facts/brief_exact_chips");
    let cases = [
        OwnerChipPhase::Waiting { dependencies: list([u64::MAX]), more: true, holds: true },
        OwnerChipPhase::Person { address: PersonAddress::Role(u32::MAX), taken_by: Some(u64::MAX), since: None },
        OwnerChipPhase::Closing(ClosingStage::Run { attempt: u64::MAX }),
        OwnerChipPhase::Held { reason: HoldCategory::Uncertain { entry: u64::MAX }, revision: u64::MAX },
        OwnerChipPhase::Active(RunStage::Running { attempt: u64::MAX }),
        OwnerChipPhase::Ended,
    ];
    for phase in cases {
        let mut details = crate::tests::tree::fragments::chip::details();
        match &phase {
            OwnerChipPhase::Person { address, .. } => {
                details.executor = OwnerExecutor::Person(*address);
            }
            OwnerChipPhase::Active(_) => {
                details.executor = OwnerExecutor::Agent { charter: u32::MAX };
            }
            _ => {
                details.executor = OwnerExecutor::Procedure { connector: u16::MAX, code: u32::MAX };
            }
        }
        let tree = brief(&chip(phase, details));
        goldens.check(&tree);
    }
}
#[test]
fn task_overview_expands_whole_metadata_but_nested_verdict_chips_stay_brief() {
    let mut goldens = Goldens::new("tree/chip_facts/task_overview_expands");
    let mut inner_details = crate::tests::tree::fragments::chip::details();
    inner_details.ending =
        Some(OwnerChipEnding::Failed(DocumentText::complete(text(b"nested ending remains carried"))));
    let inner = chip(OwnerChipPhase::Ended, inner_details);
    let mut details = crate::tests::tree::fragments::chip::details();
    details.resources_cut = true;
    details.resources_observed = false;
    details.ended_at = None;
    details.ending = Some(OwnerChipEnding::Done(ResultValue::Verdict {
        verdict: u32::MAX,
        words: DocumentText { bytes: text(b"whole outer result"), cut: true },
        fields: list([]),
        tasks: list([inner.clone()]),
        proposal: None,
    }));
    let mut doc = crate::tests::tree::cards_objects::task_doc(&[]);
    doc.chip = chip(OwnerChipPhase::Ended, details);
    let (mut client, out) = crate::tests::state::page_capacity::entered(Address::task(8));
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
    crate::tests::tree::pages::deliver(&mut client, watch, Delivery::Task(Box::new(doc)));
    let tree = crate::tests::state::page_capacity::render(&client, words());
    goldens.check(&tree);
    assert_eq!(inner.phase, Phase::Ended);
    assert_eq!(inner.title.whole(), None);
    let bound = crate::tests::render_limits(&CLIENT, words()).unwrap();
    assert!(tree.nodes().iter().all(|node| crate::tests::text_bytes(node) <= bound.text_per_node));
}
