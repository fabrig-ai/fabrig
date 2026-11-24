use super::*;
use crate::directories::jig_directories;
use crate::testing::init_fixtures::{directories, environment, scratch};
use skein_fake_machine::Item;
use std::fs;

use super::startup::{Effect, ListenerScript, Startup};

#[test]
fn path_lookup_uses_only_absolute_executable_candidates_in_order() {
    for (row, entries, executable, expected) in [
        ("absolute", "/one:/two", vec![true, true], Some("/one/git")),
        ("relative skipped", "relative::/two", vec![true], Some("/two/git")),
        ("none", "relative:", vec![], None),
        ("non executable", "/one:/two", vec![false, true], Some("/two/git")),
        ("no hit", "/one", vec![false], None),
    ] {
        let env = vec![(Box::from(b"PATH".as_slice()), Box::from(entries.as_bytes()))];
        let candidates = temper_shell::lookup::candidates(&env, b"git");
        assert_eq!(candidates.len(), executable.len(), "{row}");
        let facts: Vec<_> = candidates.into_iter().zip(executable).collect();
        assert_eq!(temper_shell::lookup::lookup(&env, b"git", &facts).as_deref(), expected.map(Path::new), "{row}");
    }
}
#[test]
fn taken_webhooks_ports_get_distinct_kernel_ports() {
    let preferred_port = 7101;
    let preferred = [0, preferred_port];
    let mut io = ListenerScript::ports([(0, 7100), (preferred_port, 7102)]);
    let held = deployment_ports(&mut io, &preferred).expect("reserved ports");
    let chosen = held.iter().map(|(_, port)| *port).collect::<Vec<_>>();
    assert_ne!(chosen[1].get(), preferred_port);
    for (index, port) in chosen.iter().enumerate() {
        assert!(!chosen[..index].contains(port), "two distinct ports");
    }
    io.finish();
}

fn executable_items(git: bool, claude: bool) -> Vec<Item> {
    let mut items = vec![Item::directory(b"bin")];
    if git {
        items.push(Item::file(b"bin/git", b"").mode(0o755));
    }
    if claude {
        items.push(Item::file(b"bin/claude", b"").mode(0o755));
    }
    items
}

#[test]
fn init_finds_ports_and_programs_and_writes_exact_modes_in_order() {
    for (held, claude, git, same) in [
        (false, false, true, false),
        (false, true, true, false),
        (true, false, true, false),
        (true, true, true, false),
        (false, false, false, false),
        (false, false, true, true),
    ] {
        let row = format!("held={held}, claude={claude}, git={git}, same={same}");
        let preferred = [7300, if same { 7300 } else { 7301 }];
        let available = [!held, !held && !same];
        let bound = [if available[0] { preferred[0] } else { 7400 }, if available[1] { preferred[1] } else { 7401 }];
        let items = executable_items(git, claude);
        let programs = (git, claude);
        let ports = boxed_deployment(&items, preferred, bound, programs, &row);
        for (at, available) in available.into_iter().enumerate() {
            assert_eq!(ports[at] == preferred[at], available, "{row}: preferred port {at}");
        }
        // A fresh tree beside the first; its two requests meet the first's held ports.
        let second = boxed_deployment(&items, ports, [7500, 7501], programs, &row);
        for port in second {
            assert!(!ports.contains(&port), "{row}: deployments have disjoint ports");
        }
    }
}

fn boxed_deployment(
    items: &[Item],
    preferred: [u16; 2],
    bound: [u16; 2],
    (git, claude): (bool, bool),
    row: &str,
) -> [u16; 2] {
    let script = ListenerScript::ports([(preferred[0], bound[0]), (preferred[1], bound[1])]);
    let mut startup = Startup::with_listeners(items, Some(script));
    let written = startup.init_with_preferred(true, None, preferred).expect("boxed init");
    let root = Path::new("/fixture");
    let secret = root.join("data/worker.secret");
    let link = root.join("data/demo/bin/git");
    let config = root.join("config").join(temper_config_shell::FILE);
    assert_eq!(written.paths, [secret.clone(), link.clone(), config.clone()], "{row}: file order");
    assert_eq!(
        startup.effects,
        [
            Effect::Directory(root.join("data"), 0o700),
            Effect::Directory(root.join("cache"), 0o700),
            Effect::Directory(root.join("data/demo"), 0o700),
            Effect::Directory(root.join("data/demo/bin"), 0o700),
            Effect::Directory(root.join("config"), 0o700),
            Effect::Write(secret, 0o600),
            Effect::Link(link, root.join("bin/temper")),
            Effect::Write(config, 0o644),
        ],
        "{row}: effects in order"
    );
    assert_eq!(written.missing_git, !git, "{row}");
    check_lookups(&startup, git, claude, true);
    assert_eq!(startup.kind(&root.join("data/demo/llm.key")), None, "{row}: belongs to playground start");
    assert_eq!(startup.kind(&root.join("config/charters")), None, "{row}");
    let document = startup.document();
    let application = &document["application"];
    assert!(application["clients"]["bundle"].is_null(), "{row}");
    assert_eq!(application["clients"].get("public"), Some(&serde_json::Value::Null), "{row}");
    for omitted in ["workers", "machine", "demo"] {
        assert!(application.get(omitted).is_none(), "{row}: no application.{omitted}");
    }
    assert!(document["engine"].get("llm").is_none(), "{row}: providers are settings");
    check_agent(&document);
    let ports = check_ports(&document, &written, row);
    assert_eq!(ports, bound, "{row}: ports returned by the script");
    ports
}

