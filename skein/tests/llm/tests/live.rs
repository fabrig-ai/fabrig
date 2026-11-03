//! Real provider endpoints. Selected only by `cargo nextest run --profile live-skein`.
//! Credentials come from the files the caller names, read and never
//! written: no refreshes, retries, or alternate endpoints in this harness.
use std::io::{Read as _, Write as _};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::{CertificateDer, ServerName, pem::PemObject};
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use skein_json::{Document, Kind, document, tokenizer};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Env, Intake, Queue, Time, Token, Wall};
use skein_llm::{
    Block, Call, Completion, Credential, Delta, Endpoint, Failure, Json, Message, Prompt, Provider, Role, Stop, Tool,
    client,
};

// Environment input is intentional for this opt-in, nondeterministic suite.
#[expect(clippy::disallowed_methods, reason = "live credentials and profile selection come from the caller")]
fn setting(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.trim().is_empty())
}
fn enabled() -> bool {
    setting("NEXTEST_PROFILE").as_deref() == Some("live-skein")
}
/// A login the caller lends to the live run (skein/crates/skein-llm/README.md):
/// its variable names the login's file, or a directory holding it as `file`.
struct Login {
    variable: &'static str,
    file: &'static str,
}
/// Codex's `auth.json`, or a `CODEX_HOME` holding it.
const CODEX: Login = Login { variable: "SKEIN_TEST_LIVE_OPENAI_AUTH", file: "auth.json" };
/// A plaintext Anthropic key file, readable by its owner alone.
const ANTHROPIC_KEY: &str = "SKEIN_TEST_LIVE_ANTHROPIC_KEY";
/// The longest login file the harness reads: one is a few kilobytes.
const LOGIN_BYTES: u32 = 1 << 16;
/// A login's grammar: no string or number is longer than its file, and a
/// login nests a few containers deep. The chunk, the most one scan of a
/// string demands, only paces `parse`.
const LOGIN_JSON: tokenizer::Limits =
    tokenizer::Limits { depth: 8, string: LOGIN_BYTES, number: LOGIN_BYTES, chunk: 4096, length: LOGIN_BYTES };
