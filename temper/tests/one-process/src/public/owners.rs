//! A public goal read at both owners (temper/docs/design/client/web/public.md, 4).
use super::{
    checks::PROXY, deployment, forge_target, http::Answer, jig_target, proxy::Proxy, service, service_checks, target,
    visitor::Visitor,
};
use crate::{ctl::Caller, fixtures, referee::Store};
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire::public_v1 as p;
use skein_http::Method;
use skein_lib::{Reader, Time};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(super) const DECLARED: &str = "every_owner_version_and_kind_answers_as_declared";
pub(super) const REVOKED: &str = "a_project_made_private_answers_its_card_and_change_unknown";
const CHAT_TITLE: &str = "@plan Private conversation title";
const CHAT_REQUEST: &str = "PRIVATE_WORDS";
const PUBLIC_PROJECT: u32 = 1;
const PRIVATE_PROJECT: u32 = 2;
const OTHER_PUBLIC_PROJECT: u32 = 3;
const PRIVATE_MARKERS: &[&[u8]] = &[
    CHAT_TITLE.as_bytes(),
    CHAT_REQUEST.as_bytes(),
    deployment::CHAT_REPLY,
    b"PRIVATE_SPEC_BODY",
    b"PRIVATE_RESULT",
    b"PRIVATE_PARAMETER",
    b"PRIVATE_PULL_TITLE",
    b"PRIVATE_PULL_BODY",
    deployment::PRIVATE_PROJECT.as_bytes(),
    deployment::OTHER_TASK_TITLE.as_bytes(),
    deployment::SCRIPTED_REPORT.as_bytes(),
    b"owner",
    b"owner-a",
    b"acme",
    b"acme/app",
    b"main",
    b"temper/",
    fixtures::MERGE_BRANCH,
    deployment::REVIEW_TITLE.as_bytes(),
    deployment::REVIEW_REPORT.as_bytes(),
    deployment::FORGE_WEB.as_bytes(),
    b"https://",
    b"fake-provider",
];

