//! Findings' bounds and records (temper/docs/design/client/forge/records.md).
use super::{filled, v1};
use crate::v1::Record;
use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer};

fn file(title: Box<[u8]>) -> v1::FileFinding {
    v1::FileFinding {
        project: 3,
        kind: v1::FindingKind::Red,
        severity: v1::Severity::Medium,
        area: Some(b"tests".as_slice().into()),
        title,
        body: filled(4096),
    }
}

#[test]
fn finding_requests_subjects_and_reads_round_trip_with_bounded_words() {
    let limits = &v1::CEILINGS;
    let mut requests = List::uncapped();
    for request in [
        v1::Request::FileFinding(file(b"Flaky boot".as_slice().into())),
        v1::Request::FileFinding(v1::FileFinding { area: Some(filled(32)), ..file(filled(200)) }),
        v1::Request::HandFinding(v1::HandFinding { project: 3, number: 7, area: filled(32), why: filled(200) }),
    ] {
        requests.push(request).unwrap();
    }
    for act in [
        v1::Triage::Route(v1::FindingRoute { task: u64::MAX }),
        v1::Triage::Backlog,
        v1::Triage::Drop(v1::FindingDrop { reason: filled(400) }),
    ] {
        requests.push(v1::Request::TriageFinding(v1::TriageFinding { project: 3, number: 7, act })).unwrap();
    }
    for request in requests.into_boxed() {
        let record = v1::Requesting { request };
        assert_eq!(crate::decode_requesting(&crate::requesting(&record), limits), Some(record));
    }
    let empty = v1::Requesting { request: v1::Request::FileFinding(file(Box::default())) };
    assert!(crate::decode_requesting(&crate::requesting(&empty), limits).is_none());
    for request in [
        v1::Request::HandFinding(v1::HandFinding { project: 3, number: 7, area: filled(32), why: Box::default() }),
        v1::Request::TriageFinding(v1::TriageFinding {
            project: 3,
            number: 7,
            act: v1::Triage::Drop(v1::FindingDrop { reason: Box::default() }),
        }),
    ] {
        let record = v1::Requesting { request };
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
        Err(v1::Problem { path: v1::paths::FILE_FINDING_TITLE, reason: skein_codec::Reason::Bound })
    );
    for (parts, path) in [
        (file(filled(201)), v1::paths::FILE_FINDING_TITLE),
        (v1::FileFinding { area: Some(filled(33)), ..file(filled(200)) }, v1::paths::FILE_FINDING_AREA),
        (v1::FileFinding { body: filled(4097), ..file(filled(200)) }, v1::paths::FILE_FINDING_BODY),
    ] {
        assert_eq!(
            {
                let value = parts;
                match value.encode(limits) {
                    Ok(_) => Ok(value),
                    Err(problem) => Err(problem),
                }
            },
            Err(v1::Problem { path, reason: skein_codec::Reason::Bound })
        );
    }
    (v1::HandFinding { project: 3, number: 7, area: filled(33), why: filled(200) }).encode(limits).unwrap_err();
    (v1::HandFinding { project: 3, number: 7, area: filled(32), why: filled(201) }).encode(limits).unwrap_err();
    (v1::FindingDrop { reason: filled(401) }).encode(limits).unwrap_err();
    for state in [
        None,
        Some(v1::FindingState::New),
        Some(v1::FindingState::Routed),
        Some(v1::FindingState::Backlog),
        Some(v1::FindingState::Dropped),
    ] {
        for mine in [false, true] {
            let watching = v1::Watching {
                watch: v1::Watch::Findings(v1::FindingsSubject { project: 3, state: state.clone(), mine }),
            };
            assert_eq!(crate::decode_watching(&crate::watching(&watching), limits), Some(watching));
        }
    }
    let reading = v1::Reading { read: v1::Read::Finding(v1::FindingRead { project: 3, number: 7 }) };
    assert_eq!(crate::decode_reading(&crate::reading(&reading), limits), Some(reading));
}