/// Each record reads at least a byte of the file, and no text decodes longer than it is.
const LOGIN_RETAINED: document::Limits = document::Limits { tokens: LOGIN_BYTES, text: LOGIN_BYTES };
/// The strings at `fields` in the login its variable names. A missing
/// variable, file or field fails the live run, naming the variable, the file
/// and the field, never a value read from it.
fn lent<const N: usize>(login: &Login, fields: [&[&str]; N]) -> [Box<[u8]>; N] {
    let variable = login.variable;
    let named = lent_path(variable);
    login_fields(&named, login.file, fields).unwrap_or_else(|refusal| panic!("{variable}: {refusal}"))
}
fn lent_path(variable: &str) -> PathBuf {
    PathBuf::from(setting(variable).unwrap_or_else(|| panic!("live profile requires {variable}")))
}
fn lent_key() -> Box<[u8]> {
    key_file(&lent_path(ANTHROPIC_KEY)).unwrap_or_else(|refusal| panic!("{ANTHROPIC_KEY}: {refusal}"))
}
/// Read the owner's plaintext key once; refusals never include its bytes.
fn key_file(path: &Path) -> Result<Box<[u8]>, String> {
    use std::os::unix::fs::PermissionsExt as _;
    let file = std::fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let metadata = file.metadata().map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(format!("{}: key file must have mode 0600", path.display()));
    }
    let mut text = Vec::new();
    file.take(u64::from(LOGIN_BYTES) + 1)
        .read_to_end(&mut text)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if text.len() > usize::try_from(LOGIN_BYTES).expect("u32 fits usize") {
        return Err(format!("{}: key file is too large", path.display()));
    }
    let value = text.trim_ascii();
    if value.is_empty() {
        return Err(format!("{}: key file is empty", path.display()));
    }
    Ok(value.into())
}
/// The file a login's variable names: `named` itself, or `file` in the
/// directory `named`.
fn login_file(named: &Path, file: &str) -> PathBuf {
    if named.is_dir() { named.join(file) } else { named.to_path_buf() }
}
/// The non-empty strings at `fields` in a login's JSON, each field a path of
/// members from the root object. A refusal names the file and the first
/// field it lacks, never a value read from it.
fn login_fields<const N: usize>(named: &Path, file: &str, fields: [&[&str]; N]) -> Result<[Box<[u8]>; N], String> {
    let path = login_file(named, file);
    let text = std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let document = skein_json::parse(&text, &LOGIN_JSON, &LOGIN_RETAINED)
        .map_err(|error| format!("{}: not a login's JSON: {error:?}", path.display()))?;
    let mut values: [Box<[u8]>; N] = std::array::from_fn(|_| Box::default());
    for (value, field) in values.iter_mut().zip(fields) {
        *value = login_string(&document, field)
            .ok_or_else(|| format!("{}: no non-empty string at .{}", path.display(), field.join(".")))?;
    }
    Ok(values)
}
/// The decoded bytes of the string at `field` below the root object, or
/// nothing when a member is absent, is not a string, or is empty.
fn login_string(document: &Document, field: &[&str]) -> Option<Box<[u8]>> {
    let mut at = 0;
    for name in field {
        at = document.member(at, name.as_bytes())?;
    }
    let record = document.token(at).filter(|record| record.kind == Kind::String)?;
    document.text(record).filter(|text| !text.is_empty()).map(Box::from)
}
fn bytes(text: &str) -> Box<[u8]> {
    text.as_bytes().into()
}
fn text_message(text: &str) -> Message {
    Message { role: Role::User, content: Box::new([Block::Text { text: bytes(text), replay: None }]) }
}
fn live_limits() -> client::Limits {
    let mut limits = skein_llm_world::limits();
    limits.http.request = 16_384;
    limits.http.head = 32_768;
    limits.http.headers = 64;
    limits.http.read = 4096;
    limits.http.send = 4096;
    limits.skip = 262_144;
    limits.skip = 524_288;
    limits.sse.chunk = 4096;
    limits.request = 524_288;
    limits.retained = 524_288;
    limits.strings = 262_144;
    limits.tokens = 32_768;
    limits.output_items = 64;
    limits.input = 262_144;
    limits.reasoning = 262_144;
    limits.answer = 524_288;
    limits.error_bytes = 8192;
    limits
}
fn input(provider: Provider, owner: u64) -> Call {
    let (endpoint, credential, model, instructions, effort, max_tokens) = match provider {
        Provider::OpenAi => {
            let endpoint = Endpoint::openai();
            let [value] = lent(&CODEX, [&["tokens", "access_token"]]);
            (
                endpoint,
                Credential::bearer(value),
                setting("SKEIN_TEST_LIVE_OPENAI_MODEL").unwrap_or_else(|| "gpt-5.5".into()),
                bytes("Follow the user's instructions exactly. Be terse."),
                Some(bytes("low")),
                None,
            )
        }
        Provider::Anthropic => (
            Endpoint::anthropic(),
            Credential::key(lent_key()),
            setting("SKEIN_TEST_LIVE_ANTHROPIC_MODEL").unwrap_or_else(|| "claude-haiku-4-5".into()),
            bytes("Follow the user's instructions exactly. Be terse."),
            None,
            Some(128),
        ),
    };
    Call {
        owner: Token::new(owner),
        endpoint,
        credential: Some(credential),
        prompt: Prompt {
            model: model.into_bytes().into(),
            instructions,
            tools: Box::new([]),
            messages: Box::new([text_message("Reply with exactly: skein-live-ok")]),
            reasoning_effort: effort,
            affinity: None,
            choice: skein_llm::ToolChoice::Auto,
            max_output_tokens: max_tokens,
        },
    }
}

