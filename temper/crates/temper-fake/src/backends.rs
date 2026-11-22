//! Bounded public-document rewrites for protocol/configuration/directories.md, 1. No files or sockets are opened here.
//! The fake box's steps follow temper/docs/design/protocol/setup/box-backends.md, 1.
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
/// Shared by the supervised welcome peers (protocol/setup/box-backends.md, 1).
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
        // protocol/setup/box-backends.md, 1: scripts and strict belong to this start's model alone.
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
    pub providers: Box<[u8]>,
    pub github: Box<[u8]>,
    pub oauth: Box<[u8]>,
    pub llm: Box<[u8]>,
    pub forgejo: Box<[u8]>,
}
const OUTPUT: Limits = Limits { depth: 32, length: crate::DOCUMENT_BYTES };
// The Model step must see OpenAI, demo and Anthropic in one provider offer.
const PROVIDER_LINES: u32 = 3;
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
            b"config" => b"/github.json".len(),
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
/// Wire welcome peers and a program model in ordinary documents. The shell validates and rebases imported resources first.
pub fn rewrite(temper: &[u8], paths: &Paths, peers: &Peers) -> Result<Documents, Refusal> {
    if temper.len() > usize::try_from(crate::DOCUMENT_BYTES).expect("document bound fits") {
        return Err(read::why(&Path::root().member(b"temper"), b"past its bound"));
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
    duplicates(&d)?;
    let root = Path::root();
    let application = member(&d, 0, b"application", &root)?;
    let ap = root.member(b"application");
    let git = path(&paths.data, b"demo/bin/git");
    let engine = member(&d, 0, b"engine", &root)?;
    if read::object(&d, engine, &root.member(b"engine"))?.member(&d, b"llm").is_some() {
        return Err(read::why(
            &root.member(b"engine").member(b"llm"),
            b"moved: LLM providers are settings; see them with temper ctl settings show provider",
        ));
    }
    let clients = member(&d, application, b"clients", &ap)?;
    capacity();
    Ok(Documents {
        providers: providers(peers)?,
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
            br##", "owner":"fake-person","made_ci":{"declared":".fake-forge/ci"},"repositories":[{"name":"widget","landing":"main","ci":{"declared":".fake-forge/ci"}}],
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
/// The sandbox's kept providers, constructed from the binary's defaults, never a document member.
pub fn providers(peers: &Peers) -> Result<Box<[u8]>, Refusal> {
    let oauth = socket(peers.oauth, &Path::root().member(b"oauth"))?;
    let _llm = socket(peers.llm, &Path::root().member(b"llm"))?;
    let mut providers = temper_engine_service_stock::providers();
    for value in &mut providers {
        if value.provider.number == 1 {
            let registration = value.provider.sign_in.as_mut().expect("stock sign-in");
            let origin = read::joined(&[b"http://", &oauth]);
            registration.issuer.clone_from(&origin);
            registration.authorization = read::joined(&[&origin, b"/api/accounts/authorize"]);
            registration.token = read::joined(&[&origin, b"/api/accounts/oauth/token"]);
            registration.revocation = Some(read::joined(&[&origin, b"/api/accounts/oauth/revoke"]));
            registration.keys = read::joined(&[&origin, b"/.well-known/jwks.json"]);
            registration.endpoint_transport = jig_protocol_config::Transport::Plaintext;
        } else if value.provider.number == 2 {
            value.provider.endpoint = Some(bytes::copy_of(b"fake-claude"));
            value.provider.direct = None;
            value.provider.keys = false;
        }
        let mut endpoints = List::uncapped();
        for endpoint in &value.endpoints {
            if endpoint.number == 4 {
                continue;
            }
            let mut endpoint = endpoint.clone();
            endpoint.address = jig_protocol_config::EndpointAddress::Socket(peers.llm);
            endpoint.transport = jig_protocol_config::Transport::Plaintext;
            if value.provider.number == 2 {
                endpoint.name = bytes::copy_of(b"fake-claude");
            }
            endpoints.push(endpoint).expect("stock endpoints");
        }
        value.endpoints = endpoints.into_boxed();
    }
    let mut measure = Encoder::measure(&OUTPUT);
    jig_protocol_config::llm::render_providers(&providers, &mut measure);
    let mut out = Encoder::write(measure.measured().expect("stock provider output"), &OUTPUT);
    jig_protocol_config::llm::render_providers(&providers, &mut out);
    Ok(out.finish())
}

fn capacity() {
    let stock = jig_protocol_config::stock();
    assert!(
        stock.core.documents.lines >= PROVIDER_LINES && stock.core.accounts.credentials >= 4,
        "the stock carries the fake backend provider offer and credentials"
    );
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
        br#", "owner":"fake-person","made_ci":{"declared":".fake-forge/ci"},"repositories":[{"name":"widget","landing":"main","ci":{"declared":".fake-forge/ci"}}],"github":{"app":{"id":1,"slug":"temper-fake","bot":2,"key":0,"setup_url":"#,&setup,br#"},"accounts":[{"login":"fake-person","id":3,"kind":"user","installation":{"id":1,"repositories":"all"}}]}}"#]))
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
#[cfg(test)]
mod tests;
