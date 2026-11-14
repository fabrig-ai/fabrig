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
        d.commit_finding(&limits, write, &mut out);
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
    d.commit_finding(&limits, ended, &mut out);
    let write = d.prepare_file_finding(&limits, Wall::EPOCH, Filer::Task(40), call()).unwrap();
    assert_eq!(write.erase, Some(2));
    assert_eq!(write.row.number, 4);
    d.commit_finding(&limits, write, &mut out);
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

fn signature() -> FindingSignature {
    FindingSignature {
        repository: REPO,
        source: b"check".as_slice().into(),
        ids: Box::new([b"test a::t x".as_slice().into()]),
        first: Box::default(),
    }
}
fn flake(d: &Domain, limits: &Limits, project: u32, signature: FindingSignature) -> Result<CheckFlake, FindingRefusal> {
    d.prepare_check_flake(
        limits,
        Wall::EPOCH,
        Filer::Task(51),
        project,
        signature,
        b"delivery flake".as_slice().into(),
        b"log".as_slice().into(),
    )
}
#[test]
fn check_flakes_file_with_a_signature_and_count_only_equal_open_flakes_in_the_project() {
    let mut d = domain();
    let limits = Limits { findings_per_project: 1, ..LIMITS };
    let CheckFlake::Filed(write) = flake(&d, &limits, 1, signature()).unwrap() else { panic!("new filing") };
    assert_eq!(write.erase, None);
    assert_eq!(write.row.kind, FindingKind::Flake);
    assert_eq!(write.row.severity, Severity::Low);
    assert_eq!(write.row.area, None);
    assert_eq!(write.row.state, FindingState::New);
    assert_eq!(write.row.title.as_ref(), b"delivery flake");
    assert_eq!(write.row.body.as_ref(), b"log");
    assert_eq!(write.row.filer, Filer::Task(51));
    assert_eq!(write.row.at, Wall::EPOCH);
    assert_eq!(write.row.signature, Some(signature()));
    assert_eq!(write.row.seen, Some(1));
    let original = write.row.clone();
    assert!(d.finding(1, 1).is_none(), "preparation writes nothing");
    d.commit_finding(&limits, write, &mut Queue::uncapped());
    let mut different = signature();
    different.ids = Box::new([b"another key".as_slice().into()]);
    assert!(matches!(flake(&d, &limits, 1, different), Err(FindingRefusal::Full(1))));
    for state in [FindingState::New, FindingState::Backlog, FindingState::Routed] {
        d.finding_rows_mut(&limits).get_mut(&(1, 1)).unwrap().state = state;
        let CheckFlake::Counted(write) = d
            .prepare_check_flake(
                &limits,
                Wall::from_nanos(99),
                Filer::Party(4),
                1,
                signature(),
                b"ignored title".as_slice().into(),
                b"ignored body".as_slice().into(),
            )
            .unwrap()
        else {
            panic!("counted at the bound")
        };
        assert_eq!(write.erase, None);
        assert_eq!(write.row, Finding { seen: Some(2), state, ..original.clone() });
    }
    assert!(matches!(flake(&d, &limits, 2, signature()).unwrap(), CheckFlake::Filed(_)), "project-local");
    let mut other = signature();
    other.repository.repository += 1;
    assert!(matches!(flake(&d, &limits, 1, other).unwrap(), CheckFlake::Filed(_)), "repository differs");
    d.finding_rows_mut(&limits).get_mut(&(1, 1)).unwrap().kind = FindingKind::Red;
    assert!(matches!(flake(&d, &limits, 1, signature()).unwrap(), CheckFlake::Filed(_)), "kind differs");
    d.finding_rows_mut(&limits).get_mut(&(1, 1)).unwrap().kind = FindingKind::Flake;
    d.finding_rows_mut(&limits).get_mut(&(1, 1)).unwrap().state = FindingState::Dropped;
    let CheckFlake::Filed(write) = flake(&d, &limits, 1, signature()).unwrap() else {
        panic!("dropped match files anew")
    };
    assert_eq!(write.erase, Some(1));
    assert_eq!(write.row.number, 2);
    assert_eq!(write.row.seen, Some(1));
}
