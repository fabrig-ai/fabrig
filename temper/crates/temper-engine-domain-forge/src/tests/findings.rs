//! The owner chain and bounded retained findings (domain/areas.md, 4; findings.md, 9).
use super::*;
use crate::findings::findings_filter;
use crate::settings::{Area, AreaOwner, ProjectAreas};
use skein_lib::List;

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
#[test]
fn findings_filter_keys_are_bounded_invertible_and_apart_from_other_subjects() {
    let goal = FindingsFilter::Goal(820);
    let milestone = FindingsFilter::Milestone(812);
    assert_eq!(findings_subject(3, goal), Some((1_u64 << 62_u32) | (1 << 41_u32) | 0x0334));
    assert_eq!(findings_subject(3, milestone), Some((1_u64 << 62_u32) | (1 << 42_u32) | 0x032c));
    for project in [1, u32::MAX] {
        let mut keys = List::uncapped();
        for filter in [
            goal,
            milestone,
            FindingsFilter::Goal(0),
            FindingsFilter::Milestone(0),
            FindingsFilter::Goal((1_u64 << 41_u32) - 1),
            FindingsFilter::Milestone((1_u64 << 41_u32) - 1),
        ] {
            let key = findings_subject(project, filter).unwrap();
            assert_eq!(findings_filter(key, project), Some(filter));
            assert_eq!(findings_filter(key, 2), Some(filter), "task numbers are deployment-wide");
            assert!(!keys.as_slice().contains(&key));
            keys.push(key).unwrap();
        }
        for (state, code) in [
            (None, 0_u64),
            (Some(FindingState::New), 1),
            (Some(FindingState::Routed), 2),
            (Some(FindingState::Backlog), 3),
            (Some(FindingState::Dropped), 4),
        ] {
            for mine in [false, true] {
                let filter = FindingsFilter::Triage { state, mine };
                let key = findings_subject(project, filter).unwrap();
                assert_eq!(
                    key,
                    (1_u64 << 59_u32) | (code << 33_u32) | (u64::from(mine) << 32_u32) | u64::from(project)
                );
                assert_eq!(findings_filter(key, project), Some(filter));
                assert_eq!(findings_filter(key, 2), None);
                assert!(!keys.as_slice().contains(&key));
                keys.push(key).unwrap();
            }
        }
        for other in [
            0,
            820,
            812,
            (1_u64 << 63_u32) | 0x0334,
            AWAY_SUBJECT,
            DEVICES_SUBJECT,
            DISMISSED_SUBJECT,
            APPROVALS_SUBJECT,
            decided_subject(project),
            drafts_subject(project),
            status_subject(project),
            changes_subject(project),
            landings_subject(project, false),
            landings_subject(project, true),
            follows_subject(project),
            temper_engine_domain_trials::TRIALS_SUBJECT,
            temper_engine_domain_ways::subject(project),
        ] {
            assert!(!keys.as_slice().contains(&other), "collision with {other:x}");
            assert_eq!(findings_filter(other, project), None);
        }
    }
    for number in [1_u64 << 41_u32, (1_u64 << 41_u32) + 1, u64::MAX] {
        assert_eq!(findings_subject(3, FindingsFilter::Goal(number)), None);
        assert_eq!(findings_subject(3, FindingsFilter::Milestone(number)), None);
    }
    assert_eq!(findings_filter(u64::MAX, 3), None);
    assert_eq!(findings_filter((1_u64 << 62_u32) | (1 << 41_u32) | (1 << 42_u32), 3), None);
}

#[test]
fn finding_rows_keep_routing_drops_and_clearing_with_or_without_a_signature() {
    let mut row = blamed_red(1, FindingState::Dropped);
    row.goal = Some(820);
    row.reason = Some(b"duplicate".as_slice().into());
    row.cleared = Some(FindingCleared { run: 17, at: Wall::from_nanos(19) });
    for source in [None, Some(b"slow".as_slice())] {
        row.signature = match source {
            Some(source) => Some(FindingSignature {
                repository: REPO,
                source: source.into(),
                ids: Box::new([b"a::one".as_slice().into()]),
                first: Box::new([]),
                many: false,
            }),
            None => None,
        };
        let got = row.row(Holder::Party(4));
        assert_eq!(got.goal, Some(820));
        assert_eq!(got.reason.as_deref(), Some(b"duplicate".as_slice()));
        assert_eq!(
            got.cleared,
            Some(FindingClearing { tier: source.unwrap_or_default().into(), run: 17, at: Wall::from_nanos(19) })
        );
    }
    row.cleared = None;
    assert_eq!(row.row(Holder::Role(0)).cleared, None);
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
    let mut saves = List::with_capacity(3);
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
            marked |= tasks.contains(
                &findings_subject(1, FindingsFilter::Triage { state: Some(FindingState::New), mine: true }).unwrap(),
            );
        }
    }
    assert!(marked);
    let mut erased = Queue::uncapped();
    crate::findings::erase_project(&mut d, 1, &mut erased);
    assert_eq!(d.findings(1).count(), 0);
}

