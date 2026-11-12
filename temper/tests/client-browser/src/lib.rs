//! The browser tier (client/web/testing.md, 3): Chromium against `temper serve`, on a scratch deployment whose
//! forge and LLM endpoint are unreached, or the box's first-run demo with its own fakes, serving the
//! bundle scripts/browser.sh builds. Its helpers are jig's
//! `tests/client-browser/src/lib.rs`'s, for temper's binary and bundle.
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{
    Setup, Step,
    harness::{Names, Output, Printed},
};
use skein_ui::Role;
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
};
mod authorize;
mod configuration;
mod forgery;
mod github;
mod history;
mod issuer;
mod public;
pub mod reading;
mod story;
pub use history::reader_opens;
pub use story::Stories;

/// How long a story waits for what a page shows.
pub const WITHIN: skein_lib::Duration = skein_lib::Duration::from_secs(10);
/// How long a story watches for what a page must not show.
pub const ABSENT: skein_lib::Duration = skein_lib::Duration::from_secs(1);
/// The first chat's link on the Chats page.
pub const FIRST_CHAT: &[u8] = b"T1";
/// Its address.
pub const FIRST_CHAT_ADDRESS: &[u8] = b"/chats/1";

const PAGE_QUERY: &[u8] = b"internal-loopback-navigation=1&internal-small-window=1";

/// The page's addresses (client/web/testing.md, 3), excluding the service's
/// GitHub routes and their closed queries (client/forge/served.md, 1).
fn is_page(address: &[u8]) -> bool {
    let address = address.split(|byte| *byte == b'?' || *byte == b'#').next().expect("address before query");
    let path = if let Some(scheme) = skein_lib::bytes::find(address, b"://") {
        let authority = &address[scheme + 3..];
        authority.iter().position(|byte| *byte == b'/').map_or(b"".as_slice(), |at| &authority[at..])
    } else {
        address
    };
    let Some(forge) = path.strip_prefix(b"/forges/") else { return true };
    let digits = forge.iter().take_while(|byte| byte.is_ascii_digit()).count();
    digits == 0 || !matches!(&forge[digits..], b"/github/manifest" | b"/github/redirect" | b"/github/setup")
}

/// A fresh load, adding the tier's settings only to the page (client/web/testing.md, 3).
#[must_use]
pub fn page_address(address: &[u8]) -> Box<[u8]> {
    skein_fake_person_browser::harness::page_with_query(address, PAGE_QUERY, is_page)
}

/// The tier's printed and command-opened pages carry the same settings as explicit loads.
#[must_use]
pub fn harness(setup: Setup<'_>) -> skein_fake_person_browser::Harness {
    skein_fake_person_browser::Harness::new(setup).page_query(PAGE_QUERY).page_addresses(is_page)
}

/// The `temper-box` binary beside the test executable.
#[must_use]
pub fn temper() -> PathBuf {
    binary("temper-box")
}

/// The static host beside the test executable (temper/docs/design/testing/tools.md, 5).
#[must_use]
pub fn playground_serve() -> PathBuf {
    binary("temper-playground-serve")
}

