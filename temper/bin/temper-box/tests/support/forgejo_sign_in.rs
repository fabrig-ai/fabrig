//! Owned Forgejo signing in a Google member or its box owner through temper.
//! temper/docs/design/protocol/openid-provider.md, 9.
//! temper/docs/design/testing/process-tier.md.
use super::{
    configuration,
    forge_sign_in::{command, configure, success},
    story_socket::{self, Scope, Socket},
};
use jig_client_protocol_wire as wire;
use serde_json::{Value, json};
use skein_lib::{
    Reader, Writer,
    base64::{self, Alphabet},
};
use std::{fmt::Write, fs, path::Path, process::Command};
use std::{
    io,
    os::{fd::AsRawFd, unix::process::CommandExt},
    process::{Child, Stdio},
};

pub(super) struct Serve {
    pub(super) child: Child,
    pub(super) reaped: bool,
}
impl Serve {
    pub(super) fn stop(mut self) {
        assert!(self.child.id() > 1, "owned serve group");
        skein_shell::keeper::signal_group(self.child.id(), skein_io::kernel::Signal::Terminate)
            .expect("terminate serve");
        let status = self.child.wait().expect("serve exit");
        self.reaped = true;
        assert!(status.success(), "serve drained successfully");
        assert!(!skein_shell::keeper::group_live(self.child.id()).expect("serve group"), "no surviving serve child");
    }
}
impl Drop for Serve {
    fn drop(&mut self) {
        if self.reaped {
            return;
        }
        let _ = skein_shell::keeper::signal_group(self.child.id(), skein_io::kernel::Signal::Kill);
        self.child.wait().expect("serve reaped");
    }
}

pub(super) struct Answer {
    pub(super) status: u16,
    pub(super) location: Option<String>,
    pub(super) body: Vec<u8>,
}
#[derive(Default)]
pub(super) struct Browser {
    // These two loopback origins share a host, as browser cookies do; paths still apply.
    cookies: Vec<(String, String, String)>,
}
impl Browser {
    pub(super) fn call(&mut self, url: &str, body: Option<&[u8]>) -> io::Result<Answer> {
        self.request(url, body, "application/octet-stream")
    }
    pub(super) fn form(&mut self, url: &str, fields: &[(&str, &str)]) -> io::Result<Answer> {
        let encode = |value: &str| {
            let mut encoded = String::new();
            for byte in value.bytes() {
                if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                    encoded.push(char::from(byte));
                } else {
                    write!(encoded, "%{byte:02X}").expect("form byte");
                }
            }
            encoded
        };
        let body = fields
            .iter()
            .map(|(name, value)| format!("{}={}", encode(name), encode(value)))
            .collect::<Vec<_>>()
            .join("&");
        self.request(url, Some(body.as_bytes()), "application/x-www-form-urlencoded")
    }
    fn signed_out_of_forgejo(&mut self) {
        self.cookies.retain(|(name, _, _)| name.starts_with("jig-"));
    }
    pub(super) fn follow(&mut self, mut address: String, mut answer: Answer) -> io::Result<Landed> {
        for _ in 0..5 {
            if !matches!(answer.status, 302 | 303 | 307) {
                break;
            }
            address = answer.location.expect("browser redirect");
            answer = self.call(&address, None)?;
        }
        Ok(Landed { status: answer.status, address, body: answer.body })
    }
    fn request(&mut self, url: &str, body: Option<&[u8]>, content_type: &str) -> io::Result<Answer> {
        let (host, path) = url.strip_prefix("http://").expect("local HTTP").split_once('/').expect("URL path");
        let path = format!("/{path}");
        let address: std::net::SocketAddr = host.parse().expect("literal socket");
        assert!(address.ip().is_loopback(), "fixture peers are loopback only");
        let mut stream = Socket::connect(
            "the Forgejo browser's connection",
            skein_lib::Duration::from_secs(2),
            address,
            Scope::Owned,
        )?;
        let cookie = self
            .cookies
            .iter()
            .filter(|(_, _, scope)| path.starts_with(scope))
            .map(|(name, value, _)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");
        let data = body.unwrap_or_default();
        let method = if body.is_some() { "POST" } else { "GET" };
        let mut request =
            format!("{method} {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\nCookie: {cookie}\r\n");
        if body.is_some() {
            write!(request, "Content-Type: {content_type}\r\nJig-Client: browser\r\nOrigin: http://{host}\r\nSec-Fetch-Site: same-origin\r\nContent-Length: {}\r\n", data.len()).expect("browser headers");
        }
        request.push_str("\r\n");
        stream.write_all("the Forgejo browser's request", skein_lib::Duration::from_secs(2), request.as_bytes())?;
        stream.write_all("the Forgejo browser's body", skein_lib::Duration::from_secs(2), data)?;
        let bytes = stream.read_to_end("the Forgejo browser's reply", skein_lib::Duration::from_secs(10), 1 << 20)?;
        let split = bytes.windows(4).position(|part| part == b"\r\n\r\n").expect("HTTP head");
        let head = std::str::from_utf8(&bytes[..split]).expect("head text");
        let status = head.split_whitespace().nth(1).expect("status").parse().expect("status number");
        let mut location = None;
        for line in head.lines().skip(1) {
            let (name, value) = line.split_once(':').expect("header");
            let value = value.trim();
            if name.eq_ignore_ascii_case("location") {
                location = Some(if value.starts_with('/') { format!("http://{host}{value}") } else { value.into() });
            }
            if name.eq_ignore_ascii_case("set-cookie") {
                let mut parts = value.split(';');
                let (name, value) = parts.next().expect("cookie").split_once('=').expect("cookie value");
                let scope = parts.find_map(|part| part.trim().strip_prefix("Path=")).unwrap_or("/");
                self.cookies.retain(|(old, _, path)| old != name || path != scope);
                self.cookies.push((name.into(), value.into(), scope.into()));
            }
        }
        Ok(Answer { status, location, body: bytes[split + 4..].to_vec() })
    }
}

