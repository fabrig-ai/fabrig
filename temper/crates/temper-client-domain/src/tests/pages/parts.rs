use crate::pages::{
    home,
    parts::{self, HomeFacts, HomeParts, NeedsFacts, NeedsParts},
};

#[test]
fn home_parts_each_follow_their_own_facts() {
    let empty = HomeFacts::default();
    let ask = HomeParts { ask: true, ..HomeParts::default() };
    for (case, facts, expected) in [
        ("empty", empty, ask),
        ("brief", HomeFacts { brief: true, ..empty }, HomeParts { brief: true, ..ask }),
        ("needs", HomeFacts { open: 1, ..empty }, HomeParts { needs: true, ..ask }),
        ("decided", HomeFacts { showing: 1, ..empty }, HomeParts { decided: true, ..ask }),
        ("follows on project home", HomeFacts { follows: 1, ..empty }, ask),
        ("follows across", HomeFacts { across: true, follows: 1, ..empty }, HomeParts { follows: true, ..ask }),
        ("no follows across", HomeFacts { across: true, follows: 0, ..empty }, ask),
        ("projects on project home", HomeFacts { projects: 2, ..empty }, ask),
        ("one across", HomeFacts { across: true, projects: 1, ..empty }, ask),
        ("two across", HomeFacts { across: true, projects: 2, ..empty }, HomeParts { projects: true, ..ask }),
        ("to begin", HomeFacts { to_begin: true, brief: true, open: 1, showing: 1, ..empty }, HomeParts::default()),
        (
            "to begin keeps projects",
            HomeFacts { to_begin: true, across: true, projects: 2, brief: true, open: 1, showing: 1, ..empty },
            HomeParts { projects: true, ..HomeParts::default() },
        ),
        (
            "all",
            HomeFacts { across: true, projects: 2, brief: true, open: 1, showing: 1, ..empty },
            HomeParts { brief: true, needs: true, decided: true, projects: true, ask: true, follows: false },
        ),
    ] {
        assert_eq!(parts::home(&facts), expected, "{case}");
    }
}
#[test]
fn needs_parts_wait_for_known_and_keep_decided_independent() {
    for (case, facts, expected) in [
        (
            "unknown empty",
            NeedsFacts { known: false, open: 0, showing: 0, only: 0, ..NeedsFacts::default() },
            NeedsParts { list: false, done: false, decided: false, only: false, ..NeedsParts::default() },
        ),
        (
            "unknown cards",
            NeedsFacts { known: false, open: 1, showing: 0, only: 0, ..NeedsFacts::default() },
            NeedsParts { list: false, done: false, decided: false, only: false, ..NeedsParts::default() },
        ),
        (
            "known empty",
            NeedsFacts { known: true, open: 0, showing: 0, only: 0, ..NeedsFacts::default() },
            NeedsParts { list: false, done: true, decided: false, only: false, ..NeedsParts::default() },
        ),
        (
            "known cards",
            NeedsFacts { known: true, open: 1, showing: 0, only: 0, ..NeedsFacts::default() },
            NeedsParts { list: true, done: false, decided: false, only: false, ..NeedsParts::default() },
        ),
        (
            "only unknown",
            NeedsFacts { known: false, open: 0, showing: 0, only: 1, ..NeedsFacts::default() },
            NeedsParts { list: false, done: false, decided: false, only: false, ..NeedsParts::default() },
        ),
        (
            "only done",
            NeedsFacts { known: true, open: 0, showing: 0, only: 1, ..NeedsFacts::default() },
            NeedsParts { list: false, done: true, decided: false, only: true, ..NeedsParts::default() },
        ),
        (
            "only with cards",
            NeedsFacts { known: true, open: 1, showing: 0, only: 1, ..NeedsFacts::default() },
            NeedsParts { list: true, done: false, decided: false, only: true, ..NeedsParts::default() },
        ),
        (
            "decided unknown",
            NeedsFacts { known: false, open: 0, showing: 1, only: 0, ..NeedsFacts::default() },
            NeedsParts { list: false, done: false, decided: true, only: false, ..NeedsParts::default() },
        ),
        (
            "decided with cards",
            NeedsFacts { known: true, open: 1, showing: 1, only: 0, ..NeedsFacts::default() },
            NeedsParts { list: true, done: false, decided: true, only: false, ..NeedsParts::default() },
        ),
    ] {
        assert_eq!(parts::needs(&facts), expected, "{case}");
    }
}
#[test]
fn project_parts_keep_to_begin_and_away_counts() {
    for (case, delivered, needs, status, away, expected) in [
        ("unknown", false, home::Needs::Unknown, home::Status::Unknown, None, HomeFacts::default()),
        (
            "to begin",
            true,
            home::Needs::Count { count: 2, cut: true },
            home::Status::Unknown,
            None,
            HomeFacts { open: 2, to_begin: true, ..HomeFacts::default() },
        ),
        (
            "brief and away",
            false,
            home::Needs::Count { count: 2, cut: false },
            home::Status::Shown(Box::new(crate::tests::status::document(2, 1, Some(2)))),
            Some(crate::pages::decided::Shown {
                more: false,
                showing: 3,
                rows: Box::default(),
                expires: skein_lib::Wall::EPOCH,
            }),
            HomeFacts { open: 2, brief: true, showing: 3, ..HomeFacts::default() },
        ),
    ] {
        let facts = home::Facts {
            configured: false,
            repository: home::Repository::None,
            needs,
            delivered,
            skip: home::Skip::NotSkipped,
            skip_refused: false,
            status,
            decided: away,
            spell: None,
        };
        assert_eq!(facts.parts_facts(), expected, "{case}");
    }
}