fn binary(name: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("test executable");
    let binary = exe.parent().and_then(Path::parent).expect("target directory").join(name);
    assert!(binary.is_file(), "build the browser server with cargo build -p {name}: {}", binary.display());
    binary
}
#[must_use]
pub fn chromium() -> PathBuf {
    if let Some(path) = temper_test_env::value(&temper_test_env::SKEIN_TEST_CHROMIUM) {
        let path = PathBuf::from(path);
        assert!(path.is_file(), "SKEIN_TEST_CHROMIUM must name a file");
        return path.canonicalize().expect("configured Chromium");
    }
    for directory in std::env::split_paths(&temper_test_env::value(&temper_shell::environment::PATH).expect("PATH")) {
        let path = directory.join("chromium");
        if path.is_file() {
            return path.canonicalize().expect("Chromium on PATH");
        }
    }
    panic!("Chromium is required: set SKEIN_TEST_CHROMIUM or put chromium on PATH");
}
/// The bundle scripts/browser.sh builds, named by `TEMPER_TEST_BUNDLE`.
#[must_use]
pub fn bundle() -> PathBuf {
    let path = PathBuf::from(
        temper_test_env::value(&temper_test_env::BUNDLE)
            .expect("run scripts/browser.sh to build the bundle and set TEMPER_TEST_BUNDLE"),
    );
    let path = if path.is_absolute() { path } else { skein_scratch::paths::tree().join(path) };
    assert!(
        path.join("index.html").is_file(),
        "build the bundle with scripts/bundle.sh {}: it holds no index.html",
        path.display()
    );
    path.canonicalize().expect("absolute bundle")
}
/// The heading of the first chat a story starts: its number and its first line, `title`.
#[must_use]
pub fn first_chat(title: &[u8]) -> Find {
    Find::exactly_named(Role::Heading { level: 1 }, &[b"T1 \xc2\xb7 ".as_slice(), title].concat())
}
/// Open the rail's new box and choose a change (temper/docs/design/client/view.md, 3).
#[must_use]
pub fn start_change(drafts: bool) -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |role, name: &[u8]| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    if drafts {
        vec![
            see(Find::exactly_named(Role::Button, b"New draft or change")),
            press(Role::Button, b"New draft or change"),
            see(Find::exactly_named(Role::MenuItem, b"Change")),
            press(Role::MenuItem, b"Change"),
        ]
    } else {
        vec![see(Find::exactly_named(Role::Link, b"New change")), press(Role::Link, b"New change")]
    }
}
/// From the welcome's Forge step in a world with a usable model, navigate to `/chats`,
/// open `NewChat` and start the first chat with `words`, one line, and see it titled by them.
#[must_use]
pub fn first_chat_steps(words: &[u8]) -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let composer = || Find::exactly_named(Role::TextArea, b"What do you want done?");
    let mut steps = vec![
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Connect a forge")),
        Step::Person(PersonStep::Go { address: crate::page_address(b"/chats".as_slice()) }),
        see(live()),
        connected(),
    ];
    steps.extend(start_change(true));
    steps.extend([
        see(composer()),
        Step::Person(PersonStep::Type { find: composer(), words: Box::from(words) }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Send") }),
        see(first_chat(words)),
    ]);
    steps
}
/// Choose the playground at the printed link's welcome and await its ready project.
#[must_use]
pub fn playground_ready() -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    vec![
        see(Find::exactly_named(Role::Button, b"Try the playground")),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Try the playground") }),
        see(Find::exactly_named(Role::Region, b"Playground")),
        Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Status, b"Starting"), within: WITHIN }),
        see(Find::exactly_named(Role::Status, b"Ready")),
        see(live()),
        connected(),
    ]
}
/// From the ready playground, open the empty Chats list and start a change with `words`.
#[must_use]
pub fn playground_chat(words: &[u8]) -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let composer = || Find::exactly_named(Role::TextArea, b"What do you want done?");
    // With no chat that started work, Work has no Chats filter.
    let mut steps = vec![
        Step::Person(PersonStep::Go { address: crate::page_address(b"/chats".as_slice()) }),
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
    ];
    steps.extend(start_change(true));
    steps.extend([
        see(Find::exactly_named(Role::Heading { level: 1 }, b"New chat")),
        see(composer()),
        Step::Focused(composer()),
        Step::Address(b"/chats/new".as_slice().into()),
        Step::Type { find: composer(), words: words.into() },
        Step::Key(skein_browser::boundary::Key::Enter),
        see(first_chat(words)),
    ]);
    steps
}
fn live() -> Find {
    Find::exactly_named(Role::Button, b"Owner")
}
/// Read the positive Live DOM marker, which the project rail hides from accessibility
/// (temper/docs/design/client/view.md, 3), after the page's readiness observation.
#[must_use]
pub fn connected() -> Step {
    Step::Keep {
        selector: b"[data-region='temper-link'] > [role='status'].temper-live".as_slice().into(),
        attribute: Some(b"class".as_slice().into()),
        name: b"live-link".as_slice().into(),
    }
}

