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
        ("projects on project home", HomeFacts { projects: 2, ..empty }, ask),
        ("one across", HomeFacts { across: true, projects: 1, ..empty }, ask),
        ("two across", HomeFacts { across: true, projects: 2, ..empty }, HomeParts { projects: true, ..ask }),
        ("to begin", HomeFacts { to_begin: true, brief: true, open: 1, showing: 1, ..empty }, HomeParts::default()),
        (
            "to begin keeps projects",
            HomeFacts { to_begin: true, across: true, projects: 2, brief: true, open: 1, showing: 1 },
            HomeParts { projects: true, ..HomeParts::default() },
        ),
        (
            "all",
            HomeFacts { across: true, projects: 2, brief: true, open: 1, showing: 1, ..empty },
            HomeParts { brief: true, needs: true, decided: true, projects: true, ask: true },
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
            NeedsFacts { known: false, open: 0, showing: 0 },
            NeedsParts { list: false, done: false, decided: false },
        ),
        (
            "unknown cards",
            NeedsFacts { known: false, open: 1, showing: 0 },
            NeedsParts { list: false, done: false, decided: false },
        ),
        (
            "known empty",
            NeedsFacts { known: true, open: 0, showing: 0 },
            NeedsParts { list: false, done: true, decided: false },
        ),
        (
            "known cards",
            NeedsFacts { known: true, open: 1, showing: 0 },
            NeedsParts { list: true, done: false, decided: false },
        ),
        (
            "decided unknown",
            NeedsFacts { known: false, open: 0, showing: 1 },
            NeedsParts { list: false, done: false, decided: true },
        ),
        (
            "decided with cards",
            NeedsFacts { known: true, open: 1, showing: 1 },
            NeedsParts { list: true, done: false, decided: true },
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
            Some(home::AwayForm {
                span: None,
                landed: None,
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
            away,
        };
        assert_eq!(facts.parts_facts(), expected, "{case}");
    }
}
