use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, Write},
    path::{Path, PathBuf},
    rc::Rc,
};

use skein_io::{
    file::Unsafe,
    kernel::{Kind, Stat},
};
use skein_shell::{Origin, Unread};
use temper_box_shell::{
    directories::{Directories, jig_directories},
    fake::{Disk, Inputs, Metadata},
};
use temper_forge_setup::{setup::Entry, setup_file};

pub const ROOT: &str = "/fixture";
const USER: u32 = 1000;

pub fn directories() -> Directories {
    Directories {
        config: b"/fixture/config".as_slice().into(),
        data: ROOT.as_bytes().into(),
        cache: b"/fixture/cache".as_slice().into(),
        origins: [Origin::Flag; 3],
    }
}

#[derive(Clone, Default)]
pub struct Capture(Rc<RefCell<Vec<u8>>>);
impl Capture {
    pub fn bytes(&self) -> Vec<u8> {
        self.0.borrow().clone()
    }
}
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct File {
    kind: Kind,
    mode: u32,
    id: u64,
    content: Capture,
}
impl File {
    fn metadata(&self) -> Metadata {
        Metadata {
            stat: Stat {
                kind: self.kind,
                mode: self.mode,
                owner: USER,
                links: 1,
                size: skein_lib::len_u64(self.content.0.borrow().len()),
            },
            identity: (1, self.id),
        }
    }
}
impl Write for File {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.content.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.content.flush()
    }
}

#[derive(Default)]
pub struct Tree {
    named: BTreeMap<PathBuf, File>,
    next: u64,
    setup: Option<Entry>,
}
impl Tree {
    pub fn new() -> Self {
        let mut tree = Self::default();
        tree.insert(ROOT.as_ref(), Kind::Directory, 0o700);
        tree
    }
    fn named(&self, path: &Path) -> io::Result<&File> {
        self.named.get(path).ok_or_else(|| io::Error::from_raw_os_error(2))
    }
    fn insert(&mut self, path: &Path, kind: Kind, mode: u32) -> File {
        self.next += 1;
        let file = File { kind, mode, id: self.next, content: Capture::default() };
        self.named.insert(path.into(), file.clone());
        file
    }
    pub fn seed(&mut self, path: &Path, bytes: &[u8]) {
        let parent = path.parent().expect("fixture parent");
        self.create_directories(parent, 0o700).expect("fixture directories");
        self.insert(path, Kind::File, 0o600).write_all(bytes).expect("fixture bytes");
    }
    pub fn setup_at(&mut self, address: std::net::SocketAddr) {
        let dirs = jig_directories(&directories());
        let setup = setup_file::decode(
            format!(r#"{{"version":1,"kind":"fake","listen":{{"socket":"{address}"}},"remote":"http://{address}","git":"demo/bin/git"}}"#).as_bytes(),
            &dirs,
        ).expect("fake setup");
        self.setup = Some(setup.entry(1, &dirs).expect("forge entry"));
    }
    pub fn capture(&self, path: &Path) -> Capture {
        self.named(path).expect("fixture file").content.clone()
    }
    pub fn bytes(&self, path: &Path) -> Vec<u8> {
        self.capture(path).bytes()
    }
    pub fn mode(&self, path: &Path) -> u32 {
        self.named(path).expect("fixture path").mode
    }
    pub fn contains(&self, path: &Path) -> bool {
        self.named.contains_key(path)
    }
    pub fn snapshot(&self, except: &Path) -> BTreeMap<PathBuf, (Kind, u32, Vec<u8>)> {
        self.named
            .iter()
            .filter(|(path, _)| *path != except)
            .map(|(path, file)| (path.clone(), (file.kind, file.mode, file.content.bytes())))
            .collect()
    }
    fn bounded(&self, path: &Path, bound: u32, private: bool) -> Result<Box<[u8]>, Unread> {
        let file = self.named.get(path).ok_or(Unread::Absent)?;
        let stat = file.metadata().stat;
        if private && let Some(why) = Unsafe::private_file(stat, USER) {
            return Err(Unread::Unsafe(why));
        }
        if stat.size > u64::from(bound) {
            return Err(Unread::TooLarge { max: bound });
        }
        Ok(file.content.bytes().into_boxed_slice())
    }
}
impl Disk for Tree {
    type File = File;
    fn user(&self) -> u32 {
        USER
    }
    fn metadata(&mut self, path: &Path) -> io::Result<Metadata> {
        Ok(self.named(path)?.metadata())
    }
    fn open(&mut self, path: &Path) -> io::Result<File> {
        self.named(path).cloned()
    }
    fn opened_metadata(&mut self, file: &File) -> io::Result<Metadata> {
        Ok(file.metadata())
    }
    fn read(&mut self, file: File, bound: u64) -> io::Result<Box<[u8]>> {
        let bytes = file.content.0.borrow();
        let end = bytes.len().min(usize::try_from(bound).unwrap_or(usize::MAX));
        Ok(bytes[..end].into())
    }
    fn create(&mut self, path: &Path, mode: u32) -> io::Result<File> {
        assert_eq!(self.named(path.parent().expect("private parent"))?.kind, Kind::Directory);
        if self.contains(path) {
            return Err(io::Error::from_raw_os_error(17));
        }
        Ok(self.insert(path, Kind::File, mode))
    }
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> io::Result<()> {
        file.write_all(bytes)
    }
    fn sync(&mut self, _: &File) -> io::Result<()> {
        Ok(())
    }
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        let file = self.named.remove(from).expect("the prepared image");
        self.named.insert(to.into(), file);
        Ok(())
    }
    fn create_directories(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        let missing: Vec<_> = path.ancestors().take_while(|path| !self.contains(path)).collect();
        for path in missing.into_iter().rev() {
            self.insert(path, Kind::Directory, mode);
        }
        Ok(())
    }
    fn remove_file(&mut self, path: &Path) -> io::Result<()> {
        assert_eq!(self.named(path)?.kind, Kind::File);
        self.named.remove(path);
        Ok(())
    }
    fn remove_directory(&mut self, path: &Path) -> io::Result<()> {
        assert_eq!(self.named(path)?.kind, Kind::Directory);
        assert!(!self.named.keys().any(|entry| entry != path && entry.starts_with(path)));
        self.named.remove(path);
        Ok(())
    }
}
impl Inputs for Tree {
    fn document(&mut self, path: &Path, bound: u32) -> Result<Box<[u8]>, Unread> {
        self.bounded(path, bound, false)
    }
    fn private_file(&mut self, path: &Path, bound: u32) -> Result<skein_lib::Secret, Unread> {
        self.bounded(path, bound, true).map(skein_lib::Secret::new)
    }
    fn forge_entry(&mut self, directories: &Directories, forge: u16) -> Result<Entry, String> {
        assert_eq!(directories.data.as_ref(), ROOT.as_bytes());
        assert_eq!(forge, 1);
        Ok(self.setup.clone().expect("supplied forge setup"))
    }
}
