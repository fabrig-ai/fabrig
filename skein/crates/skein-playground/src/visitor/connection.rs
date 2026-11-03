//! One HTTP connection and its read demand (skein/docs/design/playground.md, 4).
use super::jar::Jar;
use skein_http::client as http;
use skein_io as io;
use skein_lib::{
    Env, Queue, Token,
    stream::{Down, Read, Up},
};
use skein_ui_web::fetch::{Fetch, Report};

#[derive(Debug, PartialEq, Eq)]
enum State {
    Connecting,
    Active,
    Closing,
}

#[derive(Debug)]
pub(super) struct Connection {
    socket: Option<Token>,
    state: State,
    http: http::Client,
    events: Queue<http::Event>,
    calls: Queue<http::Request>,
    below: Queue<Down>,
    path: Box<[u8]>,
    body: Box<[u8]>,
    sent: usize,
    status: Option<u16>,
    stream: Option<u32>,
    reading: bool,
    keep: Vec<String>,
    bound: u32,
    headers: Vec<(String, String)>,
}
impl Connection {
    pub(super) fn new(
        limits: &http::Limits,
        call: http::Call,
        body: Box<[u8]>,
        stream: Option<u32>,
        keep: Vec<String>,
        bound: u32,
    ) -> Self {
        let mut calls = Queue::uncapped();
        let path = call.target.clone();
        calls.push(http::Request::Call(call));
        if body.is_empty() {
            calls.push(http::Request::Upload(Down::Finish));
        } else {
            calls.push(http::Request::Upload(Down::Demand {
                read: Read::Nothing,
                room: limits.send.min(u32::try_from(body.len()).unwrap_or(u32::MAX)),
            }));
        }
        Self {
            socket: None,
            state: State::Connecting,
            http: http::Client::new(limits),
            events: Queue::uncapped(),
            calls,
            below: Queue::uncapped(),
            path,
            body,
            sent: 0,
            status: None,
            stream,
            reading: false,
            keep,
            bound,
            headers: Vec::new(),
        }
    }
    pub(super) fn connecting(&mut self, socket: Token, requests: &mut Queue<io::Request>) {
        self.socket = Some(socket);
        if self.state == State::Closing {
            requests.push(io::Request::Abort { entity: socket });
        }
    }
    pub(super) fn connected(&mut self) {
        if self.state == State::Connecting {
            self.state = State::Active;
        }
    }
    pub(super) fn close(&mut self, requests: &mut Queue<io::Request>) {
        if self.state != State::Closing {
            self.state = State::Closing;
            self.calls.clear();
            self.events.clear();
            self.below.clear();
            self.body = Box::from([]);
            // An abort gives abandons and stream replacements no peer-dependent wait.
            if let Some(entity) = self.socket {
                requests.push(io::Request::Abort { entity });
            }
        }
    }
    pub(super) fn up(&mut self, up: Up, env: &Env<http::Limits>) {
        if self.state == State::Active {
            http::up(&mut self.http, env, up, &mut self.events, &mut self.below);
        }
    }
    pub(super) fn read_stream(&mut self) {
        if self.state == State::Active && self.status.is_some() && !self.reading {
            self.demand(self.stream.expect("stream connection"));
        }
    }
    fn demand(&mut self, max: u32) {
        self.reading = true;
        self.calls.push(http::Request::Body(Down::Demand { read: Read::Arrived { max }, room: 0 }));
    }
    fn terminal(
        &mut self,
        ticket: u64,
        fetch: &mut Fetch,
        reports: &mut Vec<Report>,
        requests: &mut Queue<io::Request>,
        done: bool,
    ) {
        let report = if self.stream.is_some() {
            fetch.end_stream(ticket)
        } else if done {
            fetch.finish(ticket, self.status.expect("answer before Done"), &self.headers)
        } else {
            fetch.fail(ticket)
        };
        reports.extend(report);
        self.close(requests);
    }
    fn keep_headers(&mut self, headers: &[skein_http::Header]) -> bool {
        let bound = usize::try_from(self.bound).expect("bound fits");
        for name in &self.keep {
            let mut value = String::new();
            let mut present = false;
            for header in headers.iter().filter(|header| header.is(name.as_bytes())) {
                let part = String::from_utf8_lossy(&header.value);
                let separator = if present { ", " } else { "" };
                if value.len().saturating_add(separator.len()).saturating_add(part.len()) > bound {
                    return false;
                }
                value.push_str(separator);
                value.push_str(&part);
                present = true;
            }
            if present {
                self.headers.push((name.to_ascii_lowercase(), value));
            }
        }
        true
    }
    pub(super) fn progress(
        &mut self,
        ticket: u64,
        env: &Env<http::Limits>,
        fetch: &mut Fetch,
        jar: &mut Jar,
        reports: &mut Vec<Report>,
        requests: &mut Queue<io::Request>,
    ) {
        if self.state != State::Active {
            return;
        }
        for _ in 0..self.events.len() {
            let Some(event) = self.events.pop() else { break };
            match event {
                http::Event::Response(response) => {
                    for header in &response.headers {
                        if header.is(b"set-cookie") {
                            jar.set(&header.value, &self.path, env.wall);
                        }
                    }
                    if !self.keep_headers(&response.headers) {
                        self.terminal(ticket, fetch, reports, requests, false);
                        return;
                    }
                    self.status = Some(response.status);
                    if let Some(bound) = self.stream {
                        if response.status == 200 {
                            reports.push(Report::StreamOpened);
                        } else {
                            reports.extend(fetch.refuse_stream(ticket, response.status));
                        }
                        self.demand(bound.min(env.limits.read));
                    } else {
                        self.demand(fetch.remaining(ticket).expect("live ticket").max(1).min(env.limits.read));
                    }
                }
                http::Event::Body(Up::Bytes(bytes)) => {
                    self.reading = false;
                    if self.stream.is_some() {
                        reports.extend(fetch.stream_piece(ticket, bytes));
                    } else {
                        let report = fetch.piece(ticket, &bytes);
                        if report.is_some() {
                            reports.extend(report);
                            self.close(requests);
                        } else {
                            self.demand(fetch.remaining(ticket).expect("live ticket").max(1).min(env.limits.read));
                        }
                    }
                }
                http::Event::Done(_) => self.terminal(ticket, fetch, reports, requests, true),
                http::Event::Failed(_) | http::Event::Body(Up::Failed(_)) => {
                    self.terminal(ticket, fetch, reports, requests, false);
                }
                http::Event::Upload(Up::Room) => {
                    let end = self
                        .sent
                        .saturating_add(usize::try_from(env.limits.send).expect("send fits"))
                        .min(self.body.len());
                    self.calls.push(http::Request::Upload(Down::Send(
                        self.body.get(self.sent..end).expect("upload within body").into(),
                    )));
                    self.sent = end;
                    if end == self.body.len() {
                        self.calls.push(http::Request::Upload(Down::Finish));
                        self.body = Box::from([]);
                    } else {
                        self.calls.push(http::Request::Upload(Down::Demand {
                            read: Read::Nothing,
                            room: env
                                .limits
                                .send
                                .min(u32::try_from(self.body.len().saturating_sub(end)).unwrap_or(u32::MAX)),
                        }));
                    }
                }
                // A final response may stop the upload; the answer still stands (http.md, 3.4).
                http::Event::Upload(Up::Failed(_) | Up::Bytes(_) | Up::End)
                | http::Event::Body(Up::Room | Up::End)
                | http::Event::Closed => {}
            }
        }
        if self.state != State::Active {
            return;
        }
        // Observe a refused call before issuing its upload or a subsequent demand.
        if let Some(call) = self.calls.pop() {
            http::down(&mut self.http, env, call, &mut self.events, &mut self.below);
        }
        let socket = self.socket.expect("active connection");
        for _ in 0..self.below.len() {
            if let Some(down) = self.below.pop() {
                requests.push(io::Request::Stream { stream: socket, down });
            }
        }
    }
    pub(super) fn work_pending(&self) -> bool {
        self.state == State::Active && (!self.events.is_empty() || !self.calls.is_empty() || !self.below.is_empty())
    }
}