fn holder() -> v1::Holder {
    v1::Holder::Role(v1::HolderRole { number: u32::MAX })
}
fn row() -> v1::FindingRow {
    v1::FindingRow {
        number: u32::MAX,
        kind: v1::FindingKind::Feedback,
        severity: v1::Severity::High,
        area: Some(filled(32)),
        state: v1::FindingState::Dropped,
        title: filled(200),
        holder: holder(),
        at: u64::MAX,
        why: Some(filled(200)),
        hander: Some(v1::Filer::Party(v1::HolderParty { number: u64::MAX })),
    }
}
fn owner() -> v1::AreaOwner {
    v1::AreaOwner::Charter(v1::AreaCharter { number: u32::MAX })
}
#[test]
fn findings_documents_and_details_round_trip_at_their_caps() {
    let limits = &v1::CEILINGS;
    let area = v1::AreaRow { name: filled(32), words: filled(480), owner: owner(), holder: Some(holder()) };
    let mut rows = List::with_capacity(128);
    for _ in 0_u32..128 {
        rows.push(row()).unwrap();
    }
    let mut areas = List::with_capacity(16);
    for _ in 0_u32..16 {
        areas.push(area.clone()).unwrap();
    }
    let document = v1::Findings { project: 3, rows, areas, triager: Some(owner()), holder: holder(), cut: true };
    let bytes = crate::findings(&document);
    assert_eq!(crate::decode_findings(&bytes, limits), Some(document.clone()));
    let mut trailing = Writer::new(bytes.len().checked_add(1).unwrap());
    trailing.put(&bytes).unwrap();
    trailing.put(&[0]).unwrap();
    assert!(crate::decode_findings(&trailing.finish(), limits).is_none());
    let mut parts = document.clone();
    let mut rows = List::with_capacity(129);
    for _ in 0_u32..129 {
        rows.push(row()).unwrap();
    }
    parts.rows = rows;
    assert_eq!(
        {
            let value = parts;
            match value.encode(limits) {
                Ok(_) => Ok(value),
                Err(problem) => Err(problem),
            }
        },
        Err(v1::Problem { path: v1::paths::FINDINGS_ROWS, reason: skein_codec::Reason::Bound })
    );
    let mut parts = document;
    let mut areas = List::with_capacity(17);
    for _ in 0_u32..17 {
        areas.push(area.clone()).unwrap();
    }
    parts.areas = areas;
    assert_eq!(
        {
            let value = parts;
            match value.encode(limits) {
                Ok(_) => Ok(value),
                Err(problem) => Err(problem),
            }
        },
        Err(v1::Problem { path: v1::paths::FINDINGS_AREAS, reason: skein_codec::Reason::Bound })
    );
    for (name, words) in [(33, 480), (32, 481)] {
        (v1::AreaRow { name: filled(name), words: filled(words), owner: owner(), holder: None })
            .encode(limits)
            .unwrap_err();
    }
    for (count, path) in [(129_u32, v1::paths::FINDINGS_ROWS), (17, v1::paths::FINDINGS_AREAS)] {
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
        v1::FindingResult::Finding(v1::FindingDetail {
            row: row(),
            body: filled(4096),
            filer: v1::Filer::Task(v1::HolderTask { number: u64::MAX }),
            goal: Some(u64::MAX),
            reason: Some(filled(400)),
        }),
    ] {
        let answer = v1::FindingAnswer { result };
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
    for outcome in [v1::Outcome::Filed(v1::Filed { number: u32::MAX }), v1::Outcome::Handed, v1::Outcome::Triaged] {
        outcomes.push(outcome).unwrap();
    }
    for reason in [
        v1::FindingRefusal::NoArea(v1::FindingAreas { areas }),
        v1::FindingRefusal::Full(v1::FindingsFull { count: u32::MAX }),
        v1::FindingRefusal::NotYours(v1::FindingNotYours { holder: holder() }),
        v1::FindingRefusal::NoFinding,
        v1::FindingRefusal::NoGoal,
        v1::FindingRefusal::Ended,
        v1::FindingRefusal::Words(v1::FindingWords { field: b"title".as_slice().into() }),
    ] {
        outcomes.push(v1::Outcome::FindingRefused(v1::FindingRefused { reason })).unwrap();
    }
    for outcome in outcomes.into_boxed() {
        let answer = v1::Answer { outcome };
        assert_eq!(v1::Answer::decode(limits, &mut Reader::new(&crate::answer(&answer))), Ok(answer));
    }
    let mut areas = List::with_capacity(17);
    for _ in 0_u32..17 {
        areas.push(filled(32)).unwrap();
    }
    (v1::FindingAreas { areas }).encode(limits).unwrap_err();
    let mut areas = List::with_capacity(1);
    areas.push(filled(33)).unwrap();
    (v1::FindingAreas { areas }).encode(limits).unwrap_err();
}
