//! One request per connection (skein/docs/design/forge/setup/README.md, 4.3),
//! over the transport of skein/docs/design/net/link.md, 3.
use super::{
    Limits, limits,
    settings::{self, Forge, Transport},
};
use crate::{
    call::{Auth, Call},
    reconcile::Reply,
};
use alloc::boxed::Box;
use skein_forge::{
    credential::Credential,
    forgejo::{
        client as documents,
        decode::Step,
        document::{Document, Row},
        request,
    },
    github as dialect,
};
use skein_http::client as http;
use skein_json::tokenizer;
use skein_lib::{
    Duration, Env, List, Queue, Secret, Token, Wall, bytes,
    stream::{Down, Read, Up},
    usize_of,
};

pub(crate) struct Connection {
    pub(super) run: Token,
    pub(super) forge: u16,
    pub(crate) link: skein_link::Link,
    pub(super) closed: bool,
    pub(super) closing: Option<bool>,
    asked: Option<bool>,
    pub(crate) result: Option<Reply>,
    pub(crate) words: Option<Box<[u8]>>,
    refusal: Option<u16>,
    answered: bool,
    http: http::Client,
    http_up: Queue<http::Event>,
    http_down: Queue<Down>,
    json: tokenizer::Tokenizer,
    json_up: Queue<tokenizer::Event>,
    json_down: Queue<Down>,
    json_next: Option<tokenizer::Request>,
    speech: Speech,
    pub(crate) call: Option<http::Call>,
    pub(crate) upload: Option<skein_link::Upload>,
}
#[expect(clippy::large_enum_variant, reason = "a connection reserves one concrete decoder for either dialect")]
enum Speech {
    Forgejo {
        request: request::Request,
        decoder: Option<documents::Decoder>,
        document: Option<Document>,
        rows: List<Row>,
    },
    GitHub {
        kind: Option<dialect::document::Kind>,
        decoder: Option<dialect::decode::Decoder>,
        document: Option<dialect::document::Document>,
        rows: List<dialect::document::Row>,
    },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Refused {
    Unexpected,
    Unsigned,
}
impl Connection {
    pub(crate) fn new(
        run: Token,
        forge: &Forge,
        agent: &[u8],
        call: &Call,
        token: &Secret,
        l: &Limits,
    ) -> Option<Self> {
        let capacity = row_bound(call)?;
        let request = match call {
            Call::Forgejo(request) => request,
            Call::GitHub(_) => return None,
        };
        let encoded = documents::encode(request, &limits::documents(l)).ok()?;
        let mut headers = List::with_capacity(4);
        headers.push(field(b"Host", &settings::host(forge)?)).expect("four headers");
        let auth = skein_forge::forgejo::credential::authorization(&Credential::Token(token.clone()))?;
        headers.push(field(b"Authorization", auth.expose())).expect("four headers");
        headers.push(field(b"User-Agent", agent)).expect("four headers");
        let body = match &encoded.body {
            Some(body) => {
                headers.push(field(b"Content-Type", b"application/json")).expect("four headers");
                http::Body::Length(skein_lib::len_u64(body.len()))
            }
            None => http::Body::None,
        };
        let method = match encoded.method {
            request::Method::Get => skein_http::Method::Get,
            request::Method::Post => skein_http::Method::Post,
            request::Method::Put => skein_http::Method::Put,
            request::Method::Patch => skein_http::Method::Patch,
            request::Method::Delete => skein_http::Method::Delete,
        };
        let tls = tls(forge, l);
        Some(Self {
            run,
            forge: forge.number(),
            closed: false,
            closing: None,
            asked: None,
            result: None,
            words: None,
            refusal: None,
            answered: false,
            http: http::Client::new(&limits::http_transport(l, tls.is_some())),
            link: skein_link::Link::new(tls),
            http_up: Queue::uncapped(),
            http_down: Queue::uncapped(),
            json: tokenizer::Tokenizer::new(&limits::documents(l).json),
            json_up: Queue::uncapped(),
            json_down: Queue::uncapped(),
            json_next: None,
            speech: Speech::Forgejo {
                request: request.clone(),
                decoder: None,
                document: None,
                rows: List::with_capacity(capacity),
            },
            call: Some(http::Call { method, target: encoded.target, headers: headers.into_boxed(), body, close: true }),
            upload: skein_link::Upload::of(encoded.body),
        })
    }
    pub(crate) fn github(
        run: Token,
        forge: &Forge,
        agent: &[u8],
        call: &Call,
        auth: &Auth,
        wall: Wall,
        l: &Limits,
    ) -> Result<Self, Refused> {
        let capacity = row_bound(call).ok_or(Refused::Unexpected)?;
        let request = match call {
            Call::GitHub(request) => request,
            Call::Forgejo(_) => return Err(Refused::Unexpected),
        };
        let kind = kind_of(request)?;
        let encoded = dialect::request::encode(request, &limits::github(l)).or(Err(Refused::Unexpected))?;
        let authorization = authorization(auth, encoded.auth, wall)?;
        let mut headers = List::with_capacity(6);
        let host = settings::host(forge).ok_or(Refused::Unexpected)?;
        headers.push(field(b"Host", &host)).expect("six headers");
        headers.push(field(b"User-Agent", agent)).expect("six headers");
        if let Some(authorization) = authorization {
            headers.push(field(b"Authorization", authorization.expose())).expect("six headers");
        }
        for (name, value) in dialect::request::fields(&encoded) {
            headers.push(field(name, value)).expect("six headers");
        }
        let body = match &encoded.body {
            Some(body) => {
                headers.push(field(b"Content-Type", b"application/json")).expect("six headers");
                http::Body::Length(skein_lib::len_u64(body.len()))
            }
            None => http::Body::None,
        };
        let method = match encoded.method {
            dialect::request::Method::Get => skein_http::Method::Get,
            dialect::request::Method::Post => skein_http::Method::Post,
            dialect::request::Method::Put => skein_http::Method::Put,
            dialect::request::Method::Patch => skein_http::Method::Patch,
            dialect::request::Method::Delete => skein_http::Method::Delete,
        };
        let tls = tls(forge, l);
        Ok(Self {
            run,
            forge: forge.number(),
            closed: false,
            closing: None,
            asked: None,
            result: None,
            words: None,
            refusal: None,
            answered: false,
            http: http::Client::new(&limits::http_transport(l, tls.is_some())),
            link: skein_link::Link::new(tls),
            http_up: Queue::uncapped(),
            http_down: Queue::uncapped(),
            json: tokenizer::Tokenizer::new(&limits::github(l).json),
            json_up: Queue::uncapped(),
            json_down: Queue::uncapped(),
            json_next: None,
            speech: Speech::GitHub { kind, decoder: None, document: None, rows: List::with_capacity(capacity) },
            call: Some(http::Call { method, target: encoded.target, headers: headers.into_boxed(), body, close: true }),
            upload: skein_link::Upload::of(encoded.body),
        })
    }
    fn json_limits(&self, l: &Limits) -> tokenizer::Limits {
        self.document_limits(l).json
    }
    fn document_limits(&self, l: &Limits) -> skein_forge::forgejo::limits::Limits {
        let mut documents = limits::documents(l);
        if self.refusal.is_some() {
            // Keep the bounded error document's message before cutting its UTF-8 words (6.3).
            documents.string_bytes = l.body_bytes_in;
            documents.json.string = l.body_bytes_in;
        }
        match &self.speech {
            Speech::Forgejo { .. } => documents,
            Speech::GitHub { .. } => limits::github(l),
        }
    }
    pub(crate) fn connect(&mut self, env: &Env<Limits>) {
        match self.link.connected(&link_env(env)) {
            skein_link::Start::Ready => self.start(env),
            skein_link::Start::Handshaking | skein_link::Start::Closing => (),
        }
    }
    fn start(&mut self, env: &Env<Limits>) {
        if let Some(call) = self.call.take() {
            self.http_down(env, http::Request::Call(call));
            if self.upload.is_some() {
                self.http_down(
                    env,
                    http::Request::Upload(Down::Demand {
                        read: Read::Nothing,
                        room: limits::http_transport(&env.limits, self.link.encrypted()).send,
                    }),
                );
            }
        }
    }
    pub(crate) fn stream(&mut self, env: &Env<Limits>, event: Up) {
        self.link.up(&link_env(env), event);
    }
    fn http_stream(&mut self, env: &Env<Limits>, event: Up) {
        let h = env.with(limits::http_transport(&env.limits, self.link.encrypted()));
        http::up(&mut self.http, &h, event, &mut self.http_up, &mut self.http_down);
    }
    fn http_down(&mut self, env: &Env<Limits>, request: http::Request) {
        let h = env.with(limits::http_transport(&env.limits, self.link.encrypted()));
        http::down(&mut self.http, &h, request, &mut self.http_up, &mut self.http_down);
    }
    fn json_down(&mut self, env: &Env<Limits>, request: tokenizer::Request) {
        let j = env.with(self.json_limits(&env.limits));
        tokenizer::down(&mut self.json, &j, request, &mut self.json_up, &mut self.json_down);
    }
    pub(super) fn fail(&mut self) {
        if !self.answered {
            let reply = match self.refusal {
                Some(status) => Reply::Status(status),
                None => Reply::Down,
            };
            self.words = None;
            self.finish(reply);
        }
    }
    fn finish(&mut self, reply: Reply) {
        self.result = Some(reply);
        self.answered = true;
        self.closing = Some(false);
        self.upload = None;
        self.call = None;
    }
    pub(super) fn abort(&mut self) {
        self.answered = true;
        self.closing = Some(true);
        self.result = None;
        self.words = None;
        self.upload = None;
        self.call = None;
    }
    /// Routes one owned movement, with room reserved for one io request.
    pub(crate) fn pump(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>) -> bool {
        if let Some(abort) = self.closing {
            if self.asked != Some(abort) {
                self.asked = Some(abort);
                self.link.close(&link_env(env), abort, io);
                return true;
            }
            return match self.link.pump(io) {
                skein_link::Moved::Nothing => false,
                skein_link::Moved::Within | skein_link::Moved::Sent | skein_link::Moved::Above(_) => true,
            };
        }
        match self.link.pump(io) {
            skein_link::Moved::Above(event) => {
                match event {
                    skein_link::Event::Ready => self.start(env),
                    skein_link::Event::Stream(event) => self.http_stream(env, event),
                    skein_link::Event::Failed => self.fail(),
                }
                return true;
            }
            skein_link::Moved::Within | skein_link::Moved::Sent => return true,
            skein_link::Moved::Nothing => (),
        }
        if let Some(down) = self.http_down.pop() {
            self.link.down(&link_env(env), down);
            return true;
        }
        if let Some(down) = self.json_down.pop() {
            self.http_down(env, http::Request::Body(down));
            return true;
        }
        if let Some(next) = self.json_next.take() {
            self.json_down(env, next);
            return true;
        }
        if let Some(event) = self.json_up.pop() {
            self.json_event(env, &event);
            return true;
        }
        if let Some(event) = self.http_up.pop() {
            self.http_event(env, event);
            return true;
        }
        if self.closed && !self.answered {
            self.fail();
            return true;
        }
        false
    }
    pub(super) fn ready(&self) -> bool {
        if self.closing.is_some() {
            return self.asked != self.closing || self.link.has_work();
        }
        self.link.has_work()
            || !self.http_down.is_empty()
            || !self.http_up.is_empty()
            || !self.json_down.is_empty()
            || !self.json_up.is_empty()
            || self.json_next.is_some()
            || self.closed
    }
    fn http_event(&mut self, env: &Env<Limits>, event: http::Event) {
        match event {
            http::Event::Response(response) => {
                if (200..300).contains(&response.status) {
                    match &mut self.speech {
                        Speech::Forgejo { request, decoder, .. } => {
                            *decoder = documents::decoder(request, &limits::documents(&env.limits));
                            match decoder {
                                Some(_) => self.json_next = Some(tokenizer::Request::Next),
                                None => self.finish(Reply::Empty),
                            }
                        }
                        Speech::GitHub { kind, decoder, .. } => match kind {
                            Some(kind) => {
                                *decoder = Some(dialect::decode::decoder(*kind, &limits::github(&env.limits)));
                                self.json_next = Some(tokenizer::Request::Next);
                            }
                            None => self.finish(Reply::Empty),
                        },
                    }
                } else {
                    match &mut self.speech {
                        Speech::Forgejo { request: request::Request::Migrate { .. }, decoder, .. }
                            if response.status != 401 =>
                        {
                            self.refusal = Some(response.status);
                            let mut l = limits::documents(&env.limits);
                            l.string_bytes = env.limits.body_bytes_in;
                            l.json.string = env.limits.body_bytes_in;
                            self.json = tokenizer::Tokenizer::new(&l.json);
                            *decoder = Some(documents::error_decoder(&l));
                            self.json_next = Some(tokenizer::Request::Next);
                        }
                        Speech::Forgejo { .. } | Speech::GitHub { .. } => {
                            self.finish(Reply::Status(response.status));
                        }
                    }
                }
            }
            http::Event::Body(event) => {
                let j = env.with(self.json_limits(&env.limits));
                tokenizer::up(&mut self.json, &j, event, &mut self.json_up, &mut self.json_down);
            }
            http::Event::Upload(event) => match event {
                Up::Room => self.upload(env),
                Up::Failed(_) | Up::Bytes(_) | Up::End => self.fail(),
            },
            http::Event::Done(_) => {
                if let Some(status) = self.refusal {
                    self.finish(Reply::Status(status));
                    return;
                }
                let reply = match &mut self.speech {
                    Speech::Forgejo { document, rows, .. } => match document.take() {
                        Some(Document::Listed { skipped: 0 } | Document::RepoSearch { ok: true }) => {
                            let rows = core::mem::replace(rows, List::with_capacity(0));
                            Reply::Rows(rows.into_boxed())
                        }
                        Some(document) => Reply::Document(Box::new(document)),
                        None => Reply::Down,
                    },
                    Speech::GitHub { document, rows, .. } => match document.take() {
                        Some(dialect::document::Document::Rows) => {
                            let rows = core::mem::replace(rows, List::with_capacity(0));
                            Reply::GitHubRows(rows.into_boxed())
                        }
                        Some(document) => Reply::GitHubDocument(document),
                        None => Reply::Down,
                    },
                };
                self.finish(reply);
            }
            http::Event::Failed(_) => self.fail(),
            http::Event::Closed => (),
        }
    }
    fn json_event(&mut self, env: &Env<Limits>, event: &tokenizer::Event) {
        let l = self.document_limits(&env.limits);
        match &mut self.speech {
            Speech::Forgejo { decoder, document, rows, .. } => {
                let Some(decoder) = decoder else {
                    return;
                };
                match documents::step(decoder, &l, event) {
                    Step::Demand(next) => self.json_next = Some(next),
                    Step::Row(row, next) => {
                        if rows.push(row).is_err() {
                            self.fail();
                        } else {
                            self.json_next = Some(next);
                        }
                    }
                    Step::Done(Document::ApiSettings(mut settings)) => {
                        // Every setup listing uses the effective cap for sending and finishing.
                        settings.max_response_items = settings.max_response_items.min(env.limits.inner);
                        *document = Some(Document::ApiSettings(settings));
                    }
                    Step::Done(Document::Error(error)) if self.refusal.is_some() => match error.message {
                        skein_forge::text::Body::Kept(message) => {
                            let cap = usize_of(env.limits.string_bytes);
                            let words = bytes::utf8_prefix(&message, cap);
                            if !words.is_empty() {
                                self.words = Some(bytes::copy_of(words));
                            }
                        }
                        skein_forge::text::Body::Long(_) => (),
                    },
                    Step::Done(done) => *document = Some(done),
                    Step::Malformed(_) => self.fail(),
                }
            }
            Speech::GitHub { decoder, document, rows, .. } => {
                let Some(decoder) = decoder else {
                    return;
                };
                let (next, step) = decoder.step(event.clone());
                match step {
                    dialect::decode::Step::Demand => self.json_next = Some(next),
                    dialect::decode::Step::Row(row) => {
                        if rows.push(row).is_err() {
                            self.fail();
                        } else {
                            self.json_next = Some(next);
                        }
                    }
                    dialect::decode::Step::Done(done) => *document = Some(done),
                    dialect::decode::Step::Malformed(_) => self.fail(),
                }
            }
        }
    }
    fn upload(&mut self, env: &Env<Limits>) {
        let Some(upload) = &mut self.upload else {
            return;
        };
        let (send, next) = upload.room(limits::http_transport(&env.limits, self.link.encrypted()).send);
        if next == Down::Finish {
            self.upload = None;
        }
        self.http_down(env, http::Request::Upload(send));
        self.http_down(env, http::Request::Upload(next));
    }
}
/// Rows in one setup reply, from its requested page or the pinned installation page.
/// Unpaged listings and requests outside setup have no admitted row bound.
pub(crate) fn row_bound(call: &Call) -> Option<u32> {
    match call {
        Call::Forgejo(request) => forgejo_row_bound(request),
        Call::GitHub(request) => github_row_bound(request),
    }
}
fn forgejo_row_bound(request: &request::Request) -> Option<u32> {
    use request::Request;
    match request {
        Request::Tokens { page, .. }
        | Request::RepoHooks { page, .. }
        | Request::RepoSearch { page }
        | Request::UserRepos { page } => page.limit,
        Request::AddCollaborator { .. }
        | Request::CreateRepoHook { .. }
        | Request::EditRepoHook { .. }
        | Request::DeleteRepoHook { .. }
        | Request::Org { .. }
        | Request::CreateOrg { .. }
        | Request::CreateFile { .. }
        | Request::Migrate { .. }
        | Request::DeleteRepository { .. }
        | Request::CreateRepository { .. }
        | Request::Repository { .. }
        | Request::Branch { .. }
        | Request::Raw { .. }
        | Request::User
        | Request::ApiSettings
        | Request::RepositoryById { .. }
        | Request::Permission { .. }
        | Request::CreateToken { .. }
        | Request::DeleteToken { .. }
        | Request::UserOf { .. }
        | Request::CreateUser { .. } => Some(0),
        Request::Contents { .. }
        | Request::OrgHooks { .. }
        | Request::CreateOrgHook { .. }
        | Request::EditOrgHook { .. }
        | Request::DeleteOrgHook { .. }
        | Request::Teams { .. }
        | Request::CreateTeam { .. }
        | Request::EditTeam { .. }
        | Request::TeamMembers { .. }
        | Request::AddMember { .. }
        | Request::UserSearch { .. }
        | Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullFor { .. }
        | Request::CombinedStatus { .. }
        | Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::Branches { .. }
        | Request::Protection { .. }
        | Request::Protections { .. }
        | Request::Collaborators { .. }
        | Request::Pulls { .. }
        | Request::PullFiles { .. }
        | Request::Compare { .. }
        | Request::Artifacts { .. }
        | Request::ArtifactZip { .. }
        | Request::Commits { .. }
        | Request::Tree { .. }
        | Request::WorkflowRuns { .. }
        | Request::Dispatch { .. }
        | Request::Rerun { .. }
        | Request::Runs { .. }
        | Request::Run { .. }
        | Request::Jobs { .. }
        | Request::Job { .. }
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
        | Request::DeleteBranch { .. } => None,
    }
}
fn github_row_bound(request: &dialect::request::Request) -> Option<u32> {
    use dialect::request::Request;
    match request {
        Request::Installations { .. } => Some(dialect::pin::PAGE_CAP),
        Request::Reachable { per_page, .. } => Some(*per_page),
        Request::Installation { .. }
        | Request::Mint { .. }
        | Request::Viewer
        | Request::App
        | Request::HookConfig
        | Request::SetHook { .. }
        | Request::Uninstall { .. }
        | Request::Conversion { .. }
        | Request::BotUser { .. } => Some(0),
        Request::LogPage { .. }
        | Request::LogDelivery { .. }
        | Request::Merge { .. }
        | Request::Enqueue { .. }
        | Request::PullGraphql { .. }
        | Request::Repository { .. }
        | Request::Locate { .. }
        | Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullsFor { .. }
        | Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::CombinedStatus { .. }
        | Request::CheckRuns { .. }
        | Request::Job { .. }
        | Request::Followed { .. }
        | Request::JobLog { .. }
        | Request::Branch { .. }
        | Request::Branches { .. }
        | Request::PullFiles { .. }
        | Request::Compare { .. }
        | Request::Commits { .. }
        | Request::Raw { .. }
        | Request::RulesForBranch { .. }
        | Request::Ruleset { .. }
        | Request::ClassicProtection { .. }
        | Request::Collaborators { .. }
        | Request::Permission { .. }
        | Request::User { .. }
        | Request::Reopen { .. }
        | Request::Edit { .. }
        | Request::Status { .. }
        | Request::CreateBranch { .. }
        | Request::Reviewers { .. }
        | Request::Update { .. }
        | Request::Review { .. }
        | Request::Post { .. }
        | Request::CreateIssue { .. }
        | Request::OpenPull { .. }
        | Request::Close { .. }
        | Request::DeleteBranch { .. } => None,
    }
}

fn tls(forge: &Forge, l: &Limits) -> Option<skein_tls::client::Client> {
    match &forge.transport {
        Transport::Plaintext { .. } => None,
        Transport::Tls { server_name, trust } => {
            Some(skein_tls::client::Client::new(trust, server_name.clone(), &l.tls))
        }
    }
}
fn field(name: &[u8], value: &[u8]) -> skein_http::Header {
    skein_http::Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }
}
impl core::fmt::Debug for Connection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Connection")
            .field("run", &self.run)
            .field("closing", &self.closing)
            .field("closed", &self.closed)
            .finish_non_exhaustive()
    }
}

