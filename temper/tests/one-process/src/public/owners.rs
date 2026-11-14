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
    fn last<'a>(&self, visitor: &'a Visitor) -> Result<&'a Answer, String> {
        Ok({
            visitor.answers[self.calls.last().ok_or_else(|| String::from("call"))?]
                .as_ref()
                .ok_or_else(|| String::from("answer"))?
        })
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
    ) -> Result<bool, String> {
        if self.wake.is_some_and(|wake| now < wake) || self.calls.iter().any(|id| !visitor.answers.contains_key(id)) {
            return Ok(false);
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
                    return Ok(false);
                }
                for answer in &caller.answers {
                    match (&(answer.exit), &(skein_io::kernel::Exit::Code(0))) {
                        (left, right) if left != right => {
                            let finding = format!(
                                "adopt before merge: {} {}",
                                String::from_utf8_lossy(&answer.output),
                                String::from_utf8_lossy(&answer.error)
                            );
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
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
                    return Ok(false);
                }
                for answer in &caller.answers {
                    match (&(answer.exit), &(skein_io::kernel::Exit::Code(0))) {
                        (left, right) if left != right => {
                            let finding = format!(
                                "public merge setup: {} {}",
                                String::from_utf8_lossy(&answer.output),
                                String::from_utf8_lossy(&answer.error)
                            );
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                }
                if !self.landing_complete() {
                    return Ok(false);
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
                let answer = self.last(visitor)?;
                match (&(answer.status), &(200)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let board = p::PublicBoard::decode(&p::CEILINGS, &mut Reader::new(&answer.body))
                    .map_err(|why| format!("board: {why:?}"))?;
                let goals: Vec<_> = board.live().iter().chain(board.ended().iter()).collect();
                if !super::TRACKED.with(core::cell::Cell::get) {
                    if !(goals.is_empty()) {
                        return Err(String::from("untracked change and its conversation delegates leave no goal"));
                    }
                    self.calls.push(visitor.send(
                        fixtures::CLIENTS,
                        Method::Get,
                        format!("/jig/v1/public/2/2/2/{project:016x}").as_bytes(),
                        &[(b"Host", b"one.example")],
                    ));
                    self.stage = 100;
                    return Ok(false);
                }
                match (&(goals.len()), &(1)) {
                    (left, right) if left != right => {
                        let finding = format!("one tracked goal, conversation excluded: {}", self.diagnostic());
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let goal = goals[0];
                super::identifiers::remember(
                    project,
                    goal.line().task(),
                    goal.tree().iter().map(p::PublicLine::task).collect(),
                );
                if !(goal.line().goal()) {
                    return Err(format!("assertion failed: {}", stringify!(goal.line().goal())));
                }
                match (&(goal.line().parent()), &(&None)) {
                    (left, right) if left != right => {
                        let finding = String::from("conversation ancestry never crosses");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(goal.line().title()), &(b"@edit Public change title")) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (
                    &(goal.line().delegates().working()
                        + goal.line().delegates().waiting()
                        + goal.line().delegates().held()
                        + goal.line().delegates().closing()
                        + goal.line().delegates().ended()),
                    &(goal.tree().len()),
                ) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.task = Some(goal.line().task());
                self.stage = 4;
            }
            100 => {
                let answer = self.last(visitor)?;
                match (&(answer.status), &(200)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                service_checks::policy(answer, false)?;
                let app = p::PublicApplication::decode(&p::CEILINGS, &mut Reader::new(&answer.body))
                    .map_err(|why| format!("untracked forge envelope: {why:?}"))?;
                if !(app.tasks().is_empty()) {
                    return Err(String::from("no excluded task enters the envelope"));
                }
                let mut fields = Reader::new(app.record());
                match (&(fields.u16()), &(Some(2))) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(fields.u8()), &(Some(0))) {
                    (left, right) if left != right => {
                        let finding = String::from("no excluded turn holder");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(fields.u32()), &(Some(0))) {
                    (left, right) if left != right => {
                        let finding = String::from("no excluded landing counted");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(fields.u32()), &(Some(0))) {
                    (left, right) if left != right => {
                        let finding = String::from("no excluded queue counted or ordered");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                if !(fields.is_empty()) {
                    return Err(format!("assertion failed: {}", stringify!(fields.is_empty())));
                }
                return Ok(true);
            }
            200..=205 => return self.cross_project(visitor, project),
            _ => {
                let row = (self.stage - 4) / 2;
                let task = self.task.ok_or_else(|| String::from("public change"))?;
                let paths = [
                    "/jig/v1/public/0/1/1".to_owned(),
                    format!("/jig/v1/public/0/1/2/{project:016x}"),
                    format!("/jig/v1/public/0/1/3/{project:016x}/{task:016x}"),
                    format!("/jig/v1/public/2/2/1/{project:016x}/{task:016x}"),
                    format!("/jig/v1/public/2/2/2/{project:016x}"),
                    format!("/jig/v1/public/2/2/3/{project:016x}/{task:016x}"),
                    "/jig/v1/public/0/2/1".to_owned(),
                    format!("/jig/v1/public/2/1/2/{project:016x}"),
                    "/jig/v1/public/0/1/65535".to_owned(),
                    format!("/jig/v1/public/65535/1/2/{project:016x}"),
                ];
                if row == paths.len() {
                    self.stage = 200;
                    return Ok(false);
                }
                if (self.stage - 4).is_multiple_of(2) {
                    self.calls.push(visitor.send(
                        PROXY,
                        Method::Get,
                        paths[row].as_bytes(),
                        &[(b"Host", b"one.example")],
                    ));
                } else {
                    let answer = self.last(visitor)?;
                    match (&(answer.status), &(if row < 8 { 200 } else { 404 })) {
                        (left, right) if left != right => {
                            let finding = format!("owner/version/kind row {row}: {}", paths[row]);
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    service_checks::policy(answer, true)?;
                    declared(row, answer, project, task)?;
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
                        if answer.body.windows(marker.len()).any(|bytes| bytes == marker) {
                            return Err(format!("private source in row {row}: {marker:?}"));
                        }
                    }
                    if (6..8).contains(&row) {
                        match (&(answer.header(b"cache-control")), &(Some(b"no-store".as_slice()))) {
                            (left, right) if left != right => {
                                let finding = String::from("assertion `left == right` failed");
                                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                            }
                            _ => {}
                        }
                    }
                    // Two read tokens refill between independently priced kinds.
                    self.wake = Some(Time::from_nanos(now.as_nanos() + 250_000_000));
                }
                self.stage += 1;
            }
        }
        Ok(false)
    }
    #[expect(clippy::too_many_lines, reason = "the ordered referee checks return their findings at each step")]
    fn cross_project(&mut self, visitor: &mut Visitor, project: u64) -> Result<bool, String> {
        use jig_client::contract as c;
        match self.stage {
            200 => self.calls.push(super::requests::ask(
                visitor,
                &c::Ask::MakeProject {
                    name: c::Text::new(b"Other public project".as_slice().into(), 64)
                        .map_err(|why| format!("name: {why:?}"))?,
                    policy: 1,
                },
                171,
            )),
            201 => {
                if !(matches!(
                    super::requests::answer(&self.last(visitor)?.body)?,
                    c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectMade { project: 2 }))
                )) {
                    return Err(format!(
                        "assertion failed: {}",
                        stringify!(matches!(
                            super::requests::answer(&self.last(visitor)?.body)?,
                            c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectMade { project: 2 }))
                        ))
                    ));
                }
                self.calls.push(super::requests::ask(
                    visitor,
                    &c::Ask::SetVisibility { project: 2, public: true },
                    172,
                ));
            }
            202 => {
                if !(matches!(super::requests::answer(&self.last(visitor)?.body)?, c::Answer::Done(_))) {
                    return Err(format!(
                        "assertion failed: {}",
                        stringify!(matches!(super::requests::answer(&self.last(visitor)?.body)?, c::Answer::Done(_)))
                    ));
                }
                self.calls.push(visitor.send(
                    crate::fixtures::CLIENTS,
                    Method::Get,
                    b"/jig/v1/public/0/1/1",
                    &[(b"Host", b"one.example")],
                ));
            }
            203 => {
                let projects = p::PublicProjects::decode(&p::CEILINGS, &mut Reader::new(&self.last(visitor)?.body))
                    .map_err(|why| format!("public projects: {why:?}"))?;
                match (&(projects.projects().len()), &(2)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let other_row = projects
                    .projects()
                    .iter()
                    .find(|row| row.project() != project)
                    .ok_or_else(|| String::from("other project"))?;
                match (&(other_row.name()), &(b"Other public project")) {
                    (left, right) if left != right => {
                        let finding = String::from("the public record carries the project's name");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let other = other_row.project();
                let task = self.task.ok_or_else(|| String::from("a genuinely public task of project one"))?;
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
                let baseline = self.last(visitor)?;
                for id in self.calls.iter().rev().take(3) {
                    let answer = visitor.answers[id].as_ref().ok_or_else(|| String::from("cross-project answer"))?;
                    match (&(answer.status), &(404)) {
                        (left, right) if left != right => {
                            let finding = String::from("a public task is still excluded from another public project");
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    if !(answer.body.is_empty()) {
                        return Err(format!("assertion failed: {}", stringify!(answer.body.is_empty())));
                    }
                    service_checks::policy(answer, false)?;
                    match (&(answer), &(baseline)) {
                        (left, right) if left != right => {
                            let finding = String::from(
                                "a foreign public task is the same unknown directly and through the proxy",
                            );
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                }
                self.calls.push(super::requests::ask(
                    visitor,
                    &c::Ask::SetVisibility { project: 2, public: false },
                    173,
                ));
            }
            205 => {
                if !(matches!(super::requests::answer(&self.last(visitor)?.body)?, c::Answer::Done(_))) {
                    return Err(format!(
                        "assertion failed: {}",
                        stringify!(matches!(super::requests::answer(&self.last(visitor)?.body)?, c::Answer::Done(_)))
                    ));
                }
                return Ok(true);
            }
            _ => return Err(String::from("cross-project stages")),
        }
        self.stage += 1;
        Ok(false)
    }
}

#[expect(clippy::too_many_lines, reason = "the ordered referee checks return their findings at each step")]
fn declared(row: usize, answer: &Answer, project: u64, task: u64) -> Result<(), String> {
    let mut reader = Reader::new(&answer.body);
    if !(6..8).contains(&row) {
        service_checks::cacheable(answer)?;
    }
    if row < 6 {
        match (&(answer.header(b"content-type")), &(Some(b"application/octet-stream".as_slice()))) {
            (left, right) if left != right => {
                let finding = String::from("assertion `left == right` failed");
                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
            }
            _ => {}
        }
    }
    match row {
        0 => {
            let projects = p::PublicProjects::decode(&p::CEILINGS, &mut reader)
                .map_err(|why| format!("projects record: {why:?}"))?;
            match (&(projects.projects().iter().map(p::PublicProject::project).collect::<Vec<_>>()), &([project])) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
        }
        1 => {
            let board =
                p::PublicBoard::decode(&p::CEILINGS, &mut reader).map_err(|why| format!("board record: {why:?}"))?;
            match (&(board.project()), &(project)) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            match (&(board.live().len() + board.ended().len()), &(1)) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
        }
        2 => {
            let card =
                p::PublicCard::decode(&p::CEILINGS, &mut reader).map_err(|why| format!("card record: {why:?}"))?;
            match (&(card.line().task()), &(task)) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            match (&(card.line().title()), &(b"@edit Public change title")) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
        }
        3..=5 => {
            let application = p::PublicApplication::decode(&p::CEILINGS, &mut reader)
                .map_err(|why| format!("forge record in its public envelope: {why:?}"))?;
            match (&(application.tasks().iter().copied().collect::<Vec<_>>()), &([task])) {
                (left, right) if left != right => {
                    let finding = String::from("only the public member is named; the landing's task stays private");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            forge_references(row, application.record())?;
        }
        6 | 7 => {
            use jig_client_protocol_wire::v22 as w;
            let versions =
                w::NotServed::decode(&w::CEILINGS, &mut reader).map_err(|why| format!("not-served record: {why:?}"))?;
            match (&(versions.served().len()), &(1)) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            let version = versions.served().iter().next().ok_or_else(|| String::from("served version"))?;
            match (&(version.first(), version.last()), &(if row == 6 { (1, 1) } else { (2, 2) })) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
        }
        _ => {
            if !(answer.body.is_empty()) {
                return Err(String::from("unknown carries no body"));
            }
        }
    }
    if !(reader.is_empty()) {
        return Err(String::from("the declared record consumes every byte"));
    }

    Ok(())
}

// The public wire's leading fields bind task references to envelope places (temper/docs/design/client/forge/README.md, 9.4).
#[expect(clippy::too_many_lines, reason = "the ordered referee checks return their findings at each step")]
fn forge_references(row: usize, record: &[u8]) -> Result<(), String> {
    let mut fields = Reader::new(record);
    match (&(fields.u16()), &(Some(2))) {
        (left, right) if left != right => {
            let finding = String::from("the declared public record version");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }
    match row {
        3 => match (&(fields.u16()), &(Some(0))) {
            (left, right) if left != right => {
                let finding = String::from("a change names its envelope place, never its internal task number");
                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
            }
            _ => {}
        },
        4 => {
            match (&(fields.u8()), &(Some(0))) {
                (left, right) if left != right => {
                    let finding = String::from("the ended landing holds no turn");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            match (&(fields.u32()), &(Some(2))) {
                (left, right) if left != right => {
                    let finding = String::from("the conflicted landing and the later landed one");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            let mut landed = 0;
            for _ in 0..2 {
                match (&(fields.u32()), &(Some(1))) {
                    (left, right) if left != right => {
                        let finding = String::from("one public member");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(fields.u16()), &(Some(0))) {
                    (left, right) if left != right => {
                        let finding = String::from("member is the envelope's task");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(fields.u8()), &(Some(5))) {
                    (left, right) if left != right => {
                        let finding = String::from("ended phase");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let commits = fields.u32().ok_or_else(|| String::from("landed commits"))?;
                if commits > 1 {
                    return Err(format!("assertion failed: {}", stringify!(commits <= 1)));
                }
                landed += commits;
                if commits == 1 {
                    fields.bytes(32).ok_or_else(|| String::from("public merge commit"))?;
                }
            }
            match (&(landed), &(1)) {
                (left, right) if left != right => {
                    let finding = String::from("only the later landing brought the member");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            match (&(fields.u32()), &(Some(0))) {
                (left, right) if left != right => {
                    let finding = String::from("ended changes occupy no queue");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            if !(fields.is_empty()) {
                return Err(format!("assertion failed: {}", stringify!(fields.is_empty())));
            }
        }
        5 => {
            match (&(fields.u8()), &(Some(5))) {
                (left, right) if left != right => {
                    let finding = String::from("landed phase");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            match (&(fields.u8()), &(Some(0))) {
                (left, right) if left != right => {
                    let finding = String::from("no landing hold");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            fields.u32().ok_or_else(|| String::from("preparation count"))?;
            match (&(fields.u32()), &(Some(1))) {
                (left, right) if left != right => {
                    let finding = String::from("one public member");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            match (&(fields.u16()), &(Some(0))) {
                (left, right) if left != right => {
                    let finding = String::from("member is the envelope's task");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
        }
        _ => return Err(String::from("forge kinds")),
    }

    Ok(())
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
