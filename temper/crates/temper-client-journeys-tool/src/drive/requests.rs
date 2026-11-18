//! The last document's requests, waves and quiet spell (client/drive.md, 3.2).
use super::{Action, Machine, Phase, QUIET, resource};
use skein_lib::Time;

pub(super) struct Log {
    load: Option<Box<[u8]>>,
    origin: Option<Box<[u8]>>,
    requests: Vec<Request>,
    activity: Time,
}
struct Request {
    id: Box<[u8]>,
    method: Box<[u8]>,
    path: Box<[u8]>,
    wave: Option<u64>,
    answered: bool,
}
impl Request {
    fn line(&self) -> String {
        let mut line = format!("{} {}", String::from_utf8_lossy(&self.method), String::from_utf8_lossy(&self.path));
        if let Some(wave) = self.wave {
            use std::fmt::Write;
            write!(line, ", wave {wave}").expect("string write");
        }
        line
    }
}
impl Log {
    pub fn new(now: Time) -> Self {
        Self { load: None, origin: None, requests: vec![], activity: now }
    }
    pub fn sent(&mut self, id: Box<[u8]>, load: &[u8], document: bool, method: Box<[u8]>, url: &[u8], now: Time) {
        let address = address(url);
        if document {
            self.load = Some(load.into());
            self.origin = address.map(|(origin, _)| origin.into());
            self.requests.clear();
        }
        let Some((origin, path)) = address else { return };
        if self.load.as_deref() != Some(load) || self.origin.as_deref() != Some(origin) {
            return;
        }
        let listed = !document
            && !(method.as_ref() == b"GET"
                && (path.starts_with(b"/bundle/")
                    || [b"/robots.txt".as_slice(), b"/sw.js", b"/manifest.webmanifest"].contains(&path)));
        let wave = listed.then(|| {
            self.requests
                .iter()
                .filter(|request| request.answered)
                .filter_map(|request| request.wave)
                .max()
                .unwrap_or(0)
                .saturating_add(1)
        });
        self.requests.push(Request { id, method, path: path.into(), wave, answered: false });
        self.activity = now;
    }
    pub fn answered(&mut self, id: &[u8], now: Time) {
        if let Some(request) = self.requests.iter_mut().find(|request| request.id.as_ref() == id && !request.answered) {
            request.answered = true;
            self.activity = now;
        }
    }
    fn quiet_at(&self) -> Option<Time> {
        self.requests.iter().all(|request| request.answered).then(|| self.activity.saturating_add(QUIET))
    }
    fn shown(&self) -> Vec<String> {
        let mut shown = vec![];
        let mut waves = 0;
        for request in &self.requests {
            if let Some(wave) = request.wave {
                waves = waves.max(wave);
                shown.push(request.line());
            }
        }
        shown.push(format!("{} requests in {waves} waves", shown.len()));
        shown
    }
    fn why(&self) -> String {
        let pending = self.requests.iter().filter(|request| !request.answered).map(Request::line).collect::<Vec<_>>();
        if pending.is_empty() {
            "web requests: the page kept sending".into()
        } else {
            format!("web requests: unanswered: {}", pending.join("; "))
        }
    }
}
impl Machine {
    pub(super) fn requests_check(&mut self, now: Time, out: &mut Vec<Action>) {
        if self.phase != Phase::Requests {
            return;
        }
        let until = self.until.expect("requests line bound");
        let quiet = self.requests.quiet_at();
        if quiet.is_some_and(|at| now >= at && at <= until) {
            self.shown = self.requests.shown();
            self.answer(None, out);
        } else if now >= until {
            self.shown = self.requests.shown();
            self.answer(Some(self.requests.why()), out);
        } else {
            self.poll = quiet;
        }
    }
}

// Chromium supplies canonical URLs. Compare the whole authority, not a URL prefix.
fn address(url: &[u8]) -> Option<(&[u8], &[u8])> {
    let rest = url.strip_prefix(b"http://").or_else(|| url.strip_prefix(b"https://"))?;
    let authority = rest.iter().position(|byte| b"/?#".contains(byte)).unwrap_or(rest.len());
    if authority == 0 {
        return None;
    }
    let end = url.len().checked_sub(rest.len())?.checked_add(authority)?;
    Some((url.get(..end)?, resource(url.get(end..)?)))
}
