use super::{DeploymentIo, ProviderKey, StartupIo, Written, write_deployment_with};
use crate::{Directories, EnvironmentEntry, path, testing::init_fixtures::directories};
use skein_fake_machine::{Facts, How, Is, Item, Machine, Opened, Refusal};
use skein_io::kernel::{Done, Error, Family, Fd, Op};
use std::{
    collections::VecDeque,
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

pub(super) struct ListenerScript(VecDeque<(Op, Result<Done, Error>)>);

impl ListenerScript {
    /// Each row gives the bind request and the port returned. A different
    /// answer to a nonzero request means that port was already taken.
    pub(super) fn ports(bindings: [(u16, u16); 2]) -> Self {
        let backlog = jig_protocol_config::stock().io.backlog;
        let address = |port| ([127, 0, 0, 1], port).into();
        let mut steps = VecDeque::new();
        for (fd, (requested, chosen)) in [11, 12].map(Fd::new).into_iter().zip(bindings) {
            steps.push_back((Op::Socket { family: Family::Ipv4 }, Ok(Done::Fd(fd))));
            let bind = |port| Op::Bind { fd, addr: address(port) };
            let bound = Ok(Done::Bound(address(chosen)));
            if requested != 0 && requested != chosen {
                steps.push_back((bind(requested), Err(Error::AddressInUse)));
                steps.push_back((bind(0), bound));
            } else {
                steps.push_back((bind(requested), bound));
            }
            steps.push_back((Op::Listen { fd, backlog }, Ok(Done::Nothing)));
        }
        Self(steps)
    }

    pub(super) fn finish(&self) {
        assert!(self.0.is_empty());
    }
}

impl StartupIo for ListenerScript {
    fn perform(&mut self, kind: Op) -> Result<Done, Error> {
        let (expected, result) = self.0.pop_front().expect("scripted startup operation");
        assert_eq!(kind, expected);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Effect {
    Directory(PathBuf, u32),
    Write(PathBuf, u32),
    Link(PathBuf, PathBuf),
}

pub(crate) struct Startup {
    files: Machine,
    root: Opened,
    listeners: Option<ListenerScript>,
    lookups: Vec<(Box<[u8]>, Option<PathBuf>)>,
    pub(super) effects: Vec<Effect>,
}

impl Startup {
    pub(crate) fn new(items: &[Item], accepts: bool) -> Self {
        Self::with_listeners(items, accepts.then(|| ListenerScript::ports([(0, 7100), (7101, 7101)])))
    }

    pub(super) fn with_listeners(items: &[Item], listeners: Option<ListenerScript>) -> Self {
        let mut files = Machine::new();
        let root = files.lay(items);
        Self { files, root, listeners, lookups: Vec::new(), effects: Vec::new() }
    }

    pub(crate) fn init(&mut self, boxed: bool, program: Option<&Path>) -> Result<Written, String> {
        self.init_with_preferred(boxed, program, [0; 2])
    }

    pub(super) fn init_with_preferred(
        &mut self,
        boxed: bool,
        program: Option<&Path>,
        preferred: [u16; 2],
    ) -> Result<Written, String> {
        let dirs = directories(Path::new("/fixture"));
        let environment = [(Box::from(b"PATH".as_slice()), Box::from(b"/fixture/bin".as_slice()))];
        write_deployment_with(
            &dirs,
            &environment,
            Path::new("/fixture/bin/temper"),
            &preferred,
            program,
            boxed,
            ProviderKey::Generated,
            self,
        )
    }

    pub(super) fn lookups(&self) -> Vec<(&[u8], Option<&Path>)> {
        self.lookups.iter().map(|(name, found)| (name.as_ref(), found.as_deref())).collect()
    }

    pub(super) fn snapshot(&self) -> ([u8; 32], usize) {
        (self.files.state_digest(), self.effects.len())
    }

    pub(super) fn hold(&mut self, name: &Path) -> Opened {
        self.files.open(self.root, relative(name), How::Read).expect("retained file")
    }

    pub(super) fn stat(&self, file: Opened) -> Facts {
        self.files.stat(file)
    }

    pub(super) fn release(&mut self, file: Opened) {
        self.files.close(file);
    }

    pub(crate) fn read(&mut self, name: &Path) -> Vec<u8> {
        self.read_bounded(name, u32::MAX).expect("fixture file").into_vec()
    }

    fn read_bounded(&mut self, name: &Path, max: u32) -> Result<Box<[u8]>, skein_shell::Unread> {
        let file = match self.files.open(self.root, relative(name), How::Read) {
            Ok(file) => file,
            Err(Refusal::NotFound) => return Err(skein_shell::Unread::Absent),
            Err(why) => panic!("unexpected read refusal: {why:?}"),
        };
        let size = self.files.stat(file).size.try_into().expect("fixture length");
        let bytes = if size > max {
            Err(skein_shell::Unread::TooLarge { max })
        } else {
            Ok(self.files.read(file, 0, size).expect("read fixture file").into_boxed_slice())
        };
        self.files.close(file);
        bytes
    }

    pub(crate) fn kind(&mut self, name: &Path) -> Option<Is> {
        let opened = self.files.open(self.root, relative(name), How::PathNoFollow).ok()?;
        let kind = self.files.stat(opened).is;
        self.files.close(opened);
        Some(kind)
    }

    pub(crate) fn first_entry(&mut self, config: &Path) -> io::Result<Option<io::Result<()>>> {
        let dir = match self.files.open(self.root, relative(config), How::Directory) {
            Ok(dir) => dir,
            Err(Refusal::NotFound) => return Err(io::ErrorKind::NotFound.into()),
            Err(why) => panic!("unexpected open refusal: {why:?}"),
        };
        // One entry, with room for any name.
        let names = self.files.list(dir, 1, 4096).expect("list fixture directory");
        self.files.close(dir);
        Ok(names.first().map(|_| Ok(())))
    }

    fn configuration_file(
        &mut self,
        dirs: &Directories,
    ) -> Result<temper_config_shell::Read, temper_config_shell::Refusal> {
        let config = path(&dirs.config);
        let former = self.symlink_metadata(&config.join(temper_config_shell::FORMER));
        let provider_key = self.symlink_metadata(&path(&dirs.data).join("provider.key")).is_ok();
        temper_config_shell::read_with(
            temper_config_shell::Shape::Standalone,
            || former,
            |max| self.read_bounded(&config.join(temper_config_shell::FILE), max),
            || Ok(temper_config_shell::Facts { login: "ada".into(), hostname: "fixture".into(), provider_key }),
        )
    }

    pub(crate) fn read_configuration(
        &mut self,
        dirs: &Directories,
    ) -> Result<temper_service_shell::configuration::Read, String> {
        temper_service_shell::configuration::read_with(self.configuration_file(dirs), || self.open_directories(dirs))
    }

    pub(super) fn document(&mut self) -> serde_json::Value {
        let read = self.configuration_file(&directories(Path::new("/fixture"))).expect("configuration");
        serde_json::from_slice(&read.document).expect("start document")
    }
}

fn relative(name: &Path) -> &[u8] {
    name.strip_prefix("/fixture").expect("fixture path").as_os_str().as_bytes()
}

impl DeploymentIo for Startup {
    type ListenerIo = ListenerScript;

    fn symlink_metadata(&mut self, name: &Path) -> io::Result<()> {
        // The snapshot includes handle allocation. Probe on a copy so stat leaves it alone.
        let mut files = self.files.fork();
        let root = files.reopen_root(0);
        match files.open(root, relative(name), How::PathNoFollow) {
            Ok(_) => Ok(()),
            Err(Refusal::NotFound) => Err(io::ErrorKind::NotFound.into()),
            Err(why) => panic!("unexpected stat refusal: {why:?}"),
        }
    }

    fn find(&mut self, environment: &[EnvironmentEntry], name: &[u8]) -> Option<PathBuf> {
        let facts = temper_shell::lookup::candidates(environment, name)
            .into_iter()
            .map(|candidate| {
                let executable = self.files.open(self.root, relative(&candidate), How::Read).is_ok_and(|opened| {
                    let facts = self.files.stat(opened);
                    self.files.close(opened);
                    facts.is == Is::File && facts.mode & 0o111 != 0
                });
                (candidate, executable)
            })
            .collect::<Vec<_>>();
        let found = temper_shell::lookup::lookup(environment, name, &facts);
        self.lookups.push((name.into(), found.clone()));
        found
    }

    fn open(&mut self) -> Result<ListenerScript, String> {
        Ok(self.listeners.take().expect("init passed all destination checks"))
    }

    fn seed(&mut self) -> Result<skein_lib::Secret, i32> {
        Ok(skein_lib::Secret::new(Box::new([0; 32])))
    }

    fn open_directories(&mut self, directories: &Directories) -> Result<(), String> {
        self.make_private(path(&directories.data))?;
        self.make_private(path(&directories.cache))
    }

    fn login(&mut self) -> Result<Box<[u8]>, String> {
        Ok(b"ada".as_slice().into())
    }

    fn make_private(&mut self, name: &Path) -> Result<(), String> {
        let parent = relative(name.parent().expect("directory parent"));
        let parent = if parent.is_empty() { b"." } else { parent };
        let directory = self.files.open(self.root, parent, How::Directory).expect("parent directory");
        let result = self.files.make_directory(directory, name.file_name().expect("name").as_bytes(), 0o700);
        self.files.close(directory);
        match result {
            Ok(()) => {
                self.effects.push(Effect::Directory(name.to_owned(), 0o700));
                Ok(())
            }
            Err(Refusal::Exists) => Ok(()),
            Err(why) => Err(format!("{why:?}")),
        }
    }

    fn exists(&mut self, name: &Path) -> bool {
        let Ok(opened) = self.files.open(self.root, relative(name), How::Read) else {
            return false;
        };
        self.files.close(opened);
        true
    }

    fn write(&mut self, io: &mut ListenerScript, name: &Path, bytes: Box<[u8]>, mode: u32) -> Result<(), String> {
        io.finish();
        self.effects.push(Effect::Write(name.to_owned(), mode));
        let opened = self.files.open(self.root, relative(name), How::Create { mode }).expect("init creates its file");
        self.files.write(opened, 0, &bytes).expect("init writes its file");
        self.files.close(opened);
        Ok(())
    }

    fn symlink(&mut self, target: &Path, name: &Path) -> Result<(), String> {
        assert_eq!(name, Path::new("/fixture/data/demo/bin/git"));
        assert_eq!(target, Path::new("/fixture/bin/temper"));
        self.effects.push(Effect::Link(name.to_owned(), target.to_owned()));
        Ok(())
    }

    fn make_provider_key(&mut self, name: &Path, _: ProviderKey) -> Result<(), String> {
        self.symlink_metadata(name).expect("the startup fixture can keep a key but does not run OpenSSL");
        Ok(())
    }
}