/// The setup document a request is answered with, or no document for uninstall.
pub(crate) fn kind_of(request: &dialect::request::Request) -> Result<Option<dialect::document::Kind>, Refused> {
    use dialect::{document::Kind, request::Request};
    Ok(Some(match request {
        Request::Installations { .. } => Kind::InstallationRows,
        Request::Installation { .. } => Kind::Installation,
        Request::Mint { .. } => Kind::Minted,
        Request::Reachable { .. } => Kind::ReachableRows,
        Request::Viewer => Kind::Viewer,
        Request::App => Kind::App,
        Request::HookConfig | Request::SetHook { .. } => Kind::HookConfig,
        Request::Conversion { .. } => Kind::Conversion,
        Request::BotUser { .. } => Kind::BotUser,
        Request::Uninstall { .. } => return Ok(None),
        Request::PullGraphql { .. }
        | Request::Repository { .. }
        | Request::Locate { .. }
        | Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullsFor { .. }
        | Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::CombinedStatus { .. }
        | Request::CheckRuns { .. }
        | Request::Job { .. }
        | Request::JobLog { .. }
        | Request::Followed { .. }
        | Request::Branch { .. }
        | Request::Branches { .. }
        | Request::PullFiles { .. }
        | Request::Compare { .. }
        | Request::Commits { .. }
        | Request::Raw { .. }
        | Request::RulesForBranch { .. }
        | Request::Ruleset { .. }
        | Request::ClassicProtection { .. }
        | Request::Collaborators { .. }
        | Request::Permission { .. }
        | Request::User { .. }
        | Request::Reopen { .. }
        | Request::Edit { .. }
        | Request::Status { .. }
        | Request::CreateBranch { .. }
        | Request::Update { .. }
        | Request::Review { .. }
        | Request::Post { .. }
        | Request::CreateIssue { .. }
        | Request::OpenPull { .. }
        | Request::Close { .. }
        | Request::DeleteBranch { .. }
        | Request::Reviewers { .. }
        | Request::Merge { .. }
        | Request::LogPage { .. }
        | Request::LogDelivery { .. }
        | Request::Enqueue { .. } => return Err(Refused::Unexpected),
    }))
}
fn authorization(auth: &Auth, encoded: dialect::request::Auth, wall: Wall) -> Result<Option<Secret>, Refused> {
    use dialect::request::Auth as Selected;
    match auth {
        Auth::Token(_) => match encoded {
            Selected::App | Selected::Installation | Selected::None => Err(Refused::Unexpected),
        },
        Auth::App { id, key } => match encoded {
            Selected::App => {
                let issued = Wall::from_nanos(wall.as_nanos().saturating_sub(Duration::from_secs(60).as_nanos()));
                let expires = Wall::from_nanos(issued.as_nanos().saturating_add(dialect::jwt::LIFE.as_nanos()));
                let input = dialect::jwt::signing_input(*id, issued, expires).ok_or(Refused::Unsigned)?;
                let signature = skein_tls::sign_rs256(key.expose(), &input).ok_or(Refused::Unsigned)?;
                let jwt = dialect::jwt::token(&input, &signature).ok_or(Refused::Unsigned)?;
                let bearer = dialect::credential::bearer(&jwt).ok_or(Refused::Unsigned)?;
                Ok(Some(bearer))
            }
            Selected::Installation | Selected::None => Err(Refused::Unexpected),
        },
        Auth::Bearer(token) => match encoded {
            Selected::Installation => {
                let bearer = dialect::credential::bearer(token).ok_or(Refused::Unexpected)?;
                Ok(Some(bearer))
            }
            Selected::App | Selected::None => Err(Refused::Unexpected),
        },
        Auth::None => match encoded {
            Selected::None => Ok(None),
            Selected::App | Selected::Installation => Err(Refused::Unexpected),
        },
    }
}

fn link_env(env: &Env<Limits>) -> Env<skein_link::Limits> {
    env.with(skein_link::Limits { tls: env.limits.tls, close_timeout: env.limits.io.close_timeout })
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod tests;
