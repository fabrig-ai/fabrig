use super::*;
use crate::directories::jig_directories;
use crate::testing::fake_backend_fixtures::{
    PAGE, Scratch, config_path, initialized_source, parsed, prepare_empty, prepared_source, provision_bundle,
    provisioned_source, read_json, start, store_file,
};
use machine::Tree;
use skein_fake_machine::Item;
use std::{ffi::OsString, io::Write};

mod machine;

fn fixture_directories() -> Directories {
    Directories {
        config: Box::from(b"/source-config".as_slice()),
        data: Box::from(b"/source-data".as_slice()),
        cache: Box::from(b"/cache".as_slice()),
        origins: [skein_shell::Origin::Flag; 3],
    }
}

fn init_file(bundle: Option<&Path>) -> temper_config_shell::File {
    let mut file = crate::init::configuration(NonZeroU16::new(7100).unwrap(), NonZeroU16::new(8373).unwrap());
    file.internal.bundle = bundle.map(Path::to_path_buf);
    file
}

fn source_items(file: &temper_config_shell::File) -> Vec<Item> {
    vec![
        Item::directory(b"source-config").mode(0o700),
        Item::file(format!("source-config/{FILE}").as_bytes(), &temper_config_shell::render(file, Shape::Standalone))
            .mode(0o600),
    ]
}

fn stock_box_source(sign_in: bool) -> (Directories, Tree) {
    let mut items = source_items(&init_file(None));
    if sign_in {
        items.push(Item::directory(b"source-data").mode(0o700));
        items.push(Item::file(b"source-data/provider.key", b"").mode(0o600));
    }
    (fixture_directories(), Tree::new(&items))
}

fn check_tree_source(tree: &Tree, dirs: &Directories, bundle: Option<&Path>) -> Result<Option<PathBuf>, String> {
    let (root, _) = directories(dirs)?;
    let (_, provisioned) = check_source(
        &root,
        tree.exists(&root),
        bundle,
        || tree.source(dirs),
        |path, limits| {
            provisioned_bundle_with(
                path,
                limits,
                |page| tree.public_path(page),
                |_, _| panic!("fixture should refuse before reading the bundle"),
            )
        },
        || panic!("fixture should refuse before checking the adapter"),
    )?;
    Ok(provisioned)
}

pub(super) struct Effective {
    directories: Directories,
    peers: Peers,
    tree: Tree,
}

impl Effective {
    fn new() -> Self {
        Self::with_listeners(NonZeroU16::new(7100).unwrap(), NonZeroU16::new(8373).unwrap())
    }

    pub(super) fn with_listeners(clients: NonZeroU16, webhooks: NonZeroU16) -> Self {
        Self::with_source(clients, webhooks, Vec::new())
    }

    fn with_source(clients: NonZeroU16, webhooks: NonZeroU16, mut items: Vec<Item>) -> Self {
        let (_, directories) = directories(&fixture_directories()).expect("effective directories");
        let peers = Peers {
            github: ([127, 0, 0, 1], 7106).into(),
            oauth: ([127, 0, 0, 1], 7107).into(),
            llm: ([127, 0, 0, 1], 7108).into(),
            forgejo: ([127, 0, 0, 1], 7109).into(),
        };
        let facts =
            temper_config_shell::Facts { login: "fixture".into(), hostname: "localhost".into(), provider_key: false };
        let initial = temper_config_shell::parse(
            &temper_config_shell::render(&crate::init::configuration(clients, webhooks), Shape::Standalone),
            Shape::Standalone,
            &facts,
        )
        .expect("init document");
        let (text, peers_documents) =
            new_documents(&directories, initial, "/bundle".into(), Path::new("/fake-claude"), &peers)
                .expect("fake deployment documents");
        items.extend([
            Item::directory(b"source-data"),
            Item::directory(b"source-data/box"),
            Item::directory(b"source-data/box/fake-backends"),
            Item::directory(b"source-data/box/fake-backends/config"),
        ]);
        for (name, document) in [
            (FILE, text.as_ref()),
            ("github.json", &peers_documents.github),
            ("oauth.json", &peers_documents.oauth),
            ("llm.json", &peers_documents.llm),
            ("forgejo.json", &peers_documents.forgejo),
            (ADAPTER, b"/fake-claude"),
        ] {
            items.push(
                Item::file(format!("source-data/box/fake-backends/config/{name}").as_bytes(), document).mode(0o600),
            );
        }
        Self { directories, peers, tree: Tree::new(&items) }
    }

    pub(super) fn document(&self) -> Box<[u8]> {
        self.tree
            .parse(&self.directories, &self.tree.read(&config_path(&self.directories)))
            .expect("effective file")
            .document
    }

    fn script_steps(&self, options: &Options) -> Result<Box<[temper_forge_setup::Step]>, String> {
        let config = crate::path(&self.directories.config);
        scripts_for_start_with(
            &self.directories,
            &self.peers,
            options,
            |path| {
                if !self.tree.exists(path) {
                    return Err(std::io::ErrorKind::NotFound.into());
                }
                self.tree.remove(path);
                Ok(())
            },
            || Ok(self.tree.read(&config.join("llm.json"))),
            |path, bound| self.tree.read_bounded(path, bound),
            |path, bytes| {
                if self.tree.exists(path) {
                    self.tree.remove(path);
                }
                self.tree.create(path, bytes);
                Ok(())
            },
        )
    }
}

#[test]
fn untouched_box_init_imports_with_and_without_sign_in() {
    for sign_in in [false, true] {
        let (source, tree) = stock_box_source(sign_in);
        let original = tree.read(&config_path(&source));
        let imported = tree.source(&source).expect("untouched box init imports");
        let (_, effective) = directories(&source).expect("sandbox directories");
        let temper = imported.rebase(&effective).expect("source file");
        let read = tree.parse(&source, &temper).expect("imported file");
        let rebased: serde_json::Value = serde_json::from_slice(&read.document).expect("rebased document");
        let original_tree: serde_json::Value =
            serde_json::from_slice(&tree.parse(&source, &original).expect("source document").document)
                .expect("source JSON");
        assert_eq!(rebased["engine"]["clients"]["provider"], original_tree["engine"]["clients"]["provider"]);
        assert!(rebased["application"].get("machine").is_none());
        let checked =
            crate::setup::check(&read.document, &jig_directories(&effective)).expect("ordinary effective admission");
        assert_eq!(checked.named.deployment, bytes(&crate::path(&effective.data).join("state")));
        assert_rebased_box_paths(&rebased, &effective, sign_in);
        assert_eq!(tree.read(&config_path(&source)), original);
        assert_eq!(tree.exists(crate::path(&source.data)), sign_in, "only the fixture key marker needs source data");
        assert!(!tree.exists(crate::path(&effective.data)), "import and rebase only inspect public documents");
    }
}

fn assert_rebased_box_paths(temper: &serde_json::Value, effective: &Directories, sign_in: bool) {
    {
        let (pointer, name) = ("workers/0/secret", "worker.secret");
        assert_eq!(
            resolved(
                crate::path(&effective.data),
                temper.pointer(&format!("/{pointer}")).expect("resource path").as_str().unwrap().as_bytes()
            )
            .expect("resolved"),
            crate::path(&effective.data).join(name),
            "{pointer}"
        );
    }
    if sign_in {
        assert_eq!(
            resolved(
                crate::path(&effective.data),
                temper.pointer("/engine/clients/provider/key").expect("key path").as_str().unwrap().as_bytes()
            )
            .expect("effective provider key"),
            crate::path(&effective.data).join("provider.key")
        );
    }
}

#[test]
fn modified_box_sign_in_is_refused() {
    let (source, tree) = stock_box_source(true);
    let path = config_path(&source);
    let stock = tree.parse(&source, &tree.read(&path)).expect("stock document").file;
    for (key, openid) in [
        ("key", temper_config_shell::OpenId { key: Some("other.key".into()), ..Default::default() }),
        (
            "clients",
            temper_config_shell::OpenId {
                clients: vec![temper_config_shell::OpenIdClient {
                    id: "other".into(),
                    secret: None,
                    redirect: "http://127.0.0.1:9999/callback".into(),
                }],
                ..Default::default()
            },
        ),
        ("addresses", temper_config_shell::OpenId { addresses: Some(vec![]), ..Default::default() }),
    ] {
        let mut changed = stock.clone();
        changed.openid = Some(openid);
        tree.remove(&path);
        tree.create(&path, &temper_config_shell::render(&changed, Shape::Standalone));
        let error = tree.source(&source).err().expect("modified sign-in refused");
        assert!(error.contains(key), "{key}: {error}");
    }
}

