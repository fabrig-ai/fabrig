//! Bounded public-document rewrites for protocol/configuration/directories.md, 1. No files or sockets are opened here.
use crate::{Refusal, read};
use alloc::boxed::Box;
use core::net::{IpAddr, SocketAddr};
use jig_protocol_config::{Path, read as config};
use skein_json::{
    Document,
    document::Kind,
    writer::{Encoder, Limits},
};
use skein_lib::{Decimal, List, bytes};
use temper_forge_setup::{Kind as StepKind, Step};

/// Absolute effective directories and the already provisioned bundle.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Paths {
    pub config: Box<[u8]>,
    pub data: Box<[u8]>,
    pub cache: Box<[u8]>,
    pub bundle: Box<[u8]>,
    pub claude: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Peers {
    pub github: SocketAddr,
    pub oauth: SocketAddr,
    pub llm: SocketAddr,
    pub forgejo: SocketAddr,
}
/// Shared by the supervised welcome peers (protocol/setup/box.md, 5).
pub const SEED: u64 = 7;

/// The fake program's model, alongside the credential-backed `OpenAI` model.
pub const PROGRAM_MODEL: &str = "claude-opus-5-5";

/// Long peers before ordinary box steps. The caller checks absolute, bounded config bytes and sockets.
#[must_use]
pub fn steps(config: &[u8], data: &[u8], peers: &Peers, scripts: Option<&[u8]>, strict: bool) -> Box<[Step]> {
    let mut steps = List::with_capacity(4);
    let seed = Decimal::of(SEED);
    for (name, command, document, listen) in [
        (b"fake github".as_slice(), b"forge".as_slice(), b"github.json".as_slice(), peers.github),
        (b"fake issuer", b"oauth", b"oauth.json", peers.oauth),
        (b"fake model", b"llm", b"llm.json", peers.llm),
        (b"fake forgejo", b"forge", b"forgejo.json", peers.forgejo),
    ] {
        let mut arguments = List::with_capacity(9);
        for word in [b"fake".as_slice(), command, b"--document"] {
            arguments.push(bytes::copy_of(word)).expect("peer arguments counted");
        }
        arguments.push(path(config, document)).expect("peer document counted");
        arguments.push(bytes::copy_of(b"--seed")).expect("seed flag counted");
        arguments.push(bytes::copy_of(seed.as_bytes())).expect("seed counted");
        // protocol/setup/box.md, 5: scripts and strict belong to this start's model alone.
        if command == b"llm" {
            if let Some(scripts) = scripts {
                arguments.push(bytes::copy_of(b"--scripts")).expect("scripts flag counted");
                arguments.push(bytes::copy_of(scripts)).expect("scripts path counted");
            }
            if strict {
                arguments.push(bytes::copy_of(b"--strict")).expect("strict flag counted");
            }
        }
        if command == b"forge" {
            arguments.push(bytes::copy_of(b"--state")).expect("state flag counted");
            let name = if name == b"fake github" { b"github-forge.state".as_slice() } else { b"forgejo-forge.state" };
            arguments.push(path(data, name)).expect("state path counted");
        }
        steps
            .push(Step::new(bytes::copy_of(name), arguments.into_boxed(), StepKind::Long).listening(Box::new([listen])))
            .expect("four welcome peers counted");
    }
    steps.into_boxed()
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Documents {
    pub temper: Box<[u8]>,
    pub agent: Box<[u8]>,
    pub github: Box<[u8]>,
    pub oauth: Box<[u8]>,
    pub llm: Box<[u8]>,
    pub forgejo: Box<[u8]>,
}
const OUTPUT: Limits = Limits { depth: 32, length: crate::DOCUMENT_BYTES };
// Init-shaped temper: forge roots (1), sign-in (6), OpenAI endpoint (2),
// Anthropic endpoint and provider endpoint (2), its direct way removed (3), bundle (1), worker (2),
// capacity overrides (3). Sign-in and OpenAI may remove four TLS members each.
// The agent's OpenAI endpoint (6), Anthropic endpoints (2) and adapter (1) fit too.
const EDITS: u32 = 1 + 6 + 2 + 2 + 3 + 1 + 2 + 3 + 2 * 4;
// The Model step must see OpenAI, demo and Anthropic in one provider offer.
const PROVIDER_LINES: u32 = 3;
#[derive(Debug)]
struct Edit {
    first: u32,
    after: u32,
    value: Option<Document>,
}
#[derive(Debug)]
struct Edits {
    rows: List<Edit>,
}
impl Edits {
    fn new() -> Self {
        Self { rows: List::with_capacity(EDITS) }
    }
    fn set(&mut self, d: &Document, at: u32, bytes: &[u8], p: &Path) -> Result<(), Refusal> {
        let after = config::after(d, at).or(Err(read::why(p, b"invalid value")))?;
        let value = Some(read::document(bytes)?);
        self.rows.push(Edit { first: at, after, value }).or(Err(read::why(p, b"too many rewrites")))
    }
    fn text(&mut self, d: &Document, at: u32, value: &[u8], p: &Path) -> Result<(), Refusal> {
        self.set(d, at, &text(value)?, p)
    }
    fn remove(&mut self, d: &Document, at: u32, p: &Path) -> Result<(), Refusal> {
        let previous = at.checked_sub(1).expect("a removed value has a parent");
        let first = if d.token(previous).expect("preceding token").kind == Kind::Key { previous } else { at };
        let after = config::after(d, at).or(Err(read::why(p, b"invalid value")))?;
        self.rows.push(Edit { first, after, value: None }).or(Err(read::why(p, b"too many rewrites")))
    }
    fn write(&self, d: &Document, out: &mut Encoder) {
        let mut skip = 0;
        for at in 0..d.len() {
            if at < skip {
                continue;
            }
            let mut edited = false;
            for edit in &self.rows {
                if edit.first == at {
                    if let Some(value) = &edit.value {
                        if edit.first == edit.after
                            && value.token(0).expect("inserted value").kind == Kind::ObjectStart
                            && d.token(at).expect("container close").kind == Kind::ObjectEnd
                        {
                            for inner in 1..value.len().checked_sub(1).expect("object close") {
                                token(value, inner, out);
                            }
                        } else {
                            out.document(value);
                        }
                    }
                    skip = edit.after;
                    edited = true;
                    if skip > at {
                        break;
                    }
                }
            }
            if !edited || skip == at {
                token(d, at, out);
            }
        }
    }
    fn finish(&self, d: &Document) -> Result<Box<[u8]>, Refusal> {
        let mut out = Encoder::measure(&OUTPUT);
        self.write(d, &mut out);
        let size = out.measured().or(Err(read::why(&Path::root(), b"effective document past its byte bound")))?;
        let mut out = Encoder::write(size, &OUTPUT);
        self.write(d, &mut out);
        Ok(out.finish())
    }
}
fn token(d: &Document, at: u32, out: &mut Encoder) {
    let t = d.token(at).expect("bounded document token");
    let bytes = d.text(t).expect("document text admitted");
    match t.kind {
        Kind::ObjectStart => out.object_start(),
        Kind::ObjectEnd => out.object_end(),
        Kind::ArrayStart => out.array_start(),
        Kind::ArrayEnd => out.array_end(),
        Kind::Key => out.key(bytes),
        Kind::String => out.string(bytes),
        Kind::Number => out.number(bytes),
        Kind::True => out.boolean(true),
        Kind::False => out.boolean(false),
        Kind::Null => out.null(),
        Kind::Long => unreachable!("whole document parsing retains bounded strings"),
    }
}
fn text(bytes: &[u8]) -> Result<Box<[u8]>, Refusal> {
    let mut out = Encoder::measure(&OUTPUT);
    out.string(bytes);
    let size = out.measured().or(Err(read::why(&Path::root(), b"text past its byte bound")))?;
    let mut out = Encoder::write(size, &OUTPUT);
    out.string(bytes);
    Ok(out.finish())
}
fn member(d: &Document, at: u32, name: &[u8], p: &Path) -> Result<u32, Refusal> {
    let mut o = read::object(d, at, p)?;
    read::required(&mut o, d, name, p)
}
fn selected(d: &Document, at: u32, key: &[u8], wanted: &[u8], number: bool, p: &Path) -> Result<u32, Refusal> {
    optional_selected(d, at, key, wanted, number, p)?.ok_or(read::why(p, b"target missing"))
}
fn optional_selected(
    d: &Document,
    at: u32,
    key: &[u8],
    wanted: &[u8],
    number: bool,
    p: &Path,
) -> Result<Option<u32>, Refusal> {
    let items = read::list(d, at, 1, p)?;
    let mut found = None;
    for (i, at) in items.iter().enumerate() {
        let row = read::index(p, i);
        let index = member(d, *at, key, &row)?;
        let value = if number {
            let n = read::integer(d, index, u64::from(u32::MAX), &row.member(key))?;
            read::joined(&[Decimal::of(n).as_bytes()])
        } else {
            read::text(d, index, 128, &row.member(key))?
        };
        if value.as_ref() == wanted {
            if found.is_some() {
                return Err(read::why(p, b"target must be unique"));
            }
            found = Some(*at);
        }
    }
    Ok(found)
}
fn socket(addr: SocketAddr, at: &Path) -> Result<Box<[u8]>, Refusal> {
    if !addr.ip().is_loopback() || addr.port() == 0 || addr.port() == 1455 {
        return Err(read::why(at, b"expected a nonzero loopback peer, distinct from callback 1455"));
    }
    match addr {
        SocketAddr::V6(ip) if ip.scope_id() != 0 || ip.flowinfo() != 0 => {
            return Err(read::why(at, b"scoped IPv6 peers are unsupported"));
        }
        SocketAddr::V4(_) | SocketAddr::V6(_) => {}
    }
    let port = Decimal::of(u64::from(addr.port()));
    match addr.ip() {
        IpAddr::V4(ip) => {
            let [a, b, c, d] = ip.octets();
            let a = Decimal::of(u64::from(a));
            let b = Decimal::of(u64::from(b));
            let c = Decimal::of(u64::from(c));
            let d = Decimal::of(u64::from(d));
            Ok(read::joined(&[
                a.as_bytes(),
                b".",
                b.as_bytes(),
                b".",
                c.as_bytes(),
                b".",
                d.as_bytes(),
                b":",
                port.as_bytes(),
            ]))
        }
        IpAddr::V6(_) => Ok(read::joined(&[b"[::1]:", port.as_bytes()])),
    }
}
fn address(socket: &[u8]) -> Result<Box<[u8]>, Refusal> {
    Ok(read::joined(&[b"{\"socket\":", &text(socket)?, b"}"]))
}
fn path(base: &[u8], name: &[u8]) -> Box<[u8]> {
    if base.ends_with(b"/") { read::joined(&[base, name]) } else { read::joined(&[base, b"/", name]) }
}
#[expect(clippy::disallowed_methods, reason = "public path input is validated at this JSON protocol boundary")]
fn paths(paths: &Paths) -> Result<(), Refusal> {
    for (name, bytes) in [
        (b"config".as_slice(), &paths.config),
        (b"data", &paths.data),
        (b"cache", &paths.cache),
        (b"bundle", &paths.bundle),
        (b"claude", &paths.claude),
    ] {
        let p = Path::root().member(name);
        read::absolute(bytes, &p)?;
        if core::str::from_utf8(bytes).is_err() {
            return Err(read::why(&p, b"path must be UTF-8"));
        }
        let suffix = match name {
            b"config" => b"/agent.json".len(),
            b"data" => b"/github-admin.token".len(),
            _ => 0,
        };
        if bytes.len().saturating_add(suffix) > 4096 || bytes.ends_with(b"/..") || bytes::find(bytes, b"/../").is_some()
        {
            return Err(read::why(&p, b"path past its bound"));
        }
    }
    Ok(())
}
fn tls(d: &Document, at: u32, edits: &mut Edits, p: &Path) -> Result<(), Refusal> {
    let mut o = read::object(d, at, p)?;
    for name in [b"server_name".as_slice(), b"trust", b"trust_der", b"identity_file"] {
        if let Some(value) = o.member(d, name) {
            edits.remove(d, value, &p.member(name))?;
        }
    }
    Ok(())
}
fn input_address(d: &Document, at: u32, p: &Path) -> Result<(), Refusal> {
    let mut problems = jig_protocol_config::Problems::empty();
    let value = jig_protocol_config::address::read(d, at, p, &mut problems);
    if let Some(problem) = problems.problems.get(0) {
        return Err(read::kind(&problem.path, problem.what));
    }
    match value {
        Some(_) => Ok(()),
        None => Err(read::why(p, b"wrong value kind")),
    }
}
fn endpoint(d: &Document, at: u32, socket: &[u8], agent: bool, edits: &mut Edits, p: &Path) -> Result<(), Refusal> {
    let at = selected(d, at, b"name", b"openai", false, p)?;
    let number = member(d, at, b"number", p)?;
    if read::integer(d, number, u64::from(u32::MAX), &p.member(b"number"))? != 1 {
        return Err(read::why(&p.member(b"number"), b"openai endpoint must be number 1"));
    }
    let address_at = member(d, at, b"address", p)?;
    if agent {
        let _value = read::text(d, address_at, 4096, &p.member(b"address"))?;
    } else {
        input_address(d, address_at, &p.member(b"address"))?;
    }
    let transport = member(d, at, b"transport", p)?;
    let _value = read::text(d, transport, 32, &p.member(b"transport"))?;
    edits.set(d, address_at, &if agent { text(socket)? } else { address(socket)? }, &p.member(b"address"))?;
    edits.text(d, transport, b"plaintext", &p.member(b"transport"))?;
    tls(d, at, edits, p)
}

fn program_endpoint(socket: &[u8], agent: bool) -> Result<Box<[u8]>, Refusal> {
    let tail = if agent {
        bytes::copy_of(br#""connect":10000,"handshake":10000,"limits":{"window":1048576,"output":8192,"reasoning_item":65536,"head":60000,"idle":30000}"#)
    } else {
        bytes::copy_of(br#""connect_ms":10000,"handshake_ms":10000"#)
    };
    Ok(read::joined(&[
        br#"{"name":"fake-claude","number":2,"dialect":2,"provider":"anthropic","program":"claude_code","address":"#,
        &if agent { text(socket)? } else { address(socket)? },
        br#", "transport":"plaintext","headers":[],"#,
        &tail,
        b"}",
    ]))
}

fn program_route(d: &Document, llm: u32, socket: &[u8], edits: &mut Edits, p: &Path) -> Result<(), Refusal> {
    let endpoints = member(d, llm, b"endpoints", p)?;
    let endpoint_path = p.member(b"endpoints");
    let endpoint = selected(d, endpoints, b"name", b"anthropic", false, &endpoint_path)?;
    edits.set(d, endpoint, &program_endpoint(socket, false)?, &endpoint_path)?;
    remove_direct_endpoint(d, endpoints, edits, &endpoint_path)?;
    let providers = member(d, llm, b"providers", p)?;
    let provider_path = p.member(b"providers");
    let provider = selected(d, providers, b"name", b"anthropic", false, &provider_path)?;
    let endpoint = member(d, provider, b"endpoint", &provider_path)?;
    edits.text(d, endpoint, b"fake-claude", &provider_path.member(b"endpoint"))?;
    remove_member(d, provider, b"direct", edits, &provider_path)?;
    let sources = member(d, provider, b"sources", &provider_path)?;
    remove_member(d, sources, b"keys", edits, &provider_path.member(b"sources"))
}
fn remove_member(d: &Document, at: u32, name: &[u8], edits: &mut Edits, p: &Path) -> Result<(), Refusal> {
    if let Some(at) = read::object(d, at, p)?.member(d, name) {
        edits.remove(d, at, &p.member(name))?;
    }
    Ok(())
}
fn remove_direct_endpoint(d: &Document, endpoints: u32, edits: &mut Edits, p: &Path) -> Result<(), Refusal> {
    if let Some(at) = optional_selected(d, endpoints, b"name", b"anthropic-api", false, p)? {
        edits.remove(d, at, p)?;
    }
    Ok(())
}

/// Wire welcome peers and a program model in ordinary documents. The shell validates and rebases imported resources first.
#[expect(clippy::too_many_lines, reason = "the bounded rewrite writes its six coordinated documents together")]
pub fn rewrite(temper: &[u8], agent: &[u8], paths: &Paths, peers: &Peers) -> Result<Documents, Refusal> {
    for (name, bytes) in [(b"temper".as_slice(), temper), (b"agent", agent)] {
        if bytes.len() > usize::try_from(crate::DOCUMENT_BYTES).expect("document bound fits") {
            return Err(read::why(&Path::root().member(name), b"past its bound"));
        }
    }
    self::paths(paths)?;
    let github = socket(peers.github, &Path::root().member(b"github"))?;
    let oauth = socket(peers.oauth, &Path::root().member(b"oauth"))?;
    let llm = socket(peers.llm, &Path::root().member(b"llm"))?;
    let forgejo = socket(peers.forgejo, &Path::root().member(b"forgejo"))?;
    if peers.github == peers.oauth
        || peers.github == peers.llm
        || peers.oauth == peers.llm
        || [peers.github, peers.oauth, peers.llm].contains(&peers.forgejo)
    {
        return Err(read::why(&Path::root(), b"peer sockets must be distinct"));
    }
    let d = read::document(temper)?;
    let a = read::document(agent)?;
    duplicates(&d)?;
    duplicates(&a)?;
    let root = Path::root();
    let mut edits = Edits::new();
    let application = member(&d, 0, b"application", &root)?;
    let ap = root.member(b"application");
    let machine_at = member(&d, application, b"machine", &ap)?;
    let mut problems = temper_forge_setup::problem::Problems::empty();
    let directories = jig_protocol_config::Directories::given(&paths.config, &paths.data, &paths.cache)
        .expect("checked effective directories");
    let machine = temper_forge_setup::machine::read(
        &d,
        machine_at,
        &temper_forge_setup::limits::small(),
        &directories,
        &mut problems,
    )
    .ok_or(read::why(&ap.member(b"machine"), b"invalid machine"))?;
    let git = machine.fake.ok_or(read::why(&ap.member(b"machine"), b"fake machine required"))?.git;
    forge(&d, application, &github, &git, &mut edits, &ap)?;
    let engine = member(&d, 0, b"engine", &root)?;
    let ep = root.member(b"engine").member(b"llm");
    let llm_at = member(&d, engine, b"llm", &root.member(b"engine"))?;
    sign_in(&d, llm_at, &oauth, &mut edits, &ep)?;
    let endpoints = member(&d, llm_at, b"endpoints", &ep)?;
    endpoint(&d, endpoints, &llm, false, &mut edits, &ep.member(b"endpoints"))?;
    program_route(&d, llm_at, &llm, &mut edits, &ep)?;
    let clients = member(&d, application, b"clients", &ap)?;
    let bundle = member(&d, clients, b"bundle", &ap.member(b"clients"))?;
    if !read::is_null(&d, bundle) {
        let _value = read::text(&d, bundle, 4096, &ap.member(b"clients").member(b"bundle"))?;
    }
    edits.text(&d, bundle, &paths.bundle, &ap.member(b"clients").member(b"bundle"))?;
    worker(&d, &paths.config, &mut edits)?;
    capacity(&d, &mut edits)?;
    let mut agent_edits = Edits::new();
    let endpoints = member(&a, 0, b"endpoints", &root)?;
    endpoint(&a, endpoints, &llm, true, &mut agent_edits, &root.member(b"endpoints"))?;
    let target = selected(&a, endpoints, b"name", b"anthropic", false, &root.member(b"endpoints"))?;
    agent_edits.set(&a, target, &program_endpoint(&llm, true)?, &root.member(b"endpoints"))?;
    remove_direct_endpoint(&a, endpoints, &mut agent_edits, &root.member(b"endpoints"))?;
    let programs = member(&a, 0, b"programs", &root)?;
    let adapter = read::joined(&[
        b"{\"path\":",
        &text(&paths.claude)?,
        b",\"directory\":",
        &text(&path(&paths.data, b"programs"))?,
        b"}",
    ]);
    object_member(&a, programs, b"claude_code", &adapter, &mut agent_edits)?;
    Ok(Documents {
        temper: edits.finish(&d)?,
        agent: agent_edits.finish(&a)?,
        github: forge_document(&github, &paths.data, &d, clients)?,
        oauth: read::joined(&[
            b"{\"listen\":",
            &address(&oauth)?,
            b",\"redirect_port\":1455,\"redirect_path\":\"auth/callback\"}",
        ]),
        llm: model_document(&llm, &paths.data)?,
        forgejo: read::joined(&[
            b"{\"listen\":",
            &address(&forgejo)?,
            b",\"git_program\":",
            &text(&git)?,
            b",\"admin_token\":",
            &text(&path(&paths.data, b"forgejo-admin.token"))?,
            br##", "owner":"fake-person","repositories":[{"name":"widget","landing":"main","ci":null}],
                "sources":[
                    {"address":"https://code.example.invalid/acme/widget","repository":{
                        "owner":"acme","name":"widget","landing":"main","tree":[
                            {"path":"README.md","content":"# Widget\n"},
                            {"path":".github/workflows/ci.yml","content":"name: ci\non: [push, pull_request]\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo widget\n"}
                        ]}},
                    {"address":"https://code.example.invalid/acme/gadget","repository":{
                        "owner":"acme","name":"gadget","landing":"trunk","branches":["topic"],"tree":[
                            {"path":"README.md","content":"# Gadget\n"},
                            {"path":".github/workflows/ci.yml","content":"name: ci\non: [push, pull_request]\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo gadget\n"}
                        ]}}
                ]}"##,
        ]),
    })
}
// Insert at the closing token so edits to existing children remain independent.
fn append(d: &Document, at: u32, bytes: &[u8], edits: &mut Edits) -> Result<(), Refusal> {
    let close = config::after(d, at)
        .or(Err(read::why(&Path::root(), b"invalid container")))?
        .checked_sub(1)
        .expect("container close");
    let value = read::document(bytes)?;
    // A zero-width insertion leaves the original container close to be emitted.
    edits
        .rows
        .push(Edit { first: close, after: close, value: Some(value) })
        .or(Err(read::why(&Path::root(), b"too many rewrites")))
}
fn object_member(d: &Document, at: u32, name: &[u8], value: &[u8], edits: &mut Edits) -> Result<(), Refusal> {
    let p = Path::root().member(name);
    let mut object = read::object(d, at, &p)?;
    if let Some(old) = object.member(d, name) {
        edits.set(d, old, value, &p)
    } else {
        // A fragment is encoded as an object; write strips its outer container.
        append(d, at, &read::joined(&[b"{", &text(name)?, b":", value, b"}"]), edits)
    }
}
fn capacity(d: &Document, edits: &mut Edits) -> Result<(), Refusal> {
    let engine = member(d, 0, b"engine", &Path::root())?;
    let limits = member(d, engine, b"limits", &Path::root())?;
    let overrides = member(d, limits, b"overrides", &Path::root())?;
    object_member(d, overrides, b"accounts.credentials", b"4", edits)?;
    let path = Path::root().member(b"engine").member(b"limits");
    let parsed = jig_protocol_config::overrides::parse(d, limits, &path);
    if !parsed.problems.is_empty() {
        return Err(read::why(&path, b"invalid engine limits"));
    }
    let receiving = u64::from(parsed.limits.core.documents.lines);
    let lines = Decimal::of(receiving.max(u64::from(PROVIDER_LINES)));
    object_member(d, overrides, b"documents.lines", lines.as_bytes(), edits)?;
    let workers = member(d, 0, b"workers", &Path::root())?;
    for worker in &read::list(d, workers, 1, &Path::root().member(b"workers"))? {
        let limits = member(d, *worker, b"limits", &Path::root())?;
        let overrides = member(d, limits, b"overrides", &Path::root())?;
        object_member(d, overrides, b"accounts.credentials", b"4", edits)?;
    }
    Ok(())
}
fn forge(
    d: &Document,
    application: u32,
    github: &[u8],
    git: &[u8],
    edits: &mut Edits,
    ap: &Path,
) -> Result<(), Refusal> {
    let machine = member(d, application, b"machine", ap)?;
    let origin = text(&read::joined(&[b"http://", github]))?;
    let value = read::joined(&[
        b"{\"api\":",
        &address(github)?,
        b",\"git\":",
        &text(git)?,
        b",\"transport\":\"plaintext\",\"remote\":",
        &origin,
        b",\"web\":",
        &origin,
        b"}",
    ]);
    object_member(d, machine, b"github", &value, edits)
}
fn sign_in(d: &Document, llm_at: u32, oauth: &[u8], edits: &mut Edits, ep: &Path) -> Result<(), Refusal> {
    let providers = member(d, llm_at, b"providers", ep)?;
    let pp = ep.member(b"providers");
    let provider = selected(d, providers, b"number", b"1", true, &pp)?;
    let name = member(d, provider, b"name", &pp)?;
    if read::text(d, name, 128, &pp.member(b"name"))?.as_ref() != b"openai" {
        return Err(read::why(&pp.member(b"name"), b"provider 1 must be openai"));
    }
    let sources = member(d, provider, b"sources", &pp)?;
    let sp = pp.member(b"sources").member(b"sign_in");
    let sign_in = member(d, sources, b"sign_in", &pp.member(b"sources"))?;
    registration(d, sign_in, &sp)?;
    let origin = read::joined(&[b"http://", oauth]);
    for (name, suffix) in [
        (b"issuer".as_slice(), b"".as_slice()),
        (b"authorization", b"/api/accounts/authorize"),
        (b"token", b"/api/accounts/oauth/token"),
        (b"revocation", b"/api/accounts/oauth/revoke"),
        (b"keys", b"/.well-known/jwks.json"),
    ] {
        let at = member(d, sign_in, name, &sp)?;
        let old = read::text(d, at, 1024, &sp.member(name))?;
        let expected = read::joined(&[b"https://auth.openai.com", suffix]);
        if old != expected {
            return Err(read::why(&sp.member(name), b"expected the shipped sign-in origin and path"));
        }
        edits.text(d, at, &read::joined(&[&origin, suffix]), &sp.member(name))?;
    }
    let transport = member(d, sign_in, b"transport", &sp)?;
    let _value = read::text(d, transport, 32, &sp.member(b"transport"))?;
    edits.text(d, transport, b"plaintext", &sp.member(b"transport"))?;
    tls(d, sign_in, edits, &sp)?;
    Ok(())
}
fn worker(d: &Document, config: &[u8], edits: &mut Edits) -> Result<(), Refusal> {
    let workers = member(d, 0, b"workers", &Path::root())?;
    let wp = Path::root().member(b"workers");
    let workers = read::list(d, workers, 1, &wp)?;
    if workers.len() != 1 {
        return Err(read::why(&wp, b"expected one standalone worker"));
    }
    let worker = *workers.get(0).expect("one worker");
    let agent_at = member(d, worker, b"agent", &wp)?;
    let program_at = member(d, agent_at, b"program", &wp.member(b"agent"))?;
    let program = read::text(d, program_at, 4096, &wp.member(b"agent").member(b"program"))?;
    read::absolute(&program, &wp.member(b"agent").member(b"program"))?;
    let configuration = member(d, agent_at, b"configuration", &wp.member(b"agent"))?;
    let arguments = member(d, agent_at, b"arguments", &wp.member(b"agent"))?;
    let args = read::list(d, arguments, 1, &wp.member(b"agent").member(b"arguments"))?;
    if args.len() != 3
        || read::text(d, *args.get(0).expect("nonempty argv"), 16, &wp)?.as_ref() != b"internal"
        || read::text(d, *args.get(1).expect("three argv entries"), 16, &wp)?.as_ref() != b"agent"
    {
        return Err(read::why(&wp.member(b"agent").member(b"arguments"), b"expected agent and its configuration"));
    }
    let previous = read::text(d, configuration, 4096, &wp.member(b"agent").member(b"configuration"))?;
    let argument =
        read::text(d, *args.get(2).expect("three argv entries"), 4096, &wp.member(b"agent").member(b"arguments"))?;
    if previous != argument {
        return Err(read::why(
            &wp.member(b"agent").member(b"arguments"),
            b"configuration and argv must name the same agent document",
        ));
    }
    let name = path(config, b"agent.json");
    edits.text(d, configuration, &name, &wp)?;
    edits.set(d, arguments, &read::joined(&[b"[\"internal\",\"agent\",", &text(&name)?, b"]"]), &wp)?;
    Ok(())
}
fn forge_document(socket: &[u8], data: &[u8], d: &Document, clients: u32) -> Result<Box<[u8]>, Refusal> {
    let p = Path::root().member(b"application").member(b"clients").member(b"listener");
    let listener = member(d, clients, b"listener", &p)?;
    let at = member(d, listener, b"socket", &p)?;
    let clients = read::text(d, at, 128, &p.member(b"socket"))?;
    // The sandbox starts without forge records; Connect makes the first one
    // (temper/docs/design/protocol/forge/engine.md, 2).
    let setup = text(&read::joined(&[b"http://", &clients, b"/forges/1/github/setup"]))?;
    Ok(read::joined(&[b"{\"listen\":", &address(socket)?, b",\"admin_token\":", &text(&path(data, b"github-admin.token"))?,
        br#", "owner":"fake-person","repositories":[{"name":"widget","landing":"main","ci":null}],"github":{"app":{"id":1,"slug":"temper-fake","bot":2,"key":0,"setup_url":"#,&setup,br#"},"accounts":[{"login":"fake-person","id":3,"kind":"user","installation":{"id":1,"repositories":"all"}}]}}"#]))
}
// The document scope names a signed-in fake box's first chat, T2, after the trial's T1.
// The demo's document revision names note 3; the farewell draft is note 2.
// Chat-draft's report names goal T3; the local goal walks keep their goal T2 scope.
pub(crate) fn model_document(socket: &[u8], data: &[u8]) -> Result<Box<[u8]>, Refusal> {
    // Open chats cue from the latest request; restored chats may cue from their
    // opening brief in the system text. Both cues share the same input rules.
    let farewell = br##""turns":[{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":2},"description":"Farewell line","body":"# Farewell line\n\n_Draft, with the user._\n\n## Purpose\n\nAdd a farewell line to hello, alongside its greeting.\n\n## Checks\n\nCheck README.md keeps its greeting and contains one farewell line.\n\n## Open questions\n\nWhere should the farewell line go?\nRecommendation: at the end of README.md so it closes the page.\n"}}],"finish":"tool_calls","tokens":200},"I wrote the farewell draft beside us. Where should the farewell line go?\n\n1. At the end of README.md \u2014 it closes the page\n2. In its own file \u2014 the greeting stays alone\n3. Tell me something else",{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":2},"description":"Farewell line","body":"Put the farewell line at the end of README.md.\nNo open questions block the work.\n","name":2,"recalled":1,"first":15,"remove":2}}],"finish":"tool_calls","tokens":200},"I settled the location at the end of README.md. The draft is ready; nothing blocks the work."],"inputs":[{"cue":"Note 2 revision 2 written","turn":3},{"cue":"Note 2 revision 1 written","turn":1},{"cue":"At the end of README.md","turn":2},{"cue":"","turn":0}]"##;
    Ok(read::joined(&[b"{\"listen\":", &address(socket)?, b",\"key\":", &text(&path(data, b"llm.key"))?,
        b",\"added_keys\":[", &text(&path(data, b"signed-in.key"))?,
        br#"],"tls":null,"models":[{"name":"gpt-6.1-sol","window":1048576,"output":8192},{"name":"gpt-6-luna","window":1048576,"output":8192},{"name":"gpt-6-astra","window":1048576,"output":8192},{"name":"gpt-6.1-terra","window":1048576,"output":8192}],"cued":[{"cue":"Plan a farewell line for hello.","#,
        farewell,
        br#"},{"cue":"At the end of README.md","#,
        farewell,
        br###"},{"cue":"Choose the CSV export delimiter.","turns":[{"lines":[{"call":"message","arguments":{"target":2,"form":"question","words":"csv-export: comma or semicolon?\nChoice: Comma (recommended)\nChoice: Semicolon"}}],"finish":"tool_calls","tokens":32},{"lines":[{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":16},{"lines":[{"call":"finish","arguments":{"form":"report","text":"The CSV export delimiter is agreed.","fields":{}}}],"finish":"tool_calls","tokens":16}],"inputs":[{"cue":"Comma","turn":2},{"cue":"Semicolon","turn":2},{"cue":"Message ","turn":1},{"cue":"","turn":0}]},{ "cue":"Show drafts and goals for this project.","turns":[{"lines":[{"call":"propose_settings","arguments":{"against":1,"items":[{"setting":{"kind":"project-ways","project":1},"set":{"drafts":true,"goals":true,"changes":false,"way":0}}],"reason":"Show drafts and goals\ntemper enforces\nDrafts and goals are available in this project.\nyour agents follow\nKeep each change checked.\nRunning work stays on its held revision."}}],"finish":"tool_calls","tokens":64},{"lines":[{"text":"I proposed showing drafts and goals. Accept the proposal when you are ready."}],"finish":"stop","tokens":20}]},{"cue":"Write this up as a draft beside us.","turns":[{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":2},"description":"Away mode","body":"# Away mode\n\nDraft, 2026-11-05, with the user.\n\n## Purpose\n\nAway mode lets the person step away from ongoing work.\nAgents keep working within the authority already granted.\nThey leave a clear account of progress for the person's return.\n\nThe person chooses when to leave and when to return.\nAway mode does not grant extra authority.\n\n## When the person returns\n\nShow the work completed while the person was away.\nKeep the decisions that need the person in one place.\nMake each question name the goal and the choice it needs.\n\nThe person can answer questions before starting more work.\nThe conversation and its document remain available.\n\n## Open questions\n\n- Where should the person switch away mode on?\n- Should it apply to one project or every project?\n- How should urgent questions be shown on return?\n\nThese choices remain for the person to settle.\n"}}],"finish":"tool_calls","tokens":242},{"lines":[{"text":"I wrote an Away mode draft with its purpose, return flow and open questions."}],"finish":"stop","tokens":19}]},{"cue":"Add what happens to a running goal.","turns":[{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":2},"description":"Away mode","body":"# Away mode\n\nDraft, 2026-11-05, with the user.\n\n## Purpose\n\nAway mode lets the person step away from ongoing work.\nAgents keep working within the authority already granted.\nThey leave a clear account of progress for the person's return.\n\nThe person chooses when to leave and when to return.\nAway mode does not grant extra authority.\n\n## A running goal\n\nA running goal continues within its existing authority and budget.\nWhen it needs the person's decision, it waits with a clear question.\nOther work that can proceed independently keeps moving.\nAway mode never supplies an answer on the person's behalf.\n\n## When the person returns\n\nShow the work completed while the person was away.\nKeep the decisions that need the person in one place.\nMake each question name the goal and the choice it needs.\n\nThe person can answer questions before starting more work.\nThe conversation and its document remain available.\n\n## Open questions\n\n- Where should the person switch away mode on?\n- Should it apply to one project or every project?\n- How should urgent questions be shown on return?\n\nThese choices remain for the person to settle.\n","name":3,"recalled":1}}],"finish":"tool_calls","tokens":317},{"lines":[{"text":"I added how a running goal continues and waits for your decisions."}],"finish":"stop","tokens":17}]},{"cue":"walk-away-mode\n\nImplement this draft.","turns":[{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":2},"description":"report: The local goal is waiting for direction.","body":"The local goal keeps its draft and report.\n\nLeft: Wait for the person to pause, resume or stop it.\nRisk: This scripted model does not change a repository."}}],"finish":"tool_calls","tokens":64},{"lines":[{"call":"message","arguments":{"target":1,"form":"question","words":"May the local draft goal continue?\nChoice: Keep waiting\nChoice: Finish the demonstration"}}],"finish":"tool_calls","tokens":32},{"lines":[{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":16}],"inputs":[{"cue":"Note ","turn":1},{"cue":"Message ","turn":2},{"cue":"","turn":0}]},{ "cue":"away-mode\n\nImplement this draft.","turns":[{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":3},"description":"report: The local goal wrote its report.","body":"I wrote this report on the local fake box.\n\nLeft: Nothing remains in this demonstration.\nRisk: This scripted model does not change a repository."}}],"finish":"tool_calls","tokens":64},{"lines":[{"call":"finish","arguments":{"form":"report","text":"The local goal wrote its report.","fields":{}}}],"finish":"tool_calls","tokens":16}]},{ "cue":"You are a goal: the work your person asked for","turns":[{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":2},"description":"report: The local goal wrote its report.","body":"I wrote this report on the local fake box.\n\nLeft: Nothing remains in this demonstration.\nRisk: This scripted model does not change a repository."}}],"finish":"tool_calls","tokens":64},{"lines":[{"call":"finish","arguments":{"form":"report","text":"The local goal wrote its report.","fields":{}}}],"finish":"tool_calls","tokens":16}]},{"cue":"Walk the local away decision.","turns":[{"lines":[{"call":"decided","arguments":{"question":"Which local implementation?","options":["Small","Broad"],"took":0,"why":"The small implementation fits the design.","kind":"implementation"}}],"finish":"tool_calls","tokens":32},{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":2},"description":"report: I carried on with Small.","body":"I carried on with the small implementation before asking anyone."}}],"finish":"tool_calls","tokens":32},{"lines":[{"call":"message","arguments":{"target":1,"form":"question","words":"Which local implementation after overturn?\nChoice: Try the other implementation\nChoice: Keep the chosen implementation"}}],"finish":"tool_calls","tokens":32},{"lines":[{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":32},{"lines":[{"call":"finish","arguments":{"form":"report","text":"The person answered the local question.","fields":{}}}],"finish":"tool_calls","tokens":32},{"lines":[{"call":"message","arguments":{"target":1,"form":"question","words":"Which local implementation while here?\nChoice: Use the small implementation\nChoice: Use the broad implementation"}}],"finish":"tool_calls","tokens":32}],"inputs":[{"cue":"I overturn your decision","turn":2},{"cue":"Try the other implementation","turn":0},{"cue":"Use the small implementation","turn":4},{"cue":"Decided: carry on","turn":1},{"cue":"Ask: your person","turn":5},{"cue":"Message ","turn":3},{"cue":"Note ","turn":3},{"cue":"","turn":0}]},{"cue":"You write the few lines at the top of a project's home page","turns":[{"lines":[{"call":"finish","arguments":{"form":"report","text":"1 change landed on the local forge.\n\nThe README now records the local status change.","fields":{}}}],"finish":"tool_calls","tokens":32}]},{"cue":"## Task\n\nAdd a local status line to README.md.","turns":[{"lines":[{"call":"read","arguments":{"path":"README.md"}}],"finish":"tool_calls","tokens":16},{"lines":[{"call":"edit","arguments":{"path":"README.md","edits":[{"old":"# widget\n","new":"# widget\n\nA local status change landed.\n"}]}}],"finish":"tool_calls","tokens":32},{"lines":[{"call":"finish","arguments":{"form":"change","fields":{"title":"Record the local status change","body":"Add a status line to README.md."}}}],"finish":"tool_calls","tokens":16}]},{"cue":"## Task\n\nAdd a local hello status line to README.md.","turns":[{"lines":[{"call":"read","arguments":{"path":"README.md"}}],"finish":"tool_calls","tokens":16},{"lines":[{"call":"write","arguments":{"path":"README.md","content":"# hello\n\nA local status change landed.\n"}}],"finish":"tool_calls","tokens":32},{"lines":[{"call":"finish","arguments":{"form":"change","fields":{"title":"Record the local hello status change","body":"Add a status line to README.md."}}}],"finish":"tool_calls","tokens":16}]},{ "cue":"Review this change at the named head","turns":[{"lines":[{"call":"finish","arguments":{"form":"verdict","label":"1","text":"The local README change is ready.","fields":{},"items":[]}}],"finish":"tool_calls","tokens":16}]}],"asked":[{"cue":"Add a CSV export to the report","turns":[{"lines":[{"call":"goal","arguments":{"words":"Choose the CSV export delimiter."}}],"finish":"tool_calls","tokens":32},"A CSV export question waits for you."]},{"cue":"Keep only working notes for this chat.","turns":[{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":1},"description":"work: settings","body":"Internal working memory.","name":2,"recalled":2}}],"finish":"tool_calls","tokens":64},{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":1},"description":"work: page 0","body":"Internal working memory."}}],"finish":"tool_calls","tokens":64},{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":1},"description":"work: page 1","body":"Internal working memory."}}],"finish":"tool_calls","tokens":64},{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":1},"description":"work: page 2","body":"Internal working memory."}}],"finish":"tool_calls","tokens":64},{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":1},"description":"work: page 3","body":"Internal working memory."}}],"finish":"tool_calls","tokens":64},"Only working notes remain."]},{"cue":"Restore the Away mode document after the working notes.","turns":[{"lines":[{"call":"note","arguments":{"scope":{"kind":"goal","project":1,"goal":1},"description":"Away mode","body":"# Away mode\n\nDraft, 2026-11-05, with the user.\n\n## Purpose\n\nAway mode lets the person step away from ongoing work.\nAgents keep working within the authority already granted.\nThey leave a clear account of progress for the person's return.\n\nThe person chooses when to leave and when to return.\nAway mode does not grant extra authority.\n\n## A running goal\n\nA running goal continues within its existing authority and budget.\nWhen it needs the person's decision, it waits with a clear question.\nOther work that can proceed independently keeps moving.\nAway mode never supplies an answer on the person's behalf.\n\n## When the person returns\n\nShow the work completed while the person was away.\nKeep the decisions that need the person in one place.\nMake each question name the goal and the choice it needs.\n\nThe person can answer questions before starting more work.\nThe conversation and its document remain available.\n\n## Open questions\n\n- Where should the person switch away mode on?\n- Should it apply to one project or every project?\n- How should urgent questions be shown on return?\n\nThese choices remain for the person to settle.\n"}}],"finish":"tool_calls","tokens":64},"The document follows the working notes."]},{"cue":"Land a local hello status change.","turns":[{"lines":[{"call":"change","arguments":{"words":"Add a local hello status line to README.md.","repository":"work/hello"}}],"finish":"tool_calls","tokens":16},{"lines":[{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":8},"The local hello status change landed."]},{"cue":"Start a local away goal.","turns":[{"lines":[{"call":"goal","arguments":{"words":"Walk the local away decision."}}],"finish":"tool_calls","tokens":32},"I started the local away goal."]},{"cue":"Start a local goal and write its report.","turns":[{"lines":[{"call":"goal","arguments":{"words":"Write a report for this local demonstration."}}],"finish":"tool_calls","tokens":16},"I started a goal to write its report."]},{"cue":"Land a local status change.","turns":[{"lines":[{"call":"change","arguments":{"words":"Add a local status line to README.md.","repository":"work/widget"}}],"finish":"tool_calls","tokens":16},{"lines":[{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":8},"The local status change landed."]}],"otherwise":["Hello from the local fake model."]}"###]))
}
fn duplicates(d: &Document) -> Result<(), Refusal> {
    for at in 0..d.len() {
        if d.token(at).expect("document token").kind != Kind::ObjectStart {
            continue;
        }
        let p = Path::root();
        let mut object = read::object(d, at, &p)?;
        let end = config::after(d, at).or(Err(read::why(&p, b"invalid object")))?;
        let mut cursor = at.checked_add(1).expect("bounded token");
        for _ in at..end {
            if cursor == end.checked_sub(1).expect("object close") {
                break;
            }
            let key = d.token(cursor).expect("key token");
            let name = d.text(key).expect("key text");
            let _member = object.member(d, name);
            cursor = config::after(d, cursor.checked_add(1).expect("value token"))
                .or(Err(read::why(&p, b"invalid value")))?;
        }
        read::finish(object, d, &p)?;
    }
    Ok(())
}
fn registration(d: &Document, at: u32, p: &Path) -> Result<(), Refusal> {
    const SCOPES: &[&[u8]] =
        &[b"openid", b"profile", b"email", b"offline_access", b"resource.invoke", b"chatgpt.tokens.use.direct"];
    const MUST_GRANT: &[&[u8]] = &[b"chatgpt.tokens.use.direct"];
    for (name, expected) in [
        (b"client".as_slice(), b"dynamic_agent_client".as_slice()),
        (b"name_hint", b"temper"),
        (b"resource", b"https://api.openai.com/v1"),
        (b"redirect_path", b"auth/callback"),
    ] {
        let value = member(d, at, name, p)?;
        if read::text(d, value, 1024, &p.member(name))?.as_ref() != expected {
            return Err(read::why(&p.member(name), b"expected the shipped registration"));
        }
    }
    for (name, expected) in [(b"scopes".as_slice(), SCOPES), (b"must_grant", MUST_GRANT)] {
        let value = member(d, at, name, p)?;
        let values = read::list(d, value, 1, &p.member(name))?;
        if usize::try_from(values.len()).expect("bounded list") != expected.len() {
            return Err(read::why(&p.member(name), b"expected shipped scopes"));
        }
        for (value, wanted) in values.iter().zip(expected) {
            if read::text(d, *value, 1024, &p.member(name))?.as_ref() != *wanted {
                return Err(read::why(&p.member(name), b"expected shipped scopes"));
            }
        }
    }
    let port = member(d, at, b"redirect_port", p)?;
    if read::integer(d, port, u64::from(u16::MAX), &p.member(b"redirect_port"))? != 1455 {
        return Err(read::why(&p.member(b"redirect_port"), b"expected shipped callback 1455"));
    }
    Ok(())
}
#[cfg(test)]
mod tests;
