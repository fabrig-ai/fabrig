//! A landed public goal read at both owners (temper/docs/design/protocol/sign-in.md, 2.5).
use super::{checks::PROXY, http::Answer, proxy::Proxy, service, service_checks, visitor::Visitor};
use crate::{ctl::Caller, fixtures, referee::Store};
use jig_client_protocol_wire::public_v1 as p;
use skein_http::Method;
use skein_lib::{Reader, Time};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(super) struct Walk {
    store: Rc<RefCell<Store>>,
    stage: usize,
    calls: Vec<u64>,
    task: Option<u64>,
    wake: Option<Time>,
    landing: Cell<Option<(u64, bool)>>,
}
impl Walk {
    pub fn new(store: Rc<RefCell<Store>>) -> Self {
        Self { store, stage: 0, calls: Vec::new(), task: None, wake: None, landing: Cell::new(None) }
    }
    pub fn ready(&self, caller: &Caller, visitor: &Visitor) -> bool {
        match self.stage {
            0 => true,
            1 => caller.answers.len() == 6,
            2 => caller.answers.len() == 7 && self.landing_complete(),
            _ => self.calls.iter().all(|id| visitor.answers.contains_key(id)),
        }
    }
    fn landed(&self) -> bool {
        use temper_engine_domain::Key;
        use temper_engine_domain_forge::Key::Landed;
        let store = self.store.borrow();
        if let Some((applied, landed)) = self.landing.get()
            && applied == store.applied
        {
            return landed;
        }
        let landed = store.rows.range(Key::Forge(Landed([0; 32]))..=Key::Forge(Landed([u8::MAX; 32]))).next().is_some();
        self.landing.set(Some((store.applied, landed)));
        landed
    }
    fn landing_complete(&self) -> bool {
        self.landed()
            && (!super::TRACKED.with(Cell::get)
                || self.store.borrow().rows.values().any(|record| {
                    if let temper_engine_domain::Record::Forge { row } = record
                        && let temper_engine_domain_forge::Stored::Landing(row) = row.as_ref()
                    {
                        row.landing.phase == temper_engine_domain_forge::LandingPhase::Landed
                    } else {
                        false
                    }
                }))
    }
    pub fn diagnostic(&self) -> String {
        let tasks: Vec<_> = self
            .store
            .borrow()
            .rows
            .values()
            .filter_map(|row| {
                if let temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
                    jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task),
                )) = row
                {
                    Some((
                        task.number,
                        task.project,
                        task.tracked,
                        task.flags.conversation,
                        matches!(task.phase, jig_domain_tasks::Phase::Closing(_)),
                        matches!(task.phase, jig_domain_tasks::Phase::Ended(_)),
                    ))
                } else {
                    None
                }
            })
            .collect();
        format!("merge stage {}, tasks {tasks:?}", self.stage)
    }
    pub fn wake(&self) -> Option<Time> {
        self.wake
    }
    fn last<'a>(&self, visitor: &'a Visitor) -> &'a Answer {
        visitor.answers[self.calls.last().expect("call")].as_ref().expect("answer")
    }
    #[expect(clippy::too_many_lines, reason = "the merge prerequisites and per-kind assertions form one state machine")]
    pub fn act(
        &mut self,
        now: Time,
        caller: &mut Caller,
        _proxy: &Proxy,
        visitor: &mut Visitor,
        project: u64,
        transport: &temper_fake_git_hosted::Typed,
    ) -> bool {
        if self.wake.is_some_and(|wake| now < wake) || self.calls.iter().any(|id| !visitor.answers.contains_key(id)) {
            return false;
        }
        self.wake = None;
        match self.stage {
            0 => {
                for words in [
                    vec!["account", "add-key", "1", "/setup/llm.key", "--deployment", "--key", "public-account"],
                    vec![
                        "forge",
                        "connect",
                        "forgejo-admin",
                        "/setup/admin.token",
                        "--api",
                        "127.0.0.1:3000",
                        "--git",
                        "http://127.0.0.1:3000",
                        "--key",
                        "public-forge",
                    ],
                    vec!["forge", "list", "--until", "connected:1"],
                    vec![
                        "repository",
                        "adopt",
                        "--project",
                        "1",
                        "--owner",
                        "acme",
                        "--name",
                        "app",
                        "--landing",
                        "main",
                        "--key",
                        "public-adopt",
                    ],
                ] {
                    service::command(caller, &words);
                }
                self.stage = 1;
            }
            1 => {
                if caller.answers.len() != 6 {
                    return false;
                }
                for answer in &caller.answers {
                    assert_eq!(
                        answer.exit,
                        skein_io::kernel::Exit::Code(0),
                        "adopt before merge: {} {}",
                        String::from_utf8_lossy(&answer.output),
                        String::from_utf8_lossy(&answer.error)
                    );
                }
                crate::stories::prepare_merge_branch(transport);
                service::command(
                    caller,
                    &[
                        "chat",
                        "start",
                        "--project",
                        "1",
                        "--key",
                        "public-chat",
                        "@plan Public conversation title\nPRIVATE_WORDS",
                    ],
                );
                self.stage = 2;
            }
            2 => {
                if caller.answers.len() != 7 {
                    return false;
                }
                for answer in &caller.answers {
                    assert_eq!(
                        answer.exit,
                        skein_io::kernel::Exit::Code(0),
                        "public merge setup: {} {}",
                        String::from_utf8_lossy(&answer.output),
                        String::from_utf8_lossy(&answer.error)
                    );
                }
                if !self.landing_complete() {
                    return false;
                }
                self.calls.push(visitor.send(
                    fixtures::CLIENTS,
                    Method::Get,
                    format!("/jig/v1/public/0/1/2/{project:016x}").as_bytes(),
                    &[(b"Host", b"one.example")],
                ));
                self.stage = 3;
            }
            3 => {
                let answer = self.last(visitor);
                assert_eq!(answer.status, 200);
                let board = p::PublicBoard::decode(&p::CEILINGS, &mut Reader::new(&answer.body)).expect("board");
                let goals: Vec<_> = board.live().iter().chain(board.ended().iter()).collect();
                if !super::TRACKED.with(core::cell::Cell::get) {
                    assert!(goals.is_empty(), "untracked change and its conversation delegates leave no goal");
                    self.calls.push(visitor.send(
                        fixtures::CLIENTS,
                        Method::Get,
                        format!("/jig/v1/public/2/1/2/{project:016x}").as_bytes(),
                        &[(b"Host", b"one.example")],
                    ));
                    self.stage = 100;
                    return false;
                }
                assert_eq!(goals.len(), 1, "one tracked goal, conversation excluded: {}", self.diagnostic());
                let goal = goals[0];
                super::identifiers::remember(
                    project,
                    goal.line().task(),
                    goal.tree().iter().map(p::PublicLine::task).collect(),
                );
                assert!(goal.line().goal());
                assert_eq!(goal.line().parent(), &None, "conversation ancestry never crosses");
                assert_eq!(goal.line().title(), b"@edit Public change title");
                assert_eq!(
                    goal.line().delegates().working()
                        + goal.line().delegates().waiting()
                        + goal.line().delegates().held()
                        + goal.line().delegates().closing()
                        + goal.line().delegates().ended(),
                    goal.tree().len()
                );
                self.task = Some(goal.line().task());
                self.stage = 4;
            }
            100 => {
                let answer = self.last(visitor);
                assert_eq!(answer.status, 200);
                service_checks::policy(answer, false);
                let app = p::PublicApplication::decode(&p::CEILINGS, &mut Reader::new(&answer.body))
                    .expect("untracked forge envelope");
                assert!(app.tasks().is_empty(), "no excluded task enters the envelope");
                let mut fields = Reader::new(app.record());
                assert_eq!(fields.u16(), Some(1));
                assert_eq!(fields.u8(), Some(0), "no excluded turn holder");
                assert_eq!(fields.u32(), Some(0), "no excluded landing counted");
                assert_eq!(fields.u32(), Some(0), "no excluded queue counted or ordered");
                assert!(fields.is_empty());
                return true;
            }
            200..=205 => return self.cross_project(visitor, project),
            _ => {
                let row = (self.stage - 4) / 2;
                let task = self.task.expect("public change");
                let paths = [
                    "/jig/v1/public/0/1/1".to_owned(),
                    format!("/jig/v1/public/0/1/2/{project:016x}"),
                    format!("/jig/v1/public/0/1/3/{project:016x}/{task:016x}"),
                    format!("/jig/v1/public/2/1/1/{project:016x}/{task:016x}"),
                    format!("/jig/v1/public/2/1/2/{project:016x}"),
                    format!("/jig/v1/public/2/1/3/{project:016x}/{task:016x}"),
                    "/jig/v1/public/0/2/1".to_owned(),
                    format!("/jig/v1/public/2/2/2/{project:016x}"),
                    "/jig/v1/public/0/1/65535".to_owned(),
                    format!("/jig/v1/public/65535/1/2/{project:016x}"),
                ];
                if row == paths.len() {
                    self.stage = 200;
                    return false;
                }
                if (self.stage - 4).is_multiple_of(2) {
                    self.calls.push(visitor.send(
                        PROXY,
                        Method::Get,
                        paths[row].as_bytes(),
                        &[(b"Host", b"one.example")],
                    ));
                } else {
                    let answer = self.last(visitor);
                    assert_eq!(
                        answer.status,
                        if row < 8 { 200 } else { 404 },
                        "owner/version/kind row {row}: {}",
                        paths[row]
                    );
                    service_checks::policy(answer, true);
                    declared(row, answer, project, task);
                    for marker in [
                        b"PRIVATE_SPEC_BODY".as_slice(),
                        b"PRIVATE_WORDS",
                        b"PRIVATE_RESULT",
                        b"PRIVATE_PARAMETER",
                        b"PRIVATE_PULL_TITLE",
                        b"PRIVATE_PULL_BODY",
                        b"owner-a",
                        b"fake-provider",
                    ] {
                        assert!(
                            !answer.body.windows(marker.len()).any(|bytes| bytes == marker),
                            "private source in row {row}: {marker:?}"
                        );
                    }
                    if (6..8).contains(&row) {
                        assert_eq!(answer.header(b"cache-control"), Some(b"no-store".as_slice()));
                    }
                    // Two read tokens refill between independently priced kinds.
                    self.wake = Some(Time::from_nanos(now.as_nanos() + 250_000_000));
                }
                self.stage += 1;
            }
        }
        false
    }
    fn cross_project(&mut self, visitor: &mut Visitor, project: u64) -> bool {
        use jig_client::contract as c;
        match self.stage {
            200 => self.calls.push(super::requests::ask(
                visitor,
                &c::Ask::MakeProject {
                    name: c::Text::new(b"Other public project".as_slice().into(), 64).expect("name"),
                    policy: 1,
                },
                171,
            )),
            201 => {
                assert!(matches!(
                    super::requests::answer(&self.last(visitor).body),
                    c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectMade { project: 2 }))
                ));
                self.calls.push(super::requests::ask(
                    visitor,
                    &c::Ask::SetVisibility { project: 2, public: true },
                    172,
                ));
            }
            202 => {
                assert!(matches!(super::requests::answer(&self.last(visitor).body), c::Answer::Done(_)));
                self.calls.push(visitor.send(
                    crate::fixtures::CLIENTS,
                    Method::Get,
                    b"/jig/v1/public/0/1/1",
                    &[(b"Host", b"one.example")],
                ));
            }
            203 => {
                let projects = p::PublicProjects::decode(&p::CEILINGS, &mut Reader::new(&self.last(visitor).body))
                    .expect("public projects");
                assert_eq!(projects.projects().len(), 2);
                let other_row = projects.projects().iter().find(|row| row.project() != project).expect("other project");
                assert_eq!(other_row.name(), b"Other public project", "the public record carries the project's name");
                let other = other_row.project();
                let task = self.task.expect("a genuinely public task of project one");
                for (address, task) in [(crate::fixtures::CLIENTS, task), (PROXY, task), (crate::fixtures::CLIENTS, 0)]
                {
                    self.calls.push(visitor.send(
                        address,
                        Method::Get,
                        format!("/jig/v1/public/0/1/3/{other:016x}/{task:016x}").as_bytes(),
                        &[(b"Host", b"one.example")],
                    ));
                }
            }
            204 => {
                let baseline = self.last(visitor);
                for id in self.calls.iter().rev().take(3) {
                    let answer = visitor.answers[id].as_ref().expect("cross-project answer");
                    assert_eq!(answer.status, 404, "a public task is still excluded from another public project");
                    assert!(answer.body.is_empty());
                    service_checks::policy(answer, false);
                    assert_eq!(
                        answer, baseline,
                        "a foreign public task is the same unknown directly and through the proxy"
                    );
                }
                self.calls.push(super::requests::ask(
                    visitor,
                    &c::Ask::SetVisibility { project: 2, public: false },
                    173,
                ));
            }
            205 => {
                assert!(matches!(super::requests::answer(&self.last(visitor).body), c::Answer::Done(_)));
                return true;
            }
            _ => unreachable!("cross-project stages"),
        }
        self.stage += 1;
        false
    }
}