#[test]
fn program_login_line_names_the_model_steps_placeholder_and_quotes_effective_directories() {
    let dirs = Directories {
        config: b"/effective owner's/config".as_slice().into(),
        data: b"/effective data".as_slice().into(),
        cache: b"/shared cache".as_slice().into(),
        origins: [skein_shell::Origin::Flag; 3],
    };
    assert_eq!(
        program_login_line(&dirs, Path::new("/bin owner's/temper")),
        "'/bin owner'\\''s/temper' program-login --config '/effective owner'\\''s/config' --data '/effective data' --cache '/shared cache' NAME (the name the Model step shows)"
    );
}

#[test]
fn fake_backends_is_an_additive_box_only_switch() {
    let environment = [(Box::<[u8]>::from(b"HOME".as_slice()), Box::<[u8]>::from(b"/fixture/home".as_slice()))];
    for (words, expected) in [
        (vec!["--fake-backends"], true),
        (vec!["--front", "--fake-backends", "--bundle", "/bundle"], true),
        (vec!["--fake-backends", "--front", "--bundle", "/bundle"], true),
        (vec!["--fake-backends", "--bundle", "/bundle", "--front"], true),
        (vec!["--front", "--fake-backends", "--front"], false),
        (vec!["--fake-backends", "--config", "/c", "--data", "/d", "--cache", "/k"], true),
        (vec!["--fake-backends", "--fake-backends"], false),
        (vec!["--fake-backends", "extra"], false),
        (vec!["serve", "--fake-backends"], false),
        (vec!["init", "--fake-backends"], false),
    ] {
        let args = words.iter().map(|s| Box::<[u8]>::from(s.as_bytes())).collect::<Vec<_>>();
        let result = crate::configure_with(&args, &environment, b"/fixture");
        assert_eq!(matches!(result, Ok(crate::Command::BoxFakeBackends { .. })), expected, "{words:?}: {result:?}");
    }
    let args = [];
    assert!(matches!(crate::configure_with(&args, &environment, b"/fixture"), Ok(crate::Command::Box { .. })));
}

fn configure(words: &[&[u8]]) -> Result<crate::Command, String> {
    let environment = [(Box::<[u8]>::from(b"HOME".as_slice()), Box::<[u8]>::from(b"/fixture/home".as_slice()))];
    let args = words.iter().map(|word| Box::<[u8]>::from(*word)).collect::<Vec<_>>();
    crate::configure_with(&args, &environment, b"/fixture")
}

#[test]
fn fake_box_refuses_the_removed_start_flag() {
    let refusal = configure(&[b"--fake-backends", b"--fixed\x2dstart", b"1767690000,7"]).expect_err("removed flag");
    assert_eq!(refusal.lines().next(), Some("temper-box: unknown flag --fixed\x2dstart"));
}

#[test]
fn box_only_options_name_the_three_requirements() {
    for words in [vec![b"--bundle".as_slice(), b"assets"], vec![b"--scripts", b"scripts.json"], vec![b"--strict"]] {
        let refusal = configure(&words).expect_err("fake backends required");
        assert_eq!(
            refusal.lines().next(),
            Some("temper-box: --bundle, --scripts and --strict require --fake-backends")
        );
    }
}

#[test]
fn fake_box_steps_have_no_removed_start_flag_on_new_and_retained_starts() {
    let prepared = Effective::new();
    let options = Options { bundle: Some("/bundle".into()), strict: true, ..Options::default() };
    for _ in 0..2 {
        let first = prepared.script_steps(&options).expect("prepare steps, new then retained");
        let deployment = crate::setup::check(&prepared.document(), &jig_directories(&prepared.directories))
            .expect("admit the effective deployment");
        for record in [false, true] {
            let (steps, _) = crate::box_command::configuration_steps_checked(
                &prepared.directories,
                &deployment,
                first.clone(),
                record,
            )
            .expect("ordinary arguments");
            assert_eq!(steps.always.len(), 5, "four peers and serve");
            for step in steps
                .always
                .iter()
                .chain(steps.asked.iter().flat_map(|group| group.steps.iter().map(|(_, step)| step)))
                .chain(steps.asked.iter().filter_map(|group| group.remover.as_ref()))
            {
                assert!(!step.arguments().iter().any(|word| word.as_ref() == b"--fixed\x2dstart"), "{:?}", step.name());
            }
        }
    }
}

#[test]
fn script_flags_resolve_in_order_and_require_fake_backends() {
    let command = configure(&[
        b"--scripts",
        b"first.json",
        b"--config",
        b"/c",
        b"--strict",
        b"--fake-backends",
        b"--scripts",
        b"/second.json",
    ])
    .expect("repeatable scripts with interleaved directory settings");
    let crate::Command::BoxFakeBackends { scripts, strict, .. } = command else { panic!("fake box") };
    assert_eq!(scripts, [PathBuf::from("/fixture/first.json"), PathBuf::from("/second.json")]);
    assert!(strict);
    assert!(matches!(
        configure(&[b"--fake-backends", b"--strict"]),
        Ok(crate::Command::BoxFakeBackends { strict: true, .. })
    ));
    for words in [
        vec![b"--scripts".as_slice(), b"first.json"],
        vec![b"--strict"],
        vec![b"--fake-backends", b"--scripts"],
        vec![b"--fake-backends", b"--scripts", b""],
        vec![b"--fake-backends", b"--scripts", b"bad\0path"],
        vec![b"--fake-backends", b"--strict", b"--strict"],
    ] {
        assert!(configure(&words).is_err(), "{words:?}");
    }
}

#[test]
fn script_join_refuses_the_file_before_start() {
    let prepared = Effective::new();
    let tree = &prepared.tree;
    let model = crate::path(&prepared.directories.config).join("llm.json");
    let original = tree.read(&model);
    let joined = crate::path(&prepared.directories.config).join("llm-scripts.json");
    let first = PathBuf::from("/first.json");
    let second = PathBuf::from("/second.json");
    let options = Options { scripts: vec![first.clone(), second.clone()], ..Options::default() };
    // Each individual group fits; their joined list crosses the document reader's bound.
    let full = (0..temper_fake::ENTRIES)
        .map(|i| serde_json::json!({"cue":format!("cue {i}"),"turns":["ok"]}))
        .collect::<Vec<_>>();
    for (a, b, detail) in [
        (
            serde_json::json!({"cued":[{"cue":"same","turns":["a"]}]}),
            Some(serde_json::json!({"cued":[{"cue":"same","turns":["b"]}]})),
            "unique",
        ),
        (
            serde_json::json!({"asked":[{"cue":"same","turns":["a"]}]}),
            Some(serde_json::json!({"asked":[{"cue":"same","turns":["b"]}]})),
            "unique",
        ),
        (serde_json::json!({}), Some(serde_json::json!({"mystery":[]})), "mystery"),
        (serde_json::json!({"cued":full}), Some(serde_json::json!({"cued":[{"cue":"extra","turns":["b"]}]})), "bound"),
        (serde_json::json!({}), None, "second.json"),
    ] {
        tree.create(&first, &serde_json::to_vec(&a).expect("first JSON"));
        if let Some(b) = b {
            tree.create(&second, &serde_json::to_vec(&b).expect("second JSON"));
        }
        let error = prepared.script_steps(&options).expect_err("refuse before steps start");
        assert!(error.contains(second.to_str().expect("path")), "{error}");
        assert!(error.contains(detail), "{error}");
        assert!(!tree.exists(&joined), "no unvalidated join published");
        assert_eq!(tree.read(&model), original);
        tree.remove(&first);
        if tree.exists(&second) {
            tree.remove(&second);
        }
    }
}

#[test]
fn bundle_paths_resolve_against_the_invocation_directory() {
    for (given, expected) in
        [(b"assets".as_slice(), "/fixture/assets"), (b"/bundles/loopback".as_slice(), "/bundles/loopback")]
    {
        let command = configure(&[b"--fake-backends", b"--bundle", given]).expect("bundle arguments");
        let crate::Command::BoxFakeBackends { bundle, .. } = command else { panic!("fake box") };
        assert_eq!(bundle, Some(PathBuf::from(expected)));
    }
}

