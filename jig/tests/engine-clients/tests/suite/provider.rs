#![expect(
    clippy::disallowed_types,
    reason = "the task requires real elapsed measurements; protocol deadlines use only the injected world clock"
)]
//! The application's page and endpoint routes around the real clients' component (protocol/client/openid.md).
use jig_client_protocol_wire::v28 as w;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_engine_clients_world::fixtures;
use jig_protocol_config::clients::{Provider, ProviderClient, ProviderUrls, provider_issuer};
use jig_protocol_engine_clients::{
    Component, Event, FromTransport, Kept, Message, ProviderAnswer, ProviderClientChange, ProviderClientRefusal, Reply,
    Request, SecretOwner, SecretsAnswer, SecretsRequest, ToApplication,
};
use serde_json::{Value, json};
use skein_lib::{
    Duration, Env, Queue, Reader, ReplyTo, Secret, Time, Token, Wall, Writer,
    base64::{self, Alphabet},
};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    process::Command,
    sync::OnceLock,
    time::Instant,
};
const REDIRECT: &str = "https://app.example.com/callback";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
const PERSON_PROVIDER: u16 = 3;
// Two code slots, and two token slots: the story reaches both admission bounds independently.
const ROOM: u32 = 2;
struct Keys {
    pem: Secret,
    secret: Secret,
}
fn keys() -> &'static Keys {
    static KEYS: OnceLock<Keys> = OnceLock::new();
    KEYS.get_or_init(|| {
        let scratch = tempfile::tempdir().expect("private provider scratch");
        let path = scratch.path().join("provider.pem");
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).expect("private key file");
        let started = Instant::now();
        let result = Command::new("openssl")
            .args(["genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:3072", "-out"])
            .arg(&path)
            .output()
            .expect("openssl installed");
        assert!(result.status.success(), "key generation succeeds");
        eprintln!("provider world RSA 3072 key generation: {:?}", started.elapsed());
        assert_eq!(fs::metadata(&path).expect("key metadata").permissions().mode() & 0o777, 0o600);
        let pem = Secret::new(fs::read(path).expect("key file").into_boxed_slice());
        let path = scratch.path().join("client.secret");
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).expect("private secret file");
        let result = Command::new("openssl")
            .args(["rand", "-hex", "-out"])
            .arg(&path)
            .arg("32")
            .output()
            .expect("openssl installed");
        assert!(result.status.success(), "client secret generation succeeds");
        let secret = jig_protocol_config::secret(fs::read(path).expect("client secret file").into_boxed_slice());
        Keys { pem, secret }
    })
}
fn urls() -> ProviderUrls {
    ProviderUrls {
        issuer: Box::from(b"https://app.example.com".as_slice()),
        public_origin: Box::from(b"https://app.example.com".as_slice()),
        authorization_endpoint: Box::from(b"https://app.example.com/authorize".as_slice()),
        token_endpoint: Box::from(b"https://app.example.com/token".as_slice()),
        userinfo_endpoint: Box::from(b"https://app.example.com/userinfo".as_slice()),
        jwks_uri: Box::from(b"https://app.example.com/keys".as_slice()),
    }
}
struct World {
    component: Component,
    env: Env<jig_protocol_engine_clients::Limits>,
    out: Queue<Request>,
    kept: BTreeMap<SecretOwner, Kept>,
    signer: Option<people::Signer>,
    delay_whose: bool,
    delay_credential: bool,
    credential_read: Option<SecretsAnswer>,
    whose: Option<ReplyTo>,
    credential: Option<Secret>,
    reply: Option<Reply>,
    endpoint: Option<ProviderAnswer>,
    changed: Option<Result<(), ProviderClientRefusal>>,
    next: u64,
    key_set: Value,
    secret: Secret,
}
impl World {
    fn new(provider: bool) -> Self {
        Self::build(
            provider,
            Box::new([ProviderClient {
                id: Box::from(b"app".as_slice()),
                secret: Some(Box::from(b"client.secret".as_slice())),
                redirect_uri: REDIRECT.as_bytes().into(),
            }]),
            ROOM,
        )
    }
    fn empty() -> Self {
        Self::build(true, Box::new([]), 4)
    }
    fn build(provider: bool, clients: Box<[ProviderClient]>, room: u32) -> Self {
        let mut limits = fixtures::limits();
        limits.provider_codes = room;
        limits.provider_tokens = room;
        let mut settings = fixtures::settings();
        settings.link = None;
        settings.addresses = Box::new([PERSON_PROVIDER]);
        settings.providers = skein_lib::List::with_capacity(1);
        settings.providers.push(PERSON_PROVIDER).expect("application provider room");
        let key_set = if provider {
            let configuration =
                Provider { key: Box::from(b"provider.pem".as_slice()), clients, addresses: settings.addresses.clone() };
            let issuer = provider_issuer(
                &configuration,
                &[
                    (configuration.key.clone(), keys().pem.clone()),
                    (Box::from(b"client.secret".as_slice()), keys().secret.clone()),
                ],
                urls(),
                &limits,
            )
            .expect("startup builds issuer");
            let discovery: Value = serde_json::from_slice(&issuer.discovery()).expect("discovery JSON");
            assert_eq!(discovery["issuer"], "https://app.example.com");
            let key_set = serde_json::from_slice(&issuer.key_set()).expect("key set JSON");
            settings.provider = Some(issuer);
            key_set
        } else {
            Value::Null
        };
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut out = Queue::with_capacity(64);
        let component = Component::new(settings, &env, &mut out).expect("component starts");
        Self {
            component,
            env,
            out,
            kept: BTreeMap::new(),
            signer: None,
            delay_whose: false,
            delay_credential: false,
            credential_read: None,
            whose: None,
            credential: None,
            reply: None,
            endpoint: None,
            changed: None,
            next: 1,
            key_set,
            secret: keys().secret.clone(),
        }
    }
    fn send(&mut self, event: Event) {
        self.component.down(&self.env, event, &mut self.out);
        for _ in 0..256 {
            let active = self.component.up(&self.env, &mut self.out);
            let Some(request) = self.out.pop() else {
                if !active {
                    return;
                }
                continue;
            };
            match request {
                Request::Core(event) => {
                    if let core::Event::SignIn { reply_to, identity, .. } = *event {
                        self.signer = Some(people::Signer { identity, kind: people::Kind::Person });
                        self.component.down(
                            &self.env,
                            Event::Reply {
                                to: reply_to,
                                reply: people::Reply::SignedIn {
                                    person: 1,
                                    sign_in: 7,
                                    expires: Wall::from_nanos(u64::MAX),
                                },
                            },
                            &mut self.out,
                        );
                    } else {
                        let core::Event::People(people::Event::Whose { reply_to, .. }) = *event else {
                            panic!("unexpected core event");
                        };
                        if self.delay_whose {
                            assert!(self.whose.replace(reply_to).is_none());
                        } else {
                            self.component.down(
                                &self.env,
                                Event::Reply { to: reply_to, reply: people::Reply::Signer(self.signer.clone()) },
                                &mut self.out,
                            );
                        }
                    }
                }
                Request::Secrets(request) => {
                    let answer = match request {
                        SecretsRequest::Keep { owner, value } => {
                            self.kept.insert(owner, value);
                            SecretsAnswer::Kept { owner }
                        }
                        SecretsRequest::Get { owner } => {
                            SecretsAnswer::Got { owner, value: self.kept.get(&owner).cloned() }
                        }
                        SecretsRequest::Erase { owner } => {
                            self.kept.remove(&owner);
                            SecretsAnswer::Erased { owner }
                        }
                        SecretsRequest::Hold { .. } | SecretsRequest::Carry { .. } => panic!("no carrying request"),
                    };
                    if self.delay_credential
                        && matches!(&answer, SecretsAnswer::Got { owner: SecretOwner::SignIn(_), .. })
                    {
                        assert!(self.credential_read.replace(answer).is_none());
                    } else {
                        self.component.down(&self.env, Event::Secrets(answer), &mut self.out);
                    }
                }
                Request::Transport(jig_protocol_engine_clients::ToTransport::Reply { reply, .. }) => {
                    self.reply = Some(reply);
                }
                Request::Redeemed { exchange, answer } | Request::Claimed { exchange, answer } => {
                    assert_eq!(exchange, Token::new(99));
                    self.endpoint = Some(answer);
                }
                Request::Minted { credential, .. } => self.credential = Some(credential),
                Request::Application(ToApplication::ProviderClientChanged { reply, result }) => {
                    assert_eq!(reply.into_token(), Token::new(97));
                    assert!(self.changed.replace(result).is_none(), "one answer per change");
                }
                Request::Application(_)
                | Request::Link { .. }
                | Request::LocalLink { .. }
                | Request::StreamFailed { .. }
                | Request::LinkSignedIn
                | Request::TakeMoved { .. }
                | Request::MintRefused { .. }
                | Request::Closed
                | Request::Transport(_) => panic!("unexpected provider story output"),
            }
        }
        panic!("provider did not settle");
    }
    fn change(&mut self, change: ProviderClientChange) -> Result<(), ProviderClientRefusal> {
        self.send(Event::ProviderClient { reply: ReplyTo::new(Token::new(97)), change });
        self.changed.take().expect("registration request answered")
    }
    fn sign_in(&mut self) {
        self.send(Event::Mint {
            exchange: Token::new(98),
            identity: people::Identity {
                key: people::IdentityKey {
                    provider: PERSON_PROVIDER,
                    subject: Box::from(b"alice@example.com".as_slice()),
                },
                login: Box::from(b"alice".as_slice()),
                name: Box::from(b"Alice".as_slice()),
            },
        });
    }
    fn authorize(&mut self, redirect: &str) -> w::AuthorizeAnswer {
        self.authorize_client("app", redirect)
    }
    fn authorize_client(&mut self, client: &str, redirect: &str) -> w::AuthorizeAnswer {
        let query = format!(
            "client_id={client}&redirect_uri={redirect}&response_type=code&scope=openid&code_challenge_method=S256&code_challenge={CHALLENGE}&state=back&nonce=once"
        );
        self.query(query.as_bytes())
    }
    fn query(&mut self, query: &[u8]) -> w::AuthorizeAnswer {
        self.begin_query(query);
        self.authorization_answer()
    }
    fn begin_query(&mut self, query: &[u8]) {
        let record =
            w::Authorize::new(&w::CEILINGS, w::AuthorizeParts { query: query.into() }).expect("bounded authorization");
        let mut bytes = Writer::new(skein_lib::length::usize_of(record.measure()));
        record.encode(&mut bytes).expect("measured authorization");
        let exchange = Token::new(self.next);
        self.next += 1;
        self.send(Event::Transport(FromTransport::Exchange {
            browser: false,
            exchange,
            credential: self.credential.clone(),
            message: Message::Authorize(bytes.finish()),
        }));
    }
    fn authorization_answer(&mut self) -> w::AuthorizeAnswer {
        let Reply::Record(body) = self.reply.take().expect("authorization answered") else {
            panic!("record answer");
        };
        w::AuthorizeAnswer::decode(&w::CEILINGS, &mut Reader::new(&body)).expect("authorization answer")
    }
    fn code(&mut self) -> Secret {
        self.code_for("app")
    }
    fn code_for(&mut self, client: &str) -> Secret {
        let w::AuthorizeAnswer::Go(go) = self.authorize_client(client, REDIRECT) else {
            panic!("authorization granted");
        };
        let location = std::str::from_utf8(go.location()).expect("redirect UTF-8");
        assert!(location.ends_with("&state=back"));
        let code = location.split("code=").nth(1).expect("code parameter").split('&').next().expect("code");
        assert_eq!(code.len(), 43);
        Secret::new(code.as_bytes().into())
    }
    fn redeem(
        &mut self,
        code: &Secret,
        redirect: &str,
        verifier: &str,
        header: bool,
        correct_secret: bool,
    ) -> ProviderAnswer {
        let mut secret = self.secret.expose().to_vec();
        if !correct_secret {
            secret[0] ^= 1;
        }
        let credentials = if header {
            String::new()
        } else {
            format!("&client_id=app&client_secret={}", std::str::from_utf8(&secret).expect("hex secret"))
        };
        let body = Secret::new(
            format!(
                "grant_type=authorization_code&code={}&redirect_uri={redirect}&code_verifier={verifier}{credentials}",
                std::str::from_utf8(code.expose()).expect("code UTF-8")
            )
            .into_bytes()
            .into_boxed_slice(),
        );
        let authorization = header.then(|| {
            let credentials = [b"app:".as_slice(), secret.as_slice()].concat();
            let encoded = encode(Alphabet::Standard, &credentials);
            Secret::new([b"Basic ".as_slice(), encoded.as_ref()].concat().into_boxed_slice())
        });
        self.token_request(authorization, body)
    }
    fn redeem_public(&mut self, client: &str, code: &Secret) -> ProviderAnswer {
        let body = Secret::new(format!(
            "grant_type=authorization_code&client_id={client}&code={}&redirect_uri={REDIRECT}&code_verifier={VERIFIER}",
            std::str::from_utf8(code.expose()).expect("code UTF-8")
        ).into_bytes().into_boxed_slice());
        self.token_request(None, body)
    }
    fn token_request(&mut self, authorization: Option<Secret>, body: Secret) -> ProviderAnswer {
        self.send(Event::Redeem {
            exchange: Token::new(99),
            authorization,
            content_type: Some(Box::from(b"application/x-www-form-urlencoded".as_slice())),
            body,
        });
        self.endpoint.take().expect("token answer")
    }
    fn claims(&mut self, token: &str) -> ProviderAnswer {
        self.send(Event::Claims {
            exchange: Token::new(99),
            authorization: Some(Secret::new(format!("Bearer {token}").into_bytes().into_boxed_slice())),
        });
        self.endpoint.take().expect("userinfo answer")
    }
    fn advance(&mut self, seconds: u64) {
        let span = Duration::from_secs(seconds);
        self.env.now = self.env.now.saturating_add(span);
        self.env.wall = Wall::from_nanos(self.env.wall.as_nanos().saturating_add(span.as_nanos()));
    }
}
fn encode(alphabet: Alphabet, bytes: &[u8]) -> Box<[u8]> {
    let length = base64::encoded_len(alphabet, u32::try_from(bytes.len()).expect("short bytes")).expect("encoded size");
    let mut writer = Writer::new(skein_lib::length::usize_of(length));
    base64::encode(alphabet, bytes, &mut writer).expect("measured base64");
    writer.finish()
}
fn decode(value: &str) -> Box<[u8]> {
    base64::decode(Alphabet::Url, value.as_bytes(), 8192).expect("base64url")
}
fn error(answer: &ProviderAnswer, expected: &str) {
    let body: Value = serde_json::from_slice(answer.body.expose()).expect("error JSON");
    assert_eq!(body["error"].as_str(), Some(expected));
}
fn invalid_grant(answer: &ProviderAnswer) {
    assert_eq!(answer.status, 400);
    error(answer, "invalid_grant");
}
fn refused(answer: w::AuthorizeAnswer, why: &w::AuthorizeWhy) {
    let w::AuthorizeAnswer::Refused(found) = answer else {
        panic!("authorization refused");
    };
    assert_eq!(found.why(), why);
}
#[test]
fn provider_codes_tokens_and_claims_follow_one_signed_in_person() {
    keys();
    let started = Instant::now();
    let mut world = World::new(true);
    refused(world.authorize("https://other.example.com/callback"), &w::AuthorizeWhy::UnknownRedirect);
    refused(world.query(b"%invalid"), &w::AuthorizeWhy::Malformed);
    let w::AuthorizeAnswer::Go(back) = world.query(format!("client_id=app&redirect_uri={REDIRECT}").as_bytes()) else {
        panic!("issuer error sent back");
    };
    assert!(back.location().windows(b"error=invalid_request".len()).any(|part| part == b"error=invalid_request"));
    assert_eq!(world.authorize(REDIRECT), w::AuthorizeAnswer::SignedOut);
    world.sign_in();
    let code = world.code();
    let answer = world.redeem(&code, REDIRECT, VERIFIER, false, true);
    assert_eq!(answer.status, 200);
    let tokens: Value = serde_json::from_slice(answer.body.expose()).expect("token JSON");
    assert_eq!(tokens["token_type"], "Bearer");
    assert_eq!(tokens["expires_in"], 300);
    let jwt = tokens["id_token"].as_str().expect("ID token");
    let parts: Vec<_> = jwt.split('.').collect();
    assert_eq!(parts.len(), 3);
    let jwk = &world.key_set["keys"][0];
    let header: Value = serde_json::from_slice(&decode(parts[0])).expect("JWT header");
    assert_eq!(header["alg"], "RS256");
    assert_eq!(header["kid"], jwk["kid"]);
    let signed = format!("{}.{}", parts[0], parts[1]);
    let n = decode(jwk["n"].as_str().expect("modulus"));
    let e = decode(jwk["e"].as_str().expect("exponent"));
    let signature = decode(parts[2]);
    assert!(skein_tls::verify_rs256(&n, &e, signed.as_bytes(), &signature));
    assert!(!skein_tls::verify_rs256(&n, &e, b"different signing input", &signature));
    let claims: Value = serde_json::from_slice(&decode(parts[1])).expect("JWT claims");
    assert_eq!(
        claims,
        json!({"iss":"https://app.example.com","sub":"3:alice@example.com","aud":"app","exp":300,"iat":0,"nonce":"once","preferred_username":"alice","email":"alice@example.com","email_verified":true,"name":"Alice"})
    );
    let access = tokens["access_token"].as_str().expect("access token");
    let answer = world.claims(access);
    assert_eq!(answer.status, 200);
    let info: Value = serde_json::from_slice(answer.body.expose()).expect("userinfo JSON");
    assert_eq!(
        info,
        json!({"sub":"3:alice@example.com","preferred_username":"alice","email":"alice@example.com","email_verified":true,"name":"Alice"})
    );
    invalid_grant(&world.redeem(&code, REDIRECT, VERIFIER, false, true));
    let expiring = world.code();
    assert_eq!(world.authorize(REDIRECT), w::AuthorizeAnswer::Busy, "live code reserves remaining token slot");
    world.advance(60);
    invalid_grant(&world.redeem(&expiring, REDIRECT, VERIFIER, true, true));
    let code = world.code();
    invalid_grant(&world.redeem(&code, "https://other.example.com/callback", VERIFIER, false, true));
    invalid_grant(&world.redeem(&code, REDIRECT, VERIFIER, false, true));
    let code = world.code();
    invalid_grant(&world.redeem(&code, REDIRECT, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", false, true));
    invalid_grant(&world.redeem(&code, REDIRECT, VERIFIER, false, true));
    let code = world.code();
    for header in [false, true] {
        let answer = world.redeem(&code, REDIRECT, VERIFIER, header, false);
        assert_eq!(answer.status, 401);
        error(&answer, "invalid_client");
        assert_eq!(answer.challenge.as_deref(), header.then_some(b"Basic".as_slice()));
    }
    let answer = world.redeem(&code, REDIRECT, VERIFIER, true, true);
    assert_eq!(answer.status, 200, "Basic accepted; invalid client attempts did not authenticate or spend code");
    assert_eq!(world.authorize(REDIRECT), w::AuthorizeAnswer::Busy, "token table full");
    world.advance(240);
    let answer = world.claims(access);
    assert_eq!(answer.status, 401);
    assert_eq!(answer.challenge.as_deref(), Some(b"Bearer error=\"invalid_token\"".as_slice()));
    assert!(answer.body.is_empty());
    world.signer.as_mut().expect("signed in").kind = people::Kind::Service;
    refused(world.authorize(REDIRECT), &w::AuthorizeWhy::NotAPerson);
    world.signer.as_mut().expect("signed in").kind = people::Kind::Person;
    world.signer.as_mut().expect("signed in").identity.key.subject = Box::new([255]);
    refused(world.authorize(REDIRECT), &w::AuthorizeWhy::NotAPerson);
    world.sign_in();
    world.advance(60);
    let _first = world.code();
    let _second = world.code();
    assert_eq!(world.authorize(REDIRECT), w::AuthorizeAnswer::Busy, "code table full");
    world.advance(60);
    let fresh = world.code();
    let mut restarted = World::new(true);
    invalid_grant(&restarted.redeem(&fresh, REDIRECT, VERIFIER, false, true));
    assert_eq!(restarted.claims(access).status, 401, "restart retains no access token");
    let mut absent = World::new(false);
    let answer = absent.redeem(&code, REDIRECT, VERIFIER, false, true);
    error(&answer, "invalid_client");
    let answer = absent.claims("unknown");
    error(&answer, "invalid_client");
    eprintln!("provider story excluding key generation: {:?}", started.elapsed());
}

fn public_client(id: &[u8]) -> ProviderClientChange {
    ProviderClientChange::Add(skein_oauth_issuer::Client {
        id: id.into(),
        secret: None,
        redirect_uri: REDIRECT.as_bytes().into(),
    })
}
fn remove_client(id: &[u8]) -> ProviderClientChange {
    ProviderClientChange::Remove { id: id.into() }
}
fn tokens(answer: &ProviderAnswer) -> Value {
    assert_eq!(answer.status, 200);
    serde_json::from_slice(answer.body.expose()).expect("token JSON")
}
fn subject(world: &mut World, tokens: &Value) -> Value {
    let jwt = tokens["id_token"].as_str().expect("ID token");
    let claims: Value =
        serde_json::from_slice(&decode(jwt.split('.').nth(1).expect("JWT claims"))).expect("claims JSON");
    let answer = world.claims(tokens["access_token"].as_str().expect("access token"));
    assert_eq!(answer.status, 200);
    let info: Value = serde_json::from_slice(answer.body.expose()).expect("userinfo JSON");
    assert_eq!(info["sub"], claims["sub"]);
    info["sub"].clone()
}

#[test]
fn runtime_client_signs_in_and_removal_revokes_only_its_grants() {
    let mut configured = World::new(true);
    configured.sign_in();
    let code = configured.code();
    let configured_tokens = tokens(&configured.redeem(&code, REDIRECT, VERIFIER, false, true));
    let configured_sub = subject(&mut configured, &configured_tokens);

    let mut world = World::empty();
    refused(world.authorize_client("tracker", REDIRECT), &w::AuthorizeWhy::UnknownClient);
    world.change(public_client(b"tracker")).expect("application adds public client");
    assert_eq!(world.authorize_client("tracker", REDIRECT), w::AuthorizeAnswer::SignedOut);
    world.sign_in();
    let code = world.code_for("tracker");
    let added_tokens = tokens(&world.redeem_public("tracker", &code));
    assert_eq!(subject(&mut world, &added_tokens), configured_sub);
    let stale = world.code_for("tracker");
    world.change(public_client(b"other")).expect("another client");
    let other_code = world.code_for("other");
    let other_tokens = tokens(&world.redeem_public("other", &other_code));
    let other_pending = world.code_for("other");

    world.change(remove_client(b"tracker")).expect("remove held client");
    refused(world.authorize_client("tracker", REDIRECT), &w::AuthorizeWhy::UnknownClient);
    error(&world.redeem_public("tracker", &stale), "invalid_client");
    assert_eq!(world.claims(added_tokens["access_token"].as_str().expect("access token")).status, 401);
    // Re-adding must not resurrect codes or access tokens from the removed registration.
    world.change(public_client(b"tracker")).expect("slot reused");
    invalid_grant(&world.redeem_public("tracker", &stale));
    assert_eq!(world.claims(added_tokens["access_token"].as_str().expect("access token")).status, 401);
    assert_eq!(subject(&mut world, &other_tokens), configured_sub);
    tokens(&world.redeem_public("other", &other_pending));
    let fresh = world.code_for("tracker");
    tokens(&world.redeem_public("tracker", &fresh));
}

#[test]
fn runtime_client_changes_name_each_refusal_and_leave_registrations_unchanged() {
    use skein_oauth_issuer::{Client, ClientError, Error};
    let client_error = |why| Err(ProviderClientRefusal::Client(why));
    let mut absent = World::new(false);
    assert_eq!(absent.change(public_client(b"tracker")), Err(ProviderClientRefusal::NoProvider));
    assert_eq!(absent.change(remove_client(b"tracker")), Err(ProviderClientRefusal::NoProvider));
    let mut world = World::empty();
    assert_eq!(world.change(remove_client(b"tracker")), client_error(ClientError::Unknown));
    for (id, secret, uri, why) in [
        (b"".as_slice(), None, REDIRECT, Error::Client),
        (b"bad id".as_slice(), None, REDIRECT, Error::Client),
        (b"tracker".as_slice(), Some(Secret::new(Box::new([]))), REDIRECT, Error::Client),
        (b"tracker".as_slice(), None, "http://remote.example:80/cb", Error::Address),
    ] {
        assert_eq!(
            world.change(ProviderClientChange::Add(Client {
                id: id.into(),
                secret,
                redirect_uri: uri.as_bytes().into()
            })),
            client_error(ClientError::Invalid(why))
        );
    }
    world.change(public_client(b"tracker")).expect("first client");
    assert_eq!(world.change(public_client(b"tracker")), client_error(ClientError::Duplicate));
    let cap = jig_protocol_engine_clients::provider_limits(&world.env.limits).clients;
    for number in 1..cap {
        world.change(public_client(format!("client-{number}").as_bytes())).expect("within client limit");
    }
    assert_eq!(world.change(public_client(b"past")), client_error(ClientError::Full));
    assert_eq!(world.change(public_client(b"tracker")), client_error(ClientError::Duplicate));
    world.change(remove_client(b"tracker")).expect("remove at capacity");
    assert_eq!(world.change(remove_client(b"tracker")), client_error(ClientError::Unknown));
    world.change(public_client(b"replacement")).expect("freed slot");
    world.send(Event::Stop);
    assert_eq!(world.change(public_client(b"stopped")), Err(ProviderClientRefusal::NotReady));
    assert_eq!(world.change(remove_client(b"replacement")), Err(ProviderClientRefusal::NotReady));
}

#[test]
fn removal_while_people_is_read_does_not_issue_a_code_after_readding() {
    let mut world = World::empty();
    world.change(public_client(b"tracker")).expect("client");
    world.sign_in();
    world.delay_whose = true;
    world.begin_query(format!("client_id=tracker&redirect_uri={REDIRECT}&response_type=code&scope=openid&code_challenge_method=S256&code_challenge={CHALLENGE}").as_bytes());
    let reply = world.whose.take().expect("people read outstanding");
    assert!(world.reply.is_none());
    world.change(remove_client(b"tracker")).expect("removed while people is read");
    refused(world.authorization_answer(), &w::AuthorizeWhy::UnknownClient);
    world.change(public_client(b"tracker")).expect("new registration");
    world.send(Event::Reply { to: reply, reply: people::Reply::Signer(world.signer.clone()) });
    assert!(world.reply.is_none(), "late reply issues no code and no second answer");
    world.delay_whose = false;
    let fresh = world.code_for("tracker");
    tokens(&world.redeem_public("tracker", &fresh));
}

#[test]
fn removal_while_credential_is_read_rechecks_the_client() {
    let mut world = World::empty();
    world.change(public_client(b"tracker")).expect("client");
    world.sign_in();
    world.delay_credential = true;
    world.delay_whose = true;
    world.begin_query(format!("client_id=tracker&redirect_uri={REDIRECT}&response_type=code&scope=openid&code_challenge_method=S256&code_challenge={CHALLENGE}").as_bytes());
    let reply = world.credential_read.take().expect("credential read outstanding");
    world.change(remove_client(b"tracker")).expect("removed while credential is read");
    world.send(Event::Secrets(reply));
    refused(world.authorization_answer(), &w::AuthorizeWhy::UnknownClient);
    assert!(world.whose.is_none(), "removed client reaches no people read");
}
