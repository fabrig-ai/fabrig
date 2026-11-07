//! One bounded HTTP connection over GitHub's API/browser or blob listener (skein/docs/design/fake-forge.md, 10.1, 10.6 and 12).
use crate::{
    answers, pages,
    service::{Limits, Outcome, Pending, Problem, Request, Response, Service},
};
use alloc::boxed::Box;
use skein_fake_forge_domain as domain;
use skein_fake_forge_http::{
    Hosted,
    connection::{self, State, Turn},
    git,
};
use skein_forge::github::request::Method;
use skein_http::{Header, Method as HttpMethod, server as http};
use skein_lib::bytes::concat;
use skein_lib::{
    Env, Intake, List, Queue, ReplyTo, Token, bytes,
    stream::{Down, Read, Up},
    usize_of,
};

#[derive(Debug)]
pub enum Event {
    Domain(domain::Event),
    Rate { key: domain::RateKey, limit: u32, window: skein_lib::Duration },
    Close,
    Closed,
}
#[expect(missing_debug_implementations, reason = "HTTP heads and body intake can contain credentials")]
pub struct Server {
    owner: Token,
    blob: bool,
    first_byte: bool,
    issue_write: bool,
    connection: connection::Connection,
    call: Option<http::Call>,
    body: Intake,
    pending: Option<Pending>,
    admission: Option<crate::rates::Admission>,
}
impl Server {
    pub fn new(owner: Token, limits: &Limits) -> Result<Self, Problem> {
        if !valid(limits) {
            return Err(Problem::Limits);
        }
        Ok(Self {
            owner,
            blob: false,
            first_byte: false,
            issue_write: false,
            connection: connection::Connection::new(&limits.http),
            call: None,
            body: Intake::with_capacity(limits.body),
            pending: None,
            admission: None,
        })
    }
    /// A connection on the peer's credential-free blob listener.
    pub fn blob(owner: Token, limits: &Limits) -> Result<Self, Problem> {
        let mut server = Self::new(owner, limits)?;
        server.blob = true;
        Ok(server)
    }
    /// The request in hand's Authorization value, until it is dispatched. A world's tally reads it beside
    /// `take_head`, which blanks the head's own.
    #[must_use]
    pub fn credential(&self) -> Option<&[u8]> {
        self.call.as_ref()?.header(b"authorization")
    }
    pub(crate) fn head_pending(&self) -> bool {
        self.connection.head_pending()
    }
    /// Takes the head with every Authorization value removed.
    pub fn take_head(&mut self) -> Option<http::Call> {
        self.connection.take_head()
    }
    #[must_use]
    pub fn is_settled(&self) -> bool {
        self.connection.is_settled()
    }
    #[must_use]
    pub fn has_work(&self) -> bool {
        self.connection.has_work()
    }
}
/// Whether these limits are representable: every edge bound they size is finite
/// and every capacity they derive fits (programming-model.md, 6.3).
/// HTTP is usable, bodies and responses are nonempty, and git decoding fits.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.body > 0
        && limits.response >= 128
        && http::valid(&limits.http)
        && limits.github.inner.checked_mul(4).is_some()
        && git::decoder_bound(&crate::git::limits(limits)).is_some()
        && skein_json::tokenizer::tokenizer_bound(&crate::git::limits(limits).documents.json).is_some()
}

