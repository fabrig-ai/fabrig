//! Service answers observed through HTTP (temper/docs/design/protocol/sign-in.md, 2.5).
use super::{
    checks::{self, PROXY},
    http::Answer,
    proxy::Proxy,
    service::{self, Extra, Proc},
    visitor::Visitor,
};
use crate::{
    ctl::{Caller, Command},
    fixtures,
    world::HarnessProcess,
};
use jig_client_protocol_wire::{public_v1 as public, v22 as wire};
use skein_http::Method;
use skein_lib::{Reader, Time};
use std::net::SocketAddr;

const PROJECTS: &[u8] = b"/jig/v1/public/0/1/1";
const HOST: &[u8] = b"one.example";
const MALFORMED: &[&[u8]] = &[
    b"/jig/v1/public/65536/1/1",
    b"/jig/v1/public/0/65536/1",
    b"/jig/v1/public/0/1/65536",
    b"/jig/v1/public/00/1/1",
    b"/jig/v1/public/0/01/1",
    b"/jig/v1/public/0/1/01",
    b"/jig/v1/public/0/1/2/123456789abcdef",
    b"/jig/v1/public/0/1/2/123456789abcdef01",
    b"/jig/v1/public/0/1/2/123456789ABCDEFA",
    b"/jig/v1/public/0/1/2/%2F123456789abcde",
    b"/jig/v1/public/0/1/2/%61123456789abcde",
    b"/jig/v1/public/0//1/1",
    b"/jig/v1/public/0/./1/1",
    b"/jig/v1/public/0/1/1?query",
    b"/jig/v1/public/0/1/1/",
];

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
    wake: Option<Time>,
    stopping: bool,
    ready: bool,
    project: Option<u64>,
    baseline: Option<Answer>,
    owners: super::owners::Walk,
    load: super::load::Walk,
    abandoned: super::abandoned::Walk,
    revocation: super::revocation::Walk,
    identifiers: super::identifiers::Walk,
    prepared: bool,
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
            wake: None,
            stopping: false,
            ready: false,
            project: None,
            baseline: None,
            owners: super::owners::Walk::new(store.clone()),
            load: super::load::Walk::new(),
            abandoned: super::abandoned::Walk::new(),
            revocation: super::revocation::Walk::new(),
            identifiers: super::identifiers::Walk::new(store),
            prepared: false,
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
        if self.stopping || self.wake.is_some_and(|wake| now < wake) {
            return Ok(());
        }
        self.wake = None;
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
                self.send(visitor, fixtures::CLIENTS, Method::Get, PROJECTS);
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
                match (&(projects.projects().len()), &(1)) {
                    (left, right) if left != right => {
                        let finding = String::from("only the opened project");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.project =
                    Some(projects.projects().iter().next().ok_or_else(|| String::from("one project"))?.project());
            }
            if self.stage != usize::MAX {
                match self.name.as_str() {
                    "every_owner_version_and_kind_answers_as_declared" => {
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
                    "every_inaccessible_identifier_is_one_unknown_byte_for_byte" => {
                        if !self.prepared {
                            self.prepared = self.owners.act(
                                now,
                                caller,
                                proxy,
                                visitor,
                                self.project.ok_or_else(|| String::from("project"))?,
                                &service.parameters.forge,
                            )?;
                        } else if self.identifiers.act(
                            now,
                            &mut service.observer,
                            proxy,
                            visitor,
                            self.project.ok_or_else(|| String::from("project"))?,
                        )? {
                            self.stage = usize::MAX;
                        }
                    }
                    "a_members_set_visibility_is_refused" => self.member(caller, visitor)?,
                    "malformed_paths_are_404_no_store_and_never_kept" => self.malformed(proxy, visitor)?,
                    "no_credential_anywhere_else_is_signed_out" => self.signed_out(visitor)?,
                    "a_cookie_or_bearer_on_a_public_read_changes_no_byte" => self.credentials(now, proxy, visitor)?,
                    "no_answer_carries_an_access_control_header" => {
                        if !self.prepared {
                            self.cors(proxy, visitor)?;
                        } else if self.load.act(
                            now,
                            &mut service.observer,
                            proxy,
                            visitor,
                            self.project.ok_or_else(|| String::from("project"))?,
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
    fn member(&mut self, caller: &mut Caller, visitor: &mut Visitor) -> Result<(), String> {
        use jig_client::contract as c;
        match self.stage {
            0 => service::command(
                caller,
                &["project", "amend", "1", "--visibility", "private", "--key", "close-for-member"],
            ),
            1 => {
                if caller.answers.len() != 3 {
                    return Ok(());
                }
                match (&(caller.answers[2].exit), &(skein_io::kernel::Exit::Code(0))) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let mut holdings = skein_lib::List::with_capacity(1);
                holdings
                    .push(c::Holding { party: 1, role: c::HeldRole::Member })
                    .map_err(|why| format!("one member: {why:?}"))?;
                self.calls.push(super::requests::ask(visitor, &c::Ask::SetRoles { project: 1, holdings }, 71));
            }
            2 => {
                let answer = super::requests::answer(&self.last(visitor)?.body)?;
                if !(matches!(
                    answer,
                    c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::RolesSet { project: 1 }))
                )) {
                    return Err(format!("owner changes its own role: {answer:?}"));
                }
                service::command(
                    caller,
                    &["project", "amend", "1", "--visibility", "public", "--key", "member-cannot-open"],
                );
            }
            3 => {
                if caller.answers.len() != 4 {
                    return Ok(());
                }
                match (&(caller.answers[3].exit), &(skein_io::kernel::Exit::Code(2))) {
                    (left, right) if left != right => {
                        let finding = format!(
                            "member cannot widen visibility: {}",
                            String::from_utf8_lossy(&caller.answers[3].error)
                        );
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.send(
                    visitor,
                    fixtures::CLIENTS,
                    Method::Get,
                    format!("/jig/v1/public/0/1/2/{:016x}", self.project.ok_or_else(|| String::from("known project"))?)
                        .as_bytes(),
                );
            }
            4 => {
                match (&(self.last(visitor)?.status), &(404)) {
                    (left, right) if left != right => {
                        let finding = String::from("the member's project remains private");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.send(visitor, fixtures::CLIENTS, Method::Get, PROJECTS);
            }
            5 => {
                let projects =
                    public::PublicProjects::decode(&public::CEILINGS, &mut Reader::new(&self.last(visitor)?.body))
                        .map_err(|why| format!("public projects: {why:?}"))?;
                if !(projects.projects().is_empty()) {
                    return Err(format!("assertion failed: {}", stringify!(projects.projects().is_empty())));
                }
                self.stage = usize::MAX;
                return Ok(());
            }
            other => return Err(format!("member stage {other}")),
        }
        self.stage += 1;

        Ok(())
    }
    fn malformed(&mut self, proxy: &Proxy, visitor: &mut Visitor) -> Result<(), String> {
        let row = self.stage / 3;
        if row == MALFORMED.len() {
            self.stage = usize::MAX;
            return Ok(());
        }
        let path = MALFORMED[row];
        match self.stage % 3 {
            0 | 1 => {
                if self.stage % 3 == 1 {
                    self.malformed_answer(visitor)?;
                }
                self.send(visitor, PROXY, Method::Get, path);
            }
            2 => {
                self.malformed_answer(visitor)?;
                match (
                    &(proxy.counts.upstream_public),
                    &(u32::try_from((row + 1) * 2).map_err(|why| format!("requests: {why:?}"))?),
                ) {
                    (left, right) if left != right => {
                        let finding = format!("malformed request reaches upstream twice: {path:?}");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(proxy.counts.served_from_cache), &(0)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
            }
            other => return Err(format!("malformed stage {other}")),
        }
        self.stage += 1;

        Ok(())
    }
    fn malformed_answer(&self, visitor: &Visitor) -> Result<(), String> {
        let answer = self.last(visitor)?;
        match (&(answer.status), &(404)) {
            (left, right) if left != right => {
                let finding = format!("malformed row {}", self.stage / 3);
                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
            }
            _ => {}
        }
        match (&(answer.header(b"cache-control")), &(Some(b"no-store".as_slice()))) {
            (left, right) if left != right => {
                let finding = String::from("assertion `left == right` failed");
                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
            }
            _ => {}
        }
        if !(answer.body.is_empty()) {
            return Err(format!("assertion failed: {}", stringify!(answer.body.is_empty())));
        }
        policy(answer, false)?;

        Ok(())
    }
    fn signed_out(&mut self, visitor: &mut Visitor) -> Result<(), String> {
        const ROUTES: &[&[u8]] = &[b"request", b"read", b"watch", b"unwatch", b"stream-gone", b"stream", b"sign-out"];
        let row = self.stage / 2;
        if row == ROUTES.len() * 4 {
            self.stage = usize::MAX;
            return Ok(());
        }
        let route = ROUTES[row % ROUTES.len()];
        if self.stage.is_multiple_of(2) {
            let body = anonymous_body(route);
            let address = if row / ROUTES.len() < 2 { fixtures::CLIENTS } else { PROXY };
            let mut headers: Vec<(&[u8], &[u8])> =
                vec![(b"Host", HOST), (b"Content-Type", b"application/octet-stream")];
            if row / ROUTES.len() % 2 == 1 {
                headers.push((b"Cookie", b"jig-sign-in=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
                headers.push((b"Jig-Client", b"browser"));
            }
            let path = [b"/jig/v1/".as_slice(), route].concat();
            self.calls.push(visitor.send_body(
                address,
                if route == b"stream" { Method::Get } else { Method::Post },
                &path,
                &headers,
                &body,
            ));
        } else {
            let answer = self.last(visitor)?;
            checks::no_cors(answer)?;
            if route == b"stream" {
                match (&(answer.status), &(401)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
            } else {
                match (&(answer.status), &(200)) {
                    (left, right) if left != right => {
                        let finding = format!("signed-out route {route:?}: {answer:?}");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                assert_signed_out(route, &answer.body)?;
            }
        }
        self.stage += 1;

        Ok(())
    }
    #[expect(clippy::too_many_lines, reason = "the ordered referee checks return their findings at each step")]
    fn credentials(&mut self, now: Time, proxy: &Proxy, visitor: &mut Visitor) -> Result<(), String> {
        if self.stage == 0 {
            let authorization = [b"Bearer ".as_slice(), service::bearer().as_ref()].concat();
            self.calls.push(visitor.send_body(
                fixtures::CLIENTS,
                Method::Post,
                b"/jig/v1/read",
                &[(b"Host", HOST), (b"Content-Type", b"application/octet-stream"), (b"Authorization", &authorization)],
                &super::requests::read_body(&wire::Read::Me),
            ));
        } else if self.stage == 1 {
            let answer = self.last(visitor)?;
            match (&(answer.status), &(200)) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            if !(matches!(
                wire::ReadAnswer::decode(&wire::CEILINGS, &mut Reader::new(&answer.body))
                    .map_err(|why| format!("signed-in me: {why:?}"))?,
                wire::ReadAnswer::Value(_)
            )) {
                return Err(String::from("the credential belongs to a signed-in person"));
            }
            self.send(visitor, fixtures::CLIENTS, Method::Get, PROJECTS);
        } else if self.stage == 2 {
            self.baseline = Some(self.last(visitor)?.clone());
        } else {
            let row = (self.stage - 3) / 2;
            if row == 6 {
                self.stage = usize::MAX;
                return Ok(());
            }
            if (self.stage - 3).is_multiple_of(2) {
                let token = service::bearer();
                let cookie = [b"jig-sign-in=".as_slice(), &token].concat();
                let bearer = [b"Bearer ".as_slice(), &token].concat();
                let mut headers: Vec<(&[u8], &[u8])> = vec![
                    (b"Host", HOST),
                    (b"Forwarded", b"for=untrusted"),
                    (b"X-Forwarded-For", b"untrusted"),
                    (b"X-Forwarded-Host", b"untrusted"),
                ];
                if row % 3 != 1 {
                    headers.push((b"Cookie", &cookie));
                }
                if !row.is_multiple_of(3) {
                    headers.push((b"Authorization", &bearer));
                }
                if row >= 3 {
                    self.send(visitor, fixtures::CLIENTS, Method::Get, PROJECTS);
                }
                self.calls.push(visitor.send(
                    if row < 3 { fixtures::CLIENTS } else { PROXY },
                    Method::Get,
                    PROJECTS,
                    &headers,
                ));
            } else {
                let bare = if row < 3 {
                    self.baseline.as_ref().ok_or_else(|| String::from("bare read"))?
                } else {
                    visitor.answers[&self.calls[self.calls.len() - 2]]
                        .as_ref()
                        .ok_or_else(|| String::from("simultaneous bare read"))?
                };
                match (&(self.last(visitor)?), &(bare)) {
                    (left, right) if left != right => {
                        let finding = format!("credentials change no byte, row {row}");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                if row >= 3 {
                    let bytes = proxy.forwarded.last().ok_or_else(|| String::from("forwarded public read"))?;
                    let (call, _) = super::http::request(bytes).ok_or_else(|| String::from("upstream request"))?;
                    for h in &call.headers {
                        if h.is(b"cookie")
                            && !h.is(b"authorization")
                            && !h.is(b"forwarded")
                            && !h.name.to_ascii_lowercase().starts_with(b"x-forwarded-")
                        {
                            return Err(String::from("proxy removed visitor credentials and address claims"));
                        }
                    }
                    match (
                        &(proxy.counts.upstream_public),
                        &(u32::try_from(row - 2).map_err(|why| format!("public calls: {why:?}"))?),
                    ) {
                        (left, right) if left != right => {
                            let finding = String::from("assertion `left == right` failed");
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    self.wake = Some(Time::from_nanos(now.as_nanos() + 2_000_000_000));
                }
            }
        }
        self.stage += 1;

        Ok(())
    }
    #[expect(
        clippy::type_complexity,
        reason = "each table row carries its destination, method, path, Host and expected status"
    )]
    fn cors(&mut self, proxy: &Proxy, visitor: &mut Visitor) -> Result<(), String> {
        let row = self.stage / 2;
        let cases: &[(SocketAddr, Method, &[u8], &[u8], u16)] = &[
            (fixtures::CLIENTS, Method::Get, PROJECTS, HOST, 200),
            (PROXY, Method::Get, PROJECTS, HOST, 200),
            (fixtures::CLIENTS, Method::Get, b"/jig/v1/public/0/1/65535", HOST, 404),
            (PROXY, Method::Get, b"/jig/v1/public/0/1/65535", HOST, 404),
            (fixtures::CLIENTS, Method::Get, PROJECTS, b"refused.example", 421),
            (PROXY, Method::Get, PROJECTS, b"refused.example", 421),
            (fixtures::CLIENTS, Method::Options, PROJECTS, HOST, 404),
            (PROXY, Method::Options, PROJECTS, HOST, 404),
            (fixtures::CLIENTS, Method::Post, b"/jig/v1/read", HOST, 415),
            (PROXY, Method::Post, b"/jig/v1/read", HOST, 415),
        ];
        if row == cases.len() {
            if proxy.counts.refused_host == 0 {
                return Err(format!("assertion failed: {}", stringify!(proxy.counts.refused_host > 0)));
            }
            self.prepared = true;
            return Ok(());
        }
        let (address, method, path, host, status) = cases[row];
        if self.stage.is_multiple_of(2) {
            self.calls.push(visitor.send(
                address,
                method,
                path,
                &[(b"Host", host), (b"Origin", b"https://another.example"), (b"Access-Control-Request-Method", b"GET")],
            ));
        } else {
            match (&(self.last(visitor)?.status), &(status)) {
                (left, right) if left != right => {
                    let finding = format!("CORS case {row}");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            policy(self.last(visitor)?, address == PROXY)?;
            if status == 200 {
                cacheable(self.last(visitor)?)?;
            }
        }
        self.stage += 1;

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
        if (self.name == "public_reads_past_the_limits_are_busy_while_signed_in_work_is_answered"
            || (self.name == "no_answer_carries_an_access_control_header" && self.prepared))
            && self.setup
        {
            self.ready = self.load.ready(visitor, &service.observer);
        }
        if matches!(
            self.name.as_str(),
            "every_owner_version_and_kind_answers_as_declared"
                | "every_inaccessible_identifier_is_one_unknown_byte_for_byte"
        ) && self.setup
            && !self.prepared
        {
            self.ready &= self.owners.ready(caller, visitor);
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake
            .or_else(|| self.owners.wake())
            .or_else(|| self.load.wake())
            .or_else(|| self.abandoned.wake())
            .or_else(|| self.revocation.wake())
            .or_else(|| self.identifiers.wake())
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
fn owned() -> wire::Owned {
    wire::Owned::new(&wire::CEILINGS, wire::OwnedParts { owner: wire::Owner::Jig, version: 22, record: Box::default() })
        .expect("owned read")
}
fn anonymous_body(route: &[u8]) -> Box<[u8]> {
    match route {
        b"request" => {
            let record = wire::Keyed::new(
                &wire::CEILINGS,
                wire::KeyedParts {
                    key: [7; 16],
                    owner: wire::Owner::Jig,
                    version: 22,
                    request: Box::default(),
                    value: None,
                },
            )
            .expect("keyed request");
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode keyed");
            writer.finish()
        }
        b"read" => {
            let record = owned();
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode read");
            writer.finish()
        }
        b"watch" => {
            let record =
                wire::Open::new(&wire::CEILINGS, wire::OpenParts { stream: [7; 16], watch: 1, subject: owned() })
                    .expect("watch");
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode watch");
            writer.finish()
        }
        b"unwatch" => {
            let record = wire::WatchClose::new(&wire::CEILINGS, wire::WatchCloseParts { stream: [7; 16], watch: 1 })
                .expect("watch close");
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode close");
            writer.finish()
        }
        b"stream-gone" => {
            let record = wire::Gone::new(&wire::CEILINGS, wire::GoneParts { stream: [7; 16] }).expect("gone");
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode gone");
            writer.finish()
        }
        b"sign-out" | b"stream" => Box::default(),
        _ => unreachable!("route table"),
    }
}
fn assert_signed_out(route: &[u8], bytes: &[u8]) -> Result<(), String> {
    let mut reader = Reader::new(bytes);
    match route {
        b"request" => match (
            &(wire::KeyedAnswer::decode(&wire::CEILINGS, &mut reader)
                .map_err(|why| format!("keyed answer: {why:?}"))?),
            &(wire::KeyedAnswer::SignedOut),
        ) {
            (left, right) if left != right => {
                let finding = String::from("assertion `left == right` failed");
                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
            }
            _ => {}
        },
        b"read" => {
            if !(matches!(wire::ReadAnswer::decode(&wire::CEILINGS, &mut reader).map_err(|why| format!("read answer: {why:?}"))?, wire::ReadAnswer::None(row) if row.value() == &wire::ReadNone::SignedOut))
            {
                return Err(format!(
                    "assertion failed: {}",
                    stringify!(
                        matches!(wire::ReadAnswer::decode(&wire::CEILINGS, &mut reader).ok_or_else(|| String::from("read answer"))?, wire::ReadAnswer::None(row) if row.value() == &wire::ReadNone::SignedOut)
                    )
                ));
            }
        }
        b"watch" => {
            if !(matches!(wire::OpenAnswer::decode(&wire::CEILINGS, &mut reader).map_err(|why| format!("watch answer: {why:?}"))?, wire::OpenAnswer::Refused(row) if row.why() == &wire::WatchRefusal::SignedOut))
            {
                return Err(format!(
                    "assertion failed: {}",
                    stringify!(
                        matches!(wire::OpenAnswer::decode(&wire::CEILINGS, &mut reader).ok_or_else(|| String::from("watch answer"))?, wire::OpenAnswer::Refused(row) if row.why() == &wire::WatchRefusal::SignedOut)
                    )
                ));
            }
        }
        b"stream-gone" => {
            if !(bytes.is_empty()) {
                return Err(String::from("stream-gone acknowledges without a record"));
            }
        }
        b"unwatch" => match (
            &(wire::CloseAnswer::decode(&wire::CEILINGS, &mut reader)
                .map_err(|why| format!("close answer: {why:?}"))?),
            &(wire::CloseAnswer::SignedOut),
        ) {
            (left, right) if left != right => {
                let finding = String::from("assertion `left == right` failed");
                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
            }
            _ => {}
        },
        b"sign-out" => match (
            &(wire::SignOutAnswer::decode(&wire::CEILINGS, &mut reader)
                .map_err(|why| format!("sign-out answer: {why:?}"))?),
            &(wire::SignOutAnswer::SignedOut),
        ) {
            (left, right) if left != right => {
                let finding = String::from("assertion `left == right` failed");
                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
            }
            _ => {}
        },
        _ => return Err(String::from("route table")),
    }

    Ok(())
}