/// Forgejo's API names the account the provider's claims made.
fn member_named(instance: &skein_throwaway_forgejo::Instance) {
    let response = instance
        .exchange(&skein_throwaway_forgejo::Outgoing::get("/api/v1/users/sam", Some(&instance.admin().token)))
        .expect("member API");
    assert_eq!(response.status, 200, "Forgejo created the member");
    let member: Value = serde_json::from_slice(&response.body).expect("member JSON");
    assert_eq!(member["login"], "sam");
    assert_eq!(member["email"], "sam@example.com");
    // Forgejo keeps the claims' `sub` as the account's login name: Google's
    // provider and the member's address, never the printed link's provider.
    assert_eq!(member["login_name"], "3:sam@example.com");
}

/// The one process proof of Forgejo's settings and the provider's claims.
pub fn run(directory: &Path) {
    if let Err(error) = run_owned(directory, false) {
        story_socket::finish(error);
    }
}

/// The real Forgejo proof for the box's public client and repeatable source command.
pub fn public(directory: &Path) {
    if let Err(error) = run_owned(directory, true) {
        story_socket::finish(error);
    }
}
fn run_owned(directory: &Path, public: bool) -> io::Result<()> {
    let clock = skein_shell::Clock::new();
    let whole = clock.now().now;
    let forge = super::owned::tier_configuration(directory);
    let [temper] = super::free_addresses(&[forge]);
    let (_google, instructions) = client_configuration(directory, forge, temper, public);
    fixture_bundle(directory);
    success(&super::owned::prepare(directory, 1));
    assert!(!directory.join("box/groups").exists(), "prepare needs no group's file");
    let running = super::owned::run(directory, "forgejo");
    let instance = super::owned::attach(directory, forge.port(), running);
    let past_start = clock.now().now;
    let mut serve = start_serve(directory);
    let stdout = serve.child.stdout.take().expect("stdout pipe");
    let link = super::read_line(skein_io::kernel::Fd::new(stdout.as_raw_fd()), super::WITHIN);
    if let Some(instructions) = instructions {
        confidential_source(directory, &instructions);
    } else {
        public_source(directory);
    }
    let link = std::str::from_utf8(&link).expect("printed link");
    let mut browser = Browser::default();
    sign_in(&mut browser, temper, forge, link)?;
    if public {
        owner_named(directory, &instance);
        serve.stop();
        instance.stop().expect("public Forgejo stopped");
        eprintln!("public Forgejo source and token exchange: {:?}", clock.now().now.saturating_since(whole));
        return Ok(());
    }
    member_named(&instance);
    let member_signed_in = clock.now().now;
    let instance = no_registration(directory, &mut browser, temper, forge, instance)?;
    let registration_checked = clock.now().now;
    connect(directory, forge, temper, &stdout, link, &mut browser)?;
    let connected = clock.now().now;
    serve.stop();
    instance.stop().expect("Forgejo stopped");
    assert!(!directory.join("box/groups").exists(), "the walk needs no group's file");
    eprintln!(
        "Forgejo sign-in: whole {:?}; past Forgejo start {:?}; member sign-in {:?}; no registration {:?}; admin connect {:?}; shutdown {:?}",
        clock.now().now.saturating_since(whole),
        clock.now().now.saturating_since(past_start),
        member_signed_in.saturating_since(past_start),
        registration_checked.saturating_since(member_signed_in),
        connected.saturating_since(registration_checked),
        clock.now().now.saturating_since(connected)
    );
    Ok(())
}