/// A scratch deployment for one story's `temper serve` or `temper box`.
pub struct Tier {
    scratch: skein_scratch::Scratch,
    forged: Box<[u8]>,
    command: &'static str,
}
impl Tier {
    #[must_use]
    pub fn new(name: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(name);
        configuration::write(scratch.path(), &temper(), &bundle());
        Self { scratch, forged: forgery::request(), command: "serve" }
    }
    /// Give the deployment a usable model without a credential or a reachable LLM peer.
    #[must_use]
    pub fn with_model(self) -> Self {
        let path = self.scratch.path().join("temper.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        // As in jig's browser deployment, a provider offering no credential source gets a
        // deployment-held None account at startup: the view's demo_accounts fixture has this shape.
        value["engine"]["llm"]["providers"][0]["sources"] = serde_json::json!({});
        configuration::private(&path, &serde_json::to_vec(&value).expect("model deployment JSON"));
        self
    }
    /// Offer the stock models and their efforts to the composer, with matching worker bounds.
    #[must_use]
    pub fn with_model_picker(self) -> Self {
        self
    }
    /// The first-run demo, with the box's own fake LLM and forge, started when the playground is chosen.
    #[must_use]
    pub fn demo(name: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(name);
        configuration::demo(scratch.path(), &temper(), &bundle());
        Self { scratch, forged: forgery::request(), command: "" }
    }
    /// The demo with reading's replies on a private bundle, whose stylesheet turns the browser's
    /// anchoring off at `reading::anchoring_off`'s height alone (temper/docs/design/client/web/testing.md, 3).
    #[must_use]
    pub fn reading(name: &str) -> Self {
        let tier = Self::demo(name);
        let private_bundle = tier.scratch.path().join("bundle");
        copy_bundle(&bundle(), &private_bundle);
        let digest = fs::read_dir(&private_bundle)
            .expect("bundle entries")
            .map(|entry| entry.expect("bundle entry"))
            .find(|entry| entry.file_type().expect("bundle entry type").is_dir())
            .expect("digest directory");
        let path = digest.path().join("temper.css");
        let mut css = fs::read(&path).expect("bundle stylesheet");
        css.extend_from_slice(
            b"\n@media (height: 801px) { #app > .temper-session .temper-thread > [role=log] { overflow-anchor: none !important; margin-top: 7px !important; } }\n",
        );
        let mut compressed = vec![0; skein_http::gzip::bound(css.len()).expect("stylesheet gzip bound")];
        let length = skein_http::gzip::encode(&css, &mut compressed).expect("compressed private stylesheet");
        compressed.truncate(length);
        fs::write(path.with_extension("css.gz"), compressed).expect("private compressed stylesheet");
        fs::write(path, css).expect("private stylesheet");
        let path = tier.scratch.path().join("temper.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        value["application"]["clients"]["bundle"] = serde_json::json!(private_bundle);
        configuration::private(&path, &serde_json::to_vec(&value).expect("private bundle JSON"));
        tier.with_replies(&reading::replies())
    }
    /// Script the demo model's replies, with each cue selecting one whole answer.
    #[must_use]
    fn with_replies(self, replies: &[(String, String)]) -> Self {
        let path = self.scratch.path().join("temper.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        reading::configure(&mut value, replies);
        configuration::private(&path, &serde_json::to_vec(&value).expect("scripted demo JSON"));
        self
    }
    /// The box's demo with one suggested request's scripted turns replaced.
    #[must_use]
    pub fn demo_asked(name: &str, cue: &str, turns: serde_json::Value) -> Self {
        let tier = Self::demo(name);
        configuration::demo_asked(tier.scratch.path(), cue, turns);
        tier
    }
    /// Account offers with a static public key set, served by the same scratch server before Visit.
    #[must_use]
    pub fn accounts(name: &str) -> Self {
        let tier = Self::reserved(name);
        let path = tier.scratch.path().join("temper.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        let private_bundle = tier.scratch.path().join("bundle");
        copy_bundle(&bundle(), &private_bundle);
        let digest = fs::read_dir(&private_bundle)
            .expect("bundle entries")
            .map(|entry| entry.expect("bundle entry"))
            .find(|entry| entry.file_type().expect("bundle entry type").is_dir())
            .expect("digest directory");
        // Public, synthetic 2048-bit RSA modulus; no callback or signature is exercised here.
        let keys = serde_json::json!({"keys":[{"kty":"RSA","n":format!("g{}", "A".repeat(341)),"e":"AQAB"}]});
        fs::write(digest.path().join("issuer.webmanifest"), serde_json::to_vec(&keys).expect("key set"))
            .expect("static issuer key set");
        let socket = value["application"]["clients"]["listener"]["socket"].as_str().expect("reserved socket");
        let keys =
            format!("http://{socket}/bundle/{}/issuer.webmanifest", digest.file_name().to_str().expect("digest"));
        value["application"]["clients"]["bundle"] = serde_json::json!(private_bundle);
        value["engine"]["limits"]["overrides"]["accounts.sign_in_deadline_ms"] = 2000.into();
        let mut sign_in = account_registration("http://127.0.0.1:9");
        sign_in["keys"] = keys.into();
        account_provider(&mut value, &sign_in);
        configuration::private(&path, &serde_json::to_vec(&value).expect("accounts deployment JSON"));
        tier
    }
    /// A real loopback issuer for the tab's complete sign-in round trip.
    #[must_use]
    pub fn accounts_sign_in(name: &str) -> (Self, skein_fake_peers::oauth::Peer) {
        use std::os::fd::{AsRawFd, IntoRawFd};
        let tier = Self::reserved(name);
        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve issuer port");
        let address = listener.local_addr().expect("issuer address");
        let origin = format!("http://{address}");
        let path = tier.scratch.path().join("temper.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        let sign_in = account_registration(&origin);
        account_provider(&mut value, &sign_in);
        welcome_provider(&mut value, &sign_in);
        configuration::private(&path, &serde_json::to_vec(&value).expect("accounts deployment JSON"));
        let mut peer = issuer::peer(address, &sign_in);
        peer.adopt_listener(skein_io::kernel::Fd::new(listener.as_raw_fd()), address).expect("held issuer listener");
        let _ = listener.into_raw_fd();
        (tier, peer)
    }
    /// A GitHub forge on the browser harness's loop, with a fixed clients' listener.
    #[must_use]
    pub fn github(name: &str) -> (Self, skein_fake_peers::forge::Peer) {
        Self::github_repositories(name, 1, 1)
    }
    /// A fake GitHub with enough repositories to read on by scrolling the sheet.
    #[must_use]
    pub fn github_repositories(name: &str, repositories: u32, page: u32) -> (Self, skein_fake_peers::forge::Peer) {
        use std::os::fd::{AsRawFd, IntoRawFd};
        let tier = Self::reserved(name);
        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve GitHub port");
        let address = listener.local_addr().expect("GitHub address");
        let path = tier.scratch.path().join("temper.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        value["application"]["machine"]["github"] = serde_json::json!({
            "api":{"socket":address.to_string()}, "transport":"plaintext",
            "remote":format!("http://{address}"), "web":format!("http://{address}")
        });
        github::project_creation(&mut value, address);
        value["application"]["limits"]["overrides"]["picker.page"] = page.into();
        // repo-NN is longer than the single-repository fixture's app.
        value["engine"]["limits"]["overrides"]["people.amendment_bytes"] = 2048.into();
        configuration::private(&path, &serde_json::to_vec(&value).expect("GitHub deployment JSON"));
        let mut peer = github::peer_repositories(address, repositories, page);
        peer.adopt_listener(skein_io::kernel::Fd::new(listener.as_raw_fd()), address).expect("held GitHub listener");
        let _ = listener.into_raw_fd();
        (tier, peer)
    }
    /// Sign in ctl with the held server link, then open its printed manifest link in the tab.
    #[must_use]
    pub fn github_connect(&self) -> Vec<Step> {
        use skein_fake_person_browser::{Command, Opens, Stdin};
        let command = |words: &[&str], stdin, opens| {
            let mut args = vec!["ctl".into(), self.scratch.path().join("client").into_os_string()];
            args.extend(words.iter().map(std::ffi::OsString::from));
            Step::Run(Command { program: temper(), args, env: vec![], stdin, opens, within: WITHIN })
        };
        vec![
            command(&["sign-in", "-"], Stdin::Printed, None),
            Step::Exited { code: 0, within: WITHIN },
            command(
                &["forge", "connect", "github", "--key", "github-connect"],
                Stdin::Closed,
                Some(Opens {
                    prefix: b"http://127.0.0.1:".as_slice().into(),
                    on: Output::Stdout,
                    keep: b"manifest-link".as_slice().into(),
                }),
            ),
            Step::Exited { code: 0, within: WITHIN },
        ]
    }
    /// A tier whose server listens at a loopback port reserved beforehand, so that a restart opens the same
    /// address and a page reconnects to it. Jig's tier has the same hook.
    #[must_use]
    pub fn reserved(name: &str) -> Self {
        let tier = Self::new(name);
        let address = temper_fixtures::ports::addresses::<1>(&[])[0];
        let path = tier.scratch.path().join("temper.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        value["application"]["clients"]["listener"]["socket"] = serde_json::json!(address.to_string());
        configuration::private(&path, &serde_json::to_vec(&value).expect("reserved deployment JSON"));
        tier
    }
    /// The box's serve step, on this deployment's directories rather than its startup command.
    #[must_use]
    pub fn box_serve(&self) -> Vec<std::ffi::OsString> {
        let mut args = vec!["internal".into(), "serve".into()];
        args.extend(configuration::flags(self.scratch.path()));
        args.push("--box".into());
        args
    }
    #[must_use]
    pub fn setup(&self) -> Setup<'_> {
        let mut args = Vec::new();
        if !self.command.is_empty() {
            args.push(self.command.into());
        }
        args.extend(configuration::flags(self.scratch.path()));
        Setup {
            server: temper(),
            args: args.clone(),
            restart: args,
            printed: Printed {
                sent: skein_fake_person_browser::harness::Sent::Page,
                prefix: b"http://127.0.0.1:",
                on: Output::Stdout,
                names: Names::Url,
            },
            chromium: chromium(),
            downloads: None,
            expected: vec![],
            origins: vec![],
        }
    }
    /// Owner observes the signed-in document; `connected` checks Live even when hidden.
    /// Offline and signed-out states remain visible.
    #[must_use]
    pub fn states(&self) -> jig_client_fake_person_browser::States {
        jig_client_fake_person_browser::States {
            live: live(),
            offline: Find::exactly_named(Role::Status, b"Offline"),
            signed_out: Find::text(b"To sign in, open a sign-in link temper printed."),
        }
    }
    /// A forged chat would show its words on the chats page if accepted.
    #[must_use]
    pub fn target(&self) -> jig_client_fake_person_browser::Target<'_> {
        jig_client_fake_person_browser::Target {
            prefix: b"/jig",
            page: b"/chats",
            request: &self.forged,
            done: Find::text(b"Forged"),
        }
    }
}

/// Public registration shared by the browser's account fixtures.
fn account_registration(origin: &str) -> serde_json::Value {
    serde_json::json!({
        "issuer":origin, "client":"temper-test", "scopes":["openid","chat"],
        "authorization":format!("{origin}/authorize"), "token":format!("{origin}/token"),
        "revocation":format!("{origin}/revoke"), "keys":format!("{origin}/keys"),
        "transport":"plaintext", "must_grant":["chat"], "redirect_path":"callback"
    })
}

/// The two account stories share their provider and room for its driver.
fn account_provider(value: &mut serde_json::Value, sign_in: &serde_json::Value) {
    // One git operation leaves the small profile's sockets room for the accounts driver.
    value["application"]["limits"]["overrides"]["git_component.operations"] = 1.into();
    value["application"]["limits"]["overrides"]["checkout.workspaces"] = 1.into();
    value["engine"]["llm"]["providers"] = serde_json::json!([{
        "name":"Test provider", "number":1, "signed_in_plan":false, "endpoint":"fake",
        "sources":{"keys":true,"sign_in":sign_in}
    }]);
}

/// The welcome's own provider on the same issuer, whose one subject would renew on the first
/// provider (temper/docs/design/client/web/testing.md, 2). Provider 2 has its own stock models.
fn welcome_provider(value: &mut serde_json::Value, sign_in: &serde_json::Value) {
    value["engine"]["llm"]["providers"].as_array_mut().expect("providers").push(serde_json::json!({
        "name":"Welcome provider", "number":2, "signed_in_plan":false, "endpoint":"fake",
        "sources":{"keys":true,"sign_in":sign_in}
    }));
}

/// Keep the built bundle immutable while giving this one-serve story its static issuer document.
fn copy_bundle(from: &Path, to: &Path) {
    fs::create_dir(to).expect("private bundle directory");
    for entry in fs::read_dir(from).expect("built bundle") {
        let entry = entry.expect("bundle entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("bundle entry type").is_dir() {
            copy_bundle(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy bundle file");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::configuration::{UNREACHED, documents};
    use std::path::Path;
    #[test]
    fn the_page_excludes_only_the_three_decimal_github_routes() {
        for (path, page) in [
            ("/forges/1/github/manifest", false),
            ("/forges/12/github/redirect", false),
            ("/forges/0/github/setup", false),
            ("/forges", true),
            ("/forges/connect?forge=1", true),
            ("/chats", true),
            ("/", true),
            ("/projects/1/settings/repositories", true),
            ("/forges/1/github", true),
            ("/forges//github/manifest", true),
            ("/forges/1x/github/manifest", true),
            ("/forges/1/github/manifest/", true),
            ("/forges/1/github/setup/other", true),
        ] {
            for prefix in ["", "http://127.0.0.1:9", "http://127.0.0.1:41000"] {
                for suffix in ["", "?state=abc", "#t", "?x=1#t"] {
                    let address = format!("{prefix}{path}{suffix}");
                    assert_eq!(super::is_page(address.as_bytes()), page, "{address}");
                    let opened = super::page_address(address.as_bytes());
                    let expected = if page {
                        skein_fake_person_browser::harness::with_query(address.as_bytes(), super::PAGE_QUERY)
                    } else {
                        address.as_bytes().into()
                    };
                    assert_eq!(opened, expected, "{address}");
                }
            }
        }
    }

    fn tier_configuration() -> (serde_json::Value, serde_json::Value) {
        let (agent, value) =
            documents(Path::new("/private/test"), Path::new("/test/temper"), Path::new("/test/bundle"));
        (serde_json::from_slice(&agent).expect("agent JSON"), serde_json::from_slice(&value).expect("deployment JSON"))
    }
    fn stock_defaults(value: &serde_json::Value) -> jig_domain_core::settings::Defaults {
        let document = jig_protocol_config::document(
            &serde_json::to_vec(&value["engine"]).expect("engine JSON"),
            &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
        )
        .expect("engine document");
        let engine = temper_engine_service::startup::engine(&document, 0, &jig_protocol_config::Directories::working())
            .expect("account deployment admits stock settings");
        let core = jig_protocol_config::core(&engine, [0; 16], 1);
        core.defaults
    }
    #[test]
    fn the_tier_passes_startup_validation_with_stock_defaults() {
        let (_, deployment) =
            documents(Path::new("/private/test"), Path::new("/test/temper"), Path::new("/test/bundle"));
        let shape = temper_service::startup::shape(&deployment, &jig_protocol_config::Directories::working())
            .expect("browser deployment admits stock settings");
        assert_eq!(shape, temper_service_core::startup::Shape::Standalone);
    }
    #[test]
    fn the_accounts_provider_keeps_the_stock_offer() {
        let mut value = tier_configuration().1;
        super::account_provider(&mut value, &super::account_registration("http://127.0.0.1:9"));
        let defaults = stock_defaults(&value);
        assert_eq!(defaults.models.len(), 4);
        assert!(defaults.models.iter().all(|entry| entry.model.provider == 1));
    }
    #[test]
    fn the_welcome_provider_has_its_own_stock_offer() {
        let mut value = tier_configuration().1;
        let sign_in = super::account_registration("http://127.0.0.1:9");
        super::account_provider(&mut value, &sign_in);
        super::welcome_provider(&mut value, &sign_in);
        let defaults = stock_defaults(&value);
        assert!(defaults.models.iter().any(|entry| entry.model.provider == 2));
    }
    #[test]
    fn the_sign_in_tiers_two_providers_pass_startup_validation() {
        let (agent, mut value) = tier_configuration();
        let sign_in = super::account_registration("http://127.0.0.1:9");
        super::account_provider(&mut value, &sign_in);
        super::welcome_provider(&mut value, &sign_in);
        let shape = temper_service::startup::shape(
            &serde_json::to_vec(&value).expect("two-provider deployment JSON"),
            &jig_protocol_config::Directories::working(),
        )
        .expect("browser deployment admits two sign-in providers");
        assert_eq!(shape, temper_service_core::startup::Shape::Standalone);
        let engine = super::configuration::stock_engine(&value);
        let core = jig_protocol_config::core(&engine, [0; 16], 1);
        let defaults = &core.defaults;
        let bound = &agent["endpoints"][0]["limits"];
        assert_eq!(defaults.charters.len(), 9);
        for charter in &defaults.charters {
            let resolved =
                jig_domain_core::settings::charters::resolved(charter, &defaults.models, &engine.limits.core)
                    .expect("two-provider stock charter resolves");
            for model in std::iter::once(&resolved.policy.model).chain(resolved.policy.alternatives.iter()) {
                assert!(bound["window"].as_u64().expect("window") >= u64::from(model.window));
                assert!(bound["output"].as_u64().expect("output") >= u64::from(model.output));
            }
        }
    }
    #[test]
    fn the_tier_names_the_bundle_for_its_server() {
        assert_eq!(tier_configuration().1["application"]["clients"]["bundle"], "/test/bundle");
    }
    #[test]
    fn the_tier_listens_on_loopback_at_a_dynamic_port() {
        assert_eq!(tier_configuration().1["application"]["clients"]["listener"]["socket"], "127.0.0.1:0");
    }
    #[test]
    fn the_tier_starts_without_a_kept_forge() {
        let value = tier_configuration().1;
        assert!(value["application"]["machine"]["github"].is_null());
        assert!(value["application"].get("forges").is_none());
    }
    #[test]
    fn the_tiers_llm_endpoint_is_unreached() {
        let (agent, value) = tier_configuration();
        assert_eq!(value["engine"]["llm"]["endpoints"][0]["address"]["socket"], UNREACHED);
        assert_eq!(agent["endpoints"][0]["address"], UNREACHED);
    }
    #[test]
    fn the_tiers_paths_are_absolute() {
        let value = tier_configuration().1;
        assert_eq!(value["application"]["workers"][0]["root"], "/private/test/worker-root");
    }
}