fn check_lookups(startup: &Startup, git: bool, claude: bool, asks_openssl: bool) {
    let mut expected = vec![
        (b"git".as_slice(), git.then(|| Path::new("/fixture/bin/git"))),
        (b"claude".as_slice(), claude.then(|| Path::new("/fixture/bin/claude"))),
    ];
    if asks_openssl {
        expected.push((b"openssl", None));
    }
    assert_eq!(startup.lookups(), expected);
}

fn check_ports(document: &serde_json::Value, written: &Written, row: &str) -> [u16; 2] {
    let addresses = ["clients", "webhooks"].map(|name| {
        document["application"][name]["listener"]["socket"]
            .as_str()
            .expect("socket")
            .parse::<std::net::SocketAddr>()
            .expect("address")
    });
    let ports = addresses.map(|address| address.port());
    assert_ne!(ports[0], ports[1], "{row}: two distinct reservations");
    assert!(addresses.iter().all(|address| address.ip() == std::net::Ipv4Addr::LOCALHOST), "{row}: loopback");
    let listeners = [11, 12].map(Fd::new).into_iter().zip(ports.map(|port| NonZeroU16::new(port).expect("port")));
    assert_eq!(written.listeners, listeners.collect::<Vec<_>>(), "{row}: reservations match the document");
    ports
}

fn check_agent(document: &serde_json::Value) {
    let agent = temper_agent_fixtures::configuration(document);
    assert_eq!(agent["programs"]["claude_code"]["path"], "fetched");
    let tools = agent["profile"]["declared"]["tool_payload"].as_u64().expect("declared tool payload");
    let bytes = serde_json::to_vec(&agent).expect("agent JSON").into_boxed_slice();
    let prepared = temper_box_kit::prepare_agent(bytes);
    assert_eq!(u64::from(prepared.agent.tool_payload), tools);
}

#[test]
fn init_renders_only_listen_unless_its_next_port_is_taken() {
    use temper_config_shell::{FILE, FORMER, File, Shape, render};
    for (boxed, taken) in [(false, false), (true, false), (true, true)] {
        let row = format!("boxed={boxed}, taken={taken}");
        let port = 7100;
        let next = port + 1;
        let webhooks = if taken { 7102 } else { next };
        let script = ListenerScript::ports([(port, port), (next, webhooks)]);
        let mut startup = Startup::with_listeners(&[], Some(script));
        let written = startup.init_with_preferred(boxed, None, [port, 0]).expect("init");
        assert_eq!(written.listeners[0].1.get(), port, "{row}");
        let actual_webhooks = written.listeners[1].1.get();
        assert_eq!(actual_webhooks, webhooks, "{row}");
        assert_eq!(actual_webhooks == next, !taken, "{row}");
        let expected = File {
            listen: Some(([127, 0, 0, 1], port).into()),
            webhooks: taken.then(|| ([127, 0, 0, 1], webhooks).into()),
            ..File::default()
        };
        let file = Path::new("/fixture/config").join(FILE);
        assert_eq!(startup.read(&file), render(&expected, Shape::Standalone).as_ref(), "{row}: exact bytes");
        assert_eq!(written.paths.last(), Some(&file), "{row}: configuration last");
        assert_eq!(startup.effects.last(), Some(&Effect::Write(file.clone(), 0o644)), "{row}");
        assert_eq!(startup.kind(&file.with_file_name(FORMER)), None, "{row}");
    }
}

