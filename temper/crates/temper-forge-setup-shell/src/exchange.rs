//! One plaintext exchange, with streamed document decoding (protocol/setup.md, 5).
use crate::reconcile::Settings;
use skein_forge::forgejo::{
    client,
    decode::Step,
    document::{Document, Row},
    request::{Method, Request},
};
use skein_forge_setup::reconcile::Reply;
use skein_http::client as http;
use skein_json::tokenizer;
use skein_lib::{
    Duration, Env, List, Queue, Secret, Time,
    stream::{Down, Read, Up},
};
/// The secret Authorization value, including its token scheme.
#[must_use]
pub fn authorization(token: &Secret) -> Secret {
    let mut bytes = b"token ".to_vec();
    bytes.extend_from_slice(token.expose());
    Secret::new(bytes.into_boxed_slice())
}
/// The next tokenizer demand or the decoded reply.
#[expect(clippy::large_enum_variant, reason = "documents move whole")]
pub enum Next {
    Demand(tokenizer::Request),
    Reply(Reply),
}
/// Rows retained in their received order for one page.
#[derive(Default)]
pub struct Gather {
    pub rows: Vec<Row>,
}
impl Gather {
    /// Keeps rows until the listing ends; malformed input ends unavailable.
    pub fn take(&mut self, step: Step) -> Next {
        match step {
            Step::Demand(request) => Next::Demand(request),
            Step::Row(row, request) => {
                self.rows.push(row);
                Next::Demand(request)
            }
            Step::Done(Document::Listed { .. }) => {
                Next::Reply(Reply::Rows(std::mem::take(&mut self.rows).into_boxed_slice()))
            }
            Step::Done(document) => Next::Reply(Reply::Document(document)),
            Step::Malformed(_) => Next::Reply(Reply::Down),
        }
    }
}
/// Doubles the retry wait, capped by both the configured maximum and time left.
#[must_use]
pub fn pause(
    backoff: Duration,
    backoff_max: Duration,
    deadline: Duration,
    attempt: u32,
    began: Time,
    now: Time,
) -> Option<Duration> {
    let doubled = backoff.saturating_mul(1_u64.checked_shl(attempt).unwrap_or(u64::MAX));
    let left = deadline.as_nanos().checked_sub(now.saturating_since(began).as_nanos()).filter(|left| *left > 0)?;
    Some(Duration::from_nanos(doubled.as_nanos().min(backoff_max.as_nanos()).min(left)))
}
pub(crate) struct Exchange {
    http: http::Client,
    json: tokenizer::Tokenizer,
    decoder: Option<client::Decoder>,
    gathered: Gather,
    upload: Option<Box<[u8]>>,
    uploaded: usize,
    above: Queue<http::Event>,
    below: Queue<Down>,
    json_up: Queue<tokenizer::Event>,
    json_down: Queue<Down>,
    decoded: Option<Reply>,
    pub reply: Option<Reply>,
}
impl Exchange {
    pub fn new(
        settings: &Settings,
        request: &Request,
        token: &Secret,
        now: Time,
        wall: skein_lib::Wall,
    ) -> Result<Self, client::Refusal> {
        let encoded = client::encode(request, &settings.documents)?;
        let mut headers = List::with_capacity(4);
        let auth = authorization(token);
        for (name, value) in [
            (b"Host".as_slice(), settings.host.as_ref()),
            (b"Authorization", auth.expose()),
            (b"User-Agent", settings.user_agent.as_bytes()),
        ] {
            headers.push(skein_http::Header { name: name.into(), value: value.into() }).expect("four headers");
        }
        let body = if let Some(body) = &encoded.body {
            headers
                .push(skein_http::Header {
                    name: b"Content-Type".as_slice().into(),
                    value: b"application/json".as_slice().into(),
                })
                .expect("four headers");
            http::Body::Length(u64::try_from(body.len()).expect("bounded body"))
        } else {
            http::Body::None
        };
        let method = match encoded.method {
            Method::Get => skein_http::Method::Get,
            Method::Post => skein_http::Method::Post,
            Method::Put => skein_http::Method::Put,
            Method::Patch => skein_http::Method::Patch,
            Method::Delete => skein_http::Method::Delete,
        };
        let mut exchange = Self {
            http: http::Client::new(&settings.http),
            json: tokenizer::Tokenizer::new(&settings.documents.json),
            decoder: client::decoder(request, &settings.documents),
            gathered: Gather::default(),
            upload: encoded.body,
            uploaded: 0,
            above: Queue::with_capacity(8),
            below: Queue::with_capacity(8),
            json_up: Queue::with_capacity(8),
            json_down: Queue::with_capacity(8),
            decoded: None,
            reply: None,
        };
        let env = Env { now, wall, limits: settings.http };
        exchange.request(
            &env,
            http::Request::Call(http::Call {
                method,
                target: encoded.target,
                headers: headers.into_boxed(),
                body,
                close: true,
            }),
        );
        if exchange.upload.is_some() {
            exchange
                .request(&env, http::Request::Upload(Down::Demand { read: Read::Nothing, room: settings.http.send }));
        }
        Ok(exchange)
    }
    fn request(&mut self, env: &Env<http::Limits>, request: http::Request) {
        http::down(&mut self.http, env, request, &mut self.above, &mut self.below);
    }
    pub fn stream(&mut self, settings: &Settings, now: Time, wall: skein_lib::Wall, up: Up) {
        http::up(&mut self.http, &Env { now, wall, limits: settings.http }, up, &mut self.above, &mut self.below);
    }
    pub fn pump(&mut self, settings: &Settings, now: Time, wall: skein_lib::Wall) -> Option<Down> {
        let env = Env { now, wall, limits: settings.http };
        let json_env = Env { now, wall, limits: settings.documents.json };
        loop {
            if self.reply.is_some() {
                return None;
            }
            if let Some(down) = self.below.pop() {
                return Some(down);
            }
            if let Some(down) = self.json_down.pop() {
                self.request(&env, http::Request::Body(down));
                continue;
            }
            if let Some(event) = self.json_up.pop() {
                let step =
                    client::step(self.decoder.as_mut().expect("decoding a document"), &settings.documents, &event);
                match self.gathered.take(step) {
                    Next::Demand(request) => {
                        tokenizer::down(&mut self.json, &json_env, request, &mut self.json_up, &mut self.json_down);
                    }
                    Next::Reply(reply) => {
                        self.decoded = Some(reply);
                        self.request(&env, http::Request::Discard);
                    }
                }
                continue;
            }
            let event = self.above.pop()?;
            match event {
                http::Event::Response(response) => {
                    if !(200..300).contains(&response.status) {
                        self.reply = Some(Reply::Status(response.status));
                    } else if self.decoder.is_some() {
                        tokenizer::down(
                            &mut self.json,
                            &json_env,
                            tokenizer::Request::Next,
                            &mut self.json_up,
                            &mut self.json_down,
                        );
                    } else {
                        self.decoded = Some(Reply::Empty);
                        self.request(&env, http::Request::Discard);
                    }
                }
                http::Event::Body(up) => {
                    if self.decoder.is_some() && self.decoded.is_none() {
                        tokenizer::up(&mut self.json, &json_env, up, &mut self.json_up, &mut self.json_down);
                    }
                }
                http::Event::Upload(Up::Room) => {
                    if let Some(body) = &self.upload {
                        let end = body
                            .len()
                            .min(self.uploaded.saturating_add(usize::try_from(settings.http.send).expect("send fits")));
                        let piece = body[self.uploaded..end].into();
                        self.uploaded = end;
                        let finished = end == body.len();
                        self.request(&env, http::Request::Upload(Down::Send(piece)));
                        if finished {
                            self.upload = None;
                            self.request(&env, http::Request::Upload(Down::Finish));
                        } else {
                            self.request(
                                &env,
                                http::Request::Upload(Down::Demand { read: Read::Nothing, room: settings.http.send }),
                            );
                        }
                    }
                }
                http::Event::Done(_) => self.reply = Some(self.decoded.take().unwrap_or(Reply::Down)),
                http::Event::Failed(_) => self.reply = Some(Reply::Down),
                http::Event::Upload(Up::Failed(_) | Up::End | Up::Bytes(_)) | http::Event::Closed => {}
            }
        }
    }
}
