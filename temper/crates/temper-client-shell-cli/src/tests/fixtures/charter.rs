//! A deployment's one charter and the directory tree the terminal can write to.
use super::*;
use skein_lib::{List, Reader};
use std::collections::BTreeSet;

pub const NUMBER: u32 = 2;
pub const NAME: &[u8] = b"implementer";
pub const LEAF: &[u8] = b"implementer.md";
const REVISION: u32 = 13;
const KEY: w::Setting = w::Setting::Charter(w::SettingNumber { number: NUMBER });
const UNCHANGED: &[u8] = b"Keep my own words.\n";

pub struct Part {
    pub on: u32,
    pub words: &'static [u8],
}
#[derive(Clone)]
pub struct Charter {
    pub name: Option<&'static [u8]>,
    pub instructions: &'static [u8],
    pub parts: &'static [Part],
    pub overlay: Option<Overlay>,
}
#[derive(Clone)]
pub struct Overlay {
    pub instructions: &'static [u8],
    pub parts: &'static [Part],
}
pub struct Case {
    charter: Charter,
    directories: BTreeSet<Vec<u8>>,
    opens: BTreeMap<Fd, Vec<u8>>,
    pub placed_in: Option<Vec<u8>>,
    pub destination_opened: bool,
    pub asked: Vec<w::Read>,
    pub target: Option<Target>,
    pub linked_directory: Option<&'static [u8]>,
}
#[derive(Clone, Copy)]
pub enum Target {
    File,
    Link,
    Raced,
}
impl Case {
    pub fn new(charter: Charter) -> Self {
        Self {
            charter,
            directories: BTreeSet::from([Vec::new()]),
            opens: BTreeMap::from([(FILE_PARENT, Vec::new())]),
            placed_in: None,
            destination_opened: false,
            asked: Vec::new(),
            target: None,
            linked_directory: None,
        }
    }

    fn reply(&self, read: &w::Read) -> w::ReadAnswer {
        let value = if read == &w::Read::Settings {
            w::ReadValue::Settings(self.inventory())
        } else {
            let w::Read::Setting(read) = read else { panic!("charter copy sent a different read: {read:?}") };
            assert_eq!((read.revision, &read.proposal), (REVISION, &None));
            if read.setting != KEY {
                return w::ReadAnswer::None(w::ReadAnswerNone { value: w::ReadNone::Unknown });
            }
            w::ReadValue::Setting(self.document())
        };
        w::ReadAnswer::Value(w::Owned { owner: w::Owner::Jig, version: w::VERSION, record: encode!(value) })
    }

    fn inventory(&self) -> w::SettingsDocument {
        let mut settings = List::with_capacity(1);
        settings
            .push(w::SettingLine {
                setting: KEY,
                shown: None,
                follows: if self.charter.overlay.is_some() {
                    w::SettingFollows::Overlay
                } else {
                    w::SettingFollows::Change
                },
                last: Some(REVISION),
                application: None,
                name: self.charter.name.map(Box::from),
            })
            .expect("one charter in inventory");
        w::SettingsDocument {
            newest: REVISION,
            settings,
            held: List::with_capacity(0),
            pending: List::with_capacity(0),
        }
    }

    fn document(&self) -> w::SettingDocument {
        let charter = &self.charter;
        w::SettingDocument {
            setting: KEY,
            revision: REVISION,
            value: Some(w::SettingValue::Charter(Box::new(charter_record(charter)))),
            application: None,
            proposed: None,
            overlay: charter.overlay.as_ref().map(overlay_record),
        }
    }
}

