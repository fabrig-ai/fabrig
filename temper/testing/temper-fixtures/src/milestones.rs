//! Milestone row fixtures for the client's grouped lists (client/forge/parts.md, 3).
use jig_client::contract as c;
use jig_client_protocol_wire as j;
use jig_client_protocol_wire::Record as _;
use jig_client_test_records as records;
use skein_lib::{List, Wall, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub fn part(
    whole: Option<Wall>,
    totals: w::MilestoneTotals,
    goals: &[(c::TaskLine, Option<&[u8]>)],
    more: u32,
) -> c::RowPart {
    let mut rows = List::with_capacity(u32::try_from(goals.len()).expect("fixture goals"));
    for (line, headline) in goals {
        rows.push(
            w::MilestoneGoal::new(
                &w::CEILINGS,
                w::MilestoneGoalParts { line: line_bytes(line), headline: headline.map(Into::into) },
            )
            .expect("fixture goal"),
        )
        .expect("fixture goals");
    }
    let part = w::MilestonePart::new(
        &w::CEILINGS,
        w::MilestonePartParts { whole: whole.map(Wall::as_nanos), totals, goals: rows, more },
    )
    .expect("fixture milestone part");
    let mut writer = Writer::new(usize::try_from(part.measure()).expect("fixture size"));
    part.encode(&mut writer).expect("fixture encoding");
    c::RowPart::Value(jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: writer.finish() })
}

/// The fixture's goal vocabulary: active, waiting and ended, with report, failure or cancellation.
pub fn line_bytes(line: &c::TaskLine) -> Box<[u8]> {
    let mut row = records::task_line_record();
    row.parts = List::with_capacity(0);
    row.chip = chip(line);
    row.project = line.project;
    row.requester = requester(line.requester);
    row.tracked = line.tracked;
    row.label = line.label;
    row.changed = line.changed.as_nanos();
    row.last_activity = line.last_activity.as_nanos();
    row.ended_at = line.ended_at.map(Wall::as_nanos);
    row.delegates_done = line.delegates_done;
    row.delegates_total = line.delegates_total;
    let row = row;
    let mut writer = Writer::new(
        usize::try_from(row.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("fixture size"),
    );
    row.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("fixture encoding");
    writer.finish()
}

fn requester(from: c::Requester) -> j::Requester {
    match from {
        c::Requester::Task(value) => j::Requester::Task(j::RequesterTask { value }),
        c::Requester::Person(value) => j::Requester::Person(j::RequesterPerson { value }),
        c::Requester::Deployment { project } => j::Requester::Deployment(j::RequesterDeployment { project }),
    }
}
fn document(text: &c::DocumentText) -> j::DocumentText {
    j::DocumentText { bytes: text.bytes.bytes().into(), cut: text.cut }
}
fn chip(line: &c::TaskLine) -> j::Chip {
    let j::Fact::Chip(root) = records::chip_root_fact() else { unreachable!("fixture root") };
    let mut root = root;
    let mut fields = root.value;
    fields.task = line.chip.task;
    fields.title = j::DocumentText { bytes: line.chip.title.bytes().into(), cut: false };
    let (phase, compact) = match &line.chip.details.phase {
        c::ChipPhase::Waiting { dependencies, more, holds } => (
            j::ChipPhase::Waiting(j::ChipPhaseWaiting {
                dependencies: dependencies.clone(),
                more: *more,
                holds: *holds,
            }),
            j::Phase::Waiting,
        ),
        c::ChipPhase::Active(stage) => (
            j::ChipPhase::Active(j::ChipPhaseActive {
                value: match stage {
                    c::RunStage::Idle => j::RunStage::Idle,
                    c::RunStage::Running { attempt } => j::RunStage::Running(j::RunStageRunning { attempt: *attempt }),
                    _ => panic!("fixture active stage"),
                },
            }),
            j::Phase::Active,
        ),
        c::ChipPhase::Ended => (j::ChipPhase::Ended, j::Phase::Ended),
        _ => panic!("fixture goal phase"),
    };
    fields.phase = compact;
    root.value = fields;
    let mut details = records::wire_owner_chip_details_row();
    let mut value = details.value;
    value.ended_at = line.ended_at.map(Wall::as_nanos);
    value.ending = line.chip.details.details.ending.as_ref().map(|_| 3);
    details.value = value;
    let mut rows = vec![
        j::Fact::Chip(root),
        j::Fact::ChipPhase(j::ChipPhaseRow { parent: Some(0), value: j::ChipPhaseOwnerFields { phase, details: 2 } }),
        j::Fact::ChipDetails(Box::new(details)),
    ];
    if let Some(ending) = &line.chip.details.details.ending {
        let value = match ending {
            c::ChipEnding::Done(c::ResultValue::Report(_)) => {
                j::ChipEndingFields::Done(j::ChipEndingDoneFields { value: 4 })
            }
            c::ChipEnding::Failed(text) => {
                j::ChipEndingFields::Failed(j::ChipEndingFailedFields { value: document(text) })
            }
            c::ChipEnding::Cancelled { reason, by, at, result: None } => {
                j::ChipEndingFields::Cancelled(j::ChipEndingCancelledFields {
                    reason: document(reason),
                    by: requester(*by),
                    at: at.as_nanos(),
                    result: None,
                })
            }
            _ => panic!("fixture ending"),
        };
        rows.push(j::Fact::ChipEnding(j::ChipEndingRow { parent: Some(2), value }));
        if let c::ChipEnding::Done(c::ResultValue::Report(text)) = ending {
            rows.push(j::Fact::ResultValue(j::ResultValueRow {
                parent: Some(3),
                value: j::ResultValueFields::Report(j::ResultValueReportFields { value: document(text) }),
            }));
        }
    }
    records::chip_from_rows(&rows)
}