#[test]
fn a_bare_configuration_passes_the_same_checks_as_serve() {
    let root = Path::new("/fixture");
    let dirs = directories(root);
    for claude in [false, true] {
        let mut startup = Startup::new(&executable_items(true, claude), true);
        let written = startup.init(false, None).expect("bare init");
        check_lookups(&startup, true, claude, false);
        let read = startup.read_configuration(&dirs).expect("reread as serve");
        let deployment = crate::setup::check(&read.document, &jig_directories(&dirs)).expect("passes setup and named");
        assert!(deployment.demo.is_some(), "every box supplies its scripted provider");
        let document: serde_json::Value = serde_json::from_slice(&read.document).expect("JSON");
        for omitted in ["machine", "welcome", "demo"] {
            assert!(document["application"].get(omitted).is_none(), "no application.{omitted}");
        }
        assert_eq!(startup.kind(&root.join("data/demo")), None);
        assert_eq!(written.person, temper_init::Person::Found);
        assert!(written.login.is_none());
        assert_eq!(
            written.paths,
            [root.join("data/worker.secret"), root.join("config").join(temper_config_shell::FILE)]
        );
        check_ports(&document, &written, &format!("bare, claude={claude}"));
        check_agent(&document);
    }
}

#[test]
fn init_keeps_an_existing_provider_key_even_without_openssl() {
    let root = Path::new("/fixture");
    let key = root.join("data/provider.key");
    let mut items = executable_items(true, false);
    items.extend([Item::directory(b"data").mode(0o700), Item::file(b"data/provider.key", b"kept").mode(0o600)]);
    let mut startup = Startup::new(&items, true);
    let held = startup.hold(&key);
    let before = startup.stat(held);
    let written = startup.init(true, None).expect("init keeps the existing key");
    // An unlink or replacement would leave the held node without its name.
    assert_eq!(startup.stat(held), before);
    let retained = startup.hold(&key);
    assert_eq!(startup.stat(retained).size, before.size);
    startup.release(retained);
    startup.release(held);
    check_lookups(&startup, true, false, false);
    assert_eq!(
        startup.effects.iter().filter(|effect| matches!(effect, Effect::Write(..))).collect::<Vec<_>>(),
        [
            &Effect::Write(root.join("data/worker.secret"), 0o600),
            &Effect::Write(root.join("config").join(temper_config_shell::FILE), 0o644),
        ],
        "no write to the retained key or a temporary file"
    );
    assert!(written.no_sign_in.is_none());
    let document = startup.document();
    assert_eq!(document["engine"]["clients"]["provider"]["key"], "provider.key");
    assert_eq!(document["engine"]["clients"]["provider"]["clients"], serde_json::json!([]));
}

#[test]
fn init_refusals_leave_every_file_and_directory_untouched() {
    for (row, directory, parents, dangling) in [
        (temper_config_shell::FILE, "config", vec![b"config".as_slice()], false),
        (temper_config_shell::FORMER, "config", vec![b"config"], false),
        ("worker.secret", "data", vec![b"data"], false),
        ("demo/bin/git", "data", vec![b"data", b"data/demo", b"data/demo/bin"], true),
    ] {
        let destination = format!("{directory}/{row}");
        let destination = destination.as_bytes();
        let mut items: Vec<_> = parents.into_iter().map(Item::directory).collect();
        if dangling {
            items.push(Item::link(destination, b"/nonexistent/temper-init-demo"));
        } else {
            items.push(Item::file(destination, b"kept"));
        }
        let mut startup = Startup::new(&items, false);
        let before = startup.snapshot();
        let refused = format!("/fixture/{}: already exists", String::from_utf8_lossy(destination));
        if !dangling {
            assert_eq!(startup.init(false, None).expect_err("bare refused"), refused);
            assert_eq!(startup.snapshot(), before, "{row}: bare refusal changes nothing");
        }
        assert_eq!(startup.init(true, None).expect_err("refused"), refused);
        assert_eq!(startup.snapshot(), before, "{row}: every file and directory untouched");
    }
}

#[test]
fn init_leaves_an_existing_playground_key_alone() {
    let mut startup = Startup::new(
        &[
            Item::directory(b"data").mode(0o700),
            Item::directory(b"data/demo").mode(0o700),
            Item::file(b"data/demo/llm.key", b"kept"),
        ],
        true,
    );
    startup.init(true, None).expect("key is not an init destination");
    assert_eq!(startup.read(Path::new("/fixture/data/demo/llm.key")), b"kept");
}