#[test]
fn bundle_and_fake_flags_allow_interleaved_directory_settings() {
    let cases: &[&[&[u8]]] = &[
        &[b"--bundle", b"assets", b"--config", b"/c", b"--fake-backends", b"--data", b"/d", b"--cache", b"/k"],
        &[b"--data", b"/d", b"--fake-backends", b"--cache", b"/k", b"--bundle", b"assets", b"--config", b"/c"],
    ];
    for words in cases {
        let command = configure(words).expect("interleaved bundle and directory settings");
        let crate::Command::BoxFakeBackends { directories, bundle, .. } = command else { panic!("fake box") };
        assert_eq!(bundle, Some(PathBuf::from("/fixture/assets")));
        assert_eq!(directories.config.as_ref(), b"/c");
        assert_eq!(directories.data.as_ref(), b"/d");
        assert_eq!(directories.cache.as_ref(), b"/k");
    }
}

#[test]
fn malformed_bundle_grammar_is_refused() {
    let cases: &[&[&[u8]]] = &[
        &[b"--bundle", b"assets"],
        &[b"--fake-backends", b"--bundle"],
        &[b"--bundle", b"--fake-backends"],
        &[b"--fake-backends", b"--bundle", b"assets", b"extra"],
        &[b"--fake-backends", b"--bundle", b"assets", b"--bundle", b"other"],
        &[b"--fake-backends", b"--bundle", b"assets", b"--fake-backends"],
        &[b"serve", b"--fake-backends", b"--bundle", b"assets"],
        &[b"init", b"--bundle", b"assets"],
    ];
    for words in cases {
        assert!(configure(words).is_err(), "{words:?}");
    }
}

#[test]
fn bundle_paths_check_both_sides_of_the_absolute_bound() {
    // protocol/configuration/directories.md: the resolved absolute path is bounded at 4096 bytes.
    const PATH_BYTES: usize = 4096;
    const WORKING: &[u8] = b"/fixture/";
    let mut absolute = vec![b'a'; PATH_BYTES];
    absolute[0] = b'/';
    let relative = vec![b'a'; PATH_BYTES - WORKING.len()];
    for (given, accepted) in [
        (absolute.clone(), true),
        ([absolute, b"a".to_vec()].concat(), false),
        (relative.clone(), true),
        ([relative, b"a".to_vec()].concat(), false),
        (Vec::new(), false),
        (b"assets\0other".to_vec(), false),
    ] {
        assert_eq!(configure(&[b"--fake-backends", b"--bundle", &given]).is_ok(), accepted);
    }
}

#[test]
fn a_reservation_refuses_at_once_on_any_other_error() {
    let mut calls = 0;
    let error = reserve_from(&[], || {
        calls += 1;
        assert_eq!(calls, 1, "a non-address error must not retry");
        Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "injected permission denial"))
    })
    .expect_err("permission denied refuses the reservation");
    assert_eq!(calls, 1);
    assert_eq!(error, "fake peer reservation: injected permission denial");
}

#[test]
fn a_reservation_refuses_after_32_lost_ports() {
    let mut calls = 0;
    let error = reserve_from(&[], || {
        calls += 1;
        assert!(calls <= 32, "lost ports must exhaust the retry bound");
        Err(std::io::ErrorKind::AddrInUse.into())
    })
    .expect_err("all ports lost");
    assert_eq!(calls, 32);
    assert_eq!(error, "fake peer reservation: could not choose a port distinct from the deployment and callback");
}

#[test]
fn retained_peer_edits_are_checked_without_replacement() {
    let first = Effective::new();
    let tree = &first.tree;
    let config = crate::path(&first.directories.config);
    let peers = first.peers;
    let document = first.document();
    let deployment =
        crate::setup::check(&document, &jig_directories(&first.directories)).expect("effective deployment");
    let ordinary: serde_json::Value = serde_json::from_slice(&document).expect("effective JSON");
    let ordinary_socket = &ordinary["application"]["clients"]["listener"]["socket"];
    for kind in ["edited", "zero", "collision", "ordinary", "outside", "oauth"] {
        let file = if kind == "oauth" { "oauth.json" } else { "llm.json" };
        let path = config.join(file);
        let original = tree.read(&path);
        let mut edited: serde_json::Value = serde_json::from_slice(&original).expect("fixture JSON");
        match kind {
            "edited" => edited["listen"]["socket"] = "127.0.0.1:17151".into(),
            "zero" => edited["listen"]["socket"] = "127.0.0.1:0".into(),
            "collision" => edited["listen"]["socket"] = peers.github.to_string().into(),
            "ordinary" => edited["listen"]["socket"] = ordinary_socket.clone(),
            "outside" => edited["key"] = "/outside/llm.key".into(),
            "oauth" => edited["redirect_port"] = 0.into(),
            _ => unreachable!("fixture kind"),
        }
        tree.remove(&path);
        tree.create(&path, &serde_json::to_vec(&edited).expect("edited JSON"));
        let result = checked_peers_with(&first.directories, &deployment, |name, bound| {
            let peer = config.join(name);
            tree.public_path(&peer)?;
            tree.private_document(&peer)?;
            tree.read_bounded(&peer, bound).map_err(crate::unread)
        });
        if kind == "edited" {
            assert_eq!(
                result.expect("safe effective edit retained").llm.to_string(),
                edited["listen"]["socket"].as_str().expect("edited socket")
            );
        } else {
            let error = result.expect_err("unsafe retained peer refused");
            assert!(error.contains(file), "{kind}: {error}");
        }
        let retained: serde_json::Value = serde_json::from_slice(&tree.read(&path)).expect("retained JSON");
        assert_eq!(retained, edited, "retained edits never silently replaced");
        tree.remove(&path);
        tree.create(&path, &original);
    }
}

#[test]
fn explicit_bundle_initializes_without_creating_absent_source_configuration() {
    let scratch = Scratch::new("absent-explicit");
    let source = scratch.dirs();
    let _prepared = prepare_empty(&scratch, &source);
    assert!(!crate::path(&source.config).exists());
}

#[test]
fn explicit_bundle_overrides_an_unavailable_source_bundle_without_rewriting_source() {
    let scratch = Scratch::new("source-override");
    let source = provisioned_source(&scratch);
    let original = fs::read(config_path(&source)).expect("source public bytes");
    fs::remove_dir_all(scratch.0.join("bundle-loopback")).expect("source bundle unavailable");
    let supplied = provision_bundle(&scratch.0.join("override"));
    let first = start(&source, Some(&supplied)).expect("explicit selection overrides source bundle");
    assert_eq!(
        read_json(&config_path(&first.directories))["application"]["clients"]["bundle"],
        supplied.to_str().expect("supplied path")
    );
    let retained = first.bytes.clone();
    drop(first);
    let second = start(&source, None).expect("restart without repeating the selection");
    assert_eq!(second.bytes, retained);
    assert_eq!(fs::read(config_path(&source)).expect("unchanged source bytes"), original);
}

#[test]
fn explicit_bundle_imports_init_configuration_without_a_source_bundle() {
    let scratch = Scratch::new("init-explicit");
    let source = initialized_source(&scratch);
    let original = fs::read(config_path(&source)).expect("source document");
    let supplied = provision_bundle(&scratch.0.join("supplied"));
    let prepared = start(&source, Some(&supplied)).expect("init-shaped source plus explicit bundle");
    assert_eq!(
        read_json(&config_path(&prepared.directories))["application"]["clients"]["bundle"],
        supplied.to_str().expect("supplied path")
    );
    assert_eq!(fs::read(config_path(&source)).expect("source unchanged"), original);
}

#[test]
fn preparation_does_not_discover_optional_programs_outside_its_environment() {
    let scratch = Scratch::new("optional-programs");
    let source = scratch.dirs();
    let bundle = provision_bundle(&scratch.0.join("bundle"));
    let prepared = start(&source, Some(&bundle)).expect("prepare without optional executables");
    let document = read_json(&config_path(&prepared.directories));
    assert!(document["application"].get("workers").is_none());
    assert!(document["engine"]["clients"]["provider"].is_null());
    assert!(document["application"].get("machine").is_none());
    assert!(!crate::path(&prepared.directories.data).join("provider.key").exists());
}

