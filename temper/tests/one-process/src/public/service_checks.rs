//! Service answers observed through HTTP (temper/docs/design/protocol/sign-in.md, 2.5).
use super::{
    checks,
    http::Answer,
    jig_target,
    service::{self, Extra, Proc},
    visitor::Visitor,
};
use crate::{ctl::Command, fixtures, world::HarnessProcess};
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire::public_v1 as public;
use skein_http::Method;
use skein_lib::{Reader, Time};
use std::net::SocketAddr;

const HOST: &[u8] = b"one.example";
pub(super) fn policy(answer: &Answer, cached: bool) -> Result<(), String> {
    checks::no_cors(answer)?;
    match (&(answer.header(b"x-content-type-options")), &(Some(b"nosniff".as_slice()))) {
        (left, right) if left != right => {
            let finding = String::from("assertion `left == right` failed");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }
    for name in [b"set-cookie".as_slice(), b"vary", b"etag", b"last-modified"] {
        if answer.header(name).is_some() {
            return Err(format!("forbidden public header {name:?}: {answer:?}"));
        }
    }
    if (!cached) && (answer.header(b"age").is_some()) {
        return Err(String::from("service never adds Age"));
    }
    if let Some(control) = answer.header(b"cache-control")
        && (control.windows(6).any(|w| w == b"stale-"))
    {
        return Err(String::from("no stale directive"));
    }

    Ok(())
}
pub(super) fn cacheable(answer: &Answer) -> Result<(), String> {
    let control = std::str::from_utf8(answer.header(b"cache-control").ok_or_else(|| String::from("cache control"))?)
        .map_err(|why| format!("ASCII policy: {why:?}"))?;
    if !(control.starts_with("public, max-age=")) {
        return Err(control.to_owned());
    }
    if !(control.ends_with(", must-revalidate")) {
        return Err(control.to_owned());
    }
    let age = control
        .strip_prefix("public, max-age=")
        .ok_or_else(|| String::from("prefix"))?
        .split(',')
        .next()
        .ok_or_else(|| String::from("age"))?
        .parse::<u32>()
        .map_err(|why| format!("seconds: {why:?}"))?;
    if age > 2 {
        return Err(String::from("deployment's max-age"));
    }

    Ok(())
}
#[expect(clippy::struct_excessive_bools, reason = "handover, setup, readiness and shutdown advance independently")]
pub(super) struct Judge {
    finding: Option<String>,
    seed: u64,
    name: String,
    controls: skein_world::Controls,
    handed: bool,
    setup: bool,
    stage: usize,
    calls: Vec<u64>,
    stopping: bool,
    ready: bool,
    project: Option<u64>,
    owners: super::owners::Walk,
    load: super::load::Walk,
    abandoned: super::abandoned::Walk,
    revocation: super::revocation::Walk,
    previous: super::previous::Walk,
}
impl Judge {
    pub fn new(
        seed: u64,
        name: &str,
        controls: skein_world::Controls,
        store: std::rc::Rc<std::cell::RefCell<crate::referee::Store>>,
    ) -> Self {
        Self {
            finding: None,
            seed,
            name: name.into(),
            controls,
            handed: false,
            setup: false,
            stage: 0,
            calls: Vec::new(),
            stopping: false,
            ready: false,
            project: None,
            owners: super::owners::Walk::new(store, name == super::owners::REVOKED),
            load: super::load::Walk::new(),
            abandoned: super::abandoned::Walk::new(),
            revocation: super::revocation::Walk::new(),
            previous: super::previous::Walk::new(name),
        }
    }
    fn send(&mut self, visitor: &mut Visitor, address: SocketAddr, method: Method, path: &[u8]) {
        self.calls.push(visitor.send(address, method, path, &[(b"Host", HOST)]));
    }
    fn last<'a>(&self, visitor: &'a Visitor) -> Result<&'a Answer, String> {
        Ok({
            visitor
                .answers
                .get(self.calls.last().ok_or_else(|| String::from("sent request"))?)
                .ok_or_else(|| String::from("completed request"))?
                .as_ref()
                .ok_or_else(|| String::from("HTTP answer"))?
        })
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the shared deployment hands each negative walk its programs and drains them together"
    )]
    fn act_inner(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let [
            Proc::Service(service),
            Proc::Other(Extra::Native(HarnessProcess::Calls(caller))),
            Proc::Other(Extra::Proxy(proxy)),
            Proc::Other(Extra::Visitor(visitor)),
            ..,
        ] = procs
        else {
            return Err(String::from("public world composition"));
        };
        if self.stopping {
            return Ok(());
        }
        if super::previous::handles(&self.name) {
            if !self.previous.act(caller, proxy, visitor, &mut service.service)? {
                return Ok(());
            }
            self.stage = usize::MAX;
        } else {
            if !self.handed
                && let Some(link) = service.service.links().pop()
            {
                let mut sign_in = Command::new(&[b"sign-in", b"-"]);
                let link = String::from_utf8(link.into_vec()).map_err(|why| format!("printed link URL: {why:?}"))?;
                // The native world uses plaintext on the listener; the public name belongs to its proxy.
                sign_in.input =
                    link.replace("https://one.example", &format!("http://{}", fixtures::CLIENTS)).into_bytes().into();
                caller.commands.push_back(sign_in);
                service::command(caller, &["project", "amend", "1", "--visibility", "public", "--key", "open-one"]);
                self.handed = true;
            }
            if !self.setup {
                if caller.answers.len() != 2 || !proxy.ready() {
                    return Ok(());
                }
                for answer in &caller.answers {
                    match (&(answer.exit), &(skein_io::kernel::Exit::Code(0))) {
                        (left, right) if left != right => {
                            let finding = format!(
                                "public deployment setup: {} {}",
                                String::from_utf8_lossy(&answer.output),
                                String::from_utf8_lossy(&answer.error)
                            );
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                }
                self.send(visitor, fixtures::CLIENTS, Method::Get, jig_target("1").as_bytes());
                self.setup = true;
                return Ok(());
            }
            if self.calls.iter().any(|id| !visitor.answers.contains_key(id)) {
                return Ok(());
            }
            if self.project.is_none() {
                let answer = self.last(visitor)?;
                match (&(answer.status), &(200)) {
                    (left, right) if left != right => {
                        let finding = String::from("public projects after owner's ctl");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                policy(answer, false)?;
                let projects = public::PublicProjects::decode(&public::CEILINGS, &mut Reader::new(&answer.body))
                    .map_err(|why| format!("public projects record: {why:?}"))?;
                match (&(projects.projects.len()), &(1)) {
                    (left, right) if left != right => {
                        let finding = String::from("only the opened project");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.project =
                    Some(projects.projects.iter().next().ok_or_else(|| String::from("one project"))?.project);
            }
            if self.stage != usize::MAX {
                match self.name.as_str() {
                    super::owners::DECLARED | super::owners::REVOKED => {
                        if self.owners.act(
                            now,
                            caller,
                            proxy,
                            visitor,
                            self.project.ok_or_else(|| String::from("public project"))?,
                            &service.parameters.forge,
                        )? {
                            self.stage = usize::MAX;
                        }
                    }
                    "public_reads_past_the_limits_are_busy_while_signed_in_work_is_answered" => {
                        if self.load.act(
                            now,
                            &mut service.observer,
                            proxy,
                            visitor,
                            self.project.ok_or_else(|| String::from("project"))?,
                        )? {
                            self.stage = usize::MAX;
                        }
                    }
                    "abandoned_reads_leave_the_budgets_whole" => {
                        if self.abandoned.act(
                            now,
                            proxy,
                            visitor,
                            self.project.ok_or_else(|| String::from("project"))?,
                        )? {
                            self.stage = usize::MAX;
                        }
                    }
                    "a_private_or_deleted_project_is_unknown_at_once_and_through_the_proxy_within_the_max_age" => {
                        let quiescent = !service.service.work_pending(now);
                        if self.revocation.act(
                            now,
                            &mut service.observer,
                            proxy,
                            visitor,
                            self.project.ok_or_else(|| String::from("project"))?,
                            quiescent,
                        )? {
                            self.stage = usize::MAX;
                        }
                    }
                    other => return Err(format!("unknown service walk {other}")),
                }
            }
        }
        if self.stage == usize::MAX && proxy.idle() {
            caller.closed = true;
            proxy.stop();
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            self.stopping = true;
            for proc in procs {
                if let Proc::Peer(peer) = proc {
                    peer.shutdown();
                }
                if let Proc::Forge(peer) = proc {
                    peer.shutdown();
                }
            }
        }

        Ok(())
    }
}

impl Judge {
    fn act_checked(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        self.act_inner(now, procs)?;

        Ok(())
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        for proc in procs {
            if let Proc::Service(service) = proc
                && let Some(why) = &service.observer.inner.finding
            {
                return Err(why.clone());
            }
        }

        let [
            Proc::Service(service),
            Proc::Other(Extra::Native(HarnessProcess::Calls(caller))),
            Proc::Other(Extra::Proxy(proxy)),
            Proc::Other(Extra::Visitor(visitor)),
            ..,
        ] = procs
        else {
            return Err(String::from("composition"));
        };
        self.ready = !self.stopping
            && proxy.ready()
            && (self.setup || caller.answers.len() == 2 || !self.handed)
            && self.calls.iter().all(|id| visitor.answers.contains_key(id));
        if self.name == "public_reads_past_the_limits_are_busy_while_signed_in_work_is_answered" && self.setup {
            self.ready = self.load.ready(visitor, &service.observer);
        }
        if matches!(self.name.as_str(), super::owners::DECLARED | super::owners::REVOKED) && self.setup {
            self.ready &= self.owners.ready(caller, visitor);
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        self.owners
            .wake()
            .or_else(|| self.load.wake())
            .or_else(|| self.abandoned.wake())
            .or_else(|| self.revocation.wake())
            .or_else(|| self.ready.then_some(Time::ZERO))
            .or_else(|| (!self.stopping).then_some(Time::from_nanos(20_000_000_000)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= Time::from_nanos(20_000_000_000)).then(|| {
            format!("public {}, stage {}, seed {}", self.name, self.stage, self.seed)
                + &self.owners.diagnostic()
                + &self.load.diagnostic()
        })
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && (self.stopping)
    }
}