#[test]
fn generated_configuration_passes_the_same_checks_as_serve() {
    let root = scratch("acceptance");
    let dirs = directories(&root);
    let env = environment(&root, true);
    let written = write(&dirs, &env, &root.join("bin/temper"), [0; 2]).expect("init");
    let bytes = temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file").document;
    let deployment =
        crate::setup::check(&bytes, &jig_directories(&dirs)).expect("init document passes setup and named");
    let document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(document["application"].get("machine").is_none());
    let demo = deployment.demo.expect("init declares the demo");
    assert_eq!(demo.provider(), 3);
    let mut providers = temper_service::stock::providers();
    for provider in &mut providers {
        provider.provider.sign_in = None;
        provider.provider.signed_in_plan = false;
        for endpoint in &mut provider.endpoints {
            endpoint.address = jig_protocol_config::EndpointAddress::Socket(([127, 0, 0, 1], 9).into());
            endpoint.transport = jig_protocol_config::Transport::Plaintext;
        }
    }
    let given = temper_service::stock::Given::Stock.with_providers(providers, Box::new([]));
    crate::prepare_in_with(&env, b"/", &dirs, &given).expect("a box prepares from init's file");
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn the_person_diagnostic_names_the_login_or_the_uid() {
    use temper_init::Person::{FellBack, Found};
    for (person, login, expected) in [
        (Found, Some(b"ada".as_slice()), None),
        (
            FellBack,
            Some(b"Admin".as_slice()),
            Some("init: Admin cannot be a Forgejo login: the owned Forgejo's person is person"),
        ),
        (FellBack, None, Some("init: uid 42 has no login in /etc/passwd: the owned Forgejo's person is person")),
    ] {
        assert_eq!(person_line(person, login, 42).as_deref(), expected);
    }
}

#[test]
fn explicit_program_fact_precedes_path_discovery() {
    let mut startup = Startup::new(
        &[Item::directory(b"bin"), Item::file(b"bin/git", b"").mode(0o755), Item::file(b"bin/claude", b"").mode(0o755)],
        true,
    );
    startup.init(true, Some(Path::new("/fixture/fake-claude"))).expect("injected init fact");
    assert_eq!(
        startup.lookups(),
        [(b"git".as_slice(), Some(Path::new("/fixture/bin/git"))), (b"openssl".as_slice(), None)]
    );
    let document = startup.document();
    assert!(!document["application"].as_object().unwrap().contains_key("workers"));
    assert!(!startup.exists(Path::new("/fixture/config/agent.json")));
}

#[test]
fn a_path_without_openssl_writes_a_box_without_sign_in_and_keeps_running() {
    let mut startup = Startup::new(&[Item::directory(b"bin"), Item::file(b"bin/git", b"").mode(0o755)], true);
    let written = startup.init(true, None).expect("init succeeds without openssl");
    assert_eq!(written.no_sign_in.as_deref(), Some("openssl is not on PATH"));
    let value = startup.document();
    assert!(value["engine"]["clients"]["provider"].is_null());
    assert!(value["application"].get("machine").is_none());
    assert!(!startup.exists(Path::new("/fixture/data/provider.key")));
}

#[test]
fn init_reports_its_writes_and_start_command_on_stderr() {
    use skein_fake_machine::Is;
    use std::fmt::Write;

    const ACCOUNT: &str = "init: after starting, continue with ChatGPT or add an OpenAI API key from the accounts page";
    const NO_GIT: &str =
        "init: no git on PATH: a real forge's repositories need git 2.41 or later, set as [programs] git\n";
    const NO_SIGN_IN: &str = "init: no sign-in through temper: openssl is not on PATH\n";

    let config_write = format!("config/{}", temper_config_shell::FILE);
    let bare_writes: &[&str] = &["data/worker.secret", &config_write];
    let box_writes: &[&str] = &["data/worker.secret", "data/demo/bin/git", &config_write];

    let root = Path::new("/fixture");
    let dirs = directories(root);
    for (boxed, git, names, sign_in, git_warning, command, demo) in [
        (false, true, bare_writes, "", "", "temper-box serve", None),
        (false, false, bare_writes, "", NO_GIT, "temper-box serve", None),
        (true, true, box_writes, NO_SIGN_IN, "", "temper box", Some(Is::Directory)),
        (true, false, box_writes, NO_SIGN_IN, NO_GIT, "temper box", Some(Is::Directory)),
    ] {
        let mut items = vec![Item::directory(b"bin")];
        if git {
            items.push(Item::file(b"bin/git", b"").mode(0o755));
        }
        let mut startup = Startup::new(&items, true);
        let written = startup.init(boxed, None).expect("init succeeds");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut closed = Vec::new();
        let code = finish_written(&dirs, boxed, written, true, || 42, (&mut stdout, &mut stderr), |fds| closed = fds);
        assert_eq!(code, 0);
        assert_eq!(stdout, [] as [u8; 0]);
        assert_eq!(closed, [11, 12].map(Fd::new));

        let mut expected = String::new();
        for name in names {
            writeln!(expected, "init: wrote {}", root.join(name).display()).expect("string");
        }
        write!(expected, "{sign_in}{git_warning}{ACCOUNT}\ninit: start it with {command} --config /fixture/config --data /fixture/data --cache /fixture/cache\n").expect("string");
        let lines = String::from_utf8(stderr).expect("init lines");
        assert_eq!(lines, expected, "boxed={boxed}, git={git}");

        let document = startup.document();
        assert!(document["application"].get("workers").is_none());
        assert!(document["application"].get("welcome").is_none());
        assert_eq!(startup.kind(&root.join("data/demo")), demo);
    }
}
