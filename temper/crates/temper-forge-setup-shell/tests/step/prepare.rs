use super::{
    disk::{Action, Disk, Handle},
    fixtures,
    owned::owned,
};
use skein_fake_machine::{How, Is, Item};
use skein_lib::{Duration, Secret};
use skein_shell::Unread;
use std::{io, path::Path};
use temper_forge_setup::{
    place::{File, Place},
    prepare::{Prepare, Ran},
    setup::Owned,
    spec::Spec,
    state,
};
use temper_forge_setup_shell::{command, files, prepare, process};

const STATE: &str = "/fixture/forge-1/state";
const ROW: &str = "/fixture/forge-1/row";
const HEADERS: [&str; 2] = ["/fixture/forge-1/person_read.header", "/fixture/forge-1/person_write.header"];
const PASSWORD: &str = "/fixture/forge-1/person_password";
const RUNNER_SECRET: &str = "/fixture/forge-1/runner/secret";
const FRESH: &[u8] = b"Authorization: token private-token\n";
const KEPT: &[u8] = b"Authorization: token kept-token\n";

struct Host {
    disk: Disk,
    registrations: Vec<Box<[Box<[u8]>]>>,
}

impl Host {
    fn new() -> Self {
        Self {
            disk: Disk::new(&[Item::directory(b"fixture"), Item::directory(b"fixture/forge-1")]),
            registrations: Vec::new(),
        }
    }

    fn run(&mut self, owned: &Owned) -> Result<(), prepare::Failed> {
        let place = fixtures::place();
        prepare::run_with(self, &place, Prepare::new(owned.clone(), place.clone(), None))
    }

    fn read(&self, name: &str) -> Vec<u8> {
        self.disk.read(Path::new(name))
    }

    fn private_mode_was_requested(&self, name: &str) -> bool {
        let partial = Path::new(name).with_extension("partial");
        self.disk
            .calls
            .iter()
            .any(|call| call.action == Action::CreateNew(0o600) && call.path == partial && call.result.is_ok())
    }

    fn write_fixture(&mut self, name: &str, bytes: &[u8]) {
        files::write_private_with(&mut self.disk, Path::new(name), bytes).expect("fixture write");
    }
}

impl prepare::Host for Host {
    type Lock = Handle;

    fn lock(&mut self, path: &Path) -> Result<Handle, process::Refused> {
        self.disk.lock(path)
    }
    fn read_private(&mut self, path: &Path, bound: u32) -> Result<Secret, Unread> {
        let file = match self.disk.open_at(path, How::ReadNoFollow) {
            Ok(file) => file,
            Err(why) if why.kind() == io::ErrorKind::NotFound => return Err(Unread::Absent),
            Err(why) => panic!("fixture read: {why}"),
        };
        let stat = file.stat();
        assert_eq!(stat.is, Is::File);
        assert_eq!(stat.mode & 0o077, 0);
        assert_eq!(stat.links, 1);
        if stat.size > u64::from(bound) {
            return Err(Unread::TooLarge { max: bound });
        }
        Ok(Secret::new(file.bytes(bound).into_boxed_slice()))
    }
    fn exists(&mut self, path: &Path) -> bool {
        self.disk.open_at(path, How::Read).is_ok()
    }
    fn present(&mut self, place: &Place) -> io::Result<bool> {
        files::present_with(&mut self.disk, place)
    }
    fn clear(&mut self, place: &Place) -> io::Result<()> {
        files::clear_with(&mut self.disk, place)
    }
    fn layout(&mut self, place: &Place) -> io::Result<()> {
        files::layout_with(&mut self.disk, place)
    }
    fn write(&mut self, path: &Path, contents: &[u8]) -> io::Result<()> {
        files::write_private_with(&mut self.disk, path, contents)
    }
    fn backup_exists(&mut self, path: &Path) -> io::Result<bool> {
        files::exists_with(&mut self.disk, path)
    }
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        files::rename_with(&mut self.disk, from, to)
    }
    fn draw(&mut self) -> Result<Secret, i32> {
        Ok(Secret::new(Box::new([0x37; 32])))
    }
    fn capture(&mut self, spec: &Spec, within: Option<Duration>) -> Result<Ran, command::Error> {
        assert_eq!(within, None, "no source command in these preparations");
        let argv: Vec<_> = spec.arguments.iter().map(Box::as_ref).collect();
        let stdout: &[u8] = match argv.get(4..).unwrap_or(&argv) {
            [b"--version"] => match spec.program.as_ref() {
                b"/programs/forgejo" => b"forgejo version 17.0.1\n",
                b"/programs/runner" => b"forgejo-runner version v13.2.1\n",
                program => panic!("no capture for {program:?}"),
            },
            [b"migrate"] => b"",
            [b"admin", b"user", b"create", ..] => b"generated random password is 'private-password'\n",
            [b"admin", b"user", b"generate-access-token", ..] => b"private-token\n",
            [b"forgejo-cli", b"actions", b"register", ..] => {
                self.registrations.push(spec.arguments.clone());
                let mut row = argv.join(&b' ');
                row.push(b'\n');
                self.write_fixture(ROW, &row);
                b"kept-runner\n"
            }
            other => panic!("unexpected captured command {other:?}"),
        };
        Ok(fixtures::capture(0, stdout, b""))
    }
    fn note(&mut self, _: &[u8]) -> io::Result<()> {
        Ok(())
    }
}

