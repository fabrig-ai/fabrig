//! Findings' bounds and records (temper/docs/design/client/forge/records.md).
use super::{filled, v1};
use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer};

fn file(title: Box<[u8]>) -> v1::FileFindingParts {
    v1::FileFindingParts {
        project: 3,
        kind: v1::FindingKind::Red,
        severity: v1::Severity::Medium,
        area: Some(b"tests".as_slice().into()),
        title,
        body: filled(4096),
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one findings entrance scenario covers requests, filters and their word bounds")]
fn finding_requests_subjects_and_reads_round_trip_with_bounded_words() {
    let limits = &v1::CEILINGS;
    let mut requests = List::uncapped();
    for request in [
        v1::Request::FileFinding(v1::FileFinding::new(limits, file(b"Flaky boot".as_slice().into())).unwrap()),
        v1::Request::FileFinding(
            v1::FileFinding::new(limits, v1::FileFindingParts { area: Some(filled(32)), ..file(filled(200)) }).unwrap(),
        ),
        v1::Request::HandFinding(
            v1::HandFinding::new(
                limits,
                v1::HandFindingParts { project: 3, number: 7, area: filled(32), why: filled(200) },
            )
            .unwrap(),
        ),
    ] {
        requests.push(request).unwrap();
    }
    for act in [
        v1::Triage::Route(v1::FindingRoute::new(limits, v1::FindingRouteParts { task: u64::MAX }).unwrap()),
        v1::Triage::Backlog,
        v1::Triage::Drop(v1::FindingDrop::new(limits, v1::FindingDropParts { reason: filled(400) }).unwrap()),
    ] {
        requests
            .push(v1::Request::TriageFinding(
                v1::TriageFinding::new(limits, v1::TriageFindingParts { project: 3, number: 7, act }).unwrap(),
            ))
            .unwrap();
    }
    for request in requests.into_boxed() {
        let record = v1::Requesting::new(limits, v1::RequestingParts { request }).unwrap();
        assert_eq!(crate::decode_requesting(&crate::requesting(&record), limits), Some(record));
    }
    let empty = v1::Requesting::new(
        limits,
        v1::RequestingParts {
            request: v1::Request::FileFinding(v1::FileFinding::new(limits, file(Box::default())).unwrap()),
        },
    )
    .unwrap();
    assert!(crate::decode_requesting(&crate::requesting(&empty), limits).is_none());
    for request in [
        v1::Request::HandFinding(
            v1::HandFinding::new(
                limits,
                v1::HandFindingParts { project: 3, number: 7, area: filled(32), why: Box::default() },
            )
            .unwrap(),
        ),
        v1::Request::TriageFinding(
            v1::TriageFinding::new(
                limits,
                v1::TriageFindingParts {
                    project: 3,
                    number: 7,
                    act: v1::Triage::Drop(
                        v1::FindingDrop::new(limits, v1::FindingDropParts { reason: Box::default() }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ] {
        let record = v1::Requesting::new(limits, v1::RequestingParts { request }).unwrap();
        assert!(crate::decode_requesting(&crate::requesting(&record), limits).is_none());
    }
    // FileFinding's project, kind, severity, no area, title length and bytes, empty body.
    let mut writer = Writer::new(216);
    writer.put(&3_u32.to_be_bytes()).unwrap();
    writer.put(&[2, 1, 0]).unwrap();
    writer.put(&201_u32.to_be_bytes()).unwrap();
    writer.put(&filled(201)).unwrap();
    writer.put(&0_u32.to_be_bytes()).unwrap();
    assert_eq!(
        v1::FileFinding::decode(limits, &mut Reader::new(&writer.finish())),
        Err(v1::Problem { path: v1::Path::FileFindingTitle, reason: skein_codec::Reason::Bound })
    );
    for (parts, path) in [
        (file(filled(201)), v1::Path::FileFindingTitle),
        (v1::FileFindingParts { area: Some(filled(33)), ..file(filled(200)) }, v1::Path::FileFindingArea),
        (v1::FileFindingParts { body: filled(4097), ..file(filled(200)) }, v1::Path::FileFindingBody),
    ] {
        assert_eq!(v1::FileFinding::new(limits, parts), Err(v1::Problem { path, reason: skein_codec::Reason::Bound }));
    }
    v1::HandFinding::new(limits, v1::HandFindingParts { project: 3, number: 7, area: filled(33), why: filled(200) })
        .unwrap_err();
    v1::HandFinding::new(limits, v1::HandFindingParts { project: 3, number: 7, area: filled(32), why: filled(201) })
        .unwrap_err();
    v1::FindingDrop::new(limits, v1::FindingDropParts { reason: filled(401) }).unwrap_err();
    for state in [
        None,
        Some(v1::FindingState::New),
        Some(v1::FindingState::Routed),
        Some(v1::FindingState::Backlog),
        Some(v1::FindingState::Dropped),
    ] {
        for mine in [false, true] {
            let watching = v1::Watching::new(
                limits,
                v1::WatchingParts {
                    watch: v1::Watch::Findings(
                        v1::FindingsSubject::new(
                            limits,
                            v1::FindingsSubjectParts { project: 3, state: state.clone(), mine },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap();
            assert_eq!(crate::decode_watching(&crate::watching(&watching), limits), Some(watching));
        }
    }
    let reading = v1::Reading::new(
        limits,
        v1::ReadingParts {
            read: v1::Read::Finding(
                v1::FindingRead::new(limits, v1::FindingReadParts { project: 3, number: 7 }).unwrap(),
            ),
        },
    )
    .unwrap();
    assert_eq!(crate::decode_reading(&crate::reading(&reading), limits), Some(reading));
}

fn holder() -> v1::Holder {
    v1::Holder::Role(v1::HolderRole::new(&v1::CEILINGS, v1::HolderRoleParts { number: u32::MAX }).unwrap())
}
fn row() -> v1::FindingRow {
    v1::FindingRow::new(
        &v1::CEILINGS,
        v1::FindingRowParts {
            number: u32::MAX,
            kind: v1::FindingKind::Feedback,
            severity: v1::Severity::High,
            area: Some(filled(32)),
            state: v1::FindingState::Dropped,
            title: filled(200),
            holder: holder(),
            at: u64::MAX,
            why: Some(filled(200)),
        },
    )
    .unwrap()
}
fn owner() -> v1::AreaOwner {
    v1::AreaOwner::Charter(v1::AreaCharter::new(&v1::CEILINGS, v1::AreaCharterParts { number: u32::MAX }).unwrap())
}
#[test]
fn findings_documents_and_details_round_trip_at_their_caps() {
    let limits = &v1::CEILINGS;
    let area = v1::AreaRow::new(
        limits,
        v1::AreaRowParts { name: filled(32), words: filled(480), owner: owner(), holder: Some(holder()) },
    )
    .unwrap();
    let mut rows = List::with_capacity(128);
    for _ in 0_u32..128 {
        rows.push(row()).unwrap();
    }
    let mut areas = List::with_capacity(16);
    for _ in 0_u32..16 {
        areas.push(area.clone()).unwrap();
    }
    let document = v1::Findings::new(
        limits,
        v1::FindingsParts { project: 3, rows, areas, triager: Some(owner()), holder: holder(), cut: true },
    )
    .unwrap();
    let bytes = crate::findings(&document);
    assert_eq!(crate::decode_findings(&bytes, limits), Some(document.clone()));
    let mut trailing = Writer::new(bytes.len().checked_add(1).unwrap());
    trailing.put(&bytes).unwrap();
    trailing.put(&[0]).unwrap();
    assert!(crate::decode_findings(&trailing.finish(), limits).is_none());
    let mut parts = document.clone().into_parts();
    let mut rows = List::with_capacity(129);
    for _ in 0_u32..129 {
        rows.push(row()).unwrap();
    }
    parts.rows = rows;
    assert_eq!(
        v1::Findings::new(limits, parts),
        Err(v1::Problem { path: v1::Path::FindingsRows, reason: skein_codec::Reason::Bound })
    );
    let mut parts = document.into_parts();
    let mut areas = List::with_capacity(17);
    for _ in 0_u32..17 {
        areas.push(area.clone()).unwrap();
    }
    parts.areas = areas;
    assert_eq!(
        v1::Findings::new(limits, parts),
        Err(v1::Problem { path: v1::Path::FindingsAreas, reason: skein_codec::Reason::Bound })
    );
    for (name, words) in [(33, 480), (32, 481)] {
        v1::AreaRow::new(
            limits,
            v1::AreaRowParts { name: filled(name), words: filled(words), owner: owner(), holder: None },
        )
        .unwrap_err();
    }
    for (count, path) in [(129_u32, v1::Path::FindingsRows), (17, v1::Path::FindingsAreas)] {
        let mut writer = Writer::new(if count == 17 { 14 } else { 10 });
        writer.put(&u16::try_from(crate::CLIENT_VERSION).unwrap().to_be_bytes()).unwrap();
        writer.put(&3_u32.to_be_bytes()).unwrap();
        if count == 17 {
            writer.put(&0_u32.to_be_bytes()).unwrap();
        }
        writer.put(&count.to_be_bytes()).unwrap();
        let problem = v1::Findings::decode(limits, &mut Reader::new(&writer.finish())).unwrap_err();
        assert_eq!(problem, v1::Problem { path, reason: skein_codec::Reason::Bound });
    }
    for result in [
        v1::FindingResult::NoFinding,
        v1::FindingResult::Finding(
            v1::FindingDetail::new(
                limits,
                v1::FindingDetailParts {
                    row: row(),
                    body: filled(4096),
                    filer: v1::Filer::Task(
                        v1::HolderTask::new(limits, v1::HolderTaskParts { number: u64::MAX }).unwrap(),
                    ),
                    goal: Some(u64::MAX),
                    reason: Some(filled(400)),
                },
            )
            .unwrap(),
        ),
    ] {
        let answer = v1::FindingAnswer::new(limits, v1::FindingAnswerParts { result }).unwrap();
        assert_eq!(crate::decode_finding_answer(&crate::finding_answer(&answer), limits), Some(answer));
    }
}

#[test]
fn finding_outcomes_and_refusals_round_trip_with_all_area_names() {
    let limits = &v1::CEILINGS;
    let mut areas = List::with_capacity(16);
    for _ in 0_u32..16 {
        areas.push(filled(32)).unwrap();
    }
    let mut outcomes = List::uncapped();
    for outcome in [
        v1::Outcome::Filed(v1::Filed::new(limits, v1::FiledParts { number: u32::MAX }).unwrap()),
        v1::Outcome::Handed,
        v1::Outcome::Triaged,
    ] {
        outcomes.push(outcome).unwrap();
    }
    for reason in [
        v1::FindingRefusal::NoArea(v1::FindingAreas::new(limits, v1::FindingAreasParts { areas }).unwrap()),
        v1::FindingRefusal::Full(v1::FindingsFull::new(limits, v1::FindingsFullParts { count: u32::MAX }).unwrap()),
        v1::FindingRefusal::NotYours(
            v1::FindingNotYours::new(limits, v1::FindingNotYoursParts { holder: holder() }).unwrap(),
        ),
        v1::FindingRefusal::NoFinding,
        v1::FindingRefusal::NoGoal,
        v1::FindingRefusal::Ended,
        v1::FindingRefusal::Words(
            v1::FindingWords::new(limits, v1::FindingWordsParts { field: b"title".as_slice().into() }).unwrap(),
        ),
    ] {
        outcomes
            .push(v1::Outcome::FindingRefused(
                v1::FindingRefused::new(limits, v1::FindingRefusedParts { reason }).unwrap(),
            ))
            .unwrap();
    }
    for outcome in outcomes.into_boxed() {
        let answer = v1::Answer::new(limits, v1::AnswerParts { outcome }).unwrap();
        assert_eq!(v1::Answer::decode(limits, &mut Reader::new(&crate::answer(&answer))), Ok(answer));
    }
    let mut areas = List::with_capacity(17);
    for _ in 0_u32..17 {
        areas.push(filled(32)).unwrap();
    }
    v1::FindingAreas::new(limits, v1::FindingAreasParts { areas }).unwrap_err();
    let mut areas = List::with_capacity(1);
    areas.push(filled(33)).unwrap();
    v1::FindingAreas::new(limits, v1::FindingAreasParts { areas }).unwrap_err();
}