fn client_configuration(
    directory: &Path,
    forge: std::net::SocketAddr,
    temper: std::net::SocketAddr,
    public: bool,
) -> (Option<skein_fake_peers::oauth::Peer>, Option<std::process::Output>) {
    if public {
        public_configuration(directory, forge, temper);
        return (None, None);
    }
    let [listener] = super::reserve_listeners(&[forge, temper]);
    let google = listener.local_addr().expect("issuer address");
    configure(directory, &forge.to_string(), &temper.to_string(), &google.to_string());
    let registration = json!({"issuer":format!("http://{google}"),"authorization":format!("http://{google}/auth"),"token":format!("http://{google}/token"),"revocation":format!("http://{google}/revoke"),"keys":format!("http://{google}/keys"),"client":"temper-test","scopes":[]});
    let google =
        super::sign_in::issuer(listener, &registration, format!("http://{temper}/sign-in/google/return").as_bytes());
    let instructions = command(directory).output().expect("sign-in instructions");
    success(&instructions);
    (Some(google), Some(instructions))
}

fn public_configuration(directory: &Path, forge: std::net::SocketAddr, temper: std::net::SocketAddr) {
    let path = directory.join("temper.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    let email = value["application"]["machine"]["owned"]["person"]["email"].clone();
    value["engine"]["owners"] = json!([{"provider":1,"subject":email}]);
    value["engine"]["clients"]["provider"] = json!({
        "key":"provider.key", "addresses":[1],
        "clients":[{"id":"forgejo", "secret":null,
            "redirect_uri":format!("http://{forge}/user/oauth2/temper/callback")}]
    });
    value["application"]["clients"]["listener"] = json!({"socket":temper});
    value["application"]["sign_in"] = Value::Null;
    value["application"]["machine"]["owned"]["sign_in"] = json!("forgejo");
    super::write_private(&path, &serde_json::to_vec(&value).expect("public configuration"));
    super::write_private(&directory.join("provider.key"), b"");
    let key = Command::new("openssl")
        .args(["genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:3072", "-out"])
        .arg(directory.join("provider.key"))
        .output()
        .expect("public provider key generation");
    success(&key);
    assert!(!directory.join("forgejo.secret").exists(), "public client needs no secret");
    assert!(!directory.join("google.secret").exists(), "box needs no Google sign-in");
}

fn start_serve(directory: &Path) -> Serve {
    let log = fs::File::create(directory.join("sign-in-serve.log")).expect("private log");
    Serve {
        child: Command::new(skein_scratch::paths::binary("temper-box"))
            .arg("serve")
            .args(configuration::flags(directory))
            .current_dir(directory)
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(log)
            .spawn()
            .expect("serve"),
        reaped: false,
    }
}

fn confidential_source(directory: &Path, instructions: &std::process::Output) {
    let secret = fs::read(directory.join("forgejo.secret")).expect("scratch client secret");
    for output in [&instructions.stdout, &instructions.stderr] {
        assert!(!output.windows(secret.len()).any(|part| part == secret), "credential absent from output");
    }
    let printed = std::str::from_utf8(&instructions.stdout).expect("instructions").lines().next().expect("command");
    let added = Command::new("/bin/sh")
        .args(["-c", &owned_command(directory, printed)])
        .current_dir(directory.join("forge-1/forgejo"))
        .output()
        .expect("printed command");
    assert!(added.status.success(), "printed add-oauth command failed");
    for output in [&added.stdout, &added.stderr] {
        let lower = String::from_utf8_lossy(output).to_lowercase();
        assert!(!lower.contains("failed") && !lower.contains("error"), "add-oauth reported failure");
        assert!(!output.windows(secret.len()).any(|part| part == secret), "credential absent from add-oauth output");
    }
}

fn public_source(directory: &Path) {
    for _ in 0..2 {
        let added = Command::new(skein_scratch::paths::binary("temper-box"))
            .args(["forge", "source"])
            .args(configuration::flags(directory))
            .arg("1")
            .current_dir(directory)
            .output()
            .expect("public source command");
        success(&added);
        assert_eq!(added.stdout, b"source added\n", "{}", String::from_utf8_lossy(&added.stdout));
    }
    let list = Command::new(super::owned::binaries().0)
        .arg("--work-path")
        .arg(directory.join("forge-1/forgejo"))
        .arg("--config")
        .arg(directory.join("forge-1/forgejo/app.ini"))
        .args(["admin", "auth", "list"])
        .output()
        .expect("real auth list");
    success(&list);
    assert!(temper_forge_setup::sign_in::source_present(&list.stdout));
    let renamed = String::from_utf8_lossy(&list.stdout).replace("temper", "temper-other");
    assert!(!temper_forge_setup::sign_in::source_present(renamed.as_bytes()), "source names are exact columns");
    let count = String::from_utf8_lossy(&list.stdout)
        .lines()
        .filter(|line| line.split_whitespace().nth(1) == Some("temper"))
        .count();
    assert_eq!(count, 1, "a second source command keeps exactly one source");
    eprintln!("public source list: {}", String::from_utf8_lossy(&list.stdout));
}

fn owner_named(directory: &Path, instance: &skein_throwaway_forgejo::Instance) {
    let document: Value =
        serde_json::from_slice(&fs::read(directory.join("temper.json")).expect("configuration")).expect("JSON");
    let person = &document["application"]["machine"]["owned"]["person"];
    let login = person["login"].as_str().expect("owner login");
    let email = person["email"].as_str().expect("owner email");
    let response = instance
        .exchange(&skein_throwaway_forgejo::Outgoing::get(
            &format!("/api/v1/users/{login}"),
            Some(&instance.admin().token),
        ))
        .expect("owner API");
    assert_eq!(response.status, 200);
    let owner: Value = serde_json::from_slice(&response.body).expect("owner JSON");
    assert_eq!(owner["login"], login);
    assert_eq!(owner["email"], email);
    // Internal accounts keep their password login_name; the provider's claims
    // are kept with the external identity linked to that account.
    let linked = Command::new("python3")
        .args(["-I", "-B", "-c", "import json,sqlite3,sys; c=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True); print(json.dumps(c.execute('select u.name,u.email,e.external_id from external_login_user e join user u on u.id=e.user_id').fetchall()))"])
        .arg(directory.join("forge-1/forgejo/forgejo.db"))
        .output()
        .expect("linked owner claims");
    success(&linked);
    let claims: Value = serde_json::from_slice(&linked.stdout).expect("linked identity JSON");
    assert_eq!(claims, json!([[login, email, format!("1:{email}")]]), "box owner's link claims");
}

fn sign_in(
    browser: &mut Browser,
    temper: std::net::SocketAddr,
    forge: std::net::SocketAddr,
    link: &str,
) -> io::Result<()> {
    let limits = jig_protocol_config::profile::profile(jig_protocol_config::profile::Profile::Small).clients.wire;
    let secret = base64::decode(Alphabet::Url, link.split_once("secret=").expect("link secret").1.as_bytes(), 32)
        .expect("link encoding");
    let record = wire::LinkSignIn::new(&limits, wire::LinkSignInParts { secret: skein_lib::Secret::new(secret) })
        .expect("link record");
    let mut out = Writer::new(usize::try_from(record.measure()).expect("record size"));
    record.encode(&mut out).expect("link wire");
    let page = browser.call(&format!("http://{temper}/"), None)?;
    assert_eq!(page.status, 200);
    assert!(
        !String::from_utf8(page.body).expect("bundle page").contains("temper-forge-sign-in"),
        "a forge not yet connected has no sign-in meta"
    );
    let signed = browser.call(&format!("http://{temper}/jig/v1/sign-in/link"), Some(&out.finish()))?;
    assert_eq!(signed.status, 200);
    assert!(browser.cookies.iter().any(|(name, _, _)| name == "jig-sign-in"), "printed link set the browser cookie");
    let landed = through_temper(browser, temper, forge)?;
    assert_eq!(landed.status, 200);
    Ok(())
}

pub(super) struct Landed {
    pub(super) status: u16,
    pub(super) address: String,
    pub(super) body: Vec<u8>,
}

fn through_temper(
    browser: &mut Browser,
    temper: std::net::SocketAddr,
    forge: std::net::SocketAddr,
) -> io::Result<Landed> {
    let limits = jig_protocol_config::profile::profile(jig_protocol_config::profile::Profile::Small).clients.wire;
    let opened = browser.call(&format!("http://{forge}/user/oauth2/temper"), None)?;
    assert_eq!(opened.status, 307);
    let authorization = opened.location.expect("provider authorization redirect");
    assert!(authorization.starts_with(&format!("http://{temper}/oauth/authorize?")));
    let page = browser.call(&authorization, None)?;
    assert_eq!(page.status, 200);
    let query = authorization.split_once('?').expect("authorization query").1;
    let record = wire::Authorize::new(&limits, wire::AuthorizeParts { query: query.as_bytes().into() })
        .expect("authorize record");
    let mut out = Writer::new(usize::try_from(record.measure()).expect("record size"));
    record.encode(&mut out).expect("authorize wire");
    let authorized = browser.call(&format!("http://{temper}/jig/v1/authorize"), Some(&out.finish()))?;
    assert_eq!(authorized.status, 200);
    let wire::AuthorizeAnswer::Go(go) =
        wire::AuthorizeAnswer::decode(&limits, &mut Reader::new(&authorized.body)).expect("authorize answer")
    else {
        panic!("signed-in page must receive go")
    };
    let address = std::str::from_utf8(go.location()).expect("callback URL");
    let callback = browser.call(address, None)?;
    browser.follow(address.into(), callback)
}

/// The real settings' registration and password-linking proof
/// (temper/docs/design/draft/dogfood-hacks.md, 2.8).
fn no_registration(
    directory: &Path,
    browser: &mut Browser,
    temper: std::net::SocketAddr,
    forge: std::net::SocketAddr,
    instance: skein_throwaway_forgejo::Instance,
) -> io::Result<skein_throwaway_forgejo::Instance> {
    use skein_throwaway_forgejo::Outgoing;
    instance.stop().expect("Forgejo stopped for preparation");
    success(&super::owned::prepare_no_registration(directory, 1));
    let running = super::owned::run(directory, "forgejo");
    let instance = super::owned::attach(directory, forge.port(), running);

    browser.signed_out_of_forgejo();
    let linked = through_temper(browser, temper, forge)?;
    landing("(2) already linked", &linked);
    assert_eq!(linked.status, 200);
    assert!(!linked.address.contains("/user/link_account"));
    signed_in_as_sam(browser, forge)?;

    let deleted = instance
        .exchange(&Outgoing::json("DELETE", "/api/v1/admin/users/sam", Some(&instance.admin().token), b"{}"))
        .expect("delete member");
    assert_eq!(deleted.status, 204, "admin deletes sam");
    no_sam(&instance);
    browser.signed_out_of_forgejo();
    let missing = through_temper(browser, temper, forge)?;
    no_sam(&instance);
    link_page("(1) no account", &missing, forge);

    let csrf = csrf(&missing.body);
    eprintln!("Forgejo link form CSRF field: {}", csrf.is_some());
    let password = skein_throwaway_forgejo::fresh_secret().expect("throwaway password");
    let mut fields = vec![
        ("user_name", "sam"),
        ("email", "sam@example.com"),
        ("password", password.as_str()),
        ("retype", password.as_str()),
    ];
    if let Some(value) = &csrf {
        fields.push(("_csrf", value));
    }
    let signup = browser.form(&format!("http://{forge}/user/link_account_signup"), &fields)?;
    eprintln!("(3) POST /user/link_account_signup: {}", signup.status);
    assert_eq!(signup.status, 403);
    let signup = browser.form(&format!("http://{forge}/user/sign_up"), &fields)?;
    eprintln!("(3) POST /user/sign_up: {}", signup.status);
    assert_eq!(signup.status, 403);
    no_sam(&instance);

    let body = serde_json::to_vec(
        &json!({"username":"sam", "email":"sam@example.com", "password":password, "must_change_password":false}),
    )
    .expect("member JSON");
    let created = instance
        .exchange(&Outgoing::json("POST", "/api/v1/admin/users", Some(&instance.admin().token), &body))
        .expect("make unlinked member");
    assert_eq!(created.status, 201, "admin makes sam");
    browser.signed_out_of_forgejo();
    let unlinked = through_temper(browser, temper, forge)?;
    link_page("(3) existing unlinked account", &unlinked, forge);
    let wrong = link_password(browser, forge, &unlinked.body, "wrong-throwaway-password")?;
    landing("(3) wrong password", &wrong);
    browser.signed_out_of_forgejo();
    let still_unlinked = through_temper(browser, temper, forge)?;
    link_page("(3) wrong password linked nothing", &still_unlinked, forge);
    let right = link_password(browser, forge, &still_unlinked.body, &password)?;
    landing("(3) right password", &right);
    assert_eq!(right.status, 200);
    assert!(!right.address.contains("/user/link_account"));
    signed_in_as_sam(browser, forge)?;
    browser.signed_out_of_forgejo();
    let once = through_temper(browser, temper, forge)?;
    landing("(3) linked once; (2) signs in again", &once);
    assert_eq!(once.status, 200);
    assert!(!once.address.contains("/user/link_account"));
    signed_in_as_sam(browser, forge)?;
    Ok(instance)
}

fn landing(leg: &str, landed: &Landed) {
    eprintln!("{leg}: {} {}", landed.status, landed.address.split('?').next().expect("address"));
}

fn link_page(leg: &str, landed: &Landed, forge: std::net::SocketAddr) {
    landing(leg, landed);
    let page = std::str::from_utf8(&landed.body).expect("link page text");
    let signin = page.contains("action=\"/user/link_account_signin\"");
    let signup = page.contains("action=\"/user/link_account_signup\"");
    eprintln!(
        "{leg}: link_account_signin form {signin}; link_account_signup form {signup}; CSRF field {}",
        csrf(&landed.body).is_some()
    );
    for form in page.split("<form ").skip(1) {
        let (tag, contents) = form.split_once('>').expect("form tag");
        if !tag.contains("action=\"/user/link_account_signup\"") {
            continue;
        }
        let contents = contents.split_once("</form>").expect("closed signup form").0;
        let mut names = Vec::new();
        for element in contents.split('<').skip(1) {
            let tag = element.split_once('>').expect("element tag").0;
            let kind = tag.split_whitespace().next().expect("element name");
            if ["input", "button", "select", "textarea"].contains(&kind) {
                let name = tag.split_once("name=\"").map(|(_, after)| after.split_once('"').expect("name end").0);
                names.push((kind, name));
            }
        }
        let mut text = String::new();
        let mut in_tag = false;
        for character in contents.chars() {
            match character {
                '<' => in_tag = true,
                '>' => {
                    in_tag = false;
                    text.push(' ');
                }
                _ if !in_tag => text.push(character),
                _ => {}
            }
        }
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        eprintln!("{leg}: link_account_signup controls (tag, name): {names:?}; text: {text:?}");
        assert!(names.is_empty(), "link page offers no registration fields");
    }
    assert_eq!(landed.status, 200);
    assert_eq!(landed.address, format!("http://{forge}/user/link_account"));
    assert!(signin, "link page offers password linking");
}

fn no_sam(instance: &skein_throwaway_forgejo::Instance) {
    let missing = instance
        .exchange(&skein_throwaway_forgejo::Outgoing::get("/api/v1/users/sam", Some(&instance.admin().token)))
        .expect("missing member API");
    eprintln!("GET /api/v1/users/sam: {}", missing.status);
    assert_eq!(missing.status, 404, "sign-in made no account");
}

fn signed_in_as_sam(browser: &mut Browser, forge: std::net::SocketAddr) -> io::Result<()> {
    let member = browser.call(&format!("http://{forge}/user/settings"), None)?;
    eprintln!("GET /user/settings: {}", member.status);
    assert_eq!(member.status, 200, "only the signed-in account can read its settings");
    let page = std::str::from_utf8(&member.body).expect("signed-in settings text");
    assert!(page.contains("value=\"sam\""), "private profile settings name sam");
    Ok(())
}

pub(super) fn csrf(page: &[u8]) -> Option<String> {
    let page = std::str::from_utf8(page).expect("form text");
    let after = page.split_once("name=\"_csrf\"")?.1;
    Some(after.split_once("value=\"")?.1.split_once('"')?.0.into())
}

fn link_password(
    browser: &mut Browser,
    forge: std::net::SocketAddr,
    page: &[u8],
    password: &str,
) -> io::Result<Landed> {
    let csrf = csrf(page);
    eprintln!("Forgejo password link POST CSRF field: {}", csrf.is_some());
    let mut fields = vec![("user_name", "sam"), ("password", password)];
    if let Some(value) = &csrf {
        fields.push(("_csrf", value));
    }
    let address = format!("http://{forge}/user/link_account_signin");
    let answer = browser.form(&address, &fields)?;
    browser.follow(address, answer)
}

pub(super) fn fixture_bundle(directory: &Path) {
    let bundle = directory.join("bundle");
    fs::create_dir_all(bundle.join("0123456789abcdef")).expect("bundle directory");
    fs::write(bundle.join("index.html"), b"<html><head></head><body></body></html>").expect("bundle page");
    fs::write(bundle.join("0123456789abcdef/boot.js"), b"boot();").expect("bundle asset");
    let path = directory.join("temper.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["application"]["clients"]["bundle"] = json!(bundle);
    super::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
}

fn connect(
    directory: &Path,
    forge: std::net::SocketAddr,
    temper: std::net::SocketAddr,
    stdout: &std::process::ChildStdout,
    link: &str,
    browser: &mut Browser,
) -> io::Result<()> {
    let link_file = directory.join("ctl-link");
    super::write_private(&link_file, link.as_bytes());
    let refused = ctl(directory, &["sign-in", "ctl-link"]);
    assert!(!refused.status.success(), "the browser spent the printed link");
    let link = super::read_line(skein_io::kernel::Fd::new(stdout.as_raw_fd()), super::WITHIN);
    super::write_private(&link_file, &link);
    success(&ctl(directory, &["sign-in", "ctl-link"]));
    let forges = ctl(directory, &["forge", "list", "--json"]);
    success(&forges);
    let forges: Value = serde_json::from_slice(&forges.stdout).expect("forges JSON");
    assert_eq!(forges["forges"], json!([]), "serve without the box keeps no owned forge");
    success(&ctl(
        directory,
        &[
            "forge",
            "connect",
            "forgejo-admin",
            "forge-1/admin_token",
            "--api",
            &forge.to_string(),
            "--git",
            &format!("http://{forge}"),
            "--key",
            "walk-connect",
        ],
    ));
    let forges = ctl(directory, &["forge", "list", "--until", "connected:1", "--json"]);
    success(&forges);
    let forges: Value = serde_json::from_slice(&forges.stdout).expect("connected forges JSON");
    assert_eq!(forges["met"], true);
    let forges = forges["forges"]["forges"].as_array().expect("forges");
    assert_eq!(forges.len(), 1);
    assert_eq!(forges[0]["number"], 1);
    assert_eq!(forges[0]["mode"], "managed");
    assert_eq!(forges[0]["state"], "connected");
    assert!(directory.join("forge-1/admin_token").is_file(), "managed connect keeps the admin handover");
    let page = browser.call(&format!("http://{temper}/"), None)?;
    assert_eq!(page.status, 200);
    assert!(
        String::from_utf8(page.body)
            .expect("connected bundle page")
            .contains(&format!("<meta name=\"temper-forge-sign-in\" content=\"1 forgejo http://{forge} other\">")),
        "the connected forge serves its sign-in meta"
    );
    Ok(())
}

fn ctl(directory: &Path, arguments: &[&str]) -> std::process::Output {
    super::story_command::standalone(
        Command::new(skein_scratch::paths::binary("temper-box"))
            .arg("ctl")
            .arg(directory.join("ctl"))
            .args(arguments)
            .current_dir(directory)
            .stdin(Stdio::null()),
        "the walk's ctl command",
        super::WITHIN,
    )
}

fn owned_command(directory: &Path, printed_command: &str) -> String {
    // temper/docs/design/protocol/setup/owned.md, 8: sign-in instructions name no kept forge.
    // Supply this disposable server's binary and work path around that generic command.
    let quote = |path: &Path| {
        use std::os::unix::ffi::OsStrExt;
        String::from_utf8(temper_forge_setup::sign_in::quote(path.as_os_str().as_bytes()).into_vec())
            .expect("quoted path")
    };
    let work = directory.join("forge-1/forgejo");
    format!(
        "{} --work-path {} --config {}{}",
        quote(&super::owned::binaries().0),
        quote(&work),
        quote(&work.join("app.ini")),
        printed_command.strip_prefix("'forgejo'").expect("generic Forgejo command")
    )
}
