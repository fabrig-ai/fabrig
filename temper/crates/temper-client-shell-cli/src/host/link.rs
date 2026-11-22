//! Asked links reach only a durable private file (client/ctl/command-line.md, 1.1).
use super::{Class, OneShot, Parsed};
use crate::signing;
use jig_client::{AskedLink, NotAsked};
use skein_lib::{Duration, Secret, Time, Wall};
use temper_client_tools::jig::call::{Value, Values};

pub(super) struct LinkFile {
    pub(super) path: Box<[u8]>,
    pub(super) checked: bool,
    pub(super) retry: Option<Time>,
    backoff: Duration,
    last: Option<NotAsked>,
    json: bool,
    printed: Option<Box<[u8]>>,
}
impl LinkFile {
    pub(super) fn for_call(parsed: &Parsed) -> Option<Self> {
        let Parsed::Call { call, json } = parsed else { return None };
        if crate::arguments::class(call) != Class::Link {
            return None;
        }
        let Some(Some(Values::One(Value::File(path)))) = call.values.first() else { return None };
        Some(Self {
            path: path.clone(),
            checked: false,
            retry: None,
            backoff: signing::FIRST,
            last: None,
            json: *json,
            printed: None,
        })
    }
}
impl OneShot {
    pub(super) fn link_retry(&mut self) {
        let Some(file) = &mut self.link_file else { return };
        if file.retry.is_some_and(|at| self.env.now >= at) && self.until.is_some_and(|until| self.env.now < until) {
            file.retry = None;
            self.root_event(jig_client::Event::AskLink);
        }
    }
    pub(super) fn link_answer(&mut self, answer: AskedLink) {
        if self.status.is_some() || self.class != Class::Link {
            return;
        }
        match answer {
            AskedLink::Asked { secret, expires } => {
                let file = self.link_file.as_mut().expect("link output");
                file.retry = None;
                file.last = None;
                let mapping = jig_client_protocol_http::Mapping::with_page(
                    b"/jig", temper_client_domain_forge::sign_in::LINK_PAGE,
                ).expect("mapping");
                let host = self.directory.address.as_deref().expect("signed-in address")
                    .strip_prefix(b"http://").expect("loopback HTTP address");
                let mut bytes = jig_client_protocol_http::link_url(&mapping, host, &secret).into_vec();
                bytes.push(b'\n');
                file.printed = Some(written(&file.path, expires, file.json));
                self.directory.store_service(&Secret::new(bytes.into_boxed_slice()));
                // Once answered, only the disk's own deadlines govern the write.
                self.until = None;
            }
            AskedLink::NotAsked(NotAsked::Refused) => self.fail(b"the engine refused a sign-in link: it gives one to a person's sign-in alone, where its printed link is enabled"),
            AskedLink::NotAsked(reason @ (NotAsked::NotReady | NotAsked::Busy | NotAsked::Lost)) => {
                let file = self.link_file.as_mut().expect("link output");
                if reason != NotAsked::Lost {
                    file.last = Some(reason);
                }
                file.retry = Some(signing::again(self.env.now, self.until.expect("link deadline"), &mut file.backoff));
            }
            AskedLink::SignedOut => self.fail(b"signed out"),
        }
    }
    pub(super) fn link_deadline(&self) -> Option<&'static [u8]> {
        match self.link_file.as_ref().and_then(|file| file.last) {
            Some(NotAsked::Busy) => Some(b"temper is busy"),
            Some(NotAsked::NotReady) => Some(
                b"temper is not ready: it is starting, or its core is not answering, and temper serve's log says why",
            ),
            Some(NotAsked::Refused | NotAsked::Lost) | None => None,
        }
    }
    pub(super) fn link_recorded(&mut self, result: Result<(), Box<[u8]>>) {
        let file = self.link_file.as_mut().expect("link output");
        match result {
            Ok(()) => {
                self.output.push(false, file.printed.take().expect("durable link"));
                self.finish(0, false);
            }
            Err(error) => {
                file.printed = None;
                let mut line = file.path.to_vec();
                line.extend_from_slice(b" could not be written: ");
                line.extend_from_slice(&error);
                self.fail(&line);
            }
        }
    }
}
fn written(path: &[u8], expires: u64, json: bool) -> Box<[u8]> {
    let expires = temper_client_domain_forge::sign_in::utc_second(Wall::from_nanos(expires));
    let mut line = if json {
        let limits = skein_json::writer::Limits { depth: 1, length: u32::MAX };
        let mut measured = skein_json::writer::Encoder::measure(&limits);
        record(path, &expires, &mut measured);
        let mut out = skein_json::writer::Encoder::write(measured.measured().expect("link record length"), &limits);
        record(path, &expires, &mut out);
        out.finish().into_vec()
    } else {
        let mut line = b"sign-in link written to ".to_vec();
        line.extend_from_slice(path);
        line.extend_from_slice(b"; it signs in once, until ");
        line.extend_from_slice(&expires);
        line
    };
    line.push(b'\n');
    line.into_boxed_slice()
}
fn record(path: &[u8], expires: &[u8], out: &mut skein_json::writer::Encoder) {
    out.object_start();
    out.key(b"file");
    out.string(path);
    out.key(b"expires");
    out.string(expires);
    out.object_end();
}