impl Frame {
    pub(super) fn read_charter_setting(&mut self, fd: Fd) {
        let request = &self.sockets.get(&fd).expect("read socket").request;
        let start = skein_lib::bytes::find(request, b"\r\n\r\n").expect("HTTP head");
        let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(&request[start + 4..])).expect("owned read");
        let read = w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record.as_ref())).expect("setting read");
        let charter = self.charter.as_mut().expect("charter fixture");
        let answer = charter.reply(&read);
        charter.asked.push(read);
        self.reply(fd, &encode!(answer));
    }

    pub(super) fn open_charter(&mut self, root: Fd, path: &[u8], how: OpenHow) -> Option<Result<Done, Error>> {
        let charter = self.charter.as_mut()?;
        let base = charter.opens.get(&root)?;
        charter.destination_opened = true;
        if path == LEAF {
            assert_eq!(how, OpenHow::ReadNoFollow);
            return Some(match charter.target {
                Some(Target::File) => {
                    let fd = Fd::new(self.next_socket);
                    self.next_socket += 1;
                    self.files.insert(fd, UNCHANGED.to_vec());
                    Ok(Done::Fd(fd))
                }
                Some(Target::Link) => Err(Error::TooManyLinks),
                Some(Target::Raced) | None => Err(Error::NotFound),
            });
        }
        if matches!(how, OpenHow::Directory | OpenHow::DirectoryNoFollow) {
            let path = if path == b"." {
                base.clone()
            } else {
                assert_eq!(how, OpenHow::DirectoryNoFollow);
                child(base, path)
            };
            if charter.linked_directory == Some(path.as_slice()) {
                return Some(Err(Error::TooManyLinks));
            }
            if !charter.directories.contains(&path) {
                return Some(Err(Error::NotFound));
            }
            let fd = Fd::new(self.next_socket);
            self.next_socket += 1;
            charter.opens.insert(fd, path);
            return Some(Ok(Done::Fd(fd)));
        }
        None // The store's temporary file is handled by the main file fake.
    }

    pub(super) fn make_charter_directory(&mut self, dir: Fd, name: &[u8]) -> Result<Done, Error> {
        let charter = self.charter.as_mut().expect("only charter copy makes directories in this fixture");
        let base = charter.opens.get(&dir).expect("open destination parent");
        if charter.directories.insert(child(base, name)) { Ok(Done::Nothing) } else { Err(Error::Exists) }
    }

    pub(super) fn charter_rename(&mut self, dir: Fd, name: &[u8], no_replace: bool) -> Result<(), Error> {
        if let Some(charter) = &mut self.charter
            && let Some(path) = charter.opens.get(&dir)
        {
            assert_eq!(name, LEAF);
            assert!(no_replace, "publish the charter without replacing a name");
            if charter.target.is_some() {
                return Err(Error::Exists);
            }
            charter.placed_in = Some(path.clone());
        }
        Ok(())
    }
}

fn child(base: &[u8], name: &[u8]) -> Vec<u8> {
    if base.is_empty() { name.to_vec() } else { [base, b"/", name].concat() }
}
fn parts(values: &[Part]) -> List<w::CharterPart> {
    let mut parts = List::with_capacity(u32::try_from(values.len()).expect("part count"));
    for part in values {
        parts.push(w::CharterPart { on: part.on, instructions: part.words.into() }).expect("part room");
    }
    parts
}
fn charter_record(charter: &Charter) -> w::Charter {
    let mut parts = parts(charter.parts);
    if let Some(overlay) = &charter.overlay {
        // The fixture supplies composed instructions and the base's parts.
        let mut joined = List::uncapped();
        for part in parts.into_boxed() {
            joined.push(part).expect("base part");
        }
        for part in overlay.parts {
            joined.push(w::CharterPart { on: part.on, instructions: part.words.into() }).expect("overlay part");
        }
        parts = joined;
    }
    w::Charter {
        rest: false,
        number: NUMBER,
        instructions: charter.instructions.into(),
        waiting: Duration::from_secs(1),
        resume: false,
        turns: 10,
        time: Duration::from_secs(1),
        reserve_turns: 0,
        reserve_time: Duration::ZERO,
        model: model(),
        alternatives: List::with_capacity(0),
        inspect: false,
        modify: false,
        shell: false,
        agents: false,
        call_timeout: Duration::from_secs(1),
        tools: 0,
        hosts: w::SettingHosts::Both,
        spend: 100,
        wake: w::WakePolicy {
            words: w::WakeRule::Immediate,
            notices: w::WakeRule::Immediate,
            news: w::WakeRule::Immediate,
            results: w::ResultsWake::LastOrFailure,
            questions: false,
            answers: false,
            timers: false,
        },
        documents: List::with_capacity(0),
        parts,
        name: charter.name.map(Box::from),
    }
}
fn model() -> w::SettingModel {
    w::SettingModel {
        provider: 1,
        dialect: 1,
        endpoint: 1,
        program: None,
        name: b"model".as_slice().into(),
        window: 1024,
        output: 64,
        input_price: 0,
        cached_price: 0,
        cache_write_price: None,
        output_price: 0,
        price_unit: 1,
        efforts: List::with_capacity(0),
        effort: None,
        speeds: List::with_capacity(0),
        speed: None,
    }
}
fn overlay_record(overlay: &Overlay) -> w::Overlay {
    w::Overlay {
        rest: None,
        model: None,
        alternatives: None,
        inspect: None,
        modify: None,
        shell: None,
        agents: None,
        tools: None,
        hosts: None,
        waiting: None,
        resume: None,
        turns: None,
        time: None,
        reserve_turns: None,
        reserve_time: None,
        call_timeout: None,
        spend: None,
        documents: None,
        instructions: overlay.instructions.into(),
        parts: parts(overlay.parts),
    }
}
