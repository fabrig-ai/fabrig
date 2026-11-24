use super::*;
use crate::testing::provider_fixture as local;
use skein_fake_machine::{How, Item, Machine, Opened, Refusal};

const COPY: &str = "data/providers.setting";
const TEMPORARY: &str = "data/providers.setting.new";

#[derive(Debug, PartialEq, Eq)]
enum Effect {
    Remove,
    Create(u32),
    Write,
    SyncFile,
    Rename,
    SyncDirectory,
}

const PUBLISH: [Effect; 6] =
    [Effect::Remove, Effect::Create(0o600), Effect::Write, Effect::SyncFile, Effect::Rename, Effect::SyncDirectory];

struct Disk {
    machine: Machine,
    data: Opened,
    temporary: Option<Opened>,
    effects: Vec<Effect>,
}

impl Disk {
    fn new(items: &[Item]) -> Self {
        let mut machine = Machine::for_user(1000);
        let data = machine.lay(items);
        Self { machine, data, temporary: None, effects: Vec::new() }
    }

    fn publish(&mut self, values: &[ProviderSetting]) -> io::Result<bool> {
        self.effects.clear();
        let result = write_with(Path::new(COPY), values, self);
        if let Some(file) = self.temporary.take() {
            self.machine.close(file);
        }
        result
    }

    fn read_back(&mut self) -> Box<[ProviderSetting]> {
        let mut log = Vec::new();
        let values = read_using(Path::new(COPY), Box::new([]), &mut log, |path| self.previous(path).map(Some));
        assert!(log.is_empty(), "the provider copy decoded");
        values
    }
}

fn name(path: &Path) -> &[u8] {
    assert_eq!(path.parent(), Some(Path::new("data")));
    path.file_name().expect("file in data").as_encoded_bytes()
}

fn refused(why: Refusal) -> io::Error {
    io::Error::from_raw_os_error(temper_shell::directories::errno(skein_fake_machine::kernel_error(why)))
}

impl Publication for Disk {
    type File = Opened;

    fn previous(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        let file = self.machine.open(self.data, name(path), How::Read).map_err(refused)?;
        let len = u32::try_from(self.machine.stat(file).size).expect("fixture length");
        let bytes = self.machine.read(file, 0, len).map_err(refused);
        self.machine.close(file);
        bytes
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        assert_eq!(path, Path::new(TEMPORARY));
        self.effects.push(Effect::Remove);
        self.machine.remove(self.data, name(path), false).map_err(refused)
    }

    fn create_new(&mut self, path: &Path, mode: u32) -> io::Result<Opened> {
        assert_eq!(path, Path::new(TEMPORARY));
        self.effects.push(Effect::Create(mode));
        let file = self.machine.open(self.data, name(path), How::Create { mode }).map_err(refused)?;
        self.temporary = Some(file);
        Ok(file)
    }

    fn write_all(&mut self, file: &mut Opened, bytes: &[u8]) -> io::Result<()> {
        self.effects.push(Effect::Write);
        self.machine.write(*file, 0, bytes).map_err(refused)
    }

    fn sync_file(&mut self, file: &Opened) -> io::Result<()> {
        self.effects.push(Effect::SyncFile);
        self.machine.sync(*file);
        Ok(())
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        self.effects.push(Effect::Rename);
        self.machine.rename(self.data, name(from), self.data, name(to), false).map_err(refused)
    }

    fn sync_directory(&mut self, path: &Path) -> io::Result<()> {
        assert_eq!(path, Path::new("data"));
        self.effects.push(Effect::SyncDirectory);
        self.machine.sync(self.data);
        Ok(())
    }
}

#[test]
fn restore_and_changes_are_whole_private_and_unchanged_bytes_are_not_written() {
    let mut disk = Disk::new(&[Item::file(name(Path::new(TEMPORARY)), b"stale")]);
    let mut values = [local()];
    assert!(disk.publish(&values).expect("restore writes an absent copy"));
    assert_eq!(disk.effects, PUBLISH);
    assert_eq!(disk.read_back().as_ref(), &values);

    assert!(!disk.publish(&values).expect("equal bytes kept"));
    assert!(disk.effects.is_empty(), "equal bytes ask for no effect, including rename");

    values[0].endpoints[0].address = config::EndpointAddress::Socket("127.0.0.1:8081".parse().expect("changed port"));
    assert!(disk.publish(&values).expect("replace with changed bytes"));
    assert_eq!(disk.effects, PUBLISH);
    assert_eq!(disk.read_back().as_ref(), &values);
    assert_eq!(disk.previous(Path::new(TEMPORARY)).expect_err("no temporary remains").kind(), io::ErrorKind::NotFound);
}

#[test]
fn absent_and_unreadable_copies_fall_back_and_bad_copies_explain_why() {
    let scratch = skein_scratch::Scratch::new("providers-fallback");
    let path = scratch.path().join("providers.setting");
    let mut log = Vec::new();
    assert_eq!(read(&path, Box::new([local()]), &mut log).as_ref(), &[local()]);
    assert_eq!(log, [] as [u8; 0]);
    for bad in [false, true] {
        if bad {
            std::fs::write(&path, b"not JSON").unwrap();
        } else {
            std::fs::create_dir(&path).unwrap();
        }
        assert_eq!(read(&path, Box::new([local()]), &mut log).as_ref(), &[local()]);
        let line = String::from_utf8(core::mem::take(&mut log)).unwrap();
        assert!(line.starts_with("providers.setting: "), "{line}");
        assert!(line.ends_with("; the stock's providers run until the store's are kept again\n"), "{line}");
        if !bad {
            std::fs::remove_dir(&path).unwrap();
        }
    }
}
