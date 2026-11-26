//! Cache effects for the pinned fetch (temper/docs/design/testing/shells.md, 3.4).
use super::{CHUNK, Event, Failed, Origin, Request, file_error, path};
use crate::zstd::{Magic, UnpackError, decode_zstd};
use skein_lib::sha256::Sha256;
use std::fs;
use std::io::{self, Seek as _, SeekFrom, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

pub(super) enum WireRequest {
    Get { origin: Origin, path: Box<[u8]> },
    Read,
}

pub(super) trait Cache {
    type Input: io::Read;
    type Part;

    fn open_input(&mut self, file: &Path) -> io::Result<Self::Input>;
    fn make_parent(&mut self, parent: &Path, mode: u32) -> io::Result<()>;
    fn open_part(&mut self, part: &Path, truncate: bool, mode: u32) -> io::Result<Self::Part>;
    fn set_mode(&mut self, part: &Self::Part, mode: u32) -> io::Result<()>;
    fn write_at(&mut self, part: &mut Self::Part, offset: u64, bytes: &[u8]) -> io::Result<()>;
    fn sync(&mut self, part: &Self::Part) -> io::Result<()>;
    fn rename(&mut self, part: &Path, file: &Path) -> io::Result<()>;
    fn sync_parent(&mut self, parent: &Path) -> io::Result<()>;
    fn remove(&mut self, part: &Path) -> io::Result<()>;
    fn close(&mut self, part: Self::Part);
}

pub(super) struct NativeCache;

impl Cache for NativeCache {
    type Input = fs::File;
    type Part = fs::File;

    fn open_input(&mut self, file: &Path) -> io::Result<fs::File> {
        fs::File::open(file)
    }
    fn make_parent(&mut self, parent: &Path, mode: u32) -> io::Result<()> {
        fs::DirBuilder::new().recursive(true).mode(mode).create(parent)
    }
    fn open_part(&mut self, part: &Path, truncate: bool, mode: u32) -> io::Result<fs::File> {
        fs::OpenOptions::new().write(true).create(true).truncate(truncate).mode(mode).open(part)
    }
    fn set_mode(&mut self, part: &fs::File, mode: u32) -> io::Result<()> {
        part.set_permissions(fs::Permissions::from_mode(mode))
    }
    fn write_at(&mut self, part: &mut fs::File, offset: u64, bytes: &[u8]) -> io::Result<()> {
        part.seek(SeekFrom::Start(offset)).and_then(|_| part.write_all(bytes))
    }
    fn sync(&mut self, part: &fs::File) -> io::Result<()> {
        part.sync_all()
    }
    fn rename(&mut self, part: &Path, file: &Path) -> io::Result<()> {
        fs::rename(part, file)
    }
    fn sync_parent(&mut self, parent: &Path) -> io::Result<()> {
        fs::File::open(parent).and_then(|dir| dir.sync_all())
    }
    fn remove(&mut self, part: &Path) -> io::Result<()> {
        fs::remove_file(part)
    }
    fn close(&mut self, part: fs::File) {
        drop(part);
    }
}

// The decoder writes in order; cache effects keep their explicit offsets.
struct PartWriter<'a, C: Cache> {
    cache: &'a mut C,
    part: &'a mut C::Part,
    offset: u64,
}

impl<C: Cache> Write for PartWriter<'_, C> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.cache.write_at(self.part, self.offset, bytes)?;
        self.offset += skein_lib::len_u64(bytes.len());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) struct Download<C: Cache> {
    pub(super) cache: C,
    part: Option<(Box<[u8]>, C::Part)>,
}

impl<C: Cache> Download<C> {
    pub(super) fn new(cache: C) -> Self {
        Self { cache, part: None }
    }

    pub(super) fn answer(
        &mut self,
        request: Request,
        wire: impl FnOnce(WireRequest) -> Result<Event, Failed>,
    ) -> Result<Event, Failed> {
        match request {
            Request::Hash { file } => hash(&file, self.cache.open_input(path(&file))).map(Event::Hashed),
            Request::Get { origin, path } => wire(WireRequest::Get { origin, path }),
            Request::Read => wire(WireRequest::Read),
            Request::Open { part } => {
                let handle = open(&mut self.cache, &part)?;
                self.close_part();
                self.part = Some((part, handle));
                Ok(Event::Done)
            }
            Request::Write { offset, bytes } => {
                let (name, handle) = self.part.as_mut().expect("write follows Open");
                self.cache.write_at(handle, offset, &bytes).map(|()| Event::Done).map_err(|why| file_error(name, &why))
            }
            Request::Unpack { packed, part, bound } => {
                self.close_part();
                let input = self.cache.open_input(path(&packed)).map_err(|why| file_error(&packed, &why))?;
                let handle = open(&mut self.cache, &part)?;
                let (name, handle) = self.part.insert((part, handle));
                let mut writer = PartWriter { cache: &mut self.cache, part: handle, offset: 0 };
                decode_zstd(io::BufReader::new(input), &mut writer, bound, Magic::Prefix).map(Event::Unpacked).map_err(
                    |why| match why {
                        UnpackError::Input(why) => file_error(&packed, &why),
                        UnpackError::Output(why) => file_error(name, &why),
                    },
                )
            }
            Request::Finish { part, file } => {
                let (_, handle) = self.part.take().expect("finish follows Open");
                let result = finish(&mut self.cache, &handle, &part, &file);
                self.cache.close(handle);
                result.map(|()| Event::Done)
            }
            Request::Remove { part } => {
                self.close_part();
                self.cache.remove(path(&part)).map(|()| Event::Done).map_err(|why| file_error(&part, &why))
            }
            Request::Say(_) | Request::End(_) => unreachable!("handled by the fetch driver"),
        }
    }

    pub(super) fn close_part(&mut self) {
        if let Some((_, handle)) = self.part.take() {
            self.cache.close(handle);
        }
    }
}

fn hash(file: &[u8], input: io::Result<impl io::Read>) -> Result<Option<[u8; 32]>, Failed> {
    let mut input = match input {
        Ok(file) => file,
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(file_error(file, &why)),
    };
    let mut hash = Sha256::new();
    let mut buffer = [0; CHUNK as usize];
    loop {
        let count = input.read(&mut buffer).map_err(|why| file_error(file, &why))?;
        if count == 0 {
            return Ok(Some(hash.finish()));
        }
        hash.update(&buffer[..count]);
    }
}

fn open<C: Cache>(cache: &mut C, part: &[u8]) -> Result<C::Part, Failed> {
    let at = path(part);
    let parent = at.parent().expect("cache path has a parent");
    cache.make_parent(parent, 0o700).map_err(|why| file_error(part, &why))?;
    let handle = cache.open_part(at, true, 0o600).map_err(|why| file_error(part, &why))?;
    if let Err(why) = cache.set_mode(&handle, 0o600) {
        cache.close(handle);
        return Err(file_error(part, &why));
    }
    Ok(handle)
}

fn finish<C: Cache>(cache: &mut C, handle: &C::Part, part: &[u8], file: &[u8]) -> Result<(), Failed> {
    cache.set_mode(handle, 0o755).map_err(|why| file_error(part, &why))?;
    cache.sync(handle).map_err(|why| file_error(part, &why))?;
    cache.rename(path(part), path(file)).map_err(|why| file_error(file, &why))?;
    cache.sync_parent(path(file).parent().expect("cache parent")).map_err(|why| file_error(file, &why))
}
