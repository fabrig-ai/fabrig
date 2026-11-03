//! A landed public goal read at both owners (temper/docs/design/protocol/sign-in.md, 2.5).
use super::{checks::PROXY, http::Answer, proxy::Proxy, service, service_checks, visitor::Visitor};
use crate::{ctl::Caller, fixtures, referee::Store};
use jig_client_protocol_wire::public_v1 as p;
use skein_http::Method;
use skein_lib::{Reader, Time};
use std::{cell::RefCell, rc::Rc};

pub(super) struct Walk {
    store: Rc<RefCell<Store>>,
    stage: usize,
    calls: Vec<u64>,
    task: Option<u64>,
    wake: Option<Time>,
}
impl Walk {
    pub fn new(store: Rc<RefCell<Store>>) -> Self {
        Self { store, stage: 0, calls: Vec::new(), task: None, wake: None }
    }
    pub fn ready(&self, caller: &Caller, visitor: &Visitor) -> bool {
        match self.stage {
            0 => true,
            1 => caller.answers.len() == 6,
            2 => caller.answers.len() == 7 && self.landed(),
            _ => self.calls.iter().all(|id| visitor.answers.contains_key(id)),
        }
    }
    fn landed(&self) -> bool {
        self.store.borrow().rows.values().any(|row| matches!(row, temper_engine_domain::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Landed { .. })))
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
                let landed = self.store.borrow().rows.values().any(|row| matches!(row, temper_engine_domain::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Landed { .. })));
                if !landed {
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
                assert_eq!(goals.len(), 1, "one tracked goal, conversation excluded: {}", self.diagnostic());
                let goal = goals[0];
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
            _ => {
                let row = (self.stage - 4) / 2;
                let task = self.task.expect("public change");
                let paths = [
                    "/jig/v1/public/0/1/1".to_owned(),
                    format!("/jig/v1/public/0/1/2/{project:016x}"),
                    format!("/jig/v1/public/0/1/3/{project:016x}/{task:016x}"),
                    format!("/jig/v1/public/1/1/1/{project:016x}/{task:016x}"),
                    format!("/jig/v1/public/1/1/2/{project:016x}"),
                    format!("/jig/v1/public/1/1/3/{project:016x}/{task:016x}"),
                    "/jig/v1/public/0/2/1".to_owned(),
                    format!("/jig/v1/public/1/2/2/{project:016x}"),
                    "/jig/v1/public/0/1/65535".to_owned(),
                    format!("/jig/v1/public/65535/1/2/{project:016x}"),
                ];
                if row == paths.len() {
                    return true;
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
                    // Refill the eight-read bucket between independently priced kinds.
                    self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
                }
                self.stage += 1;
            }
        }
        false
    }
}