fn declared(row: usize, answer: &Answer, project: u64, task: u64) {
    let mut reader = Reader::new(&answer.body);
    if !(6..8).contains(&row) {
        service_checks::cacheable(answer);
    }
    if row < 6 {
        assert_eq!(answer.header(b"content-type"), Some(b"application/octet-stream".as_slice()));
    }
    match row {
        0 => {
            let projects = p::PublicProjects::decode(&p::CEILINGS, &mut reader).expect("projects record");
            assert_eq!(projects.projects().iter().map(p::PublicProject::project).collect::<Vec<_>>(), [project]);
        }
        1 => {
            let board = p::PublicBoard::decode(&p::CEILINGS, &mut reader).expect("board record");
            assert_eq!(board.project(), project);
            assert_eq!(board.live().len() + board.ended().len(), 1);
        }
        2 => {
            let card = p::PublicCard::decode(&p::CEILINGS, &mut reader).expect("card record");
            assert_eq!(card.line().task(), task);
            assert_eq!(card.line().title(), b"@edit Public change title");
        }
        3..=5 => {
            let application =
                p::PublicApplication::decode(&p::CEILINGS, &mut reader).expect("forge record in its public envelope");
            assert_eq!(
                application.tasks().iter().copied().collect::<Vec<_>>(),
                [task],
                "only the public member is named; the landing's task stays private"
            );
            forge_references(row, application.record());
        }
        6 | 7 => {
            use jig_client_protocol_wire::v21 as w;
            let versions = w::NotServed::decode(&w::CEILINGS, &mut reader).expect("not-served record");
            assert_eq!(versions.served().len(), 1);
            let version = versions.served().iter().next().expect("served version");
            assert_eq!((version.first(), version.last()), (1, 1));
        }
        _ => assert!(answer.body.is_empty(), "unknown carries no body"),
    }
    assert!(reader.is_empty(), "the declared record consumes every byte");
}

