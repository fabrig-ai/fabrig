use super::*;
use crate::directories::jig_directories;
use crate::testing::init_fixtures::{directories, environment, scratch};
use std::{fs, net::TcpListener, os::unix::fs::MetadataExt};

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
    for at in [1] {
        let holder = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("hold preferred port");
        let preferred_port = holder.local_addr().expect("held address").port();
        let mut io = Io::open().expect("startup io");
        let mut preferred = [0; 2];
        preferred[at] = preferred_port;
        let held = io.ports(&preferred).expect("reserved ports");
        let chosen = held.iter().map(|(_, port)| *port).collect::<Vec<_>>();
        assert_ne!(chosen[at].get(), preferred_port);
        for (index, port) in chosen.iter().enumerate() {
            assert!(!chosen[..index].contains(port), "two distinct ports");
        }
        for (fd, _) in held {
            io.close(fd);
        }
        drop(holder);
    }
}

#[test]
fn init_refusals_leave_every_file_and_directory_untouched() {
    for (row, data, dangling) in [
        (temper_config_shell::FILE, false, false),
        (temper_config_shell::FORMER, false, false),
        ("worker.secret", true, false),
        ("demo/bin/git", true, true),
    ] {
        let root = scratch(&row.replace('/', "-"));
        let dirs = directories(&root);
        let env = environment(&root, false);
        let destination = path(if data { &dirs.data } else { &dirs.config }).join(row);
        fs::create_dir_all(destination.parent().expect("parent")).expect("parent");
        if dangling {
            std::os::unix::fs::symlink("/nonexistent/temper-init-demo", &destination).expect("dangling link");
        } else {
            fs::write(&destination, b"kept").expect("existing file");
        }
        let before = snapshot(&root);
        if !row.starts_with("demo/") {
            let why = write_bare(&dirs, &env, &root.join("bin/temper"), [0; 2]).expect_err("bare refused");
            assert_eq!(why, format!("{}: already exists", destination.display()));
            assert_eq!(snapshot(&root), before, "{row}: bare refusal changes nothing");
        }
        let why = write(&dirs, &env, &root.join("bin/temper"), [0; 2]).expect_err("refused");
        assert_eq!(why, format!("{}: already exists", destination.display()));
        assert_eq!(snapshot(&root), before, "{row}: every file and directory untouched");
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[test]
fn init_leaves_an_existing_playground_key_alone() {
    let root = scratch("existing-playground-key");
    let dirs = directories(&root);
    let env = environment(&root, false);
    let key = path(&dirs.data).join("demo/llm.key");
    fs::create_dir_all(key.parent().expect("demo directory")).expect("directory");
    fs::write(&key, b"kept").expect("existing fixture");
    let written = write(&dirs, &env, &root.join("bin/temper"), [0; 2]).expect("key is not an init destination");
    assert_eq!(fs::read(&key).expect("existing fixture"), b"kept");
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).expect("cleanup");
}

fn snapshot(root: &Path) -> Vec<(PathBuf, u32, Vec<u8>)> {
    let mut pending = vec![root.to_path_buf()];
    let mut entries = Vec::new();
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).expect("entry");
        let content = if metadata.is_dir() {
            pending.extend(fs::read_dir(&path).expect("directory").map(|entry| entry.expect("child").path()));
            Vec::new()
        } else if metadata.is_symlink() {
            fs::read_link(&path).expect("link").as_os_str().as_bytes().to_vec()
        } else {
            fs::read(&path).expect("file")
        };
        entries.push((path, metadata.mode(), content));
    }
    entries.sort();
    entries
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
    let root = scratch("program-fact");
    let dirs = directories(&root);
    let env = environment(&root, true);
    let binary = root.join("temper");
    let adapter = root.join("fake-claude");
    let written = write_with_program(&dirs, &env, &binary, [0; 2], Some(&adapter)).expect("injected init fact");
    let document: serde_json::Value = serde_json::from_slice(
        &temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file").document,
    )
    .expect("JSON");
    assert!(!document["application"].as_object().unwrap().contains_key("workers"));
    assert!(!path(&dirs.config).join("agent.json").exists());
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).expect("fixture cleanup");
}

#[test]
fn a_path_without_openssl_writes_a_box_without_sign_in_and_keeps_running() {
    let root = scratch("no-openssl");
    let dirs = directories(&root);
    let env = environment(&root, false);
    let written = write(&dirs, &env, &root.join("temper"), [0; 2]).expect("init succeeds without openssl");
    assert_eq!(written.no_sign_in.as_deref(), Some("openssl is not on PATH"));
    let value: serde_json::Value = serde_json::from_slice(
        &temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file").document,
    )
    .unwrap();
    assert!(value["engine"]["clients"]["provider"].is_null());
    assert!(value["application"].get("machine").is_none());
    assert!(!path(&dirs.data).join("provider.key").exists());
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).unwrap();
}