pub fn start(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    if server.connection.state != State::New {
        return;
    }
    next(server, service, env, above, below);
    resume(server, service, env, above, below);
}
pub fn up(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    event: Up,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    up_inner(server, service, env, event, None, above, below);
}
/// Receives bytes while borrowing the domain for private git store calls.
pub fn up_with_domain(
    server: &mut Server,
    service: &mut Service,
    hosted: Hosted<'_>,
    env: &Env<Limits>,
    event: Up,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    up_inner(server, service, env, event, Some(hosted), above, below);
}
fn up_inner(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    event: Up,
    hosted: Option<Hosted<'_>>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    if !connection::up(&mut server.connection, &env.with(env.limits.http), event, below) {
        return;
    }
    resume_inner(server, service, env, hosted, above, below);
}
/// The host routes only this connection's reply and lends the shared domain for follow-up reads.
pub fn down(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    reply: domain::Request,
    domain: &domain::Domain,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    let (to, result) = match reply {
        domain::Request::Reply { to, result } => (to, result),
        domain::Request::Hook { .. } => return,
    };
    if to.into_token() != server.owner || server.connection.state != State::Waiting {
        return;
    }
    let Some(pending) = server.pending.take() else { return };
    let (outcome, git_pending) = match pending.kind {
        crate::service::PendingKind::Git(pending) => {
            let response = crate::git::render(pending.name, result, &env.limits);
            (Outcome::Reply(response), Some(pending))
        }
        kind @ (crate::service::PendingKind::Mutation(_)
        | crate::service::PendingKind::CommentEdit(_)
        | crate::service::PendingKind::Read(_)
        | crate::service::PendingKind::Enqueue(_)
        | crate::service::PendingKind::Conditional { .. }
        | crate::service::PendingKind::Write(_)
        | crate::service::PendingKind::Blob
        | crate::service::PendingKind::EditReach { .. }) => {
            (service.finish(Pending { kind, rate: pending.rate }, result, ReplyTo::new(server.owner), domain), None)
        }
    };
    // A request's reads and write continuations finish before its answer is lost.
    // Drawing here keeps a merge's preliminary reads from losing its write.
    let outcome = match outcome {
        call @ (Outcome::Call(_, _) | Outcome::Admit(_)) => call,
        Outcome::Reply(reply) => {
            if service.rng.chance(service.faults.drop) {
                if let Some(pending) = &git_pending {
                    service.record_git(pending, reply.status);
                }
                close(server, service, env, above, below);
                return;
            }
            Outcome::Reply(if service.rng.chance(service.faults.server_error) {
                let mut failed = answers::failed();
                failed.rate = reply.rate;
                failed
            } else {
                reply
            })
        }
    };
    if let Some(pending) = git_pending {
        match &outcome {
            Outcome::Reply(response) => service.record_git(&pending, response.status),
            Outcome::Call(_, _) | Outcome::Admit(_) => unreachable!("git finishes with one reply"),
        }
    }
    answer(server, service, &env.limits, outcome, above);
    resume(server, service, env, above, below);
}
/// Returns the one domain-window admission for this connection.
pub fn admitted(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    result: Result<domain::Rate, domain::RateDenied>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    if server.connection.state != State::Waiting {
        return;
    }
    let Some(admission) = server.admission.take() else { return };
    let outcome = service.admitted(admission, result, ReplyTo::new(server.owner));
    answer(server, service, &env.limits, outcome, above);
    resume(server, service, env, above, below);
}
pub fn resume(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    resume_inner(server, service, env, None, above, below);
}
/// Resumes a connection while borrowing the domain for private git store calls.
pub fn resume_with_domain(
    server: &mut Server,
    service: &mut Service,
    hosted: Hosted<'_>,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    resume_inner(server, service, env, Some(hosted), above, below);
}
fn resume_inner(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    mut hosted: Option<Hosted<'_>>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    for _ in 0..8u32 {
        if let Some(turn) = connection::turn(&mut server.connection, &env.limits.http) {
            event_up(server, service, env, turn, hosted.as_mut(), above, below);
            if server.head_pending() {
                break;
            }
        } else if !connection::request(&mut server.connection, &env.with(env.limits.http), below) {
            break;
        }
    }
}
pub fn close(
    server: &mut Server,
    _service: &mut Service,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    if server.connection.state == State::Closing || server.connection.state == State::Closed {
        return;
    }
    server.pending = None;
    server.admission = None;
    server.call = None;
    server.body.clear();
    connection::close(&mut server.connection, &env.with(env.limits.http), below);
    above.push(Event::Close);
}
pub fn closed(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    if server.connection.state == State::Closed {
        return;
    }
    close(server, service, env, above, below);
    server.connection.state = State::Closed;
    above.push(Event::Closed);
}
fn next(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    if service.rng.chance(service.faults.reset) {
        close(server, service, env, above, below);
    } else {
        connection::read_next(&mut server.connection);
    }
}
fn demand(server: &mut Server, limits: &Limits) {
    server
        .connection
        .requests
        .push(http::Request::Body(Down::Demand { read: Read::Arrived { max: limits.http.read }, room: 0 }));
}
fn event_up(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    turn: Turn,
    hosted: Option<&mut Hosted<'_>>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    match turn {
        Turn::Call(call) => {
            if !server.blob
                && let Some(Ok(name)) = git::name(&call.target)
                && name.repository()
                && let Err(status) = service.git_access(call.header(b"authorization"), env.wall)
            {
                let pending = crate::git::Pending { name, minted: service.git_identity(call.header(b"authorization")) };
                service.record_git(&pending, status);
                server.connection.requests.push(http::Request::Discard);
                respond(server, &env.limits, crate::logs::empty(status), None);
                return;
            }
            server.call = Some(call);
            server.body.clear();
            server.connection.state = State::Body;
            demand(server, &env.limits);
        }
        Turn::Body(Up::Bytes(chunk)) => {
            if server.body.append(&chunk).is_err() {
                if let Some(call) = &server.call
                    && let Some(Ok(name)) = git::name(&call.target)
                {
                    let pending =
                        crate::git::Pending { name, minted: service.git_identity(call.header(b"authorization")) };
                    service.record_git(&pending, 413);
                }
                server.connection.requests.push(http::Request::Discard);
                respond(server, &env.limits, answers::refusal(413, crate::texts::PAYLOAD_TOO_LARGE, &env.limits), None);
            } else {
                demand(server, &env.limits);
            }
        }
        Turn::Body(Up::End) => dispatch(server, service, env, hosted, above),
        Turn::Body(Up::Room) | Turn::Handled => {}
        Turn::Body(Up::Failed(_)) | Turn::Failed => close(server, service, env, above, below),
        Turn::Done(reuse) => {
            server.call = None;
            server.body.clear();
            match reuse {
                http::Reuse::Keep => next(server, service, env, above, below),
                http::Reuse::Close => close(server, service, env, above, below),
            }
        }
    }
}
fn dispatch(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    hosted: Option<&mut Hosted<'_>>,
    above: &mut Queue<Event>,
) {
    let call = server.call.take().expect("accepted head");
    server.issue_write = call.method == HttpMethod::Post && call.target.ends_with(b"/issues");
    if server.blob {
        server.first_byte = call.header(b"range") == Some(b"bytes=0-0");
        let method = if call.method == HttpMethod::Get { Method::Get } else { Method::Post };
        let outcome = service.blob(
            method,
            &call.target,
            call.header(b"authorization").is_some(),
            call.header(b"range").is_some(),
            env.wall,
            ReplyTo::new(server.owner),
        );
        answer(server, service, &env.limits, outcome, above);
        return;
    }
    if let Some(name) = git::name(&call.target) {
        dispatch_git(server, service, env, hosted, &call, name, above);
        return;
    }
    let method = match call.method {
        HttpMethod::Get => Method::Get,
        HttpMethod::Post => Method::Post,
        HttpMethod::Put => Method::Put,
        HttpMethod::Patch => Method::Patch,
        HttpMethod::Delete => Method::Delete,
        HttpMethod::Head => {
            connection::respond(
                &mut server.connection,
                &env.limits.http,
                connection::Reply { status: 404, headers: Box::new([]), body: Box::new([]), bodyless: true },
            );
            return;
        }
        HttpMethod::Options => {
            respond(server, &env.limits, answers::missing(&env.limits), None);
            return;
        }
    };
    let body = if server.body.is_empty() {
        Box::new([])
    } else {
        server.body.meet(Read::Fill(server.body.len())).expect("whole body")
    };
    if pages::accepts(method, &call.target) {
        let page = service.page(&pages::Page { method, target: &call.target, body: &body }, env.wall);
        let content_type = match page.media {
            pages::Media::Html => crate::service::ContentType::Html,
            pages::Media::None => crate::service::ContentType::Json,
        };
        respond(
            server,
            &env.limits,
            Response {
                rate: None,
                etag: None,
                status: page.status,
                link: None,
                location: None,
                body: page.body,
                content_type,
            },
            page.location,
        );
    } else {
        let outcome = service.request(
            &Request {
                validator: call.header(b"if-none-match"),
                method,
                target: &call.target,
                body: &body,
                authorization: call.header(b"authorization"),
                version: call.header(b"x-github-api-version"),
                agent: call.header(b"user-agent").is_some(),
            },
            env.wall,
            ReplyTo::new(server.owner),
        );
        answer(server, service, &env.limits, outcome, above);
    }
}
fn dispatch_git(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    hosted: Option<&mut Hosted<'_>>,
    call: &http::Call,
    name: Result<git::Name, git::Unrouted>,
    above: &mut Queue<Event>,
) {
    let body = if server.body.is_empty() {
        Box::new([])
    } else {
        server.body.meet(Read::Fill(server.body.len())).expect("whole body")
    };
    let method = if call.method == HttpMethod::Post { Method::Post } else { Method::Get };
    let outcome = service.git_request(
        &Request {
            validator: None,
            method,
            target: &call.target,
            body: &body,
            authorization: call.header(b"authorization"),
            version: None,
            agent: false,
        },
        name,
        env.wall,
        ReplyTo::new(server.owner),
        hosted,
    );
    answer(server, service, &env.limits, outcome, above);
}
fn answer(server: &mut Server, service: &Service, limits: &Limits, outcome: Outcome, above: &mut Queue<Event>) {
    match outcome {
        Outcome::Admit(admission) => {
            let key = admission.key();
            server.admission = Some(admission);
            server.connection.state = State::Waiting;
            above.push(Event::Rate { key, limit: service.rate_quota(), window: service.rate_window() });
        }
        Outcome::Reply(reply) => respond(server, limits, reply, None),
        Outcome::Call(pending, event) => {
            server.pending = Some(pending);
            server.connection.state = State::Waiting;
            above.push(Event::Domain(event));
        }
    }
}
// http.md, 5.4: these statuses have no reply stream.
fn status_has_no_body(status: u16) -> bool {
    status == 204 || status == 304
}
fn respond(server: &mut Server, limits: &Limits, mut reply: Response, mut location: Option<Box<[u8]>>) {
    if location.is_none() {
        location = reply.location.take();
    }
    let location_bytes = match &location {
        Some(value) => value.len(),
        None => 0,
    };
    if reply.body.len() > usize_of(limits.response) || location_bytes > usize_of(limits.response) {
        reply = answers::failed();
        location = None;
    }
    let mut headers = List::with_capacity(11);
    if server.blob && server.first_byte && reply.status == 200 && !reply.body.is_empty() {
        let size = skein_lib::Decimal::of(skein_lib::len_u64(reply.body.len()));
        let value = concat(&[b"bytes 0-0/", size.as_bytes()]);
        headers.push(Header { name: bytes::copy_of(b"Content-Range"), value }).expect("range slot");
        reply.status = 206;
        reply.body = bytes::copy_of(reply.body.get(..1).expect("nonempty blob"));
    }
    if !server.blob && reply.status == 201 && server.issue_write {
        headers
            .push(Header {
                name: bytes::copy_of(b"X-Accepted-GitHub-Permissions"),
                value: bytes::copy_of(b"issues=write"),
            })
            .expect("permissions slot");
    }
    if let Some(rate) = reply.rate {
        crate::rates::headers(rate, reply.status == 403, &mut headers);
    }
    if let Some(etag) = reply.etag {
        headers.push(Header { name: bytes::copy_of(b"ETag"), value: etag }).expect("etag slot");
    }
    if let Some(link) = reply.link {
        headers.push(Header { name: bytes::copy_of(b"Link"), value: link }).expect("link slot");
    }
    if let Some(location) = location {
        headers.push(Header { name: bytes::copy_of(b"Location"), value: location }).expect("location slot");
    }
    let bodyless = status_has_no_body(reply.status);
    let length = if bodyless { 0 } else { skein_lib::len_u64(reply.body.len()) };
    if length > 0 {
        headers
            .push(Header { name: bytes::copy_of(b"Content-Type"), value: bytes::copy_of(reply.content_type.bytes()) })
            .expect("media slot");
    }
    connection::respond(
        &mut server.connection,
        &limits.http,
        connection::Reply {
            status: reply.status,
            headers: headers.into_boxed(),
            body: if bodyless { Box::new([]) } else { reply.body },
            bodyless,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures;

    // The body fixture has four bytes; the refusal fixture adds one.
    const BODY_BOUND: u32 = 4;
    fn intake(body: &[u8]) -> (u16, u32) {
        let limits = Limits { body: BODY_BOUND, ..fixtures::limits() };
        let mut server = Server::new(Token::new(1), &limits).expect("bounded HTTP server");
        let mut service = Service::new(fixtures::config(), &limits, 1).expect("bounded service");
        let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        let mut call = http::Call {
            method: HttpMethod::Post,
            version: skein_http::Version::Http11,
            body: http::Body::Length(u64::from(BODY_BOUND)),
            target: bytes::copy_of(b"/unknown"),
            headers: Box::new([]),
        };
        call.headers = Box::new([
            Header { name: bytes::copy_of(b"User-Agent"), value: bytes::copy_of(b"example-app") },
            Header {
                name: bytes::copy_of(b"X-GitHub-Api-Version"),
                value: bytes::copy_of(skein_forge::github::pin::API_VERSION.as_bytes()),
            },
        ]);
        server.connection.events.push(http::Event::Call(call));
        let turn = connection::turn(&mut server.connection, &limits.http).expect("request head");
        event_up(&mut server, &mut service, &env, turn, None, &mut above, &mut below);
        drop(server.connection.requests.pop());
        event_up(
            &mut server,
            &mut service,
            &env,
            Turn::Body(Up::Bytes(bytes::copy_of(body))),
            None,
            &mut above,
            &mut below,
        );
        if server.connection.state == State::Body {
            drop(server.connection.requests.pop());
            event_up(&mut server, &mut service, &env, Turn::Body(Up::End), None, &mut above, &mut below);
        } else {
            drop(server.connection.requests.pop());
        }
        let Some(http::Request::Respond(response)) = server.connection.requests.pop() else { panic!("HTTP refusal") };
        (response.status, server.body.len())
    }
#[test]
fn http_intake_enforces_its_request_body_byte_bound() {
    skein_rows::check(
        [
            ("github_http_intake_accepts_a_body_at_its_bound", (b"body".as_slice(), 404, 0..=0)),
            ("github_http_intake_refuses_one_byte_past_its_bound", (b"body!".as_slice(), 413, 0..=BODY_BOUND)),
        ],
        |(body, expected, retained)| {
            let (status, bytes) = intake(body);
            assert_eq!(status, expected);
            assert!(retained.contains(&bytes));
        },
    );
}

}

#[cfg(test)]
mod git_tests {
    use super::*;
    use crate::{GitCall, Minted, service::PendingKind, tests::fixtures};

    fn git_call(target: &[u8], body: http::Body) -> http::Call {
        http::Call {
            method: HttpMethod::Post,
            version: skein_http::Version::Http11,
            body,
            target: bytes::copy_of(target),
            headers: Box::new([]),
        }
    }
    #[test]
    fn a_git_record_uses_the_status_after_a_server_fault() {
        let mut h = fixtures::Harness::new();
        h.service.faults.server_error = 1000;
        let owner = Token::new(77);
        let mut server = Server::new(owner, &h.limits).expect("fixture HTTP server");
        server.connection.state = State::Waiting;
        let minted = Some(Minted { installation: 1, mint: 1 });
        server.pending =
            Some(Pending { rate: None, kind: PendingKind::Git(crate::git::Pending { name: git::Name::Push, minted }) });
        let env = Env { now: h.env.now, wall: h.now, limits: h.limits };
        let http_env = env.with(env.limits.http);
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        http::down(
            &mut server.connection.http,
            &http_env,
            http::Request::Next,
            &mut server.connection.events,
            &mut below,
        );
        let Some(Down::Demand { read: Read::Nothing, room }) = below.pop() else { panic!("response reservation") };
        assert!(room > 0);
        http::up(&mut server.connection.http, &http_env, Up::Room, &mut server.connection.events, &mut below);
        for line in [b"POST /.fake/git/push HTTP/1.1\r\n".as_slice(), b"Host: forge.test\r\n", b"\r\n"] {
            let Some(Down::Demand { .. }) = below.pop() else { panic!("request head demand") };
            http::up(
                &mut server.connection.http,
                &http_env,
                Up::Bytes(bytes::copy_of(line)),
                &mut server.connection.events,
                &mut below,
            );
        }
        let Some(Turn::Call(_)) = connection::turn(&mut server.connection, &h.limits.http) else {
            panic!("captured HTTP call")
        };
        drop(server.take_head());
        down(
            &mut server,
            &mut h.service,
            &Env { now: h.env.now, wall: h.now, limits: h.limits },
            domain::Request::Reply {
                to: ReplyTo::new(owner),
                result: Ok(domain::api::Answer::Pushed(domain::api::Pushed::Pushed)),
            },
            &h.domain,
            &mut above,
            &mut below,
        );
        let Some(Down::Send(head)) = below.pop() else { panic!("reply head bytes") };
        assert!(head.starts_with(b"HTTP/1.1 500 "));
        assert_eq!(h.service.git_calls().len(), 1);
        assert_eq!(h.service.git_calls().iter().next(), Some(&GitCall { minted, writes: true, status: 500 }));
    }

    #[test]
    fn a_private_git_credential_refusal_precedes_the_body_demand() {
        let mut h = fixtures::Harness::new();
        let mut server = Server::new(Token::new(77), &h.limits).expect("fixture HTTP server");
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        server
            .connection
            .events
            .push(http::Event::Call(git_call(b"/.fake/git/clone", http::Body::Length(u64::from(h.limits.body)))));
        let turn = connection::turn(&mut server.connection, &h.limits.http).expect("private git head");
        event_up(
            &mut server,
            &mut h.service,
            &Env { now: h.env.now, wall: h.now, limits: h.limits },
            turn,
            None,
            &mut above,
            &mut below,
        );
        let Some(http::Request::Discard) = server.connection.requests.pop() else { panic!("discard refused body") };
        let Some(http::Request::Respond(response)) = server.connection.requests.pop() else {
            panic!("credential refusal")
        };
        assert_eq!(response.status, 401);
        assert_eq!(h.service.git_calls().len(), 1);
        let Some(http::Request::Reply(Down::Finish)) = server.connection.requests.pop() else {
            panic!("empty response finishes")
        };
        assert!(server.connection.requests.is_empty(), "no body demand");
    }
}