#[test]
fn empty_configuration_refuses_before_creating_data_or_cache() {
    let source = fixture_directories();
    // Missing provisioning refuses for both absent C and present empty C.
    assert_eq!(
        missing_bundle_refuses(&Tree::new(&[]), &source),
        "application.clients.bundle: no provisioned bundle; run scripts/bundle.sh DIR && temper-box --fake-backends --bundle BUNDLE (directory flags as usual)"
    );
    missing_bundle_refuses(&Tree::new(&[Item::directory(b"source-config")]), &source);
}

fn missing_bundle_refuses(tree: &Tree, source: &Directories) -> String {
    let error = check_tree_source(tree, source, None).expect_err("no bundle");
    assert!(error.contains("scripts/bundle.sh DIR"), "{error}");
    assert!(error.contains("application.clients.bundle"), "{error}");
    assert!(error.contains("temper-box --fake-backends --bundle BUNDLE"), "{error}");
    assert!(!tree.exists(crate::path(&source.data)), "refuse before data mutation");
    assert!(!tree.exists(crate::path(&source.cache)), "refuse before cache mutation");
    error
}

#[test]
fn initialized_source_without_bundle_refuses_before_sandbox_mutation() {
    let (source, tree) = stock_box_source(false);
    // Configuration import must not need any resource under source D.
    missing_bundle_refuses(&tree, &source);
}

#[test]
fn partial_sandbox_refuses_with_explicit_reset_instructions() {
    let source = fixture_directories();
    let (root, _) = directories(&source).expect("sandbox paths");
    let mut items = source_items(&init_file(Some(Path::new("/bundle"))));
    items.extend([
        Item::directory(b"source-data"),
        Item::directory(b"source-data/box"),
        Item::directory(b"source-data/box/fake-backends"),
    ]);
    let tree = Tree::new(&items);
    let imported = tree.source(&source).expect("source");
    let error = check_marker(&root, &imported, || {
        tree.read_bounded(&root.join("prepared"), crate::CONFIG_BYTES).map_err(crate::unread)
    })
    .expect_err("partial refused");
    assert!(
        error.contains("partial preparation") && error.contains("remove this sandbox directory explicitly"),
        "{error}"
    );
}

#[test]
fn reuse_keeps_effective_edits_peer_addresses_and_data() {
    let scratch = Scratch::new("reuse");
    let source = provisioned_source(&scratch);
    let first = start(&source, None).expect("fresh sandbox");
    let peer = fs::read(crate::path(&first.directories.config).join("oauth.json")).expect("peer document");
    let data = crate::path(&first.directories.data).join("retained-fixture");
    write(&data, b"keep this data", false).expect("fixture data");
    let mut edited = read_json(&config_path(&first.directories));
    edited["engine"]["owners"][0]["subject"] = "retained-owner@localhost".into();
    let retained_bundle = provision_bundle(&scratch.0.join("retained-bundle"));
    edited["application"]["clients"]["bundle"] = retained_bundle.to_str().expect("retained bundle path").into();
    let mut file = parsed(&config_path(&first.directories)).file;
    file.owner = Some("retained-owner@localhost".into());
    file.internal.bundle = Some(retained_bundle);
    store_file(&config_path(&first.directories), &file);
    let secret = fs::read(crate::path(&first.directories.data).join("worker.secret")).expect("generated test secret");
    drop(first);
    // Source documents stay unchanged, but the retained bundle no longer needs the source directory.
    fs::remove_dir_all(scratch.0.join("bundle-loopback")).expect("remove original provisioned bundle");
    let second = start(&source, None).expect("reuse the retained configuration's bundle");
    assert_eq!(read_json(&config_path(&second.directories)), edited);
    assert_eq!(fs::read(crate::path(&second.directories.config).join("oauth.json")).expect("peer document"), peer);
    assert_eq!(fs::read(crate::path(&second.directories.data).join("worker.secret")).expect("test secret"), secret);
    assert_eq!(fs::read(data).expect("retained data"), b"keep this data");
    assert!(second.listeners.0.is_empty(), "reuse reopens its own listeners");
}

#[test]
fn reservations_avoid_every_port_the_peer_check_refuses() {
    let first = Effective::new();
    let bytes = first.document();
    let document: serde_json::Value = serde_json::from_slice(&bytes).expect("effective JSON");
    let ports = ordinary_ports(&first.directories, &bytes).expect("ports");
    for pointer in ["/application/clients/listener/socket", "/application/webhooks/listener/socket"] {
        let socket = document.pointer(pointer).unwrap().as_str().expect("socket");
        let port = socket.parse::<SocketAddr>().expect("address").port();
        assert!(ports.contains(&port), "{socket}");
    }
}

#[test]
fn staging_file_collision_leaves_the_retained_document_untouched() {
    let first = Effective::new();
    let tree = &first.tree;
    let document = config_path(&first.directories);
    let before = tree.read(&document);
    let staging = document.with_file_name(format!("{FILE}.bundle-staging"));
    tree.create(&staging, b"existing staging file");
    let error = open_staging(&document, |path| tree.create_exclusive(path, b"").map_err(machine::io_error))
        .expect_err("staging collision refused");
    assert!(error.contains(&format!("{FILE}.bundle-staging")), "{error}");
    assert_eq!(tree.read(&document), before);
    assert_eq!(tree.read(&staging).as_ref(), b"existing staging file");
}

#[test]
fn reuse_refuses_a_missing_effective_bundle_and_preserves_edits() {
    let scratch = Scratch::new("retained-missing-bundle");
    let source = provisioned_source(&scratch);
    let first = start(&source, None).expect("fresh sandbox");
    let document = config_path(&first.directories);
    let mut edited = read_json(&document);
    let absent = scratch.0.join("absent-retained-bundle");
    edited["application"]["clients"]["bundle"] = absent.to_str().expect("absent bundle path").into();
    let mut file = parsed(&document).file;
    file.internal.bundle = edited["application"]["clients"]["bundle"].as_str().map(PathBuf::from);
    store_file(&document, &file);
    drop(first);
    let error = start(&source, None).err().expect("missing retained bundle refused");
    assert!(error.contains("absent-retained-bundle"), "{error}");
    assert_eq!(read_json(&document), edited);

    edited["application"]["clients"]["bundle"] = serde_json::Value::Null;
    let mut file = parsed(&document).file;
    file.internal.bundle = edited["application"]["clients"]["bundle"].as_str().map(PathBuf::from);
    store_file(&document, &file);
    let error = start(&source, None).err().expect("unnamed retained bundle refused");
    assert_eq!(error, "application.clients.bundle: effective document must name its bundle");
    assert_eq!(read_json(&document), edited);
}

#[test]
fn current_init_import_ignores_charter_files_and_rebases_paths() {
    let scratch = Scratch::new("import");
    let source = provisioned_source(&scratch);

    fs::create_dir_all(crate::path(&source.config).join("charters")).expect("stray public directory");
    write(&crate::path(&source.config).join("charters/chat.txt"), b"A custom public charter.\n", false)
        .expect("public charter");
    let before = [FILE, "charters/chat.txt"].map(|name| {
        let path = crate::path(&source.config).join(name);
        (name, fs::read(&path).expect("source public fixture"), fs::metadata(path).expect("mode").mode())
    });
    let source_secret = fs::read(crate::path(&source.data).join("worker.secret")).expect("generated fixture secret");
    let prepared = start(&source, None).expect("current init-shaped import");
    assert_ne!(
        fs::read(crate::path(&prepared.directories.data).join("worker.secret")).expect("fresh test secret"),
        source_secret
    );
    assert!(!crate::path(&prepared.directories.config).join("charters").exists());
    for (name, contents, mode) in &before {
        let path = crate::path(&source.config).join(name);
        assert_eq!(fs::read(&path).expect("source unchanged"), *contents, "{name}");
        assert_eq!(fs::metadata(path).expect("source mode").mode(), *mode, "{name}");
    }
    assert_eq!(
        fs::read(crate::path(&source.data).join("worker.secret")).expect("unchanged source test secret"),
        source_secret
    );
    assert_eq!(fs::read(scratch.0.join("bundle-loopback/index.html")).expect("source page unchanged"), PAGE);
    assert_eq!(
        fs::read(scratch.0.join("bundle-loopback/0000000000000001/main.js")).expect("source script unchanged"),
        b"// fixture\n"
    );
    drop(prepared);
    write(&crate::path(&source.config).join("charters/chat.txt"), b"Changed source charter.\n", true)
        .expect("changed public source");
    let reused = start(&source, None).expect("stray charter edits do not change the imported defaults");
    assert!(!crate::path(&reused.directories.config).join("charters").exists());
}

