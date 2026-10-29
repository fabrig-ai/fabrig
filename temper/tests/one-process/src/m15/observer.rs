//! Fake-issued values remain in this observer; failures name only their surface (temper/docs/design/domain/testing.md, 2.5).
use crate::fixtures;
use skein_fake_peers::forge::Peer;
use skein_io::kernel::{Complete, Done, Fd, Op};
use skein_lib::{
    Secret, Writer,
    base64::{self, Alphabet},
};
use std::{cell::RefCell, collections::BTreeMap};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Role {
    Service,
    Ctl,
    Caller,
    Client,
    Agent,
    Git,
    Shell,
    Forge,
    Provider,
}
#[derive(Clone, Copy)]
enum Kind {
    Key,
    Token,
}
struct Needle {
    kind: Kind,
    value: Secret,
}
struct Artifact {
    surface: &'static str,
    path: Vec<u8>,
    bytes: Vec<u8>,
}
struct Stream {
    role: Role,
    bytes: Vec<u8>,
    parsed: usize,
}
#[derive(Default)]
struct Observer {
    needles: Vec<Needle>,
    tokens: Vec<Secret>,
    names: BTreeMap<(Role, i32), Vec<u8>>,
    files: BTreeMap<Vec<u8>, Vec<u8>>,
    wires: BTreeMap<(Role, i32), usize>,
    streams: Vec<Stream>,
    artifacts: Vec<Artifact>,
    contexts: usize,
    pages: usize,
    last_page: Vec<u8>,
    checkpoints: usize,
}
#[derive(Default, Debug)]
pub struct Counts {
    pub tokens: usize,
    pub checkpoints: usize,
    pub contexts: usize,
    pub pages: usize,
    pub wire_replies: usize,
    pub files: usize,
    pub logs: usize,
    pub transcripts: usize,
    pub client_files: usize,
    pub worker_files: usize,
}
thread_local! { static OBSERVER: RefCell<Option<Observer>> = const { RefCell::new(None) }; }
pub(crate) fn enabled() -> bool {
    OBSERVER.with(|observer| observer.borrow().is_some())
}
pub(crate) fn start() {
    OBSERVER.with(|observer| *observer.borrow_mut() = Some(Observer::default()));
}
fn encoded(bytes: &[u8]) -> Box<[u8]> {
    let length = base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("fixture secret length"))
        .expect("encoded fixture length");
    let mut writer = Writer::new(length as usize);
    base64::encode(Alphabet::Standard, bytes, &mut writer).expect("encoded fixture");
    writer.finish()
}
impl Observer {
    fn needle(&mut self, kind: Kind, bytes: &[u8]) {
        assert!(!bytes.is_empty(), "issued secret is nonempty");
        for value in [bytes.into(), format!("{bytes:?}").into_bytes().into_boxed_slice(), encoded(bytes)] {
            self.needles.push(Needle { kind, value: Secret::new(value) });
        }
    }
    fn token(&mut self, bytes: &[u8]) {
        if self.tokens.iter().any(|token| token.expose() == bytes) {
            return;
        }
        self.needle(Kind::Token, bytes);
        self.needles
            .push(Needle { kind: Kind::Token, value: Secret::new(encoded(&[b"x-access-token:", bytes].concat())) });
        self.tokens.push(Secret::new(bytes.into()));
    }
    fn key(&mut self, pem: &[u8]) {
        self.needle(Kind::Key, pem);
        let body: Vec<_> = pem
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.starts_with(b"-----"))
            .flatten()
            .copied()
            .filter(|byte| !byte.is_ascii_whitespace())
            .collect();
        let der = base64::decode(Alphabet::Standard, &body, 8192).expect("fake PEM body");
        self.needle(Kind::Key, &der);
        self.needle(Kind::Key, &body);
        for line in pem.split(|byte| *byte == b'\n').filter(|line| !line.starts_with(b"-----") && line.len() >= 32) {
            self.needle(Kind::Key, line);
        }
    }
    fn leak(&self, bytes: &[u8]) -> Option<&'static str> {
        self.needles
            .iter()
            .find(|needle| bytes.windows(needle.value.len()).any(|part| part == needle.value.expose()))
            .map(|needle| match needle.kind {
                Kind::Key => "App key",
                Kind::Token => "installation token",
            })
    }
    fn check(&self, surface: &str, path: &[u8], bytes: &[u8]) {
        assert!(self.leak(path).is_none(), "issued secret in {surface} filename");
        assert!(
            self.leak(bytes).is_none(),
            "{} in {surface} {}",
            self.leak(bytes).unwrap_or("issued secret"),
            String::from_utf8_lossy(path)
        );
    }
    fn save(&mut self, surface: &'static str, path: &[u8], bytes: &[u8]) {
        self.artifacts.push(Artifact { surface, path: path.to_vec(), bytes: bytes.to_vec() });
    }
    fn wire(&mut self, role: Role, fd: Fd, bytes: &[u8]) {
        let index = *self.wires.entry((role, fd.raw())).or_insert_with(|| {
            self.streams.push(Stream { role, bytes: Vec::new(), parsed: 0 });
            self.streams.len() - 1
        });
        let stream = &mut self.streams[index];
        stream.bytes.extend_from_slice(bytes);
        if role != Role::Forge {
            return;
        }
        let mut issued = Vec::new();
        while let Some((used, body)) = response(&stream.bytes[stream.parsed..]) {
            if let Ok(value) = serde_json::from_slice::<serde_json::Value>(body)
                && let Some(token) = value["token"].as_str()
            {
                issued.push(Secret::new(token.as_bytes().into()));
            }
            stream.parsed += used;
        }
        for token in issued {
            self.token(token.expose());
        }
    }
    fn complete(&mut self, role: Role, complete: &Complete) {
        match (&complete.kind, &complete.result) {
            (Op::Connect { .. }, Ok(Done::Nothing)) => self.wires.retain(|(owner, _), _| *owner != role),
            (Op::Accept { .. }, Ok(Done::Accepted { fd, .. })) => {
                self.wires.remove(&(role, fd.raw()));
            }
            (Op::Open { root, path, .. }, Ok(Done::Fd(fd))) => {
                if let Some(parent) = self.names.get(&(role, root.raw())) {
                    let path =
                        if path.as_ref() == b"." { parent.clone() } else { [parent.as_slice(), b"/", path].concat() };
                    self.names.insert((role, fd.raw()), path);
                }
            }
            (Op::Write { fd, bytes, from, at }, Ok(Done::Count(count))) => {
                if let Some(path) = self.names.get(&(role, fd.raw())).cloned()
                    && !secret_path(&path)
                {
                    let file = self.files.entry(path.clone()).or_default();
                    let at = usize::try_from(*at).expect("scenario file offset");
                    file.resize(file.len().max(at + *count as usize), 0);
                    file[at..at + *count as usize].copy_from_slice(&bytes[*from as usize..(*from + *count) as usize]);
                    let version = file.clone();
                    self.save("persisted write", &path, &version);
                }
            }
            (Op::Send { fd, bytes, from }, Ok(Done::Count(count))) if role == Role::Forge => {
                self.wire(role, *fd, &bytes[*from as usize..(*from + *count) as usize])
            }
            (Op::Recv { fd, buf }, Ok(Done::Count(count)))
                if matches!(role, Role::Ctl | Role::Client | Role::Caller) =>
            {
                self.wire(role, *fd, &buf[..*count as usize])
            }
            (Op::PipeWrite { fd, bytes, from }, Ok(Done::Count(count))) => {
                if let Some(path) = self.names.get(&(role, fd.raw())).cloned()
                    && path.starts_with(b"stderr/")
                {
                    let log = self.files.entry(path.clone()).or_default();
                    log.extend_from_slice(&bytes[*from as usize..(*from + *count) as usize]);
                    let version = log.clone();
                    self.save("process log", &path, &version);
                }
            }
            (Op::Close { fd }, _) => {
                self.names.remove(&(role, fd.raw()));
                self.wires.remove(&(role, fd.raw()));
            }
            _ => {}
        }
    }
    fn scan(&mut self) {
        assert!(!self.needles.is_empty(), "the fake issued a key");
        for artifact in &self.artifacts {
            self.check(artifact.surface, &artifact.path, &artifact.bytes);
        }
        for stream in &self.streams {
            if stream.role != Role::Forge {
                self.check("client wire", b"", &stream.bytes);
            }
        }
        self.checkpoints += 1;
    }
}
fn response(bytes: &[u8]) -> Option<(usize, &[u8])> {
    let end = bytes.windows(4).position(|bytes| bytes == b"\r\n\r\n")?;
    let head = std::str::from_utf8(&bytes[..end]).ok()?;
    let length: usize = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))?
        .1
        .trim()
        .parse()
        .ok()?;
    let used = end.checked_add(4)?.checked_add(length)?;
    (bytes.len() >= used).then(|| (used, &bytes[end + 4..used]))
}
fn secret_path(path: &[u8]) -> bool {
    path.split(|byte| *byte == b'/').any(|part| part == b"secrets") || path.starts_with(b"input/")
}
pub(crate) fn root(role: Role, fd: Fd, path: &[u8]) {
    OBSERVER.with(|cell| {
        if let Some(observer) = &mut *cell.borrow_mut() {
            observer.names.insert((role, fd.raw()), path.to_vec());
        }
    });
}
pub(crate) fn process(role: Role, inherited: &skein_world::Inherited, base: &[u8]) {
    OBSERVER.with(|cell| {
        if let Some(observer) = &mut *cell.borrow_mut() {
            observer.names.retain(|(owner, _), _| *owner != role);
            observer.wires.retain(|(owner, _), _| *owner != role);
            for (name, fd) in &inherited.roots {
                let path = if name.as_ref() == b"file_parent" || name.starts_with(b"text-") {
                    b"input".as_slice()
                } else {
                    base
                };
                observer.names.insert((role, fd.raw()), path.to_vec());
            }
            if let Some((_, fd)) = inherited.pipes.iter().find(|(number, _)| *number == 2) {
                observer.names.insert((role, fd.raw()), format!("stderr/{}", observer.artifacts.len()).into_bytes());
            }
        }
    });
}
pub(crate) fn complete(role: Role, complete: &Complete) {
    OBSERVER.with(|cell| {
        if let Some(observer) = &mut *cell.borrow_mut() {
            // Retain every public response fragment, including responses on closed/reused descriptors.
            if matches!(role, Role::Ctl | Role::Client | Role::Caller)
                && let (Op::Recv { buf, .. }, Ok(Done::Count(count))) = (&complete.kind, &complete.result)
            {
                observer.save("client wire fragment", b"", &buf[..*count as usize]);
            }
            observer.complete(role, complete);
        }
    });
}
pub(crate) fn artifact(surface: &'static str, bytes: &[u8]) {
    OBSERVER.with(|cell| {
        if let Some(observer) = &mut *cell.borrow_mut() {
            observer.save(surface, b"", bytes);
        }
    });
}
pub(crate) fn page(bytes: &[u8]) {
    OBSERVER.with(|cell| {
        if let Some(observer) = &mut *cell.borrow_mut()
            && observer.last_page != bytes
        {
            observer.pages += 1;
            observer.last_page = bytes.to_vec();
            observer.save("rendered page", b"", bytes);
        }
    });
}
pub(crate) fn context(peer: &skein_fake_peers::llm::Peer) {
    OBSERVER.with(|cell| {
        if let Some(observer) = &mut *cell.borrow_mut() {
            let queries: Vec<_> = peer
                .observations()
                .iter()
                .filter_map(|seen| {
                    if let skein_fake_peers::llm::Observation::Query { query, .. } = seen { Some(query) } else { None }
                })
                .collect();
            for query in &queries[observer.contexts..] {
                observer.save("agent prompt", b"", &query.system);
                for part in query.messages.iter().flat_map(|message| message.parts.iter()) {
                    use skein_fake_llm_domain::api::Part;
                    match part {
                        Part::Opaque { bytes } => observer.save("agent context", b"", bytes),
                        Part::Text { text } => observer.save("agent context", b"", text),
                        Part::ToolCall { id, name, arguments } => {
                            observer.save("agent tool id", b"", id);
                            observer.save("agent tool name", b"", name);
                            observer.save("agent tool arguments", b"", arguments);
                        }
                        Part::ToolOutput { id, output, .. } => {
                            observer.save("agent tool id", b"", id);
                            observer.save("agent tool answer", b"", output);
                        }
                    }
                }
            }
            observer.contexts = queries.len();
        }
    });
}
pub(crate) fn checkpoint() {
    OBSERVER.with(|cell| cell.borrow_mut().as_mut().expect("active observer").scan());
}
pub(crate) fn tokens() -> usize {
    OBSERVER.with(|cell| cell.borrow().as_ref().map_or(0, |observer| observer.tokens.len()))
}
pub(crate) fn input(peer: &mut Peer) -> (u64, Vec<skein_fake_machine::Item>) {
    use skein_forge::github::{pin, request::Method, setup};
    use skein_lib::{ReplyTo, Token, Wall};
    use std::fmt::Write as _;
    let origin = format!("http://{}", fixtures::CLIENTS);
    let manifest = setup::manifest(
        &setup::Values {
            name: b"temper-person",
            url: origin.as_bytes(),
            hook_url: b"https://temper-test.invalid/",
            redirect_url: format!("{origin}/forges/1/github/redirect").as_bytes(),
            setup_url: format!("{origin}/forges/1/github/setup").as_bytes(),
        },
        &crate::lower::github::protocol().documents.writer,
    )
    .expect("fake manifest");
    let mut form = String::from("manifest=");
    for byte in &manifest {
        write!(form, "%{byte:02X}").expect("form byte");
    }
    let layer = peer.github_layer_mut().expect("GitHub layer");
    let page = layer.page(
        &skein_fake_forge_github::pages::Page {
            method: Method::Post,
            target: b"/settings/apps/new?state=person",
            body: form.as_bytes(),
        },
        Wall::EPOCH,
    );
    assert_eq!(page.status, 200);
    let html = std::str::from_utf8(&page.body).expect("fake creation page");
    let code = html.split_once("?code=").expect("conversion link").1.split('&').next().expect("code");
    let target = format!("/app-manifests/{code}/conversions");
    let skein_fake_forge_github::service::Outcome::Reply(reply) = layer.request(
        &skein_fake_forge_github::service::Request {
            method: Method::Post,
            target: target.as_bytes(),
            authorization: None,
            validator: None,
            version: Some(pin::API_VERSION.as_bytes()),
            agent: true,
            body: b"",
        },
        Wall::EPOCH,
        ReplyTo::new(Token::new(1)),
    ) else {
        panic!("fake conversion completes")
    };
    assert_eq!(reply.status, 201);
    let value: serde_json::Value = serde_json::from_slice(&reply.body).expect("fake conversion shape");
    let pem = value["pem"].as_str().expect("fake issued key").as_bytes();
    OBSERVER.with(|cell| cell.borrow_mut().as_mut().expect("observer").key(pem));
    (
        value["id"].as_u64().expect("fake App id"),
        vec![
            skein_fake_machine::Item::file(b"app.pem", pem).mode(0o600),
            skein_fake_machine::Item::file(b"llm.key", fixtures::LLM_TOKEN).mode(0o600),
        ],
    )
}
fn scan_directory(
    observer: &mut Observer,
    disk: &mut skein_fake_machine::Machine,
    dir: skein_fake_machine::Opened,
    path: &[u8],
    counts: &mut Counts,
) {
    loop {
        let entries = disk.list(dir, 32, 4096).expect("public file listing");
        if entries.is_empty() {
            break;
        }
        for (kind, name) in entries {
            let path = [path, b"/", &name].concat();
            if secret_path(&path) {
                continue;
            }
            observer.check("persisted filename", b"", &path);
            if kind == skein_fake_machine::Is::Directory {
                let child = disk.open(dir, &name, skein_fake_machine::How::Directory).expect("public directory");
                scan_directory(observer, disk, child, &path, counts);
                disk.close(child);
            } else if kind == skein_fake_machine::Is::File {
                let file = disk.open(dir, &name, skein_fake_machine::How::ReadNoFollow).expect("public file");
                let bytes = disk
                    .read(file, 0, u32::try_from(disk.stat(file).size).expect("scenario file length"))
                    .expect("public persisted bytes");
                disk.close(file);
                observer.check("persisted file", &path, &bytes);
                counts.files += 1;
                counts.logs += usize::from(name.starts_with(b"log-") || path.starts_with(b"stderr/"));
                counts.transcripts += usize::from(path.windows(11).any(|part| part == b"transcripts"));
                counts.client_files += usize::from(path.starts_with(b"client/"));
                counts.worker_files += usize::from(path.starts_with(b"worker/"));
            }
        }
    }
}
pub(crate) fn finish(disk: &mut skein_fake_machine::Machine) -> Counts {
    OBSERVER.with(|cell| {
        let mut observer = cell.borrow_mut().take().expect("active observer");
        observer.scan();
        let mut counts = Counts {
            tokens: observer.tokens.len(),
            checkpoints: observer.checkpoints,
            contexts: observer.contexts,
            pages: observer.pages,
            wire_replies: observer.artifacts.iter().filter(|artifact| artifact.surface == "wire reply body").count(),
            ..Counts::default()
        };
        for (index, name) in [b"state".as_slice(), b"worker", b"client"].into_iter().enumerate() {
            let root = disk.reopen_root(u32::try_from(index).expect("scenario root"));
            scan_directory(&mut observer, disk, root, name, &mut counts);
            disk.close(root);
        }
        counts
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_secret_scans_cover_binary_debug_and_basic_encodings() {
        let mut observer = Observer::default();
        let control = b"observer-control-value";
        observer.token(control);
        for bytes in [
            control.to_vec(),
            [b"\xff".as_slice(), control, b"\x80"].concat(),
            format!("{control:?}").into_bytes(),
            encoded(&[b"x-access-token:", control].concat()).into_vec(),
        ] {
            assert!(observer.leak(&bytes).is_some(), "the seeded representation is detected");
        }
        assert!(observer.leak(b"observer-control-other").is_none());
    }
}
