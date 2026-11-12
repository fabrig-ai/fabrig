//! The owner chain and bounded retained findings (domain/areas.md, 4; findings.md, 9).
use super::*;
use crate::settings::{Area, AreaOwner, ProjectAreas};

#[test]
fn the_owner_chain_uses_standing_then_triager_then_project_owners() {
    let areas = ProjectAreas {
        areas: Box::new([Area {
            name: b"tests".as_slice().into(),
            words: b"C tests".as_slice().into(),
            owner: AreaOwner::Task(40),
        }]),
    };
    let standing = [
        (AreaOwner::Task(40), Holder::Task(40)),
        (AreaOwner::Charter(14), Holder::Task(47)),
        (AreaOwner::Party(7), Holder::Party(7)),
    ];
    assert_eq!(owner(Some(b"tests"), &areas, Some(AreaOwner::Charter(14)), &standing), Holder::Task(40));
    assert_eq!(owner(Some(b"tests"), &areas, Some(AreaOwner::Charter(14)), &standing[1..]), Holder::Task(47));
    assert_eq!(owner(None, &areas, None, &standing), Holder::Role(0));
    assert_eq!(owner(Some(b"removed"), &areas, Some(AreaOwner::Party(7)), &standing), Holder::Party(7));
    assert_eq!(owner(Some(b"tests"), &areas, Some(AreaOwner::Party(8)), &[]), Holder::Role(0));
    assert_eq!(owner(None, &areas, Some(AreaOwner::Role(9)), &[]), Holder::Role(9));
}
fn call() -> FileFinding {
    FileFinding {
        project: 1,
        kind: FindingKind::Qa,
        severity: Severity::High,
        area: None,
        title: b"test fails".as_slice().into(),
        body: b"reproduce".as_slice().into(),
    }
}
#[test]
fn a_full_project_erases_only_its_oldest_ended_finding_and_restore_keeps_the_bound() {
    let mut d = domain();
    let limits = Limits { findings_per_project: 3, ..LIMITS };
    let mut out = Queue::uncapped();
    for number in 1..=3 {
        let write = d.prepare_file_finding(&limits, Wall::from_nanos(number), Filer::Party(7), call()).unwrap();
        d.commit_finding(write, &mut out);
    }
    assert_eq!(
        d.prepare_file_finding(&limits, Wall::EPOCH, Filer::Task(40), call()).unwrap_err(),
        FindingRefusal::Full(3)
    );
    let ended = d
        .prepare_triage_finding(TriageFinding {
            project: 1,
            number: 2,
            act: Triage::Drop(b"duplicate".as_slice().into()),
        })
        .unwrap();
    d.commit_finding(ended, &mut out);
    let write = d.prepare_file_finding(&limits, Wall::EPOCH, Filer::Task(40), call()).unwrap();
    assert_eq!(write.erase, Some(2));
    assert_eq!(write.row.number, 4);
    d.commit_finding(write, &mut out);
    assert!(d.finding(1, 1).is_some());
    assert!(d.finding(1, 2).is_none());
    let mut restored = domain_with_limits(&limits);
    restored.restore_begin(Wall::EPOCH);
    let mut saves = skein_lib::List::with_capacity(3);
    for (_, row) in d.findings(1) {
        saves.push(row.clone()).unwrap();
    }
    for row in &saves {
        step(
            &mut restored,
            &env().with(limits),
            Event::Restore { record: Stored::Finding(row.clone()) },
            &mut Queue::uncapped(),
        );
    }
    assert_eq!(restored.finding(1, 4), d.finding(1, 4));
    let extra = Finding { number: 5, ..saves.as_slice().first().unwrap().clone() };
    step(&mut restored, &env().with(limits), Event::Restore { record: Stored::Finding(extra) }, &mut Queue::uncapped());
    assert!(restored.finding(1, 5).is_none());
    assert!(restored.fit_restored(&limits).is_err());
    let mut marked = false;
    for request in &out {
        if let Request::Marked { tasks } = request {
            marked |= tasks.contains(&findings_subject(1, Some(FindingState::New), true));
        }
    }
    assert!(marked);
    let mut erased = Queue::uncapped();
    crate::findings::erase_project(&mut d, 1, &mut erased);
    assert_eq!(d.findings(1).count(), 0);
}