fn blamed_red(number: u32, state: FindingState) -> Finding {
    let d = domain();
    let mut row =
        d.prepare_file_finding(&LIMITS, Wall::from_nanos(u64::from(number)), Filer::Party(7), call()).unwrap().row;
    row.number = number;
    row.kind = FindingKind::Red;
    row.state = state;
    row.blame = Some(FindingBlame { strength: BlameStrength::Unblamed, landings: Box::new([]) });
    row
}
#[test]
fn reds_owed_counts_each_projects_uncleared_finding_once_in_every_state() {
    let mut d = domain();
    for (number, project, milestone, state, kind, cleared, strength, landings) in [
        (1, 2, 812, FindingState::New, FindingKind::Red, false, BlameStrength::Owned, 1),
        (2, 2, 812, FindingState::New, FindingKind::Red, false, BlameStrength::Suspected, 2),
        (3, 2, 812, FindingState::Dropped, FindingKind::Red, false, BlameStrength::Owned, 1),
        (4, 2, 812, FindingState::New, FindingKind::Red, true, BlameStrength::Owned, 1),
        (5, 2, 812, FindingState::New, FindingKind::Flake, false, BlameStrength::Owned, 1),
        (6, 2, 813, FindingState::New, FindingKind::Red, false, BlameStrength::Owned, 1),
        (7, 3, 812, FindingState::New, FindingKind::Red, false, BlameStrength::Owned, 1),
    ] {
        let mut row = blamed_red(number, state);
        row.project = project;
        row.kind = kind;
        row.cleared = cleared.then_some(FindingCleared { run: 17, at: Wall::from_nanos(19) });
        let mut blamed = List::with_capacity(2);
        for landing in 0..landings {
            blamed
                .push(BlamedLanding {
                    commit: [landing; 32],
                    goals: Box::new([BlamedGoal { goal: 820, milestone: Some(milestone), person: Some(4) }]),
                })
                .expect("at most two blamed landings");
        }
        row.blame = Some(FindingBlame { strength, landings: blamed.into_boxed() });
        d.commit_finding(&LIMITS, FindingWrite { row, erase: None }, &mut Queue::uncapped());
    }
    assert_eq!(d.reds_owed(2, 812), 3);
    assert_eq!(d.reds_owed(2, 813), 1);
    assert_eq!(d.reds_owed(2, 999), 0);
}
fn blame(goal: u64, milestone: u64) -> FindingBlame {
    FindingBlame {
        strength: BlameStrength::Owned,
        landings: Box::new([BlamedLanding {
            commit: [1; 32],
            goals: Box::new([BlamedGoal { goal, milestone: Some(milestone), person: None }]),
        }]),
    }
}

#[test]
fn a_finding_write_marks_the_previous_written_and_evicted_blames() {
    let mut d = domain();
    for (number, goal, milestone) in [(1, 820, 812), (2, 821, 813)] {
        let mut row = blamed_red(number, FindingState::Dropped);
        row.blame = Some(blame(goal, milestone));
        row.cleared = Some(FindingCleared { run: 17, at: Wall::from_nanos(19) });
        d.commit_finding(&LIMITS, FindingWrite { row, erase: None }, &mut Queue::uncapped());
    }
    let mut row = d.finding(1, 1).unwrap().clone();
    row.blame = Some(blame(822, 814));
    let mut out = Queue::uncapped();
    d.commit_finding(&LIMITS, FindingWrite { row, erase: Some(2) }, &mut out);
    assert!(d.finding(1, 2).is_none());
    for (goal, milestone) in [(820, 812), (821, 813), (822, 814)] {
        for filter in [FindingsFilter::Goal(goal), FindingsFilter::Milestone(milestone)] {
            let subject = findings_subject(1, filter).unwrap();
            let mut marks = 0u32;
            for request in &out {
                if let Request::Marked { tasks } = request {
                    assert!(tasks.len() <= skein_lib::usize_of(MARKED));
                    if tasks.contains(&subject) {
                        marks = marks.saturating_add(1);
                    }
                }
            }
            assert_eq!(marks, 1, "each old or new filtered subject: {filter:?}");
        }
        let mut marked = false;
        for request in &out {
            marked |= *request == Request::MilestoneMarked { task: milestone };
        }
        assert!(marked, "parts of milestone {milestone}");
    }
}