#[test]
fn unsupported_imports_refuse_with_the_public_member_named() {
    let source = fixture_directories();
    let tree = Tree::new(&source_items(&init_file(Some(Path::new("/bundle")))));
    let path = config_path(&source);
    let invalid = [tree.read(&path).as_ref(), b"\n[environment]\nCUSTOM = \"extra\"\n"].concat();
    tree.remove(&path);
    tree.create(&path, &invalid);
    let error = check_tree_source(&tree, &source, None).expect_err("custom argv refused");
    assert!(error.contains("environment"), "{error}");
    assert!(!tree.exists(&crate::path(&source.data).join("box/fake-backends")));
}

#[test]
fn effective_commands_reopen_the_retained_directories() {
    let prepared = Effective::new();
    let (steps, _) = crate::box_command::configuration_steps(
        &prepared.directories,
        &prepared.document(),
        backends::steps(&prepared.directories.config, &prepared.directories.data, &prepared.peers, None, false),
    )
    .expect("steps");
    let prefix =
        backends::steps(&prepared.directories.config, &prepared.directories.data, &prepared.peers, None, false);
    assert_eq!(&steps.always[..prefix.len()], prefix.as_ref());
    for step in steps.always[prefix.len()..]
        .iter()
        .chain(steps.asked.iter().flat_map(|group| group.steps.iter().map(|(_, step)| step)))
    {
        // All setup/serve commands receive the effective directory flags, including the retained cache.
        let args =
            step.arguments().iter().map(|arg| OsString::from(std::ffi::OsStr::from_bytes(arg))).collect::<Vec<_>>();
        for (flag, value) in [
            ("--config", &prepared.directories.config),
            ("--data", &prepared.directories.data),
            ("--cache", &prepared.directories.cache),
        ] {
            assert!(
                args.windows(2).any(|pair| pair[0] == flag && pair[1].as_os_str().as_bytes() == value.as_ref()),
                "{flag}: {args:?}"
            );
        }
    }
}

#[test]
fn missing_provisioned_directory_refuses_before_data_or_cache_mutation() {
    let source = fixture_directories();
    let tree = Tree::new(&source_items(&init_file(Some(Path::new("/bundle-loopback")))));
    missing_bundle_refuses(&tree, &source);
}

#[test]
fn oversized_explicit_bundle_refuses_before_creating_data_or_cache() {
    let scratch = Scratch::new("explicit-bundle-bytes");
    let source = scratch.dirs();
    let supplied = provision_bundle(&scratch.0.join("supplied"));
    let limits = temper_limits_fixtures::small().clients;
    let page = OpenOptions::new().write(true).open(supplied.join("index.html")).expect("fixture page");
    page.set_len(u64::from(limits.bundle_bytes) + 1).expect("one byte past the profile's bundle bound");
    let error = start(&source, Some(&supplied)).err().expect("oversized explicit bundle refused");
    assert!(error.contains("clients.bundle_bytes"), "{error}");
    assert!(!crate::path(&source.config).exists());
    assert!(!crate::path(&source.data).exists());
    assert!(!crate::path(&source.cache).exists());
}

#[test]
fn oversized_source_bundle_refuses_before_sandbox_creation() {
    let scratch = Scratch::new("source-bundle-bytes");
    let source = provisioned_source(&scratch);
    let original = fs::read(config_path(&source)).expect("source public bytes");
    let limits = temper_limits_fixtures::small().clients;
    let page = OpenOptions::new().write(true).open(scratch.0.join("bundle-loopback/index.html")).expect("fixture page");
    page.set_len(u64::from(limits.bundle_bytes) + 1).expect("one byte past the admitted source bundle bound");
    let error = start(&source, None).err().expect("oversized source bundle refused");
    assert!(error.contains("clients.bundle_bytes"), "{error}");
    assert!(!crate::path(&source.data).join("box/fake-backends").exists());
    assert_eq!(fs::read(config_path(&source)).expect("unchanged source public bytes"), original);
}

#[test]
fn unsupported_provisioned_file_refuses_before_sandbox_creation() {
    let scratch = Scratch::new("bundle-type");
    let source = provisioned_source(&scratch);
    write(&scratch.0.join("bundle-loopback/0000000000000001/unsupported.exe"), b"unsupported", false)
        .expect("invalid bundle fixture");
    let error = start(&source, None).err().expect("unsupported bundle file refused");
    assert!(error.contains("application.clients.bundle") && error.contains("unsupported.exe"), "{error}");
    assert!(!crate::path(&source.data).join("box/fake-backends").exists());
}

#[test]
fn linked_provisioned_page_refuses_before_reading_its_target() {
    let source = fixture_directories();
    let mut items = source_items(&init_file(Some(Path::new("/bundle-loopback"))));
    items.extend([Item::directory(b"bundle-loopback"), Item::link(b"bundle-loopback/index.html", b"../unread-target")]);
    let tree = Tree::new(&items);
    let error = check_tree_source(&tree, &source, None).expect_err("linked bundle page refused");
    assert!(error.contains("index.html") && error.contains("links"), "{error}");
    assert!(!tree.exists(&crate::path(&source.data).join("box/fake-backends")));
}

#[test]
fn source_public_asset_links_are_refused_before_hashing_them() {
    let source = fixture_directories();
    let document = temper_config_shell::render(&init_file(Some(Path::new("/bundle-loopback"))), Shape::Standalone);
    let tree = Tree::new(&[
        Item::directory(b"source-config"),
        Item::file(b"source-file", &document),
        Item::link(format!("source-config/{FILE}").as_bytes(), b"../source-file"),
    ]);
    // The central reader checks the source first; identity hashing still refuses links.
    let error = check_tree_source(&tree, &source, None).expect_err("source public link refused");
    assert!(error.contains("links"), "{error}");
    assert!(!tree.exists(&crate::path(&source.data).join("box/fake-backends")));
}

#[test]
fn import_preserves_disabled_git_and_adds_the_fake_program() {
    let scratch = Scratch::new("disabled-programs");
    let source = provisioned_source(&scratch);
    let prepared = start(&source, None).expect("import optional choices independently of today's PATH");
    let temper = read_json(&config_path(&prepared.directories));
    assert!(temper["application"].get("workers").is_none());
    assert_eq!(
        fs::read(crate::path(&prepared.directories.config).join(ADAPTER)).expect("pinned adapter"),
        scratch.0.join("fake-claude").as_os_str().as_bytes()
    );
}

#[test]
fn fake_github_test_passes_ordinary_admission_without_document_forges() {
    let scratch = Scratch::new("four-forge-admission-r2");
    let (_source, first) = prepared_source(&scratch);
    let admitted = crate::setup::check(&first.bytes, &jig_directories(&first.directories));
    admitted.as_ref().expect("ordinary admission");
    let deployment = admitted.expect("test peer");
    assert_eq!(
        crate::address(&deployment.named.test_github.expect("fake GitHub").api).expect("socket"),
        first.peers.github
    );
    let document = read_json(&config_path(&first.directories));
    let github = &document["application"]["test"]["github"];
    let address = first.peers.github;
    assert_eq!(github["api"], serde_json::json!({"socket":address.to_string()}));
    assert_eq!(github["remote"], format!("http://{address}"));
    assert_eq!(github["web"], format!("http://{address}"));
    assert_eq!(github["transport"], "plaintext");
    assert_eq!(github["git"], crate::path(&first.directories.data).join("demo/bin/git").to_str().unwrap());
    let file = parsed(&config_path(&first.directories)).file;
    assert!(file.test.github.is_some(), "the fake is selected by its TOML key");
    assert!(file.internal.bundle.is_some(), "the bundle is selected by its TOML key");
    validate(&first.directories, &fs::read(config_path(&first.directories)).unwrap(), &scratch.0.join("fake-claude"))
        .expect("sandbox validation");
    // Ordinary startup checks the adapter against this executable's sibling.
    // Provision the sibling as a build fixture, as the shipped binaries are provisioned.
    let sibling = std::env::current_exe().expect("test binary").with_file_name("fake-claude");
    match OpenOptions::new().write(true).create_new(true).mode(0o700).open(&sibling) {
        Ok(mut file) => file.write_all(b"#!/bin/sh\nexit 1\n").expect("fixture adapter"),
        Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(why) => panic!("fixture adapter: {why}"),
    }
    fs::write(crate::path(&first.directories.config).join(ADAPTER), sibling.as_os_str().as_bytes())
        .expect("pin the ordinary startup adapter");
    let operations =
        crate::operations_in(&[], b"/", &first.directories, &first.bytes).expect("ordinary computed startup sizing");
    assert!(operations > 0);
    eprintln!("four-forge ordinary sizing: {operations} operations");
}

