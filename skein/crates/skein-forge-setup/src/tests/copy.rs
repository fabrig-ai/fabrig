//! Hand-given copy replies, including interrupted copies (skein/docs/design/forge/setup/forgejo.md, 1.10).
use super::{fixtures, host};
use crate::{
    copy::{self, Copy},
    given::Given,
    host::{File, Plan},
    item::Item,
    reconcile::{Event, Need, Outcome, Reply, Stage, Why},
};
use core::num::NonZeroU64;
use skein_forge::forgejo::{
    document::{Branch, CreatedRepository, Document, Org, Permission},
    id::{self, ObjectId},
    request::{Author, Grant, Repo, Request},
};
use skein_lib::{Env, List, Queue, Time, Token, Wall};

const ID: u64 = 7;
const OLD_ID: u64 = 9;
const FROM: &[u8] = b"https://source.invalid/owner/repo";
const SUBJECT: &[u8] = b"first/copied";
const BRANCH: &[u8] = b"trunk";
const PATH: &[u8] = b".config/workflow.yml";
const CONTENT: &[u8] = b"owner's file";
// Five admin/org rows, four copy rows and four access/file rows.
const COPY_STEPS: u32 = 13;
fn at() -> Repo {
    Repo { owner: b"first".as_slice().into(), name: b"copied".as_slice().into() }
}
fn files() -> Box<[File]> {
    Box::new([File { path: PATH.into(), content: CONTENT.into() }])
}
fn plan() -> Plan {
    Plan::Copy { forge: 1, org: at().owner, repository: at().name, from: FROM.into(), files: files() }
}
fn machine_with(given: &Given, files: Box<[File]>) -> Copy {
    Copy::new(given, at().owner, at().name, FROM.into(), files).expect("copy fixture")
}
fn machine() -> Copy {
    machine_with(&fixtures::given(), files())
}
#[expect(clippy::disallowed_macros, reason = "terminal diagnostics check that credentials are released")]
fn go(machine: &mut Copy, event: Event) -> Need {
    let mut out = Queue::uncapped();
    copy::step(machine, event, &mut out);
    assert_eq!(out.len(), 1, "at most one need, and this event needs one");
    let need = out.pop().expect("one need");
    match &need {
        Need::Done(_) | Need::Stop(_) => {
            let debug = format!("{machine:?}");
            assert!(debug.contains("admin: None"));
            assert!(debug.contains("secret: None"));
        }
        Need::Send { .. } | Need::Read(_) | Need::Write { .. } | Need::Draw(_) => (),
    }
    need
}
fn document(document: Document) -> Event {
    Event::Reply(Reply::Document(Box::new(document)))
}
fn status(status: u16) -> Event {
    Event::Reply(Reply::Status(status))
}
fn org() -> Event {
    document(Document::Org(Org { id: 1, name: b"first".as_slice().into() }))
}
fn repository(id: u64) -> Event {
    let Event::Reply(Reply::Document(value)) = fixtures::repository(id, b"first", b"copied") else {
        panic!("repository fixture")
    };
    let Document::Repository(mut repository) = *value else { panic!("repository fixture") };
    repository.default_branch = BRANCH.into();
    document(Document::Repository(repository))
}
fn migrated(id: u64, branch: &[u8]) -> Event {
    document(Document::CreatedRepository(CreatedRepository {
        id,
        full_name: SUBJECT.into(),
        default_branch: branch.into(),
        clone_url: b"https://forge.invalid/first/copied.git".as_slice().into(),
        ssh_url: Box::new([]),
    }))
}
fn commit() -> ObjectId {
    id::read(b"0000000000000000000000000000000000000001").expect("commit fixture")
}
fn branch() -> Event {
    document(Document::Branch(Branch { name: BRANCH.into(), commit: commit(), effective_branch_protection_name: None }))
}
fn branch_read() -> Need {
    fixtures::send(Request::Branch { at: at(), name: BRANCH.into() })
}
fn mark() -> Need {
    Need::Write { item: Item::Copying, bytes: fixtures::secret(&[1]) }
}
fn migrate() -> Need {
    fixtures::send(Request::Migrate { at: at(), from: FROM.into() })
}
fn permission() -> Need {
    fixtures::send(Request::Permission { at: at(), login: b"kit".as_slice().into() })
}
fn done() -> Need {
    Need::Done(Outcome::Copied { id: NonZeroU64::new(ID).expect("id"), branch: BRANCH.into() })
}
fn failed(why: Why) -> Need {
    fixtures::named_failure(Stage::Repository, SUBJECT, why)
}
fn put_file() -> Need {
    fixtures::send(Request::CreateFile {
        at: at(),
        path: PATH.into(),
        branch: BRANCH.into(),
        content: CONTENT.into(),
        message: b"Add .config/workflow.yml".as_slice().into(),
        author: Author { name: b"kit".as_slice().into(), email: b"kit@example.invalid".as_slice().into() },
    })
}
// Every row leaves the named stage outstanding. Both org states are included.
fn prefix() -> Box<[(Event, Need, Stage)]> {
    Box::new([
        (Event::Begin, Need::Read(Item::AdminToken), Stage::Admin),
        (Event::Read(Some(fixtures::secret(fixtures::ADMIN))), fixtures::send(Request::User), Stage::Admin),
        (
            fixtures::user_event(fixtures::admin()),
            fixtures::send(Request::Org { name: b"first".as_slice().into() }),
            Stage::Org,
        ),
        (
            status(404),
            fixtures::send(Request::CreateOrg {
                person: b"kit-admin".as_slice().into(),
                name: b"first".as_slice().into(),
            }),
            Stage::Org,
        ),
        (org(), fixtures::send(Request::Repository { at: at() }), Stage::Repository),
    ])
}
fn copying() -> Box<[(Event, Need, Stage)]> {
    Box::new([
        (status(404), mark(), Stage::Repository),
        (Event::Written, migrate(), Stage::Repository),
        (migrated(ID, BRANCH), branch_read(), Stage::Repository),
        (branch(), permission(), Stage::Repository),
    ])
}
fn file_steps() -> Box<[(Event, Need, Stage)]> {
    Box::new([
        (
            fixtures::permission(Permission::None),
            fixtures::send(Request::AddCollaborator {
                at: at(),
                login: b"kit".as_slice().into(),
                permission: Grant::Write,
            }),
            Stage::Repository,
        ),
        (Event::Reply(Reply::Empty), branch_read(), Stage::File),
        (branch(), fixtures::send(Request::Raw { at: at(), path: PATH.into(), reference: commit() }), Stage::File),
        (status(404), put_file(), Stage::File),
    ])
}
fn resume() -> Box<[(Event, Need, Stage)]> {
    Box::new([
        (repository(OLD_ID), Need::Read(Item::Copying), Stage::Repository),
        (Event::Read(Some(fixtures::secret(&[1]))), branch_read(), Stage::Repository),
        (status(404), fixtures::send(Request::DeleteRepository { at: at() }), Stage::Repository),
    ])
}
fn replay(machine: &mut Copy, rows: &[(Event, Need, Stage)]) {
    for (event, _, _) in rows {
        go(machine, event.clone());
    }
}
fn at_repository() -> Copy {
    let mut machine = machine();
    replay(&mut machine, &prefix());
    machine
}
fn at_migrate() -> Copy {
    let mut machine = at_repository();
    go(&mut machine, status(404));
    go(&mut machine, Event::Written);
    machine
}
fn at_files() -> Copy {
    let mut machine = at_repository();
    replay(&mut machine, &copying());
    replay(&mut machine, file_steps().get(..2).expect("permission and grant rows"));
    machine
}
fn silent(machine: &mut Copy) {
    let mut out = Queue::uncapped();
    for event in [Event::Begin, Event::Read(None), Event::Written, Event::Reply(Reply::Down), fixtures::drawn()] {
        copy::step(machine, event, &mut out);
        copy::halt(machine, Why::Down, &mut out);
        assert!(out.is_empty());
    }
}
#[test]
fn a_copy_reads_marks_migrates_grants_and_puts_its_files() {
    let mut machine = machine();
    for rows in [prefix(), copying(), file_steps()] {
        for (event, need, _) in rows {
            assert_eq!(go(&mut machine, event), need);
        }
    }
    assert_eq!(go(&mut machine, document(Document::CreatedFile { commit: commit() })), done());
    silent(&mut machine);
}
#[test]
fn a_repository_with_no_mark_is_taken_and_nothing_is_written() {
    for mark in [None, Some(fixtures::secret(b""))] {
        let mut machine = at_repository();
        assert_eq!(go(&mut machine, repository(ID)), Need::Read(Item::Copying));
        assert_eq!(go(&mut machine, Event::Read(mark)), failed(Why::Taken));
        silent(&mut machine);
    }
}
#[test]
fn a_marked_repository_with_its_branch_is_the_copy_finished() {
    let mut machine = machine_with(&fixtures::given(), Box::new([]));
    replay(&mut machine, &prefix());
    assert_eq!(go(&mut machine, repository(ID)), Need::Read(Item::Copying));
    assert_eq!(go(&mut machine, Event::Read(Some(fixtures::secret(&[1])))), branch_read());
    assert_eq!(go(&mut machine, branch()), permission());
    assert_eq!(go(&mut machine, fixtures::permission(Permission::Write)), done());
}
#[test]
fn a_marked_repository_with_no_branch_is_deleted_and_copied_again() {
    for deleted in [Reply::Empty, Reply::Status(404)] {
        let mut machine = at_repository();
        for (event, need, _) in resume() {
            assert_eq!(go(&mut machine, event), need);
        }
        assert_eq!(go(&mut machine, Event::Reply(deleted)), mark());
        assert_eq!(go(&mut machine, Event::Written), migrate());
        assert_eq!(go(&mut machine, migrated(ID, BRANCH)), branch_read());
        assert_eq!(go(&mut machine, branch()), permission());
        go(&mut machine, fixtures::permission(Permission::Write));
        go(&mut machine, branch());
        assert_eq!(go(&mut machine, Event::Reply(Reply::Empty)), done());
    }
}
#[test]
fn a_migrate_answered_409_is_taken() {
    let mut machine = at_migrate();
    assert_eq!(go(&mut machine, status(409)), failed(Why::Taken));
    silent(&mut machine);
}
#[test]
fn a_migrate_refused_stops_at_repository_with_its_status() {
    for status_code in [401, 403, 404, 422, 500] {
        assert_eq!(go(&mut at_migrate(), status(status_code)), failed(Why::Refused(status_code)));
    }
}
#[test]
fn a_source_with_no_branch_stops_at_empty() {
    let mut machine = at_migrate();
    go(&mut machine, migrated(ID, BRANCH));
    assert_eq!(go(&mut machine, status(404)), failed(Why::Empty));
    silent(&mut machine);
}
#[test]
fn a_users_name_stops_at_org() {
    let mut machine = machine();
    replay(&mut machine, prefix().get(..3).expect("admin rows"));
    assert_eq!(
        go(&mut machine, status(404)),
        fixtures::send(Request::CreateOrg { person: b"kit-admin".as_slice().into(), name: b"first".as_slice().into() })
    );
    assert_eq!(go(&mut machine, status(422)), fixtures::named_failure(Stage::Org, b"first", Why::Refused(422)));
}
#[test]
fn a_file_the_source_has_is_left_as_it_is() {
    let mut machine = at_files();
    assert_eq!(
        go(&mut machine, branch()),
        fixtures::send(Request::Raw { at: at(), path: PATH.into(), reference: commit() })
    );
    assert_eq!(go(&mut machine, Event::Reply(Reply::Empty)), done());
}
#[expect(clippy::disallowed_macros, reason = "halt checks that neither administrator nor hook secret remains")]
fn halted(mut machine: Copy, stage: Stage) {
    let subject = match stage {
        Stage::Admin => b"kit-admin".as_slice(),
        Stage::Org => b"first",
        Stage::Repository => SUBJECT,
        Stage::File => PATH,
        Stage::Bot | Stage::Token | Stage::App | Stage::Installation | Stage::State | Stage::Person => {
            panic!("copy stage")
        }
    };
    let mut out = Queue::uncapped();
    copy::halt(&mut machine, Why::Down, &mut out);
    assert_eq!(out.pop(), Some(fixtures::named_failure(stage, subject, Why::Down)));
    assert!(out.is_empty());
    let debug = format!("{machine:?}");
    assert!(debug.contains("admin: None"));
    assert!(debug.contains("secret: None"));
    silent(&mut machine);
}
#[test]
fn a_halt_at_each_state_stops_at_its_stage_and_drops_the_token() {
    halted(machine(), Stage::Admin);
    let mut script = List::with_capacity(COPY_STEPS);
    for rows in [prefix(), copying(), file_steps()] {
        for row in rows {
            script.push(row).expect("copy script capacity");
        }
    }
    let script = script.as_slice();
    for (index, (_, _, stage)) in script.iter().enumerate() {
        let count = index.checked_add(1).expect("bounded script");
        let mut machine = machine();
        replay(&mut machine, script.get(..count).expect("script prefix"));
        halted(machine, *stage);
    }
    let script = resume();
    for count in 1..=script.len() {
        let mut machine = at_repository();
        replay(&mut machine, script.get(..count).expect("resume prefix"));
        halted(machine, Stage::Repository);
    }
    // Exercise the access helper with a live hook secret too.
    let mut given = fixtures::given();
    given.listener = Some(fixtures::listener());
    let script = [
        fixtures::permission(Permission::Write),
        Event::Read(None),
        fixtures::drawn(),
        Event::Written,
        fixtures::settings_event(2),
        fixtures::rows(Box::new([])),
    ];
    for count in 1..=script.len() {
        let mut machine = machine_with(&given, files());
        replay(&mut machine, &prefix());
        replay(&mut machine, &copying());
        for event in script.iter().take(count) {
            go(&mut machine, event.clone());
        }
        halted(machine, Stage::Repository);
    }
}
#[test]
fn a_copy_runs_under_the_host() {
    use crate::host::{Answer, Event, Host, Need, Request};
    let limits = host::limits();
    let mut host = Host::new(&limits, host::settings(&[1]), host::crypto()).expect("host fixture");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    let run = Token::new(1);
    host.down(&env, Request::Run { run, plan: plan() }, &mut up, &mut io);
    assert_eq!(up.pop(), Some(Event::Need { run, need: Need::Read(Item::AdminToken) }));
    host.down(&env, Request::Answer { run, answer: Answer::Read(None) }, &mut up, &mut io);
    let crate::reconcile::Need::Stop(failure) = fixtures::named_failure(Stage::Admin, b"kit-admin", Why::Missing)
    else {
        panic!("stop fixture")
    };
    assert_eq!(up.pop(), Some(Event::Stop { run, failure }));
    assert!(up.is_empty());
    assert!(io.is_empty());
}
#[test]
fn an_invalid_migration_identity_is_unexpected() {
    for (id, branch) in [(0, BRANCH), (ID, b"".as_slice())] {
        assert_eq!(go(&mut at_migrate(), migrated(id, branch)), failed(Why::Unexpected));
    }
}
