//! Milestone row fixtures for the client's grouped lists (client/forge/parts.md, 3).
use jig_client::contract as c;
use jig_client_protocol_wire as j;
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
            w::MilestoneGoal { line: line_bytes(line), headline: headline.map(Into::into) },
        )
        .expect("fixture goals");
    }
    let part = w::MilestonePart { whole: whole.map(Wall::as_nanos), totals, goals: rows, more };
    let mut writer = Writer::new(usize::try_from(part.measure()).expect("fixture size"));
    part.encode(&mut writer).expect("fixture encoding");
    c::RowPart::Value(jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: writer.finish() })
}

/// The fixture's goal vocabulary: active, waiting and ended, with report, failure or cancellation.
pub fn line_bytes(line: &c::TaskLine) -> Box<[u8]> {
    let mut row = records::task_line_record().into_parts();
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
    let row = j::TaskLine::new(&j::CEILINGS, row).expect("fixture task line");
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("fixture size"));
    row.encode(&mut writer).expect("fixture encoding");
    writer.finish()
}

fn requester(from: c::Requester) -> j::Requester {
    match from {
        c::Requester::Task(value) => j::Requester::Task(
            j::RequesterTask::new(&j::CEILINGS, j::RequesterTaskParts { value }).expect("fixture requester"),
        ),
        c::Requester::Person(value) => j::Requester::Person(
            j::RequesterPerson::new(&j::CEILINGS, j::RequesterPersonParts { value }).expect("fixture requester"),
        ),
        c::Requester::Deployment { project } => j::Requester::Deployment(
            j::RequesterDeployment::new(&j::CEILINGS, j::RequesterDeploymentParts { project })
                .expect("fixture requester"),
        ),
    }
}
fn document(text: &c::DocumentText) -> j::DocumentText {
    j::DocumentText::new(&j::CEILINGS, j::DocumentTextParts { bytes: text.bytes.bytes().into(), cut: text.cut })
        .expect("fixture text")
}
#[expect(clippy::too_many_lines, reason = "one fixture assembles the chip's flat wire facts and their child positions")]
fn chip(line: &c::TaskLine) -> j::Chip {
    let j::Fact::Chip(root) = records::chip_root_fact() else { unreachable!("fixture root") };
    let mut root = root.into_parts();
    let mut fields = root.value.into_parts();
    fields.task = line.chip.task;
    fields.title =
        j::DocumentText::new(&j::CEILINGS, j::DocumentTextParts { bytes: line.chip.title.bytes().into(), cut: false })
            .expect("fixture title");
    let (phase, compact) = match &line.chip.details.phase {
        c::ChipPhase::Waiting { dependencies, more, holds } => (
            j::ChipPhase::Waiting(
                j::ChipPhaseWaiting::new(
                    &j::CEILINGS,
                    j::ChipPhaseWaitingParts { dependencies: dependencies.clone(), more: *more, holds: *holds },
                )
                .expect("fixture phase"),
            ),
            j::Phase::Waiting,
        ),
        c::ChipPhase::Active(stage) => (
            j::ChipPhase::Active(
                j::ChipPhaseActive::new(
                    &j::CEILINGS,
                    j::ChipPhaseActiveParts {
                        value: match stage {
                            c::RunStage::Idle => j::RunStage::Idle,
                            c::RunStage::Running { attempt } => j::RunStage::Running(
                                j::RunStageRunning::new(&j::CEILINGS, j::RunStageRunningParts { attempt: *attempt })
                                    .expect("fixture stage"),
                            ),
                            _ => panic!("fixture active stage"),
                        },
                    },
                )
                .expect("fixture phase"),
            ),
            j::Phase::Active,
        ),
        c::ChipPhase::Ended => (j::ChipPhase::Ended, j::Phase::Ended),
        _ => panic!("fixture goal phase"),
    };
    fields.phase = compact;
    root.value = j::ChipFields::new(&j::CEILINGS, fields).expect("fixture root");
    let mut details = records::wire_owner_chip_details_row().into_parts();
    let mut value = details.value.into_parts();
    value.ended_at = line.ended_at.map(Wall::as_nanos);
    value.ending = line.chip.details.details.ending.as_ref().map(|_| 3);
    details.value = j::ChipDetailsFields::new(&j::CEILINGS, value).expect("fixture details");
    let mut rows = vec![
        j::Fact::Chip(j::ChipRow::new(&j::CEILINGS, root).expect("fixture root")),
        j::Fact::ChipPhase(
            j::ChipPhaseRow::new(
                &j::CEILINGS,
                j::ChipPhaseRowParts {
                    parent: Some(0),
                    value: j::ChipPhaseOwnerFields::new(
                        &j::CEILINGS,
                        j::ChipPhaseOwnerFieldsParts { phase, details: 2 },
                    )
                    .expect("fixture phase"),
                },
            )
            .expect("fixture phase"),
        ),
        j::Fact::ChipDetails(Box::new(j::ChipDetailsRow::new(&j::CEILINGS, details).expect("fixture details"))),
    ];
    if let Some(ending) = &line.chip.details.details.ending {
        let value = match ending {
            c::ChipEnding::Done(c::ResultValue::Report(_)) => j::ChipEndingFields::Done(
                j::ChipEndingDoneFields::new(&j::CEILINGS, j::ChipEndingDoneFieldsParts { value: 4 })
                    .expect("fixture ending"),
            ),
            c::ChipEnding::Failed(text) => j::ChipEndingFields::Failed(
                j::ChipEndingFailedFields::new(&j::CEILINGS, j::ChipEndingFailedFieldsParts { value: document(text) })
                    .expect("fixture ending"),
            ),
            c::ChipEnding::Cancelled { reason, by, at, result: None } => j::ChipEndingFields::Cancelled(
                j::ChipEndingCancelledFields::new(
                    &j::CEILINGS,
                    j::ChipEndingCancelledFieldsParts {
                        reason: document(reason),
                        by: requester(*by),
                        at: at.as_nanos(),
                        result: None,
                    },
                )
                .expect("fixture ending"),
            ),
            _ => panic!("fixture ending"),
        };
        rows.push(j::Fact::ChipEnding(
            j::ChipEndingRow::new(&j::CEILINGS, j::ChipEndingRowParts { parent: Some(2), value })
                .expect("fixture ending"),
        ));
        if let c::ChipEnding::Done(c::ResultValue::Report(text)) = ending {
            rows.push(j::Fact::ResultValue(
                j::ResultValueRow::new(
                    &j::CEILINGS,
                    j::ResultValueRowParts {
                        parent: Some(3),
                        value: j::ResultValueFields::Report(
                            j::ResultValueReportFields::new(
                                &j::CEILINGS,
                                j::ResultValueReportFieldsParts { value: document(text) },
                            )
                            .expect("fixture result"),
                        ),
                    },
                )
                .expect("fixture result"),
            ));
        }
    }
    records::chip_from_rows(&rows)
}
