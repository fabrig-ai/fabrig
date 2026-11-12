//! One bounded HTTP connection over Forgejo documents and the fake's calls (skein/docs/design/testing/fakes/forge.md, 4, and skein/docs/design/testing/fakes/github.md, 3).
mod git;
use crate::{Error, Hosted, Limits, Service, errors, response, routes, service::Call};
use alloc::boxed::Box;
use skein_fake_forge_domain::{self as domain, api};
use skein_fake_forge_http::connection::{self, State, Turn};
use skein_forge::forgejo::{
    document,
    failure::Refusal,
    request::{self, Request},
    server as documents,
};
use skein_http::{Header, Method, server as http};
use skein_json::tokenizer::{self as json, Tokenizer};
use skein_lib::stream::{Down, Up};
use skein_lib::{Env, Id, List, Queue, ReplyTo, Token, bytes};
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    Domain(domain::Event),
    Close,
    Closed,
}
#[expect(clippy::struct_excessive_bools, reason = "independent Forgejo scope grants and the request write mode")]
struct Scopes {
    user: bool,
    repository: bool,
    admin: bool,
    organization: bool,
    writing: bool,
}
#[expect(missing_debug_implementations, reason = "HTTP queues temporarily hold authorization headers")]
pub struct Server {
    owner: Token,
    connection: connection::Connection,
    json: Tokenizer,
    json_started: bool,
    json_events: Queue<json::Event>,
    json_down: Queue<Down>,
    decoder: Option<documents::BodyDecoder>,
    git: Option<skein_fake_forge_http::git::Name>,
    git_decoder: Option<skein_fake_forge_http::git::server::BodyDecoder>,
    user: u64,
    scopes: Scopes,
    request: Option<Request>,
    requests: Queue<Request>,
    select: response::Select,
    range: Result<Option<response::Range>, api::Error>,
    if_none_match: Option<Box<[u8]>>,
    call: Option<Id<Call>>,
    followed: bool,
    listing: Option<crate::setup::repos::Listing>,
}
impl Server {
    pub fn new(owner: Token, limits: &Limits) -> Result<Self, Error> {
        if !http::valid(&limits.http)
            || json::tokenizer_bound(&limits.documents.json).is_none()
            || limits.calls == 0
            || limits.connections == 0
            || limits.raw_bytes == 0
            || json::largest_demand(&limits.documents.json) > limits.http.read
            || skein_forge::forgejo::limits::check(&limits.documents).is_err()
        {
            return Err(Error::Limits);
        }
        Ok(Self {
            owner,
            connection: connection::Connection::new(&limits.http),
            json: Tokenizer::new(&limits.documents.json),
            json_started: false,
            json_events: Queue::uncapped(),
            json_down: Queue::uncapped(),
            decoder: None,
            git: None,
            git_decoder: None,
            user: 0,
            scopes: Scopes { user: false, repository: false, admin: false, organization: false, writing: false },
            request: None,
            requests: Queue::uncapped(),
            select: response::Select::Document,
            range: Ok(None),
            if_none_match: None,
            call: None,
            followed: false,
            listing: None,
        })
    }
    /// Takes the sanitized head captured at the last request entrance.
    pub fn take_head(&mut self) -> Option<http::Call> {
        self.connection.take_head()
    }
    /// Takes decoded API requests in entrance order, including refused requests.
    pub fn take_request(&mut self) -> Option<Request> {
        self.requests.pop()
    }
    /// Whether no accepted request remains to be answered on this connection.
    #[must_use]
    pub fn is_settled(&self) -> bool {
        self.connection.is_settled()
    }
    #[must_use]
    pub fn has_work(&self) -> bool {
        self.connection.has_work() || !self.json_events.is_empty() || !self.json_down.is_empty()
    }
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
    up_on(server, service, None, env, event, above, below);
}
/// Serves bytes with setup collaborator grants applied to the lent domain before replying.
pub fn up_with_domain(
    server: &mut Server,
    service: &mut Service,
    domain: Hosted<'_>,
    env: &Env<Limits>,
    event: Up,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    up_on(server, service, Some(domain), env, event, above, below);
}
fn up_on(
    server: &mut Server,
    service: &mut Service,
    domain: Option<Hosted<'_>>,
    env: &Env<Limits>,
    event: Up,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    if !connection::up(&mut server.connection, &env.with(env.limits.http), event, below) {
        return;
    }
    resume_on(server, service, domain, env, above, below);
}
#[expect(clippy::too_many_lines, reason = "migration replies share the domain call's byte faults and lifecycle")]
pub fn down(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    reply: domain::Request,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    let (to, result) = match reply {
        domain::Request::Reply { to, result } => (to, result),
        domain::Request::Hook { .. } => return,
    };
    let id = Id::<Call>::from_token(to.into_token());
    if server.connection.state != State::Waiting || server.call != Some(id) {
        return;
    }
    let Some(call) = service.calls.get(id) else {
        return;
    };
    if !call.active || call.server != server.owner {
        return;
    }
    if server.listing.is_some() {
        match result {
            Ok(api::Answer::Permission(permission)) => {
                let accepted = match &mut server.listing {
                    Some(listing) => {
                        listing.accept(&service.setup, &service.config, server.user, permission.permission)
                    }
                    None => Err(Error::Limits),
                };
                if accepted.is_err() {
                    server.listing = None;
                    retire(server, service);
                    fail(server, &env.limits, 503);
                } else {
                    user_repos_next(server, service, env, above);
                }
            }
            Ok(answer) => {
                drop(answer);
                server.listing = None;
                retire(server, service);
                fail(server, &env.limits, 500);
            }
            Err(error) => {
                server.listing = None;
                retire(server, service);
                error_reply(server, env, error);
            }
        }
        resume(server, service, env, above, below);
        return;
    }
    if server.git.is_some() {
        git::down(server, service, env, result, above, below);
        resume(server, service, env, above, below);
        return;
    }
    let request = server.request.as_ref().expect("waiting request");
    let migration = service.setup.migration_reply(request, &result, &mut service.config, &env.limits, env.now);
    if !server.followed {
        if service.rng.chance(service.config.faults.drop) {
            close(server, service, env, above, below);
            return;
        }
        if service.rng.chance(service.config.faults.server_error) {
            retire(server, service);
            fail(server, &env.limits, 500);
            resume(server, service, env, above, below);
            return;
        }
        if migration.is_none()
            && let Ok(answer) = &result
        {
            match response::follow(request, answer) {
                Ok(Some((read, select))) => {
                    server.select = select;
                    server.followed = true;
                    above.push(Event::Domain(domain::Event::Call {
                        reply_to: ReplyTo::new(id.token()),
                        user: server.user,
                        repository: routes::repository(&service.config, request).expect("call repository"),
                        op: api::Op::Read(read),
                    }));
                    return;
                }
                Ok(None) => {}
                Err(error) => {
                    retire(server, service);
                    error_reply(server, env, error);
                    resume(server, service, env, above, below);
                    return;
                }
            }
        }
    }
    let reply = if let Some(reply) = migration {
        Ok(reply)
    } else {
        match result {
            Ok(answer) => {
                response::answer(&service.config, &service.setup, &env.limits, request, &server.select, &answer)
            }
            Err(error) => Err(error),
        }
    };
    retire(server, service);
    match reply {
        Ok(reply) => respond(server, &env.limits, reply),
        Err(error) => error_reply(server, env, error),
    }
    resume(server, service, env, above, below);
}
pub fn resume(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    resume_on(server, service, None, env, above, below);
}
/// Resumes a pass with the same setup grant crossing as `up_with_domain`.
pub fn resume_with_domain(
    server: &mut Server,
    service: &mut Service,
    domain: Hosted<'_>,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    resume_on(server, service, Some(domain), env, above, below);
}
fn resume_on(
    server: &mut Server,
    service: &mut Service,
    mut domain: Option<Hosted<'_>>,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    let http_env = env.with(env.limits.http);
    for _ in 0..8u32 {
        if let Some(turn) = connection::turn(&mut server.connection, &env.limits.http) {
            http_event(server, service, domain.as_mut(), env, turn, above, below);
            if server.connection.head_pending() {
                break;
            }
        } else if let Some(event) = server.json_events.pop() {
            if let Some(decoder) = &mut server.git_decoder {
                match skein_fake_forge_http::git::server::body(decoder, &env.limits.git(), &event) {
                    skein_fake_forge_http::git::server::BodyStep::Demand(request) => json_down(server, env, request),
                    skein_fake_forge_http::git::server::BodyStep::Decoded(call) => {
                        server.git_decoder = None;
                        git::decoded(server, service, domain.as_mut(), env, call, above, below);
                    }
                    skein_fake_forge_http::git::server::BodyStep::Malformed => {
                        server.git_decoder = None;
                        malformed(server, env);
                    }
                }
                continue;
            }
            let Some(decoder) = &mut server.decoder else {
                continue;
            };
            match documents::body(decoder, &env.limits.documents, &event) {
                documents::BodyStep::Demand(request) => json_down(server, env, request),
                documents::BodyStep::Decoded(request) => {
                    server.decoder = None;
                    decoded(server, service, domain.as_mut(), env, request, above);
                }
                documents::BodyStep::Malformed(_) => {
                    server.decoder = None;
                    malformed(server, env);
                }
            }
        } else if let Some(down) = server.json_down.pop() {
            connection::down(&mut server.connection, &http_env, http::Request::Body(down), below);
        } else if !connection::request(&mut server.connection, &http_env, below) {
            break;
        }
    }
}
pub fn close(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    if server.connection.state == State::Closing || server.connection.state == State::Closed {
        return;
    }
    retire(server, service);
    server.listing = None;
    server.request = None;
    server.decoder = None;
    server.git_decoder = None;
    server.json_events.clear();
    server.json_down.clear();
    json_down(server, env, json::Request::Close);
    server.json_events.clear();
    server.json_down.clear();
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
    if server.connection.state != State::Closing {
        close(server, service, env, above, below);
    }
    server.connection.state = State::Closed;
    above.push(Event::Closed);
}
fn retire(server: &mut Server, service: &mut Service) {
    if let Some(id) = server.call.take() {
        if let Some(call) = service.calls.get_mut(id) {
            call.active = false;
        }
        service.calls.retire(id);
    }
}
fn next(
    server: &mut Server,
    service: &mut Service,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    if service.rng.chance(service.config.faults.reset) {
        close(server, service, env, above, below);
    } else {
        connection::read_next(&mut server.connection);
    }
}
fn json_down(server: &mut Server, env: &Env<Limits>, request: json::Request) {
    let limits = git::json_limits(server, &env.limits);
    json::down(&mut server.json, &env.with(limits), request, &mut server.json_events, &mut server.json_down);
}
fn http_event(
    server: &mut Server,
    service: &mut Service,
    domain: Option<&mut Hosted<'_>>,
    env: &Env<Limits>,
    turn: Turn,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    match turn {
        Turn::Call(call) => entrance(server, service, domain, env, call, above),
        Turn::Body(up) => {
            if server.connection.state == State::Body {
                let limits = git::json_limits(server, &env.limits);
                json::up(&mut server.json, &env.with(limits), up, &mut server.json_events, &mut server.json_down);
            }
        }
        Turn::Failed => close(server, service, env, above, below),
        Turn::Done(reuse) => {
            server.git = None;
            server.git_decoder = None;
            server.listing = None;
            server.request = None;
            server.select = response::Select::Document;
            match reuse {
                http::Reuse::Keep => next(server, service, env, above, below),
                http::Reuse::Close => close(server, service, env, above, below),
            }
        }
        Turn::Handled => {}
    }
}
#[expect(clippy::manual_map, reason = "the step subset keeps validator capture explicit")]
fn entrance(
    server: &mut Server,
    service: &mut Service,
    domain: Option<&mut Hosted<'_>>,
    env: &Env<Limits>,
    call: http::Call,
    above: &mut Queue<Event>,
) {
    if let Some(name) = skein_fake_forge_http::git::name(&call.target) {
        git::entrance(server, service, env, &call, name);
        return;
    }
    let user = service.user(call.header(b"authorization"));
    service.tally.count(None, user);
    server.range = response::range(call.header(b"range"));
    server.if_none_match = match call.header(b"if-none-match") {
        Some(value) => Some(bytes::copy_of(value)),
        None => None,
    };
    let Some(user) = user else {
        let method = match call.method {
            Method::Get => Some(request::Method::Get),
            Method::Post => Some(request::Method::Post),
            Method::Put => Some(request::Method::Put),
            Method::Patch => Some(request::Method::Patch),
            Method::Delete => Some(request::Method::Delete),
            Method::Head | Method::Options => None,
        };
        if let Some(method) = method
            && let Ok(documents::Route::Ready(request)) = documents::route(method, &call.target, &env.limits.documents)
        {
            server.requests.push(request);
        }
        server.connection.requests.push(http::Request::Discard);
        fail(server, &env.limits, 401);
        return;
    };
    server.user = user;
    server.scopes = Scopes {
        user: service.reads_user(call.header(b"authorization")),
        admin: service.accesses_admin(call.header(b"authorization"), call.method != Method::Get),
        organization: service.accesses_organization(call.header(b"authorization"), call.method != Method::Get),
        writing: call.method != Method::Get,
        repository: service.accesses_repository(call.header(b"authorization"), call.method != Method::Get),
    };
    let method = match call.method {
        Method::Get => request::Method::Get,
        Method::Post => request::Method::Post,
        Method::Put => request::Method::Put,
        Method::Patch => request::Method::Patch,
        Method::Delete => request::Method::Delete,
        Method::Head | Method::Options => {
            server.connection.requests.push(http::Request::Discard);
            fail(server, &env.limits, 405);
            return;
        }
    };
    match documents::route(method, &call.target, &env.limits.documents) {
        Ok(documents::Route::Ready(request)) => {
            server.connection.requests.push(http::Request::Discard);
            decoded(server, service, domain, env, request, above);
        }
        Ok(documents::Route::Body(decoder)) => {
            server.connection.state = State::Body;
            if server.json_started {
                server.json.restart();
            }
            server.json_started = true;
            let first = json::Request::Next;
            server.decoder = Some(decoder);
            json_down(server, env, first);
        }
        Err(error) => {
            server.connection.requests.push(http::Request::Discard);
            fail(
                server,
                &env.limits,
                match error {
                    documents::Unrouted::NotFound => 404,
                    documents::Unrouted::Method => 405,
                    documents::Unrouted::Malformed => 422,
                },
            );
        }
    }
}
fn decoded(
    server: &mut Server,
    service: &mut Service,
    domain: Option<&mut Hosted<'_>>,
    env: &Env<Limits>,
    request: Request,
    above: &mut Queue<Event>,
) {
    server.requests.push(request.clone());
    let route = crate::route(&request);
    if service.refused == Some(route) {
        fail(server, &env.limits, 403);
        return;
    }
    service.tally.unrouted = service.tally.unrouted.saturating_sub(1);
    let count = service.tally.routes.get(&route).copied().unwrap_or(0).saturating_add(1);
    service.tally.routes.insert(route, count).expect("route capacity");
    server.followed = false;
    if let Request::Merge { style, .. } = &request
        && *style != document::MergeStyle::Squash
        && *style != document::MergeStyle::FastForwardOnly
    {
        respond(server, &env.limits, response::failure(&env.limits, errors::refusal(Refusal::MergeStyle)));
        return;
    }
    if let Some(message) = missing_scope(&request, &server.scopes) {
        respond(
            server,
            &env.limits,
            response::failure(&env.limits, errors::Failure { status: 403, message, retry_after: None }),
        );
        return;
    }
    if route == crate::Route::Migrate || route == crate::Route::DeleteRepository {
        let Ok(id) = service.calls.insert(Call { server: server.owner, active: true }) else {
            fail(server, &env.limits, 503);
            return;
        };
        match service.setup.migration(&request, server.user, &mut service.config, &env.limits, domain) {
            Ok((repository, op)) => {
                server.call = Some(id);
                server.request = Some(request);
                server.select = response::Select::Document;
                server.connection.state = State::Waiting;
                above.push(Event::Domain(domain::Event::Call {
                    reply_to: ReplyTo::new(id.token()),
                    user: server.user,
                    repository,
                    op,
                }));
            }
            Err(reply) => {
                service.calls.retire(id);
                respond(server, &env.limits, reply);
            }
        }
        return;
    }
    if let Some(reply) = service.setup.answer_with_domain(&request, server.user, &mut service.config, env, domain) {
        respond(server, &env.limits, reply);
        return;
    }
    if let Some(answer) = table(service, &request, server.user) {
        match documents::encode(&answer, &env.limits.documents) {
            Ok(body) => respond(server, &env.limits, response::Reply { status: 200, headers: Box::from([]), body }),
            Err(_) => fail(server, &env.limits, 413),
        }
        return;
    }
    if let Request::UserRepos { page } = &request {
        let Ok(id) = service.calls.insert(Call { server: server.owner, active: true }) else {
            fail(server, &env.limits, 503);
            return;
        };
        server.call = Some(id);
        server.listing = Some(crate::setup::repos::Listing::new(*page, &service.config));
        server.request = Some(request);
        server.connection.state = State::Waiting;
        user_repos_next(server, service, env, above);
        return;
    }
    let op = routes::op(&service.config, &request);
    let select = match server.range {
        Ok(range) => response::select(&service.config, &request, range),
        Err(error) => Err(error),
    };
    let repository = routes::repository(&service.config, &request);
    server.request = Some(request);
    match (op, select, repository) {
        (Ok(op), Ok(select), Some(repository)) => {
            let Ok(id) = service.calls.insert(Call { server: server.owner, active: true }) else {
                fail(server, &env.limits, 503);
                return;
            };
            server.call = Some(id);
            server.select = select;
            server.connection.state = State::Waiting;
            above.push(Event::Domain(domain::Event::Call {
                reply_to: ReplyTo::new(id.token()),
                user: server.user,
                repository,
                op,
            }));
        }
        (Err(error), _, _) | (_, Err(error), _) => error_reply(server, env, error),
        (Ok(_), Ok(_), None) => error_reply(server, env, api::Error::Missing(api::What::Repository)),
    }
}
fn missing_scope(request: &Request, scopes: &Scopes) -> Option<&'static [u8]> {
    let (granted, read, write): (bool, &[u8], &[u8]) = match request {
        Request::Repository { .. }
        | Request::Branch { .. }
        | Request::Artifacts { .. }
        | Request::ArtifactZip { .. }
        | Request::Commits { .. }
        | Request::Contents { .. }
        | Request::Raw { .. }
        | Request::ChangeFiles { .. }
        | Request::CreateFile { .. }
        | Request::RepositoryById { .. }
        | Request::RepoSearch { .. }
        | Request::UserRepos { .. }
        | Request::Permission { .. }
        | Request::AddCollaborator { .. }
        | Request::RepoHooks { .. }
        | Request::CreateRepoHook { .. }
        | Request::EditRepoHook { .. }
        | Request::Migrate { .. }
        | Request::DeleteRepoHook { .. } => (
            scopes.repository,
            b"token does not have at least one of required scope(s): [read:repository]",
            b"token does not have at least one of required scope(s): [write:repository]",
        ),
        Request::User | Request::UserOf { .. } => (
            scopes.user,
            b"token does not have at least one of required scope(s): [read:user]",
            b"token does not have at least one of required scope(s): [write:user]",
        ),
        Request::Tokens { .. }
        | Request::CreateToken { .. }
        | Request::DeleteToken { .. }
        | Request::CreateUser { .. }
        | Request::CreateOrg { .. } => (
            scopes.admin,
            b"token does not have at least one of required scope(s): [read:admin]",
            b"token does not have at least one of required scope(s): [write:admin]",
        ),
        Request::Org { .. } | Request::CreateRepository { .. } | Request::DeleteRepository { .. } => (
            scopes.organization,
            b"token does not have at least one of required scope(s): [read:organization]",
            b"token does not have at least one of required scope(s): [write:organization]",
        ),
        Request::ApiSettings
        | Request::UserSearch { .. }
        | Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullFor { .. }
        | Request::Pulls { .. }
        | Request::CombinedStatus { .. }
        | Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::Branches { .. }
        | Request::Protection { .. }
        | Request::Protections { .. }
        | Request::Collaborators { .. }
        | Request::PullFiles { .. }
        | Request::Compare { .. }
        | Request::Tree { .. }
        | Request::WorkflowRuns { .. }
        | Request::Dispatch { .. }
        | Request::Rerun { .. }
        | Request::Runs { .. }
        | Request::Run { .. }
        | Request::Job { .. }
        | Request::Jobs { .. }
        | Request::JobLog { .. }
        | Request::CreateIssue { .. }
        | Request::OpenPull { .. }
        | Request::PostComment { .. }
        | Request::PostReview { .. }
        | Request::EditIssue { .. }
        | Request::RequestReviewers { .. }
        | Request::RemoveReviewers { .. }
        | Request::Merge { .. }
        | Request::Update { .. }
        | Request::PostStatus { .. }
        | Request::CreateBranch { .. }
        | Request::Teams { .. }
        | Request::CreateTeam { .. }
        | Request::EditTeam { .. }
        | Request::TeamMembers { .. }
        | Request::OrgHooks { .. }
        | Request::CreateOrgHook { .. }
        | Request::EditOrgHook { .. }
        | Request::DeleteOrgHook { .. }
        | Request::AddMember { .. }
        | Request::DeleteBranch { .. } => return None,
    };
    if granted {
        None
    } else if scopes.writing {
        Some(write)
    } else {
        Some(read)
    }
}
fn table(service: &Service, request: &Request, user: u64) -> Option<documents::Answer> {
    match request {
        Request::User => Some(documents::Answer::User(service.setup.resolve_user(&service.config, user))),
        Request::ApiSettings => Some(documents::Answer::ApiSettings(document::ApiSettings {
            max_response_items: service.config.max_response_items,
            default_paging_num: service.config.default_paging_num,
        })),
        Request::UserSearch { uid, .. } => Some(documents::Answer::Search {
            ok: true,
            data: Box::new([service.setup.resolve_user(&service.config, *uid)]),
        }),
        Request::RepoSearch { .. }
        | Request::UserRepos { .. }
        | Request::RepositoryById { .. }
        | Request::Repository { .. }
        | Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullFor { .. }
        | Request::Pulls { .. }
        | Request::CombinedStatus { .. }
        | Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::Branch { .. }
        | Request::Branches { .. }
        | Request::Protection { .. }
        | Request::Protections { .. }
        | Request::Collaborators { .. }
        | Request::Permission { .. }
        | Request::PullFiles { .. }
        | Request::Compare { .. }
        | Request::Artifacts { .. }
        | Request::ArtifactZip { .. }
        | Request::Commits { .. }
        | Request::Contents { .. }
        | Request::Raw { .. }
        | Request::Tree { .. }
        | Request::WorkflowRuns { .. }
        | Request::Dispatch { .. }
        | Request::Rerun { .. }
        | Request::Runs { .. }
        | Request::Run { .. }
        | Request::Job { .. }
        | Request::Jobs { .. }
        | Request::JobLog { .. }
        | Request::CreateIssue { .. }
        | Request::OpenPull { .. }
        | Request::PostComment { .. }
        | Request::PostReview { .. }
        | Request::EditIssue { .. }
        | Request::RequestReviewers { .. }
        | Request::RemoveReviewers { .. }
        | Request::Merge { .. }
        | Request::Update { .. }
        | Request::PostStatus { .. }
        | Request::CreateBranch { .. }
        | Request::Tokens { .. }
        | Request::CreateToken { .. }
        | Request::DeleteToken { .. }
        | Request::UserOf { .. }
        | Request::Org { .. }
        | Request::CreateOrg { .. }
        | Request::ChangeFiles { .. }
        | Request::CreateFile { .. }
        | Request::Migrate { .. }
        | Request::DeleteRepository { .. }
        | Request::CreateRepository { .. }
        | Request::Teams { .. }
        | Request::CreateTeam { .. }
        | Request::EditTeam { .. }
        | Request::TeamMembers { .. }
        | Request::OrgHooks { .. }
        | Request::CreateOrgHook { .. }
        | Request::EditOrgHook { .. }
        | Request::DeleteOrgHook { .. }
        | Request::AddCollaborator { .. }
        | Request::RepoHooks { .. }
        | Request::CreateRepoHook { .. }
        | Request::EditRepoHook { .. }
        | Request::DeleteRepoHook { .. }
        | Request::AddMember { .. }
        | Request::CreateUser { .. }
        | Request::DeleteBranch { .. } => None,
    }
}
fn fail(server: &mut Server, limits: &Limits, status: u16) {
    respond(server, limits, response::failure(limits, errors::Failure { status, message: b"", retry_after: None }));
}
fn error_reply(server: &mut Server, env: &Env<Limits>, error: api::Error) {
    let request = server.request.as_ref().expect("decoded request");
    respond(server, &env.limits, response::failure(&env.limits, errors::for_request(error, request, env.now)));
}
fn respond(server: &mut Server, limits: &Limits, mut reply: response::Reply) {
    if reply.status == 200 || reply.status == 206 {
        for header in &reply.headers {
            if header.is(b"ETag") && server.if_none_match.as_deref() == Some(header.value.as_ref()) {
                reply.status = 304;
                reply.body = Box::from([]);
                break;
            }
        }
    }
    let mut headers =
        List::with_capacity(u32::try_from(reply.headers.len()).expect("bounded headers").saturating_add(1));
    let mut content_type = false;
    for header in reply.headers {
        content_type |= header.name.eq_ignore_ascii_case(b"content-type");
        headers.push(header).expect("response headers");
    }
    if !reply.body.is_empty() && !content_type {
        headers
            .push(Header {
                name: bytes::copy_of(b"content-type"),
                value: bytes::copy_of(match server.select {
                    response::Select::Raw(_) => b"text/plain",
                    response::Select::Document
                    | response::Select::Permission(_)
                    | response::Select::Branch(_)
                    | response::Select::Branches(_)
                    | response::Select::Review(_)
                    | response::Select::Status(_) => b"application/json",
                }),
            })
            .expect("content type");
    }
    connection::respond(
        &mut server.connection,
        &limits.http,
        connection::Reply {
            status: reply.status,
            headers: headers.into_boxed(),
            body: reply.body,
            bodyless: reply.status == 204 || reply.status == 304,
        },
    );
}

fn user_repos_next(server: &mut Server, service: &mut Service, env: &Env<Limits>, above: &mut Queue<Event>) {
    let Some(listing) = &mut server.listing else {
        return;
    };
    if let Some(repo) = listing.next(&service.config) {
        let Some(id) = server.call else {
            return;
        };
        listing.current = Some(repo.id);
        above.push(Event::Domain(domain::Event::Call {
            reply_to: ReplyTo::new(id.token()),
            user: server.user,
            repository: repo.full.clone(),
            op: api::Op::Read(api::Read::ListingPermission),
        }));
    } else {
        let reply = listing.reply(&env.limits);
        server.listing = None;
        retire(server, service);
        respond(server, &env.limits, reply);
    }
}

fn malformed(server: &mut Server, env: &Env<Limits>) {
    json_down(server, env, json::Request::Close);
    server.json = Tokenizer::new(&env.limits.documents.json);
    server.json_started = false;
    server.connection.requests.push(http::Request::Discard);
    fail(server, &env.limits, 422);
}
