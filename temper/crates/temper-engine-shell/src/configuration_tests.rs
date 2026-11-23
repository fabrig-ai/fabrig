//! Engine command boundaries read only the engine's file (configuration/file.md, 6).
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};
use temper_config_shell::{FILE, FORMER};
use temper_service_shell::configuration::Read;
use temper_shell::directories::Directories;

const ENGINE: &str = r#"listen = "127.0.0.1:7100"
public = "https://temper.example.com"
workers = "127.0.0.1:7200"
owner = "sam@example.com"
[sign_in.google]
client = "1234.apps.googleusercontent.com"
secret = "secrets/google.secret"
[[sign_in.members]]
address = "sam@example.com"
login = "sam"
[test.google]
issuer = "http://127.0.0.1:9000"
authorization = "http://127.0.0.1:9000/auth"
token = "http://127.0.0.1:9000/token"
keys = "http://127.0.0.1:9000/keys"
transport = "plaintext"
"#;

fn fixture(text: &str) -> (skein_scratch::Scratch, Directories) {
    let scratch = skein_scratch::Scratch::new("engine-config-file");
    let arguments = [
        b"--config".as_slice(),
        scratch.path().as_os_str().as_bytes(),
        b"--data",
        scratch.path().as_os_str().as_bytes(),
        b"--cache",
        scratch.path().as_os_str().as_bytes(),
    ]
    .map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories");
    fs::write(scratch.path().join(FILE), text).expect("configuration file");
    (scratch, directories)
}

fn engine_fixture() -> (skein_scratch::Scratch, Directories) {
    let (scratch, directories) = fixture(ENGINE);
    fs::create_dir(scratch.path().join("secrets")).expect("fixture secrets");
    let secret = scratch.path().join("secrets/google.secret");
    fs::write(&secret, b"fixture-google-client-secret").expect("fixture secret");
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o600)).expect("private");
    (scratch, directories)
}

#[test]
fn toml_engine_prepares_serve_from_its_file() {
    let (_scratch, directories) = engine_fixture();
    let mut service =
        crate::serve::prepare_in("temper-engine", &[], b"/", &directories).unwrap_or_else(|why| panic!("{why}"));
    assert!(service.operations() > 0);
    crate::serve::close_limits_store(service.engine_component().store()).expect("close store");
}

#[test]
fn toml_engine_stopped_tools_read_the_file() {
    let (scratch, directories) = engine_fixture();
    let (service, deployment) =
        crate::stopped::prepare("temper-engine", &directories).unwrap_or_else(|why| panic!("{why}"));
    assert!(service.operations() > 0);
    assert_eq!(deployment.as_ref(), scratch.path().join("state").as_os_str().as_bytes());
}

#[test]
fn toml_engine_refuses_a_box_file_without_workers() {
    let directories = crate::test_configuration::directories(std::path::Path::new("/fixture"));
    let Err(why) = crate::serve::prepare_in_with_read(
        "temper-engine",
        &[],
        b"/",
        &directories,
        &temper_engine_service::stock::Given::Stock,
        |_, shape| read_scripted(b"listen = \"127.0.0.1:7100\"\n", shape),
    ) else {
        panic!("engine requires workers listener")
    };
    assert_eq!(why, format!("temper-engine serve: {FILE}: workers: required in temper-engine's file\n"));
}

pub(super) fn read_scripted(bytes: &[u8], shape: temper_config_shell::Shape) -> Result<Read, String> {
    let facts = temper_config_shell::Facts { login: "sam".into(), hostname: "fixture".into(), provider_key: false };
    let temper_config_shell::Read { file, document, keys } =
        temper_config_shell::parse(bytes, shape, &facts).map_err(|why| why.to_string())?;
    Ok(Read { file, document, keys })
}

fn fake_setup(directories: &Directories) {
    let jig = temper_service_shell::jig_directories(directories);
    let setup =
        temper_forge_setup::setup_file::chosen(temper_forge_setup::machine::BoxKind::Fake, 3007, b"sam", false, &jig)
            .expect("fake setup");
    temper_forge_setup_shell::setup_file::write(&jig, 7, &setup).expect("setup file");
    let root = std::path::Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    fs::create_dir_all(root.join("box/groups")).expect("group directory");
    fs::write(root.join("box/groups/7"), b"fake\n").expect("fake group");
}

