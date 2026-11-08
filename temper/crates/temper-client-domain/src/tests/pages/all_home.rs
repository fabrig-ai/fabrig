use crate::pages::{
    all_home::{self, Brief, BriefLine, Facts, Intro, Landed, Project, Spell},
    home::{Needs, Status},
    parts,
};
use skein_lib::{Decimal, List, Wall, bytes};
use temper_client_protocol_wire::v1 as w;

const FIRST: u32 = 1;
const SECOND: u32 = 2;
const THIRD: u32 = 3;
const FOURTH: u32 = 4;
// The contract keeps four briefs, so five exercises the first excluded row.
const FIFTH: u32 = 5;

fn name(project: u32) -> Box<[u8]> {
    bytes::concat(&[b"Project ", Decimal::of(u64::from(project)).as_bytes()])
}
fn shown(project: u32, landed: u64, seen: u64, previous: Option<u64>, last: Option<u64>) -> Project {
    let mut document = crate::tests::status::document(landed, seen, None).into_parts();
    document.project = project;
    document.previous = previous;
    document.last_landed = last;
    project_status(document)
}
fn project_status(document: w::ProjectStatusParts) -> Project {
    Project {
        project: document.project,
        name: name(document.project),
        status: Status::Shown(Box::new(w::ProjectStatus::new(&w::CEILINGS, document).unwrap())),
    }
}
fn unknown() -> Project {
    Project { project: THIRD, name: name(THIRD), status: Status::Unknown }
}
fn facts(projects: Box<[Project]>) -> Facts {
    Facts {
        projects,
        needs: Needs::Count { count: 2, cut: true },
        decided: Box::default(),
        showing: 3,
        away: Some(Spell { since: Wall::from_nanos(10), off: Wall::from_nanos(20) }),
    }
}
fn expected_intro(landed: Landed, previous: Option<u64>) -> Intro {
    Intro {
        last_looked: match previous {
            Some(at) => Some(Wall::from_nanos(at)),
            None => None,
        },
        away: Some(Spell { since: Wall::from_nanos(10), off: Wall::from_nanos(20) }),
        landed,
        needs: Needs::Count { count: 2, cut: true },
        brief: None,
    }
}
#[test]
fn intro_counts_only_shown_projects_since_their_looks() {
    let rows: [(&str, Box<[Project]>, Intro); 11] = [
        ("empty", Box::new([]), expected_intro(Landed::Nothing, None)),
        ("unknown", Box::new([unknown()]), expected_intro(Landed::Nothing, None)),
        ("nothing landed", Box::new([shown(FIRST, 0, 0, None, None)]), expected_intro(Landed::Nothing, None)),
        (
            "one project",
            Box::new([shown(FIRST, 4, 1, Some(50), Some(70))]),
            expected_intro(Landed::In { project: FIRST, name: name(FIRST), count: 3 }, Some(50)),
        ),
        (
            "two projects",
            Box::new([shown(FIRST, 2, 1, None, None), shown(SECOND, 3, 1, None, None)]),
            expected_intro(Landed::Across { count: 3, projects: 2 }, None),
        ),
        (
            "three projects",
            Box::new([
                shown(FIRST, 4, 1, Some(50), None),
                shown(SECOND, 3, 1, Some(60), None),
                shown(THIRD, 2, 1, Some(40), None),
            ]),
            expected_intro(Landed::Across { count: 6, projects: 3 }, Some(60)),
        ),
        (
            "newest last landing",
            Box::new([
                shown(FIRST, 4, 4, Some(80), Some(70)),
                shown(SECOND, 3, 3, Some(90), Some(75)),
                shown(THIRD, 0, 0, Some(100), Some(110)),
            ]),
            expected_intro(Landed::Last { at: Wall::from_nanos(75) }, Some(100)),
        ),
        ("no last landing", Box::new([shown(FIRST, 4, 4, None, None)]), expected_intro(Landed::Nothing, None)),
        (
            "seen ahead saturates",
            Box::new([shown(FIRST, 1, 2, None, Some(70))]),
            expected_intro(Landed::Last { at: Wall::from_nanos(70) }, None),
        ),
        (
            "unknown ignored and newest look",
            Box::new([shown(FIRST, 0, 0, Some(90), None), unknown(), shown(SECOND, 0, 0, Some(50), None)]),
            expected_intro(Landed::Nothing, Some(90)),
        ),
        (
            "total saturates",
            Box::new([shown(FIRST, u64::MAX, 0, None, None), shown(SECOND, 1, 0, None, None)]),
            expected_intro(Landed::Across { count: u64::MAX, projects: 2 }, None),
        ),
    ];
    for (case, projects, expected) in rows {
        assert_eq!(all_home::intro(&facts(projects)), expected, "{case}");
    }
}
fn with_brief(project: u32, landed: u64, seen: u64, previous: Option<u64>, at: u64) -> Project {
    let mut document = crate::tests::status::document(landed, seen, Some(landed)).into_parts();
    document.project = project;
    document.previous = previous;
    let mut brief = document.brief.unwrap().into_parts();
    brief.at = at;
    brief.lede = b"Ready! More follows.".as_slice().into();
    document.brief = Some(w::StatusBrief::new(&w::CEILINGS, brief).unwrap());
    project_status(document)
}
fn line(project: u32) -> BriefLine {
    BriefLine { project, name: name(project), sentence: b"Ready!".as_slice().into() }
}
fn joined(projects: &[u32], at: u64) -> Brief {
    let mut lines = List::with_capacity(u32::try_from(projects.len()).unwrap());
    for &project in projects {
        lines.push(line(project)).unwrap();
    }
    Brief { lines: lines.into_boxed(), at: Wall::from_nanos(at) }
}
type BriefRow = (&'static str, Box<[Project]>, Option<Brief>);
#[test]
fn joined_briefs_keep_the_four_newest_new_to_you() {
    let rows: [BriefRow; 10] = [
        ("none shown", Box::new([unknown()]), None),
        ("no brief", Box::new([shown(FIRST, 2, 1, Some(10), None)]), None),
        ("new landings", Box::new([with_brief(FIRST, 2, 1, Some(100), 50)]), Some(joined(&[FIRST], 50))),
        ("new landings without previous", Box::new([with_brief(FIRST, 2, 1, None, 50)]), Some(joined(&[FIRST], 50))),
        ("written since look", Box::new([with_brief(FIRST, 2, 2, Some(50), 60)]), Some(joined(&[FIRST], 60))),
        ("old brief", Box::new([with_brief(FIRST, 2, 2, Some(60), 50)]), None),
        ("at the look", Box::new([with_brief(FIRST, 2, 2, Some(50), 50)]), None),
        ("no previous or landings", Box::new([with_brief(FIRST, 2, 2, None, 50)]), None),
        (
            "newest four",
            Box::new([
                with_brief(FIRST, 2, 1, None, 10),
                with_brief(THIRD, 2, 1, None, 30),
                with_brief(FIFTH, 2, 1, None, 50),
                with_brief(SECOND, 2, 1, None, 20),
                with_brief(FOURTH, 2, 1, None, 40),
            ]),
            Some(joined(&[FIFTH, FOURTH, THIRD, SECOND], 50)),
        ),
        (
            "equal times keep both",
            Box::new([with_brief(FIRST, 2, 1, None, 50), with_brief(SECOND, 2, 1, None, 50)]),
            Some(joined(&[SECOND, FIRST], 50)),
        ),
    ];
    for (case, projects, expected) in rows {
        let facts = facts(projects);
        assert_eq!(all_home::brief(&facts), expected, "{case}");
        assert_eq!(all_home::intro(&facts).brief, expected, "intro: {case}");
    }
}
#[test]
fn first_sentence_ends_at_punctuation_then_space_or_end() {
    let rows: &[(&[u8], &[u8])] = &[
        (b"One. Two.", b"One."),
        (b"v1.2 is out. Then", b"v1.2 is out."),
        (b"No stop", b"No stop"),
        (b"Why? Because.", b"Why?"),
        (b"Ready! More.", b"Ready!"),
        (b"End.", b"End."),
        (b"End!", b"End!"),
        (b"End?", b"End?"),
        (b"No!space?here.Then", b"No!space?here.Then"),
        (b"Line.\nNext", b"Line.\nNext"),
        ("Café. Next".as_bytes(), "Café.".as_bytes()),
        (b"", b""),
    ];
    for (lede, expected) in rows {
        assert_eq!(all_home::first_sentence(lede), *expected, "{lede:?}");
    }
}
#[test]
fn across_parts_use_counts_and_fresh_briefs() {
    for (case, facts, expected) in [
        (
            "unknown needs",
            Facts { needs: Needs::Unknown, ..facts(Box::new([])) },
            parts::HomeFacts { across: true, projects: 0, brief: false, open: 0, showing: 3, to_begin: false },
        ),
        (
            "shown with brief",
            facts(Box::new([with_brief(FIRST, 2, 1, None, 50), unknown()])),
            parts::HomeFacts { across: true, projects: 2, brief: true, open: 2, showing: 3, to_begin: false },
        ),
    ] {
        assert_eq!(facts.parts_facts(), expected, "{case}");
    }
}