#[test]
fn uncleared_red_requires_red_kind_blame_and_no_clearing() {
    let mut row = blamed_red(1, FindingState::Dropped);
    assert!(row.uncleared_red());
    row.blame = None;
    assert!(!row.uncleared_red());
    row.blame = Some(FindingBlame { strength: BlameStrength::Owned, landings: Box::new([]) });
    row.cleared = Some(FindingCleared { run: 8, at: Wall::from_nanos(9) });
    assert!(!row.uncleared_red());
    row.cleared = None;
    row.kind = FindingKind::Flake;
    assert!(!row.uncleared_red());
}
#[test]
fn a_full_project_keeps_uncleared_reds_and_erases_the_oldest_erasable_by_time() {
    let limits = Limits { findings_per_project: 3, ..LIMITS };
    let mut d = domain_with_limits(&limits);
    let red = blamed_red(1, FindingState::Dropped);
    let routed = Finding { blame: None, at: Wall::from_nanos(3), ..blamed_red(2, FindingState::Routed) };
    let older_routed = Finding { blame: None, at: Wall::from_nanos(2), ..blamed_red(3, FindingState::Routed) };
    let mut out = Queue::uncapped();
    for row in [red.clone(), routed, older_routed] {
        d.commit_finding(&limits, FindingWrite { row, erase: None }, &mut out);
    }
    let write = d.prepare_file_finding(&limits, Wall::from_nanos(4), Filer::Party(7), call()).unwrap();
    assert_eq!(write.erase, Some(3));
    d.commit_finding(&limits, write, &mut out);
    assert_eq!(d.finding(1, 1), Some(&red));
    assert!(d.finding(1, 2).is_some());
    assert!(d.finding(1, 3).is_none());
    assert!(d.finding(1, 4).is_some());
}
#[test]
fn only_uncleared_reds_and_new_or_backlog_findings_leave_no_room() {
    let limits = Limits { findings_per_project: 6, ..LIMITS };
    let mut d = domain_with_limits(&limits);
    for (number, state, blame) in [
        (1, FindingState::New, true),
        (2, FindingState::Backlog, true),
        (3, FindingState::Routed, true),
        (4, FindingState::Dropped, true),
        (5, FindingState::New, false),
        (6, FindingState::Backlog, false),
    ] {
        let mut row = blamed_red(number, state);
        if !blame {
            row.blame = None;
        }
        d.commit_finding(&limits, FindingWrite { row, erase: None }, &mut Queue::uncapped());
    }
    let why = d.prepare_file_finding(&limits, Wall::EPOCH, Filer::Party(7), call()).unwrap_err();
    assert_eq!(why, FindingRefusal::Full(2));
    assert_eq!(why.words(1, 0, b"").as_ref(), b"the project's findings are full: 2 new");
}
#[test]
fn cleared_new_reds_and_ended_reds_without_blame_can_be_erased() {
    let limits = Limits { findings_per_project: 2, ..LIMITS };
    for (state, cleared) in [(FindingState::New, true), (FindingState::Routed, false), (FindingState::Dropped, false)] {
        let mut d = domain_with_limits(&limits);
        let mut row = blamed_red(1, state);
        if cleared {
            row.cleared = Some(FindingCleared { run: 8, at: Wall::from_nanos(9) });
        } else {
            row.blame = None;
        }
        d.commit_finding(&limits, FindingWrite { row, erase: None }, &mut Queue::uncapped());
        d.commit_finding(
            &limits,
            FindingWrite { row: blamed_red(2, FindingState::Routed), erase: None },
            &mut Queue::uncapped(),
        );
        let write = d.prepare_file_finding(&limits, Wall::EPOCH, Filer::Party(7), call()).unwrap();
        assert_eq!(write.erase, Some(1), "{state:?}, cleared {cleared}");
        d.commit_finding(&limits, write, &mut Queue::uncapped());
        assert!(d.finding(1, 1).is_none());
        assert!(d.finding(1, 2).is_some());
    }
}

fn signature() -> FindingSignature {
    FindingSignature {
        repository: REPO,
        source: b"check".as_slice().into(),
        ids: Box::new([b"test a::t x".as_slice().into()]),
        first: Box::default(),
        many: false,
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
