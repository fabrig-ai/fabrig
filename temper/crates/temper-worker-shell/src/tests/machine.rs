use skein_fake_machine::{Facts, How, Is, Item, Machine, Opened, Refusal};
use std::{cell::RefCell, ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
use temper_config_shell::{FILE, Shape};
use temper_service_shell::configuration::Read;

pub(crate) const ROOT: &str = "/fixture/worker";

pub(crate) struct Disk {
    files: RefCell<Machine>,
    root: Opened,
}

impl Disk {
    pub(crate) fn new(items: &[Item]) -> Self {
        let mut files = Machine::new();
        let root = files.lay(items);
        Self { files: RefCell::new(files), root }
    }

    fn relative(path: &Path) -> &[u8] {
        path.strip_prefix(ROOT).expect("worker fixture path").as_os_str().as_bytes()
    }

    pub(crate) fn metadata(&self, path: &Path) -> Result<Facts, Refusal> {
        let mut files = self.files.borrow_mut();
        let opened = files.open(self.root, Self::relative(path), How::PathNoFollow)?;
        let facts = files.stat(opened);
        files.close(opened);
        Ok(facts)
    }

    pub(crate) fn executable(&self, path: &Path) -> bool {
        self.metadata(path).is_ok_and(|facts| facts.is == Is::File && facts.mode & 0o111 != 0)
    }

    pub(crate) fn former_agent(&self) -> std::io::Result<()> {
        match self.metadata(&Path::new(ROOT).join("agent.json")) {
            Ok(_) => Ok(()),
            Err(Refusal::NotFound) => Err(std::io::ErrorKind::NotFound.into()),
            Err(why) => panic!("former agent fixture: {why:?}"),
        }
    }

    pub(crate) fn read(&self, path: &Path) -> Vec<u8> {
        let mut files = self.files.borrow_mut();
        let file = files.open(self.root, Self::relative(path), How::Read).expect("fixture file");
        let size = u32::try_from(files.stat(file).size).expect("fixture size");
        let bytes = files.read(file, 0, size).expect("fixture bytes");
        files.close(file);
        bytes
    }

    pub(crate) fn create(&self, path: &Path, bytes: &[u8], mode: u32) {
        let mut files = self.files.borrow_mut();
        let file = files.open(self.root, Self::relative(path), How::Create { mode }).expect("new fixture file");
        files.write(file, 0, bytes).expect("fixture bytes");
        files.close(file);
    }

    pub(crate) fn overwrite(&self, path: &Path, bytes: &[u8]) {
        let mut files = self.files.borrow_mut();
        let file = files.open(self.root, Self::relative(path), How::Write).expect("existing fixture file");
        files.truncate(file, 0).expect("truncate fixture");
        files.write(file, 0, bytes).expect("fixture bytes");
        files.close(file);
    }

    pub(crate) fn rename(&self, from: &Path, to: &Path) {
        self.files
            .borrow_mut()
            .rename(self.root, Self::relative(from), self.root, Self::relative(to), false)
            .expect("rename fixture file");
    }

    pub(crate) fn remove(&self, path: &Path) {
        self.files.borrow_mut().remove(self.root, Self::relative(path), false).expect("remove fixture file");
    }

    fn mkdir(&self, path: &Path) {
        let parent = Self::relative(path.parent().expect("directory parent"));
        let name = path.file_name().expect("directory name").as_bytes();
        let mut files = self.files.borrow_mut();
        let directory = if parent.is_empty() {
            self.root
        } else {
            files.open(self.root, parent, How::Directory).expect("fixture parent")
        };
        files.make_directory(directory, name, 0o700).expect("fixture directory");
        if directory != self.root {
            files.close(directory);
        }
    }

    pub(crate) fn configuration(&self, shape: Shape) -> Read {
        let bytes = self.read(&Path::new(ROOT).join(FILE));
        Read::from(super::configuration::parse(&bytes, shape))
    }

    pub(crate) fn login(&self) -> Result<std::process::Command, String> {
        let root = Path::new(ROOT);
        let directories = super::configuration::directories(root);
        crate::program_login::command_with(
            &directories,
            OsStr::new("source"),
            [("PATH".into(), root.as_os_str().to_owned())],
            |shape| {
                let read = self.configuration(shape);
                let programs = read.file.programs.clone();
                Ok((read, programs))
            },
            |part, name, environment| {
                crate::program_login::from_machine_with(
                    part,
                    &directories,
                    name,
                    environment,
                    |path| self.executable(path),
                    |programs, _, name| {
                        self.mkdir(programs);
                        let source = programs.join(name);
                        self.mkdir(&source);
                        Ok(source)
                    },
                )
            },
        )
    }

    pub(crate) fn pin(
        &self,
        root: &Path,
        agent: &jig_protocol_config::Agent,
        configuration: &[u8],
    ) -> jig_protocol_config::Agent {
        crate::worker::pin_with(
            root,
            agent,
            configuration,
            |directory| {
                self.mkdir(directory);
                Ok(())
            },
            |from, to| {
                self.create(to, &self.read(from), 0o700);
                Ok(())
            },
            |to, bytes| {
                self.create(to, bytes, 0o600);
                Ok(())
            },
        )
        .expect("pin fixture")
    }
}
