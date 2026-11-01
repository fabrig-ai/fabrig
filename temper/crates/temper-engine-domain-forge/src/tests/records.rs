use super::*;
use crate::record as r;

pub(super) fn forge() -> Forge {
    Forge {
        number: 2,
        kind: Kind::GitHub,
        api: r::Address::Host { name: b"api.github.com".as_slice().into(), port: 443 },
        transport: r::Transport::Tls { server_name: b"api.github.com".as_slice().into(), trust: None },
        git: b"https://github.com".as_slice().into(),
        git_program: None,
        web: Some(b"https://github.com".as_slice().into()),
        read_only: false,
        identity: r::Identity { number: 1, login: Box::new([]), name: Box::new([]), email: Box::new([]) },
        files: None,
        setup: None,
        first: None,
        playground: false,
        removing: false,
    }
}
fn account() -> r::Account {
    r::Account { login: b"temper".as_slice().into(), email: b"temper@localhost".as_slice().into() }
}
fn setup(mode: r::SetupMode) -> r::Setup {
    r::Setup { directory: b"/data/forge-3".as_slice().into(), bot: account(), orgs: Box::new([]), mode, asked: false }
}
fn owned() -> r::OwnedEntry {
    r::OwnedEntry {
        public: None,
        fetch: r::Fetch { forgejo: true, runner: true, mirror: None },
        forgejo: b"/cache/forgejo".as_slice().into(),
        runner: Some(r::Runner {
            binary: b"/cache/runner".as_slice().into(),
            labels: Box::new([b"temper-host:host".as_slice().into()]),
            capacity: 1,
            path: b"/usr/local/bin:/usr/bin:/bin".as_slice().into(),
        }),
        listen: core::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
        person: account(),
    }
}
#[test]
fn restored_rows_are_given_back() {
    let mut d = domain();
    let row = forge();
    let playground =
        Playground { state: PlaygroundState::Resetting, forge: Some(1), project: Some(2), account: Some(3) };
    let acts = [([2; 16], KeptAct::RemoveForge { forge: 1 }), ([1; 16], KeptAct::RemoveProject { project: 2 })];
    for row in [
        Stored::Forge(row.clone()),
        Stored::Forges { next: 4 },
        Stored::Playground(playground),
        Stored::Kept(Kept { key: acts[0].0, act: acts[0].1 }),
        Stored::Kept(Kept { key: acts[1].0, act: acts[1].1 }),
    ] {
        assert!(outputs(&mut d, Event::Restore { record: row }).is_empty());
    }
    let mut earlier = row.clone();
    earlier.number = 1;
    assert!(outputs(&mut d, Event::Restore { record: Stored::Forge(earlier.clone()) }).is_empty());
    assert_eq!(d.forge_record(2), Some(&row));
    assert_eq!(d.forge_record(9), None);
    assert!(d.forge_records().eq([&earlier, &row]));
    assert_eq!(d.next_forge(), 4);
    assert_eq!(d.playground(), Some(&playground));
    assert!(d.kept().eq([(&acts[1].0, &acts[1].1), (&acts[0].0, &acts[0].1)]));
    assert_eq!(d.restore_fit_failure(), None);
    assert_eq!(d.fit_restored(&LIMITS), Ok(()));
}
#[test]
fn next_forge_is_one_without_its_row() {
    let d = domain();
    assert_eq!(d.next_forge(), 1);
    assert_eq!(d.playground(), None);
    assert_eq!(d.forge_records().count(), 0);
    assert_eq!(d.kept().count(), 0);
}
#[test]
fn configured_from_record_follows_each_mode() {
    use skein_forge::forgejo::pin;
    let mut row = forge();
    let mut expected = Configured {
        admin_connect: false,
        bot: Box::new([]),
        owned: None,
        git: b"https://github.com".as_slice().into(),
        web: Some(b"https://github.com".as_slice().into()),
        read_only: false,
        number: 2,
        kind: Kind::GitHub,
        mode: Mode::Connected,
        asked: false,
    };
    assert_eq!(Configured::from_record(&row), expected);
    row.number = 1;
    row.kind = Kind::Forgejo;
    row.git = b"http://127.0.0.1:8369".as_slice().into();
    row.web = None;
    row.playground = true;
    row.setup = Some(setup(r::SetupMode::Fake(r::FakeEntry {
        listen: core::net::SocketAddr::from(([127, 0, 0, 1], 8369)),
        git: b"/data/demo/bin/git".as_slice().into(),
        owner: b"playground".as_slice().into(),
        repositories: Box::new([]),
    })));
    expected.number = 1;
    expected.kind = Kind::Forgejo;
    expected.git = b"http://127.0.0.1:8369".as_slice().into();
    expected.web = None;
    expected.bot = b"temper".as_slice().into();
    expected.mode = Mode::Fake;
    expected.asked = true;
    assert_eq!(Configured::from_record(&row), expected);
    row.number = 3;
    row.git = b"http://127.0.0.1:3000".as_slice().into();
    row.playground = false;
    let mut entry = setup(r::SetupMode::Owned(owned()));
    entry.asked = true;
    row.setup = Some(entry);
    expected.number = 3;
    expected.git = b"http://127.0.0.1:3000".as_slice().into();
    expected.mode = Mode::Owned;
    expected.owned = Some(Owned {
        forgejo: pin::VERSION.as_bytes().into(),
        runner: Some(pin::RUNNER_VERSION.as_bytes().into()),
        password: b"/data/forge-3/person_password".as_slice().into(),
    });
    assert_eq!(Configured::from_record(&row), expected);
    row.setup = Some(setup(r::SetupMode::Managed { admin_token: None }));
    expected.mode = Mode::Managed;
    expected.owned = None;
    expected.admin_connect = true;
    expected.asked = false;
    assert_eq!(Configured::from_record(&row), expected);
    row.setup = Some(setup(r::SetupMode::Managed { admin_token: Some(b"/data/admin".as_slice().into()) }));
    expected.admin_connect = false;
    assert_eq!(Configured::from_record(&row), expected);
    row.git = skein_lib::bytes::zeroed(usize::try_from(PULL_ADDRESS_BYTES).unwrap() + 1);
    expected.git = Box::new([]);
    row.read_only = true;
    expected.read_only = true;
    assert_eq!(Configured::from_record(&row), expected);
}
#[test]
fn fits_refuses_a_field_past_its_bound() {
    let mut row = forge();
    assert!(row.fits());
    row.git = skein_lib::bytes::zeroed(r::PATH_BYTES);
    assert!(row.fits());
    row.git = skein_lib::bytes::zeroed(r::PATH_BYTES + 1);
    assert!(!row.fits());
    row = forge();
    row.identity.login = skein_lib::bytes::zeroed(r::HOST_BYTES);
    assert!(row.fits());
    row.identity.login = skein_lib::bytes::zeroed(r::HOST_BYTES + 1);
    assert!(!row.fits());
    row = forge();
    let mut entry = setup(r::SetupMode::Owned(owned()));
    entry.orgs = orgs(r::SETUP_ORGS);
    row.setup = Some(entry.clone());
    assert!(row.fits());
    entry.orgs = orgs(r::SETUP_ORGS + 1);
    row.setup = Some(entry);
    assert!(!row.fits());
    let mut l = LIMITS;
    l.forges = u32::MAX;
    assert_eq!(kept_bound(&l), None);
    assert!(!valid(&l));
    assert_eq!(kept_bound(&LIMITS), Some(LIMITS.tasks + 1 + 2 * LIMITS.forges + 2));
}

fn orgs(count: usize) -> Box<[Box<[u8]>]> {
    let mut names = skein_lib::List::with_capacity(u32::try_from(count).unwrap());
    for _ in 0..count {
        names.push(Box::<[u8]>::from([])).unwrap();
    }
    names.into_boxed()
}
