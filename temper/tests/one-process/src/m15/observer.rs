//! Fake-issued values remain in this observer; failures name only their surface (temper/docs/design/testing/tiers.md, 1.5).
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
const ANCHOR_BYTES: usize = 16;

struct Matcher {
    anchors: aho_corasick::AhoCorasick,
    groups: Vec<Vec<usize>>,
}
impl Matcher {
    fn new(needles: &[Needle]) -> Self {
        let mut by_anchor: BTreeMap<&[u8], Vec<usize>> = BTreeMap::new();
        for (index, needle) in needles.iter().enumerate() {
            let bytes = needle.value.expose();
            by_anchor.entry(&bytes[..bytes.len().min(ANCHOR_BYTES)]).or_default().push(index);
        }
        let (anchors, groups): (Vec<_>, Vec<_>) = by_anchor.into_iter().unzip();
        let anchors = aho_corasick::AhoCorasick::builder()
            .kind(Some(aho_corasick::AhoCorasickKind::DFA))
            .prefilter(false)
            .build(anchors)
            .expect("scenario secret anchors");
        Self { anchors, groups }
    }
    fn find(&self, needles: &[Needle], bytes: &[u8], first: usize) -> Option<usize> {
        // Matches arrive in window order, but the finding names the earliest issued value.
        let mut best = needles.len();
        for matched in self.anchors.find_overlapping_iter(bytes) {
            for &index in &self.groups[matched.pattern().as_usize()] {
                if index >= best {
                    break;
                }
                if index >= first && bytes[matched.start()..].starts_with(needles[index].value.expose()) {
                    if index == first {
                        return Some(index);
                    }
                    best = index;
                    break;
                }
            }
        }
        (best < needles.len()).then_some(best)
    }
}

#[cfg(test)]
struct Window {
    bytes: Secret,
    first: usize,
    issued: usize,
    finding: Option<&'static str>,
}

struct Artifact {
    surface: &'static str,
    path: Vec<u8>,
    // An append keeps its predecessor, so every earlier version remains searchable.
    bytes: Vec<u8>,
    before: Option<usize>,
    length: usize,
    checked_needles: usize,
}
struct Stream {
    role: Role,
    bytes: Vec<u8>,
    parsed: usize,
    checked_bytes: usize,
    checked_needles: usize,
}
#[derive(Default)]
struct Observer {
    needles: Vec<Needle>,
    search: RefCell<Option<Matcher>>,
    #[cfg(test)]
    windows: RefCell<Vec<Window>>,
    tokens: Vec<Secret>,
    names: BTreeMap<(Role, i32), Vec<u8>>,
    files: BTreeMap<Vec<u8>, usize>,
    wires: BTreeMap<(Role, i32), usize>,
    streams: Vec<Stream>,
    artifacts: Vec<Artifact>,
    scanned_artifacts: usize,
    scanned_needles: usize,
    context_cursor: usize,
    contexts: usize,
    pages: usize,
    last_page: Option<Vec<skein_ui::Node>>,
    checkpoints: usize,
    check_token_shape: bool,
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
    OBSERVER
        .with(|observer| *observer.borrow_mut() = Some(Observer { check_token_shape: true, ..Observer::default() }));
}
pub(crate) fn start_manifest() {
    // The fake's first manifest conversion uses its first fixed test key.
    let pem = skein_fake_forge_github::keys::pem(skein_fake_forge_github::keys::KEYS[0]).expect("fake manifest PEM");
    let mut observer = Observer { check_token_shape: true, ..Observer::default() };
    observer.key(&pem);
    OBSERVER.with(|cell| *cell.borrow_mut() = Some(observer));
}

