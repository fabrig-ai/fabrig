//! Forgejo exchanges held by their plans (skein/docs/design/forge/connection.md, 8.1).
use super::{Connection, Ended, Keep, Moved, Payload, Phase, field};
use crate::limits::{self, Limits};
use crate::settings::Forge;
use alloc::boxed::Box;
use skein_forge::forgejo::{
    client as documents,
    decode::Step,
    header,
    plans::{self, Body, Fields, Heard, plan::Range},
    request,
};
use skein_forge::{decode, time};
use skein_http::client as http;
use skein_json::tokenizer;
use skein_lib::{Env, List, Queue, Secret, Writer};

#[derive(Debug)]
pub struct Exchange {
    request: Option<request::Request>,
    decoder: Option<Box<documents::Decoder>>,
    done: Option<Body>,
    heard: Queue<Heard>,
    after: u64,
    keep: Keep,
    head_total: Option<u64>,
}
impl Exchange {
    pub fn new() -> Self {
        Self {
            request: None,
            decoder: None,
            done: None,
            heard: Queue::uncapped(),
            after: 0,
            keep: Keep::Head(0),
            head_total: None,
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the request borrows the component's conditional store while it is composed"
    )]
    pub fn start(
        &mut self,
        c: &mut Connection,
        env: &Env<Limits>,
        forge: &Forge,
        user_agent: &[u8],
        exchange: plans::plan::Exchange,
        authorization: Secret,
        kept: &mut crate::kept::Store,
    ) -> Result<(), documents::Refusal> {
        let plans::plan::Exchange { request, range } = exchange;
        let encoded = documents::encode(&request, &limits::documents(&env.limits))?;
        // Host, Authorization, User-Agent, Content-Type and Range; HTTP adds Content-Length.
        let mut headers = List::with_capacity(5);
        headers.push(field(b"Host", &c.host(forge))).expect("five Forgejo header fields");
        headers.push(field(b"Authorization", authorization.expose())).expect("five Forgejo header fields");
        headers.push(field(b"User-Agent", user_agent)).expect("five Forgejo header fields");
        let method = match encoded.method {
            request::Method::Get => skein_http::Method::Get,
            request::Method::Post => skein_http::Method::Post,
            request::Method::Put => skein_http::Method::Put,
            request::Method::Patch => skein_http::Method::Patch,
            request::Method::Delete => skein_http::Method::Delete,
        };
        let body = Connection::body(&mut headers, encoded.body.as_deref());
        if let Some(range) = range {
            let (prefix, max) = match range {
                Range::Head(n) => (b"bytes=0-".as_slice(), n.saturating_sub(1)),
                Range::Tail(n) => (b"bytes=-".as_slice(), n),
            };
            let decimal = skein_lib::Decimal::of(u64::from(max));
            let mut value = Writer::new(decimal.as_bytes().len().saturating_add(prefix.len()));
            value.put(prefix).expect("range prefix measured");
            value.put(decimal.as_bytes()).expect("range measured");
            headers
                .push(skein_http::Header { name: Box::from(&b"Range"[..]), value: value.finish() })
                .expect("five Forgejo header fields");
        }
        self.head_total = None;
        self.request = Some(request);
        self.decoder = None;
        self.done = None;
        self.after = 0;
        self.keep = match range {
            Some(Range::Head(bytes)) => Keep::Head(bytes),
            Some(Range::Tail(bytes)) => Keep::Tail(bytes),
            None => Keep::Head(c.raw_cap),
        };
        c.send(
            env,
            http::Call { method, target: encoded.target, headers: headers.into_boxed(), body, close: false },
            encoded.body,
            kept,
        );
        Ok(())
    }
    fn response(&mut self, c: &mut Connection, env: &Env<Limits>, response: http::Response) {
        let date = match response.header(b"Date") {
            Some(date) => time::http_date(date),
            None => None,
        };
        let retry_after = match response.header(b"Retry-After") {
            Some(value) => header::retry_after(value, date.unwrap_or(env.wall)),
            None => None,
        };
        let has_more = match response.header(b"X-HasMore") {
            Some(value) => header::has_more(value),
            None => None,
        };
        let content_range = match response.header(b"Content-Range") {
            Some(value) => header::content_range(value),
            None => None,
        };
        self.head_total = match self.keep {
            Keep::Head(_) => match content_range {
                Some(range) => range.total,
                None => None,
            },
            Keep::Tail(_) => None,
        };
        let close = match response.header(b"Connection") {
            Some(value) => value.eq_ignore_ascii_case(b"close"),
            None => false,
        };
        self.heard.push(Heard::Head {
            status: response.status,
            fields: Fields { date, retry_after, has_more, content_range, close },
        });
        let l = limits::documents(&env.limits);
        let request = self.request.as_ref().expect("head belongs to a request");
        if response.status == 401 || response.status == 403 || response.status == 404 {
            c.discard(env);
            return;
        }
        let decoder =
            if response.status >= 300 { Some(documents::error_decoder(&l)) } else { documents::decoder(request, &l) };
        match decoder {
            Some(mut decoder) => {
                documents::keep_after(&mut decoder, self.after);
                self.decoder = Some(Box::new(decoder));
                c.begin_json(env, l.json);
            }
            None => {
                if raw_request(request) {
                    c.begin_raw(env, response.framing, self.keep, c.raw_cap, false);
                } else {
                    c.discard(env);
                }
            }
        }
    }
    fn token(&mut self, c: &mut Connection, env: &Env<Limits>, event: tokenizer::Event) {
        let Some(decoder) = self.decoder.as_mut() else { return };
        match documents::step(decoder, &limits::documents(&env.limits), &event) {
            Step::Demand(request) => c.json_request(env, request),
            Step::Row(row, request) => {
                self.heard.push(Heard::Row(row));
                c.set_json_next(request);
            }
            Step::Done(document) => self.done = Some(Body::Document(document)),
            Step::Malformed(decode::Malformed::Json(tokenizer::Error::Stream(_))) => c.failed(super::Failure::Seen),
            Step::Malformed(error) => c.malformed_json(error),
        }
    }
    pub fn keep_after(&mut self, after: u64) {
        self.after = after;
        if let Some(decoder) = &mut self.decoder {
            documents::keep_after(decoder, after);
        }
    }
    pub fn cut(&mut self, c: &mut Connection) {
        c.cancel_capture();
        if let Some(decoder) = &mut self.decoder {
            let request = documents::cut(decoder);
            c.set_json_next(request);
        }
    }
    /// Routes the same bounded run of movements before handing an answer to the plan.
    pub fn pump(
        &mut self,
        c: &mut Connection,
        env: &Env<Limits>,
        io: &mut Queue<skein_io::Request>,
        kept: &mut crate::kept::Store,
    ) {
        for _ in 0_u32..64 {
            match c.step(env, io, !self.heard.is_empty(), kept) {
                Moved::Within => {}
                Moved::Nothing => break,
                Moved::Head(response) => self.response(c, env, response),
                Moved::Token(event) => self.token(c, env, event),
                Moved::Finished { reuse, raw } => {
                    let body = match raw {
                        Payload::Json => {
                            self.done.take().unwrap_or(Body::Malformed(decode::Malformed::Missing { field: b"" }))
                        }
                        Payload::Empty => self.done.take().unwrap_or(Body::Empty),
                        Payload::Raw { bytes, more } => {
                            let omitted = match self.head_total {
                                Some(total) => total > skein_lib::len_u64(bytes.len()),
                                None => false,
                            };
                            Body::Raw { bytes, more: more || omitted }
                        }
                    };
                    let closes = match &body {
                        Body::Malformed(_) | Body::Raw { more: true, .. } => true,
                        Body::Document(_) | Body::Raw { more: false, .. } | Body::Empty => false,
                    };
                    self.request = None;
                    self.decoder = None;
                    self.heard.push(Heard::Done(body));
                    c.finish(env, reuse, closes);
                }
            }
        }
        if c.phase() == Phase::Closed {
            self.request = None;
            self.decoder = None;
        }
    }
    pub fn heard(&mut self, c: &mut Connection) -> Option<Heard> {
        if let Some(heard) = self.heard.pop() {
            return Some(heard);
        }
        let ended = c.take_ended()?;
        self.request = None;
        self.decoder = None;
        Some(match ended {
            Ended::Failed(failure) => Heard::Failed(failure),
            Ended::Malformed(error) => Heard::Done(Body::Malformed(error)),
        })
    }
    pub fn ready(&self, c: &Connection) -> bool {
        !self.heard.is_empty() || c.ready()
    }
}

