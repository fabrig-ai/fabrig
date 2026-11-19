//! A box's own issuer and its real owned Forgejo (protocol/openid-provider.md, 9).
use super::{
    configuration,
    forge_sign_in::success,
    forgejo_sign_in::{Browser, Landed, Serve, csrf, fixture_bundle},
};
use jig_client_protocol_wire as wire;
use jig_client_protocol_wire::Record as _;
use serde_json::Value;
use skein_lib::{
    Reader, Writer,
    base64::{self, Alphabet},
};
use std::{
    fs, io,
    net::SocketAddr,
    os::{
        fd::AsRawFd,
        unix::{fs::PermissionsExt, process::CommandExt},
    },
    path::Path,
    process::{Command, Stdio},
};

pub fn run(root: &Path) {
    if let Err(error) = story(root) {
        super::story_socket::finish(error);
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the process story keeps setup, restart and fallback assertions within one owned lifetime"
)]
fn story(root: &Path) -> io::Result<()> {
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    let initialized = super::story_command::standalone(
        Command::new(skein_scratch::paths::binary("temper-box"))
            .args(["init", "--box"])
            .args(configuration::flags(root))
            .current_dir(root),
        "initialize the sign-in box",
        super::WITHIN,
    );
    success(&initialized);
    super::own_listener_files(root);
    fixture_bundle(root);
    let path = root.join(temper_config_shell::FILE);
    let temper = super::read_file(&path, temper_config_shell::Shape::Standalone).listen.expect("clients address");
    super::first_repository::stage_binaries(root);
    assert!(super::private(&root.join("provider.key")), "private issuer key");
    assert_eq!(fs::metadata(root)?.permissions().mode() & 0o777, 0o700);
    let mut kept_setup = None;
    for round in ["first", "second"] {
        let mut running = Serve {
            child: Command::new(skein_scratch::paths::binary("temper-box"))
                .args(["--echo", "steps"])
                .args(configuration::flags(root))
                .current_dir(root)
                .process_group(0)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(fs::File::create(root.join(format!("story-box-{round}.log")))?)
                .spawn()
                .expect("sign-in box"),
            reaped: false,
        };
        let stdout = running.child.stdout.take().expect("printed link pipe");
        let link = super::read_line(skein_io::kernel::Fd::new(stdout.as_raw_fd()), super::WITHIN);
        let signed = super::story_command::output(
            Command::new(skein_scratch::paths::binary("temper-box"))
                .arg("ctl")
                .arg(root.join("ctl"))
                .args(["sign-in", "-"])
                .current_dir(root),
            "the setup client's sign-in",
            super::WITHIN,
            Some(&link),
        );
        success(&signed);
        if round == "first" {
            success(&ctl_command(root, &["forge", "setup"]));
        }
        success(&ctl_command(root, &["forge", "list", "--until", "settled"]));
        wait_for_start(root, running.child.id())?;
        let listed = ctl_command(root, &["forge", "list", "--json"]);
        success(&listed);
        let row: Value = serde_json::from_slice(&listed.stdout).expect("forges JSON");
        let row = super::first_repository::owned_row(&row);
        let number = u16::try_from(row["number"].as_u64().expect("forge number")).expect("forge u16");
        let setup = super::owned::setup_value(root, number, "owned");
        let forge: SocketAddr =
            setup["listen"]["socket"].as_str().expect("chosen forge socket").parse().expect("forge address");
        let login = setup["person"]["login"].as_str().expect("chosen person login");
        let email = setup["person"]["email"].as_str().expect("chosen person email");
        if let Some(kept) = &kept_setup {
            assert_eq!(&setup, kept, "restart keeps its chosen setup");
        }
        assert_eq!(row["state"], "connected");
        assert_eq!(row["first"]["state"], "made");
        let before = users(root, forge);
        // Spending an already used link asks serve to print its next one.
        let mut browser = Browser::default();
        let _spent = link_sign_in(&mut browser, temper, &link)?;
        let link = super::read_line(skein_io::kernel::Fd::new(stdout.as_raw_fd()), super::WITHIN);
        assert_eq!(link_sign_in(&mut browser, temper, &link)?.status, 200);
        let page = browser.call(&format!("http://{temper}/"), None)?;
        let page = std::str::from_utf8(&page.body).expect("page text");
        assert!(
            page.contains(&format!("<meta name=\"temper-forge-sign-in\" content=\"1 forgejo http://{forge} owned\">")),
            "ready source meta"
        );
        assert!(!page.contains("temper-forge-sign-in-off"));
        for press in ["first", "second"] {
            let (landed, redirects) = repository(&mut browser, temper, forge)?;
            assert_eq!(landed.status, 200);
            assert_eq!(landed.address, format!("http://{forge}/work/hello"));
            assert_eq!(redirects, 3, "Forgejo start, issuer go, repository landing");
            signed_in(&mut browser, forge, login)?;
            eprintln!("{round} start, {press} press: {redirects} redirects; repository signed in");
        }
        assert_eq!(users(root, forge), before, "no second account");
        let (_, person) = api(root, forge, &format!("/users/{login}"));
        assert_eq!(person["login"], login);
        external_login(root, &format!("1:{email}"));
        password_sign_in(root, forge, login)?;
        let sources = auth_list(root);
        assert_eq!(
            sources.lines().filter(|line| line.split_whitespace().nth(1) == Some("temper")).count(),
            1,
            "exactly one source after either start"
        );
        let log = super::boxed::box_log(root);
        assert!(!log.contains("no sign-in through temper"), "{log}");
        // The next start uses its printed link in this same client directory.
        // End this sign-in first (client/ctl/README.md, 6.1).
        success(&ctl_command(root, &["sign-out"]));
        running.stop();
        kept_setup = Some(setup);
    }
    eprintln!("box sign-in proof: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
    Ok(())
}

fn ctl_command(root: &Path, args: &[&str]) -> std::process::Output {
    super::story_command::output(
        Command::new(skein_scratch::paths::binary("temper-box"))
            .arg("ctl")
            .arg(root.join("ctl"))
            .args(args)
            .current_dir(root),
        "the owned setup act",
        super::boxed::BOX_DEADLINE,
        None,
    )
}

fn wait_for_start(root: &Path, pid: u32) -> io::Result<()> {
    let mut deadline = skein_world::Deadline::arm("the restored source's final outcome", super::WITHIN);
    deadline.watch_process(pid);
    loop {
        if super::boxed::box_log(root).contains("Forge 1 (Forgejo): started") {
            return Ok(());
        }
        if deadline.passed() {
            return match deadline.expired() {
                skein_world::Expired::Starved(starved) => {
                    Err(io::Error::other(super::story_socket::StarvedSocket(starved)))
                }
                skein_world::Expired::Failed(figures) => {
                    Err(io::Error::new(io::ErrorKind::TimedOut, figures.to_string()))
                }
            };
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn api(root: &Path, forge: SocketAddr, route: &str) -> (u16, Value) {
    super::first_repository::header_request(
        &format!("http://{forge}/api/v1"),
        root.join("forge-1/person_read.header").to_str().expect("header path"),
        "GET",
        route,
        None,
    )
}

fn users(root: &Path, forge: SocketAddr) -> Value {
    let (status, users) = api(root, forge, "/users/search?limit=50");
    assert_eq!(status, 200);
    users["data"]
        .as_array()
        .expect("user list")
        .iter()
        .map(|user| serde_json::json!({"id":user["id"], "login":user["login"]}))
        .collect::<Vec<_>>()
        .into()
}

fn auth_list(root: &Path) -> String {
    let output = super::story_command::output(
        Command::new(super::owned::binaries().0)
            .arg("--work-path")
            .arg(root.join("forge-1/forgejo"))
            .arg("--config")
            .arg(root.join("forge-1/forgejo/app.ini"))
            .args(["admin", "auth", "list"]),
        "the real source list",
        super::WITHIN,
        None,
    );
    success(&output);
    String::from_utf8(output.stdout).expect("source list text")
}

fn link_sign_in(browser: &mut Browser, temper: SocketAddr, link: &[u8]) -> io::Result<super::forgejo_sign_in::Answer> {
    let link = std::str::from_utf8(link).expect("link text").trim();
    let secret = base64::decode(Alphabet::Url, link.split_once("secret=").expect("printed secret").1.as_bytes(), 32)
        .expect("link encoding");
    let record = wire::LinkSignIn { secret: skein_lib::Secret::new(secret) };
    let mut out = Writer::new(
        usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("record size"),
    );
    record.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("link wire");
    browser.call(&format!("http://{temper}/jig/v1/sign-in/link"), Some(&out.finish()))
}

fn repository(browser: &mut Browser, temper: SocketAddr, forge: SocketAddr) -> io::Result<(Landed, usize)> {
    let address = format!("http://{forge}/user/oauth2/temper?redirect_to=%2Fwork%2Fhello");
    let opened = browser.call(&address, None)?;
    assert_eq!(opened.status, 307);
    let mut redirects = usize::from(opened.location.is_some());
    let authorization = opened.location.expect("issuer redirect");
    assert!(authorization.starts_with(&format!("http://{temper}/oauth/authorize?")), "every press passes the issuer");
    assert_eq!(browser.call(&authorization, None)?.status, 200);
    let limits = jig_protocol_config_fixtures::small().clients.wire;
    let record = wire::Authorize { query: authorization.split_once('?').expect("query").1.as_bytes().into() };
    let mut out = Writer::new(
        usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("record size"),
    );
    record.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("authorize wire");
    let authorized = browser.call(&format!("http://{temper}/jig/v1/authorize"), Some(&out.finish()))?;
    assert_eq!(authorized.status, 200);
    let wire::AuthorizeAnswer::Go(go) =
        wire::AuthorizeAnswer::decode(&limits, &mut Reader::new(&authorized.body)).expect("authorize answer")
    else {
        panic!("owner receives go")
    };
    redirects += 1; // The client replaces the issuer page with its answer.
    let callback = std::str::from_utf8(go.location.as_ref()).expect("callback URL");
    assert!(callback.starts_with(&format!("http://{forge}/user/oauth2/temper/callback?")));
    let answer = browser.call(callback, None)?;
    assert_eq!(answer.status, 303);
    assert_eq!(answer.location.as_deref(), Some(format!("http://{forge}/work/hello").as_str()), "no Link account hop");
    redirects += usize::from(answer.location.is_some());
    let address = answer.location.expect("repository redirect");
    let answer = browser.call(&address, None)?;
    let landed = Landed { status: answer.status, address, body: answer.body };
    Ok((landed, redirects))
}

fn signed_in(browser: &mut Browser, forge: SocketAddr, login: &str) -> io::Result<()> {
    let settings = browser.call(&format!("http://{forge}/user/settings"), None)?;
    assert_eq!(settings.status, 200);
    assert!(
        std::str::from_utf8(&settings.body).expect("profile page").contains(&format!("value=\"{login}\"")),
        "private profile names the person"
    );
    Ok(())
}

fn password_sign_in(root: &Path, forge: SocketAddr, login: &str) -> io::Result<()> {
    // This story alone reads the throwaway person's password; never print it.
    let password = fs::read_to_string(root.join("forge-1/person_password"))?;
    let mut browser = Browser::default();
    let page = browser.call(&format!("http://{forge}/user/login"), None)?;
    assert_eq!(page.status, 200);
    let csrf = csrf(&page.body).unwrap_or_default();
    let answer = browser.form(
        &format!("http://{forge}/user/login"),
        &[("user_name", login), ("password", password.trim()), ("_csrf", &csrf)],
    )?;
    assert!(matches!(answer.status, 302 | 303));
    signed_in(&mut browser, forge, login)
}

fn external_login(root: &Path, subject: &str) {
    // Existing internal accounts keep their password login_name; the linked
    // provider subject is a separate Forgejo identity, not a credential.
    let output = super::story_command::output(Command::new("python3")
        .args(["-I", "-B", "-c", "import sqlite3,sys; c=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True); print('\\n'.join(x[0] for x in c.execute('select external_id from external_login_user')))"])
        .arg(root.join("forge-1/forgejo/forgejo.db")), "the external identity's subject", super::WITHIN, None);
    success(&output);
    assert_eq!(String::from_utf8(output.stdout).expect("external subject").trim(), subject);
}
