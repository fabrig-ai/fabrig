//! A private store survives opening public reading (jig/docs/design/domain/people/public-reading.md, 2).
use super::{checks::PROXY, proxy::Proxy, requests, service, service_checks, visitor::Visitor};
use crate::{
    ctl::{Caller, Command},
    fixtures,
    referee::Store,
};
use jig_client::contract as c;
use jig_client_protocol_wire::public_v1 as p;
use skein_http::Method;
use skein_lib::Reader;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
const RESTORED: &str = "a_store_written_before_public_reading_reads_every_project_private";
thread_local! {
    static BEFORE: Cell<bool> = const { Cell::new(false) };
    static IDS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}
pub(super) fn handles(name: &str) -> bool {
    matches!(name, "calibrate_projects" | "write_private_store") || name == RESTORED
}
pub(super) fn configure(value: &mut serde_json::Value) {
    if BEFORE.with(Cell::get) {
        for name in [
            "clients.public.reads",
            "clients.public.rate",
            "clients.public.rate_burst",
            "clients.public.bytes",
            "clients.public.bytes_burst",
            "clients.public.answer_bytes",
            "clients.public.max_age",
            "clients_http.public_calls",
            "core.public.goals",
            "core.public.ended",
            "core.public.lines",
            "core.public.depth",
        ] {
            value["engine"]["limits"]["overrides"][name] = 0.into();
        }
    }
}
pub(super) fn run(seed: u64) {
    // Only public answers calibrate identifiers; no secret record is inspected.
    service::run(seed, "calibrate_projects");
    BEFORE.with(|flag| flag.set(true));
    let mut disk = temper_playground::machine::fresh();
    temper_playground::machine::setup(&mut disk);
    let store = Rc::new(RefCell::new(Store::new()));
    let disk = service::phase(seed, "write_private_store", disk, store.clone());
    assert!(
        store.borrow().rows.values().all(|row| !matches!(
            row,
            temper_engine_domain::Record::Core(jig_domain_core::Record::People(
                jig_domain_people::Stored::Public { .. }
            ))
        )),
        "a store written with public figures at zero has no public project"
    );
    BEFORE.with(|flag| flag.set(false));
    store.borrow_mut().recovering();
    service::phase(seed, RESTORED, disk, store);
}
pub(super) struct Walk {
    name: String,
    stage: usize,
    handed: bool,
    calls: Vec<u64>,
}
impl Walk {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), stage: 0, handed: false, calls: Vec::new() }
    }
    pub fn act(
        &mut self,
        caller: &mut Caller,
        proxy: &Proxy,
        visitor: &mut Visitor,
        service: &mut temper_service::Service,
    ) -> bool {
        if !self.handed {
            let Some(link) = service.links().pop() else {
                return false;
            };
            if self.name == RESTORED {
                caller.commands.push_back(Command::new(&[b"sign-out"]));
            }
            let mut sign_in = Command::new(&[b"sign-in", b"-"]);
            sign_in.input = String::from_utf8(link.into_vec())
                .expect("printed URL")
                .replace("https://one.example", &format!("http://{}", fixtures::CLIENTS))
                .into_bytes()
                .into();
            caller.commands.push_back(sign_in);
            self.handed = true;
        }
        let signing = usize::from(self.name == RESTORED);
        if caller.answers.len() <= signing
            || !proxy.ready()
            || self.calls.iter().any(|id| !visitor.answers.contains_key(id))
        {
            return false;
        }
        assert_eq!(
            caller.answers[signing].exit,
            skein_io::kernel::Exit::Code(0),
            "sign-in: {}",
            String::from_utf8_lossy(&caller.answers[signing].error)
        );
        if self.name == RESTORED {
            return self.restored(visitor);
        }
        match self.stage {
            // Keep both projects within the shipped client's me receiving bound.
            0 => {
                self.calls.push(requests::ask(
                    visitor,
                    &c::Ask::MakeProject {
                        name: c::Text::new(b"Project 2".as_slice().into(), 64).expect("name"),
                        policy: 1,
                    },
                    152,
                ));
            }
            1 => {
                let answer = requests::answer(
                    &visitor.answers[self.calls.last().expect("create project")].as_ref().expect("HTTP response").body,
                );
                assert!(matches!(answer, c::Answer::Done(_)), "project made before opening: {answer:?}");
            }
            2 if self.name == "write_private_store" => return true,
            2..=3 => self.calls.push(requests::ask(
                visitor,
                &c::Ask::SetVisibility { project: u32::try_from(self.stage - 1).expect("project"), public: true },
                u8::try_from(160 + self.stage).expect("key"),
            )),
            4 => self.calls.push(visitor.send(
                fixtures::CLIENTS,
                Method::Get,
                b"/jig/v1/public/0/1/1",
                &[(b"Host", b"one.example")],
            )),
            5 => {
                let answer = visitor.answers[self.calls.last().expect("public projects")].as_ref().expect("answer");
                assert_eq!(answer.status, 200);
                let projects =
                    p::PublicProjects::decode(&p::CEILINGS, &mut Reader::new(&answer.body)).expect("projects");
                assert_eq!(projects.projects().len(), 2, "calibration exposes each real project's identifier");
                IDS.with(|ids| *ids.borrow_mut() = projects.projects().iter().map(p::PublicProject::project).collect());
                return true;
            }
            other => panic!("previous store stage {other}"),
        }
        self.stage += 1;
        false
    }
    fn restored(&mut self, visitor: &mut Visitor) -> bool {
        if self.stage == 0 {
            self.calls.push(visitor.send(
                fixtures::CLIENTS,
                Method::Get,
                b"/jig/v1/public/0/1/1",
                &[(b"Host", b"one.example")],
            ));
        } else if self.stage == 1 {
            let answer = visitor.answers[&self.calls[0]].as_ref().expect("projects answer");
            assert_eq!(answer.status, 200);
            service_checks::policy(answer, false);
            let projects = p::PublicProjects::decode(&p::CEILINGS, &mut Reader::new(&answer.body)).expect("projects");
            assert!(projects.projects().is_empty(), "opening an old store opens no project");
            for project in IDS.with(|ids| ids.borrow().clone()) {
                for address in [fixtures::CLIENTS, PROXY] {
                    self.calls.push(visitor.send(
                        address,
                        Method::Get,
                        format!("/jig/v1/public/0/1/2/{project:016x}").as_bytes(),
                        &[(b"Host", b"one.example")],
                    ));
                }
            }
        } else {
            assert_eq!(self.calls.len(), 5);
            let baseline = visitor.answers[&self.calls[1]].as_ref().expect("unknown");
            assert_eq!(baseline.status, 404);
            assert!(baseline.body.is_empty());
            for id in self.calls.iter().skip(1) {
                let answer = visitor.answers[id].as_ref().expect("private project's answer");
                assert_eq!(
                    answer, baseline,
                    "every restored project is the same unknown directly and through the proxy"
                );
                service_checks::policy(answer, false);
            }
            return true;
        }
        self.stage += 1;
        false
    }
}