// The public wire's leading fields bind task references to envelope places (temper/docs/design/client/forge/README.md, 9.4).
fn forge_references(row: usize, record: &[u8]) {
    let mut fields = Reader::new(record);
    assert_eq!(fields.u16(), Some(1), "the declared public record version");
    match row {
        3 => assert_eq!(fields.u16(), Some(0), "a change names its envelope place, never its internal task number"),
        4 => {
            assert_eq!(fields.u8(), Some(0), "the ended landing holds no turn");
            assert_eq!(fields.u32(), Some(2), "the conflicted landing and the later landed one");
            let mut landed = 0;
            for _ in 0..2 {
                assert_eq!(fields.u32(), Some(1), "one public member");
                assert_eq!(fields.u16(), Some(0), "member is the envelope's task");
                assert_eq!(fields.u8(), Some(5), "ended phase");
                let commits = fields.u32().expect("landed commits");
                assert!(commits <= 1);
                landed += commits;
                if commits == 1 {
                    fields.bytes(32).expect("public merge commit");
                }
            }
            assert_eq!(landed, 1, "only the later landing brought the member");
            assert_eq!(fields.u32(), Some(0), "ended changes occupy no queue");
            assert!(fields.is_empty());
        }
        5 => {
            assert_eq!(fields.u8(), Some(5), "landed phase");
            assert_eq!(fields.u8(), Some(0), "no landing hold");
            fields.u32().expect("preparation count");
            assert_eq!(fields.u32(), Some(1), "one public member");
            assert_eq!(fields.u16(), Some(0), "member is the envelope's task");
        }
        _ => unreachable!("forge kinds"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temper_engine_domain::{Key, Record};
    use temper_engine_domain_forge::{Key as ForgeKey, Stored};

    #[test]
    fn landing_readiness_finds_only_landed_keys_including_both_extremes() {
        let store = Rc::new(RefCell::new(Store::new()));
        let walk = Walk::new(store.clone());
        for (key, row) in [
            (ForgeKey::Names(2), Stored::Names { task: 2, resources: Box::default() }),
            (ForgeKey::ForgeNumber, Stored::ForgeNumber { last: 1 }),
        ] {
            store.borrow_mut().rows.insert(Key::Forge(key), Record::Forge { row: Box::new(row) });
        }
        assert!(!walk.landed(), "unrelated rows on either side are not a landing");
        for commit in [[0; 32], [u8::MAX; 32]] {
            let key = Key::Forge(ForgeKey::Landed(commit));
            store
                .borrow_mut()
                .rows
                .insert(key.clone(), Record::Forge { row: Box::new(Stored::Landed { commit, task: 2 }) });
            store.borrow_mut().applied += 1;
            assert!(walk.landed(), "every commit identifier can make the walk ready");
            assert!(walk.landed(), "the same durable commit remains ready");
            store.borrow_mut().rows.remove(&key);
            store.borrow_mut().applied += 1;
            assert!(!walk.landed(), "readiness reads the current committed rows");
        }
    }
}