fn token_shape(mut bytes: &[u8]) -> bool {
    while let Some(at) = skein_lib::bytes::find(bytes, b"ghs_") {
        if bytes.get(at + 4..at + 40).is_some_and(|suffix| suffix.iter().all(u8::is_ascii_alphanumeric)) {
            return true;
        }
        bytes = &bytes[at + 4..];
    }
    false
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
        let debug = format!("{bytes:?}").into_bytes();
        *self.search.get_mut() = None;
        for value in [bytes.into(), debug.clone().into_boxed_slice(), debug[1..debug.len() - 1].into(), encoded(bytes)]
        {
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
    #[cfg(test)]
    fn leak(&self, bytes: &[u8]) -> Option<&'static str> {
        self.leak_since(bytes, 0)
    }
    fn leak_since(&self, bytes: &[u8], first: usize) -> Option<&'static str> {
        assert!(first <= self.needles.len(), "checked needles are a prefix of the issued values");
        if first == self.needles.len() {
            return None;
        }
        let mut search = self.search.borrow_mut();
        let search = search.get_or_insert_with(|| Matcher::new(&self.needles));
        search.find(&self.needles, bytes, first).map(|index| match self.needles[index].kind {
            Kind::Key => "App key",
            Kind::Token => "installation token",
        })
    }
    fn check(&self, surface: &str, path: &[u8], bytes: &[u8]) -> Result<(), String> {
        self.check_since(surface, path, bytes, 0)?;

        Ok(())
    }
    fn check_since(&self, surface: &str, path: &[u8], bytes: &[u8], first: usize) -> Result<(), String> {
        let inspect = |bytes: &[u8], first| {
            let finding = self.leak_since(bytes, first);
            #[cfg(test)]
            self.windows.borrow_mut().push(Window {
                bytes: Secret::new(bytes.into()),
                first,
                issued: self.needles.len(),
                finding,
            });
            finding
        };
        if inspect(path, first).is_some() || (self.check_token_shape && first == 0 && token_shape(path)) {
            return Err(format!("issued secret in {surface} filename"));
        }
        if inspect(bytes, first).is_some() {
            return Err(format!(
                "{} in {surface} {}",
                inspect(bytes, 0).unwrap_or("issued secret"),
                String::from_utf8_lossy(path)
            ));
        }
        if self.check_token_shape && first == 0 && token_shape(bytes) {
            return Err(format!("installation token shape in {surface}"));
        }

        Ok(())
    }
    fn save(&mut self, surface: &'static str, path: &[u8], bytes: &[u8]) {
        self.version(surface, path, None, bytes.to_vec());
    }
    fn version(&mut self, surface: &'static str, path: &[u8], before: Option<usize>, bytes: Vec<u8>) -> usize {
        let length = before.map_or(0, |index| self.artifacts[index].length) + bytes.len();
        let index = self.artifacts.len();
        self.artifacts.push(Artifact { surface, path: path.to_vec(), bytes, before, length, checked_needles: 0 });
        index
    }
    fn suffix(&self, index: usize, length: usize) -> Vec<u8> {
        let mut chunks = Vec::new();
        let mut remaining = length;
        let mut next = Some(index);
        while let Some(index) = next {
            if remaining == 0 {
                break;
            }
            let artifact = &self.artifacts[index];
            let take = remaining.min(artifact.bytes.len());
            chunks.push(&artifact.bytes[artifact.bytes.len() - take..]);
            remaining -= take;
            next = artifact.before;
        }
        chunks.reverse();
        chunks.concat()
    }
    fn file(&mut self, surface: &'static str, path: &[u8], at: Option<usize>, bytes: &[u8]) {
        let before = self.files.get(path).copied();
        let length = before.map_or(0, |index| self.artifacts[index].length);
        let at = at.unwrap_or(length);
        let index = if at >= length {
            let mut chunk = vec![0; at - length];
            chunk.extend_from_slice(bytes);
            self.version(surface, path, before, chunk)
        } else {
            // An overwrite starts a new chain; the old chain still holds every replaced version.
            let mut version = self.suffix(before.expect("existing file"), length);
            version.resize(length.max(at + bytes.len()), 0);
            version[at..at + bytes.len()].copy_from_slice(bytes);
            self.version(surface, path, None, version)
        };
        self.files.insert(path.to_vec(), index);
    }
    fn wire(&mut self, role: Role, fd: Fd, bytes: &[u8]) {
        let index = *self.wires.entry((role, fd.raw())).or_insert_with(|| {
            self.streams.push(Stream { role, bytes: Vec::new(), parsed: 0, checked_bytes: 0, checked_needles: 0 });
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
            (Op::Connect { fd, .. }, Ok(Done::Nothing)) | (Op::Accept { .. }, Ok(Done::Accepted { fd, .. })) => {
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
                    self.file(
                        "persisted write",
                        &path,
                        Some(usize::try_from(*at).expect("scenario file offset")),
                        &bytes[*from as usize..(*from + *count) as usize],
                    );
                }
            }
            (Op::Send { fd, bytes, from }, Ok(Done::Count(count))) if role == Role::Forge => {
                self.wire(role, *fd, &bytes[*from as usize..(*from + *count) as usize]);
            }
            (Op::Recv { fd, buf }, Ok(Done::Count(count)))
                if matches!(role, Role::Ctl | Role::Client | Role::Caller) =>
            {
                self.wire(role, *fd, &buf[..*count as usize]);
            }
            (Op::PipeWrite { fd, bytes, from }, Ok(Done::Count(count))) => {
                if matches!(role, Role::Agent | Role::Git | Role::Shell) {
                    self.wire(role, *fd, &bytes[*from as usize..(*from + *count) as usize]);
                }
                if let Some(path) = self.names.get(&(role, fd.raw())).cloned()
                    && path.starts_with(b"stderr/")
                {
                    self.file("process log", &path, None, &bytes[*from as usize..(*from + *count) as usize]);
                }
            }
            (Op::Close { fd }, _) => {
                self.names.remove(&(role, fd.raw()));
                self.wires.remove(&(role, fd.raw()));
            }
            _ => {}
        }
    }
    fn overlap(&self, first: usize) -> usize {
        self.needles[first..].iter().map(|needle| needle.value.expose().len() - 1).max().expect("unchecked needles")
    }
    fn scan(&mut self) -> Result<(), String> {
        if self.needles.is_empty() {
            return Err(String::from("the fake issued a key"));
        }
        let overlap = self.overlap(0);
        let first = if self.scanned_needles == self.needles.len() { self.scanned_artifacts } else { 0 };
        for index in first..self.artifacts.len() {
            let artifact = &self.artifacts[index];
            if artifact.checked_needles == self.needles.len() {
                continue;
            }
            if let Some(before) = artifact.before {
                let mut window = self.suffix(before, self.overlap(artifact.checked_needles));
                window.extend_from_slice(&artifact.bytes);
                self.check_since(artifact.surface, &artifact.path, &window, artifact.checked_needles)?;
            } else {
                self.check_since(artifact.surface, &artifact.path, &artifact.bytes, artifact.checked_needles)?;
            }
            self.artifacts[index].checked_needles = self.needles.len();
        }
        for index in 0..self.streams.len() {
            let stream = &self.streams[index];
            if stream.role == Role::Forge {
                continue;
            }
            let surface = if matches!(stream.role, Role::Agent | Role::Git | Role::Shell) {
                "worker process output"
            } else {
                "client wire"
            };
            // Newly issued needles also search the prefix kept before their issue.
            self.check_since(surface, b"", &stream.bytes[..stream.checked_bytes], stream.checked_needles)?;
            if stream.checked_bytes != stream.bytes.len() {
                self.check(surface, b"", &stream.bytes[stream.checked_bytes.saturating_sub(overlap)..])?;
            }
            self.streams[index].checked_bytes = self.streams[index].bytes.len();
            self.streams[index].checked_needles = self.needles.len();
        }
        self.scanned_artifacts = self.artifacts.len();
        self.scanned_needles = self.needles.len();
        self.checkpoints += 1;

        Ok(())
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
    if path == b"state/secrets" || path.starts_with(b"state/secrets/") {
        return true;
    }
    let Some(generation) = path.strip_prefix(b"state/generations/") else { return false };
    let mut segments = generation.split(|byte| *byte == b'/');
    segments.next().is_some_and(|number| !number.is_empty() && number.iter().all(u8::is_ascii_digit))
        && segments.next() == Some(b"secrets".as_slice())
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
pub(crate) fn page(nodes: &[skein_ui::Node]) {
    OBSERVER.with(|cell| {
        if let Some(observer) = &mut *cell.borrow_mut()
            && observer.last_page.as_deref().is_none_or(|last| {
                last != nodes
                    // Debug includes the bounded lists' capacities as well as their items.
                    || last.iter().zip(nodes).any(|(old, new)| {
                        old.states.capacity() != new.states.capacity()
                            || old.styles.capacity() != new.styles.capacity()
                            || old.text.capacity() != new.text.capacity()
                    })
            })
        {
            observer.pages += 1;
            observer.last_page = Some(nodes.to_vec());
            observer.save("rendered page", b"", format!("{nodes:?}").as_bytes());
        }
    });
}
pub(crate) fn context(peer: &skein_fake_peers::llm::Peer) -> Result<(), String> {
    OBSERVER.with(|cell| {
        if let Some(observer) = &mut *cell.borrow_mut() {
            let observations = peer.observations();
            for seen in observations
                .get(observer.context_cursor..)
                .ok_or_else(|| String::from("observed contexts are an observation prefix"))?
            {
                let skein_fake_peers::llm::Observation::Query { query, .. } = seen else { continue };
                observer.contexts += 1;
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
            observer.context_cursor = observations.len();
        }
        Ok(())
    })
}
pub(crate) fn checkpoint() -> Result<(), String> {
    OBSERVER.with(|cell| cell.borrow_mut().as_mut().ok_or_else(|| String::from("active observer"))?.scan())
}
pub(crate) fn tokens() -> usize {
    OBSERVER.with(|cell| cell.borrow().as_ref().map_or(0, |observer| observer.tokens.len()))
}
const APP_DOWNLOAD: &[u8] = b"app.pem";

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
            skein_fake_machine::Item::file(APP_DOWNLOAD, pem).mode(0o600),
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
    allow_download: bool,
) -> Result<(), String> {
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
            observer.check("persisted filename", b"", &path)?;
            if allow_download && path.strip_prefix(b"setup/") == Some(APP_DOWNLOAD) {
                // The one downloaded input, checked by metadata before this walk.
                continue;
            }
            if kind == skein_fake_machine::Is::Directory {
                let child = disk.open(dir, &name, skein_fake_machine::How::Directory).expect("public directory");
                let finding = scan_directory(observer, disk, child, &path, counts, allow_download);
                disk.close(child);
                finding?;
            } else if kind == skein_fake_machine::Is::File {
                let file = disk.open(dir, &name, skein_fake_machine::How::ReadNoFollow).expect("public file");
                let bytes = disk
                    .read(file, 0, u32::try_from(disk.stat(file).size).expect("scenario file length"))
                    .expect("public persisted bytes");
                disk.close(file);
                observer.check("persisted file", &path, &bytes)?;
                counts.files += 1;
                counts.logs += usize::from(name.starts_with(b"log-") || path.starts_with(b"stderr/"));
                let payload = jig_channel_kit::world::store::logs::path(&path);
                let turns = payload.is_some()
                    && std::str::from_utf8(&name)
                        .ok()
                        .and_then(|name| u64::from_str_radix(name, 16).ok())
                        .is_some_and(|log| log % 2 == 1);
                if turns {
                    let mut remaining = bytes.as_slice();
                    while let Some((length, body)) = jig_channel_kit::world::store::logs::record(remaining, u32::MAX) {
                        observer.check("persisted turn body", &path, &body)?;
                        remaining = &remaining[length..];
                    }
                }
                counts.logs += usize::from(payload.is_some());
                counts.transcripts += usize::from(turns || path.windows(11).any(|part| part == b"transcripts"));
                counts.client_files += usize::from(path.starts_with(b"client/"));
                counts.worker_files += usize::from(path.starts_with(b"worker/"));
            } else {
                panic!("public fixture entry is neither a file nor a directory");
            }
        }
    }
    Ok(())
}
fn scan_root(
    observer: &mut Observer,
    disk: &mut skein_fake_machine::Machine,
    root: skein_fake_machine::Opened,
    name: &[u8],
    counts: &mut Counts,
    allow_download: bool,
) -> Result<(), String> {
    use skein_fake_machine::How;
    if name == b"state" {
        let secrets = disk
            .open(root, b"generations/1/secrets", How::Directory)
            .map_err(|_| String::from("jig's secrets store must exist"))?;
        disk.close(secrets);
    }
    if allow_download && name == b"setup" {
        let download = disk
            .open(root, APP_DOWNLOAD, How::ReadNoFollow)
            .map_err(|_| String::from("the person's App key download must exist"))?;
        let facts = disk.stat(download);
        disk.close(download);
        if facts.is != skein_fake_machine::Is::File || facts.mode & 0o777 != 0o600 || facts.links != 1 {
            return Err(String::from("only the person's single private App key download may be left in setup"));
        }
    }
    scan_directory(observer, disk, root, name, counts, allow_download)
}

