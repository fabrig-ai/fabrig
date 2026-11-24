use super::*;
use crate::directories::jig_directories;
use crate::testing::init_fixtures::{directories, environment, scratch};
use skein_fake_machine::Item;
use std::fs;

use super::startup::{ListenerScript, Startup};

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
    let mut io = ListenerScript::new(preferred, true);
    let held = deployment_ports(&mut io, &preferred).expect("reserved ports");
    let chosen = held.iter().map(|(_, port)| *port).collect::<Vec<_>>();
    assert_ne!(chosen[1].get(), preferred_port);
    for (index, port) in chosen.iter().enumerate() {
        assert!(!chosen[..index].contains(port), "two distinct ports");
    }
    io.finish();
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
    assert_eq!(startup.lookups, [Box::from(b"git".as_slice()), Box::from(b"openssl".as_slice())]);
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
    const BARE_WRITES: &[&str] = &["data/worker.secret", "config/temper.toml"];
    const BOX_WRITES: &[&str] = &["data/worker.secret", "data/demo/bin/git", "config/temper.toml"];

    let root = Path::new("/fixture");
    let dirs = directories(root);
    for (boxed, git, names, sign_in, git_warning, command, demo) in [
        (false, true, BARE_WRITES, "", "", "temper-box serve", None),
        (false, false, BARE_WRITES, "", NO_GIT, "temper-box serve", None),
        (true, true, BOX_WRITES, NO_SIGN_IN, "", "temper box", Some(Is::Directory)),
        (true, false, BOX_WRITES, NO_SIGN_IN, NO_GIT, "temper box", Some(Is::Directory)),
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