/// Only transports bytes: all request encoding, HTTP framing, SSE decoding,
/// demand management and provider translation run through the real Client.
struct Connection {
    machine: client::Client,
    env: Env<client::Limits>,
    wire: StreamOwned<ClientConnection, TcpStream>,
    intake: Intake,
    above: Queue<client::Event>,
    below: Queue<Down>,
    demand: Option<(Read, u32)>,
    grant: u32,
}
impl Connection {
    fn new(call: Call) -> Self {
        let limits = live_limits();
        let host = String::from_utf8(call.endpoint.authority.to_vec()).expect("provider authority is UTF-8");
        let machine = client::Client::prepare(call, &limits).expect("live prompt admitted before connecting");
        let ca_file = setting("SKEIN_TEST_LIVE_CA_FILE").unwrap_or_else(|| "/etc/ssl/certs/ca-certificates.crt".into());
        let mut roots = RootCertStore::empty();
        for cert in CertificateDer::pem_file_iter(ca_file).expect("open system CA bundle") {
            roots.add(cert.expect("PEM certificate")).expect("valid trust anchor");
        }
        assert!(!roots.is_empty(), "live TLS needs trusted roots");
        let mut config = ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .expect("ring supports default TLS versions")
            .with_root_certificates(roots)
            .with_no_client_auth();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        let tls =
            ClientConnection::new(Arc::new(config), ServerName::try_from(host.clone()).expect("provider DNS name"))
                .expect("create verified TLS client");
        let mut socket = None;
        for address in (host.as_str(), 443).to_socket_addrs().expect("resolve provider") {
            if let Ok(connected) = TcpStream::connect_timeout(&address, Duration::from_secs(20)) {
                socket = Some(connected);
                break;
            }
        }
        let socket = socket.expect("connect to real provider on port 443");
        socket.set_read_timeout(Some(Duration::from_mins(1))).expect("set socket read timeout");
        socket.set_write_timeout(Some(Duration::from_mins(1))).expect("set socket write timeout");
        Self {
            machine,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            wire: StreamOwned::new(tls, socket),
            intake: Intake::with_capacity(skein_http::client::largest_read(&limits.http)),
            above: Queue::uncapped(),
            below: Queue::uncapped(),
            demand: None,
            grant: 0,
        }
    }
    fn request(&mut self, request: client::Request) {
        client::down(&mut self.machine, &self.env, request, &mut self.above, &mut self.below);
    }
    fn exchange(&mut self) -> Vec<client::Event> {
        let mut events = Vec::new();
        self.request(client::Request::Start);
        for _ in 0..1_000_000 {
            let mut settled = false;
            while let Some(event) = self.above.pop() {
                match &event {
                    client::Event::Close => {
                        self.wire.sock.shutdown(Shutdown::Both).expect("close real socket");
                        client::closed(&mut self.machine, &self.env, &mut self.above, &mut self.below);
                    }
                    client::Event::Reusable | client::Event::Closed => settled = true,
                    client::Event::Delta { owner, .. }
                    | client::Event::Block { owner, .. }
                    | client::Event::Completed { owner, .. }
                    | client::Event::Failed { owner, .. }
                    | client::Event::Cancelled { owner } => assert_eq!(*owner, self.machine.owner()),
                }
                events.push(event);
            }
            while let Some(request) = self.below.pop() {
                match request {
                    Down::Demand { read: Read::Nothing, room: 0 } => self.demand = None,
                    Down::Demand { read, room } => {
                        assert!(self.demand.is_none(), "answer the previous demand first");
                        assert_eq!(self.grant, 0, "spend the previous send grant first");
                        self.demand = Some((read, room));
                    }
                    Down::Send(data) => {
                        assert!(data.len() <= self.grant as usize && self.grant > 0, "send within grant");
                        self.grant = 0;
                        self.wire.write_all(&data).expect("send provider request over TLS");
                        self.wire.flush().expect("flush request over TLS");
                    }
                    Down::Finish => panic!("HTTP keeps its transport reusable"),
                }
            }
            if settled {
                return events;
            }
            if self.machine.has_work() {
                client::resume(&mut self.machine, &self.env, &mut self.above, &mut self.below);
                continue;
            }
            if self.machine.waiting() == client::Waiting::Next {
                self.request(client::Request::Next);
                continue;
            }
            let (read, room) = self.demand.expect("unsettled call must demand transport progress");
            if room > 0 {
                self.demand = None;
                self.grant = room;
                client::up(&mut self.machine, &self.env, Up::Room, &mut self.above, &mut self.below);
            } else if let Some(data) = self.intake.meet(read) {
                self.demand = None;
                client::up(&mut self.machine, &self.env, Up::Bytes(data), &mut self.above, &mut self.below);
            } else {
                assert_ne!(read, Read::Nothing, "a read is required for progress");
                let mut buffer = vec![0; self.intake.room() as usize];
                assert!(!buffer.is_empty(), "intake can meet the client's demand");
                let count = self.wire.read(&mut buffer).expect("read provider response over verified TLS");
                if count == 0 {
                    client::up(&mut self.machine, &self.env, Up::End, &mut self.above, &mut self.below);
                } else {
                    self.intake.append(&buffer[..count]).expect("socket bytes fit intake room");
                }
            }
        }
        panic!("live client failed to settle within the step budget");
    }
    fn next(&mut self, call: Call) -> Vec<client::Event> {
        let prepared = client::Client::prepare(call, &self.env.limits).expect("replayed prompt admitted");
        assert!(self.machine.next_call(prepared).is_ok(), "reuse after the real HTTP body drains");
        self.exchange()
    }
    fn close(&mut self) {
        self.request(client::Request::Close);
        assert_eq!(self.exchange(), [client::Event::Close, client::Event::Closed], "settle the real transport");
        assert_eq!(self.machine.waiting(), client::Waiting::Nothing);
    }
}
fn completion(events: &[client::Event]) -> Completion {
    let terminals: Vec<_> = events
        .iter()
        .filter(|event| {
            matches!(
                event,
                client::Event::Completed { .. } | client::Event::Failed { .. } | client::Event::Cancelled { .. }
            )
        })
        .collect();
    assert_eq!(terminals.len(), 1, "one terminal event: {events:?}");
    let completion = match terminals[0] {
        client::Event::Completed { completion, .. } => completion,
        client::Event::Failed { failure, evidence, detail, .. } => {
            panic!("provider call failed: {failure:?}, {evidence:?}: {}", String::from_utf8_lossy(detail));
        }
        client::Event::Cancelled { .. } => panic!("live call was unexpectedly cancelled"),
        client::Event::Delta { .. }
        | client::Event::Block { .. }
        | client::Event::Reusable
        | client::Event::Close
        | client::Event::Closed => unreachable!("filtered to terminal events"),
    };
    let blocks: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            client::Event::Block { block, .. } => Some(block.clone()),
            client::Event::Delta { .. }
            | client::Event::Completed { .. }
            | client::Event::Failed { .. }
            | client::Event::Cancelled { .. }
            | client::Event::Reusable
            | client::Event::Close
            | client::Event::Closed => None,
        })
        .collect();
    assert_eq!(blocks.as_slice(), completion.content.as_ref(), "completed blocks preserve stream order");
    assert!(
        completion.usage.input.is_some_and(|count| count > 0) && completion.usage.output.is_some_and(|count| count > 0),
        "real token usage"
    );
    assert_eq!(events.iter().filter(|event| matches!(event, client::Event::Reusable)).count(), 1);
    completion.clone()
}
fn assert_text(events: &[client::Event], expected: &str) -> Completion {
    let done = completion(events);
    assert_eq!(done.stop, Stop::EndTurn);
    let final_text: Vec<u8> = done
        .content
        .iter()
        .flat_map(|block| match block {
            Block::Text { text, .. } => text.to_vec(),
            Block::Refusal { .. }
            | Block::ToolCall { .. }
            | Block::ToolResult { .. }
            | Block::Reasoning { .. }
            | Block::Oversize { .. }
            | Block::Cut { .. }
            | Block::Dropped { .. } => Vec::new(),
        })
        .collect();
    let deltas: Vec<u8> = events
        .iter()
        .flat_map(|event| match event {
            client::Event::Delta { delta: Delta::Text { text, .. }, .. } => text.to_vec(),
            client::Event::Delta { .. }
            | client::Event::Block { .. }
            | client::Event::Completed { .. }
            | client::Event::Failed { .. }
            | client::Event::Cancelled { .. }
            | client::Event::Reusable
            | client::Event::Close
            | client::Event::Closed => Vec::new(),
        })
        .collect();
    assert!(!deltas.is_empty(), "text arrives incrementally");
    assert_eq!(deltas, final_text, "streamed text equals completed text");
    assert_eq!(String::from_utf8(final_text).expect("provider text is UTF-8").trim(), expected);
    done
}
fn text_and_history(provider: Provider) {
    if !enabled() {
        return;
    }
    let first = input(provider, 1);
    let mut history = first.prompt.messages.to_vec();
    let mut connection = Connection::new(first);
    let done = assert_text(&connection.exchange(), "skein-live-ok");
    history.push(Message { role: Role::Assistant, content: done.content });
    history.push(text_message("What exact string did you just reply with? Reply with only that string."));
    let mut second = input(provider, 2);
    second.prompt.messages = history.into();
    assert_text(&connection.next(second), "skein-live-ok");
    connection.close();
}
fn tool_round_trip(provider: Provider) {
    if !enabled() {
        return;
    }
    let mut first = input(provider, 1);
    first.prompt.messages = Box::new([text_message(
        "Call skein_lookup once with key='live'. After its result, reply with only the exact returned value. Do not invent a value or call another tool.",
    )]);
    first.prompt.tools = Box::new([Tool {
        name: bytes("skein_lookup"),
        description: bytes("Returns the secret test value for a key. You must call this to discover the value."),
        schema: Json::from_bytes(br#"{"type":"object","properties":{"key":{"type":"string","enum":["live"]}},"required":["key"],"additionalProperties":false}"#, &(live_limits().native()).document()).expect("valid tool schema or arguments"),
    }]);
    let tools = first.prompt.tools.clone();
    let mut history = first.prompt.messages.to_vec();
    let mut connection = Connection::new(first);
    let events = connection.exchange();
    let done = completion(&events);
    assert_eq!(done.stop, Stop::ToolUse);
    let calls: Vec<_> = done
        .content
        .iter()
        .filter_map(|block| match block {
            Block::ToolCall { id, name, arguments, .. } => Some((id, name, arguments)),
            Block::Text { .. }
            | Block::Refusal { .. }
            | Block::ToolResult { .. }
            | Block::Reasoning { .. }
            | Block::Oversize { .. }
            | Block::Cut { .. }
            | Block::Dropped { .. } => None,
        })
        .collect();
    assert_eq!(calls.len(), 1, "one requested tool call");
    let (id, name, arguments) = calls[0];
    assert!(!id.is_empty());
    assert_eq!(name.as_ref(), b"skein_lookup");
    let parsed = Json::from_bytes(arguments, &(live_limits().native()).document()).expect("complete tool JSON");
    let expected = Json::from_bytes(br#"{"key":"live"}"#, &(live_limits().native()).document())
        .expect("valid tool schema or arguments");
    assert_eq!(parsed, expected);
    let argument_deltas: Vec<u8> = events
        .iter()
        .flat_map(|event| match event {
            client::Event::Delta { delta: Delta::ToolArguments { delta, .. }, .. } => delta.to_vec(),
            client::Event::Delta { .. }
            | client::Event::Block { .. }
            | client::Event::Completed { .. }
            | client::Event::Failed { .. }
            | client::Event::Cancelled { .. }
            | client::Event::Reusable
            | client::Event::Close
            | client::Event::Closed => Vec::new(),
        })
        .collect();
    assert_eq!(argument_deltas, arguments.as_ref(), "argument deltas assemble into final JSON");
    let result = Block::ToolResult { id: id.clone(), text: bytes("skein-tool-result-7319"), is_error: false };
    history.push(Message { role: Role::Assistant, content: done.content });
    history.push(Message { role: Role::User, content: Box::new([result]) });
    let mut second = input(provider, 2);
    second.prompt.tools = tools;
    second.prompt.messages = history.into();
    assert_text(&connection.next(second), "skein-tool-result-7319");
    connection.close();
}
fn unauthorized(provider: Provider) {
    if !enabled() {
        return;
    }
    let mut call = input(provider, 1);
    call.credential.as_mut().expect("fixture credential").value = bytes("skein-live-intentionally-invalid-token");
    let mut connection = Connection::new(call);
    let events = connection.exchange();
    let failures: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            client::Event::Failed { failure, evidence, .. } => Some((*failure, *evidence)),
            client::Event::Delta { .. }
            | client::Event::Block { .. }
            | client::Event::Completed { .. }
            | client::Event::Cancelled { .. }
            | client::Event::Reusable
            | client::Event::Close
            | client::Event::Closed => None,
        })
        .collect();
    assert_eq!(
        failures,
        [(Failure::Unauthorized, client::Evidence::Response { status: 401 })],
        "real auth failure: {events:?}"
    );
    assert!(!events.iter().any(|event| matches!(event, client::Event::Completed { .. })));
}
#[test]
fn openai_streamed_text_and_history() {
    text_and_history(Provider::OpenAi);
}
#[test]
fn anthropic_streamed_text_and_history() {
    text_and_history(Provider::Anthropic);
}
#[test]
fn openai_tool_round_trip() {
    tool_round_trip(Provider::OpenAi);
}
#[test]
fn anthropic_tool_round_trip() {
    tool_round_trip(Provider::Anthropic);
}
#[test]
fn openai_unauthorized() {
    unauthorized(Provider::OpenAi);
}
#[test]
fn anthropic_unauthorized() {
    unauthorized(Provider::Anthropic);
}