#[test]
fn missing_required_retained_documents_refuse_without_writes() {
    let first = Effective::new();
    let tree = &first.tree;
    let config = crate::path(&first.directories.config);
    let root = config.parent().expect("sandbox");
    let marker_path = root.join("prepared");
    let marker = tree.source(&fixture_directories()).expect("source").marker;
    tree.create(&marker_path, &marker);
    for name in REQUIRED {
        let path = config.join(name);
        let original = tree.read(&path);
        tree.remove(&path);
        let before = tree.list(config, u32::MAX).expect("config listing").len();
        let error = check_required(
            root,
            &first.directories,
            |path| tree.exists(path),
            |path| tree.public_path(path).and_then(|()| tree.private_document(path)),
        )
        .expect_err("missing required document refused");
        assert!(error.contains(name) && error.contains("reset"), "{error}");
        assert_eq!(tree.list(config, u32::MAX).expect("unchanged listing").len(), before);
        assert!(tree.read(&marker_path) == marker);
        assert!(!tree.exists(&path));
        tree.create(&path, &original);
    }
}

#[test]
fn foreign_retained_adapter_refuses_without_resetting_or_rewriting_it() {
    let first = Effective::new();
    let tree = &first.tree;
    let path = crate::path(&first.directories.config).join(ADAPTER);
    tree.remove(&path);
    tree.create(&path, b"/foreign/claude");
    let before = tree.read(&path);
    let error = check_adapter(
        Path::new("/fake-claude"),
        || tree.read_bounded(&path, 4096).map_err(crate::unread),
        || panic!("refuse before checking the executable"),
    )
    .expect_err("foreign adapter refused");
    assert!(error.contains(ADAPTER) && error.contains("reset"), "{error}");
    assert!(tree.read(&path) == before);
}

#[test]
fn exact_sibling_and_provisioning_refusals_use_explicit_paths() {
    let expected = sibling(Path::new("/fixture/temper")).expect("absolute sibling");
    assert_eq!(expected, Path::new("/fixture/fake-claude"));
    adapter_identity(b"/fixture/fake-claude", &expected).expect("correct identity");
    assert!(adapter_identity(b"/foreign/claude", &expected).expect_err("foreign").contains("reset"));
    let tree = Tree::new(&[Item::directory(b"fixture")]);
    for exists in [false, true] {
        if exists {
            tree.create(&expected, b"not executable");
        }
        let metadata = tree.metadata(&expected).ok().map(|meta| (meta.is == skein_fake_machine::Is::File, meta.mode));
        let error = provisioned_program_from(&expected, metadata).expect_err("provision outside startup");
        assert!(
            error.contains(expected.to_str().expect("path"))
                && error.contains("cargo build -p smith-fake-claude --bin fake-claude"),
            "{error}"
        );
        assert!(!error.contains("reset"), "{error}");
    }
}

#[test]
fn fake_backends_echo_takes_the_flag() {
    use temper_box_machine::host::Echo;
    let args =
        ["--fake-backends", "--config", "/c", "--data", "/d", "--cache", "/k"].map(|word| Box::from(word.as_bytes()));
    for (flags, expected) in
        [(vec![], Echo::None), (vec!["--echo", "steps"], Echo::Steps), (vec!["--echo", "none"], Echo::None)]
    {
        let mut arguments = args.to_vec();
        arguments.extend(flags.into_iter().map(|word| Box::from(word.as_bytes())));
        let crate::Command::BoxFakeBackends { echo, .. } =
            crate::configure_with(&arguments, &[], b"/").expect("valid fake-backend echo")
        else {
            panic!("fake box")
        };
        assert_eq!(echo, expected);
    }
    let mut arguments = args.to_vec();
    arguments.extend(["--echo", "bad"].map(|word| Box::from(word.as_bytes())));
    assert!(
        crate::configure_with(&arguments, &[], b"/")
            .expect_err("invalid echo")
            .contains("--echo: expected none or steps")
    );
}

#[test]
fn changed_source_bytes_or_directory_require_explicit_reset_without_writes() {
    let source = fixture_directories();
    let mut items = source_items(&init_file(Some(Path::new("/bundle"))));
    items.push(Item::directory(b"other-source"));
    let prepared = Effective::with_source(NonZeroU16::new(7100).unwrap(), NonZeroU16::new(8373).unwrap(), items);
    let tree = &prepared.tree;
    let (root, _) = directories(&source).expect("sandbox");
    let marker = root.join("prepared");
    tree.create(&marker, &tree.source(&source).expect("initial source").marker);
    let retained = config_path(&prepared.directories);
    let before = tree.read(&retained);
    let refuse = |dirs| {
        let imported = tree.source(dirs).expect("imported source");
        check_marker(&root, &imported, || Ok(tree.read(&marker))).expect_err("source change refused")
    };
    let path = config_path(&source);
    let original = tree.read(&path);
    let changed = [original.as_ref(), b"\n"].concat();
    tree.remove(&path);
    tree.create(&path, &changed);
    let refusal = refuse(&source);
    assert!(refusal.contains("changed public source documents"), "{refusal}");
    assert_eq!(tree.read(&retained), before);
    tree.remove(&path);
    tree.create(&path, &original);
    let relocated = Path::new("/other-source");
    tree.create(&relocated.join(FILE), &original);
    let other = Directories {
        config: bytes(relocated),
        data: source.data.clone(),
        cache: source.cache.clone(),
        origins: source.origins,
    };
    let refusal = refuse(&other);
    assert!(refusal.contains("remove this sandbox directory explicitly to reset it"), "{refusal}");
    assert_eq!(tree.read(&retained), before);
}

#[test]
fn chosen_ports_refuses_all_but_nine_nonzero_ports() {
    use std::os::unix::ffi::OsStringExt;
    for value in [
        OsString::from("1,2,3,4,5,6,7,8"),
        OsString::from("1,2,3,4,5,6,7,8,9,10"),
        OsString::from("1,2,3,4,0,6,7,8,9"),
        OsString::from("1,2,3,4,word,6,7,8,9"),
        OsString::from_vec(b"1,2,3,4,\xff,6,7,8,9".to_vec()),
    ] {
        let why = chosen_ports(&value).err().expect("invalid choice refused");
        assert!(why.contains(CHOSEN_PORTS), "{why}");
    }
}

#[test]
fn import_refuses_the_former_file_before_hashing_source_bytes() {
    use temper_config_shell::{FILE, FORMER};
    let scratch = skein_scratch::Scratch::new("fake-import-former");
    let directories = crate::tests::configuration::directories(scratch.path());
    std::fs::create_dir(scratch.path().join(FORMER)).expect("former directory");
    for present in [false, true] {
        if present {
            std::fs::write(scratch.path().join(FILE), b"not TOML").expect("current file");
        }
        let Err(why) = super::import::Source::read(&directories, &[]) else { panic!("former refused") };
        assert_eq!(
            why,
            format!(
                "{FORMER}: moved: the configuration is {FILE}; write it from temper-box help config, then remove {FORMER}"
            )
        );
    }
}

const BUNDLE: &str = "/bundle";
const REPLACEMENT: &str = "/replacement";

