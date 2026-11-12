//! GitHub exchanges held by their plans (skein/docs/design/forge/connection.md, 8.1).
use super::{Connection, Ended, Keep, Moved, Payload, Phase, field};
use crate::{
    limits::{self, Limits},
    settings::Forge,
};
use alloc::boxed::Box;
use github::{
    plans::plan::Retention,
    plans::{Body, Heard},
};
use skein_forge::{decode, github};
use skein_http::client as http;
use skein_json::tokenizer;
use skein_lib::{Env, List, Queue, Secret, usize_of};

// Host, User-Agent, Accept, pin, Authorization and Content-Type; HTTP adds Content-Length.
const HEADERS: u32 = 6;
#[derive(Debug)]
pub struct Exchange {
    request: Option<Box<github::plans::plan::Exchange>>,
    decoder: Option<Box<github::decode::Decoder>>,
    // Completed documents occupy room only while waiting for the transport end.
    done: Option<Box<Body>>,
    heard: Queue<Heard>,
}
impl Exchange {
    pub fn new() -> Self {
        Self { request: None, decoder: None, done: None, heard: Queue::uncapped() }
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
        exchange: github::plans::plan::Exchange,
        authorization: Option<Secret>,
        kept: &mut crate::kept::Store,
    ) -> Result<(), github::request::Refusal> {
        let encoded = github::request::encode(&exchange.request, &limits::documents(&env.limits))?;
        match (encoded.auth, &authorization) {
            (github::request::Auth::None, None)
            | (github::request::Auth::App | github::request::Auth::Installation, Some(_)) => {}
            (github::request::Auth::None, Some(_))
            | (github::request::Auth::App | github::request::Auth::Installation, None) => {
                unreachable!("component selects Authorization according to Encoded.auth");
            }
        }
        let mut headers = List::with_capacity(HEADERS);
        headers.push(field(b"Host", &c.host(forge))).expect("six GitHub header fields");
        headers.push(field(b"User-Agent", user_agent)).expect("six GitHub header fields");
        for (name, value) in github::request::fields(&encoded) {
            headers.push(field(name, value)).expect("six GitHub header fields");
        }
        if let Some(authorization) = authorization {
            headers.push(field(b"Authorization", authorization.expose())).expect("six GitHub header fields");
        }
        let method = match encoded.method {
            github::request::Method::Get => skein_http::Method::Get,
            github::request::Method::Post => skein_http::Method::Post,
            github::request::Method::Put => skein_http::Method::Put,
            github::request::Method::Patch => skein_http::Method::Patch,
            github::request::Method::Delete => skein_http::Method::Delete,
        };
        let body = Connection::body(&mut headers, encoded.body.as_deref());
        self.request = Some(Box::new(exchange));
        self.decoder = None;
        self.done = None;
        c.identity = Some(c.identity.unwrap_or(0));
        c.send(
            env,
            http::Call { method, target: encoded.target, headers: headers.into_boxed(), body, close: false },
            encoded.body,
            kept,
        );
        Ok(())
    }
    fn response(&mut self, c: &mut Connection, env: &Env<Limits>, response: http::Response) {
        let exchange = self.request.as_ref().expect("head belongs to a request");
        let request = &exchange.request;
        let mut l = limits::documents(&env.limits);
        if github::decode::kind(request) == Some(github::document::Kind::Minted) && response.status == 201 {
            l = limits::github_mint_documents(&env.limits);
        }
        if github::decode::kind(request) == Some(github::document::Kind::LogDelivery) {
            l.json.length = env.limits.delivery_bytes;
        }
        let mut head = github::header::Head::new();
        for field in &response.headers {
            github::header::field(&mut head, &field.name, &field.value, &l);
        }
        let mut directory = false;
        if let github::request::Request::Raw { .. } = request {
            directory = head.json;
        }
        self.heard.push(Heard::Head { status: response.status, head });
        let decoder = if response.status >= 300 {
            Some(github::decode::decoder(github::document::Kind::Error, &l))
        } else if directory {
            Some(github::decode::decoder(github::document::Kind::EntryRows, &l))
        } else {
            exchange.decoder(&l)
        };
        match decoder {
            Some(decoder) => {
                self.decoder = Some(Box::new(decoder));
                c.begin_json(env, l.json);
            }
            None => {
                if raw_request(request) {
                    let retention = exchange.retention.unwrap_or(Retention::Head { bytes: c.shared.answer_bytes });
                    Self::begin_raw(c, env, response.framing, retention);
                } else {
                    c.discard(env);
                }
            }
        }
    }
    fn begin_raw(c: &mut Connection, env: &Env<Limits>, framing: http::Framing, retention: Retention) {
        let (keep, cap, failed_step) = match retention {
            Retention::Head { bytes } | Retention::Whole { bytes } => (Keep::Head(bytes), bytes, false),
            Retention::Tail { bytes } => (Keep::Tail(bytes), bytes, false),
            Retention::FailedStep { bytes } => (Keep::Tail(bytes), bytes, true),
        };
        c.begin_raw(env, framing, keep, cap.min(c.shared.answer_bytes), failed_step);
    }
    fn token(&mut self, c: &mut Connection, env: &Env<Limits>, event: tokenizer::Event) {
        let Some(decoder) = self.decoder.as_mut() else { return };
        let (request, step) = decoder.step(event);
        match step {
            github::decode::Step::Demand => c.json_request(env, request),
            github::decode::Step::Row(row) => {
                self.heard.push(Heard::Row(row));
                c.set_json_next(request);
            }
            github::decode::Step::Done(github::document::Document::Minted(minted))
                if minted.token.len() > usize_of(env.limits.token_bytes) =>
            {
                c.malformed_json(decode::Malformed::Bound { field: b"token" });
            }
            github::decode::Step::Done(document) => self.done = Some(Box::new(Body::Document(document))),
            github::decode::Step::Malformed(decode::Malformed::Json(tokenizer::Error::Stream(_))) => {
                c.failed(super::Failure::Seen);
            }
            github::decode::Step::Malformed(error) => c.malformed_json(error),
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
                        Payload::Json => match self.done.take() {
                            Some(body) => *body,
                            None => Body::Malformed(decode::Malformed::Missing { field: b"" }),
                        },
                        Payload::Empty => match self.done.take() {
                            Some(body) => *body,
                            None => Body::Empty,
                        },
                        Payload::Raw { bytes, more } => Body::Raw { bytes, more },
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

fn raw_request(request: &github::request::Request) -> bool {
    use github::request::Request;
    match request {
        Request::Raw { .. } | Request::JobLog { .. } | Request::Followed { .. } => true,
        Request::LogPage { .. }
        | Request::LogDelivery { .. }
        | Request::Installations { .. }
        | Request::Installation { .. }
        | Request::Mint { .. }
        | Request::Reachable { .. }
        | Request::Enqueue { .. }
        | Request::Merge { .. }
        | Request::PullGraphql { .. }
        | Request::Viewer
        | Request::App
        | Request::HookConfig
        | Request::SetHook { .. }
        | Request::Uninstall { .. }
        | Request::Conversion { .. }
        | Request::BotUser { .. }
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
        | Request::Branch { .. }
        | Request::Branches { .. }
        | Request::PullFiles { .. }
        | Request::Compare { .. }
        | Request::Commits { .. }
        | Request::RulesForBranch { .. }
        | Request::Ruleset { .. }
        | Request::ClassicProtection { .. }
        | Request::Collaborators { .. }
        | Request::Permission { .. }
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
        | Request::DeleteBranch { .. }
        | Request::User { .. } => false,
    }
}

#[cfg(test)]
impl Exchange {
    pub(crate) fn test_fresh(&self) -> bool {
        self.request.is_none() && self.decoder.is_none() && self.done.is_none() && self.heard.is_empty()
    }
    pub(crate) fn test_hold_heard(&mut self) {
        self.heard.push(Heard::Done(Body::Empty));
    }
}

impl Exchange {
    pub(crate) fn test_github_handoff(&self) -> bool {
        match &self.request {
            Some(exchange) => {
                exchange.retention.is_some() || exchange.comparison == github::plans::plan::Comparison::MergeBase
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests;