fn check_registration(host: &Host, count: usize) {
    assert_eq!(host.registrations.len(), count);
    let argv = &host.registrations[count - 1];
    assert!(argv.windows(2).any(|pair| pair[0].as_ref() == b"--labels" && pair[1].as_ref() == b"temper-host:host"));
    assert!(String::from_utf8(host.read(ROW)).expect("fixture argv is text").contains("temper-host:host"));
}

#[test]
fn a_later_preparation_leaves_the_row_a_runner_declared() {
    let mut host = Host::new();
    let mut owned = owned(true);
    let configuration = fixtures::RUNNER_CONFIGURATION;
    host.run(&owned).expect("first preparation");
    check_registration(&host, 1);
    host.write_fixture(ROW, b"temper-host");
    host.run(&owned).expect("same settings");
    assert_eq!(host.read(ROW), b"temper-host", "later preparation keeps declared labels");
    owned.runner.as_mut().expect("runner").capacity = 2;
    host.run(&owned).expect("changed capacity");
    assert_eq!(host.read(ROW), b"temper-host");
    assert_eq!(host.registrations.len(), 1);
    let text = String::from_utf8(host.read(configuration)).expect("YAML");
    assert!(text.starts_with("runner:\n  capacity: 2\n"));
    assert!(text.contains("\n      uuid: \"kept-runner\"\n"));
    files::Disk::remove_file(&mut host.disk, Path::new(configuration)).expect("remove configuration");
    host.run(&owned).expect("registration after loss of the configuration");
    check_registration(&host, 2);
    let text = String::from_utf8(host.read(configuration)).expect("YAML");
    assert!(text.contains("\n      uuid: \"kept-runner\"\n"));
}

#[test]
fn person_headers_are_whole_private_and_each_missing_header_is_remade_alone() {
    let mut host = Host::new();
    let owned = owned(false);
    host.run(&owned).expect("first preparation");
    for header in HEADERS {
        assert_eq!(host.read(header), FRESH, "{header}");
        assert!(host.private_mode_was_requested(header), "{header}");
    }
    assert!(host.private_mode_was_requested(PASSWORD));
    let [read, write] = HEADERS;
    for (missing, kept) in [(read, write), (write, read)] {
        host.write_fixture(kept, KEPT);
        let original = host.disk.open_at(Path::new(kept), How::Read).expect("kept header");
        files::Disk::remove_file(&mut host.disk, Path::new(missing)).expect("remove one header");
        host.disk.calls.clear();
        host.run(&owned).expect("repeat preparation");
        assert_eq!(host.read(kept), KEPT);
        assert_eq!(original.stat().links, 1, "{kept} still names the same node");
        assert_eq!(host.read(missing), FRESH);
        assert!(host.private_mode_was_requested(missing));
    }
}

#[test]
fn an_interrupted_first_preparation_discards_both_headers_before_starting_again() {
    let mut host = Host::new();
    let owned = owned(false);
    host.run(&owned).expect("first preparation");
    let mut record = state::decode(&host.read(STATE)).expect("preparation state");
    record.prepared = false;
    host.write_fixture(STATE, state::encode(&record).expose());
    for header in HEADERS {
        host.write_fixture(header, b"Authorization: token abandoned-token\n");
    }
    host.disk.calls.clear();
    host.run(&owned).expect("preparation restarted");
    let first_creation = host
        .disk
        .calls
        .iter()
        .position(|call| matches!(call.action, Action::CreateNew(_)))
        .expect("fresh preparation writes files");
    for header in HEADERS {
        assert!(
            host.disk.calls[..first_creation].iter().any(|call| {
                call.action == Action::RemoveFile && call.path == Path::new(header) && call.result.is_ok()
            }),
            "{header} is removed before starting again"
        );
        assert_eq!(host.read(header), FRESH);
    }
    assert!(state::decode(&host.read(STATE)).expect("preparation state").prepared);
}

#[test]
fn changed_runner_capacity_needs_the_lock_and_keeps_the_registration() {
    let mut host = Host::new();
    let mut owned = owned(true);
    let configuration = fixtures::RUNNER_CONFIGURATION;
    host.run(&owned).expect("first preparation");
    let secret = host.disk.open_at(Path::new(RUNNER_SECRET), How::Read).expect("runner secret");
    let before = host.read(configuration);
    let original = host.disk.open_at(Path::new(configuration), How::Read).expect("original configuration");
    let lock = host.disk.lock(Path::new(fixtures::RUNNER_LOCK)).expect("runner lock held");
    host.run(&owned).expect("same configuration while the lock is held");
    assert_eq!(original.stat().links, 1, "identical configuration is not published again");
    owned.runner.as_mut().expect("runner").capacity = 2;
    assert!(matches!(host.run(&owned), Err(prepare::Failed::Held(File::RunnerLock))));
    assert_eq!(host.read(configuration), before);
    drop(lock);
    host.disk.calls.clear();
    host.run(&owned).expect("lock released");
    let after = host.read(configuration);
    assert!(after.starts_with(b"runner:\n  capacity: 2\n"));
    assert!(after.split(|byte| *byte == b'\n').skip(2).eq(before.split(|byte| *byte == b'\n').skip(2)));
    assert_eq!(secret.stat().links, 1, "still the original runner secret");
    assert!(!host.private_mode_was_requested(RUNNER_SECRET), "runner secret was not written");
    assert_eq!(host.registrations.len(), 1);
}