fn raw_request(request: &request::Request) -> bool {
    match request {
        request::Request::ArtifactZip { .. } | request::Request::Raw { .. } | request::Request::JobLog { .. } => true,
        request::Request::Artifacts { .. } | request::Request::Contents { .. }
        | request::Request::User
        | request::Request::ApiSettings
        | request::Request::RepositoryById { .. }
        | request::Request::Repository { .. }
        // user/repos is used by the setup host, not the connection.
        | request::Request::UserRepos { .. }
        // repos/search is used by the setup host, not the connection.
        | request::Request::RepoSearch { .. }
        | request::Request::UserSearch { .. }
        | request::Request::Issues { .. }
        | request::Request::Issue { .. }
        | request::Request::Comments { .. }
        | request::Request::Pull { .. }
        | request::Request::PullFor { .. }
        | request::Request::CombinedStatus { .. }
        | request::Request::Reviews { .. }
        | request::Request::ReviewComments { .. }
        | request::Request::Branch { .. }
        | request::Request::Branches { .. }
        | request::Request::Protection { .. }
        | request::Request::Protections { .. }
        | request::Request::Collaborators { .. }
        | request::Request::Permission { .. }
        | request::Request::Pulls { .. }
        | request::Request::PullFiles { .. }
        | request::Request::Compare { .. }
        | request::Request::Commits { .. }
        | request::Request::Tree { .. }
        | request::Request::WorkflowRuns { .. }
        | request::Request::Dispatch { .. }
        | request::Request::Rerun { .. }
        | request::Request::Runs { .. }
        | request::Request::Run { .. }
        | request::Request::Job { .. }
        | request::Request::Jobs { .. }
        | request::Request::CreateIssue { .. }
        | request::Request::OpenPull { .. }
        | request::Request::PostComment { .. }
        | request::Request::PostReview { .. }
        | request::Request::EditIssue { .. }
        | request::Request::RequestReviewers { .. }
        | request::Request::RemoveReviewers { .. }
        | request::Request::Merge { .. }
        | request::Request::Update { .. }
        | request::Request::PostStatus { .. }
        | request::Request::CreateBranch { .. }
        | request::Request::Tokens { .. }
        | request::Request::CreateToken { .. }
        | request::Request::DeleteToken { .. }
        | request::Request::UserOf { .. }
        | request::Request::Org { .. }
        | request::Request::CreateOrg { .. }
        | request::Request::ChangeFiles { .. }
        | request::Request::CreateFile { .. }
        | request::Request::Migrate { .. }
        | request::Request::DeleteRepository { .. }
        | request::Request::CreateRepository { .. }
        | request::Request::Teams { .. }
        | request::Request::CreateTeam { .. }
        | request::Request::EditTeam { .. }
        | request::Request::TeamMembers { .. }
        | request::Request::OrgHooks { .. }
        | request::Request::CreateOrgHook { .. }
        | request::Request::EditOrgHook { .. }
        | request::Request::DeleteOrgHook { .. }
        | request::Request::AddMember { .. }
        // setup requests are the setup host's, not the connection's.
        | request::Request::AddCollaborator { .. }
        | request::Request::RepoHooks { .. }
        | request::Request::CreateRepoHook { .. }
        | request::Request::EditRepoHook { .. }
        | request::Request::DeleteRepoHook { .. }
        | request::Request::CreateUser { .. }
        | request::Request::DeleteBranch { .. } => false,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
impl Exchange {
    pub(crate) fn test_fresh(&self) -> bool {
        self.request.is_none() && self.decoder.is_none() && self.done.is_none() && self.heard.is_empty()
    }
    pub(crate) fn test_hold_heard(&mut self) {
        self.heard.push(Heard::Done(Body::Empty));
    }
}
