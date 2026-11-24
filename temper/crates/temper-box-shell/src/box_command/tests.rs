use super::*;
use crate::{dispatch_box, init::startup::Startup, path, testing::init_fixtures::directories};
use skein_fake_machine::{Is, Item};
use std::{cell::RefCell, io, path::Path};
use temper_service_shell::configuration::Read;

fn read_configuration(startup: &RefCell<Startup>, dirs: &crate::Directories) -> Result<ConfigurationRead, String> {
    let config = path(&dirs.config);
    let first = startup.borrow_mut().first_entry(config);
    configuration_read_with(
        config,
        first,
        || startup.borrow_mut().init(true, None),
        || startup.borrow_mut().read_configuration(dirs),
        |closed| assert!(closed.is_empty(), "no listeners to close after a read without init"),
    )
}

#[test]
fn absent_or_empty_configuration_is_initialized_and_then_kept() {
    let root = Path::new("/fixture");
    let dirs = directories(root);
    for items in [&[][..], &[Item::directory(b"config")]] {
        let startup = RefCell::new(Startup::new(items, true));
        let (first, held, _) = read_configuration(&startup, &dirs).expect("initialization");
        // The scripted clients and webhooks listeners both stay held after init.
        let expected = [(11, 7100), (12, 7101)].map(|(fd, port)| (Fd::new(fd), port.try_into().expect("port")));
        assert_eq!(held, expected);
        assert_eq!(startup.borrow_mut().kind(&root.join("data/worker.secret")), Some(Is::File));
        assert_eq!(startup.borrow_mut().kind(&root.join("cache")), Some(Is::Directory));

        let (second, reopened, _) = read_configuration(&startup, &dirs).expect("second read");
        assert_eq!(second.document, first.document, "existing configuration is kept");
        assert!(reopened.is_empty(), "reading an existing configuration reserves no listeners");
    }
}

#[test]
fn a_nonempty_directory_without_configuration_is_refused_at_dispatch() {
    let root = Path::new("/fixture");
    let dirs = directories(root);
    let startup =
        RefCell::new(Startup::new(&[Item::directory(b"config"), Item::file(b"config/other", b"kept")], false));
    let read = read_configuration(&startup, &dirs);
    let missing = format!("{}: absent; temper-box init writes one", temper_config_shell::FILE);
    assert_eq!(read.as_ref().err(), Some(&missing));
    let exit = run_with(read, |_, _, _| panic!("a missing configuration never reaches startup"));
    assert_eq!(dispatch_box(exit), Err(format!("temper-box: {missing}\n")));
    let mut startup = startup.into_inner();
    assert_eq!(startup.read(&root.join("config/other")), b"kept");
    assert_eq!(startup.kind(&root.join("data")), None);
    assert_eq!(startup.kind(&root.join("cache")), None);
}

#[test]
fn box_planning_reports_each_missing_member_on_its_own_line() {
    let dirs = directories(Path::new("/fixture"));
    let read = Read {
        file: temper_config_shell::File::default(),
        document: b"{}".as_slice().into(),
        keys: temper_config_shell::Keys::default(),
    };
    let exit = run_with(Ok((read, Vec::new(), None)), |bytes, _, _| {
        configuration_steps(&dirs, bytes, Box::new([])).map(|_| Exit::Code(0))
    });
    assert_eq!(
        dispatch_box(exit),
        Err("temper-box: engine: missing\ntemper-box: workers: missing\ntemper-box: application: missing\n".into())
    );
}

#[test]
fn directory_errors_refuse_before_initializing_or_reading() {
    let config = Path::new("/fixture/config");
    for (entry, words) in [
        (Err(io::Error::new(io::ErrorKind::PermissionDenied, "cannot open directory")), "cannot open directory"),
        (Ok(Some(Err(io::Error::new(io::ErrorKind::NotFound, "entry disappeared")))), "entry disappeared"),
    ] {
        let result = configuration_read_with(
            config,
            entry,
            || panic!("an error is not an empty directory"),
            || panic!("a directory error stops the read"),
            |_| panic!("no listeners were acquired"),
        );
        assert_eq!(result.err().as_deref(), Some(format!("/fixture/config: {words}").as_str()));
    }
}

#[test]
fn a_failed_reread_closes_the_listeners_held_by_init() {
    let config = Path::new("/fixture/config");
    let mut startup = Startup::new(&[], true);
    let mut closed = Vec::new();
    let why = "configuration disappeared after init";
    let result =
        configuration_read_with(config, Ok(None), || startup.init(true, None), || Err(why.into()), |fds| closed = fds);
    assert_eq!(result.err().as_deref(), Some(why));
    assert_eq!(closed, [11, 12].map(Fd::new));
}

#[test]
fn first_configuration_without_openssl_admits_serve_and_supplies_one_warning() {
    let dirs = directories(Path::new("/fixture"));
    // No executable on the fixture PATH; its key maker panics if reached.
    let startup = RefCell::new(Startup::new(&[], true));
    let exit = run_with(read_configuration(&startup, &dirs), |bytes, _, opening| {
        let (steps, _) = configuration_steps(&dirs, bytes, Box::new([]))?;
        assert!(steps.always.iter().any(|step| step.name() == b"serve"), "serve is admitted");
        let expected = format!(
            "First run: no sign-in through temper: openssl is not on PATH\nFirst run: wrote the configuration, /fixture/config/{}\n",
            temper_config_shell::FILE
        );
        assert_eq!(opening.log_lines.as_ref(), [expected.into_bytes().into_boxed_slice()]);
        Ok(Exit::Code(0))
    });
    assert_eq!(exit, Ok(Exit::Code(0)));
}

#[test]
fn front_refuses_a_fixed_link_member_before_starting_steps() {
    let dirs = directories(Path::new("/fixture"));
    let mut document: serde_json::Value =
        serde_json::from_slice(&temper_config_shell::init_document(false)).expect("init's start document");
    document["engine"]["clients"]["link"] = false.into();
    let bytes = serde_json::to_vec(&document).expect("start JSON");
    assert_eq!(
        configuration_steps(&dirs, &bytes, Box::new([])).expect_err("no steps built"),
        "engine.clients.link: moved: temper fixes the clients' heartbeat, printed link and approval\n"
    );
}