fn sandbox(import: bool) -> (Directories, machine::Sandbox) {
    let mut items = if import {
        source_items(&init_file(Some(Path::new(BUNDLE))))
    } else {
        vec![Item::directory(b"source-config").mode(0o700)]
    };
    items.extend([
        Item::directory(b"bundle"),
        Item::file(b"bundle/index.html", PAGE),
        Item::directory(b"replacement"),
        Item::file(b"replacement/index.html", PAGE),
        Item::directory(b"programs"),
        Item::file(b"programs/fake-claude", b"inert adapter").mode(0o700),
        Item::file(
            b"a.json",
            br#"{"cued":[{"cue":"A","turns":["a"]}],"asked":[{"cue":"A","turns":["asked a"]}],"otherwise":["X"]}"#,
        ),
        Item::file(b"b.json", br#"{"cued":[{"cue":"B","turns":["b"]}],"asked":[{"cue":"B","turns":["asked b"]}]}"#),
    ]);
    (fixture_directories(), machine::Sandbox::new(&items))
}

fn sandbox_start(
    source: &Directories,
    disk: &machine::Sandbox,
    bundle: Option<&str>,
) -> Result<Startup<machine::Sandbox>, String> {
    prepare_with(
        source,
        &Options { bundle: bundle.map(PathBuf::from), ..Options::default() },
        Path::new(machine::ADAPTER_PATH),
        None,
        disk,
    )
}

fn sandbox_json(bytes: &[u8]) -> serde_json::Value {
    serde_json::from_slice(bytes).expect("document JSON")
}

#[test]
fn given_scripts_join_at_each_start_without_replacing_the_model() {
    let (source, disk) = sandbox(true);
    let run = |options: &Options| {
        prepare_with(&source, options, Path::new(machine::ADAPTER_PATH), None, &disk).expect("scripted start")
    };
    let options = Options { scripts: vec!["/a.json".into(), "/b.json".into()], strict: true, ..Options::default() };
    let first = run(&options);
    let config = crate::path(&first.directories.config);
    let model = config.join("llm.json");
    let original = disk.tree.read(&model);
    let joined = config.join("llm-scripts.json");
    assert!(disk.effects.borrow().contains(&machine::Effect::Open(joined.clone(), 0o600, false)));
    assert_eq!(disk.metadata(&joined).unwrap().stat.mode, 0o600);
    assert_eq!(
        sandbox_json(&disk.tree.read(&joined)),
        serde_json::json!({
            "cued":[{"cue":"A","turns":["a"]},{"cue":"B","turns":["b"]}],
            "asked":[{"cue":"A","turns":["asked a"]},{"cue":"B","turns":["asked b"]}],
            "otherwise":["X"]
        })
    );
    assert_model_script_arguments(&first.steps, Some(&joined), true);
    disk.effects.borrow_mut().clear();
    write_with(&disk, Path::new("/b.json"), br#"{"otherwise":[]}"#, true).expect("new scripts");
    let retained = run(&options);
    assert_eq!(sandbox_json(&disk.tree.read(&joined))["otherwise"], serde_json::json!([]));
    assert_model_script_arguments(&retained.steps, Some(&joined), true);
    assert_eq!(disk.tree.read(&model), original);

    let plain = run(&Options::default());
    assert!(!disk.exists(&joined));
    assert_model_script_arguments(&plain.steps, None, false);
    assert_eq!(disk.tree.read(&model), original);
    let strict = run(&Options { strict: true, ..Options::default() });
    assert!(!disk.exists(&joined));
    assert_model_script_arguments(&strict.steps, None, true);
    assert_eq!(disk.tree.read(&model), original);
    assert!(disk.effects.borrow().iter().all(|effect| !effect.under(&model)), "the model node is never changed");
}

fn assert_model_script_arguments(steps: &[temper_forge_setup::Step], scripts: Option<&Path>, strict: bool) {
    let mut models = 0;
    for step in steps {
        if step.name() != b"fake model" {
            assert!(
                !step.arguments().iter().any(|word| [b"--scripts".as_slice(), b"--strict"].contains(&word.as_ref()))
            );
            continue;
        }
        models += 1;
        let mut expected = Vec::new();
        if let Some(path) = scripts {
            expected.extend([Box::from(b"--scripts".as_slice()), bytes(path)]);
        }
        if strict {
            expected.push(Box::from(b"--strict".as_slice()));
        }
        assert_eq!(step.arguments().get(6..).expect("base peer arguments"), expected);
    }
    assert_eq!(models, 1, "exactly one fake model step");
}

#[test]
fn a_reservation_that_loses_its_port_takes_another() {
    let mut calls = 0;
    let held: SocketAddr = ([127, 0, 0, 1], 9100).into();
    let reserved = reserve_from_with(
        &[],
        || {
            calls += 1;
            match calls {
                1 => Err(io::ErrorKind::AddrInUse.into()),
                2 => Ok(held),
                _ => panic!("only one replacement bind"),
            }
        },
        |port| Ok(*port),
    )
    .expect("another reservation after losing a port");
    assert_eq!(calls, 2);
    assert_eq!(reserved, held);
    assert_ne!(reserved.port(), 0);
}

#[test]
fn explicit_bundle_leaves_existing_empty_source_configuration_untouched() {
    let (source, disk) = sandbox(false);
    let config = crate::path(&source.config);
    let before = disk.metadata(config).expect("the empty directory already exists");
    let started = sandbox_start(&source, &disk, Some(BUNDLE)).expect("bundle in otherwise empty source");
    assert_eq!(disk.tree.list(config, 1).unwrap(), []);
    assert_eq!(disk.metadata(config).unwrap(), before);
    assert!(!disk.exists(&config_path(&source)));
    assert!(disk.effects.borrow().iter().all(|effect| !effect.under(config)), "no source configuration effects");
    assert_eq!(sandbox_json(&started.bytes)["application"]["clients"]["bundle"], BUNDLE);
    let document = disk.tree.read(&config_path(&started.directories));
    let validated = validate_with(&disk, &started.directories, &document, Path::new(machine::ADAPTER_PATH))
        .expect("ordinary effective admission");
    let secret = crate::path(&started.directories.data).join("worker.secret");
    assert!(validated.named.files.iter().any(|file| file.private && file.path == bytes(&secret)));
    assert_eq!(disk.tree.read(&secret).as_ref(), machine::SECRET);
}

#[test]
fn fresh_preparation_is_private_and_isolates_imported_resources() {
    use machine::Effect;
    let (source, disk) = sandbox(true);
    let started = sandbox_start(&source, &disk, None).expect("fresh preparation from provisioned source");
    let effective = &started.directories;
    assert_ne!(effective.config, source.config);
    assert_ne!(effective.data, source.data);
    assert_eq!(effective.cache, source.cache);
    let root = crate::path(&source.data).join("box/fake-backends");
    assert_eq!(effective.config, bytes(&root.join("config")));
    let document = disk.tree.read(&config_path(effective));
    let tree = sandbox_json(&started.bytes);
    assert!(tree["workers"][0].get("agent").is_none());
    assert_eq!(tree["application"]["clients"]["bundle"], BUNDLE);
    validate_with(&disk, effective, &document, Path::new(machine::ADAPTER_PATH)).expect("ordinary effective admission");
    assert!(started.listeners.is_none(), "import keeps the original source ports, not the init reservations");

    let effects = disk.effects.borrow();
    for name in ["", "config", "data"] {
        assert!(effects.contains(&Effect::Directory(root.join(name), 0o700)), "{name}");
    }
    for name in [
        "prepared",
        "lock",
        "config/github.json",
        "config/oauth.json",
        "config/llm.json",
        "config/forgejo.json",
        "config/fake-claude.path",
        "data/worker.secret",
        "data/providers.setting",
        "data/github-app.pem",
    ] {
        assert!(effects.contains(&Effect::Open(root.join(name), 0o600, false)), "{name}");
    }
    assert!(effects.contains(&Effect::Open(config_path(effective), 0o600, true)));
    assert_eq!(effects.last(), Some(&Effect::Sync(root.join("prepared"))), "publish the marker last");
}

#[test]
fn reuse_atomically_replaces_bundle_and_preserves_every_other_edit() {
    use machine::Effect;
    let (source, disk) = sandbox(true);
    let first = sandbox_start(&source, &disk, None).expect("fresh sandbox");
    let document = config_path(&first.directories);
    let mut file = disk.read_config(&first.directories).unwrap().file;
    file.owner = Some("retained-owner@localhost".into());
    write_with(&disk, &document, &temper_config_shell::render(&file, Shape::Standalone), true).unwrap();
    let mut edited = sandbox_json(&disk.read_config(&first.directories).unwrap().document);
    let old_node = disk.metadata(&document).unwrap().identity;
    let root = crate::path(&source.data).join("box/fake-backends");
    let data = root.join("data/retained-fixture");
    write_with(&disk, &data, b"keep this data", false).unwrap();
    let resources = [
        "prepared",
        "config/github.json",
        "config/oauth.json",
        "config/llm.json",
        "data/worker.secret",
        "data/retained-fixture",
    ]
    .map(|name| {
        let path = root.join(name);
        let content = disk.tree.read(&path);
        (path, content)
    });
    disk.remove(&Path::new(BUNDLE).join("index.html")).expect("old bundle unavailable");
    disk.effects.borrow_mut().clear();

    let second = sandbox_start(&source, &disk, Some(REPLACEMENT)).expect("replace retained bundle");
    edited["application"]["clients"]["bundle"] = REPLACEMENT.into();
    assert_eq!(sandbox_json(&second.bytes), edited);
    assert_eq!(second.bytes, disk.read_config(&second.directories).unwrap().document);
    assert_ne!(disk.metadata(&document).unwrap().identity, old_node);
    let staging = document.with_file_name(format!("{FILE}.bundle-staging"));
    assert!(!disk.exists(&staging));
    let effects = disk.effects.borrow();
    assert_eq!(
        effects.iter().filter(|effect| effect.under(&staging) || effect.under(&document)).collect::<Vec<_>>(),
        [
            &Effect::Open(staging.clone(), 0o600, false),
            &Effect::Write(staging.clone()),
            &Effect::Sync(staging.clone()),
            &Effect::Rename(staging, document),
        ]
    );
    for (path, bytes) in resources {
        assert_eq!(disk.tree.read(&path), bytes);
        assert!(effects.iter().all(|effect| !effect.under(&path)), "unchanged node {}", path.display());
    }
    drop(effects);
    let third = sandbox_start(&source, &disk, None).expect("no flag keeps the replacement selected");
    assert_eq!(third.bytes, second.bytes);
}

#[test]
fn repeating_the_same_bundle_preserves_document_bytes_and_inode() {
    let (source, disk) = sandbox(true);
    let first = sandbox_start(&source, &disk, None).expect("fresh sandbox");
    let document = config_path(&first.directories);
    let retained = [b"\n ".as_slice(), &disk.tree.read(&document), b" \n"].concat();
    write_with(&disk, &document, &retained, true).expect("custom formatting");
    let before = disk.metadata(&document).unwrap();
    disk.effects.borrow_mut().clear();

    let second = sandbox_start(&source, &disk, Some(BUNDLE)).expect("same bundle passes validation");
    assert_eq!(disk.tree.read(&document).as_ref(), retained);
    assert_eq!(second.bytes, disk.read_config(&second.directories).unwrap().document);
    assert_eq!(disk.metadata(&document).unwrap(), before);
    assert_no_bundle_publication(&disk, &document);
}

fn assert_no_bundle_publication(disk: &machine::Sandbox, document: &Path) {
    let staging = document.with_file_name(format!("{FILE}.bundle-staging"));
    assert!(!disk.exists(&staging));
    assert!(disk.effects.borrow().iter().all(|effect| !effect.under(&staging) && !effect.under(document)));
}

#[test]
fn invalid_replacement_bundle_preserves_retained_document() {
    let (source, disk) = sandbox(true);
    let first = sandbox_start(&source, &disk, None).expect("fresh sandbox");
    refuse_replacement(&source, &disk, &config_path(&first.directories), "/absent", "index.html");
}

#[test]
fn invalid_retained_configuration_refuses_a_valid_replacement_without_mutation() {
    let (source, disk) = sandbox(true);
    let first = sandbox_start(&source, &disk, None).expect("fresh sandbox");
    let document = config_path(&first.directories);
    let invalid = [&disk.tree.read(&document)[..], b"\n[application]\nworkers = []\n"].concat();
    write_with(&disk, &document, &invalid, true).expect("invalid retained document");
    assert_eq!(
        provisioned_bundle(&disk, Some(Path::new(REPLACEMENT)), temper_service::limits::stock().clients)
            .expect("valid replacement"),
        Path::new(REPLACEMENT)
    );
    refuse_replacement(&source, &disk, &document, REPLACEMENT, "application");
}

fn refuse_replacement(source: &Directories, disk: &machine::Sandbox, document: &Path, replacement: &str, member: &str) {
    let before = disk.tree.read(document);
    let node = disk.metadata(document).unwrap().identity;
    disk.effects.borrow_mut().clear();
    let why = sandbox_start(source, disk, Some(replacement)).err().expect("replacement refused");
    assert!(why.contains(member), "{why}");
    assert_no_bundle_publication(disk, document);
    assert_eq!(disk.tree.read(document), before);
    assert_eq!(disk.metadata(document).unwrap().identity, node);
}

#[test]
fn fake_adapter_is_retained_outside_fixed_worker_configuration() {
    let (source, disk) = sandbox(true);
    let first = sandbox_start(&source, &disk, None).expect("fresh sandbox");
    assert!(sandbox_json(&first.bytes)["application"].get("workers").is_none());
    let record = crate::path(&first.directories.config).join(ADAPTER);
    assert_eq!(disk.tree.read(&record).as_ref(), machine::ADAPTER_PATH.as_bytes());
    assert!(disk.effects.borrow().contains(&machine::Effect::Open(record.clone(), 0o600, false)));
    let adapter = PathBuf::from(machine::ADAPTER_PATH);
    assert_eq!(configured_claude_with(&first.directories, &disk).unwrap(), Some(adapter.clone()));
    assert_eq!(disk.program_reads.borrow().last(), Some(&adapter));
    disk.remove(&record).expect("absent record with the marker still present");
    disk.binary_reads.set(0);
    disk.program_reads.borrow_mut().clear();
    let why = configured_claude_with(&first.directories, &disk).expect_err("no fall back to PATH");
    assert!(why.contains(ADAPTER), "{why}");
    assert_eq!(disk.binary_reads.get(), 0);
    assert!(disk.program_reads.borrow().is_empty());
}

#[test]
fn own_app_key_is_the_seeded_apps_private_pem() {
    let (source, disk) = sandbox(true);
    let first = sandbox_start(&source, &disk, None).expect("fresh sandbox");
    let path = crate::path(&first.directories.data).join("github-app.pem");
    let key = disk.tree.read(&path);
    let github = crate::path(&first.directories.config).join("github.json");
    let forge = temper_fake::forge::Document::decode(&disk.tree.read(&github)).expect("generated GitHub document");
    assert!(key == forge.github_app_key().expect("seeded App"), "publish the seeded App's key");
    assert!(key.starts_with(b"-----BEGIN RSA PRIVATE KEY-----"));
    assert!(disk.effects.borrow().contains(&machine::Effect::Open(path.clone(), 0o600, false)));
    disk.effects.borrow_mut().clear();
    let _reused = sandbox_start(&source, &disk, None).expect("reuse keeps the key");
    assert!(disk.tree.read(&path) == key, "reuse retains the seeded bytes");
    assert!(disk.effects.borrow().iter().all(|effect| !effect.under(&path)));
}

#[test]
fn a_new_sandbox_takes_the_nine_ports_its_test_named_and_a_restart_keeps_them() {
    let (source, disk) = sandbox(false);
    let options = Options { bundle: Some(BUNDLE.into()), ..Options::default() };
    let chosen = chosen_ports(OsStr::new("8200,8201,8202,8203,8204,8205,8206,8207,8208")).expect("nine named ports");
    assert_eq!(chosen.listeners, [8200, 8201, 8202, 8203, 8204]);
    assert_eq!(chosen.peers, [8205, 8206, 8207, 8208]);
    let start = |ports| prepare_with(&source, &options, Path::new(machine::ADAPTER_PATH), Some(ports), &disk);
    let first = start(&chosen).expect("chosen fresh sandbox");
    assert_eq!(*disk.init_ports.borrow(), [[8200, 8204]], "the standalone file uses clients and webhooks");
    assert_eq!(*disk.binds.borrow(), [8205, 8206, 8207, 8208]);
    let doc = sandbox_json(&first.bytes);
    for (section, port) in [("clients", 8200), ("webhooks", 8204)] {
        assert_eq!(doc["application"][section]["listener"]["socket"], format!("127.0.0.1:{port}"));
    }
    let Peers { github, oauth, llm, forgejo } = first.peers;
    assert_eq!([github, oauth, llm, forgejo].map(|address| address.port()), [8205, 8206, 8207, 8208]);

    disk.init_ports.borrow_mut().clear();
    disk.binds.borrow_mut().clear();
    let other = chosen_ports(OsStr::new("8300,8301,8302,8303,8304,8305,8306,8307,8308")).expect("other named ports");
    let retained = start(&other).expect("retained sandbox ignores new choices");
    assert_eq!(retained.bytes, first.bytes);
    assert_eq!(retained.peers, first.peers);
    assert!(disk.init_ports.borrow().is_empty());
    assert!(disk.binds.borrow().is_empty());
}