/// Synthetic: a Codex `auth.json`'s fields, its values fixtures.
const CODEX_AUTH: &[u8] = br#"{"OPENAI_API_KEY":null,"tokens":{"id_token":"fixture-id","access_token":"fixture-responses-access","refresh_token":"fixture-responses-refresh"},"last_refresh":"2026-11-01T00:00:00Z"}"#;
/// Offline: a login's field read from the file its variable names, or from
/// the directory holding that file.
#[test]
fn a_login_is_read_from_its_file_or_the_directory_holding_it() {
    let scratch = skein_scratch::Scratch::new("live-logins");
    std::fs::write(scratch.path().join(CODEX.file), CODEX_AUTH).expect("write the Codex login");
    let rows: [(&str, &str, &[&str], &[u8]); 2] = [
        (CODEX.file, CODEX.file, &["tokens", "access_token"], b"fixture-responses-access"),
        (".", CODEX.file, &["tokens", "access_token"], b"fixture-responses-access"),
    ];
    for (named, file, field, expected) in rows {
        let [value] = login_fields(&scratch.path().join(named), file, [field]).expect("a lent login");
        assert_eq!(&*value, expected, "{file} named as {named}");
    }
}
/// Offline: the refusal names the file and the field, and no value the file holds.
#[test]
fn a_login_lacking_a_field_is_refused_naming_only_the_field() {
    let rows: [&[u8]; 3] =
        [br#"{"tokens":{}}"#, br#"{"tokens":{"access_token":null}}"#, br#"{"tokens":{"access_token":""}}"#];
    for contents in rows {
        let scratch = skein_scratch::Scratch::new("live-login");
        let path = scratch.path().join(CODEX.file);
        std::fs::write(&path, contents).expect("write the Codex login");
        let refusal = login_fields(scratch.path(), CODEX.file, [&["tokens", "access_token"]])
            .expect_err("a login without its access token");
        assert_eq!(refusal, format!("{}: no non-empty string at .tokens.access_token", path.display()));
    }
}

/// Offline: a synthetic key is read as plaintext only from a private file.
#[test]
fn an_anthropic_key_file_requires_private_nonempty_plaintext() {
    use std::os::unix::fs::PermissionsExt as _;
    let scratch = skein_scratch::Scratch::new("live-key");
    let path = scratch.path().join("key");
    for (contents, mode, refusal) in [
        (b"fixture-key\n".as_slice(), 0o600, None),
        (b"fixture-key".as_slice(), 0o644, Some("key file must have mode 0600")),
        (b" \n".as_slice(), 0o600, Some("key file is empty")),
    ] {
        std::fs::write(&path, contents).expect("write synthetic key");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("set synthetic key mode");
        match refusal {
            Some(refusal) => assert_eq!(key_file(&path).err(), Some(format!("{}: {refusal}", path.display()))),
            None => assert_eq!(key_file(&path).expect("private key file").as_ref(), b"fixture-key"),
        }
    }
}