pub(crate) fn finish(disk: &mut skein_fake_machine::Machine) -> Result<Counts, String> {
    finish_roots(disk, true)
}

/// A manifest browser downloads no handover, so setup has no exception.
pub(crate) fn finish_manifest(disk: &mut skein_fake_machine::Machine) -> Result<Counts, String> {
    finish_roots(disk, false)
}

fn finish_roots(disk: &mut skein_fake_machine::Machine, allow_download: bool) -> Result<Counts, String> {
    OBSERVER.with(|cell| {
        let mut observer = cell.borrow_mut().take().expect("active observer");
        observer.scan()?;
        let mut counts = Counts {
            tokens: observer.tokens.len(),
            checkpoints: observer.checkpoints,
            contexts: observer.contexts,
            pages: observer.pages,
            wire_replies: observer.artifacts.iter().filter(|artifact| artifact.surface == "wire reply body").count(),
            ..Counts::default()
        };
        for (index, name) in [b"state".as_slice(), b"worker", b"client", b"setup"].into_iter().enumerate() {
            let root = disk.reopen_root(u32::try_from(index).expect("scenario root"));
            let finding = scan_root(&mut observer, disk, root, name, &mut counts, allow_download);
            disk.close(root);
            finding?;
        }
        Ok(counts)
    })
}