#[test]
fn needs_counts_all_decisions_with_its_project_filter() {
    use super::all_home::{Across, closes, task_reads};
    use crate::{Address, address::Filter, forge, pages::all_home};
    use skein_lib::List;
    use skein_lib::Wall;
    // Nine showing rows exceed the seven drawn; three belong to the filtered project.
    const NINE: u32 = 9;
    const THREE: u32 = 3;
    const FIRST: u32 = 1;
    const SECOND: u32 = 2;
    const THIRD: u32 = 3;
    let rows: [(&str, Filter, &[u32], u32); 2] = [
        ("across", Filter::ALL, &[FIRST, SECOND, THIRD], NINE),
        ("one project", Filter { project: Some(SECOND), ..Filter::ALL }, &[SECOND], THREE),
    ];
    for (case, filter, read, expected) in rows {
        let mut fixture = Across::new(&[FIRST, SECOND, THIRD], Address::Inbox { filter });
        for project in [FIRST, SECOND, THIRD] {
            assert_eq!(
                fixture.watches(forge::status::watch(project, &crate::tests::fixtures::limits().forge).unwrap()),
                0,
                "{case}: decisions only"
            );
            assert_eq!(
                fixture.watches(forge::decided::watch(project, &crate::tests::fixtures::limits().forge).unwrap()),
                usize::from(read.contains(&project)),
                "{case}"
            );
        }
        for &project in read {
            let mut values = List::with_capacity(THREE);
            for number in 1..=THREE {
                values.push(crate::tests::decided::decision(project * THREE + number, u64::from(number))).unwrap();
            }
            let out = fixture.decisions(project, values.as_slice());
            assert_eq!(task_reads(&out).len(), usize::try_from(THREE).unwrap(), "{case}");
        }
        let facts = parts::needs_facts(&fixture.root, Wall::EPOCH);
        assert_eq!(facts.showing, expected, "{case}");
        assert!(parts::needs(&facts).decided, "{case}: independent of inbox delivery");
        let drawn = all_home::decided(&fixture.root, Wall::EPOCH);
        assert_eq!(drawn.len(), usize::try_from(expected.min(7)).unwrap(), "{case}");
        fixture.ack();
        let mut watches = List::uncapped();
        for &project in read {
            watches
                .push(fixture.watch(forge::decided::watch(project, &crate::tests::fixtures::limits().forge).unwrap()))
                .unwrap();
        }
        let out = fixture.go(Address::Accounts);
        let closed = closes(&out);
        for watch in &watches {
            assert!(closed.contains(watch), "{case}: leave");
        }
        assert_eq!(parts::needs_facts(&fixture.root, Wall::EPOCH).showing, 0, "{case}: closed");
    }
}

#[test]
fn tell_part_needs_a_project_and_its_findings_read() {
    for (project, findings, expected) in
        [(false, false, false), (true, false, false), (false, true, false), (true, true, true)]
    {
        let facts = NeedsFacts { project, findings, ..NeedsFacts::default() };
        assert_eq!(parts::needs(&facts).tell, expected, "{facts:?}");
    }
}