#[test]
fn toml_engine_forge_prepare_reads_the_document() {
    use crate::forge::{Inputs, Preparation, start_with_preparation};
    use skein_fake_machine::{How, Is, Item, Machine};
    use std::{cell::RefCell, path::Path};
    use temper_config_shell::Shape;
    use temper_forge_setup::{machine::BoxKind, setup_file};

    let fixture = Path::new("/fixture");
    let directories = crate::test_configuration::directories(fixture);
    let jig = temper_service_shell::jig_directories(&directories);
    let user = 1000;
    let mut machine = Machine::for_user(user);
    let root = machine.lay(&[Item::file(FILE.as_bytes(), ENGINE.as_bytes())]);
    let machine = RefCell::new(machine);
    let relative = |path: &Path| path.strip_prefix(fixture).unwrap().as_os_str().as_bytes().to_vec();
    let inputs = Inputs {
        reader: crate::setup::check,
        listener_at: |_| panic!("prepare needs no listener"),
        setup: &|_, number, verb, _| {
            assert_eq!((number, verb), (7, "prepare"));
            setup_file::chosen(BoxKind::Fake, 3007, b"sam", false, &jig).map_err(|why| format!("{why:?}"))
        },
        configuration: &|directories, shape| {
            let file = crate::path(&directories.config).join(FILE);
            let mut machine = machine.borrow_mut();
            let opened = machine.open(root, &relative(&file), How::Read).unwrap();
            let length = u32::try_from(machine.stat(opened).size).unwrap();
            let bytes = machine.read(opened, 0, length).unwrap();
            machine.close(opened);
            read_scripted(&bytes, shape)
        },
        sign_in: &|_, _, _, _, _| panic!("prepare dispatches no sign-in"),
    };
    let preparation = Preparation {
        directory: &|directory, create| {
            assert!(create);
            machine.borrow_mut().make_directory(root, &relative(directory), 0o700).unwrap();
            Ok(user)
        },
        fake: &|file| {
            let mut machine = machine.borrow_mut();
            let opened = machine.open(root, &relative(file), How::Create { mode: 0o600 }).unwrap();
            machine.close(opened);
            Ok(())
        },
    };
    assert_eq!(
        start_with_preparation(
            "temper",
            Shape::EngineAlone,
            &directories,
            &["prepare".into(), "7".into()],
            None,
            &[],
            (&inputs, &preparation),
        )
        .unwrap(),
        0
    );
    let mut machine = machine.into_inner();
    let prepared = machine.open(root, b"playground-7/admin_token", How::PathNoFollow).unwrap();
    assert_eq!(machine.stat(prepared).is, Is::File);
    machine.close(prepared);
}

#[test]
fn forge_prepare_and_run_refuse_moved_limits_before_preparing() {
    use crate::forge::{Inputs, Preparation, report_exit, start_with_preparation};
    use std::{cell::Cell, path::Path};
    use temper_config_shell::Shape;
    use temper_forge_setup::setup_file;

    let directories = crate::test_configuration::directories(Path::new("/fixture"));
    let jig = temper_service_shell::jig_directories(&directories);
    let owned = setup_file::decode(
        br#"{"version":1,"kind":"owned","listen":{"socket":"127.0.0.1:3000"},
          "remote":"http://127.0.0.1:3000","forgejo":"/opt/forgejo","runner":null,
          "person":{"login":"ada","email":"ada@example.invalid"}}"#,
        &jig,
    )
    .expect("owned setup without a runner");
    let preparation = Preparation {
        directory: &|_, _| panic!("configuration refusal must precede directory preparation and launch"),
        fake: &|_| panic!("configuration refusal must precede token preparation"),
    };
    for words in [&["prepare", "1"][..], &["run", "1", "forgejo"][..], &["run", "1", "runner"][..]] {
        let arguments: Vec<_> = words.iter().map(std::ffi::OsString::from).collect();
        let setup_read = Cell::new(false);
        let inputs = Inputs {
            reader: |_, _| panic!("no deployment document from a refused configuration"),
            listener_at: |_| panic!("LISTEN_FDS is absent"),
            setup: &|given, number, verb, flags| {
                assert_eq!(given.config, directories.config);
                assert_eq!((number, verb, flags), (1, words[0], &[][..]));
                assert!(!setup_read.replace(true), "the forge setup is read once");
                Ok(owned.clone())
            },
            configuration: &|given, shape| {
                assert!(setup_read.get(), "the forge setup is read before the configuration");
                assert_eq!(given.config, directories.config);
                assert_eq!(shape, Shape::Standalone);
                read_scripted(b"limits = {}\n", shape)
            },
            sign_in: &|_, _, _, _, _| panic!("prepare and run do not sign in"),
        };
        let result = start_with_preparation(
            "temper-box",
            Shape::Standalone,
            &directories,
            &arguments,
            None,
            &[],
            (&inputs, &preparation),
        );
        let mut stderr = String::new();
        let code = report_exit("temper-box", &arguments, result, |_| panic!("no stdout"), |line| stderr = line);
        assert_eq!(code, 1, "{words:?}");
        assert_eq!(
            stderr,
            format!(
                "temper-box forge {}: forge 1: {FILE}:1: limits: moved: limits are settings; ask temper, or use temper ctl settings apply\n",
                words[0]
            ),
            "{words:?}"
        );
    }
}

#[test]
fn toml_engine_commands_refuse_the_former_file_alone_or_beside_the_current_file() {
    for present in [false, true] {
        let (scratch, directories) = fixture(ENGINE);
        fs::create_dir(scratch.path().join(FORMER)).expect("former directory");
        if !present {
            fs::remove_file(scratch.path().join(FILE)).expect("current absent");
        }
        let expected = format!(
            "{FORMER}: moved: the configuration is {FILE}; write it from temper-engine help config, then remove {FORMER}"
        );
        let Err(serve) = crate::serve::prepare_in("temper-engine", &[], b"/", &directories) else {
            panic!("both refused")
        };
        assert_eq!(serve, format!("temper-engine serve: {expected}\n"));
        let Err(stopped) = crate::stopped::prepare("temper-engine", &directories) else { panic!("both refused") };
        assert_eq!(stopped, expected);
        fake_setup(&directories);
        for words in [
            &["fetch", "7"][..],
            &["prepare", "7"][..],
            &["run", "7", "forgejo"][..],
            &["run", "7", "runner"][..],
            &["source", "7"][..],
            &["sign-in", "7"][..],
        ] {
            let arguments: Vec<_> = words.iter().map(std::ffi::OsString::from).collect();
            let why =
                crate::forge::start(&directories, &arguments, None, &[]).unwrap_err().line_with_binary("temper-engine");
            assert_eq!(why, format!("temper-engine forge {}: forge 7: {expected}\n", words.first().expect("verb")));
        }
        assert!(!scratch.path().join("playground-7/admin_token").exists());
    }
}
