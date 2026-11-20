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
fn move_fixture() -> (Domain, Finding) {
    let mut d = tiers::restoring(Some(b"release"));
    let mut row = blamed_red(1, FindingState::New);
    row.project = 5;
    row.signature = Some(FindingSignature { source: b"browser".as_slice().into(), ..signature() });
    let member = LoggedChange {
        task: 51,
        pull: Some(101),
        title: b"member".as_slice().into(),
        signed: Box::new([]),
        goal: Some(91),
        milestone: Some(81),
        person: Some(7),
        goal_title: None,
        draft: None,
        fixes: Box::new([]),
    };
    d.landing_logs
        .insert(
            REPO,
            LandingLog {
                repository: REPO,
                landings: Box::new([
                    LoggedLanding {
                        commit: [0xab; 32],
                        branch: b"release".as_slice().into(),
                        at: Wall::EPOCH,
                        tiers: Box::new([]),
                        changes: Box::new([
                            member.clone(),
                            LoggedChange { task: 52, milestone: Some(99), ..member.clone() },
                            LoggedChange { task: 53, goal: None, ..member.clone() },
                            LoggedChange { task: 54, goal: Some(92), milestone: None, person: None, ..member.clone() },
                        ]),
                    },
                    LoggedLanding {
                        commit: [0xcd; 32],
                        branch: b"release".as_slice().into(),
                        at: Wall::EPOCH,
                        tiers: Box::new([]),
                        changes: Box::new([]),
                    },
                    LoggedLanding {
                        commit: [0xef; 32],
                        branch: b"main".as_slice().into(),
                        at: Wall::EPOCH,
                        tiers: Box::new([]),
                        changes: Box::new([member]),
                    },
                ]),
            },
        )
        .unwrap();
    (d, row)
}
fn move_call(landing: Option<skein_forge::api::Commit>) -> TriageFinding {
    TriageFinding {
        project: 5,
        number: 1,
        act: Triage::Blame(BlameAct { landing, why: b"better evidence".as_slice().into() }),
    }
}
#[test]
fn blame_replaces_only_the_blame_in_every_state_with_logged_goals_in_member_order() {
    for state in [FindingState::New, FindingState::Routed, FindingState::Backlog, FindingState::Dropped] {
        let (mut d, mut expected) = move_fixture();
        expected.state = state;
        expected.goal = Some(99);
        let mut out = Queue::uncapped();
        d.commit_finding(&LIMITS, FindingWrite { row: expected.clone(), erase: None }, &mut out);
        for (by, commit, goals) in [
            (
                Filer::Task(40),
                Some([0xab; 32]),
                [
                    BlamedGoal { goal: 91, milestone: Some(81), person: Some(7) },
                    BlamedGoal { goal: 92, milestone: None, person: None },
                ]
                .as_slice()
                .into(),
            ),
            (Filer::Party(7), Some([0xcd; 32]), Box::default()),
            (Filer::Task(41), None, Box::default()),
        ] {
            let write = d.prepare_triage_finding(by, move_call(commit)).unwrap();
            expected.blame = Some(FindingBlame {
                strength: BlameStrength::Moved,
                moved: Some(BlameMove { by, why: b"better evidence".as_slice().into() }),
                landings: match commit {
                    Some(commit) => Box::new([BlamedLanding { commit, goals }]),
                    None => Box::default(),
                },
            });
            assert_eq!(write.row, expected, "{state:?}: every other fact is unchanged");
            let mut out = Queue::uncapped();
            d.commit_finding(&LIMITS, write, &mut out);
            assert_eq!(d.finding(5, 1), Some(&expected));
            let mut saved = false;
            for request in &out {
                saved |= matches!(request, Request::Save { record: Stored::Finding(row), .. } if row == &expected);
            }
            assert!(saved);
        }
    }
}
#[test]
fn blame_refuses_non_landed_or_cleared_findings_and_commits_not_logged_on_the_tier_branch() {
    for (kind, blame, cleared, expected) in [
        (FindingKind::Qa, true, false, "finding 5/1 has no blame: it is no red filed under landed"),
        (FindingKind::Red, false, false, "finding 5/1 has no blame: it is no red filed under landed"),
        (FindingKind::Red, true, true, "finding 5/1 is cleared"),
    ] {
        let (mut d, mut row) = move_fixture();
        row.kind = kind;
        if !blame {
            row.blame = None;
        }
        if cleared {
            row.cleared = Some(FindingCleared { run: 8, at: Wall::EPOCH });
        }
        d.commit_finding(&LIMITS, FindingWrite { row: row.clone(), erase: None }, &mut Queue::uncapped());
        for commit in [None, Some([0xab; 32])] {
            let refusal = d.prepare_triage_finding(Filer::Party(7), move_call(commit)).unwrap_err();
            assert_eq!(refusal.words(5, 1, b"").as_ref(), expected.as_bytes());
            assert_eq!(d.finding(5, 1), Some(&row));
        }
    }
    let (mut d, row) = move_fixture();
    d.commit_finding(&LIMITS, FindingWrite { row: row.clone(), erase: None }, &mut Queue::uncapped());
    for (commit, short) in [([0xef; 32], "efefefefefef"), ([0xff; 32], "ffffffffffff")] {
        let refusal = d.prepare_triage_finding(Filer::Party(7), move_call(Some(commit))).unwrap_err();
        assert_eq!(
            refusal.words(5, 1, b"").as_ref(),
            format!("landing {short} is not one the landing log holds on release").as_bytes()
        );
        assert_eq!(d.finding(5, 1), Some(&row));
    }
    assert!(d.landing_logs.remove(&REPO).is_some());
    assert_eq!(
        d.prepare_triage_finding(Filer::Party(7), move_call(Some([0xab; 32]))).unwrap_err().words(5, 1, b"").as_ref(),
        b"landing abababababab is not one the landing log holds on release"
    );
}
#[test]
fn blame_uses_todays_followed_branch_and_none_needs_no_tier() {
    for (follows, expected) in [
        (None, b"finding 5/1's tier browser has no followed branch".as_slice()),
        (Some(Follows::Goals), b"finding 5/1's tier browser has no followed branch"),
        (
            Some(Follows::Branch(b"later".as_slice().into())),
            b"landing abababababab is not one the landing log holds on later",
        ),
    ] {
        let (mut d, row) = move_fixture();
        let repositories = &mut d.landing.projects.get_mut(&5).unwrap().1;
        let tiers = &mut repositories[0].tiers;
        match follows {
            Some(follows) => tiers[0].follows = follows,
            None => *tiers = Box::new([]),
        }
        d.commit_finding(&LIMITS, FindingWrite { row: row.clone(), erase: None }, &mut Queue::uncapped());
        let why = d.prepare_triage_finding(Filer::Party(7), move_call(Some([0xab; 32]))).unwrap_err();
        assert_eq!(why.words(5, 1, b"").as_ref(), expected);
        let write = d.prepare_triage_finding(Filer::Party(7), move_call(None)).unwrap();
        assert!(write.row.blame.unwrap().landings.is_empty());
    }
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
        .prepare_triage_finding(
            Filer::Party(7),
            TriageFinding { project: 1, number: 2, act: Triage::Drop(b"duplicate".as_slice().into()) },
        )
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

fn blamed_red(number: u32, state: FindingState) -> Finding {
    let d = domain();
    let mut row =
        d.prepare_file_finding(&LIMITS, Wall::from_nanos(u64::from(number)), Filer::Party(7), call()).unwrap().row;
    row.number = number;
    row.kind = FindingKind::Red;
    row.state = state;
    row.blame = Some(FindingBlame { moved: None, strength: BlameStrength::Unblamed, landings: Box::new([]) });
    row
}
#[test]
fn uncleared_red_requires_red_kind_blame_and_no_clearing() {
    let mut row = blamed_red(1, FindingState::Dropped);
    assert!(row.uncleared_red());
    row.blame = None;
    assert!(!row.uncleared_red());
    row.blame = Some(FindingBlame { moved: None, strength: BlameStrength::Owned, landings: Box::new([]) });
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
