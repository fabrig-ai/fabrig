mod launchpad;

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

use crate::{
    Action, Address, Domain, Event, Place, Request,
    address::Filter,
    forge,
    pages::{self, Act, needs},
    tests::fixtures as f,
};
use jig_client::{Down, Key, Readout, Told, Up, Watchable, contract as c};
use skein_lib::{Queue, Token};
use temper_client_protocol_wire as wire;

// ACROSS projects each take two watches; the session and rail need headroom too.
const READ_HANDLES: u32 = 128;
const THREE: u32 = 3;
const DAY: u64 = 86_400_000_000_000;
const SEVEN: usize = 7;
const NINE: u32 = 9;
#[expect(clippy::partial_pub_fields, reason = "the shared fixture exposes the domain; watch bookkeeping stays local")]
pub(super) struct Across {
    pub root: Domain,
    limits: crate::Limits,
    me: Token,
    opened: List<(Token, Watchable)>,
}
impl Across {
    pub fn new(projects: &[u32], address: Address) -> Self {
        let mut fixture = Self::waiting(address);
        fixture.frame(projects);
        fixture
    }
    fn waiting(address: Address) -> Self {
        let mut limits = f::limits();
        limits.client.domain.handles = READ_HANDLES;
        limits.client.domain.watches = READ_HANDLES;
        limits.client.domain.reads = READ_HANDLES;
        limits.client.domain.projects = all_home::ACROSS + 1;
        limits.client.domain.requests.requests = READ_HANDLES;
        limits.client.domain.text = 512;
        limits.client.domain.long_text = 512;
        limits.client.domain.turn_text = 512;
        let mut root = crate::unlinked(&limits, f::SEED);
        let me = f::start(&mut root, List::uncapped());
        let mut fixture = Self { root, limits, me, opened: List::uncapped() };
        fixture.go(address);
        fixture
    }
    fn event(&mut self, event: Event) -> Queue<Request> {
        let mut out = Queue::uncapped();
        crate::step(&mut self.root, &f::env(&self.limits), event, &mut out);
        for request in &out {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } },
            } = request
            {
                self.opened.push((*watch, subject.clone())).unwrap();
            }
        }
        out
    }
    pub(super) fn up(&mut self, up: Up) -> Queue<Request> {
        self.event(Event::Client { event: jig_client::Event::Up { up } })
    }
    pub fn go(&mut self, address: Address) -> Queue<Request> {
        self.event(Event::Went { place: Place::Address(address) })
    }
    fn frame(&mut self, projects: &[u32]) -> Queue<Request> {
        let mut me = f::chats::me();
        let template = me.projects.get(0).unwrap().clone();
        me.projects = List::uncapped();
        for &project in projects {
            let mut row = template.clone();
            row.project = project;
            row.name = c::DocumentText::complete(c::Text::new(name(project), 64).unwrap());
            me.projects.push(row).unwrap();
        }
        self.frame_me(me)
    }
    fn frame_me(&mut self, me: c::Me) -> Queue<Request> {
        self.up(Up::Watched { watch: self.me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) })
    }
    pub(super) fn watch(&self, subject: Watchable) -> Token {
        let mut found = None;
        for (watch, held) in &self.opened {
            if *held == subject {
                found = Some(*watch);
            }
        }
        found.expect("fixture opened the watch")
    }
    pub(super) fn watches(&self, subject: Watchable) -> usize {
        let mut count = 0_usize;
        for (_, held) in &self.opened {
            if *held == subject {
                count = count.checked_add(1).unwrap();
            }
        }
        count
    }
    pub(super) fn status_watch(&self, project: u32) -> Token {
        self.watch(forge::status::watch(project, &self.limits.forge).unwrap())
    }
    fn decided_watch(&self, project: u32) -> Token {
        self.watch(forge::decided::watch(project, &self.limits.forge).unwrap())
    }
    pub(super) fn family(&mut self, watch: Token, bytes: Box<[u8]>) -> Queue<Request> {
        self.up(Up::FamilyDelivered {
            watch,
            record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
        })
    }
    pub(super) fn ack(&mut self) {
        for (watch, _) in self.opened.clone().into_boxed() {
            self.up(Up::Watched { watch, event: c::WatchEvent::Opened });
        }
    }
    fn settle(&mut self) -> Queue<Request> {
        let dismissed = self.watch(forge::dismissed::watch(&self.limits.forge).unwrap());
        self.family(
            dismissed,
            wire::dismissals(
                &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed: List::with_capacity(0) }).unwrap(),
            ),
        );
        let forges = self.watch(forge::forges::watch(&self.limits.forge).unwrap());
        self.family(
            forges,
            wire::forges(
                &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: List::with_capacity(0), linked: false })
                    .unwrap(),
            ),
        );
        let settings = self.watch(Watchable::Jig(c::Subject::Settings));
        self.up(Up::Watched {
            watch: settings,
            event: c::WatchEvent::Delivered(c::Delivery::Settings(Box::new(c::SettingsDocument {
                newest: 0,
                settings: List::uncapped(),
                held: List::uncapped(),
                pending: List::uncapped(),
            }))),
        })
    }
    fn status(&mut self, project: u32, previous: u64) {
        let mut document = crate::tests::status::document(3, 1, Some(3)).into_parts();
        document.project = project;
        document.previous = Some(previous);
        self.family(
            self.status_watch(project),
            wire::project_status(&w::ProjectStatus::new(&w::CEILINGS, document).unwrap()),
        );
    }
    pub fn decisions(&mut self, project: u32, rows: &[w::Decision]) -> Queue<Request> {
        self.family(
            self.decided_watch(project),
            wire::decisions(
                &w::Decisions::new(
                    &w::CEILINGS,
                    w::DecisionsParts {
                        project,
                        rows: List::from_boxed(rows.into()),
                        asked: List::with_capacity(0),
                        cut: false,
                    },
                )
                .unwrap(),
            ),
        )
    }
    fn act(&mut self, act: Act) -> Queue<Request> {
        self.event(Event::Act { action: Action::Pages(act) })
    }
    fn told(&mut self, key: Key, outcome: jig_client::Outcome) -> Queue<Request> {
        let mut out = Queue::uncapped();
        crate::route::told(&mut self.root, &f::env(&self.limits), Told::Answered { key, outcome }, &mut out);
        out
    }
    fn task(&mut self, task: u64, read: Token) {
        let mut document = f::chat::task();
        document.chip.task = task;
        self.up(Up::ReadDone {
            read,
            result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(document)))),
        });
    }
}
pub(super) fn closes(out: &Queue<Request>) -> Box<[Token]> {
    let mut values = List::uncapped();
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
        {
            values.push(*watch).unwrap();
        }
    }
    values.into_boxed()
}
fn looks(out: &Queue<Request>) -> Box<[u32]> {
    let mut values = List::uncapped();
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Send { request: jig_client::Ask::Family { record, .. }, .. },
                },
        } = request
            && let w::Request::Looked(row) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
        {
            values.push(row.project()).unwrap();
        }
    }
    values.into_boxed()
}
pub(super) fn task_reads(out: &Queue<Request>) -> Box<[(u64, Token)]> {
    let mut values = List::uncapped();
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Read { read, what: jig_client::Readable::Jig(c::Read::Task { task }) },
                },
        } = request
        {
            values.push((*task, *read)).unwrap();
        }
    }
    values.into_boxed()
}
fn sends(out: &Queue<Request>) -> Box<[jig_client::Ask]> {
    let mut values = List::uncapped();
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { request, .. } } } =
            request
        {
            values.push(request.clone()).unwrap();
        }
    }
    values.into_boxed()
}
fn family_answer(outcome: w::Outcome) -> jig_client::Outcome {
    jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
    }))
}
fn last_key(fixture: &Across) -> Key {
    Key { asker: crate::owner::Owner::Pages.asker(), number: fixture.root.pages().serial }
}
#[test]
fn across_watches_follow_me_once_and_leave_together() {
    type WatchCase<'a> = (&'a str, bool, &'a [u32], &'a [u32], &'a [u32]);
    let mut over = List::uncapped();
    for project in 1..=all_home::ACROSS + 1 {
        over.push(project).unwrap();
    }
    let over = over.into_boxed();
    let at = &over[..usize::try_from(all_home::ACROSS).unwrap()];
    let rows: [WatchCase<'_>; 5] = [
        ("three on entry", true, &[FIRST, SECOND, THIRD], &[FIRST, SECOND, THIRD], &[FIRST, SECOND, THIRD]),
        ("delayed me", false, &[], &[FIRST, SECOND, THIRD], &[FIRST, SECOND, THIRD]),
        ("added project", true, &[FIRST, SECOND], &[FIRST, SECOND, THIRD], &[FIRST, SECOND, THIRD]),
        ("at bound", true, at, at, at),
        ("past bound", true, &over, &over, at),
    ];
    for (case, before_entry, initial, later, expected) in rows {
        let mut fixture = Across::waiting(if before_entry { Address::Accounts } else { Address::AllHome });
        fixture.frame(initial);
        fixture.go(Address::AllHome);
        fixture.frame(later);
        fixture.frame(later);
        for &project in expected {
            assert_eq!(
                fixture.watches(forge::status::watch(project, &fixture.limits.forge).unwrap()),
                1,
                "{case}: status {project}"
            );
            assert_eq!(
                fixture.watches(forge::decided::watch(project, &fixture.limits.forge).unwrap()),
                usize::from(project <= u32::try_from(pages::decided::DECIDED_PROJECTS).unwrap()),
                "{case}: decided {project}"
            );
        }
        assert_eq!(
            fixture.watches(forge::status::watch(all_home::ACROSS + 1, &fixture.limits.forge).unwrap()),
            0,
            "{case}"
        );
        assert_eq!(fixture.root.pages().inbox().unwrap().filter, Filter::ALL, "{case}");
        let inbox_handle = fixture.root.pages().inbox().unwrap().entries;
        let mut owned = List::uncapped();
        for &project in expected {
            owned.push(fixture.status_watch(project)).unwrap();
            if project <= u32::try_from(pages::decided::DECIDED_PROJECTS).unwrap() {
                owned.push(fixture.decided_watch(project)).unwrap();
            }
        }
        fixture.ack();
        let out = fixture.go(Address::Accounts);
        let closed = closes(&out);
        assert!(fixture.root.client().document(inbox_handle).is_none(), "{case}: inbox unwatched");
        for watch in &owned {
            assert!(closed.contains(watch), "{case}: leave {watch:?}");
        }
    }
}
#[test]
fn removing_projects_releases_only_their_reads() {
    let mut fixture = Across::new(&[FIRST, SECOND, THIRD], Address::AllHome);
    fixture.ack();
    let removed = [fixture.status_watch(SECOND), fixture.decided_watch(SECOND)];
    let retained = [
        fixture.status_watch(FIRST),
        fixture.decided_watch(FIRST),
        fixture.status_watch(THIRD),
        fixture.decided_watch(THIRD),
    ];
    let out = fixture.frame(&[THIRD, FIRST]);
    let closed = closes(&out);
    for token in removed {
        assert!(closed.contains(&token));
    }
    for token in retained {
        assert!(!closed.contains(&token));
    }
    let facts = all_home::facts(&fixture.root, Wall::EPOCH);
    let mut projects = List::uncapped();
    for row in &facts.projects {
        projects.push(row.project).unwrap();
    }
    assert_eq!(projects.as_slice(), [THIRD, FIRST]);
}
#[test]
fn each_status_waits_for_its_look_and_survives_other_failures() {
    for (case, document_first, end) in
        [("document first", true, c::WatchEnd::Closed), ("answer first", false, c::WatchEnd::NotAllowed)]
    {
        let mut fixture = Across::new(&[FIRST, SECOND, THIRD], Address::AllHome);
        assert!(looks(&fixture.frame(&[FIRST, SECOND, THIRD])).is_empty(), "{case}: unsettled");
        let out = fixture.settle();
        assert_eq!(looks(&out).as_ref(), [FIRST, SECOND, THIRD], "{case}");
        let last = last_key(&fixture);
        for (index, project) in [FIRST, SECOND, THIRD].into_iter().enumerate() {
            let key = Key { number: last.number - u64::from(THREE) + u64::try_from(index).unwrap() + 1, ..last };
            if document_first {
                fixture.status(project, 50);
            }
            assert_eq!(all_home::facts(&fixture.root, Wall::EPOCH).projects[index].status, Status::Unknown, "{case}");
            fixture.told(key, family_answer(w::Outcome::Looked));
            if !document_first {
                assert_eq!(
                    all_home::facts(&fixture.root, Wall::EPOCH).projects[index].status,
                    Status::Unknown,
                    "{case}: undelivered"
                );
                fixture.status(project, 50);
            }
        }
        let failed = fixture.status_watch(SECOND);
        fixture.up(Up::WatchEnded { watch: failed, end });
        let facts = all_home::facts(&fixture.root, Wall::EPOCH);
        assert_ne!(facts.projects[0].status, Status::Unknown, "{case}");
        assert_eq!(facts.projects[1].status, Status::Unknown, "{case}");
        assert_ne!(facts.projects[2].status, Status::Unknown, "{case}");
        assert!(looks(&fixture.frame(&[FIRST, SECOND, THIRD])).is_empty(), "{case}: once per entry");
    }
}
#[test]
fn decisions_keep_the_newest_seven_across_projects_and_read_them() {
    let mut fixture = Across::new(&[FIRST, SECOND, THIRD], Address::AllHome);
    let mut reads = List::uncapped();
    for project in [FIRST, SECOND, THIRD] {
        let mut rows = List::with_capacity(THREE);
        for offset in 0..THREE {
            let number = project + offset * THREE;
            let mut row = crate::tests::decided::decision(number, u64::from(number)).into_parts();
            row.overturned = number == NINE;
            rows.push(w::Decision::new(&w::CEILINGS, row).unwrap()).unwrap();
        }
        for read in task_reads(&fixture.decisions(project, rows.as_slice())) {
            reads.push(read).unwrap();
        }
    }
    for (task, read) in reads.into_boxed() {
        fixture.task(task, read);
    }
    let rows: [(&str, Wall, u32, &[u32]); 3] = [
        ("all", Wall::EPOCH, NINE, &[3, 4, 5, 6, 7, 8, 9]),
        ("oldest expires", Wall::from_nanos(DAY + 1), NINE - 1, &[3, 4, 5, 6, 7, 8, 9]),
        ("last expires", Wall::from_nanos(DAY + 9), 0, &[]),
    ];
    for (case, wall, showing, numbers) in rows {
        let facts = all_home::facts(&fixture.root, wall);
        assert_eq!(facts.showing, showing, "{case}");
        let mut values = List::uncapped();
        for row in &facts.decided {
            values.push(row.line.number).unwrap();
        }
        assert_eq!(values.as_slice(), numbers, "{case}");
        for row in &facts.decided {
            assert_eq!(row.line.state, pages::decided::DecisionTask::Live, "{case}: task read");
            assert!(row.line.title.is_some(), "{case}");
            assert_eq!(row.line.overturned, row.line.number == NINE, "{case}");
        }
        assert!(facts.decided.len() <= SEVEN);
    }
    let unchanged = fixture.decisions(
        THIRD,
        &[
            crate::tests::decided::decision(THIRD, u64::from(THIRD)),
            crate::tests::decided::decision(6, 6),
            crate::tests::decided::decision(NINE, u64::from(NINE)),
        ],
    );
    assert!(task_reads(&unchanged).is_empty(), "unchanged rows are read once");
}
#[test]
fn overturn_uses_the_displayed_rows_project_on_wide_pages() {
    for address in [
        Address::AllHome,
        Address::Inbox { filter: Filter::ALL },
        Address::Inbox { filter: Filter { project: Some(SECOND), ..Filter::ALL } },
    ] {
        let mut fixture = Across::new(&[FIRST, SECOND], address.clone());
        let out = fixture.decisions(SECOND, &[crate::tests::decided::decision(FIRST, 0)]);
        for (task, read) in task_reads(&out) {
            fixture.task(task, read);
        }
        let out = fixture.act(Act::Away(pages::away::Act::Overturn { project: SECOND, number: FIRST }));
        assert_eq!(
            sends(&out).as_ref(),
            [forge::decided::overturn(SECOND, FIRST, &fixture.limits.forge).unwrap()],
            "{address:?}"
        );
        let key = last_key(&fixture);
        let out = fixture.told(key, family_answer(w::Outcome::Overturned));
        assert_eq!(
            sends(&out).as_ref(),
            [jig_client::Ask::Jig(Box::new(c::Ask::Message {
                project: SECOND,
                task: u64::from(FIRST),
                words: c::Text::new(
                    forge::decided::overturn_words(b"Which approach?", b"The smaller change"),
                    fixture.limits.client.domain.text
                )
                .unwrap(),
            }))],
            "{address:?}: message"
        );
        assert!(all_home::decided(&fixture.root, Wall::EPOCH)[0].line.overturned, "{address:?}");
        let key = last_key(&fixture);
        fixture.told(
            key,
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::Said { task: 1, message: 9 },
            )))),
        );
        assert_eq!(fixture.root.pages().needs().toast(), Some(&needs::Toasted::Overturned));
    }
}
#[test]
fn ask_words_are_bounded_and_handed_to_the_unsent_composer() {
    type AskCase = (&'static str, Address, &'static [u32], Option<u32>, &'static [u8], Option<u32>);
    let rows: [AskCase; 5] = [
        (
            "project Home",
            Address::Home { project: SECOND },
            &[FIRST, SECOND],
            None,
            b"Explain this".as_slice(),
            Some(SECOND),
        ),
        ("open project", Address::AllHome, &[FIRST, SECOND], Some(SECOND), b"Explain this".as_slice(), Some(SECOND)),
        ("first project", Address::AllHome, &[FIRST, SECOND], None, b"Explain this".as_slice(), Some(FIRST)),
        ("empty", Address::AllHome, &[FIRST], None, b"".as_slice(), None),
        ("no project", Address::AllHome, &[], None, b"Explain this".as_slice(), None),
    ];
    for (case, address, projects, open, words, expected) in rows {
        let mut fixture = Across::new(projects, address.clone());
        if let Some(project) = open {
            fixture.act(Act::Project { project });
        }
        fixture.act(Act::AskLine { words: words.into() });
        assert_eq!(fixture.root.pages().ask_line(), words, "{case}");
        let out = fixture.act(Act::AskWith { words: words.into() });
        let mut shown = List::uncapped();
        for request in &out {
            if let Request::Show { address, .. } = request {
                shown.push(address.clone()).unwrap();
            }
        }
        let expected_address = match expected {
            Some(project) => Some(Address::Ask { project }),
            None => None,
        };
        assert_eq!(shown.as_slice(), expected_address.as_slice(), "{case}");
        assert!(sends(&out).is_empty(), "{case}: unsent");
        if let Some(project) = expected {
            fixture.go(Address::Ask { project });
            assert_eq!(fixture.root.pages().chats().unwrap().words.as_ref(), words, "{case}");
            assert!(fixture.root.pages().chats().unwrap().start.is_none(), "{case}");
        }
        fixture.go(Address::Accounts);
        assert!(fixture.root.pages().ask_line().is_empty(), "{case}: leaving drops line");
    }
    let mut fixture = Across::new(&[FIRST], Address::AllHome);
    let bound = usize::try_from(fixture.limits.client.domain.long_text).unwrap();
    for length in [bound, bound + 1] {
        fixture.act(Act::AskLine { words: bytes::zeroed(length) });
        assert_eq!(fixture.root.pages().ask_line().len(), bound);
    }
}

#[test]
fn away_uses_the_newest_finished_spell_after_the_latest_look() {
    type AwayCase = (&'static str, [u64; 2], &'static [(u32, u64, Option<u64>, Option<u64>)], Option<(u64, u64)>);
    let rows: [AwayCase; 5] = [
        ("none", [10, 20], &[], None),
        ("latest", [10, 20], &[(FIRST, 15, Some(25), None), (SECOND, 30, Some(40), None)], Some((30, 40))),
        ("at latest look", [10, 40], &[(FIRST, 15, Some(25), None), (SECOND, 30, Some(40), None)], None),
        ("still away", [10, 20], &[(FIRST, 30, None, None)], None),
        ("until passed", [10, 20], &[(FIRST, 30, None, Some(40))], Some((30, 40))),
    ];
    for (case, previous, rows, expected) in rows {
        let mut fixture = Across::new(&[FIRST, SECOND], Address::AllHome);
        fixture.settle();
        let last = last_key(&fixture);
        for (index, project) in [FIRST, SECOND].into_iter().enumerate() {
            fixture.status(project, previous[index]);
            fixture.told(
                Key { number: last.number - 1 + u64::try_from(index).unwrap(), ..last },
                family_answer(w::Outcome::Looked),
            );
        }
        let mut values = List::with_capacity(w::CEILINGS.aways_rows);
        for &(project, since, off, until) in rows {
            values
                .push(w::AwayRow::new(&w::CEILINGS, w::AwayRowParts { project, since, off, until, landed: 0 }).unwrap())
                .unwrap();
        }
        let away = fixture.watch(forge::away::watch(&fixture.limits.forge).unwrap());
        fixture.family(away, wire::aways(&w::Aways::new(&w::CEILINGS, w::AwaysParts { rows: values }).unwrap()));
        assert_eq!(
            all_home::facts(&fixture.root, Wall::from_nanos(100)).away,
            match expected {
                Some((since, off)) => Some(Spell { since: Wall::from_nanos(since), off: Wall::from_nanos(off) }),
                None => None,
            },
            "{case}"
        );
    }
}
#[test]
fn all_home_counts_unfiltered_cards_and_every_approval() {
    for (case, inbox_cut, more) in [("whole", false, 0), ("inbox cut", true, 0), ("approvals cut", false, 1)] {
        let mut fixture = Across::new(&[FIRST, SECOND, THIRD], Address::AllHome);
        let inbox =
            fixture.watch(Watchable::Jig(c::Subject::Inbox { window: fixture.root.pages().inbox().unwrap().window }));
        let mut rows = List::with_capacity(1);
        rows.push(f::inbox_entry(f::acts::QUESTION, c::InboxGroup::Questions, FIRST, 0)).unwrap();
        let document = crate::tests::documents::inbox(rows, 1, c::Count { value: 0, limited: false }, 0, inbox_cut);
        let out = fixture
            .up(Up::Watched { watch: inbox, event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(document))) });
        for request in &out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { .. }) },
                    },
            } = request
            {
                fixture
                    .up(Up::ReadDone { read: *read, result: Readout::Jig(c::ReadResult::Value(f::acts::question())) });
            }
        }
        let approvals = fixture.watch(forge::approvals::watch(&fixture.limits.forge).unwrap());
        let rows = [
            crate::tests::approvals::row(crate::tests::approvals::CHANGE, SECOND, None),
            crate::tests::approvals::row(crate::tests::approvals::OTHER_CHANGE, THIRD, None),
        ];
        fixture.family(
            approvals,
            wire::approvals(
                &w::Approvals::new(&w::CEILINGS, w::ApprovalsParts { rows: List::from_boxed(Box::new(rows)), more })
                    .unwrap(),
            ),
        );
        assert_eq!(
            all_home::facts(&fixture.root, Wall::EPOCH).needs,
            Needs::Count { count: THREE, cut: inbox_cut || more > 0 },
            "{case}"
        );
        let out =
            fixture.act(Act::Needs(needs::Act::Choose { object: f::acts::QUESTION, words: b"Yes".as_slice().into() }));
        assert!(sends(&out).is_empty(), "{case}: held before sending");
        assert!(fixture.root.pages().needs().held(f::acts::QUESTION), "{case}");
        assert_eq!(
            all_home::facts(&fixture.root, Wall::EPOCH).needs,
            Needs::Count { count: 2, cut: inbox_cut || more > 0 },
            "{case}: answered card removed"
        );
        let mut env = f::env(&fixture.limits);
        env.now = skein_lib::Time::ZERO.saturating_add(needs::HOLD);
        let mut out = Queue::uncapped();
        crate::fire(&mut fixture.root, &env, &mut out);
        assert_eq!(
            sends(&out).as_ref(),
            [jig_client::Ask::Jig(Box::new(c::Ask::Answer {
                project: FIRST,
                on: c::AnswerOn::Question {
                    task: f::acts::TASK,
                    message: f::acts::MESSAGE,
                    words: f::acts::text(b"Yes")
                }
            }))],
            "{case}: held answer sent"
        );
    }
}
#[test]
fn all_home_card_acts_keep_their_existing_confirmations() {
    use c::{ActKind, ObjectKey};
    let proposal = ObjectKey::Proposal(f::acts::PROPOSAL);
    let held = ObjectKey::Escalation(f::acts::TASK);
    for (act, object, page, confirm, expected) in [
        (
            ActKind::Accept,
            proposal,
            f::acts::proposal(ActKind::Accept, c::Admission::Allowed),
            true,
            c::Ask::Decide {
                project: FIRST,
                on: c::DecideOn::Proposal {
                    proposal: f::acts::PROPOSAL,
                    decision: c::Decision::Accepted,
                    running: None,
                },
            },
        ),
        (
            ActKind::Reject,
            proposal,
            f::acts::proposal(ActKind::Reject, c::Admission::Allowed),
            true,
            c::Ask::Decide {
                project: FIRST,
                on: c::DecideOn::Proposal {
                    proposal: f::acts::PROPOSAL,
                    decision: c::Decision::Rejected { reason: f::acts::text(b"Because") },
                    running: None,
                },
            },
        ),
        (
            ActKind::Release,
            held,
            f::acts::held(ActKind::Release),
            true,
            c::Ask::Release { project: FIRST, task: f::acts::TASK, revision: f::acts::REVISION, again: None },
        ),
        (
            ActKind::LeaveHeld,
            held,
            f::acts::held(ActKind::LeaveHeld),
            false,
            c::Ask::Decide {
                project: FIRST,
                on: c::DecideOn::Escalation {
                    task: f::acts::TASK,
                    revision: f::acts::REVISION,
                    decision: c::Decision::LeftHeld { reason: f::acts::text(b"Because") },
                },
            },
        ),
    ] {
        let mut fixture = Across::new(&[FIRST], Address::AllHome);
        let mut cards = f::acts::pages(object, page);
        fixture.root.pages.inbox.as_mut().unwrap().cards = cards.inbox.take().unwrap().cards;
        fixture.act(Act::CardWords { object, words: b"Because".as_slice().into() });
        let out = fixture.act(Act::Decide { object, revision: Some(f::acts::REVISION), act });
        assert_eq!(fixture.root.pages().confirmation().is_some(), confirm, "{act:?}");
        let out = if confirm {
            assert!(sends(&out).is_empty(), "{act:?}: waits for confirmation");
            fixture.act(Act::Confirm)
        } else {
            out
        };
        assert_eq!(sends(&out).as_ref(), [jig_client::Ask::Jig(Box::new(expected))], "{act:?}");
    }
}