pub(super) struct Walk {
    revoke: bool,
    store: Rc<RefCell<Store>>,
    stage: usize,
    calls: Vec<u64>,
    task: Option<u64>,
    wake: Option<Time>,
    landing: Cell<Option<(u64, bool)>>,
}
impl Walk {
    pub fn new(store: Rc<RefCell<Store>>, revoke: bool) -> Self {
        Self { revoke, store, stage: 0, calls: Vec::new(), task: None, wake: None, landing: Cell::new(None) }
    }
    pub fn ready(&self, caller: &Caller, visitor: &Visitor) -> bool {
        match self.stage {
            0 => true,
            1 => caller.answers.len() == 6,
            2 => caller.answers.len() == self.setup_answers() && self.change_ready(),
            50 => caller.answers.len() == self.setup_answers() + 1 && self.side_report_ready(),
            _ => self.calls.iter().all(|id| visitor.answers.contains_key(id)),
        }
    }
    fn setup_answers(&self) -> usize {
        // Sign-in, publish, account, connect, list, adopt, chat; the sweep adds a private project.
        if self.revoke { 7 } else { 8 }
    }
    fn change_ready(&self) -> bool {
        if self.revoke {
            use temper_engine_domain::Key;
            use temper_engine_domain_forge::Key::ChangeHot;
            self.store.borrow().rows.range(Key::Forge(ChangeHot(0))..=Key::Forge(ChangeHot(u64::MAX))).next().is_some()
        } else {
            self.landing_complete()
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
        if !self.landed() {
            return false;
        }
        let mut ended_landing = false;
        for record in self.store.borrow().rows.values() {
            if let temper_engine_domain::Record::Forge { row } = record
                && let temper_engine_domain_forge::Stored::Landing(row) = row.as_ref()
            {
                match row.landing.phase {
                    temper_engine_domain_forge::LandingPhase::Landed => ended_landing = true,
                    temper_engine_domain_forge::LandingPhase::Cancelled => {}
                    _ => return false,
                }
            }
        }
        ended_landing
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
        let landings: Vec<_> = self
            .store
            .borrow()
            .rows
            .values()
            .filter_map(|row| {
                if let temper_engine_domain::Record::Forge { row } = row
                    && let temper_engine_domain_forge::Stored::Landing(row) = row.as_ref()
                {
                    Some((
                        row.landing.task,
                        &row.landing.phase,
                        row.landing.preparation,
                        row.landing
                            .members
                            .iter()
                            .map(|member| (member.change, member.asked, member.candidate.is_some(), member.left))
                            .collect::<Vec<_>>(),
                    ))
                } else {
                    None
                }
            })
            .map(|row| format!("{row:?}"))
            .collect();
        format!("merge stage {}, tasks {tasks:?}, landings {landings:?}", self.stage)
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
                        "--web",
                        deployment::FORGE_WEB,
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
                // A turn's merged members are detected before their changes close.
                skein_fake_forge_domain::manual_merges(
                    &mut transport.forge.borrow_mut().domain,
                    b"acme/app",
                    skein_fake_forge_domain::api::ManualMerges {
                        allow_manual_merge: true,
                        autodetect_manual_merge: true,
                    },
                );
                service::command(
                    caller,
                    &[
                        "chat",
                        "start",
                        "--project",
                        "1",
                        "--key",
                        "public-chat",
                        &format!("{CHAT_TITLE}\n{CHAT_REQUEST}"),
                    ],
                );
                if !self.revoke {
                    service::command(
                        caller,
                        &["project", "make", deployment::PRIVATE_PROJECT, "--key", "private-project"],
                    );
                }
                self.stage = 2;
            }
            2 => {
                if caller.answers.len() != self.setup_answers() {
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
                if !self.change_ready() {
                    return Ok(false);
                }
                self.calls.push(visitor.send(
                    fixtures::CLIENTS,
                    Method::Get,
                    jig_target(&format!("2/{project:016x}")).as_bytes(),
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
                let goals: Vec<_> = board.live.iter().chain(board.ended.iter()).collect();
                match (&(goals.len()), &(1)) {
                    (left, right) if left != right => {
                        let finding = format!("one tracked goal, conversation excluded: {}", self.diagnostic());
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let goal = goals[0];
                if !(goal.line.goal) {
                    return Err(format!("assertion failed: {}", stringify!(goal.line().goal())));
                }
                match (&(&goal.line.parent), &(&None)) {
                    (left, right) if left != right => {
                        let finding = String::from("conversation ancestry never crosses");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(goal.line.title.as_ref()), &(b"@edit Public change title")) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (
                    &(goal.line.delegates.working
                        + goal.line.delegates.waiting
                        + goal.line.delegates.held
                        + goal.line.delegates.closing
                        + goal.line.delegates.ended),
                    &(goal.tree.len()),
                ) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.task = Some(goal.line.task);
                if self.revoke {
                    self.stage = 100;
                    // The card and change each draw on the public byte budget.
                    self.wake = Some(now.saturating_add(skein_lib::Duration::from_secs(1)));
                } else {
                    service::command(
                        caller,
                        &["chat", "start", "--project", "1", "--key", "private-report", deployment::OTHER_TASK_TITLE],
                    );
                    self.stage = 50;
                }
            }
            50 => {
                if caller.answers.len() != self.setup_answers() + 1 {
                    return Ok(false);
                }
                let answer = caller.answers.last().expect("side conversation answer");
                if answer.exit != skein_io::kernel::Exit::Code(0) {
                    return Err(format!("side conversation: {}", String::from_utf8_lossy(&answer.error)));
                }
                if !self.side_report_ready() {
                    return Ok(false);
                }
                self.private_sources()?;
                self.stage = 4;
            }
            100..=104 => return self.revoke(now, visitor, project),
            200..=205 => return self.cross_project(visitor, project),
            _ => {
                let row = (self.stage - 4) / 2;
                let task = self.task.ok_or_else(|| String::from("public change"))?;
                let paths = [
                    jig_target("1"),
                    jig_target(&format!("2/{project:016x}")),
                    jig_target(&format!("3/{project:016x}/{task:016x}")),
                    forge_target(&format!("1/{project:016x}/{task:016x}")),
                    forge_target(&format!("2/{project:016x}")),
                    forge_target(&format!("3/{project:016x}/{task:016x}")),
                    target(0, temper_client_protocol_wire::PUBLIC_LAYOUT, "1"),
                    target(2, jig_client_protocol_wire::PUBLIC_LAYOUT, &format!("2/{project:016x}")),
                    jig_target("65535"),
                    target(u16::MAX, jig_client_protocol_wire::PUBLIC_LAYOUT, &format!("2/{project:016x}")),
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
                            let finding = format!("owner/layout/kind row {row}: {}", paths[row]);
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    service_checks::policy(answer, true)?;
                    declared(row, answer, project, task)?;
                    if let Some(marker) = PRIVATE_MARKERS.iter().find(|marker| contains(&answer.body, marker)) {
                        return Err(format!("private source in {}: {}", paths[row], String::from_utf8_lossy(marker)));
                    }
                    // A bare repository name is private, not its letters in "appended".
                    if answer.body.split(|byte| !byte.is_ascii_alphanumeric()).any(|word| word == b"app") {
                        return Err(format!("private repository name in {}", paths[row]));
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
    fn side_report_ready(&self) -> bool {
        use jig_domain_tasks::{Ending, Party, Phase, Stored, TaskResult};
        self.store.borrow().rows.values().any(|record| matches!(record,
            temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(Stored::Ended(task)))
                if task.project == PUBLIC_PROJECT && task.tracked.is_none() && matches!(task.requester, Party::Person(_))
                    && task.spec.words.as_ref() == deployment::OTHER_TASK_TITLE.as_bytes()
                    && matches!(&task.phase, Phase::Ended(Ending::Done(TaskResult::Report { words }))
                        if words.as_ref() == deployment::SCRIPTED_REPORT.as_bytes())))
    }
    fn private_sources(&self) -> Result<(), String> {
        use jig_domain_people::{Key as PeopleKey, Stored as PeopleRow};
        use temper_engine_domain::{Key, Record};
        use temper_engine_domain_forge::{Key as ForgeKey, Stored};
        let store = self.store.borrow();
        let chat = store.rows.values().any(|record| {
            matches!(record,
            Record::Core(jig_domain_core::Record::Tasks(
                jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task)
            )) if task.flags.conversation && task.spec.words.starts_with(CHAT_TITLE.as_bytes())
                && contains(&task.spec.words, CHAT_REQUEST.as_bytes()) && !task.replies.is_empty())
        });
        if !chat {
            return Err(String::from("no committed conversation request and reply in fixture"));
        }
        // The reviewer task's title is public in the goal's tree. Its verdict is not.
        let reviewed = store
            .rows
            .range(Key::Forge(ForgeKey::ChangeHot(0))..=Key::Forge(ForgeKey::ChangeHot(u64::MAX)))
            .any(|(_, record)| {
                let Record::Forge { row } = record else { return false };
                let Stored::ChangeHot(row) = row.as_ref() else { return false };
                row.gate_remarks.iter().any(|review| {
                    let mut lines = review.words.split(|byte| *byte == b'\n');
                    lines.next() == Some(deployment::REVIEW_TITLE.as_bytes())
                        && lines.next() == Some(deployment::REVIEW_REPORT.as_bytes())
                })
            });
        if !reviewed {
            return Err(String::from("no stored verdict with the private review heading and report"));
        }
        let address = store.rows.values().any(|record| matches!(record, Record::Forge { row }
            if matches!(row.as_ref(), Stored::Forge(forge) if forge.place.web.as_deref() == Some(deployment::FORGE_WEB.as_bytes()))));
        if !address {
            return Err(String::from("no private forge web address in fixture"));
        }
        let named_private = matches!(
            store.rows.get(&Key::Core(jig_domain_core::Key::People(PeopleKey::Policy(PRIVATE_PROJECT)))),
            Some(Record::Core(jig_domain_core::Record::People(PeopleRow::Policy { name, .. })))
                if name.as_ref() == deployment::PRIVATE_PROJECT
        );
        if !named_private {
            return Err(String::from("no private project name in fixture"));
        }
        Ok(())
    }
    fn revoke(&mut self, now: Time, visitor: &mut Visitor, project: u64) -> Result<bool, String> {
        use jig_client::contract as c;
        const CARD: usize = 2;
        const CHANGE: usize = 3;
        let task = self.task.ok_or_else(|| String::from("public task"))?;
        let paths = [
            jig_target(&format!("3/{project:016x}/{task:016x}")),
            forge_target(&format!("1/{project:016x}/{task:016x}")),
        ];
        match self.stage {
            100 | 103 => {
                if self.stage == 103 {
                    let outcome = super::requests::answer(&self.last(visitor)?.body)?;
                    if !matches!(
                        outcome,
                        c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::VisibilitySet {
                            project: PUBLIC_PROJECT,
                            public: false,
                        }))
                    ) {
                        return Err(format!("owner makes project private: {outcome:?}"));
                    }
                }
                self.calls.clear();
                for path in &paths {
                    self.calls.push(visitor.send(
                        fixtures::CLIENTS,
                        Method::Get,
                        path.as_bytes(),
                        &[(b"Host", b"one.example")],
                    ));
                }
            }
            101 | 104 => {
                let revoked = self.stage == 104;
                let status = if revoked { 404 } else { 200 };
                for ((id, path), kind) in self.calls.iter().zip(&paths).zip([CARD, CHANGE]) {
                    let answer = visitor.answers[id].as_ref().ok_or_else(|| format!("no HTTP answer for {path}"))?;
                    if answer.status != status {
                        return Err(format!("{path}, revoked {revoked}: expected {status}, got {}", answer.status));
                    }
                    service_checks::policy(answer, false)?;
                    if revoked {
                        service_checks::cacheable(answer)?;
                        if !answer.body.is_empty() {
                            return Err(format!("{path}: Unknown has no body"));
                        }
                    } else {
                        declared(kind, answer, project, task)?;
                    }
                }
                if revoked {
                    return Ok(true);
                }
                // Refill the visitor's budget before revoking, so the next reads follow the committed answer.
                self.wake = Some(now.saturating_add(skein_lib::Duration::from_secs(1)));
            }
            102 => self.calls.push(super::requests::ask(
                visitor,
                &c::Ask::SetVisibility { project: PUBLIC_PROJECT, public: false },
                174,
            )),
            other => return Err(format!("card and change revocation stage {other}")),
        }
        self.stage += 1;
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
                    c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectMade {
                        project: OTHER_PUBLIC_PROJECT
                    }))
                )) {
                    return Err(format!(
                        "assertion failed: {}",
                        stringify!(matches!(
                            super::requests::answer(&self.last(visitor)?.body)?,
                            c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectMade {
                                project: OTHER_PUBLIC_PROJECT
                            }))
                        ))
                    ));
                }
                self.calls.push(super::requests::ask(
                    visitor,
                    &c::Ask::SetVisibility { project: OTHER_PUBLIC_PROJECT, public: true },
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
                    jig_target("1").as_bytes(),
                    &[(b"Host", b"one.example")],
                ));
            }
            203 => {
                let projects = p::PublicProjects::decode(&p::CEILINGS, &mut Reader::new(&self.last(visitor)?.body))
                    .map_err(|why| format!("public projects: {why:?}"))?;
                match (&(projects.projects.len()), &(2)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let other_row = projects
                    .projects
                    .iter()
                    .find(|row| row.project != project)
                    .ok_or_else(|| String::from("other project"))?;
                match (&(other_row.name.as_ref()), &(b"Other public project")) {
                    (left, right) if left != right => {
                        let finding = String::from("the public record carries the project's name");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let other = other_row.project;
                let task = self.task.ok_or_else(|| String::from("a genuinely public task of project one"))?;
                for (address, task) in [(crate::fixtures::CLIENTS, task), (PROXY, task), (crate::fixtures::CLIENTS, 0)]
                {
                    self.calls.push(visitor.send(
                        address,
                        Method::Get,
                        jig_target(&format!("3/{other:016x}/{task:016x}")).as_bytes(),
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
                    &c::Ask::SetVisibility { project: OTHER_PUBLIC_PROJECT, public: false },
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

fn contains(bytes: &[u8], marker: &[u8]) -> bool {
    bytes.windows(marker.len()).any(|part| part == marker)
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
            match (&(projects.projects.iter().map(|value| value.project).collect::<Vec<_>>()), &([project])) {
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
            match (&(board.project), &(project)) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            match (&(board.live.len() + board.ended.len()), &(1)) {
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
            match (&(card.line.task), &(task)) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            match (&(card.line.title.as_ref()), &(b"@edit Public change title")) {
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
            match (&(application.tasks.iter().copied().collect::<Vec<_>>()), &([task])) {
                (left, right) if left != right => {
                    let finding = String::from("only the public member is named; the landing's task stays private");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            forge_references(row, application.record.as_ref())?;
        }
        6 | 7 => {
            use jig_client_protocol_wire as w;
            let versions =
                w::NotServed::decode(&w::CEILINGS, &mut reader).map_err(|why| format!("not-served record: {why:?}"))?;
            match (&(versions.served.len()), &(1)) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            let version = versions.served.iter().next().ok_or_else(|| String::from("served version"))?;
            match (&(version.first, version.last), &(if row == 6 { (1, 1) } else { (4, 4) })) {
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
    match (&(fields.u16()), &(Some(4))) {
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
        let walk = Walk::new(store.clone(), false);
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