pub(crate) fn public_output(bytes: &[u8]) -> Result<String, String> {
    OBSERVER.with(|cell| {
        cell.borrow().as_ref().ok_or_else(|| String::from("active observer"))?.check("ctl diagnostic", b"", bytes)
    })?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Rng;

    struct Reference<'a> {
        needles: &'a [Needle],
        search: RefCell<Option<aho_corasick::AhoCorasick>>,
    }
    impl<'a> Reference<'a> {
        fn new(needles: &'a [Needle]) -> Self {
            Self { needles, search: RefCell::new(None) }
        }
        fn leak_since(&self, bytes: &[u8], first: usize) -> Option<&'static str> {
            assert!(first <= self.needles.len(), "checked needles are a prefix of the issued values");
            if first == self.needles.len() {
                return None;
            }
            let mut search = self.search.borrow_mut();
            let search = search.get_or_insert_with(|| {
                aho_corasick::AhoCorasick::new(self.needles.iter().map(|needle| needle.value.expose()))
                    .expect("scenario secret matcher")
            });
            search
                .find_overlapping_iter(bytes)
                .map(|matched| matched.pattern().as_usize())
                .filter(|index| *index >= first)
                .min()
                .map(|index| match self.needles[index].kind {
                    Kind::Key => "App key",
                    Kind::Token => "installation token",
                })
        }
        #[track_caller]
        fn assert_agreement(&self, observer: &Observer, bytes: &[u8], boundaries: &[usize]) {
            for &first in boundaries {
                assert_eq!(
                    observer.leak_since(bytes, first),
                    self.leak_since(bytes, first),
                    "window length {}, first needle {first}",
                    bytes.len()
                );
            }
        }
    }

    fn assert_recorded_windows(observer: &Observer) {
        let windows = observer.windows.take();
        assert!(!windows.is_empty(), "the scan judged windows");
        let mut reference = Reference::new(&[]);
        for window in &windows {
            if reference.needles.len() != window.issued {
                reference = Reference::new(&observer.needles[..window.issued]);
            }
            assert_eq!(window.finding, reference.leak_since(window.bytes.expose(), window.first));
        }
    }

    fn issue(needles: &[(Kind, &[u8])]) -> (Observer, Vec<usize>) {
        let mut observer = Observer::default();
        let mut boundaries = vec![0];
        for &(kind, value) in needles {
            observer.needle(kind, value);
            boundaries.push(observer.needles.len());
        }
        (observer, boundaries)
    }

    #[test]
    fn anchor_matches_agree_for_overlaps_and_shared_prefixes() {
        type Case<'a> = (&'a [(Kind, &'a [u8])], &'a [u8]);
        let anchor = [b'p'; ANCHOR_BYTES];
        let longer = [anchor.as_slice(), b"\0\xff"].concat();
        let sibling = [anchor.as_slice(), b"\x01\x80"].concat();
        let repeated = [longer.as_slice(), b"\x80", &longer].concat();
        let cases: &[Case<'_>] = &[
            (&[(Kind::Key, b"abc"), (Kind::Token, b"bc")], b"abc"),
            (&[(Kind::Key, b"bc"), (Kind::Token, b"abcd")], b"abcd"),
            (&[(Kind::Key, &sibling), (Kind::Token, &longer)], &longer),
            (&[(Kind::Key, &longer), (Kind::Token, &anchor)], &anchor),
            (&[(Kind::Key, &anchor), (Kind::Token, &longer)], &repeated),
            (&[(Kind::Key, &longer), (Kind::Token, &longer)], &repeated),
            (&[(Kind::Token, &longer), (Kind::Key, &sibling)], &longer[..longer.len() - 1]),
        ];
        for &(needles, bytes) in cases {
            let (observer, boundaries) = issue(needles);
            Reference::new(&observer.needles).assert_agreement(&observer, bytes, &boundaries);
        }
    }

    fn random_bytes(rng: &mut Rng, length: usize) -> Vec<u8> {
        (0..length).map(|_| rng.next_u64().to_le_bytes()[0]).collect()
    }

    #[test]
    fn key_and_installation_token_forms_agree_with_whole_values() {
        const KEY_BYTES: usize = 1192; // Size of an RSA-2048 PKCS#1 payload, with synthetic contents.
        const TOKEN_BODY: usize = 36; // The fake forge's alphanumeric bytes after `ghs_`.
        let mut rng = Rng::new(0x4d15);
        let pem = skein_fake_forge_github::keys::pem(&random_bytes(&mut rng, KEY_BYTES)).expect("fake-shaped PEM");
        let token = [b"ghs_".as_slice(), &[b'A'; TOKEN_BODY]].concat();
        let renewal = [b"ghs_".as_slice(), &[b'B'; TOKEN_BODY]].concat();
        let mut observer = Observer::default();
        observer.key(&pem);
        let after_key = observer.needles.len();
        observer.token(&token);
        let after_token = observer.needles.len();
        observer.token(&renewal);
        let boundaries = [0, after_key, after_token, observer.needles.len()];
        let reference = Reference::new(&observer.needles);
        for needle in &observer.needles {
            let window = [b"body: ".as_slice(), needle.value.expose(), b"\n"].concat();
            reference.assert_agreement(&observer, &window, &boundaries);
        }
        for cut in 0..=token.len() {
            let (left, right) = token.split_at(cut);
            reference.assert_agreement(&observer, left, &boundaries);
            reference.assert_agreement(&observer, right, &boundaries);
        }
        let page = b"[Node { text: [Waiting for checks], styles: [], states: [] }]";
        reference.assert_agreement(&observer, page, &boundaries);
    }

    #[test]
    fn seeded_binary_windows_agree_at_every_issuance_boundary() {
        // Four seeds, four needle sets each and 16 mixed windows per set give 256 cases.
        const SETS_PER_SEED: usize = 4;
        const WINDOWS_PER_SET: usize = 16;
        const NOISE_BYTES: usize = 32;
        for seed in [0x5315, 0xa151, 0xbc01, 0xfa11] {
            let mut rng = Rng::new(seed);
            for set in 0..SETS_PER_SEED {
                let tail = usize::try_from(rng.between(1, ANCHOR_BYTES as u64)).expect("short tail");
                let longer = random_bytes(&mut rng, ANCHOR_BYTES + tail);
                let mut sibling = longer.clone();
                *sibling.last_mut().expect("past the anchor") ^= 0xff;
                let inner = &longer[1..longer.len() - 1];
                // Include the minimum length in every seed.
                let length = if set == 0 {
                    1
                } else {
                    usize::try_from(rng.between(1, longer.len() as u64)).expect("small needle")
                };
                let other = random_bytes(&mut rng, length);
                let (observer, boundaries) =
                    issue(&[(Kind::Key, &longer), (Kind::Token, &sibling), (Kind::Key, inner), (Kind::Token, &other)]);
                let prefix = &longer[..ANCHOR_BYTES];
                let fragments: [&[u8]; 3] = [prefix, &longer, &sibling];
                let reference = Reference::new(&observer.needles);
                reference.assert_agreement(&observer, prefix, &boundaries);
                for _ in 0..WINDOWS_PER_SET {
                    let noise = random_bytes(&mut rng, NOISE_BYTES);
                    let suffix_at = usize::try_from(rng.below(longer.len() as u64)).expect("small offset");
                    let chosen = usize::try_from(rng.below(fragments.len() as u64)).expect("fragment index");
                    let window = [noise.as_slice(), fragments[chosen], &noise, &longer[suffix_at..], &other].concat();
                    reference.assert_agreement(&observer, &window, &boundaries);
                }
            }
        }
    }

    #[test]
    fn recv_fragments_are_judged_as_saved_artifacts() {
        let control = b"observer-control-value";
        start();
        OBSERVER.with(|cell| cell.borrow_mut().as_mut().expect("active observer").token(control));
        super::complete(
            Role::Client,
            &Complete {
                op: skein_lib::Token::new(1),
                kind: Op::Recv { fd: Fd::new(10), buf: control.as_slice().into() },
                result: Ok(Done::Count(u32::try_from(control.len()).expect("fragment length"))),
            },
        );
        let mut observer = OBSERVER.with(|cell| cell.borrow_mut().take().expect("active observer"));
        assert_eq!(observer.scan(), Err(String::from("installation token in client wire fragment ")));
        assert_recorded_windows(&observer);
    }

    #[test]
    fn new_needles_include_overlapping_matches_after_the_matcher_was_built() {
        let mut observer = Observer::default();
        observer.needle(Kind::Key, b"abc");
        assert_eq!(observer.leak(b"abc"), Some("App key"));
        let first = observer.needles.len();
        observer.needle(Kind::Token, b"bc");
        assert_eq!(observer.leak_since(b"abc", first), Some("installation token"));
        assert_eq!(observer.leak(b"abc"), Some("App key"), "the first needle still names the leak");
        assert!(observer.leak_since(b"abc", observer.needles.len()).is_none());
        Reference::new(&observer.needles).assert_agreement(&observer, b"abc", &[0, first, observer.needles.len()]);
    }
    #[test]
    fn opening_a_socket_preserves_other_response_streams() {
        let mut observer = Observer::default();
        let control = b"observer-control-value";
        observer.token(control);
        let first = Fd::new(10);
        let second = Fd::new(11);
        observer.wire(Role::Client, first, &control[..8]);
        observer.complete(
            Role::Client,
            &Complete {
                op: skein_lib::Token::new(1),
                kind: Op::Connect { fd: second, addr: fixtures::CLIENTS },
                result: Ok(Done::Nothing),
            },
        );
        observer.wire(Role::Client, second, b"unrelated response");
        observer.wire(Role::Client, first, &control[8..]);
        observer.complete(
            Role::Client,
            &Complete { op: skein_lib::Token::new(2), kind: Op::Close { fd: first }, result: Ok(Done::Nothing) },
        );
        assert!(observer.streams.iter().any(|stream| observer.leak(&stream.bytes).is_some()));
        assert!(!observer.wires.contains_key(&(Role::Client, first.raw())));
        assert!(observer.wires.contains_key(&(Role::Client, second.raw())));
    }
    #[test]
    fn a_new_token_checks_artifacts_saved_before_it_was_issued() {
        let mut observer = Observer::default();
        observer.token(b"observer-first-value");
        observer.save("earlier surface", b"", b"observer-second-value");
        assert_eq!(observer.scan(), Ok(()));
        observer.token(b"observer-second-value");
        let finding = observer.scan().expect_err("new token checks the saved artifact");
        assert_eq!(finding, "installation token in earlier surface ");
        for secret in ["observer-first-value", "observer-second-value"] {
            assert!(!finding.contains(secret), "the finding names only the surface");
        }
        assert_recorded_windows(&observer);
        OBSERVER.with(|cell| *cell.borrow_mut() = Some(observer));
        let mut disk = temper_playground::machine::fresh();
        assert_eq!(finish(&mut disk).expect_err("finish carries the scan finding"), finding);
        assert!(!enabled(), "finish clears the observer even on a finding");
    }
    #[test]
    fn appended_chunks_find_values_across_checkpoint_boundaries() {
        let control = b"observer-control-value";
        for (surface, path, append) in
            [("persisted write", b"state/log".as_slice(), false), ("process log", b"stderr/0", true)]
        {
            let mut observer = Observer::default();
            observer.token(control);
            let mut offset = 0;
            for chunk in [&control[..7], &control[7..14]] {
                observer.file(surface, path, (!append).then_some(offset), chunk);
                offset += chunk.len();
                assert_eq!(observer.scan(), Ok(()));
            }
            observer.file(surface, path, (!append).then_some(offset), &control[14..]);
            assert_eq!(
                observer.scan(),
                Err(format!("installation token in {surface} {}", String::from_utf8_lossy(path)))
            );
            assert_recorded_windows(&observer);
        }
    }
    #[test]
    fn a_new_token_checks_chunk_boundaries_in_overwritten_versions() {
        let mut observer = Observer::default();
        observer.token(b"prior");
        let future = b"observer-second-value-that-is-much-longer-than-the-first";
        observer.file("persisted write", b"state/log", Some(0), &future[..30]);
        observer.file("persisted write", b"state/log", Some(30), &future[30..]);
        observer.file("persisted write", b"state/log", Some(0), b"a replacement without the value");
        assert_eq!(observer.scan(), Ok(()));
        observer.token(future);
        assert_eq!(observer.scan(), Err(String::from("installation token in persisted write state/log")));
        assert_recorded_windows(&observer);
    }
    #[test]
    fn streams_check_appended_boundaries_and_new_tokens_in_closed_prefixes() {
        let mut observer = Observer::default();
        let control = b"observer-control-value";
        observer.token(control);
        observer.wire(Role::Client, Fd::new(10), &control[..8]);
        assert_eq!(observer.scan(), Ok(()));
        observer.wire(Role::Client, Fd::new(10), &control[8..]);
        assert_eq!(observer.scan(), Err(String::from("installation token in client wire ")));
        assert_recorded_windows(&observer);

        let mut observer = Observer::default();
        observer.token(b"observer-first-value");
        observer.wire(Role::Agent, Fd::new(11), control);
        assert_eq!(observer.scan(), Ok(()));
        observer.complete(
            Role::Agent,
            &Complete { op: skein_lib::Token::new(1), kind: Op::Close { fd: Fd::new(11) }, result: Ok(Done::Nothing) },
        );
        observer.token(control);
        assert_eq!(observer.scan(), Err(String::from("installation token in worker process output ")));
        assert_recorded_windows(&observer);
    }
    #[test]
    fn exact_secret_scans_cover_binary_debug_and_basic_encodings() {
        let mut observer = Observer::default();
        let control = b"observer-control-value";
        observer.token(control);
        for bytes in [
            control.to_vec(),
            [b"\xff".as_slice(), control, b"\x80"].concat(),
            format!("{control:?}").into_bytes(),
            format!("{:?}", [b"before-".as_slice(), control, b"-after"].concat()).into_bytes(),
            encoded(&[b"x-access-token:".as_slice(), control].concat()).into_vec(),
        ] {
            assert!(observer.leak(&bytes).is_some(), "the seeded representation is detected");
        }
        assert!(observer.leak(b"observer-control-other").is_none());
        let binary = b"\0\xff\x80\x07";
        observer.needle(Kind::Key, binary);
        assert_eq!(observer.leak(&[b"before".as_slice(), binary, b"after"].concat()), Some("App key"));
        assert!(observer.leak(b"\0\xfe\x80\x07").is_none());
        assert!(secret_path(b"state/generations/1/secrets"));
        assert!(!secret_path(b"worker/workspaces/0/app/secrets/token"));
        assert!(!secret_path(b"client/private/secrets/token"));
        assert!(!secret_path(b"state/generations/1/secrets-public"));
    }
}
